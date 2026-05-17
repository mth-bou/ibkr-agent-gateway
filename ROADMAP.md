# Production Hardening Roadmap

Liste priorisée des améliorations identifiées lors de la review du code
métier du 2026-05-17, après la remédiation `LiveOrderWriter`. Ce document
sert de référence interne pour planifier les prochains sprints avant un
déploiement live réel.

Ce fichier **n'est pas publié sur crates.io** — il vit à la racine du repo
et n'est pas listé dans l'allowlist `include` de `Cargo.toml`.

## Convention

Chaque item porte une case GitHub task list. Quand tu attaques un item,
ouvre une branche `harden/<short-name>` et mets la case à `[~]` (en
cours). Une fois merge, passe à `[x]`.

## Sommaire et séquencement recommandé

Ordre d'attaque optimisé pour ratio risque/effort :

1. [~] **#1 Approval ↔ preview binding** — Critique sécurité, 2-3h
2. [~] **#7 Paper writer pluggable** — Cohérence avec live, ~1h
3. [ ] **#5 Crash safety WAL** — Stabilise le code qu'on vient de livrer, 4-6h
4. [ ] **#2 Policy registry server-side** — Sécurité tampering, 3-4h
5. [ ] **#6 Price collar + stale quote** — Defense-in-depth, 2-3h
6. [ ] **#3 MCP live tool handlers** — Débloque l'usage agent, 4-6h
7. [ ] **#4 Lifecycle reconciliation** — Sprint dédié, 1-2 jours
8. [ ] **#8 `audit verify` CLI**, **#9 CLI writer selector**, **#10 Compteurs rate centralisés** — Polish, 1-4h chacun

Les items 1, 7, 5 sont les pré-requis "code livrable en l'état" — c'est
là qu'il faut concentrer le premier effort.

---

## CRITIQUE — à corriger avant tout déploiement live réel

### #1. Lier l'approval au preview au moment du submit

**État** : [~] en cours
**Priorité** : Critique (sécurité, replay)
**Effort estimé** : 2-3h

**Lieu** :
- `src/internal/orders/live_submit.rs` (gate `approval_record`)
- `src/internal/orders/paper_submit.rs` (même gate côté paper)
- `src/internal/approval/model.rs` (`ApprovalRecord.preview_id` existe déjà)
- `src/cli/commands/approvals.rs:18` (génère un `OrderPreviewId::new()`
  aléatoire au lieu de prendre celui du preview cible)

**Description** : `ApprovalRecord` contient un champ `preview_id:
OrderPreviewId`, mais les gates de submit (live ET paper) ne vérifient
que `status + account_id + expiry`. Le `preview_id` n'est jamais comparé
à l'ordre soumis.

**Conséquence** : un agent MCP peut demander un preview pour un ordre A
(`buy 1 AAPL @100`), faire approuver, puis soumettre un ordre B
(`buy 100 AAPL @100`) sur le même compte avec la même approval. Tous
les gates passent.

**Fix proposé** :

1. Ajouter `--preview-id <UUID>` sur `approvals create` ; rejeter si
   le preview n'existe pas dans l'audit DB.
2. Faire vérifier au gate `approval_record` que
   `approval.preview_id == <preview source du validated_order>`. Ça
   suppose que `ValidatedOrder` porte une référence au preview source
   (à ajouter — actuellement il porte `validated_order_id` et `intent_id`
   uniquement).
3. Appliquer le status `Consumed` après usage : un submit qui réussit
   transitionne l'approval de `Approved` à `Consumed`. Le gate refuse
   tout status autre que `Approved`.

**Tests à ajouter** :
- approval avec preview_id différent → refus `APPROVAL_PREVIEW_MISMATCH`
- approval réutilisée après submit réussi → refus `APPROVAL_CONSUMED`
- approval expirée → refus existant (régression)

---

### #2. Policy registry server-side (anti-tampering)

**État** : [ ] non commencé
**Priorité** : Critique (sécurité, élévation de limites)
**Effort estimé** : 3-4h

**Lieu** :
- `src/internal/orders/live_submit.rs` (le check `risk_policy_pass`
  compare deux champs caller-supplied)
- `src/internal/risk/live_limits.rs` (`LiveLimitPolicy`)
- Nouveau module `src/internal/risk/policy_registry.rs`

**Description** : `submit_live_order` reçoit `live_limit_policy:
LiveLimitPolicy` **et** `live_config: LiveTradingConfig` dans la même
`LiveSubmitRequest`. Le check `policy_id == live_config.risk_policy_id`
n'empêche pas un caller hostile (agent MCP) de fournir un
`LiveLimitPolicy` permissif avec le bon `policy_id`.

**Conséquence** : dans le CLI smoke harness c'est sans risque (même
processus). Dans un serveur MCP exposé, un agent pourrait soumettre un
ordre `notional=1_000_000` avec une policy fournie `max_notional=∞`.

**Fix proposé** :

1. Créer un trait `LivePolicyRegistry: Send + Sync` avec
   `async fn load_policy(&self, policy_id: &str) -> Result<LiveLimitPolicy, GatewayError>`.
2. Implémenter `StaticPolicyRegistry` qui charge depuis la config statique
   au démarrage (TOML/YAML).
3. Modifier `submit_live_order` pour prendre `&dyn LivePolicyRegistry` et
   résoudre la policy server-side depuis `live_config.risk_policy_id`.
4. Retirer `live_limit_policy` du `LiveSubmitRequest` — c'est désormais
   server-side.

**Tests à ajouter** :
- caller fournit un `risk_policy_id` inconnu → refus `LIVE_POLICY_UNKNOWN`
- registry retourne une policy avec des limites strictes même si le
  caller en passait une laxiste (régression intentionnelle)

---

## HAUTE — production-readiness

### #3. Handlers MCP pour les outils live

**État** : [ ] non commencé
**Priorité** : Haute (dead surface)
**Effort estimé** : 4-6h

**Lieu** :
- `src/internal/mcp/tools/orders_live.rs` (schemas existent)
- `src/internal/mcp/registry.rs` ou `server.rs` (routing)

**Description** : `LIVE_ORDER_SUBMIT_TOOL` et `LIVE_ORDER_CANCEL_TOOL`
sont déclarés en tant que `ToolSchema`. **Aucun dispatcher** ne route
les appels MCP vers `submit_live_order` / `cancel_live_order`. Malgré
le `ClientPortalLiveWriter` livré, un agent LLM ne peut pas réellement
soumettre un ordre live via MCP — l'appel échoue au routing.

**Fix proposé** :

1. Implémenter `handle_live_submit(args, ctx) -> Result<Value, _>` qui :
   - parse `{account_id, approval_id, idempotency_key, preview_id}`
     depuis le payload MCP ;
   - charge l'approval depuis l'audit DB ;
   - charge la policy depuis `LivePolicyRegistry` (dépend de #2) ;
   - construit `LiveSubmitRequest` à partir d'état server-side ;
   - appelle `submit_live_order` avec le writer injecté ;
   - écrit l'audit event ;
   - retourne un `Value` redacted.
2. Symétrique pour `handle_live_cancel`.
3. Brancher dans le `McpRegistry` côté HTTP server (le stdio reste
   read-only par défaut).
4. Le wiring du writer dans le runtime MCP doit être server-side
   (lecture de la config backend, construction d'un
   `ClientPortalLiveWriter` ou refus si pas configuré).

**Tests à ajouter** : `tests/integration_mcp_live_submit.rs` avec
wiremock pour le broker + audit en mémoire.

---

### #4. Réconciliation du lifecycle après submit

**État** : [ ] non commencé
**Priorité** : Haute (state drift)
**Effort estimé** : 1-2 jours

**Lieu** :
- Nouveau module `src/internal/orders/reconciler.rs`
- `src/internal/backend/trait.rs::order_status` (déjà présent)
- Intégration runtime dans le serveur MCP / daemon

**Description** : `ClientPortalLiveWriter.submit_live` retourne un
broker order id et un status initial (`PreSubmitted` typiquement).
**Rien ne suit ensuite** l'évolution côté broker (Submitted →
PartiallyFilled → Filled / Rejected / Cancelled). `IbkrBackend::order_status`
n'est appelé que sur demande explicite par l'agent.

**Conséquence** : l'audit log enregistre `Submitted` à 10h00, mais l'état
réel chez IBKR peut être `Filled` ou `Rejected` depuis 10h30. Le kill
switch ne sert à rien si la gateway ne sait pas qu'un ordre s'est rempli
puis a généré une nouvelle position non-désirée.

**Fix proposé** :

1. Un reconciler tokio task qui, pour chaque ordre live `Submitted`
   non-terminal :
   - poll `order_status` toutes les N secondes (ou WebSocket si on
     ajoute le client `tokio-tungstenite` plus tard) ;
   - append un audit event à chaque transition d'état ;
   - met à jour la lifecycle store ;
   - retire l'ordre du backlog quand il atteint un état terminal
     (`Filled`, `Cancelled`, `Rejected`).
2. Stockage du backlog : table SQLite `live_orders_pending` avec
   `broker_order_id, account_id, last_status, last_polled_at`.
3. Au démarrage, repeupler le backlog depuis l'audit DB (scan des
   `live.submit.recorded` sans event terminal correspondant).
4. Configuration : `live_trading.reconciler_interval_seconds` (default 5).

**Tests à ajouter** : wiremock qui change la réponse `order_status` entre
deux polls, le reconciler doit émettre les bons audit events.

---

### #5. Crash safety entre writer call et audit write

**État** : [ ] non commencé
**Priorité** : Haute (phantom orders)
**Effort estimé** : 4-6h

**Lieu** :
- `src/cli/commands/orders_live.rs` (séquence submit → audit insert)
- Futur handler MCP live (même pattern)
- `src/internal/audit/sqlite.rs` (ajouter un état `Pending`)

**Description** : séquence actuelle :

1. `submit_live_order` valide les gates et appelle le writer.
2. Le broker accepte, retourne `order_id`.
3. Gateway crashe avant `audit_writer.insert_order_idempotency`.
4. Au redémarrage, l'audit DB ignore que cet ordre a été soumis. Un
   retry avec le même `idempotency_key` rappelle le writer.

Le `cOID` IBKR limite les dégâts (broker dedup retourne le même order
id), mais le gateway n'a pas de logique "récupérer l'ordre existant
pour ce cOID" à la reprise.

**Fix proposé** : write-ahead log idempotency.

1. Avant l'appel writer : `audit_writer.insert_order_pending(idempotency_key, request_hash, status="pending_writer")`.
2. Après réponse writer OK : update status à `submitted` + receipt.
3. Si réponse writer KO (timeout/erreur transport) : status `failed_after_writer`, ne pas marquer comme complet.
4. Au démarrage, scanner les `pending_writer` orphelins et appeler
   `order_status` avec le cOID (IBKR le restitue dans le payload) pour
   décider si l'ordre existe broker-side ou non.

**Tests à ajouter** :
- crash simulé entre writer et audit write → recovery scanne les
  pending et résout correctement
- retry avec même idem-key après crash → retourne le receipt du premier
  submit, ne rappelle pas le writer

---

## MOYENNE — defense-in-depth

### #6. Price collar et stale-quote refusal

**État** : [ ] non commencé
**Priorité** : Moyenne (defense vs typo)
**Effort estimé** : 2-3h

**Lieu** :
- `src/internal/risk/live_limits.rs` (`LiveLimitPolicy`)
- `src/internal/risk/checks.rs` (logique de refus)

**Description** : `LiveLimitPolicy` contrôle notional, quantity, symbol
allowlist, asset class, frequency, session. **Rien ne borne le prix
limite par rapport au marché**. Un buy `quantity=1,
limit_price=1000` au lieu de `100` passe tous les gates si le notional
reste sous le plafond. De même, aucune vérification de la fraîcheur du
market data (`market_snapshot.timestamp` > N secondes).

**Fix proposé** :

1. Ajouter à `LiveLimitPolicy` :
   - `max_price_deviation_bps: Option<u32>` (défaut 500 = 5 %)
   - `max_quote_age_seconds: Option<u32>` (défaut 30)
2. Le risk check récupère le market snapshot pour le `contract_id` et
   calcule `|limit_price - mid| / mid`. Refus si > seuil.
3. Si le snapshot a plus de `max_quote_age_seconds`, refus
   `LIVE_STALE_QUOTE`.

**Tests à ajouter** :
- limit price à +10 % vs mid → refus
- quote vieille de 60s avec seuil 30s → refus
- snapshot manquant → refus (pas de fallback silencieux)

---

### #7. Paper writer pluggable (cohérence avec live)

**État** : [~] en cours
**Priorité** : Moyenne (cohérence + validation pré-live)
**Effort estimé** : ~1h (miroir exact du refacto live)

**Lieu** :
- `src/internal/orders/paper_submit.rs:84` (`paper-order-local` synthétique)
- `src/internal/orders/paper_cancel.rs` (même)
- Nouveau `src/internal/orders/paper_writer.rs`
- `src/internal/cpapi/orders_write.rs` (déjà des types `ClientPortalPaperSubmitResponse`
  / `ClientPortalPaperCancelResponse` orphelins prêts à être utilisés)

**Description** : même antipattern que celui retiré du live. Paper
trading IBKR passe pourtant par le **même** Client Portal Gateway
(compte `DU*` au lieu de `U*`). Sans paper writer réel, le checklist
paper-to-live ne peut pas être validé end-to-end.

**Fix proposé** : refacto miroir.

1. `PaperOrderWriter` trait avec `submit_paper` / `cancel_paper`.
2. `LocalCandidatePaperWriter`, `RefusingPaperWriter`,
   `ClientPortalPaperWriter`.
3. Refacto `submit_paper_order` / `cancel_paper_order` en async,
   prennent le writer.
4. Update CLI + tests (déjà 4 tests d'intégration paper qui utilisent
   les fixtures `paper-order-local`).

**Tests à ajouter** : wiremock paper miroir du contract test live.

---

### #8. Commande `audit verify` autonome

**État** : [ ] non commencé
**Priorité** : Moyenne (ops)
**Effort estimé** : 1-2h

**Lieu** :
- `src/internal/audit/sqlite.rs` (logique de check chain déjà présente
  pour les reads, à factoriser)
- `src/cli/commands/audit.rs` (ajouter subcommand `verify`)

**Description** : la vérification de la chain HMAC se fait
**uniquement lors d'un read** (`tail`, `export`) — lignes 223-237 de
`sqlite.rs`. Pas de commande dédiée qui scanne l'intégralité et
retourne un code de sortie clair pour la supervision/monitoring.

**Fix proposé** :

1. Extraire `AuditChainVerifier::verify_all() -> ChainVerifyReport` qui
   parcourt sequence_id ASC et recalcule la chaîne.
2. Retourner `{events_scanned, chain_valid, first_break_at_sequence, first_break_event_id}`.
3. CLI `ibkr-agent audit verify --database-url ... --json` qui :
   - exit 0 si chain valide
   - exit 2 si rupture détectée
   - JSON output pour pipeline monitoring

**Tests à ajouter** : tampering simulé d'une ligne, verify détecte la
rupture au bon sequence_id.

---

## BASSE — polish, ergonomie déploiement

### #9. CLI writer selector

**État** : [ ] non commencé
**Priorité** : Basse
**Effort estimé** : 1h

**Lieu** : `src/cli/commands/orders_live.rs`

**Description** : le CLI est hardcodé sur `LocalCandidateLiveWriter`.
Pas de flag pour basculer vers `ClientPortalLiveWriter` avec un backend
CP configuré.

**Fix proposé** : flag CLI `--live-broker {local-candidate|client-portal|refusing}`
qui choisit l'impl selon le `BrokerBackendKind` configuré dans le
runtime. Par défaut : `local-candidate`.

---

### #10. Compteurs rate-limit centralisés

**État** : [ ] non commencé
**Priorité** : Basse (mais devient haute en multi-tenant)
**Effort estimé** : 3-4h

**Lieu** :
- `src/internal/risk/live_limits.rs` (`LiveLimitContext`)
- Nouveau `src/internal/risk/rate_counter.rs`

**Description** : `LiveLimitContext.submitted_in_window` et
`submitted_in_session` sont supplied par le caller. Le gateway fait
confiance — un caller multi-tenant peut tricher en passant 0.

**Fix proposé** :

1. `LiveRateCounterStore` adossé à l'audit DB.
2. Compteur calculé à partir des audit events
   `live.submit.recorded` filtrés par `account_id` et fenêtre temporelle.
3. Le caller ne fournit plus les compteurs ; ils sont calculés
   server-side au moment du gate.
4. Possibilité de cache court (1s) pour éviter la requête DB sur chaque
   submit.

**Tests à ajouter** : compteur correctement incrémenté entre submits
successifs, refus quand le seuil est atteint malgré un caller qui passe
0.

---

## Notes opérationnelles

- Tout fix qui touche un type public du SDK (`LiveSubmitRequest`,
  `LiveLimitPolicy`, etc.) est **breaking** et doit bumper la version
  `0.1.x → 0.2.0` ou attendre une fenêtre de breaking changes.
- Les fixes #1, #2, #5 nécessitent une coordination avec un éventuel
  consommateur SDK existant — annoncer dans le CHANGELOG sous
  "Breaking changes".
- Les fixes #3, #4 sont **additifs** : ils débloquent du nouveau usage
  sans casser l'existant.
- Le fix #7 est breaking côté API publique (signatures de
  `submit_paper_order` / `cancel_paper_order` deviennent async + prennent
  un writer) mais le pattern est strictement identique à #4-5-6 du
  refacto live, donc le risque de bug est faible.

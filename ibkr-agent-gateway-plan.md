# IBKR Agent Gateway — documentation projet et plan de développement

**Version :** 0.1  
**Date :** 2026-05-14  
**Nom de travail :** `ibkr-agent-gateway`  
**Objectif :** créer from scratch une gateway Rust qui expose Interactive Brokers à des clients LLM via MCP, avec auth OAuth/OIDC, scopes par tool, audit, risk engine déterministe et workflow d’approbation pour les ordres.

---

## 1. Résumé exécutif

L’idée n’est pas de faire “un bot de trading IA”. Mauvais pitch, mauvais design, explosion garantie.

Le bon pitch :

> **Une gateway Rust, provider-agnostic, qui expose IBKR comme tools MCP typés, sécurisés, auditables et contrôlés par des policies déterministes.**

Le LLM sert à lire, résumer, préparer, expliquer et orchestrer. Le code Rust garde le contrôle sur l’auth, les permissions, les montants, les instruments, les sessions, les confirmations et l’exécution.

Le projet doit être structuré en couches séparées :

```txt
LLM clients
  OpenAI / ChatGPT / Claude / Cursor / Continue / custom agents
        │
        ▼
MCP server Rust
  tools typés, OAuth/OIDC, scopes, audit
        │
        ▼
Agent domain layer
  intents, validated orders, approvals, risk policies
        │
        ▼
IBKR adapters
  Client Portal Gateway / Web API OAuth / éventuellement TWS plus tard
        │
        ▼
Interactive Brokers
```

Principe central : **le LLM ne parle jamais directement à IBKR**.

---

## 2. Contexte technique vérifié

### 2.1 IBKR côté Web API

IBKR expose une **Client Portal Web API** donnant accès à du trading, market data, scanners, portfolio intra-day, endpoints HTTP synchrones et WebSocket asynchrone. IBKR mentionne plusieurs méthodes d’authentification, dont OAuth 1.0a, OAuth 2.0, SSO et le Java-based Client Portal Gateway.[^ibkr-cpapi]

Pour les comptes retail/individuels, l’accès Web API est encore géré via le **Client Portal Gateway**, un programme Java local qui route les requêtes avec l’authentification appropriée.[^ibkr-retail-gateway]

IBKR indique aussi qu’il n’y a actuellement pas de mécanisme côté IBKR permettant aux clients individuels d’automatiser l’authentification de la brokerage session via Client Portal API.[^ibkr-no-auto-auth]

IBKR est en train d’unifier ses produits web API dans une Web API commune avec OAuth 2.0 comme moyen partagé d’autorisation/authentification ; la doc indique aussi que certains éléments de référence restent en beta ou incomplets.[^ibkr-unified]

### 2.2 MCP côté LLM

MCP définit un modèle où un MCP server peut agir comme **OAuth 2.1 resource server**, pendant que le client MCP agit comme OAuth client et utilise des access tokens.[^mcp-auth]

OpenAI supporte les remote MCP servers dans la Responses API avec transport Streamable HTTP ou HTTP/SSE.[^openai-mcp] Pour les apps authentifiées, OpenAI indique qu’un MCP server authentifié doit implémenter un flow OAuth 2.1 conforme à la spec MCP.[^openai-auth]

Anthropic expose aussi un MCP connector permettant à Claude de se connecter à des serveurs MCP distants, avec support de bearer tokens OAuth pour serveurs authentifiés.[^anthropic-mcp]

Il existe un SDK Rust officiel MCP, `rmcp`, basé sur Tokio.[^rmcp]

---

## 3. Objectifs produit

### 3.1 Objectifs

- Fournir une **base Rust from scratch** maîtrisée de bout en bout.
- Exposer IBKR via **MCP tools typés**.
- Supporter OpenAI, ChatGPT, Anthropic Claude, Cursor, Continue et agents custom sans couplage fort à un provider.
- Être **read-only par défaut**.
- Ajouter progressivement les ordres via un flow strict : `intent -> preview -> risk validation -> approval -> submit`.
- Supporter OAuth/OIDC côté front-door MCP.
- Supporter plusieurs modes backend IBKR : Client Portal Gateway local, puis OAuth2 direct quand disponible/adapté.
- Fournir un audit log exploitable.
- Empêcher les actions ambiguës : ticker non résolu, devise manquante, ordre sans limite, session incertaine, account non sélectionné, etc.

### 3.2 Non-objectifs initiaux

- Pas de stratégie de trading automatique dans le MVP.
- Pas de prédiction de marché.
- Pas de copy/paste du code officiel IBKR.
- Pas d’exécution d’ordre depuis une phrase libre du genre `buy NVDA for me`.
- Pas de support multi-broker au départ.
- Pas de haute fréquence.
- Pas d’optimisation fiscale/comptable avancée au MVP.

---

## 4. Design principles

1. **Provider-agnostic first**  
   MCP est l’interface principale. OpenAI/Anthropic deviennent des clients, pas des dépendances du core.

2. **Deterministic finance layer**  
   Tout ce qui touche aux ordres, comptes, positions, risk checks et validations doit être déterministe, testé, typé.

3. **No prompt-to-trade**  
   Jamais de `place_order(prompt: String)`. Le modèle produit des structures strictes, pas des actions directes.

4. **Read-only by default**  
   Les write tools sont désactivés sauf opt-in explicite au runtime et scopes OAuth dédiés.

5. **Preview before submit**  
   Un ordre soumis doit dériver d’un `OrderPreview` validé, encore frais, non expiré, et approuvé.

6. **Scopes partout**  
   Chaque tool MCP a un scope minimal. Exemple : `ibkr:orders:preview` ne donne pas `ibkr:orders:submit`.

7. **Audit ou rien**  
   Toute action significative produit un audit event : input canonique, utilisateur, scope, décision, hash, résultat.

8. **Fail closed**  
   Ambiguïté = refus technique. Pas d’héroïsme.

9. **Local-first, remote-capable**  
   Le retail IBKR impose souvent un Client Portal Gateway local. Le design doit donc gérer un mode local et un mode remote avec sidecar.

---

## 5. Topologies de déploiement

### 5.1 Topologie A — local single-user

Pour Claude Desktop, Cursor, Continue, agents locaux.

```txt
Local LLM client
    │ MCP stdio / localhost HTTP
    ▼
ibkr-agent-gateway local
    │ HTTP/WebSocket localhost
    ▼
IBKR Client Portal Gateway local
    │
    ▼
IBKR
```

Avantages : simple, rapide, bon MVP.  
Limites : pas adapté à ChatGPT Apps/Responses si le serveur MCP doit être accessible depuis l’extérieur.

### 5.2 Topologie B — remote MCP avec sidecar local

Pour ChatGPT, OpenAI Responses API, Anthropic Messages API, ou toute intégration remote HTTPS.

```txt
OpenAI / Claude / remote MCP client
    │ HTTPS + OAuth
    ▼
ibkr-agent-server public
    │ multiplexing / relay
    ▼
ibkr-agentd local sidecar
    │ localhost
    ▼
IBKR Client Portal Gateway local
    │
    ▼
IBKR
```

Le sidecar local ouvre une connexion sortante persistante vers le serveur public : WebSocket mTLS, token court ou device-bound credential. Le serveur public ne doit pas nécessiter d’accès entrant vers la machine utilisateur.

### 5.3 Topologie C — remote direct OAuth2 IBKR

Pour institutions, third-party, ou futur retail si IBKR le permet proprement.

```txt
OpenAI / Claude / remote MCP client
    │ HTTPS + OAuth utilisateur
    ▼
ibkr-agent-server public
    │ IBKR OAuth2 token
    ▼
IBKR Web API
```

C’est l’architecture la plus propre, mais elle dépend de la disponibilité réelle du mode OAuth2 pour la cible utilisateur.

---

## 6. Architecture workspace Rust

Structure recommandée :

```txt
ibkr-agent-gateway/
  Cargo.toml
  crates/
    ibkr-domain/
    ibkr-cpapi/
    ibkr-backend/
    ibkr-risk/
    ibkr-audit/
    ibkr-oauth/
    ibkr-mcp/
    ibkr-llm-core/
    ibkr-llm-openai/
    ibkr-llm-anthropic/
    ibkr-cli/
    ibkr-agentd/
    ibkr-server/
  docs/
    architecture.md
    tools.md
    security.md
    order-flow.md
    deployment.md
  tests/
    fixtures/
    contract-tests/
    e2e/
```

### 6.1 `ibkr-domain`

Types métier purs, zéro HTTP, zéro MCP, zéro OAuth.

Responsabilités :

- `AccountId`
- `ContractId`
- `Symbol`
- `Currency`
- `Exchange`
- `Position`
- `PortfolioSnapshot`
- `OrderIntent`
- `ValidatedOrder`
- `OrderPreview`
- `ConfirmedOrder`
- `OrderReceipt`
- `OrderStatus`
- `RiskWarning`
- `Money`
- `Quantity`
- `Notional`

Dépendances : `serde`, `rust_decimal`, `uuid`, `time`, `thiserror`.

### 6.2 `ibkr-cpapi`

Client bas niveau pour Client Portal Web API.

Responsabilités :

- HTTP client IBKR.
- WebSocket client IBKR.
- Session status.
- Keepalive/tickle.
- Account listing.
- Portfolio.
- Positions.
- Contract search.
- Market data snapshot.
- Orders read.
- Order preview/reply/submit quand activé.
- Mapping d’erreurs IBKR vers erreurs internes.

Dépendances probables : `reqwest`, `tokio`, `tokio-tungstenite`, `serde`, `url`, `tracing`.

Important : garder cette crate **non-agentique**. Elle ne sait pas ce qu’est un LLM.

### 6.3 `ibkr-backend`

Abstraction entre plusieurs backends IBKR.

```rust
#[async_trait::async_trait]
pub trait IbkrBackend: Send + Sync {
    async fn health(&self) -> Result<BackendHealth>;
    async fn list_accounts(&self) -> Result<Vec<Account>>;
    async fn positions(&self, account: &AccountId) -> Result<Vec<Position>>;
    async fn account_summary(&self, account: &AccountId) -> Result<AccountSummary>;
    async fn search_contracts(&self, query: ContractQuery) -> Result<Vec<ContractCandidate>>;
    async fn market_snapshot(&self, contract: ContractId) -> Result<MarketSnapshot>;
    async fn preview_order(&self, order: ValidatedOrder) -> Result<OrderPreview>;
    async fn submit_order(&self, order: ConfirmedOrder) -> Result<OrderReceipt>;
    async fn cancel_order(&self, order_id: BrokerOrderId) -> Result<CancelReceipt>;
}
```

Implémentations possibles :

```txt
CpGatewayBackend
IbkrOAuthBackend
MockBackend
RecordedBackend
```

### 6.4 `ibkr-risk`

Moteur de policies déterministes.

Responsabilités :

- Validation d’un `OrderIntent`.
- Résolution contractuelle unique.
- Contrôle de notional.
- Contrôle de quantité.
- Contrôle de devise.
- Contrôle de type d’ordre.
- Contrôle market hours / stale quote.
- Contrôle account mode : paper/live.
- Contrôle allowlist/denylist instrument.
- Contrôle concentration.
- Contrôle leverage/margin si données disponibles.
- Production de warnings et décisions.

Sorties :

```rust
pub enum RiskDecision {
    Allow { warnings: Vec<RiskWarning> },
    RequireApproval { warnings: Vec<RiskWarning>, level: ApprovalLevel },
    Deny { reason: DenyReason },
}
```

### 6.5 `ibkr-audit`

Journal d’audit append-only.

Responsabilités :

- Log des tool calls MCP.
- Log des previews.
- Log des risk decisions.
- Log des approvals.
- Log des submits/cancels.
- Hash de payload canonique.
- Corrélation par `request_id`, `user_id`, `session_id`, `approval_id`.

Storage :

- MVP : SQLite.
- Production : Postgres.
- Export : JSONL.

### 6.6 `ibkr-oauth`

Front-door auth pour le serveur MCP.

Responsabilités :

- Validation JWT via JWKS.
- Support opaque token via introspection si nécessaire.
- Mapping scopes -> permissions internes.
- Resource metadata MCP.
- Compatibilité OAuth 2.1/OIDC.
- Multi-tenant optionnel.

IdP recommandés pour démarrer : Keycloak, Zitadel, Auth0, Cognito, Dex.

Ne pas construire un serveur OAuth complet au MVP. Utiliser un IdP externe, sauf raison forte.

### 6.7 `ibkr-mcp`

Serveur MCP.

Responsabilités :

- Transport `stdio` pour local.
- Transport HTTP Streamable / SSE pour remote.
- Tool registry.
- JSON schema strict.
- Scope checks.
- Tool call audit.
- Mapping erreur interne -> résultat MCP propre.

Dépendance naturelle : `rmcp`.

### 6.8 `ibkr-llm-core`, `ibkr-llm-openai`, `ibkr-llm-anthropic`

Optionnel, à développer après MCP.

But : fournir des clients directs pour ceux qui veulent orchestrer hors MCP.

À ne pas mettre dans le core IBKR.

```rust
#[async_trait::async_trait]
pub trait LlmProvider {
    async fn run_with_tools(&self, request: AgentRequest) -> Result<AgentResponse>;
}
```

Ces crates doivent rester secondaires.

### 6.9 `ibkr-cli`

CLI pour dev, debug et administration.

Commandes envisagées :

```bash
ibkr-agent health
ibkr-agent accounts list
ibkr-agent positions list --account U1234567
ibkr-agent contracts search AAPL
ibkr-agent market snapshot --conid 265598
ibkr-agent orders list --account U1234567
ibkr-agent orders preview --account U1234567 --symbol AAPL --side sell --qty 10 --limit 200
ibkr-agent mcp serve --transport stdio
ibkr-agent mcp serve --transport http --auth oidc
ibkr-agent audit tail
```

### 6.10 `ibkr-agentd`

Daemon local pour la topologie remote + retail CP Gateway.

Responsabilités :

- Se connecter au CP Gateway local.
- Maintenir la session.
- Ouvrir une connexion sortante vers `ibkr-server`.
- Relayer les requêtes backend autorisées.
- Faire du mTLS ou token device-bound.
- Exposer un health local.

### 6.11 `ibkr-server`

Binaire serveur public.

Responsabilités :

- MCP HTTPS.
- OAuth/OIDC validation.
- Routage vers backend direct ou sidecar.
- Audit centralisé.
- Config multi-user.
- Metrics.
- Rate limiting.

---

## 7. Modèle d’authentification

### 7.1 Deux auths différentes

Il faut séparer deux mondes :

```txt
Auth utilisateur vers ton MCP server
  OAuth/OIDC, scopes, token validation, user_id

Auth backend vers IBKR
  Client Portal Gateway local, OAuth2 IBKR, SSO, etc.
```

Erreur à éviter : mélanger “OAuth OpenAI/Anthropic” et “OAuth IBKR”. OpenAI/Anthropic ne doivent pas posséder les secrets IBKR. Ils appellent ton serveur MCP avec un token qui représente l’utilisateur et ses scopes.

### 7.2 Scopes MCP

Scopes proposés :

```txt
ibkr:health:read
ibkr:accounts:read
ibkr:portfolio:read
ibkr:positions:read
ibkr:marketdata:read
ibkr:orders:read
ibkr:orders:preview
ibkr:orders:submit
ibkr:orders:cancel
ibkr:audit:read
ibkr:admin
```

Mapping direct :

| Scope | Donne accès à |
|---|---|
| `ibkr:health:read` | health, session status masqué |
| `ibkr:accounts:read` | list accounts, account metadata non sensible |
| `ibkr:portfolio:read` | portfolio summary |
| `ibkr:positions:read` | positions détaillées |
| `ibkr:marketdata:read` | quotes, snapshots, historical bars |
| `ibkr:orders:read` | open orders, order status, executions |
| `ibkr:orders:preview` | préparation d’ordre, jamais submit |
| `ibkr:orders:submit` | soumission après approval |
| `ibkr:orders:cancel` | annulation d’ordre |
| `ibkr:audit:read` | lecture des logs d’audit |
| `ibkr:admin` | config, policy, users, dangerous operations |

### 7.3 Token validation

Modes :

```txt
JWT mode
  issuer check
  audience check
  exp/nbf/iat check
  jwks signature check
  scopes claim

Opaque token mode
  introspection endpoint
  cache court
  scopes retournés par IdP
```

### 7.4 Metadata OAuth/MCP

Le serveur HTTP doit exposer au minimum :

```txt
/.well-known/oauth-protected-resource
/.well-known/openid-configuration     # côté IdP si self-hosted ou proxied
```

Ou déléguer la discovery à l’IdP externe.

---

## 8. Catalogue initial de tools MCP

Convention de nommage : `ibkr_<domain>_<action>`.

### 8.1 Tools P0 — health/session

| Tool | Scope | Input | Output | Notes |
|---|---|---|---|---|
| `ibkr_health` | `ibkr:health:read` | `{}` | gateway status | Ne retourne pas de secrets |
| `ibkr_backend_status` | `ibkr:health:read` | `{}` | CP Gateway / OAuth status | Masquer détails sensibles |
| `ibkr_session_requirements` | `ibkr:health:read` | `{}` | actions manuelles nécessaires | Utile si CP Gateway non connecté |

### 8.2 Tools P1 — comptes/portfolio read-only

| Tool | Scope | Input | Output |
|---|---|---|---|
| `ibkr_accounts_list` | `ibkr:accounts:read` | `{}` | accounts disponibles |
| `ibkr_account_summary` | `ibkr:portfolio:read` | `{ account_id }` | equity, cash, margin, currency |
| `ibkr_positions_list` | `ibkr:positions:read` | `{ account_id }` | positions détaillées |
| `ibkr_portfolio_snapshot` | `ibkr:portfolio:read` | `{ account_id }` | portfolio + allocations |

### 8.3 Tools P1 — contrats/market data

| Tool | Scope | Input | Output |
|---|---|---|---|
| `ibkr_contracts_search` | `ibkr:marketdata:read` | `{ query, asset_class?, currency?, exchange? }` | candidates |
| `ibkr_contract_resolve` | `ibkr:marketdata:read` | `{ symbol, asset_class, currency, exchange? }` | contract unique ou refus |
| `ibkr_market_snapshot` | `ibkr:marketdata:read` | `{ contract_id }` | bid/ask/last, timestamp |
| `ibkr_historical_bars` | `ibkr:marketdata:read` | `{ contract_id, duration, bar_size }` | bars |

### 8.4 Tools P2 — ordres read-only

| Tool | Scope | Input | Output |
|---|---|---|---|
| `ibkr_orders_list` | `ibkr:orders:read` | `{ account_id, status? }` | open/recent orders |
| `ibkr_order_status` | `ibkr:orders:read` | `{ account_id, broker_order_id }` | state |
| `ibkr_executions_list` | `ibkr:orders:read` | `{ account_id, from?, to? }` | executions |

### 8.5 Tools P3 — preview d’ordre

| Tool | Scope | Input | Output |
|---|---|---|---|
| `ibkr_order_intent_validate` | `ibkr:orders:preview` | `OrderIntent` | `RiskDecision` |
| `ibkr_order_preview` | `ibkr:orders:preview` | `OrderIntent` | `OrderPreview` |
| `ibkr_order_preview_explain` | `ibkr:orders:preview` | `{ preview_id }` | explication structurée |

`OrderIntent` doit être strict :

```json
{
  "account_id": "U1234567",
  "symbol": "AAPL",
  "asset_class": "stock",
  "side": "sell",
  "quantity": "10",
  "order_type": "limit",
  "limit_price": "200.00",
  "currency": "USD",
  "time_in_force": "DAY",
  "rationale": "Reduce concentrated exposure"
}
```

### 8.6 Tools P4 — submit/cancel

| Tool | Scope | Input | Output |
|---|---|---|---|
| `ibkr_order_submit` | `ibkr:orders:submit` | `{ preview_id, approval_token, idempotency_key }` | `OrderReceipt` |
| `ibkr_order_cancel` | `ibkr:orders:cancel` | `{ broker_order_id, reason, idempotency_key }` | `CancelReceipt` |

Contraintes :

- `submit` refuse tout ordre qui ne vient pas d’un preview valide.
- `preview_id` expire vite.
- `approval_token` doit être généré hors LLM ou via un flow utilisateur explicite.
- `idempotency_key` obligatoire.

---

## 9. Order lifecycle

State machine cible :

```txt
DraftIntent
    │ validate
    ▼
ValidatedIntent
    │ risk check
    ├── Denied
    ├── RequiresApproval
    ▼
PreviewedOrder
    │ approve
    ▼
ConfirmedOrder
    │ submit
    ▼
SubmittedOrder
    │ broker events
    ├── PreSubmitted
    ├── Submitted
    ├── PartiallyFilled
    ├── Filled
    ├── CancelRequested
    ├── Cancelled
    ├── Rejected
    └── Inactive
```

Types Rust :

```rust
pub struct OrderIntent {
    pub account_id: AccountId,
    pub contract_hint: ContractHint,
    pub side: Side,
    pub quantity: Decimal,
    pub order_type: OrderType,
    pub limit_price: Option<Money>,
    pub time_in_force: TimeInForce,
    pub rationale: Option<String>,
}

pub struct OrderPreview {
    pub preview_id: PreviewId,
    pub expires_at: OffsetDateTime,
    pub resolved_contract: Contract,
    pub normalized_order: BrokerOrderDraft,
    pub estimated_notional: Money,
    pub risk_decision: RiskDecision,
    pub warnings: Vec<RiskWarning>,
    pub audit_hash: String,
}

pub struct ConfirmedOrder {
    pub preview_id: PreviewId,
    pub approval_id: ApprovalId,
    pub idempotency_key: IdempotencyKey,
}
```

---

## 10. Risk engine

### 10.1 Policies MVP

| Policy | Défaut | Raisonnement engineering |
|---|---:|---|
| Market order | refusé | Trop facile à exécuter à un prix surprise |
| Limit order | autorisé | Prix borné |
| Max notional par ordre | configurable | Évite les erreurs de quantité/devise |
| Max quantity par symbole | configurable | Évite `1000` au lieu de `10` |
| Ambiguous contract | refusé | Pas de guess sur le marché/instrument |
| Stale quote | warning ou refus | Preview fiable seulement si données récentes |
| Unknown currency conversion | warning ou refus | Notional trompeur sinon |
| Live trading | désactivé par défaut | Paper d’abord |
| Options/derivatives | désactivé au MVP | Complexité + risques de modèle |
| Short sell | désactivé par défaut | Besoin de checks borrow/margin |
| Leverage increase | approval fort | À traiter explicitement |

### 10.2 Exemple config risk

```yaml
risk:
  default_account_mode: paper
  allow_live_orders: false
  allowed_asset_classes: [stock, etf]
  denied_symbols: []
  max_order_notional:
    USD: "5000"
    EUR: "5000"
  require_limit_orders: true
  market_orders_allowed: false
  max_preview_age_seconds: 120
  require_fresh_quote_seconds: 30
  allow_short_selling: false
  allow_options: false
  require_human_approval_for_submit: true
```

### 10.3 Décisions

```rust
pub enum DenyReason {
    MissingAccount,
    AmbiguousContract,
    UnsupportedAssetClass,
    MarketOrderDisabled,
    ExceedsMaxNotional,
    StaleMarketData,
    LiveTradingDisabled,
    MissingApproval,
    ExpiredPreview,
    BackendSessionUnavailable,
}
```

---

## 11. Approval workflow

### 11.1 MVP local

```txt
LLM calls ibkr_order_preview
    ▼
Gateway returns preview_id + warnings + required approval
    ▼
User approves via CLI/TUI/browser local
    ▼
Gateway creates approval_token
    ▼
LLM or app calls ibkr_order_submit(preview_id, approval_token, idempotency_key)
```

### 11.2 Remote

```txt
MCP client requests preview
    ▼
Gateway stores preview
    ▼
User sees approval UI / CLI / mobile webhook
    ▼
Approval token minted server-side
    ▼
Submit tool can use approved preview
```

Approval token properties :

- lié à `preview_id`
- lié à `user_id`
- lié à `account_id`
- lié au hash canonique de l’ordre
- TTL court
- usage unique

---

## 12. Audit log

### 12.1 Événements minimum

```txt
tool.called
tool.denied_scope
tool.completed
tool.failed
risk.evaluated
order.preview.created
order.preview.expired
order.approval.created
order.submit.requested
order.submit.completed
order.cancel.requested
order.cancel.completed
backend.session.changed
```

### 12.2 Schéma audit event

```json
{
  "event_id": "018f...",
  "event_type": "order.preview.created",
  "timestamp": "2026-05-14T10:15:30Z",
  "user_id": "user_123",
  "tenant_id": "tenant_abc",
  "account_id": "U1234567",
  "tool_name": "ibkr_order_preview",
  "scopes": ["ibkr:orders:preview"],
  "request_id": "req_456",
  "idempotency_key": null,
  "input_hash": "sha256:...",
  "output_hash": "sha256:...",
  "decision": "require_approval",
  "redactions": ["account_id_partial"],
  "metadata": {
    "asset_class": "stock",
    "symbol": "AAPL",
    "side": "sell"
  }
}
```

Ne pas stocker de secrets. Redacter tokens, cookies, credentials, headers sensibles.

---

## 13. Sécurité engineering

### 13.1 Règles de base

- Le modèle est un client non-fiable.
- Les outputs de tools reviennent dans le contexte du modèle : ne jamais y mettre de secret.
- Toute donnée de marché, news, nom d’instrument, note ou champ texte peut contenir du contenu hostile ou juste débile ; ne jamais l’interpréter comme instruction système.
- Toute action write doit passer par scopes + policy + audit + idempotency.
- Les erreurs ne doivent pas fuiter tokens, cookies, chemins locaux ou configs internes.

### 13.2 Protections concrètes

| Risque | Mitigation |
|---|---|
| Tool call sans scope | middleware scope obligatoire |
| Token volé | TTL court, audience check, issuer check, rotation |
| Confusion compte live/paper | champ obligatoire + config + confirmation |
| Ambiguïté ticker | résolution contractuelle unique obligatoire |
| Double submit | idempotency key + state persisted |
| Preview périmé | expiration stricte |
| LLM hallucine un ordre | schema strict + risk denial |
| Prompt injection via données externes | pas d’instructions exécutables depuis data |
| Secret leak | redaction systématique |
| Sidecar compromis | scopes sidecar limités + mTLS + device binding |

---

## 14. Configuration

Exemple `config.yaml` :

```yaml
server:
  mode: local
  bind: "127.0.0.1:8080"
  public_base_url: "https://ibkr-agent.example.com"

mcp:
  transport: http
  enable_stdio: true
  enable_sse: true
  enable_streamable_http: true

frontend_auth:
  mode: oidc
  issuer: "https://idp.example.com/realms/ibkr-agent"
  audience: "ibkr-agent-gateway"
  jwks_url: "https://idp.example.com/realms/ibkr-agent/protocol/openid-connect/certs"

ibkr:
  backend: cp_gateway
  cp_gateway:
    base_url: "https://localhost:5000/v1/api"
    verify_tls: false
    keepalive_interval_seconds: 60
  oauth2:
    enabled: false

risk:
  allow_live_orders: false
  require_limit_orders: true
  max_order_notional:
    USD: "5000"
    EUR: "5000"
  max_preview_age_seconds: 120
  require_human_approval_for_submit: true

storage:
  driver: sqlite
  dsn: "sqlite://ibkr-agent.db"

audit:
  enabled: true
  export_jsonl: true
  redact_account_ids: false

observability:
  tracing: true
  metrics: true
  otel_endpoint: null
```

---

## 15. Observabilité

### 15.1 Logs structurés

Utiliser `tracing`.

Champs clés :

```txt
request_id
user_id
tenant_id
tool_name
scope
account_id_hash
backend
latency_ms
risk_decision
error_code
```

### 15.2 Metrics

```txt
mcp_tool_calls_total{tool, status}
mcp_tool_latency_ms{tool}
ibkr_backend_latency_ms{endpoint}
ibkr_session_status
risk_decisions_total{decision}
order_previews_total
order_submits_total
order_cancels_total
audit_events_total
sidecar_connections_active
```

### 15.3 Traces

Trace typique :

```txt
HTTP /mcp
  -> auth.validate_token
  -> mcp.parse_tool_call
  -> scope.check
  -> risk.evaluate
  -> ibkr.backend.preview_order
  -> audit.write
  -> mcp.return_result
```

---

## 16. Plan de développement

### Phase 0 — bootstrap repo

Objectif : squelette propre.

Features :

- Cargo workspace.
- Crates : `domain`, `cpapi`, `backend`, `risk`, `audit`, `cli`.
- CI GitHub Actions.
- Format/lint : `cargo fmt`, `cargo clippy`.
- Tests unitaires basiques.
- Config loader.
- Error types communs.

Acceptance criteria :

- `cargo test --workspace` vert.
- `cargo clippy --workspace --all-targets` vert.
- CLI `ibkr-agent --help` fonctionne.

### Phase 1 — IBKR CP Gateway read-only

Objectif : parler à IBKR sans MCP.

Features :

- Client HTTP CPAPI.
- Health/status backend.
- Session detection.
- Accounts list.
- Account summary.
- Positions list.
- Contract search.
- Market snapshot.
- Orders list read-only.
- CLI pour chaque action.
- Fixture tests avec fake server.

Acceptance criteria :

- Avec un CP Gateway connecté, `ibkr-agent accounts list` marche.
- Si session absente, erreur claire : action manuelle requise.
- Tests offline avec mock server.

### Phase 2 — MCP local read-only

Objectif : exposer les données à Claude Desktop/Cursor/agents locaux.

Features :

- Crate `ibkr-mcp`.
- Transport stdio.
- Transport localhost HTTP optionnel.
- Tools read-only : health, accounts, summary, positions, contracts, market snapshot, orders read.
- Schemas stricts.
- Audit log pour tool calls.

Acceptance criteria :

- MCP Inspector liste les tools.
- Un client MCP local peut lire portfolio/positions.
- Aucun write tool exposé.

### Phase 3 — OAuth/OIDC front-door

Objectif : serveur MCP remote sécurisé.

Features :

- `ibkr-oauth`.
- Validation JWT via JWKS.
- Scope middleware.
- Protected resource metadata.
- HTTP MCP avec auth.
- Tests token invalid/expired/wrong audience/missing scope.

Acceptance criteria :

- Un token sans scope est refusé.
- Un token expiré est refusé.
- Chaque tool déclare ses scopes.
- Logs/audit ne contiennent pas de token.

### Phase 4 — risk engine + order preview

Objectif : préparer des ordres sans soumission.

Features :

- `OrderIntent` strict.
- Contract resolution stricte.
- Risk policies MVP.
- Order preview.
- Preview expiration.
- Explain preview.
- Audit complet.

Acceptance criteria :

- Ticker ambigu refusé.
- Market order refusé par défaut.
- Notional excessif refusé.
- Preview produit un hash canonique et une expiration.
- Submit toujours impossible.

### Phase 5 — approval + submit paper-only

Objectif : soumettre des ordres paper avec garde-fous.

Features :

- Approval local CLI.
- Approval token usage unique.
- Idempotency keys.
- Submit order paper-only.
- Cancel order paper-only.
- Order status tracking.
- Replay audit.

Acceptance criteria :

- Aucun submit sans preview.
- Aucun submit sans approval.
- Double submit avec même idempotency key ne duplique pas l’ordre.
- Live account impossible sans flag explicite.

### Phase 6 — sidecar local + remote relay

Objectif : rendre le système compatible ChatGPT/Claude remote avec CP Gateway retail.

Features :

- `ibkr-agentd` local.
- Connexion outbound WebSocket vers serveur.
- Device registration.
- mTLS ou token device-bound.
- Backend routing vers sidecar.
- Health sidecar.
- Reconnect/backoff.

Acceptance criteria :

- Serveur remote peut interroger backend local via sidecar.
- Aucune ouverture inbound sur machine utilisateur.
- Sidecar déconnecté = tools refusés proprement.

### Phase 7 — OpenAI/Anthropic compatibility tests

Objectif : valider remote MCP avec providers.

Features :

- Exemple OpenAI Responses API avec remote MCP.
- Exemple Anthropic Messages API MCP connector.
- Documentation d’intégration.
- Tool allowlist provider.
- Tests manuels reproductibles.

Acceptance criteria :

- OpenAI peut lister les tools MCP remote.
- Anthropic peut lister les tools MCP remote.
- OAuth token accepté/refusé correctement.
- Read-only demo fonctionnelle.

### Phase 8 — live trading gated

Objectif : live trading contrôlé.

Features :

- `allow_live_orders` global false par défaut.
- Per-user live permission.
- Per-account live permission.
- Higher approval level.
- Hard limits live séparés.
- Emergency kill switch.

Acceptance criteria :

- Live impossible sans trois conditions : config globale + scope + account permission.
- Kill switch bloque preview/submit/cancel selon config.
- Audit renforcé.

---

## 17. Backlog features

### P0 — indispensable MVP

- Workspace Rust.
- Domain types.
- CP Gateway client read-only.
- CLI read-only.
- Fake CPAPI server.
- MCP stdio read-only.
- Audit SQLite.
- Config YAML.

### P1 — produit utilisable

- HTTP MCP.
- OIDC/JWT validation.
- Scopes.
- Market data snapshot.
- Contract resolution robuste.
- Positions + portfolio summaries.
- Docs d’installation.
- Dockerfile local.

### P2 — ordre preview

- `OrderIntent`.
- Risk engine.
- Preview.
- Explain preview.
- Preview TTL.
- Canonical hash.
- Audit order preview.

### P3 — paper trading

- Approval CLI.
- Submit paper.
- Cancel paper.
- Order status stream.
- Idempotency.
- Replay tests.

### P4 — remote gateway

- Public HTTP MCP.
- OAuth protected resource metadata.
- Sidecar local.
- WebSocket relay.
- Device registration.
- Multi-user minimal.

### P5 — production-grade

- Postgres.
- OpenTelemetry.
- Rate limiting.
- Admin UI minimal.
- Policy editor.
- Key rotation.
- Backup/restore audit.
- Deployment Helm/Kubernetes.

### P6 — advanced trading support

- Options read-only.
- Options preview gated.
- Multi-leg orders.
- FX conversion preview.
- Rebalancing assistant preview-only.
- Tax lots read-only si disponible.
- Journaling trades.

---

## 18. Testing strategy

### 18.1 Unit tests

- Domain parsing.
- Money/quantity precision.
- Risk policies.
- Scope mapping.
- OAuth token validation.
- Audit redaction.

### 18.2 Integration tests offline

- Fake CPAPI server avec `wiremock` ou `axum` test server.
- Fixtures JSON IBKR.
- WebSocket fake stream.
- Error mapping.
- Session missing.

### 18.3 MCP tests

- MCP Inspector.
- Tool list snapshots.
- Schema snapshots.
- Missing scope.
- Invalid input.
- Tool error mapping.

### 18.4 E2E paper

- CP Gateway lancé manuellement.
- Paper account.
- Read-only flow.
- Preview flow.
- Paper submit flow après approval.
- Cancel flow.

### 18.5 Security tests

- Token expired.
- Wrong issuer.
- Wrong audience.
- Missing scope.
- Scope escalation attempt.
- Reused approval token.
- Reused idempotency key.
- Redaction tokens.
- Prompt injection style payload dans un champ texte.

### 18.6 Replay tests

- Enregistrer des sessions read-only.
- Rejouer fixtures.
- Vérifier stabilité des mappings.
- Vérifier hash canonique inchangé.

---

## 19. Dépendances Rust probables

À figer après spike, mais base raisonnable :

```txt
tokio
reqwest
tokio-tungstenite
serde
serde_json
thiserror
anyhow
tracing
tracing-subscriber
opentelemetry
axum
tower
clap
config
rust_decimal
time
uuid
url
secrecy
zeroize
jsonwebtoken
sqlx
rmcp
async-trait
schemars
```

`schemars` est utile pour générer/valider les JSON schemas de tools.

---

## 20. Format d’erreurs

Erreur interne :

```rust
pub struct AgentError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    pub user_action: Option<UserAction>,
    pub audit_event_id: Option<AuditEventId>,
}
```

Exemples de codes :

```txt
AUTH_MISSING_TOKEN
AUTH_INVALID_TOKEN
AUTH_MISSING_SCOPE
IBKR_SESSION_REQUIRED
IBKR_BACKEND_UNAVAILABLE
CONTRACT_AMBIGUOUS
MARKET_DATA_UNAVAILABLE
RISK_DENIED
PREVIEW_EXPIRED
APPROVAL_REQUIRED
APPROVAL_INVALID
ORDER_SUBMIT_FAILED
```

---

## 21. Documentation à écrire dans le repo

```txt
docs/
  getting-started-local.md
  ibkr-client-portal-gateway.md
  mcp-local.md
  mcp-remote-oauth.md
  tools.md
  scopes.md
  risk-policies.md
  order-preview-submit-flow.md
  audit-log.md
  sidecar-remote-relay.md
  deployment.md
  provider-openai.md
  provider-anthropic.md
  testing.md
```

---

## 22. Licence et propriété intellectuelle

Recommandation :

```txt
MIT OR Apache-2.0
```

Règles :

- Ne pas copier le code officiel IBKR.
- Ne pas traduire littéralement une implémentation officielle.
- S’appuyer sur la documentation, les specs publiques, les tests contre gateway/paper account, et fixtures propres.
- Documenter la compatibilité sans se présenter comme projet officiel IBKR.

---

## 23. Première séquence concrète de dev

### Sprint 1

```txt
1. Créer workspace Cargo.
2. Créer ibkr-domain.
3. Créer ibkr-cpapi avec client reqwest minimal.
4. Implémenter health/session status.
5. Implémenter accounts list.
6. Ajouter CLI health/accounts.
7. Ajouter fake server tests.
```

### Sprint 2

```txt
1. Positions list.
2. Account summary.
3. Contract search.
4. Market snapshot.
5. Error mapping.
6. Audit SQLite minimal.
```

### Sprint 3

```txt
1. Ajouter ibkr-mcp.
2. Exposer tools read-only en stdio.
3. Tester via MCP Inspector.
4. Documenter setup local.
```

### Sprint 4

```txt
1. HTTP MCP.
2. OIDC/JWT validation.
3. Scopes par tool.
4. Protected resource metadata.
5. Tests auth.
```

### Sprint 5

```txt
1. OrderIntent.
2. Risk engine MVP.
3. Order preview.
4. Preview expiration.
5. Audit preview.
```

---

## 24. Exemple API ergonomique interne

```rust
let backend = CpGatewayBackend::new(config.ibkr.cp_gateway)?;
let risk = RiskEngine::from_config(config.risk);
let audit = AuditLog::sqlite("sqlite://ibkr-agent.db").await?;

let intent = OrderIntent {
    account_id: AccountId::new("U1234567"),
    contract_hint: ContractHint::Stock {
        symbol: "AAPL".into(),
        currency: Currency::usd(),
        exchange: Some("SMART".into()),
    },
    side: Side::Sell,
    quantity: dec!(10),
    order_type: OrderType::Limit,
    limit_price: Some(Money::usd(dec!(200))),
    time_in_force: TimeInForce::Day,
    rationale: Some("Reduce concentration".into()),
};

let preview = OrderService::new(backend, risk, audit)
    .preview_order(intent)
    .await?;

println!("{preview:#?}");
```

---

## 25. Questions ouvertes

- Quelle cible initiale : local MCP uniquement, ou remote MCP dès le départ ?
- IdP choisi : Keycloak/Zitadel/Auth0/Cognito/Dex ?
- Storage : SQLite only au MVP ou Postgres dès le début ?
- Priorité : CPAPI Web uniquement, ou prévoir TWS adapter plus tard ?
- Support live trading : jamais, plus tard, ou gated très fort ?
- Sidecar : indispensable au produit cible ou feature v2 ?
- Nom final du projet/crate ?
- Support multi-user dès le départ ou single-user propre d’abord ?

---

## 26. Verdict architectural

Le projet est pertinent s’il assume ce positionnement :

```txt
Rust-first MCP/OAuth gateway for Interactive Brokers,
focused on typed tools, auditability, risk validation,
and controlled order workflows.
```

La valeur n’est pas dans “connecter ChatGPT à IBKR”. Ça, c’est le bricolage du dimanche.

La valeur est dans :

```txt
typed schemas
strict scopes
order preview
risk engine
approval tokens
idempotency
audit logs
sidecar retail support
provider-agnostic MCP
paper/live separation
```

Le meilleur MVP : **read-only local MCP + CP Gateway**, puis OAuth/remote, puis preview, puis paper submit.

---

## Références

[^ibkr-cpapi]: IBKR Campus — Client Portal Web API v1.0 Documentation. https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/
[^ibkr-retail-gateway]: IBKR Campus — Account Management Web API, retail authentication via Client Portal Gateway. https://www.interactivebrokers.com/campus/ibkr-api-page/web-api-account-management/
[^ibkr-no-auto-auth]: IBKR Campus — Client Portal API FAQ, gateway authentication automation. https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/
[^ibkr-unified]: IBKR Campus — Web API Documentation, unified Web API and OAuth 2.0. https://www.interactivebrokers.com/campus/ibkr-api-page/webapi-doc/
[^mcp-auth]: Model Context Protocol — Authorization spec. https://modelcontextprotocol.io/specification/draft/basic/authorization
[^rmcp]: modelcontextprotocol/rust-sdk — official Rust SDK for MCP. https://github.com/modelcontextprotocol/rust-sdk
[^openai-mcp]: OpenAI Developers — MCP and connectors in Responses API. https://developers.openai.com/api/docs/guides/tools-connectors-mcp
[^openai-auth]: OpenAI Developers — Apps SDK authentication, OAuth 2.1 for authenticated MCP servers. https://developers.openai.com/apps-sdk/build/auth
[^anthropic-mcp]: Anthropic Claude docs — MCP connector. https://platform.claude.com/docs/en/agents-and-tools/mcp-connector

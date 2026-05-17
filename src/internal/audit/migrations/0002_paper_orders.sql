CREATE TABLE IF NOT EXISTS approval_records (
    approval_id TEXT PRIMARY KEY,
    preview_id TEXT NOT NULL,
    account_id_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    approved_by TEXT NOT NULL,
    approved_at INTEGER,
    expires_at INTEGER NOT NULL,
    payload_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS order_preview_records (
    preview_id TEXT PRIMARY KEY,
    validated_order_id TEXT NOT NULL,
    account_id_hash TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    payload_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS order_idempotency_records (
    idempotency_key TEXT PRIMARY KEY,
    request_hash TEXT NOT NULL,
    result_hash TEXT,
    created_at INTEGER NOT NULL,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'submitted',
    updated_at INTEGER,
    failure_json TEXT
);

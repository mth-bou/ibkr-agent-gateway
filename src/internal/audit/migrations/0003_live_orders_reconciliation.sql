CREATE TABLE IF NOT EXISTS live_orders_pending (
    account_id TEXT NOT NULL,
    broker_order_id TEXT NOT NULL,
    last_status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_polled_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, broker_order_id)
);

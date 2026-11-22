CREATE TABLE btc_broadcast_inputs (
    outpoint TEXT PRIMARY KEY NOT NULL,
    txid TEXT NOT NULL,
    created_at BIGINT NOT NULL
);

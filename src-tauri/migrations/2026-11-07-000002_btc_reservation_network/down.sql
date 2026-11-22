CREATE TABLE btc_broadcast_inputs_old (
    outpoint TEXT PRIMARY KEY NOT NULL,
    txid TEXT NOT NULL,
    created_at BIGINT NOT NULL
);
INSERT OR IGNORE INTO btc_broadcast_inputs_old (outpoint, txid, created_at)
SELECT outpoint, txid, created_at FROM btc_broadcast_inputs;
DROP TABLE btc_broadcast_inputs;
ALTER TABLE btc_broadcast_inputs_old RENAME TO btc_broadcast_inputs;

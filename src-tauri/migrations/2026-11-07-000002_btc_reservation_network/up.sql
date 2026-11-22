ALTER TABLE btc_broadcast_inputs RENAME TO btc_broadcast_inputs_old;
CREATE TABLE btc_broadcast_inputs (
    network TEXT NOT NULL,
    outpoint TEXT NOT NULL,
    txid TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    PRIMARY KEY (network, outpoint)
);
INSERT INTO btc_broadcast_inputs (network, outpoint, txid, created_at)
SELECT 'bitcoin', outpoint, txid, created_at FROM btc_broadcast_inputs_old;
DROP TABLE btc_broadcast_inputs_old;

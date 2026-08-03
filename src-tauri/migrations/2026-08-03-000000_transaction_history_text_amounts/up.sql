CREATE TABLE transactions_new (
    tx_hash         TEXT    NOT NULL,
    wallet_name     TEXT    NOT NULL,
    chain           TEXT    NOT NULL,
    account_index   INTEGER NOT NULL,
    direction       SMALLINT NOT NULL,
    status          TEXT    NOT NULL,
    from_address    TEXT,
    to_address      TEXT,
    amount          TEXT    NOT NULL,
    fee             TEXT,
    block_height    BIGINT,
    chain_data      TEXT    NOT NULL,
    created_at      BIGINT  NOT NULL,
    confirmed_at    BIGINT,
    PRIMARY KEY (wallet_name, chain, tx_hash)
);

INSERT OR IGNORE INTO transactions_new (
    tx_hash,
    wallet_name,
    chain,
    account_index,
    direction,
    status,
    from_address,
    to_address,
    amount,
    fee,
    block_height,
    chain_data,
    created_at,
    confirmed_at
)
SELECT
    tx_hash,
    wallet_name,
    chain,
    account_index,
    direction,
    status,
    from_address,
    to_address,
    CAST(amount AS TEXT),
    CASE WHEN fee IS NULL THEN NULL ELSE CAST(fee AS TEXT) END,
    block_height,
    COALESCE(chain_data, '{}'),
    created_at,
    confirmed_at
FROM transactions;

DROP INDEX IF EXISTS idx_tx_lookup;
DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;

CREATE INDEX idx_tx_lookup ON transactions (
    wallet_name,
    chain,
    account_index,
    created_at DESC
);

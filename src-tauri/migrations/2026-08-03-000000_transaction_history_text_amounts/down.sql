CREATE TABLE transactions_old (
    tx_hash         TEXT PRIMARY KEY,
    wallet_name     TEXT    NOT NULL,
    chain           TEXT    NOT NULL,
    account_index   INTEGER NOT NULL,
    direction       SMALLINT NOT NULL,
    status          TEXT    NOT NULL,
    from_address    TEXT,
    to_address      TEXT,
    amount          BIGINT  NOT NULL,
    fee             INTEGER,
    block_height    BIGINT,
    chain_data      TEXT,
    created_at      BIGINT  NOT NULL,
    confirmed_at    BIGINT
);

INSERT OR IGNORE INTO transactions_old (
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
    CAST(amount AS BIGINT),
    CASE WHEN fee IS NULL THEN NULL ELSE CAST(fee AS INTEGER) END,
    block_height,
    chain_data,
    created_at,
    confirmed_at
FROM transactions;

DROP INDEX IF EXISTS idx_tx_lookup;
DROP TABLE transactions;
ALTER TABLE transactions_old RENAME TO transactions;

CREATE INDEX idx_tx_lookup ON transactions (
    wallet_name,
    chain,
    account_index,
    created_at DESC
);

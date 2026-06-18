CREATE TABLE transactions (
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
    chain_data      OBJECT,
    created_at      BIGINT  NOT NULL,
    confirmed_at    BIGINT
);

CREATE INDEX idx_tx_lookup ON transactions (
    wallet_name,
    chain,
    account_index,
    created_at DESC
);

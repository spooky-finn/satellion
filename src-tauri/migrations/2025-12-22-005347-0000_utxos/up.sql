CREATE TABLE utxos (
    -- Transaction hash 32 bytes
    txid TEXT NOT NULL,
    -- Output index within the transaction
    vout INTEGER NOT NULL CHECK (vout >= 0),
    -- Value in sats
    value BIGINT NOT NULL CHECK (value >= 0),
    -- ScriptPubKey (raw hex)
    script_pubkey TEXT NOT NULL,
    -- Block height where this UTXO was created
    block_height INTEGER NOT NULL,
    -- Block hash for additional integrity
    block_hash TEXT NOT NULL,
    -- Whether the output has been spent
    spent INTEGER NOT NULL DEFAULT 0 CHECK (spent IN (0, 1)),
    -- Timestamps (unix seconds)
    created_at BIGINT NOT NULL,
    spent_at BIGINT,
    PRIMARY KEY (txid, vout)
);

-- Fast balance queries (unspent only)
CREATE INDEX idx_utxos_unspent ON utxos(spent) WHERE spent = 0;
-- Fast chain reorg / block queries
CREATE INDEX idx_utxos_block_height ON utxos(block_height);

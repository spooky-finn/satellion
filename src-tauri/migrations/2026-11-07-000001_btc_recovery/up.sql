CREATE TABLE btc_broadcast_payloads (
    txid TEXT NOT NULL,
    wallet_name TEXT NOT NULL,
    account_index INTEGER NOT NULL,
    network TEXT NOT NULL,
    raw_transaction TEXT NOT NULL,
    PRIMARY KEY (wallet_name, account_index, network, txid)
);

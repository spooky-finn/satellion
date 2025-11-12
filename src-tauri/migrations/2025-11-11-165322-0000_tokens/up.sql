CREATE TABLE tokens (
  wallet_id INT NOT NULL,
  chain INT NOT NULL,
  symbol TEXT NOT NULL,
  address BLOB NOT NULL,
  decimals INTEGER NOT NULL,
  CONSTRAINT unique_wallet_chain_symbol UNIQUE (wallet_id, chain, symbol)
);
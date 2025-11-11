CREATE TABLE tokens (
  wallet_id INT NO NULL,
  chain INT NOT NULL,
  symbol TEXT NOT NULL,
  address BLOB NOT NULL,
  decimals INTEGER NOT NULL
)
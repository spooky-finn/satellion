-- Your SQL goes here
CREATE TABLE "bitcoin.block_headers" (
  height INTEGER PRIMARY KEY NOT NULL,
  merkle_root TEXT NOT NULL,
  prev_blockhash TEXT NOT NULL,
  time INTEGER NOT NULL,
  version INTEGER NOT NULL,
  bits INTEGER NOT NULL,
  nonce INTEGER NOT NULL
);

CREATE UNIQUE INDEX idx_bitcoin_blocks_hash ON "bitcoin.block_headers" (merkle_root);

CREATE TABLE "keys" (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name TEXT,
  prk TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
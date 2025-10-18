-- Your SQL goes here
CREATE TABLE block_headers (
  height INTEGER PRIMARY KEY NOT NULL,
  merkle_root TEXT NOT NULL,
  prev_blockhash TEXT NOT NULL,
  time INTEGER NOT NULL,
  version INTEGER NOT NULL,
  bits INTEGER NOT NULL,
  nonce INTEGER NOT NULL
);
CREATE UNIQUE INDEX idx_blocks_hash ON block_headers (merkle_root);
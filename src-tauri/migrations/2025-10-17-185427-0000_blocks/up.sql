-- Your SQL goes here
CREATE TABLE blocks (
  height INTEGER PRIMARY KEY NOT NULL,
  merkle_root TEXT NOT NULL,
  prev_blockhash TEXT NOT NULL,
  time BIGINT NOT NULL
);
CREATE UNIQUE INDEX idx_blocks_hash ON blocks (merkle_root);
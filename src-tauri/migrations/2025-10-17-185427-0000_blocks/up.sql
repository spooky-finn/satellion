CREATE TABLE "bitcoin.block_headers" (
  height INTEGER PRIMARY KEY NOT NULL,
  blockhash TEXT NOT NULL,
  prev_blockhash TEXT NOT NULL,
  time INTEGER NOT NULL
);

CREATE UNIQUE INDEX idx_bitcoin_block_headers_blockhash ON "bitcoin.block_headers" (blockhash);
CREATE UNIQUE INDEX idx_bitcoin_block_headers_height ON "bitcoin.block_headers" (height);

CREATE TABLE "bitcoin.compact_filters" (
  blockhash TEXT PRIMARY KEY NOT NULL,
  filter_data BLOB NOT NULL
);

CREATE INDEX idx_bitcoin_compact_filters_blockhash ON "bitcoin.compact_filters" (blockhash);

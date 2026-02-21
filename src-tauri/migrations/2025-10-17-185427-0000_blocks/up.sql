CREATE TABLE "bitcoin.block_headers" (
  height INTEGER PRIMARY KEY NOT NULL,
  blockhash TEXT NOT NULL,
  prev_blockhash TEXT NOT NULL,
  time INTEGER NOT NULL
);

CREATE UNIQUE INDEX idx_bitcoin_block_headers_blockhash ON "bitcoin.block_headers" (blockhash);

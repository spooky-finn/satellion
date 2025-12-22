## 1. Starting the regtest bitcoin node

```bash
brew install bitcoin
```

```bash
bitcoind \
  -regtest \
  -server \
  -txindex \
  -blockfilterindex=1 \
  -peerblockfilters
```

## 2. Load/create wallet

```bash
bitcoin-cli -regtest loadwallet test
```

## 3. Generate blocks

```bash
bitcoin-cli \
  -regtest \
  -rpcport=18443 \
  generatetoaddress 101 $(bitcoin-cli -regtest -rpcport=18443 getnewaddress)
```

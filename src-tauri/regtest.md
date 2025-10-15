starting the regrest bitcoin node 

bitcoind \
  -regtest \
  -server \
  -txindex \
  -blockfilterindex=1 \
  -peerblockfilters


1. load/create wallet
bitcoin-cli -regtest loadwallet test
  
1. generate blocks
bitcoin-cli \
  -regtest \
    -rpcport=18443 \
  generatetoaddress 101 $(bitcoin-cli -regtest -rpcport=18443 getnewaddress)
# TokenInsight Oracle 

![ci workflow](https://github.com/tokeninsight/ti-oracle/actions/workflows/basic.yml/badge.svg)

# Developement Guide
## Run unit test for smart contracts
- firstly, install foundry: `curl -L https://foundry.paradigm.xyz | bash`
- cd contract && forge test --gas-report

## Deploy smart contract

- for example, we deploy a contract for Bitcoin price feeding
  - `forge create TIOracle --rpc-url=https://polygon-rpc.com --interactive --constructor-args bitcoin 5 300 --gas-price 65000000000`
  - you can deploy the `contracts/src/TIOracle.sol` in any ways you like, and `forge` is just one choice
  
- explaining for the above constructor arguments
  - pricing feeding is for `bitcoin`
  - feed `5` times each round
  - timeout for one round is 300 seconds
  
- adding address of the transmission nodes:
  - call this method of the contract `addNode(address newNode)`
  - you can do this by your wallet connected to etherscan
  - Or, you can use the tool `cast`, as the following command
    - `cast send --rpc-url https://polygon-rpc.com ${contract_address} 'addNode(address newNode)' ${node_address} --private-key=$NODE_PRIVATE_KEY --gas-price ${gas_price}`

## start transmission node
- cargo test --noCapture
- cd node && cargo build
  - the binary will be built under the directory `target/debug`
- export NODE_PRIVATE_KEY=${you private key}
- start the node
  - `ti-node -c config/node.yaml`
  - explaining for the configuration file
```
    #p2p listen address, ${ip}/tcp/${port}, if port is zero, random port will be used
    listen_address : /ip4/0.0.0.0/tcp/0

    #log level, info/warn/debug, $RUST_LOG enviroment variable can be used too
    log_level: info

    #RPC URL of Ethereum chain
    eth_rpc_url: https://polygon-rpc.com

    #p2p message topic, use sperated topic for each coin price feeding
    price_topic: BITCOIN

    #smart contract address
    contract_address: 0xe1489011fac9506011fb8c089ee2dda1568607cb

    #coin name flag which should be same as the one specified in contract
    coin_name: bitcoin

    #enviroment variables contains wallet key
    private_key: $NODE_PRIVATE_KEY

    #the interval in seconds between twice pricing feeding
    feed_interval: 60

    #suggested max fee per gas
    fee_per_gas: 65

    #trading pairs used of CEX & DEX to aggrate price
    mappings:
      binance:
        - BTCUSDC
        - BTCUSDT
      coinbase:
        - BTC-USD
        - BTC-USDC
      uniswapv3:
        - 0x99ac8ca7087fa4a2a1fb6357269965a2014abc35
        - 0x9db9e0e53058c89e5b94e29621a205198648425b
      ftx:
        - BTC/USD
        - BTC/USDT
      kucoin:
        - BTC-USDT
        - BTC-USDC

    #specify some bootstrap nodes, one for each line
    peers:
      - ""
```
- join the network
  - use `--peers` to specify bootstrap nodes with the IPFS-style address sperated by comma
  - e.g. `ti-node --peers /ip4/192.168.10.228/tcp/55909`

# Some Onchain Demo
- Bitcoin Spot Price
  - https://polygonscan.com/address/0xe1489011fac9506011fb8c089ee2dda1568607cb
- ETH Spot Price
  - TODO

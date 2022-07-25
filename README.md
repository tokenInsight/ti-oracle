# TokenInsight Oracle

![ci workflow](https://github.com/tokeninsight/ti-oracle/actions/workflows/basic.yml/badge.svg)

# Build dedicated oracle

## Why people need yet another oracle?
For DeFi protocols, the price of crypto assets is very important, because price is a signal that determines subsequent operations, such as the liquidation of collateral assets.

However, popular oracles in the industry, such as Chainlink, have not yet solved the problem of the accuracy of price data. The fundamental reason is that their feeding nodes do not use fresh data. The data are provided by centralized vendor such as coingecko. Chainlink only solves the problem of preventing feeding nodes doing evil.  That is not enough to reducing the security risk.

## Solution to build dedicated oracle
For critical DeFi application, relying on third-party price is dangerous, project owner must build their own feeding network with their partners.
We provide a solution to build up dedicated oracle networks, which are comprised of two components: oracle-node and oracle-contract.
- Oracle-node is used to build up a p2p network, in which all the nodes commit price data into blockchain in a round-robin way. They crawl trading pair's price from self specified exchanges and trading-pairs, and then aggragate the data to calculate out a price weighted by trading volumes.
- Oracle-contract is used to store the price on blockchain, and also used to mantain the permitted node list for the dedicated oracle network.

## Demo networks with two nodes
- https://ti-node.fly.dev/
- https://ti-node2.fly.dev/

## Architecture Overview
![image](https://user-images.githubusercontent.com/167837/177757017-bfc35f14-6d32-4f1d-8db9-5d1febab1baf.png)

## Concatenation Calculation Expression of trading-pairs
- in order to support caculate the price by using trading-paris with different quote, concatenation expression is used
- currently, we support two operators: multiplication and division
- for example
  - an exchange only provides the price off two pairs:`WBTC/ETH`, `ETH/USDC`, but sometimes pepole want use USDC as standard quote
  - In this case, a concatenation expression could be used as `WBTC/ETH` **mul** `ETH/USDC`
  - On the other side, if only `WBTC/USDT` and `USDC/USDT` are provided, and we want the quote to be `USDC`, in this case, use the expression as `WBTC/USDT` **div** `USDC/USDT`

## Price-feeding  scheduling
The basic scheduling is in a round-robbin way, each node can do feeding servral times one by one.
In each round, one node is selected as leader, who is responsible for collecting price observed by other nodes, and make a summary to commit data into smart contract.
The leader node does the following tasks:
- verify the signatures in the message sent from other nodes
- check the diffrence between price observed by other nodes and local, and reject the data with large difference
- remove the price data recognized to be outliers
  - outliers detection details: https://github.com/tokenInsight/ti-oracle/blob/main/node/src/fetcher/aggregator.rs#L88
- caculate the price weighted by the trading volumes

Suppose we maintain a counter for how many times have feeded from the begining of contract deployed, as a variable `N`.
- a variable `T`, which specify how many times one node can feed in each round.
- a variable `M`, which specify how many nodes in the network are permiteed to do price feeding works.
- Then, the accepted leader should be the one `nodes[(N / T) % M]`

For example, assuming the price data feeded once per minute, and there are a total of 3 price feeding nodes: a, b, c, each node can be fed 5 times in a row, then the ordinary sequence should be:
```
+--------+--------+--------+
| #Count | #Round | Leader |
+--------+--------+--------+
|      0 |      0 | a      |
|      1 |      0 | a      |
|      2 |      0 | a      |
|      3 |      0 | a      |
|      4 |      0 | a      |
|      5 |      1 | b      |
|      6 |      1 | b      |
|      7 |      1 | b      |
|      8 |      1 | b      |
|      9 |      1 | c      |
|     10 |      2 | c      |
|     11 |      2 | c      |
|     12 |      2 | c      |
|     13 |      2 | c      |
|     14 |      2 | c      |
|     15 |      3 | a      |
+--------+--------+--------+

```

## What-if one node is crashed or disconnected?
- Each round should have a timeout of do feeding, like, 300 seconds, to prevent the system outage
- If the time passed between last feeding and the current block.timestamp, then any node in the permitteed list is allowed to feed data
- The details can be checked in the source code of smart contract: https://github.com/tokenInsight/ti-oracle/blob/main/contracts/src/TIOracle.sol#L69
- Simply speaking, we use the smart contract as like the role of Zookeeper in traditional distributed system


# Developement Guide
## Source code overview
```
contracts/src
└── TIOracle.sol        # smart contract of TokenInsight oracle
contracts/test
└── TIOracle.t.sol      # unit test for smart contract
node/src
├── bin
│   └── server.rs       # the entry point, core logic of node
├── chains
│   ├── eth.rs          # functions about crypto sigh, hash, and smart contract invoke
│   └── mod.rs
├── fetcher
│   ├── aggregator.rs   # functions about weighted price calculating, and outliers detection
│   ├── expression.rs   # caculate price by using operators lik div, mul
│   ├── binance.rs      # fetching data from Binance
│   ├── coinbase.rs     # fetching data from Coinbase
│   ├── curve.rs        # to be done
│   ├── ftx.rs          # fetching data from FTx
│   ├── kucoin.rs       # fetching data from Kucoin
│   ├── mod.rs
│   ├── okex.rs         # fetching data from okex
│   ├── uniswapv2.rs    # fetching data from uniswap v2
│   └── uniswapv3.rs    # fetching data from uniswap v3
├── flags.rs            # command line flags & configuration options
├── lib.rs
└── processor           # network processors
    ├── gossip.rs       # p2p gossip messages handlers
    ├── mod.rs
    ├── swarm.rs        # setup swarm for serving p2p node
    └── utils.rs        # some utility functions

```
## Run unit test for smart contracts
- firstly, install foundry: `curl -L https://foundry.paradigm.xyz | bash`
- cd contract && forge test --gas-report

## Deploy smart contract

- for example, we deploy a contract for Bitcoin price feeding
  - `forge create TIOracle --rpc-url=https://polygon-rpc.com --interactive --constructor-args bitcoin 5 300 --gas-price 65000000000`
  - you can deploy the `contracts/src/TIOracle.sol` in any ways you like, and `forge` is just one choice

- meaning of the above constructor arguments
  - pricing feeding is for `bitcoin`
  - feed `5` times each round
  - timeout for one round is 300 seconds

- adding address of the permitted transmission nodes
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

#web server addres
web_address: 127.0.0.1:8080

#log level, info/warn/debug, $RUST_LOG enviroment variable can be used too
log_level: info

#RPC URL of Ethereum chain
eth_rpc_url: https://polygon-rpc.com

#smart contract address
contract_address: 0xfaaa1887a03e4df74f129dc02fa638f4563b0d06

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
    - BTCUSDT div USDCUSDT
  coinbase:
    - BTC-USD
    - BTC-USDT mul USDT-USD
  uniswapv3:
    - 0x99ac8ca7087fa4a2a1fb6357269965a2014abc35 #WBTC-USDC
    - 0xcbcdf9626bc03e24f779434178a73a0b4bad62ed div 0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8 #WBTC-ETH div USDC-ETH
  uniswapv2:
    - 0x004375dff511095cc5a197a54140a24efef3a416 #WBTC-USDC
    - 0xbb2b8038a1640196fbe3e38816f3e67cba72d940 div 0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc #WBTC-ETH div USDC-ETH
  ftx:
    - BTC/USD
    - BTC/USDT mul USDT/USD
  kucoin:
    - BTC-USDT mul USDT-USDC
    - BTC-USDC
  okex:
    - BTC-USDC
    - BTC-USDT div USDC-USDT
  sushiswap:
    - 0xceff51756c56ceffca006cd410b03ffc46dd3a58 div 0x397ff1542f962076d0bfe58ea045ffa2d347aca0 #wBTC-ETH div USDC-ETH
#specify some bootstrap nodes, one for each line
peers:
  - ""
```
when you start one node sucessfully, you will get the following logs on your terminal:

![image](https://user-images.githubusercontent.com/167837/177996801-77c5e60a-3415-42e4-a891-cfa5dc6e7f6a.png)

use `export RUST_LOG=debug`, if you want more tracing details.

- join the network
  - use `--peers` to specify bootstrap nodes with the IPFS-style address sperated by comma
  - e.g. `ti-node --peers /ip4/192.168.10.228/tcp/55909`

# Some Onchain Demo
- Bitcoin Spot Price
  - https://polygonscan.com/address/0xfaaa1887a03e4df74f129dc02fa638f4563b0d06#readContract
- ETH Spot Price
  - TODO

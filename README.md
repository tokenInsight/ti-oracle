# ti-oracle

![ci workflow](https://github.com/tokeninsight/ti-oracle/actions/workflows/basic.yml/badge.svg)

# Developement Guide
## contract test
- install foundry: `curl -L https://foundry.paradigm.xyz | bash`
- cd contract && forge test
## deploy smart contract
- forge create TIOracle --rpc-url=https://polygon-rpc.com --interactive --constructor-args bitcoin 5 300 --gas-price 65000000000
- constructor arguments: 
  - pricing feeding is for `bitcoin`
  - feed `5` times each round
  - timeout for one round is 300 seconds

## start transmission node
- cargo test --noCapture
- cd node
- cargo run --bin ti-node

# Polygon demo
- https://polygonscan.com/address/0xf3787681d966249eb4dec209227460c269c2052a

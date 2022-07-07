# ti-oracle

![ci workflow](https://github.com/tokeninsight/ti-oracle/actions/workflows/basic.yml/badge.svg)

# Developement Guide
## contract test
- install foundry: `curl -L https://foundry.paradigm.xyz | bash`
- cd contract && forge test
## deploy smart contract
- e.g forge create TIOracle --rpc-url=https://polygon-rpc.com --interactive --constructor-args bitcoin 5 300 --gas-price 65000000000
- explain constructor arguments:
  - pricing feeding is for `bitcoin`
  - feed `5` times each round
  - timeout for one round is 300 seconds

## start transmission node
- cargo test --noCapture
- cd node
- export NODE_PRIVATE_KEY=${you private key}
- cargo run --bin ti-node

# Polygon demo
- https://polygonscan.com/address/0xe1489011fac9506011fb8c089ee2dda1568607cb

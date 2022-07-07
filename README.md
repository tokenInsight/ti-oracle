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
  
- add address of transmission nodes:
  - call this method of the contract `addNode(address newNode)`

## start transmission node
- cargo test --noCapture
- cd node
- export NODE_PRIVATE_KEY=${you private key}
- cargo run --bin ti-node

# Some Onchain Demo
- Bitcoin Spot Price
  - https://polygonscan.com/address/0xe1489011fac9506011fb8c089ee2dda1568607cb
- ETH Spot Price
  - TODO

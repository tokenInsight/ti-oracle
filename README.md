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
  - TODO
- join the network
  - use `--peers` to specify bootstrap nodes with the IPFS-style address sperated by `,`
  - e.g. `ti-node --peers /ip4/192.168.10.228/tcp/55909`

# Some Onchain Demo
- Bitcoin Spot Price
  - https://polygonscan.com/address/0xe1489011fac9506011fb8c089ee2dda1568607cb
- ETH Spot Price
  - TODO

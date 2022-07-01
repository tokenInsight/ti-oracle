# ti-oracle

![ci workflow](https://github.com/tokeninsight/ti-oracle/actions/workflows/basic.yml/badge.svg)

# contract test
- install foundry: `curl -L https://foundry.paradigm.xyz | bash`
- cd contract && forge test

# start transmission node
- cargo test --noCapture
- cd node && cargo build
- ./target/debug/ti-node

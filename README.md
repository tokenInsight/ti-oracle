# ti-oracle

# contract test
- install foundry: `curl -L https://foundry.paradigm.xyz | bash`
- cd contract && forge test

# start transmission node
- cargo test --noCapture
- cd node && cargo build
- ./target/debug/ti-node

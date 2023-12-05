## watch contract
cargo watch -w "src" -s "cargo concordium build --out ciphers_nft.wasm.v1"

## watch tests
cargo watch -w "tests" -s "cargo test -- --nocapture"

cargo concordium test
cargo test -- --nocapture
cargo concordium build --out ciphers_nft.wasm.v1
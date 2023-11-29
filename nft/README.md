cargo concordium test
cargo test -- --nocapture
cargo concordium build --out nft_test.wasm.v1


cargo watch -w "src" -s "cargo concordium build --out nft_test.wasm.v1"
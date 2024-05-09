# Stylus create2 example

Contract `factory` is used to deploy `counter` with create2.

## Steps

1. Compile counter

```sh
cargo build -p counter --target wasm32-unknown-unknown --release
```

2. Obtain init code for `counter`

```sh
cargo stylus deploy --wasm-file-path ./target/wasm32-unknown-unknown/release/counter.wasm --dry-run --output-tx-data-to-dir ./crates/factory/src --private-key-path ./.localnet.key
```

2.

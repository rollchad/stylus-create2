# Stylus create2 example

Contract `factory` is used to deploy `counter` with create2.

## Steps

1. Compile counter

```sh
cargo build -p counter --target wasm32-unknown-unknown --release
```

2. Obtain init code for `counter`. This outputs file [deployment_tx_data](./crates/factory/src/deployment_tx_data) in the `factory` crate.

```sh
cargo stylus deploy --wasm-file-path ./target/wasm32-unknown-unknown/release/counter.wasm --dry-run --output-tx-data-to-dir ./crates/factory/src --mode deploy-only --private-key-path ./.privatekey
```

3. The factory contract reads bytes from `deployment_tx_data`, then deploys it with create2.

```rs
#[external]
impl Factory {
    pub fn deploy_with_create2(&mut self) -> Result<(), Vec<u8>> {
        let init_code = include_bytes!("./deployment_tx_data");

        let salt = B256::default();
        let endowment = U256::ZERO;

        let actual_address = unsafe { RawDeploy::new().salt(salt).deploy(init_code, endowment)? };

        Ok(())
    }
}
```

4. Deploy factory and call `deploy_with_create2()`

```sh
# compile
cargo build -p factory --target wasm32-unknown-unknown --release

# deploy factory- note the address
cargo stylus deploy --wasm-file-path ./target/wasm32-unknown-unknown/release/factory.wasm --private-key-path ./.privatekey

# setup PRIVATE_KEY and FACTORY_ADDRESS env variables
PRIVATE_KEY=yourkey

FACTORY_ADDRESS=deployedAddressFromDeployCommand

# call deploy_with_create2() with cast
cast send --rpc-url https://stylus-testnet.arbitrum.io/rpc --private-key $PRIVATE_KEY $FACTORY_ADDRESS "deployWithCreate2()"
```

## Activation

Activation has a nifty hack. You only need to activate one instance of a contract. Subsequent copies with the same code will automatically be activated.

We deploy a new copy of `counter` with the CLI. This will also activate the `counter` instance deployed by our factory.

```sh
cargo stylus deploy --wasm-file-path ./target/wasm32-unknown-unknown/release/counter.wasm --private-key-path ./.privatekey
```

To test, lookup the instance address that was deployed from factory using blockscout, then use `cast` to send a transaction.

```sh
# gives 0
cast call --rpc-url https://stylus-testnet.arbitrum.io/rpc --private-key $PRIVATE_KEY $COUNTER_ADDRESS "number()"

# increment
cast send --rpc-url https://stylus-testnet.arbitrum.io/rpc --private-key $PRIVATE_KEY $COUNTER_ADDRESS "increment()"
```

## Bonus- Deriving create2 address without deploying

The SDK lacked a function to derive create2 address, so I created one.

```rs
fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};

    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);

    output
}

fn get_create2_address(from: Address, salt: B256, init_code: &[u8]) -> Address {
    let init_code_hash = keccak256(init_code);

    let mut bytes = Vec::with_capacity(1 + 20 + salt.len() + init_code_hash.len());

    bytes.push(0xff);
    bytes.extend_from_slice(from.as_slice());
    bytes.extend_from_slice(salt.as_slice());
    bytes.extend_from_slice(init_code_hash.as_slice());

    let hash = keccak256(bytes.as_slice());

    let mut address_bytes = [0u8; 20];
    address_bytes.copy_from_slice(&hash[12..]);

    let address = Address::from_slice(&address_bytes);

    address
}
```
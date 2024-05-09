#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use stylus_sdk::{alloy_primitives::{tiny_keccak, Address, B256, U256}, contract, deploy::RawDeploy, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Factory {

    }
}

#[external]
impl Factory {
    pub fn deploy_with_create2(&mut self) -> Result<(), Vec<u8>> {
        let init_code = include_bytes!("./deployment_tx_data");

        let salt = B256::default();
        let endowment = U256::ZERO;

        let expected_address = get_create2_address(contract::address(), salt, init_code);
        let actual_address = unsafe { RawDeploy::new().salt(salt).deploy(init_code, endowment)? };

        assert_eq!(expected_address, actual_address);
        Ok(())
    }
}

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

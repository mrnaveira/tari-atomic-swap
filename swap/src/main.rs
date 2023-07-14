use std::{thread, time};

use ethereum::EthereumContractManager;

#[tokio::main]
async fn main() {
    let rpc_url = "http://127.0.0.1:7545".to_string();
    let eth_contract_address = "0x4e34B1f6C49E23ff9eE1C9aDd85157b534FB2Df3".to_string();
    let erc20_contract_address = "0xf59577F5af09705C83175C3e258E544A6EAa54B9".to_string();

    let manager =
        EthereumContractManager::new(rpc_url, eth_contract_address, erc20_contract_address)
            .unwrap();

    let preimage = "foo6";
    let mut preimage_bytes = [0u8; 32];
    preimage_bytes[..preimage.len()].copy_from_slice(preimage.as_bytes());

    let hashlock = EthereumContractManager::create_hashlock(preimage_bytes);
    let receiver = "0xCC4afed01b9042A39D49F58927cB5bB83Eb419AC".to_string();
    let timelock = 10;

    let contract_id = manager
        .new_contract(receiver, hashlock, timelock)
        .await
        .unwrap();

    thread::sleep(time::Duration::from_secs(20));

    manager.refund(contract_id).await.unwrap();
}

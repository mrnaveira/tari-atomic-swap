use ethereum::{EthereumContractManager, Preimage};
use ethers::{
    signers::{LocalWallet, Signer},
    types::U256,
    utils::parse_units,
};

// This test simulates an atomic swap between two accounts inside an Ethereum network.
// Obviously atomic swaps inside the same network does not have any real world utility,
// but for testing purposes it's useful as it allow us to verify the Ethereum contract
#[tokio::test]
async fn successful_swap() {
    // TODO: spawn a fresh Ganache or Anvil network each time
    let alice_wallet = "0xc48c03c85fe3d0996282ba5e501dd2682c2b8d8ab97e6b87081dff5cb0042fed"
        .parse::<LocalWallet>()
        .unwrap();
    let bob_wallet = "0x08101179aab21e1b5c215e94b8eb7c9e829b0c5044d7ab85bf09196b1bb6c2da"
        .parse::<LocalWallet>()
        .unwrap();

    let rpc_url = "http://127.0.0.1:7545".to_string();
    let eth_contract_address = "0x4e34B1f6C49E23ff9eE1C9aDd85157b534FB2Df3".to_string();

    // Alice locks her funds
    let alice_manager = EthereumContractManager::new(
        alice_wallet.clone(),
        rpc_url.clone(),
        eth_contract_address.clone(),
    )
    .await
    .unwrap();
    let amount_wei: U256 = parse_units("1000", "wei").unwrap().into();
    let preimage = build_preimage("foo".to_string());
    let hashlock = EthereumContractManager::create_hashlock(preimage);
    let alice_timelock = 100; // seconds
    let alice_contract_id = alice_manager
        .new_contract(amount_wei, bob_wallet.address(), hashlock, alice_timelock)
        .await
        .unwrap();

    // Bob locks his funds
    let bob_timelock = 50; // seconds, must be lower than Alice's timelock for Bob to have enough time to withdraw
    let bob_manager =
        EthereumContractManager::new(bob_wallet, rpc_url.clone(), eth_contract_address.clone())
            .await
            .unwrap();
    let bob_contract_id = bob_manager
        .new_contract(amount_wei, alice_wallet.address(), hashlock, bob_timelock)
        .await
        .unwrap();

    // Alice withdraws funds from Bob's contract, revealing the preimage in the process
    alice_manager
        .withdraw(bob_contract_id, preimage)
        .await
        .unwrap();

    // Bob retrieves the preimage
    let retrieved_preimage = bob_manager.get_preimage(bob_contract_id).await.unwrap();

    // Bob withdraws funds from Alice's contract
    bob_manager
        .withdraw(alice_contract_id, retrieved_preimage)
        .await
        .unwrap();
}

fn build_preimage(str: String) -> Preimage {
    let mut preimage = [0u8; 32];
    preimage[..str.len()].copy_from_slice(str.as_bytes());
    preimage
}

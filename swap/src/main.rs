extern crate dotenv;

use dotenv::dotenv;
use ethereum::EthereumContractManager;
use ethereum::Preimage;
use ethers::utils::hex;
use ethers::{
    signers::{LocalWallet, Signer},
    types::U256,
    utils::parse_units,
};
use std::env;
use tari::TariContractManager;
use tari_crypto::ristretto::RistrettoPublicKey;
use tari_crypto::tari_utilities::hex::Hex;
use tari_template_lib::prelude::TemplateAddress;

#[tokio::main]
async fn main() {
    println!("Reading environment variables");
    dotenv().ok();

    let preimage = build_preimage(get_envvar("PREIMAGE"));

    let eth_alice_private_key = get_envvar("ETHEREUM_ALICE_PRIVATE_KEY");
    let eth_bob_private_key = get_envvar("ETHEREUM_BOB_PRIVATE_KEY");
    let eth_rpc_url = get_envvar("ETHEREUM_RPC_URL");
    let eth_contract_address = get_envvar("ETHEREUM_SMART_CONTRACT_ADDRESS");
    let eth_amount_wei = get_envvar("ETHEREUM_AMOUNT_IN_WEI");
    let eth_amount_wei: U256 = parse_units(eth_amount_wei, "wei").unwrap().into();

    let tari_alice_public_key_index = get_envvar("TARI_ALICE_PUBLIC_KEY_INDEX")
        .parse::<u64>()
        .unwrap();
    let tari_alice_public_key = get_envvar("TARI_ALICE_PUBLIC_KEY");
    let tari_alice_public_key = RistrettoPublicKey::from_hex(&tari_alice_public_key).unwrap();

    let tari_bob_public_key_index = get_envvar("TARI_BOB_PUBLIC_KEY_INDEX")
        .parse::<u64>()
        .unwrap();
    let tari_bob_public_key = get_envvar("TARI_BOB_PUBLIC_KEY");
    let tari_bob_public_key = RistrettoPublicKey::from_hex(&tari_bob_public_key).unwrap();

    let tari_wallet_endpoint = get_envvar("TARI_WALLET_ENDPOINT");
    let tari_wallet_token = get_envvar("TARI_WALLET_TOKEN");
    let tari_swap_template_address = get_envvar("TARI_SWAP_TEMPLATE_ADDRESS");
    let tari_swap_template_address =
        TemplateAddress::from_hex(&tari_swap_template_address).unwrap();
    let tari_amount = get_envvar("TARI_AMOUNT");
    let tari_amount = tari_amount.parse::<i64>().unwrap();

    println!("Alice will lock her funds on the Ethereum network");
    let alice_eth_wallet = eth_alice_private_key.parse::<LocalWallet>().unwrap();
    let bob_eth_wallet = eth_bob_private_key.parse::<LocalWallet>().unwrap();
    let timelock_eth = 100; // seconds
    let hashlock = EthereumContractManager::create_hashlock(preimage);
    let alice_eth_manager = EthereumContractManager::new(
        alice_eth_wallet.clone(),
        eth_rpc_url.clone(),
        eth_contract_address.clone(),
    )
    .await
    .unwrap();
    let alice_eth_contract_id = alice_eth_manager
        .new_contract(
            eth_amount_wei,
            bob_eth_wallet.address(),
            hashlock,
            timelock_eth,
        )
        .await
        .unwrap();
    println!(
        "    - Alice funds locked on Ethereum with contract_id = '{}'",
        hex::encode(alice_eth_contract_id)
    );

    println!("Bob will lock his funds on the Tari network");
    let mut bob_tari_contract_manager = TariContractManager::new(
        tari_wallet_endpoint.clone(),
        tari_bob_public_key.clone(),
        tari_bob_public_key_index,
        tari_wallet_token.clone(),
        tari_swap_template_address,
    )
    .unwrap();
    let timelock_tari = 5; // epochs
    let contract_id_tari = bob_tari_contract_manager
        .create_lock_contract(
            tari_amount,
            tari_alice_public_key.clone(),
            hashlock,
            timelock_tari,
        )
        .await
        .unwrap();
    println!(
        "    - Bob funds locked on Tari with component address ='{}'",
        &contract_id_tari
    );

    println!("Alice will withdraw the funds from Bob's contract in Tari, revealing the preimage");
    let mut alice_tari_contract_manager = TariContractManager::new(
        tari_wallet_endpoint,
        tari_alice_public_key,
        tari_alice_public_key_index,
        tari_wallet_token,
        tari_swap_template_address,
    )
    .unwrap();
    alice_tari_contract_manager
        .withdraw(contract_id_tari, preimage)
        .await
        .unwrap();
    println!("    - Alice sucessfully withdraws funds from Bob's contract");

    println!("Bob will retrieve the preimage revealed by Alice");
    let revealed_preimage = bob_tari_contract_manager
        .get_preimage(contract_id_tari)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(preimage, revealed_preimage, "Error: preimage mistmatch");
    println!("    - Bob sucessfully retrieves the preimage");

    println!("Bob will withdraws funds from Alice's contract on Ethereum");
    let bob_eth_manager = EthereumContractManager::new(
        bob_eth_wallet.clone(),
        eth_rpc_url.clone(),
        eth_contract_address.clone(),
    )
    .await
    .unwrap();
    bob_eth_manager
        .withdraw(alice_eth_contract_id, revealed_preimage)
        .await
        .unwrap();
    println!("    - Bob sucessfully withdraws funds from Bob's contract");
    println!();
    println!("Atomic swap finished successfully");
}

fn get_envvar(key: &str) -> String {
    env::var(key).unwrap()
}

fn build_preimage(str: String) -> Preimage {
    let mut preimage = [0u8; 32];
    preimage[..str.len()].copy_from_slice(str.as_bytes());
    preimage
}

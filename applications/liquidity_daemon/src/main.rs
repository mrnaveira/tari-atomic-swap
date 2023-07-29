use std::{net::SocketAddr, sync::Arc};

use crate::{
    config::Config,
    json_rpc::{run_json_rpc, JsonRpcHandlers},
};
use ethereum::EthereumContractManager;
use ethers::signers::LocalWallet;
use log::info;
use position_manager::PositionManager;
use swap_manager::SwapManager;
use tari::contract::TariContractManager;
use tari_crypto::{ristretto::RistrettoPublicKey, tari_utilities::hex::Hex};
use tari_template_lib::prelude::TemplateAddress;
use tokio::{signal, task};

mod config;
mod json_rpc;
mod position_manager;
mod swap_manager;

const LOG_TARGET: &str = "liquidity_daemon";

#[tokio::main]
async fn main() {
    let config = Config::read("./config.json".to_string());

    // init the position manager
    let mut position_manager =
        PositionManager::new(config.clone()).expect("Could not create position manager");
    position_manager
        .sync()
        .await
        .expect("Could not sync position manager with the Tari network");

    // init the ethereum manager
    // TODO: properly handle private keys
    let eth_wallet = config.ethereum.private_key.parse::<LocalWallet>().unwrap();
    let eth_manager = EthereumContractManager::new(
        eth_wallet,
        config.ethereum.rpc_url,
        config.ethereum.contract_address,
    )
    .await
    .expect("Could not initialize the Ethereum manager");

    // init the tari manager
    let tari_public_key =
        RistrettoPublicKey::from_hex(&config.tari.public_key).expect("Invalid Tari public key ");
    let tari_swap_template = TemplateAddress::from_hex(&config.tari.swap_template)
        .expect("Invalid Tari swap template address ");
    let tari_manager = TariContractManager::new(
        config.tari.wallet_endpoint,
        tari_public_key,
        config.tari.public_key_index,
        config.tari.wallet_token.clone(),
        tari_swap_template,
    )
    .expect("Could not initialize the Tari manager");

    // init the swap manager
    let swap_manager = Arc::new(SwapManager::new(
        position_manager,
        eth_manager,
        tari_manager,
    ));

    // run the swap JSON-RPC
    info!(target: LOG_TARGET, "🌐 Starting JSON-RPC server on {}", config.network_address);
    let handlers = JsonRpcHandlers::new(swap_manager);
    let json_rpc_address: SocketAddr = config
        .network_address
        .parse()
        .expect("Invalid network address");
    task::spawn(run_json_rpc(json_rpc_address, handlers));

    // TODO: we need a recurring process to keep track of ongoing swaps, and do refunds if they expire

    match signal::ctrl_c().await {
        Ok(()) => {
            eprintln!("Shutdown signal received");
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}

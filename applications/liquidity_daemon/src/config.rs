use serde::{Deserialize, Serialize};
use std::fs;
use tari::liquidity::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    ethereum: EthereumConfig,
    tari: TariConfig,
    positions: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    rpc_url: String,
    private_key: String,
    contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TariConfig {
    public_key: String,
    public_key_index: u64,
    wallet_endpoint: String,
    wallet_token: String,
    swap_template: String,
    liquidity_component: String,
}

impl Config {
    pub fn read(path: String) -> Self {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Unable to read config file '{}'", path));

        serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("'{}' file does not have a valid JSON format", path))
    }
}

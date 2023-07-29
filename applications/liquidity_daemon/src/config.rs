use serde::{Deserialize, Serialize};
use std::fs;
use tari::liquidity::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub network_address: String,
    pub ethereum: EthereumConfig,
    pub tari: TariConfig,
    pub positions: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub private_key: String,
    pub contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TariConfig {
    pub public_key: String,
    pub public_key_index: u64,
    pub wallet_endpoint: String,
    pub wallet_token: String,
    pub swap_template: String,
    pub liquidity_component: String,
}

impl Config {
    pub fn read(path: String) -> Self {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Unable to read config file '{}'", path));

        serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("'{}' file does not have a valid JSON format", path))
    }
}

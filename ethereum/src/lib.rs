use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;

use ethers::prelude::abigen;
use ethers::prelude::decode_function_data;
use ethers::prelude::signer::SignerMiddlewareError;
use ethers::prelude::ContractError;
use ethers::prelude::Http;
use ethers::prelude::Provider;
use ethers::prelude::SignerMiddleware;
use ethers::providers::Middleware;
use ethers::signers::LocalWallet;
use ethers::signers::Signer;
use ethers::types::Address;
use ethers::types::H256;
use ethers::utils::hex::ToHex;
use sha2::Digest;
use sha2::Sha256;
use thiserror::Error;

pub type ContractId = [u8; 32];
pub type Preimage = [u8; 32];
pub type Hashlock = [u8; 32];

pub struct EthereumContractManager {
    provider: Provider<Http>,
    eth_contract_address: Address,
    _erc20_contract_address: Address,
}

impl EthereumContractManager {
    pub fn new(
        rpc_url: String,
        eth_contract_address: String,
        erc20_contract_address: String,
    ) -> Result<Self, EthereumError> {
        let provider = parse_rpc_url(rpc_url)?;
        let eth_contract_address = parse_address(eth_contract_address)?;
        let _erc20_contract_address = parse_address(erc20_contract_address)?;

        Ok(Self {
            provider,
            eth_contract_address,
            _erc20_contract_address,
        })
    }

    pub fn create_hashlock(preimage: Preimage) -> Hashlock {
        let mut hasher = Sha256::new();
        hasher.update(preimage);
        hasher.finalize().into()
    }

    pub async fn new_contract(
        &self,
        receiver: String,
        hashlock: Hashlock,
        timelock: u64,
    ) -> Result<ContractId, EthereumError> {
        let wallet = "0xc48c03c85fe3d0996282ba5e501dd2682c2b8d8ab97e6b87081dff5cb0042fed"
            .parse::<LocalWallet>()
            .map_err(|e| EthereumError::WalletError {
                detail: e.to_string(),
            })?;
        let client = SignerMiddleware::new_with_provider_chain(self.provider.clone(), wallet)
            .await
            .map_err(|e| EthereumError::WalletError {
                detail: e.to_string(),
            })?;

        // TODO: this should be generated only once
        abigen!(HashedTimelock, "abi/HashedTimelock.json");

        let contract = HashedTimelock::new(self.eth_contract_address, client.into());

        let receiver = parse_address(receiver)?;

        let timelock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .checked_add(Duration::new(timelock, 0))
            .unwrap()
            .as_secs();

        let tx = contract
            .new_contract(receiver, hashlock, timelock.into())
            .value(1000);

        let receipt = tx.send().await.unwrap().await.unwrap();
        println!("receipt: {receipt:#?}");

        // In solidity the first topic is the hash of the signature of the event
        // So "contractId" will be in second place on the topics of the "LogHTLCNew" event
        let contract_id: H256 = receipt.unwrap().logs[0].topics[1];
        let contract_id_hex: String = contract_id.encode_hex();
        println!("contract_id: {contract_id_hex}");

        Ok(contract_id.into())
    }

    pub async fn refund(&self, contract_id: ContractId) -> Result<(), EthereumError> {
        let wallet = "0xc48c03c85fe3d0996282ba5e501dd2682c2b8d8ab97e6b87081dff5cb0042fed"
            .parse::<LocalWallet>()
            .map_err(|e| EthereumError::WalletError {
                detail: e.to_string(),
            })?;
        let client = SignerMiddleware::new_with_provider_chain(self.provider.clone(), wallet)
            .await
            .map_err(|e| EthereumError::WalletError {
                detail: e.to_string(),
            })?;

        // TODO: this should be generated only once
        abigen!(HashedTimelock, "abi/HashedTimelock.json");

        let contract = HashedTimelock::new(self.eth_contract_address, client.into());

        let tx = contract.refund(contract_id);

        let receipt = tx.send().await.unwrap().await.unwrap();
        println!("receipt: {receipt:#?}");

        Ok(())
    }
}

fn parse_rpc_url(input: String) -> Result<Provider<Http>, EthereumError> {
    Provider::<Http>::try_from(input.clone()).map_err(|e| EthereumError::InvalidRpcUrl {
        input,
        detail: e.to_string(),
    })
}

fn parse_address(input: String) -> Result<Address, EthereumError> {
    input
        .parse::<Address>()
        .map_err(|e| EthereumError::InvalidAddress {
            input,
            detail: e.to_string(),
        })
}

#[derive(Error, Debug)]
pub enum EthereumError {
    #[error("Invalid address '{input}': {detail}")]
    InvalidAddress { input: String, detail: String },
    #[error("Invalid HTTP provider '{input}': {detail}")]
    InvalidRpcUrl { input: String, detail: String },
    #[error("WalletError: {detail}")]
    WalletError { detail: String },
}

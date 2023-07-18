use std::time::Duration;
use std::time::SystemTime;

use ethers::prelude::abigen;
use ethers::prelude::Http;
use ethers::prelude::Provider;
use ethers::prelude::SignerMiddleware;
use ethers::signers::LocalWallet;
use ethers::types::Address;
use ethers::types::H256;
use sha2::Digest;
use sha2::Sha256;
use thiserror::Error;

type ByteArray32 = [u8; 32];
pub type ContractId = ByteArray32;
pub type Preimage = ByteArray32;
pub type Hashlock = ByteArray32;

pub struct EthereumContractManager {
    client: SignerMiddleware<Provider<Http>, LocalWallet>,
    eth_contract_address: Address,
}

impl EthereumContractManager {
    pub async fn new(
        wallet: LocalWallet,
        rpc_url: String,
        eth_contract_address: String,
    ) -> Result<Self, EthereumError> {
        let provider = parse_rpc_url(rpc_url)?;
        let client = SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone())
            .await
            .map_err(|e| EthereumError::WalletError {
                detail: e.to_string(),
            })?;
        let eth_contract_address = parse_address(eth_contract_address)?;

        Ok(Self {
            client,
            eth_contract_address,
        })
    }

    pub fn create_hashlock(preimage: Preimage) -> Hashlock {
        let mut hasher = Sha256::new();
        hasher.update(preimage);
        hasher.finalize().into()
    }

    pub async fn new_contract(
        &self,
        receiver: Address,
        hashlock: Hashlock,
        timelock: u64,
    ) -> Result<ContractId, EthereumError> {
        // TODO: this should be generated only once
        abigen!(HashedTimelock, "abi/HashedTimelock.json");

        let contract = HashedTimelock::new(self.eth_contract_address, self.client.clone().into());

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

        // In solidity the first topic is the hash of the signature of the event
        // So "contractId" will be in second place on the topics of the "LogHTLCNew" event
        let contract_id: H256 = receipt.unwrap().logs[0].topics[1];

        Ok(contract_id.into())
    }

    pub async fn withdraw(
        &self,
        contract_id: ContractId,
        preimage: Preimage,
    ) -> Result<(), EthereumError> {
        // TODO: this should be generated only once
        abigen!(HashedTimelock, "abi/HashedTimelock.json");

        let contract = HashedTimelock::new(self.eth_contract_address, self.client.clone().into());

        let tx = contract.withdraw(contract_id, preimage);
        tx.send().await.unwrap().await.unwrap();

        Ok(())
    }

    pub async fn refund(&self, contract_id: ContractId) -> Result<(), EthereumError> {
        // TODO: this should be generated only once
        abigen!(HashedTimelock, "abi/HashedTimelock.json");

        let contract = HashedTimelock::new(self.eth_contract_address, self.client.clone().into());

        let tx = contract.refund(contract_id);
        tx.send().await.unwrap().await.unwrap();

        Ok(())
    }

    pub async fn get_preimage(&self, contract_id: ContractId) -> Result<Preimage, EthereumError> {
        // TODO: this should be generated only once
        abigen!(HashedTimelock, "abi/HashedTimelock.json");

        let contract = HashedTimelock::new(self.eth_contract_address, self.client.clone().into());

        // We don't even need to submit a transaction into the network
        // as the "call" operation will result in a state read in the provider
        let res = contract.get_contract(contract_id).call().await.unwrap();

        // In the return type of the "get_contract" solidity method, the "preimage" field has index 7
        let preimage = res.7;
        Ok(preimage)
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

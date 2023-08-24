use std::str::FromStr;
use std::sync::Arc;

use crate::config::Config;
use crate::position_manager::PositionManager;
use anyhow::anyhow;
use anyhow::bail;
use ethereum::EthereumContractManager;
use ethers::types::Address;
use ethers::utils::hex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use tari::contract::TariContractManager;
use tari_crypto::ristretto::RistrettoPublicKey;
use tari_crypto::tari_utilities::hex::Hex;
use tari_template_lib::prelude::ComponentAddress;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type ContractId = String;
pub type Preimage = [u8; 32];
pub type Hashlock = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    client_address: String,
    hashlock: Hashlock,
    position: Position,
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct Position {
    pub provided_token: String,
    #[serde_as(as = "DisplayFromStr")]
    pub provided_token_balance: u64,
    pub requested_token: String,
    #[serde_as(as = "DisplayFromStr")]
    pub requested_token_balance: u64,
}

impl From<Position> for tari::liquidity::Position {
    fn from(pos: Position) -> Self {
        Self {
            provided_token: pos.provided_token,
            provided_token_balance: pos.provided_token_balance,
            requested_token: pos.requested_token,
            requested_token_balance: pos.requested_token_balance,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SwapState {
    NotStarted(Proposal),
    Pending(PendingSwap),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingSwap {
    client_contract_id: ContractId,
    our_contract_id: ContractId,
    proposal: Proposal,
}

pub type SwapId = Uuid;
type SwapKvMap = HashMap<SwapId, SwapState>;

pub struct SwapManager {
    config: Config,
    // TODO: use a database to not lose ongoing swap information on restarts
    swaps: Arc<RwLock<SwapKvMap>>,
    position_manager: PositionManager,
    eth_manager: EthereumContractManager,
    tari_manager: Arc<RwLock<TariContractManager>>,
}

impl SwapManager {
    pub fn new(
        config: Config,
        position_manager: PositionManager,
        eth_manager: EthereumContractManager,
        tari_manager: TariContractManager,
    ) -> Self {
        Self {
            config,
            swaps: Arc::new(RwLock::new(HashMap::new())),
            position_manager,
            eth_manager,
            tari_manager: Arc::new(RwLock::new(tari_manager)),
        }
    }

    pub async fn request_swap(
        &self,
        proposal: Proposal,
    ) -> Result<(SwapId, String), anyhow::Error> {
        if self
            .position_manager
            .is_swap_proposal_valid(&proposal.position.clone().into())
            .await
        {
            let swap_id = Uuid::new_v4();
            let provider_address = self.get_provider_address(&proposal)?;
            let swap_state = SwapState::NotStarted(proposal);
            let mut guard = self.swaps.write().await;
            guard.insert(swap_id, swap_state);
            Ok((swap_id, provider_address))
        } else {
            bail!("Invalid proposal");
        }
    }

    fn get_provider_address(&self, proposal: &Proposal) -> Result<String, anyhow::Error> {
        eprintln!("get_provider_address: {:?}", proposal);
        // TODO: create enums and parsing logic for each type of token
        match proposal.position.provided_token.as_str() {
            "eth.wei" => Ok(self.config.ethereum.account_address.clone()),
            "tari" => Ok(self.config.tari.account_component.clone()),
            _ => bail!("Invalid token type"),
        }
    }

    pub async fn request_lock_funds(
        &self,
        swap_id: String,
        contract_id: ContractId,
    ) -> Result<ContractId, anyhow::Error> {
        eprintln!("request_lock_funds");
        let swap_id = SwapId::from_str(&swap_id)?;
        eprintln!("swap_id: {}", swap_id);
        let swap_state = self.get_swap_state(&swap_id).await?;
        eprintln!("swap_state: {:?}", swap_state);

        let mut write_guard = self.swaps.write().await;

        match swap_state {
            SwapState::NotStarted(proposal) => {
                eprintln!("contract_id: {:?}", contract_id);
                self.validate_contract_id(&contract_id, &proposal).await?;
                let our_contract_id = self.create_lock_contract(&proposal).await?;
                write_guard.insert(
                    swap_id,
                    SwapState::Pending(PendingSwap {
                        client_contract_id: contract_id,
                        our_contract_id: our_contract_id.clone(),
                        proposal: proposal.clone(),
                    }),
                );
                Ok(our_contract_id)
            }
            _ => bail!("Funds alredy locked"),
        }
    }

    pub async fn push_preimage(
        &self,
        swap_id: String,
        preimage: Preimage,
    ) -> Result<(), anyhow::Error> {
        eprintln!("push_preimage");
        // TODO: we need a constant polling process watching the network to not rely on the client sending the preimage
        let swap_id = SwapId::from_str(&swap_id)?;
        eprintln!("swap_id: {}", swap_id);
        let swap_state = self.get_swap_state(&swap_id).await?;
        eprintln!("swap_state: {:?}", swap_state);

        let mut write_guard = self.swaps.write().await;

        match swap_state {
            SwapState::Pending(pending) => {
                eprintln!("preimage: {:?}", preimage);
                self.withdraw_funds(&pending, preimage).await?;
                write_guard.remove(&swap_id);
                // TODO: update published balances
                eprintln!("sucess");
                Ok(())
            }
            _ => bail!("Swap has not started yet"),
        }
    }

    async fn get_swap_state(&self, swap_id: &SwapId) -> Result<SwapState, anyhow::Error> {
        let read_guard = self.swaps.read().await;
        let state = read_guard
            .get(swap_id)
            .ok_or_else(|| anyhow!("Invalid swap_id"))?;
        Ok(state.to_owned())
    }

    async fn validate_contract_id(
        &self,
        _contract_id: &ContractId,
        _proposal: &Proposal,
    ) -> Result<(), anyhow::Error> {
        // TODO: implement on-chain validation of the contract id, to check that the client did lock the funds as expected

        Ok(())
    }

    async fn create_lock_contract(&self, proposal: &Proposal) -> Result<ContractId, anyhow::Error> {
        eprintln!("create_lock_contract: {:?}", proposal);
        // TODO: create enums and parsing logic for each type of token
        match proposal.position.requested_token.as_str() {
            "eth.wei" => {
                let amount_wei = proposal.position.requested_token_balance;
                let receiver = proposal.client_address.parse::<Address>()?;
                let hashlock = proposal.hashlock;
                // TODO: constant for timelocks
                let timelock = 100;
                let contract_id = self
                    .eth_manager
                    .new_contract(amount_wei.into(), receiver, hashlock, timelock)
                    .await?;
                Ok(hex::encode(contract_id))
            }
            "tari" => {
                let mut write_guard = self.tari_manager.write().await;
                let amount_tari: i64 = proposal.position.requested_token_balance.try_into()?;
                let receiver = RistrettoPublicKey::from_hex(&proposal.client_address)?;
                let hashlock = proposal.hashlock;
                // TODO: constant for timelocks
                let timelock = 100;
                let contract_id = write_guard
                    .create_lock_contract(amount_tari, receiver, hashlock, timelock)
                    .await?;
                Ok(contract_id.to_string())
            }
            _ => bail!("Invalid token type"),
        }
    }

    async fn withdraw_funds(
        &self,
        pending_swap: &PendingSwap,
        preimage: Preimage,
    ) -> Result<(), anyhow::Error> {
        eprintln!("withdraw_funds");
        // TODO: create enums and parsing logic for each type of token
        match pending_swap.proposal.position.provided_token.as_str() {
            "eth.wei" => {
                let contract_id_hex = &pending_swap.client_contract_id.trim_start_matches("0x");
                let contract_id: [u8; 32] = hex::decode(contract_id_hex)?
                    .try_into()
                    .map_err(|_| anyhow!("Invalid contract_id"))?;
                self.eth_manager.withdraw(contract_id, preimage).await?;
                Ok(())
            }
            "tari" => {
                eprintln!("tari");
                eprintln!("pending_swap: {:?}", pending_swap);
                let mut write_guard = self.tari_manager.write().await;
                let contract_id = ComponentAddress::from_str(&pending_swap.client_contract_id)?;
                eprintln!("contract_id: {}", contract_id);
                write_guard.withdraw(contract_id, preimage).await?;
                Ok(())
            }
            _ => bail!("Invalid token type"),
        }
    }
}

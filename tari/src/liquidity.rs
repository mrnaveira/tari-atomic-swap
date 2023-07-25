use std::collections::HashMap;

use crate::TariError;
use tari_crypto::ristretto::RistrettoPublicKey;
use tari_engine_types::commit_result::ExecuteResult;
use tari_engine_types::instruction::Instruction;
use tari_engine_types::substate::SubstateAddress;
use tari_template_lib::args;
use tari_template_lib::prelude::ComponentAddress;
use tari_template_lib::prelude::NonFungibleAddress;
use tari_template_lib::prelude::RistrettoPublicKeyBytes;
use tari_transaction::SubstateRequirement;
use tari_utilities::ByteArray;
use tari_wallet_daemon_client::types::TransactionSubmitRequest;
use tari_wallet_daemon_client::types::TransactionWaitResultRequest;
use tari_wallet_daemon_client::types::TransactionWaitResultResponse;
use tari_wallet_daemon_client::WalletDaemonClient;
use thiserror::Error;

// struct definition inside the "lp_position" template
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub provided_token: String,
    pub provided_token_balance: u64,
    pub requested_token: String,
    pub requested_token_balance: u64,
}

pub struct TariLiquidityManager {
    client: WalletDaemonClient,
    wallet_public_key: RistrettoPublicKey,
    wallet_public_key_index: u64,
    lp_index_component: ComponentAddress,
    lp_position_component: Option<ComponentAddress>,
}

impl TariLiquidityManager {
    pub fn new(
        wallet_endpoint: String,
        wallet_public_key: RistrettoPublicKey,
        wallet_public_key_index: u64,
        wallet_token: String,
        lp_index_component: ComponentAddress,
        lp_position_component: Option<ComponentAddress>,
    ) -> Result<Self, TariError> {
        let client = WalletDaemonClient::connect(wallet_endpoint, Some(wallet_token))?;

        Ok(Self {
            client,
            wallet_public_key,
            wallet_public_key_index,
            lp_index_component,
            lp_position_component,
        })
    }

    pub async fn get_providers(
        &mut self,
    ) -> Result<HashMap<NonFungibleAddress, ComponentAddress>, TariLiquidityManagerError> {
        let request = TransactionSubmitRequest {
            signing_key_index: None,
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: self.lp_index_component,
                method: "get_providers".to_string(),
                args: args![],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(self.lp_index_component),
                None,
            )],
            override_inputs: false,
            new_outputs: 0,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            // This is a readonly operation
            is_dry_run: true,
            proof_ids: vec![],
        };

        let result = self.submit_dry_run_transaction(request).await?;
        let providers = result.finalize.execution_results[0]
            .decode::<HashMap<NonFungibleAddress, ComponentAddress>>()
            .unwrap();
        Ok(providers)
    }

    pub async fn register(
        &mut self,
        network_address: String,
    ) -> Result<(), TariLiquidityManagerError> {
        if self.lp_position_component.is_some() {
            return Err(TariLiquidityManagerError::AlreadyRegistered);
        }

        let owner_token = Self::get_owner_token(&self.wallet_public_key);
        let request = TransactionSubmitRequest {
            signing_key_index: None,
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: self.lp_index_component,
                method: "register".to_string(),
                args: args![owner_token, network_address],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(self.lp_index_component),
                None,
            )],
            override_inputs: false,
            new_outputs: 0,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            is_dry_run: false,
            proof_ids: vec![],
        };

        let result = self.submit_transaction(request).await?;
        let lp_position_component = result.result.unwrap().execution_results[0]
            .decode::<ComponentAddress>()
            .unwrap();
        self.lp_position_component = Some(lp_position_component);

        Ok(())
    }

    pub async fn get_positions(&mut self) -> Result<Vec<Position>, TariLiquidityManagerError> {
        let lp_position_component = self
            .lp_position_component
            .ok_or(TariLiquidityManagerError::Unregistered)?;

        let request = TransactionSubmitRequest {
            signing_key_index: None,
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: lp_position_component,
                method: "get_positions".to_string(),
                args: args![],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(lp_position_component),
                None,
            )],
            override_inputs: false,
            new_outputs: 0,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            // This is a readonly operation
            is_dry_run: true,
            proof_ids: vec![],
        };

        let result = self.submit_dry_run_transaction(request).await?;
        let positions = result.finalize.execution_results[0]
            .decode::<Vec<Position>>()
            .unwrap();
        Ok(positions)
    }

    pub async fn add_position(
        &mut self,
        position: Position,
    ) -> Result<(), TariLiquidityManagerError> {
        let lp_position_component = self
            .lp_position_component
            .ok_or(TariLiquidityManagerError::Unregistered)?;

        let request = TransactionSubmitRequest {
            signing_key_index: Some(self.wallet_public_key_index),
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: lp_position_component,
                method: "add_position".to_string(),
                args: args![position],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(lp_position_component),
                None,
            )],
            override_inputs: false,
            new_outputs: 0,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            is_dry_run: false,
            proof_ids: vec![],
        };

        self.submit_transaction(request).await?;

        Ok(())
    }

    pub async fn remove_position(&mut self, index: usize) -> Result<(), TariLiquidityManagerError> {
        let lp_position_component = self
            .lp_position_component
            .ok_or(TariLiquidityManagerError::Unregistered)?;

        let request = TransactionSubmitRequest {
            signing_key_index: Some(self.wallet_public_key_index),
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: lp_position_component,
                method: "remove_position".to_string(),
                args: args![index],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(lp_position_component),
                None,
            )],
            override_inputs: false,
            new_outputs: 0,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            is_dry_run: false,
            proof_ids: vec![],
        };

        self.submit_transaction(request).await?;

        Ok(())
    }

    pub async fn replace_positions(
        &mut self,
        new_positions: Vec<Position>,
    ) -> Result<(), TariLiquidityManagerError> {
        let lp_position_component = self
            .lp_position_component
            .ok_or(TariLiquidityManagerError::Unregistered)?;

        let request = TransactionSubmitRequest {
            signing_key_index: Some(self.wallet_public_key_index),
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: lp_position_component,
                method: "replace_positions".to_string(),
                args: args![new_positions],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(lp_position_component),
                None,
            )],
            override_inputs: false,
            new_outputs: 0,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            is_dry_run: false,
            proof_ids: vec![],
        };

        self.submit_transaction(request).await?;

        Ok(())
    }

    pub async fn set_network_address(
        &mut self,
        network_address: String,
    ) -> Result<(), TariLiquidityManagerError> {
        let lp_position_component = self
            .lp_position_component
            .ok_or(TariLiquidityManagerError::Unregistered)?;

        let request = TransactionSubmitRequest {
            signing_key_index: Some(self.wallet_public_key_index),
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: lp_position_component,
                method: "set_network_address".to_string(),
                args: args![network_address],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(lp_position_component),
                None,
            )],
            override_inputs: false,
            new_outputs: 0,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            is_dry_run: false,
            proof_ids: vec![],
        };

        self.submit_transaction(request).await?;

        Ok(())
    }

    pub async fn get_network_address(&mut self) -> Result<String, TariLiquidityManagerError> {
        let lp_position_component = self
            .lp_position_component
            .ok_or(TariLiquidityManagerError::Unregistered)?;

        let request = TransactionSubmitRequest {
            signing_key_index: None,
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: lp_position_component,
                method: "get_network_address".to_string(),
                args: args![],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(lp_position_component),
                None,
            )],
            override_inputs: false,
            new_outputs: 0,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            // This is a readonly operation
            is_dry_run: true,
            proof_ids: vec![],
        };

        let result = self.submit_dry_run_transaction(request).await?;
        let network_address = result.finalize.execution_results[0]
            .decode::<String>()
            .unwrap();
        Ok(network_address)
    }

    async fn submit_transaction(
        &mut self,
        request: TransactionSubmitRequest,
    ) -> Result<TransactionWaitResultResponse, TariError> {
        let resp = self.client.submit_transaction(&request).await?;
        let wait_resp = self
            .client
            .wait_transaction_result(TransactionWaitResultRequest {
                transaction_id: resp.transaction_id,
                timeout_secs: None,
            })
            .await?;
        if wait_resp.timed_out {
            return Err(TariError::TransactionTimeout {
                transaction_id: resp.transaction_id,
            });
        }
        Ok(wait_resp)
    }

    async fn submit_dry_run_transaction(
        &mut self,
        request: TransactionSubmitRequest,
    ) -> Result<ExecuteResult, TariError> {
        let resp = self.client.submit_transaction(&request).await?;
        let result = resp.result.unwrap();
        Ok(result)
    }

    fn get_owner_token(public_key: &RistrettoPublicKey) -> NonFungibleAddress {
        NonFungibleAddress::from_public_key(
            RistrettoPublicKeyBytes::from_bytes(public_key.as_bytes()).unwrap(),
        )
    }
}

#[derive(Error, Debug)]
pub enum TariLiquidityManagerError {
    #[error("Tari error: {0}")]
    TariError(#[from] TariError),
    #[error("The provider is already registered")]
    AlreadyRegistered,
    #[error("The provider is not registered")]
    Unregistered,
}

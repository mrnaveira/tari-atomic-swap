use sha2::Digest;
use sha2::Sha256;
use tari_crypto::ristretto::RistrettoPublicKey;
use tari_engine_types::commit_result::ExecuteResult;
use tari_engine_types::component::new_component_address_from_parts;
use tari_engine_types::instruction::Instruction;
use tari_engine_types::substate::SubstateAddress;
use tari_template_builtin::ACCOUNT_TEMPLATE_ADDRESS;
use tari_template_lib::args;
use tari_template_lib::prelude::Amount;
use tari_template_lib::prelude::ComponentAddress;
use tari_template_lib::prelude::NonFungibleAddress;
use tari_template_lib::prelude::RistrettoPublicKeyBytes;
use tari_template_lib::prelude::TemplateAddress;
use tari_template_lib::prelude::CONFIDENTIAL_TARI_RESOURCE_ADDRESS;
use tari_template_lib::Hash;
use tari_transaction::SubstateRequirement;
use tari_transaction::TransactionId;
use tari_utilities::ByteArray;
use tari_wallet_daemon_client::error::WalletDaemonClientError;
use tari_wallet_daemon_client::types::TransactionSubmitRequest;
use tari_wallet_daemon_client::types::TransactionSubmitResponse;
use tari_wallet_daemon_client::types::TransactionWaitResultRequest;
use tari_wallet_daemon_client::WalletDaemonClient;
use thiserror::Error;

type ByteArray32 = [u8; 32];
pub type Preimage = ByteArray32;
pub type Hashlock = ByteArray32;

pub struct TariContractManager {
    client: WalletDaemonClient,
    wallet_public_key: RistrettoPublicKey,
    wallet_address: ComponentAddress,
    swap_template_address: TemplateAddress,
}

impl TariContractManager {
    pub fn new(
        wallet_endpoint: String,
        wallet_public_key: RistrettoPublicKey,
        wallet_token: String,
        swap_template_address: TemplateAddress,
    ) -> Result<Self, TariError> {
        let client = WalletDaemonClient::connect(wallet_endpoint, Some(wallet_token))?;
        let wallet_address = Self::get_account_address(&wallet_public_key);

        Ok(Self {
            client,
            wallet_public_key,
            wallet_address,
            swap_template_address,
        })
    }

    // TODO: DRY up with the similar method in the Ethereum crate
    pub fn create_hashlock(preimage: Preimage) -> Hashlock {
        let mut hasher = Sha256::new();
        hasher.update(preimage);
        hasher.finalize().into()
    }

    pub async fn create_lock_contract(
        &mut self,
        amount: i64,
        receiver_public_key: RistrettoPublicKey,
        hashlock: Hashlock,
        timelock: u64,
    ) -> Result<ComponentAddress, TariError> {
        let receiver_account = Self::get_account_address(&receiver_public_key);
        let receiver_owner_token = Self::get_owner_token(&receiver_public_key);
        let sender_owner_token = Self::get_owner_token(&self.wallet_public_key);
        let request = TransactionSubmitRequest {
            // use the default signing key of the wallet
            signing_key_index: None,
            fee_instructions: vec![],
            instructions: vec![
                Instruction::CallMethod {
                    component_address: self.wallet_address,
                    method: "withdraw".to_string(),
                    // TODO: parameterize the resource to swap
                    args: args![*CONFIDENTIAL_TARI_RESOURCE_ADDRESS, Amount(amount)],
                },
                Instruction::PutLastInstructionOutputOnWorkspace {
                    key: b"bucket".to_vec(),
                },
                Instruction::CallFunction {
                    template_address: self.swap_template_address,
                    function: "create".to_string(),
                    args: args![
                        Variable("bucket"),
                        sender_owner_token,
                        receiver_owner_token,
                        hashlock,
                        timelock
                    ],
                },
            ],
            // the inputs are the sender and receiver account addresses
            inputs: vec![
                SubstateRequirement::new(SubstateAddress::Component(self.wallet_address), None),
                SubstateRequirement::new(SubstateAddress::Component(receiver_account), None),
            ],
            override_inputs: false,
            // we are creating a component with a vault
            new_outputs: 2,
            specific_non_fungible_outputs: vec![],
            new_resources: vec![],
            new_non_fungible_outputs: vec![],
            new_non_fungible_index_outputs: vec![],
            is_dry_run: false,
            proof_ids: vec![],
        };

        let response = self.submit_transaction(request).await?;

        let component = response.result.unwrap().finalize.execution_results[2]
            .decode::<ComponentAddress>()
            .unwrap();
        Ok(component)
    }

    pub async fn withdraw_funds(
        &mut self,
        contract: ComponentAddress,
        preimage: [u8; 32],
    ) -> Result<(), TariError> {
        let request = TransactionSubmitRequest {
            // use the default signing key of the wallet
            signing_key_index: None,
            fee_instructions: vec![],
            instructions: vec![
                Instruction::CallMethod {
                    component_address: contract,
                    method: "withdraw".to_string(),
                    args: args![preimage],
                },
                Instruction::PutLastInstructionOutputOnWorkspace {
                    key: b"bucket".to_vec(),
                },
                Instruction::CallMethod {
                    component_address: self.wallet_address,
                    method: "deposit".to_string(),
                    args: args![Variable("bucket")],
                },
            ],
            inputs: vec![
                SubstateRequirement::new(SubstateAddress::Component(contract), None),
                SubstateRequirement::new(SubstateAddress::Component(self.wallet_address), None),
            ],
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

    pub async fn refund(&mut self, contract: ComponentAddress) -> Result<(), TariError> {
        let request = TransactionSubmitRequest {
            // use the default signing key of the wallet
            signing_key_index: None,
            fee_instructions: vec![],
            instructions: vec![
                Instruction::CallMethod {
                    component_address: contract,
                    method: "refund".to_string(),
                    args: args![],
                },
                Instruction::PutLastInstructionOutputOnWorkspace {
                    key: b"bucket".to_vec(),
                },
                Instruction::CallMethod {
                    component_address: self.wallet_address,
                    method: "deposit".to_string(),
                    args: args![Variable("bucket")],
                },
            ],
            inputs: vec![
                SubstateRequirement::new(SubstateAddress::Component(contract), None),
                SubstateRequirement::new(SubstateAddress::Component(self.wallet_address), None),
            ],
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

    pub async fn get_preimage(
        &mut self,
        contract: ComponentAddress,
    ) -> Result<Option<Preimage>, TariError> {
        let request = TransactionSubmitRequest {
            // use the default signing key of the wallet
            signing_key_index: None,
            fee_instructions: vec![],
            instructions: vec![Instruction::CallMethod {
                component_address: contract,
                method: "get_preimage".to_string(),
                args: args![],
            }],
            inputs: vec![SubstateRequirement::new(
                SubstateAddress::Component(contract),
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
        let preimage = result.finalize.execution_results[0]
            .decode::<Option<Preimage>>()
            .unwrap();
        Ok(preimage)
    }

    async fn submit_transaction(
        &mut self,
        request: TransactionSubmitRequest,
    ) -> Result<TransactionSubmitResponse, TariError> {
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

        Ok(resp)
    }

    async fn submit_dry_run_transaction(
        &mut self,
        request: TransactionSubmitRequest,
    ) -> Result<ExecuteResult, TariError> {
        let resp = self.client.submit_transaction(&request).await?;
        let result = resp.result.unwrap();
        Ok(result)
    }

    fn get_account_address(public_key: &RistrettoPublicKey) -> ComponentAddress {
        let component_id = Hash::try_from(public_key.as_bytes()).unwrap();
        new_component_address_from_parts(&ACCOUNT_TEMPLATE_ADDRESS, &component_id)
    }

    fn get_owner_token(public_key: &RistrettoPublicKey) -> NonFungibleAddress {
        NonFungibleAddress::from_public_key(
            RistrettoPublicKeyBytes::from_bytes(public_key.as_bytes()).unwrap(),
        )
    }
}

#[derive(Error, Debug)]
pub enum TariError {
    #[error("Wallet error: {0}")]
    WalletError(#[from] WalletDaemonClientError),
    #[error("Transaction timeout for id: {transaction_id}")]
    TransactionTimeout { transaction_id: TransactionId },
}

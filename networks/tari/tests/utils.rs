use anyhow::anyhow;
use std::sync::RwLock;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};
use tari_crypto::{
    ristretto::RistrettoSecretKey,
    tari_utilities::{hex::Hex, ByteArray},
};
use tari_dan_common_types::{
    crypto::create_key_pair, services::template_provider::TemplateProvider,
};
use tari_dan_engine::{
    bootstrap_state,
    fees::{FeeModule, FeeTable},
    packager::{LoadedTemplate, Package, TemplateModuleLoader},
    runtime::{AuthParams, ConsensusContext, RuntimeModule, RuntimeModuleError, StateTracker},
    state_store::{memory::MemoryStateStore, AtomicDb, StateWriter},
    transaction::{TransactionError, TransactionProcessor},
    wasm::{compile::compile_template, WasmModule},
};
use tari_engine_types::{
    commit_result::ExecuteResult,
    hashing::template_hasher,
    instruction::Instruction,
    substate::{SubstateAddress, SubstateDiff},
};
use tari_template_builtin::{get_template_builtin, ACCOUNT_TEMPLATE_ADDRESS};
use tari_template_lib::{
    args,
    crypto::RistrettoPublicKeyBytes,
    models::{Amount, ComponentAddress, NonFungibleAddress, TemplateAddress},
};
use tari_transaction::Transaction;

pub struct TemplateTest {
    package: Arc<Package>,
    track_calls: TrackCallsModule,
    secret_key: RistrettoSecretKey,
    last_outputs: HashSet<SubstateAddress>,
    name_to_template: HashMap<String, TemplateAddress>,
    state_store: MemoryStateStore,
    // TODO: cleanup
    consensus_context: ConsensusContext,
    enable_fees: bool,
    fee_table: FeeTable,
}

impl TemplateTest {
    pub fn new<I: IntoIterator<Item = P>, P: AsRef<Path>>(template_paths: I) -> Self {
        let secret_key = RistrettoSecretKey::from_hex(
            "7e100429f979d37999f051e65b94734e206925e9346759fd73caafb2f3232578",
        )
        .unwrap();

        let mut name_to_template = HashMap::new();
        let mut builder = Package::builder();

        // Add Account template builtin
        let wasm = get_template_builtin(*ACCOUNT_TEMPLATE_ADDRESS);
        let template = WasmModule::from_code(wasm.to_vec())
            .load_template()
            .unwrap();
        builder.add_template(*ACCOUNT_TEMPLATE_ADDRESS, template);
        name_to_template.insert("Account".to_string(), *ACCOUNT_TEMPLATE_ADDRESS);

        let wasms = template_paths
            .into_iter()
            .map(|path| compile_template(path, &[]).unwrap());

        for wasm in wasms {
            let template_addr = template_hasher().chain(wasm.code()).result();
            let wasm = wasm.load_template().unwrap();
            let name = wasm.template_name().to_string();
            name_to_template.insert(name, template_addr);
            builder.add_template(template_addr, wasm);
        }
        let package = builder.build();
        let state_store = MemoryStateStore::default();
        {
            let mut tx = state_store.write_access().unwrap();
            bootstrap_state(&mut tx).unwrap();
            tx.commit().unwrap();
        }

        Self {
            package: Arc::new(package),
            track_calls: TrackCallsModule::new(),
            secret_key,
            name_to_template,
            last_outputs: HashSet::new(),
            state_store,
            // TODO: cleanup
            consensus_context: ConsensusContext { current_epoch: 0 },
            enable_fees: false,
            fee_table: FeeTable::new(1, 1),
        }
    }

    #[allow(dead_code)]
    pub fn set_consensus_context(&mut self, consensus: ConsensusContext) -> &mut Self {
        self.consensus_context = consensus;
        self
    }

    fn commit_diff(&mut self, diff: &SubstateDiff) {
        self.last_outputs.clear();
        let mut tx = self.state_store.write_access().unwrap();

        for (address, _) in diff.down_iter() {
            eprintln!("DOWN substate: {}", address);
            tx.delete_state(address).unwrap();
        }

        for (address, substate) in diff.up_iter() {
            eprintln!("UP substate: {}", address);
            self.last_outputs.insert(address.clone());
            tx.set_state(address, substate).unwrap();
        }

        tx.commit().unwrap();
    }

    pub fn get_template_address(&self, name: &str) -> TemplateAddress {
        *self
            .name_to_template
            .get(name)
            .unwrap_or_else(|| panic!("No template with name {}", name))
    }

    pub fn create_owned_account(
        &mut self,
    ) -> (ComponentAddress, NonFungibleAddress, RistrettoSecretKey) {
        let (owner_proof, secret_key) = self.create_owner_proof();
        let old_fail_fees = self.enable_fees;
        self.enable_fees = false;
        let result = self
            .execute_and_commit(
                vec![
                    Instruction::CreateFreeTestCoins {
                        revealed_amount: Amount(100000),
                        output: None,
                    },
                    Instruction::PutLastInstructionOutputOnWorkspace {
                        key: b"free_test_coins".to_vec(),
                    },
                    Instruction::CallFunction {
                        template_address: self.get_template_address("Account"),
                        function: "create_with_bucket".to_owned(),
                        args: args![&owner_proof, Workspace("free_test_coins")],
                    },
                ],
                vec![owner_proof.clone()],
            )
            .unwrap();

        result.expect_success();

        let component = result.finalize.execution_results[2]
            .decode::<ComponentAddress>()
            .unwrap();

        self.enable_fees = old_fail_fees;
        (component, owner_proof, secret_key)
    }

    pub fn create_owner_proof(&self) -> (NonFungibleAddress, RistrettoSecretKey) {
        let (secret_key, public_key) = create_key_pair();
        let public_key = RistrettoPublicKeyBytes::from_bytes(public_key.as_bytes()).unwrap();
        let owner_token = NonFungibleAddress::from_public_key(public_key);
        (owner_token, secret_key)
    }

    pub fn try_execute_instructions(
        &mut self,
        fee_instructions: Vec<Instruction>,
        instructions: Vec<Instruction>,
        proofs: Vec<NonFungibleAddress>,
    ) -> Result<ExecuteResult, TransactionError> {
        let transaction = Transaction::builder()
            .with_fee_instructions(fee_instructions)
            .with_instructions(instructions)
            .sign(&self.secret_key)
            .build();

        self.try_execute(transaction, proofs)
    }

    pub fn try_execute(
        &mut self,
        transaction: Transaction,
        proofs: Vec<NonFungibleAddress>,
    ) -> Result<ExecuteResult, TransactionError> {
        let mut modules: Vec<Arc<dyn RuntimeModule<Package>>> =
            vec![Arc::new(self.track_calls.clone())];

        if self.enable_fees {
            modules.push(Arc::new(FeeModule::new(0, self.fee_table.clone())));
        }

        let auth_params = AuthParams {
            initial_ownership_proofs: proofs,
        };
        let processor = TransactionProcessor::new(
            self.package.clone(),
            self.state_store.clone(),
            auth_params,
            self.consensus_context.clone(),
            modules,
        );

        let result = processor.execute(transaction)?;

        if self.enable_fees {
            if let Some(ref fee) = result.fee_receipt {
                eprintln!("Fee: {}", fee.total_fees_charged());
                eprintln!("Paid: {}", fee.total_fees_paid());
                eprintln!("Refund: {}", fee.total_refunded());
                eprintln!("Unpaid: {}", fee.unpaid_debt());
                for (source, amt) in &fee.cost_breakdown {
                    eprintln!("- {:?} {}", source, amt);
                }
            }
        }

        Ok(result)
    }

    pub fn execute_and_commit(
        &mut self,
        instructions: Vec<Instruction>,
        proofs: Vec<NonFungibleAddress>,
    ) -> anyhow::Result<ExecuteResult> {
        self.execute_and_commit_with_fees(vec![], instructions, proofs)
    }

    pub fn execute_and_commit_with_fees(
        &mut self,
        fee_instructions: Vec<Instruction>,
        instructions: Vec<Instruction>,
        proofs: Vec<NonFungibleAddress>,
    ) -> anyhow::Result<ExecuteResult> {
        let result = self.try_execute_instructions(fee_instructions, instructions, proofs)?;
        let diff = result.finalize.result.accept().ok_or_else(|| {
            anyhow!(
                "Transaction was rejected: {}",
                result.finalize.result.reject().unwrap()
            )
        })?;

        // It is convenient to commit the state back to the staged state store in tests.
        self.commit_diff(diff);

        if let Some(reason) = result.transaction_failure {
            return Err(anyhow!("Transaction failed: {}", reason));
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct TrackCallsModule {
    calls: Arc<RwLock<Vec<&'static str>>>,
}

impl TrackCallsModule {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(RwLock::new(Vec::new())),
        }
    }

    #[allow(dead_code)]
    pub fn get(&self) -> Vec<&'static str> {
        self.calls.read().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        self.calls.write().unwrap().clear();
    }
}

impl<TTemplateProvider: TemplateProvider<Template = LoadedTemplate>>
    RuntimeModule<TTemplateProvider> for TrackCallsModule
{
    fn on_runtime_call(
        &self,
        _tracker: &StateTracker<TTemplateProvider>,
        call: &'static str,
    ) -> Result<(), RuntimeModuleError> {
        self.calls.write().unwrap().push(call);
        Ok(())
    }
}

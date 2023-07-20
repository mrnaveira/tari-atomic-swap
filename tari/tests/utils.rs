use anyhow::anyhow;
use serde::de::DeserializeOwned;
use std::sync::RwLock;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};
use tari_bor::encode;
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
    state_store::{
        memory::{MemoryStateStore, MemoryWriteTransaction},
        AtomicDb, StateReader, StateStoreError, StateWriter,
    },
    transaction::{TransactionError, TransactionProcessor},
    wasm::{compile::compile_template, LoadedWasmTemplate, WasmModule},
};
use tari_engine_types::{
    commit_result::ExecuteResult,
    component::{ComponentBody, ComponentHeader},
    hashing::template_hasher,
    instruction::Instruction,
    resource_container::ResourceContainer,
    substate::{Substate, SubstateAddress, SubstateDiff},
    vault::Vault,
};
use tari_template_builtin::{get_template_builtin, ACCOUNT_TEMPLATE_ADDRESS};
use tari_template_lib::{
    args,
    args::Arg,
    crypto::RistrettoPublicKeyBytes,
    models::{Amount, ComponentAddress, NonFungibleAddress, TemplateAddress},
    prelude::{AccessRules, CONFIDENTIAL_TARI_RESOURCE_ADDRESS},
    Hash,
};
use tari_transaction::{id_provider::IdProvider, Transaction};
use tari_transaction_manifest::{parse_manifest, ManifestValue};

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

    pub fn enable_fees(&mut self) -> &mut Self {
        self.enable_fees = true;
        self
    }

    pub fn disable_fees(&mut self) -> &mut Self {
        self.enable_fees = false;
        self
    }

    pub fn fee_table(&self) -> &FeeTable {
        &self.fee_table
    }

    pub fn set_fee_table(&mut self, fee_table: FeeTable) -> &mut Self {
        self.fee_table = fee_table;
        self
    }

    pub fn set_consensus_context(&mut self, consensus: ConsensusContext) -> &mut Self {
        self.consensus_context = consensus;
        self
    }

    pub fn read_only_state_store(&self) -> ReadOnlyStateStore {
        ReadOnlyStateStore::new(self.state_store.clone())
    }

    pub fn default_signing_key(&self) -> &RistrettoSecretKey {
        &self.secret_key
    }

    pub fn assert_calls(&self, expected: &[&'static str]) {
        let calls = self.track_calls.get();
        assert_eq!(calls, expected);
    }

    pub fn clear_calls(&self) {
        self.track_calls.clear();
    }

    pub fn get_previous_output_address(&self, ty: SubstateType) -> SubstateAddress {
        self.last_outputs
            .iter()
            .find(|addr| ty.matches(addr))
            .cloned()
            .unwrap_or_else(|| panic!("No output of type {:?}", ty))
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

    pub fn get_module(&self, module_name: &str) -> &LoadedWasmTemplate {
        let addr = self.name_to_template.get(module_name).unwrap();
        match self.package.get_template_by_address(addr).unwrap() {
            LoadedTemplate::Wasm(wasm) => wasm,
            LoadedTemplate::Flow(_) => {
                panic!("Not supported")
            }
        }
    }

    pub fn get_template_address(&self, name: &str) -> TemplateAddress {
        *self
            .name_to_template
            .get(name)
            .unwrap_or_else(|| panic!("No template with name {}", name))
    }

    pub fn call_function<T>(
        &mut self,
        template_name: &str,
        func_name: &str,
        args: Vec<Arg>,
        proofs: Vec<NonFungibleAddress>,
    ) -> T
    where
        T: DeserializeOwned,
    {
        let result = self
            .execute_and_commit(
                vec![Instruction::CallFunction {
                    template_address: self.get_template_address(template_name),
                    function: func_name.to_owned(),
                    args,
                }],
                proofs,
            )
            .unwrap();
        result.finalize.execution_results[0].decode().unwrap()
    }

    pub fn call_method<T>(
        &mut self,
        component_address: ComponentAddress,
        method_name: &str,
        args: Vec<Arg>,
        proofs: Vec<NonFungibleAddress>,
    ) -> T
    where
        T: DeserializeOwned,
    {
        let result = self
            .execute_and_commit(
                vec![Instruction::CallMethod {
                    component_address,
                    method: method_name.to_owned(),
                    args,
                }],
                proofs,
            )
            .unwrap();

        result.finalize.execution_results[0].decode().unwrap()
    }

    pub fn create_empty_account(
        &mut self,
    ) -> (ComponentAddress, NonFungibleAddress, RistrettoSecretKey) {
        let (owner_proof, secret_key) = self.create_owner_proof();
        let old_fail_fees = self.enable_fees;
        self.enable_fees = false;
        let component = self.call_function(
            "Account",
            "create",
            args![owner_proof],
            vec![owner_proof.clone()],
        );
        self.enable_fees = old_fail_fees;
        (component, owner_proof, secret_key)
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

    pub fn try_execute_and_commit(
        &mut self,
        transaction: Transaction,
        proofs: Vec<NonFungibleAddress>,
    ) -> Result<ExecuteResult, TransactionError> {
        let result = self.try_execute(transaction, proofs)?;
        if let Some(diff) = result.finalize.result.accept() {
            self.commit_diff(diff);
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

    pub fn execute_and_commit_manifest<'a, I: IntoIterator<Item = (&'a str, ManifestValue)>>(
        &mut self,
        manifest: &str,
        variables: I,
        proofs: Vec<NonFungibleAddress>,
    ) -> anyhow::Result<ExecuteResult> {
        let template_imports = self
            .name_to_template
            .iter()
            // Account is implicitly imported.
            .filter(|(name, _)| *name != "Account")
            .map(|(name, addr)| format!("use template_{} as {};", addr, name))
            .collect::<Vec<_>>()
            .join("\n");
        let manifest = format!("{} fn main() {{ {} }}", template_imports, manifest);
        let instructions = parse_manifest(
            &manifest,
            variables
                .into_iter()
                .map(|(a, b)| (a.to_string(), b))
                .collect(),
        )
        .unwrap();
        self.execute_and_commit(instructions, proofs)
    }
}

pub struct ReadOnlyStateStore {
    store: MemoryStateStore,
}
impl ReadOnlyStateStore {
    pub fn new(store: MemoryStateStore) -> Self {
        Self { store }
    }

    pub fn get_component(
        &self,
        component_address: ComponentAddress,
    ) -> Result<ComponentHeader, StateStoreError> {
        let substate = self.get_substate(&SubstateAddress::Component(component_address))?;
        Ok(substate.into_substate_value().into_component().unwrap())
    }

    pub fn get_substate(&self, address: &SubstateAddress) -> Result<Substate, StateStoreError> {
        let tx = self.store.read_access()?;
        let substate = tx.get_state::<_, Substate>(address)?;
        Ok(substate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubstateType {
    Component,
    Resource,
    Vault,
    NonFungible,
    NonFungibleIndex,
}

impl SubstateType {
    pub fn matches(&self, addr: &SubstateAddress) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, addr) {
            (SubstateType::Component, SubstateAddress::Component(_)) => true,
            (SubstateType::Resource, SubstateAddress::Resource(_)) => true,
            (SubstateType::Vault, SubstateAddress::Vault(_)) => true,
            (SubstateType::NonFungible, SubstateAddress::NonFungible(_)) => true,
            (SubstateType::NonFungibleIndex, SubstateAddress::NonFungibleIndex(_)) => true,
            _ => false,
        }
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

    pub fn get(&self) -> Vec<&'static str> {
        self.calls.read().unwrap().clone()
    }

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
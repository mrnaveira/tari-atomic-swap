use tari_template_lib::prelude::*;
use tari_template_abi::rust::collections::HashMap;

#[template]
mod lp_index_template {
    use super::*;

    pub struct LiquidityProviderIndex {
        lp_position_template: TemplateAddress,
        providers: HashMap<NonFungibleAddress, ComponentAddress>,
    }

    impl LiquidityProviderIndex {
        pub fn new(lp_position_template: TemplateAddress) -> Self {
            Self {
                lp_position_template,
                providers: HashMap::new()
            }
        }

        pub fn register(&mut self, owner_token: NonFungibleAddress, network_address: String) -> ComponentAddress {
            // sanity check of the owner_token 
            owner_token
                .to_public_key()
                .unwrap_or_else(|| panic!("owner_token is not a valid public key: {}", owner_token));

            assert!(!self.providers.contains_key(&owner_token), "The provider is already registered");

            let lp_position_address = TemplateManager::get(self.lp_position_template)
                .call("new".to_string(), invoke_args![owner_token, network_address]);
            self.providers.insert(owner_token, lp_position_address);

            lp_position_address
        }

        pub fn get_providers(&self) -> HashMap<NonFungibleAddress, ComponentAddress> {
            self.providers.clone()
        }
    }
}

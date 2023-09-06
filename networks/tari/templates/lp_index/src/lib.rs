use tari_template_lib::prelude::*;
use tari_template_abi::rust::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub provided_token: String,
    pub provided_token_balance: u64,
    pub requested_token: String,
    pub requested_token_balance: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderPosition {
    network_address: String,
    owner_token: NonFungibleAddress,
    positions: Vec<Position>,
}

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

        pub fn get_all_provider_positions(&self) -> Vec<ProviderPosition> {
            let mut provider_positions = vec![];

            for (owner_token, component_addr) in &self.providers {
                let component = ComponentManager::get(*component_addr);

                let network_address: String = component.call("get_network_address".to_string(), vec![]);
                let positions: Vec<Position> = component.call("get_positions".to_string(), vec![]);
                
                provider_positions.push(ProviderPosition {
                    owner_token: owner_token.clone(),
                    network_address,
                    positions
                });
            }

            return provider_positions;
        }
    }
}

use tari_template_lib::prelude::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub provided_token: String,
    pub provided_token_balance: u64,
    pub requested_token: String,
    pub requested_token_balance: u64,
}

#[template]
mod lp_position_template {
    use super::*;

    pub struct LiquidityProviderPosition {
        network_address: String,
        owner_token: NonFungibleAddress,
        positions: Vec<Position>,
    }

    impl LiquidityProviderPosition {
        pub fn new(owner_token: NonFungibleAddress, network_address: String) -> LiquidityProviderPositionComponent {
            let owner_only = AccessRule::Restricted(Require(owner_token.clone()));
            let rules = AccessRules::new()
                // anyone can read the info the in component 
                .add_method_rule("get_network_address", AccessRule::AllowAll)
                .add_method_rule("get_owner_token", AccessRule::AllowAll)
                .add_method_rule("get_positions", AccessRule::AllowAll)
                // only the owner can update the componet info
                .default(owner_only);

            Self {
                network_address,
                owner_token,
                positions: Vec::new()
            }.create_with_options(rules, None)
        }

        pub fn get_network_address(&self) -> String {
            self.network_address.clone()
        }

        pub fn set_network_address(&mut self, new_network_address: String) {
            self.network_address = new_network_address;
        }

        pub fn get_owner_token(&self) -> NonFungibleAddress {
            self.owner_token.clone()
        }

        pub fn get_positions(&self) -> Vec<Position> {
            self.positions.clone()
        }

        pub fn add_position(&mut self, position: Position) {
            self.positions.push(position);
        }

        pub fn remove_position(&mut self, index: usize) {
            self.positions.remove(index);
        }
    }
}

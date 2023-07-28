use std::str::FromStr;

use tari::liquidity::TariLiquidityManager;
use tari_crypto::{ristretto::RistrettoPublicKey, tari_utilities::hex::Hex};
use tari_template_lib::prelude::ComponentAddress;

use crate::config::Config;

pub struct PositionManager {
    config: Config,
    tari_manager: TariLiquidityManager,
}

impl PositionManager {
    pub fn new(config: Config) -> Result<Self, anyhow::Error> {
        let wallet_public_key = RistrettoPublicKey::from_hex(&config.tari.public_key)?;
        let lp_index_component = ComponentAddress::from_str(&config.tari.liquidity_component)?;

        let tari_manager = TariLiquidityManager::new(
            config.tari.wallet_endpoint.clone(),
            wallet_public_key,
            config.tari.public_key_index,
            config.tari.wallet_token.clone(),
            lp_index_component,
            None,
        )?;

        Ok(Self {
            config,
            tari_manager,
        })
    }

    pub async fn sync(&mut self) -> Result<(), anyhow::Error> {
        let config_network_address = self.config.network_address.clone();
        let config_positions = self.config.positions.clone();

        if self.is_registered() {
            // we are registered

            // we need to update the network address if it changed in the config
            let published_network_address = self.tari_manager.get_network_address().await?;
            if published_network_address != config_network_address {
                self.tari_manager
                    .set_network_address(config_network_address)
                    .await?;
            }

            // we also need to update the positions if they changed in the config
            let published_positions = self.tari_manager.get_positions().await?;
            if published_positions != config_positions {
                self.tari_manager
                    .replace_positions(config_positions)
                    .await?;
            }
        } else {
            // we are not registered
            // if the user has specified some positions, we need to register and publish them
            if !self.config.positions.is_empty() {
                self.tari_manager.register(config_network_address).await?;
                self.tari_manager
                    .replace_positions(config_positions)
                    .await?;
            }
        }

        Ok(())
    }

    fn is_registered(&mut self) -> bool {
        self.tari_manager.lp_position_component.is_some()
    }
}

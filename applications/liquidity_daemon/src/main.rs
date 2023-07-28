use crate::config::Config;
use position_manager::PositionManager;
use tokio::signal;

mod config;
mod position_manager;

#[tokio::main]
async fn main() {
    let config = Config::read("./config.json".to_string());

    let mut position_manager =
        PositionManager::new(config).expect("Could not create position manager");
    position_manager
        .sync()
        .await
        .expect("Could not sync position manager with the Tari network");

    match signal::ctrl_c().await {
        Ok(()) => {
            eprintln!("Shutdown signal received");
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}

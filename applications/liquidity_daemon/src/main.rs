use crate::config::Config;

mod config;

#[tokio::main]
async fn main() {
    let config = Config::read("./config.json".to_string());

    dbg!(config);
}

use thiserror::Error;

use tari_transaction::TransactionId;
use tari_wallet_daemon_client::error::WalletDaemonClientError;

pub mod contract;
pub mod liquidity;

#[derive(Error, Debug)]
pub enum TariError {
    #[error("Wallet error: {0}")]
    WalletError(#[from] WalletDaemonClientError),
    #[error("Transaction timeout for id: {transaction_id}")]
    TransactionTimeout { transaction_id: TransactionId },
}

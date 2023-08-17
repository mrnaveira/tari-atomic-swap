use std::sync::Arc;

use axum_jrpc::{
    error::{JsonRpcError, JsonRpcErrorReason},
    JrpcResult, JsonRpcExtractor, JsonRpcResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{self as json};

use crate::swap_manager::{ContractId, Preimage, Proposal, SwapManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSwapResponse {
    pub swap_id: String,
    pub provider_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFundsRequest {
    pub swap_id: String,
    pub contract_id: ContractId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFundsResponse {
    pub contract_id: ContractId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPreimageRequest {
    pub swap_id: String,
    pub preimage: Preimage,
}

pub struct JsonRpcHandlers {
    swap_manager: Arc<SwapManager>,
}

impl JsonRpcHandlers {
    pub fn new(swap_manager: Arc<SwapManager>) -> Self {
        Self { swap_manager }
    }

    pub async fn request_swap(&self, value: JsonRpcExtractor) -> JrpcResult {
        let answer_id = value.get_answer_id();
        let proposal: Proposal = value.parse_params()?;

        let result = self.swap_manager.request_swap(proposal).await;

        match result {
            Ok((swap_id, provider_address)) => {
                let response = RequestSwapResponse {
                    swap_id: swap_id.to_string(),
                    provider_address,
                };
                Ok(JsonRpcResponse::success(answer_id, response))
            }
            Err(e) => jrpc_error(answer_id, format!("Swap request rejected: {}", e)),
        }
    }

    pub async fn request_lock_funds(&self, value: JsonRpcExtractor) -> JrpcResult {
        let answer_id = value.get_answer_id();
        let request: LockFundsRequest = value.parse_params()?;

        let result = self
            .swap_manager
            .request_lock_funds(request.swap_id, request.contract_id)
            .await;

        match result {
            Ok(contract_id) => {
                let response = LockFundsResponse { contract_id };
                Ok(JsonRpcResponse::success(answer_id, response))
            }
            Err(e) => jrpc_error(answer_id, format!("Lock funds request rejected: {}", e)),
        }
    }

    pub async fn push_preimage(&self, value: JsonRpcExtractor) -> JrpcResult {
        let answer_id = value.get_answer_id();
        let request: PushPreimageRequest = value.parse_params()?;

        let result = self
            .swap_manager
            .push_preimage(request.swap_id, request.preimage)
            .await;

        match result {
            Ok(_) => Ok(JsonRpcResponse::success(answer_id, ())),
            Err(e) => jrpc_error(answer_id, format!("Lock funds request rejected: {}", e)),
        }
    }
}

fn jrpc_error(answer_id: i64, message: String) -> JrpcResult {
    Err(JsonRpcResponse::error(
        answer_id,
        JsonRpcError::new(
            JsonRpcErrorReason::InternalError,
            message,
            json::Value::Null,
        ),
    ))
}

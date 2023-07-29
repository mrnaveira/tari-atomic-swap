use std::{net::SocketAddr, sync::Arc};

use axum::{extract::Extension, routing::post, Router};
use axum_jrpc::{JrpcResult, JsonRpcExtractor};
use log::*;
use tower_http::cors::CorsLayer;

use super::handlers::JsonRpcHandlers;

const LOG_TARGET: &str = "liquidity_daemon::json_rpc";

pub async fn run_json_rpc(
    address: SocketAddr,
    handlers: JsonRpcHandlers,
) -> Result<(), anyhow::Error> {
    let router = Router::new()
        .route("/", post(handler))
        .route("/json_rpc", post(handler))
        .layer(Extension(Arc::new(handlers)))
        .layer(CorsLayer::permissive());

    let server = axum::Server::try_bind(&address)?;
    let server = server.serve(router.into_make_service());
    info!(target: LOG_TARGET, "🌐 JSON-RPC listening on {}", server.local_addr());
    server.await?;

    info!(target: LOG_TARGET, "💤 Stopping JSON-RPC");
    Ok(())
}

async fn handler(
    Extension(handlers): Extension<Arc<JsonRpcHandlers>>,
    value: JsonRpcExtractor,
) -> JrpcResult {
    debug!(target: LOG_TARGET, "🌐 JSON-RPC request: {}", value.method);
    debug!(target: LOG_TARGET, "🌐 JSON-RPC body: {:?}", value);
    match value.method.as_str() {
        "request_swap" => handlers.request_swap(value).await,
        "request_lock_funds" => handlers.request_lock_funds(value).await,
        "push_preimage" => handlers.push_preimage(value).await,
        method => Ok(value.method_not_found(method)),
    }
}

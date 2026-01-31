use tokio::sync::mpsc;
use tracing::{info, error, warn};
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::commitment_config::CommitmentConfig;
use std::time::Duration;
use futures::StreamExt;
use crate::engine::LogMessage;

pub async fn run_listener(rpc_url: String, tx: mpsc::Sender<LogMessage>) {
    let pump_program_id = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

    loop {
        let safe_url = if rpc_url.contains("?api-key=") {
            let parts: Vec<&str> = rpc_url.split("?api-key=").collect();
            format!("{}?api-key=***REDACTED***", parts[0])
        } else {
            rpc_url.clone()
        };
        
        info!("Connecting to Solana WebSocket at {}...", safe_url);

        match PubsubClient::new(&rpc_url).await {
            Ok(pubsub_client) => {
                info!("Connected. Subscribing to logs for program: {}", pump_program_id);
                // Subscription logic pending...
            },
            Err(e) => {
                error!("Failed to connect to WebSocket: {}", e);
            }
        }

        warn!("Reconnecting in 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

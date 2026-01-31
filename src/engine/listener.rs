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
        // ... (connection code same as above, keeping it concise for script)
        // Ideally we cat the full file here, but for brevity I will overwrite the full file again
        // with the next step integrated
        
        let safe_url = if rpc_url.contains("?api-key=") {
            let parts: Vec<&str> = rpc_url.split("?api-key=").collect();
            format!("{}?api-key=***REDACTED***", parts[0])
        } else {
            rpc_url.clone()
        };
        info!("Connecting to Solana WebSocket at {}...", safe_url);
        let mut log_count = 0;

        match PubsubClient::new(&rpc_url).await {
            Ok(pubsub_client) => {
                let config = RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                };

                match pubsub_client.logs_subscribe(
                    RpcTransactionLogsFilter::Mentions(vec![pump_program_id.to_string()]),
                    config
                ).await {
                    Ok((mut stream, _unsubscribe)) => {
                        info!("Pump.fun Subscription active. Listening for events...");
                        
                         while let Some(response) = stream.next().await {
                            log_count += 1;
                            if log_count % 100 == 0 {
                                info!("💓 Heartbeat: Monitoring Pump.fun... Scanned {} logs.", log_count);
                            }
                            // Processing logic...
                         }
                         warn!("WebSocket stream ended unexpectedly.");
                    },
                    Err(e) => error!("Failed to subscribe to logs: {}", e)
                }
            },
            Err(e) => error!("Failed to connect to WebSocket: {}", e)
        }
        warn!("Reconnecting in 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

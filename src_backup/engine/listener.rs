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
        let mut log_count = 0;

        match PubsubClient::new(&rpc_url).await {
            Ok(pubsub_client) => {
                info!("Connected. Subscribing to logs for program: {}", pump_program_id);
                
                let config = RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                };

                match pubsub_client.logs_subscribe(
                    RpcTransactionLogsFilter::Mentions(vec![pump_program_id.to_string()]),
                    config
                ).await {
                    Ok((mut stream, _unsubscribe)) => {
                        info!("Pump.fun Subscription active. Listening for events...");
                        
                        // Loop over the stream
                        while let Some(response) = stream.next().await {
                            log_count += 1;
                            if log_count % 100 == 0 {
                                info!("💓 Heartbeat: Monitoring Pump.fun... Scanned {} logs.", log_count);
                            }

                            // response.value is RpcLogsResponse
                            let signature = response.value.signature;
                            let logs = response.value.logs;
                            
                            // Pump.fun specific filter: "Instruction: Create"
                            let log_string = logs.join("\n");
                            if log_string.contains("Instruction: Create") {
                                let msg = LogMessage {
                                    signature,
                                    logs,
                                };
                                
                                if let Err(e) = tx.send(msg).await {
                                    error!("Failed to send log to processor: {}", e);
                                    break; // Channel closed
                                }
                            }
                        }
                        warn!("WebSocket stream ended unexpectedly.");
                    },
                    Err(e) => {
                        error!("Failed to subscribe to logs: {}", e);
                    }
                }
            },
            Err(e) => {
                error!("Failed to connect to WebSocket: {}", e);
            }
        }

        warn!("Reconnecting in 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

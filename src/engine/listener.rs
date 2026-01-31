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
        // Connection logic will go here
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

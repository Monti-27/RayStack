use tokio::sync::mpsc;
use tracing::{info, warn, error};
use crate::notifier::DiscordNotifier;
use crate::engine::LogMessage;
use base64::{Engine as _, engine::general_purpose};
use serde_json::json;

pub async fn run_processor(mut rx: mpsc::Receiver<LogMessage>, notifier: DiscordNotifier, _rpc_url: String) {
    info!("PumpHandler (Processor) started.");

    while let Some(msg) = rx.recv().await {
        // Processing logic pending...
    }
    warn!("LogHandler channel closed. Exiting.");
}

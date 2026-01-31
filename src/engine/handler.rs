use tokio::sync::mpsc;
use tracing::{info, warn, error};
use crate::notifier::DiscordNotifier;
use crate::engine::LogMessage;
use base64::{Engine as _, engine::general_purpose};
use serde_json::json;

pub async fn run_processor(mut rx: mpsc::Receiver<LogMessage>, notifier: DiscordNotifier, _rpc_url: String) {
    info!("PumpHandler (Processor) started.");

    while let Some(msg) = rx.recv().await {
        let mut mint_address = String::new();
        
        for i in 0..msg.logs.len() {
            let line = &msg.logs[i];
            
            if line.starts_with("Program data: ") {
                let encoded_data = line.trim_start_matches("Program data: ");
                
                if let Ok(decoded) = general_purpose::STANDARD.decode(encoded_data) {
                    if decoded.len() >= 40 {
                        let mint_bytes = &decoded[8..40];
                        mint_address = bs58::encode(mint_bytes).into_string();
                        info!("💊 PumpToken Detected! Mint: {}", mint_address);
                        break;
                    }
                }
            }
        }
    }
    warn!("LogHandler channel closed. Exiting.");
}

use tokio::sync::mpsc;
use tracing::{info, warn, error};
use crate::notifier::DiscordNotifier;
use crate::engine::LogMessage;
use base64::{Engine as _, engine::general_purpose};
use serde_json::json;

pub async fn run_processor(mut rx: mpsc::Receiver<LogMessage>, notifier: DiscordNotifier, _rpc_url: String) {
    info!("PumpHandler (Processor) started.");

    // No RPC client check needed for this fast path

    while let Some(msg) = rx.recv().await {
        
        // We look for "Instruction: Create" which we filtered in listener, but good to be safe.
        // And importantly, look for "Program data: "
        
        let mut mint_address = String::new();
        
        // Iterate looking for "Program data: "
        for i in 0..msg.logs.len() {
            let line = &msg.logs[i];
            if line.contains("Instruction: Create") {
                // The data usually follows in a subsequent log line or "Program data: " line for the event
            }
            
            if line.starts_with("Program data: ") {
                let encoded_data = line.trim_start_matches("Program data: ");
                
                // Decode Base64
                if let Ok(decoded) = general_purpose::STANDARD.decode(encoded_data) {
                    // Pump.fun Create Event Layout (approx, based on user req):
                    // 0-8: Discriminator
                    // 8-40: Mint (32 bytes)
                    
                    if decoded.len() >= 40 {
                        let mint_bytes = &decoded[8..40];
                        mint_address = bs58::encode(mint_bytes).into_string();
                        info!("💊 PumpToken Detected! Mint: {}", mint_address);
                        break; // Found it
                    }
                }
            }
        }

        if !mint_address.is_empty() {
             let description = format!(
                 "**Mint:** `{}`\n\n[Pump.fun](https://pump.fun/{}) | [BullX](https://bullx.io/terminal?chainId=1399811149&address={})",
                 mint_address, mint_address, mint_address
             );

            // Construct Rich Embed
            let embed = json!({
                "username": "PumpStack Monitor",
                "embeds": [{
                    "title": "� New Pump.fun Token Created",
                    "description": description,
                    "color": 15158332, // Pink-ish/Red
                     "footer": {
                        "text": "RayStack -> PumpStack Daemon"
                    },
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }]
            });
            
            if let Err(e) = notifier.send(&embed).await {
                warn!("Failed to deliver alert: {}", e);
            }
        }
    }

    warn!("LogHandler channel closed. Exiting.");
}

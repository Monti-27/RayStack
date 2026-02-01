use tokio::sync::mpsc;
use tracing::{info, warn, error};
use crate::notifier::DiscordNotifier;
use crate::engine::LogMessage;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use serde_json::json;
use solana_transaction_status::UiTransactionEncoding;
use solana_transaction_status::option_serializer::OptionSerializer;
use std::time::Duration;
use base64::{engine::general_purpose, Engine as _};
use byteorder::{ByteOrder, LittleEndian};

fn try_parse_metadata_at_offset(data: &[u8], start_offset: usize) -> Option<(String, String)> {
    let mut offset = start_offset;

    // Safety check for bounds
    if offset + 4 > data.len() {
        return None;
    }

    // Read Name Length
    let name_len = LittleEndian::read_u32(&data[offset..offset+4]) as usize;
    offset += 4;

    // Heuristic Check: Name should be reasonable length (e.g. 1 to 50 chars)
    if name_len == 0 || name_len > 50 || offset + name_len > data.len() {
        return None;
    }

    // Read Name
    let name = match String::from_utf8(data[offset..offset+name_len].to_vec()) {
        Ok(s) => s.trim_matches(char::from(0)).to_string(),
        Err(_) => return None,
    };
    offset += name_len;

    // Safety check for Symbol Length
    if offset + 4 > data.len() {
        return None;
    }

    // Read Symbol Length
    let symbol_len = LittleEndian::read_u32(&data[offset..offset+4]) as usize;
    offset += 4;

    // Heuristic Check: Symbol should be reasonable length (e.g. 1 to 20 chars)
    if symbol_len == 0 || symbol_len > 20 || offset + symbol_len > data.len() {
        return None;
    }

    // Read Symbol
    let symbol = match String::from_utf8(data[offset..offset+symbol_len].to_vec()) {
        Ok(s) => s.trim_matches(char::from(0)).to_string(),
        Err(_) => return None,
    };

    Some((name, symbol))
}

pub async fn run_processor(mut rx: mpsc::Receiver<LogMessage>, notifier: DiscordNotifier, rpc_url: String) {
    println!("[INFO] PumpHandler (Quality Filter) started.");
    
    // We need the RPC client to fetch balance changes
    let rpc_client = RpcClient::new(rpc_url);

    while let Some(msg) = rx.recv().await {
        let signature_str = msg.signature.clone();
        
        match Signature::from_str(&signature_str) {
            Ok(sig) => {
                let mut tx_details = None;

                // Config for confirmed transactions
                let config = RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                };

                // Retry Loop (Extended for Indexing Delays)
                for attempt in 0..10 {
                    match rpc_client.get_transaction_with_config(&sig, config) {
                        Ok(details) => {
                            tx_details = Some(details);
                            break; // Success
                        },
                        Err(_) => {
                            // If failed (likely not found/null), wait and retry
                            if attempt < 9 {
                                tokio::time::sleep(Duration::from_millis(500)).await;
                            }
                        }
                    }
                }

                if let Some(tx_details) = tx_details {
                    let mut dev_spend = 0.0;
                    let mut mint_address = String::from("Unknown");
                    
                    // Metadata extraction
                    let mut token_symbol = String::from("UNKNOWN");
                    let mut token_name = String::from("Unknown Token");

                    if let Some(meta) = &tx_details.transaction.meta {
                        // 1. Calculate Dev Spend
                        if meta.pre_balances.len() > 0 && meta.post_balances.len() > 0 {
                            let pre = meta.pre_balances[0];
                            let post = meta.post_balances[0];
                            
                            if pre > post {
                                let spent_lamports = pre - post;
                                dev_spend = spent_lamports as f64 / 1_000_000_000.0;
                            }
                        }
                        
                        // 2. Extract Mint Address
                        if let OptionSerializer::Some(token_balances) = &meta.post_token_balances {
                            for balance in token_balances {
                                if let Some(amount) = &balance.ui_token_amount.ui_amount {
                                    if *amount > 1_000_000.0 {
                                        mint_address = balance.mint.clone();
                                        break;
                                    }
                                }
                            }
                        }

                        // 3. Scan ALL Log Messages for Metadata
                        // Try offsets: 8 (Standard), 104 (Event with Pubkeys)
                        if let OptionSerializer::Some(log_messages) = &meta.log_messages {
                            for log in log_messages {
                                if log.starts_with("Program data: ") {
                                    let b64_data = log.trim_start_matches("Program data: ");
                                    if let Ok(decoded) = general_purpose::STANDARD.decode(b64_data) {
                                        
                                        // Attempt Scan at Offset 8
                                        if let Some((name, symbol)) = try_parse_metadata_at_offset(&decoded, 8) {
                                            token_name = name;
                                            token_symbol = symbol;
                                            break; 
                                        }

                                        // Attempt Scan at Offset 104
                                        if let Some((name, symbol)) = try_parse_metadata_at_offset(&decoded, 104) {
                                            token_name = name;
                                            token_symbol = symbol;
                                            break; 
                                        }
                                        
                                        // Debug: Capture the first buffer for debugging if we fail
                                        // (Only usage here is falling through)
                                    }
                                }
                            }
                        }
                    }

                    // Logging Logic (Clean Single Line with Symbol)
                    if dev_spend < 0.5 {
                        println!("[REJECTED] Spend: {:.4} SOL | Mint: {} | Ticker: {}", dev_spend, mint_address, token_symbol);
                    } else {
                        // Debugging Catch-all for UNKNOWN accepted tokens
                        if token_symbol == "UNKNOWN" {
                            println!("[DEBUG] ACCEPTED BUT UNKNOWN TICKER. Dumping logs for analysis...");
                            if let Some(meta) = &tx_details.transaction.meta {
                                if let OptionSerializer::Some(log_messages) = &meta.log_messages {
                                    for log in log_messages {
                                        if log.starts_with("Program data: ") {
                                            println!("-> Raw Log: {}", log);
                                        }
                                    }
                                }
                            }
                        }

                        println!("[ACCEPTED] ${} ({}) | Spend: {:.4} SOL | Mint: {} | Sending Alert...", token_symbol, token_name, dev_spend, mint_address);
                        
                        let description = format!(
                            "**Ticker:** `${}`\n**Name:** `{}`\n**Dev Spend:** `{:.2} SOL`\n**Mint:** `{}`\n\n[BullX](https://bullx.io/terminal?chainId=1399811149&address={}) | [Pump.fun](https://pump.fun/{})",
                            token_symbol, token_name, dev_spend, mint_address, mint_address, mint_address
                        );

                        let embed = json!({
                            "username": "PumpStack Monitor",
                            "embeds": [{
                                "title": format!("HQ Launch: ${}", token_symbol), // Shortened title
                                "description": description,
                                "color": 5763719, // Green
                                "footer": {
                                    "text": "PumpStack Quality Filter"
                                },
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }]
                        });

                        if let Err(e) = notifier.send(&embed).await {
                            println!("[WARN] Failed to deliver alert: {}", e);
                        }
                    }
                } else {
                    println!("[WARN] Tx Failed (Not Found): {}", signature_str);
                }
            },
            Err(_) => {
                println!("[ERROR] Invalid sig: {}", signature_str);
            }
        }
    }

    println!("[WARN] LogHandler channel closed. Exiting.");
}

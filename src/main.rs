mod engine;
mod notifier;

use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tokio::sync::mpsc;
use tracing::info;
use engine::{listener, handler, LogMessage};
use notifier::DiscordNotifier;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup Tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("RayStack Daemon (Production - Rich Data) starting...");

    // 2. Load Config
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set (WSS endpoint)");
    // Provide HTTP URL for RPC client (derived from WSS or separate env var)
    // Often RPC providers separate WSS and HTTP. For simplicity/MVP we might assume same base or just use RPC_URL if it works (Helius usually does).
    // Ideally we should have HTTP_RPC_URL.
    // For now, let's clone rpc_url and if it starts with wss, replace with https if needed, or rely on user config.
    // Hack: Replace wss:// with https:// for the HTTP client
    let http_rpc_url = rpc_url.replace("wss://", "https://").replace("ws://", "http://");
    
    let discord_webhook = env::var("DISCORD_WEBHOOK").expect("DISCORD_WEBHOOK must be set");

    // 3. Create Channels (Capacity 1000)
    let (tx, rx) = mpsc::channel::<LogMessage>(1000);

    // 4. Initialize Notifier
    let notifier = DiscordNotifier::new(discord_webhook);

    // 5. Spawn Consumer (Handler)
    let handler_rpc_url = http_rpc_url.clone();
    let handler_handle = tokio::spawn(async move {
        handler::run_processor(rx, notifier, handler_rpc_url).await;
    });

    // 6. Spawn Producer (Listener)
    // The listener runs indefinitely with its own reconnection loop
    let listener_handle = tokio::spawn(async move {
        listener::run_listener(rpc_url, tx).await;
    });

    // 7. Wait for tasks
    let _ = tokio::join!(handler_handle, listener_handle);
    
    info!("RayStack Daemon shutting down.");
    Ok(())
}

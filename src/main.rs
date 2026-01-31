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
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("RayStack Daemon (Production - Rich Data) starting...");

    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let discord_webhook = env::var("DISCORD_WEBHOOK").expect("DISCORD_WEBHOOK must be set");
    
    // tasks...
    Ok(())
}

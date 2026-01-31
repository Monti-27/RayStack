use anyhow::{Result, Context};
use reqwest::Client;
// use serde_json::json; 
use tracing::{info, error};

#[derive(Clone)]
pub struct DiscordNotifier {
    webhook_url: String,
    client: Client,
}

impl DiscordNotifier {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: Client::new(),
        }
    }

    pub async fn send(&self, payload: &serde_json::Value) -> Result<()> {
        let response = self.client.post(&self.webhook_url)
            .json(payload)
            .send()
            .await
            .context("Failed to send request to Discord")?;

        if !response.status().is_success() {
            error!("Discord returned error status: {}", response.status());
            // We log but don't error out the flow, as notification failure shouldn't crash the daemon
        } else {
            info!("Notification sent to Discord successfully.");
        }

        Ok(())
    }
}

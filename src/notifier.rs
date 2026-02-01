use anyhow::Result;
use reqwest::{Client, StatusCode};
use tracing::{info, error, warn};
use std::time::Duration;

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
        let mut attempts = 0;
        let max_retries = 1;

        loop {
            let response = self.client.post(&self.webhook_url)
                .json(payload)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!("Notification sent to Discord successfully.");
                        return Ok(());
                    } else if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                        warn!("⚠️ Rate Limit Hit (429). Cooling down...");
                        if attempts < max_retries {
                            attempts += 1;
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        } else {
                            error!("Max retries reached for Rate Limit. Dropping message.");
                            return Ok(());
                        }
                    } else {
                        error!("Discord returned error status: {}", resp.status());
                        return Ok(());
                    }
                },
                Err(e) => {
                    error!("Failed to send request: {}", e);
                    return Ok(());
                }
            }
        }
    }
}

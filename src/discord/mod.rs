use anyhow::Result;
use discord_presence::Client;
use tokio::sync::watch;

use crate::nextendo::client::Presence;

pub struct DiscordClient {
    client: Client,
}

impl DiscordClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(1567997158009864203),
        }
    }

    pub fn start(&mut self) {
        self.client
            .on_ready(|_| {
                println!("Connected to Discord");
            })
            .persist();

        self.client.start();
    }

    pub fn set_activity(&mut self, presence: &Presence) -> Result<()> {
        self.client.set_activity(|activity| {
            activity
                .details(if presence.game.is_empty() {
                    "Browsing Nextendo".to_string()
                } else {
                    presence.game.clone()
                })
                .state(if presence.game.is_empty() {
                    presence.platform.clone()
                } else {
                    format!("On {}", presence.platform)
                })
        })?;

        Ok(())
    }

    pub async fn listen(&mut self, mut rx: watch::Receiver<Presence>) -> Result<()> {
        while rx.changed().await.is_ok() {
            let presence = rx.borrow().clone();
            self.set_activity(&presence)?;
        }

        Ok(())
    }
}

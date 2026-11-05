use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::interval;

use crate::nextendo::client::{NextendoClient, Presence};
use crate::nextendo::token::get_access_token;

pub async fn start_checker(client: Arc<NextendoClient>, tx: watch::Sender<Presence>) -> Result<()> {
    let mut interval = interval(Duration::from_secs(15));
    let mut current_client = client;

    loop {
        interval.tick().await;

        let me = match current_client.get_me().await {
            Ok(me) => me,
            Err(_) => {
                let new_token = get_access_token().await?;
                current_client = Arc::new(NextendoClient::new(new_token));
                continue;
            }
        };

        println!("{}", me.presence.game);
        if tx.send(me.presence).is_err() {
            break;
        }
    }

    Ok(())
}

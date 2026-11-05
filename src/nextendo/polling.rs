use std::time::Duration;
use anyhow::Result;
use tokio::sync::watch;

use crate::nextendo::client::{NextendoClient, Presence};

pub async fn start_checker(client: NextendoClient, tx: watch::Sender<Presence>) -> Result<()> {
    loop {
        let me = client.get_me().await?;
        println!("{}", me.presence.game);
        tx.send(me.presence)?;
    
        tokio::time::sleep(Duration::from_secs(15)).await;
    }
}

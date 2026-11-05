use tokio::sync::watch;

use crate::nextendo::token::get_access_token;
use crate::nextendo::{client::NextendoClient, client::Presence};

mod discord;
mod nextendo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let access_token = get_access_token().await?;

    let client = NextendoClient::new_arc(access_token);

    let identity = client.get_identity().await?;
    println!("Signed in as {}", identity.username);

    let (tx, rx) = watch::channel(Presence::default());

    tokio::spawn(nextendo::polling::start_checker(client, tx));

    let mut discord = discord::DiscordClient::new();
    discord.start();
    discord.listen(rx).await?;

    Ok(())
}

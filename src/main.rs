use oauth2::TokenResponse;
use tokio::sync::watch;

use crate::nextendo::{auth::OAuthConfig, client::NextendoClient, client::Presence};

mod discord;
mod nextendo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let token = nextendo::auth::login(&config).await?;

    let access_token = nextendo::token::get_access_token().await?;

    let client = NextendoClient::new(access_token);

    let identity = client.get_identity().await?;
    println!("Signed in as {}", identity.username);

    let (tx, rx) = watch::channel(Presence::default());

    tokio::spawn(nextendo::polling::start_checker(client, tx));

    let mut discord = discord::DiscordClient::new();
    discord.start();
    discord.listen(rx).await?;

    Ok(())
}

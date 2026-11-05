use oauth2::TokenResponse;

use crate::nextendo::auth::OAuthConfig;

mod nextendo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = OAuthConfig {
        client_id: "nxc_tKyBi6nlWV1b".into(),
        authorization_url: "https://nextendo.network/oauth/authorize".into(),
        token_url: "https://nextendo.network/api/oauth/token".into(),
        redirect_uri: "http://127.0.0.1:47823/callback".into(),
        scopes: vec!["identity".into(), "presence".into()],
    };

    let token = nextendo::auth::login(&config).await?;

    println!("Authenticated successfully");

    Ok(())
}

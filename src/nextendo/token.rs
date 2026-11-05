use anyhow::{Result, anyhow};
use keyring::Entry;
use oauth2::{StandardRevocableToken::AccessToken, TokenResponse, basic::BasicTokenResponse};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::nextendo::auth::{self, OAuthConfig};

#[derive(Debug, Serialize, Deserialize)]
struct StoredTokens {
    access_token: String,
    refresh_token: String,
    expires_at: SystemTime,
}

fn save_tokens(tokens: &StoredTokens) -> Result<()> {
    let entry = Entry::new("prexendo", "nextendo")?;
    let json = serde_json::to_string(tokens)?;

    entry.set_password(&json)?;

    Ok(())
}
fn load_tokens() -> Result<StoredTokens> {
    let entry = Entry::new("prexendo", "nextendo")?;
    let json = entry.get_password()?;

    Ok(serde_json::from_str(&json)?)
}

fn store_response(response: BasicTokenResponse) -> Result<StoredTokens> {
    todo!()
}

async fn refresh_tokens(tokens: &StoredTokens) -> Result<StoredTokens> {
    todo!()
}

pub async fn get_access_token() -> Result<String> {
    let tokens = match load_tokens() {
        Ok(tokens) => tokens,

        Err(err)
            if matches!(
                err.downcast_ref::<keyring::Error>(),
                Some(keyring::Error::NoEntry)
            ) =>
        {
            let config = auth::config();

            let response = auth::login(&config).await?;

            let refresh_token = response
                .refresh_token()
                .ok_or_else(|| anyhow!("Nextendo did not provide a refresh token"))?
                .secret()
                .clone();

            let expires_in = response
                .expires_in()
                .ok_or_else(|| anyhow!("OAuth response did not include token expiry"))?;

            let tokens = StoredTokens {
                access_token: response.access_token().secret().clone(),
                refresh_token: refresh_token,
                expires_at: SystemTime::now() + expires_in,
            };

            save_tokens(&tokens)?;

            tokens
        }

        Err(err) => return Err(err),
    };

    if SystemTime::now() + Duration::from_secs(60) >= tokens.expires_at {
        let response = auth::refresh(&auth::config(), &tokens.refresh_token).await?;

        let expires_in = response
            .expires_in()
            .ok_or_else(|| anyhow!("OAuth response did not include token expiry"))?;

        let tokens = StoredTokens {
            access_token: response.access_token().secret().clone(),
            refresh_token: response
                .refresh_token()
                .map(|token| token.secret().clone())
                .unwrap_or_else(|| tokens.refresh_token.clone()),
            expires_at: SystemTime::now() + expires_in,
        };

        save_tokens(&tokens)?;
        return Ok(tokens.access_token);
    }

    Ok(tokens.access_token)
}

// TODO List for LLM
// - Re-authenticate if the refresh token is expired
// - Automatically refresh the tokens while the tool is open in the background
// - Don't make a new page that just says "You can close this tab" since Nextendo's own ownauth success page is perfectly good

// Notes for LLM
// - Nextendo's own servers refresh every 15 seconds, so that's the cycle everything runs at
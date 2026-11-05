use anyhow::{Result, anyhow};
use keyring::Entry;
use oauth2::{TokenResponse, basic::BasicTokenResponse};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use crate::nextendo::auth::{config, login, refresh};

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

fn tokens_from_response(response: BasicTokenResponse) -> Result<StoredTokens> {
    let refresh_token = response
        .refresh_token()
        .ok_or_else(|| anyhow!("Nextendo did not provide a refresh token"))?
        .secret()
        .clone();

    let expires_in = response
        .expires_in()
        .ok_or_else(|| anyhow!("OAuth response did not include token expiry"))?;

    Ok(StoredTokens {
        access_token: response.access_token().secret().clone(),
        refresh_token,
        expires_at: SystemTime::now() + expires_in,
    })
}

async fn refresh_stored_tokens(tokens: &StoredTokens) -> Result<StoredTokens> {
    let response = refresh(&config(), &tokens.refresh_token).await?;
    tokens_from_response(response)
}

pub async fn get_access_token() -> Result<String> {
    let mut tokens = match load_tokens() {
        Ok(tokens) => tokens,

        Err(err)
            if matches!(
                err.downcast_ref::<keyring::Error>(),
                Some(keyring::Error::NoEntry)
            ) =>
        {
            let response = login(&config()).await?;

            let tokens = tokens_from_response(response)?;
            save_tokens(&tokens)?;
            tokens
        }

        Err(err) => return Err(err),
    };

    if SystemTime::now() + Duration::from_secs(60) >= tokens.expires_at {
        match refresh_stored_tokens(&tokens).await {
            Ok(new_tokens) => {
                tokens = new_tokens;
                save_tokens(&tokens)?;
            }
            Err(_) => {
                let response = login(&config()).await?;
                tokens = tokens_from_response(response)?;
                save_tokens(&tokens)?;
            }
        }
    }

    Ok(tokens.access_token)
}

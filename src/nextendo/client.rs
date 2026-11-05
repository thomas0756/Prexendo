use anyhow::Result;
use serde::Deserialize;

pub struct NextendoClient {
    client: reqwest::Client,
    access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Identity {
    pub pid: Option<u64>,
    pub username: String,
    pub friend_code: Option<String>,
    pub avatar_url: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct MeResponse {
    pub account: Account,
    pub presence: Presence,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub pid: u64,
    pub username: String,
    pub friend_code: String,
}

#[derive(Debug, Default, Deserialize, PartialEq, Clone)]
pub struct Presence {
    pub app_detail: String,
    pub app_id: String,
    #[serde(rename = "icone")]
    pub icon: String,
    #[serde(rename = "jeu")]
    pub game: String,
    #[serde(rename = "plateforme")]
    pub platform: String,
    #[serde(rename = "status")]
    pub status: u32,
}

impl NextendoClient {
    pub fn new(access_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token,
        }
    }

    pub async fn get_identity(&self) -> Result<Identity> {
        let response = self
            .client
            .get("https://nextendo.network/api/oauth/userinfo")
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Identity>().await?)
    }

    pub async fn get_me(&self) -> Result<MeResponse> {
    let response = self.client
        .get("https://nextendo.network/api/me")
        .bearer_auth(&self.access_token)
        .send()
        .await?
        .error_for_status()?;

    Ok(response.json::<MeResponse>().await?)
}
}

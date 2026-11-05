use anyhow::{Ok, Result, anyhow};
use axum::routing::get;
use axum::{
    Router,
    extract::{Query, State},
};

use oauth2::RefreshToken;
use oauth2::basic::BasicTokenResponse;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenUrl, basic::BasicClient, reqwest,
};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub struct OAuthConfig {
    pub client_id: String,
    pub authorization_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

pub struct PendingAuthorization {
    pub state: CsrfToken,
    pub verifier: PkceCodeVerifier,
}

pub fn authorization_url(config: &OAuthConfig) -> Result<(url::Url, PendingAuthorization)> {
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(AuthUrl::new(config.authorization_url.clone())?)
        .set_token_uri(TokenUrl::new(config.token_url.clone())?)
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

    let request = config.scopes.iter().fold(
        client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(challenge),
        |request, scope| request.add_scope(Scope::new(scope.clone())),
    );

    let (url, state) = request.url();

    Ok((url, PendingAuthorization { state, verifier }))
}

pub async fn receive_callback(
    listener: tokio::net::TcpListener,
    path: String,
) -> Result<(AuthorizationCode, String)> {
    let (callback_tx, mut callback_rx) = mpsc::channel(1);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let app = Router::new()
        .route(&path, get(callback))
        .with_state(callback_tx);

    let server = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = shutdown_rx.await;
    });

    let server = tokio::spawn(async move { server.await });

    let params = callback_rx
        .recv()
        .await
        .ok_or_else(|| anyhow!("Callback channel closed"))?;

    let _ = shutdown_tx.send(());

    server.await??;

    Ok((AuthorizationCode::new(params.code), params.state))
}

#[derive(Deserialize)]
struct CallbackParams {
    code: String,
    state: String,
}

async fn callback(
    State(tx): State<mpsc::Sender<CallbackParams>>,
    Query(params): Query<CallbackParams>,
) -> &'static str {
    let _ = tx.send(params).await;

    "Authentication received. You can close this window."
}

pub async fn exchange_code(
    config: &OAuthConfig,
    code: AuthorizationCode,
    verifier: PkceCodeVerifier,
) -> Result<oauth2::basic::BasicTokenResponse> {
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_token_uri(TokenUrl::new(config.token_url.clone())?)
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let token = client
        .exchange_code(code)
        .set_pkce_verifier(verifier)
        .request_async(&http_client)
        .await?;

    Ok(token)
}

pub async fn login(config: &OAuthConfig) -> Result<BasicTokenResponse> {
    let (url, pending) = authorization_url(config)?;

    let redirect_uri = url::Url::parse(&config.redirect_uri)?;

    let host = redirect_uri
        .host_str()
        .ok_or_else(|| anyhow!("Redirect URI has no host"))?;

    let port = redirect_uri
        .port()
        .ok_or_else(|| anyhow!("Redirect URI has no port"))?;

    let listener = tokio::net::TcpListener::bind((host, port)).await?;

    let callback = tokio::spawn(receive_callback(listener, redirect_uri.path().to_owned()));

    webbrowser::open(url.as_str())?;

    let (code, state) = callback.await??;

    if state != *pending.state.secret() {
        return Err(anyhow!("OAuth state mismatch"));
    }

    let token = exchange_code(config, code, pending.verifier).await?;

    Ok(token)
}

pub async fn refresh(config: &OAuthConfig, refresh_token: &str) -> Result<BasicTokenResponse> {
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_token_uri(TokenUrl::new(config.token_url.clone())?);
    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let response = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_owned()))
        .request_async(&http_client)
        .await?;

    Ok(response)
}

pub fn config() -> OAuthConfig {
    OAuthConfig {
        client_id: "nxc_VFatPpklaSUn".into(),
        authorization_url: "https://nextendo.network/oauth/authorize".into(),
        token_url: "https://nextendo.network/api/oauth/token".into(),
        redirect_uri: "http://127.0.0.1:47823/callback".into(),
        scopes: vec!["identity".into(), "presence".into(), "friends".into()],
    }
}
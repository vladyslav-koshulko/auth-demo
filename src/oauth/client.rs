use std::env;
use jsonwebtoken::get_current_timestamp;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::session::file::{save_session_with_user, Session};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub id_token: String,
    pub scope: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
}

pub async fn exchange_code_for_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<TokenResponse, reqwest::Error> {
    let client = Client::new();

    let params = [
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", code_verifier),
    ];

    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;

    let token = res.json::<TokenResponse>().await?;

    Ok(token)
}

pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse, String> {
    let client_id = env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID");
    let client_secret = env::var("GOOGLE_CLIENT_SECRET").expect("Missing GOOGLE_CLIENT_SECRET");

    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let client = reqwest::Client::new();
    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body = res.json::<TokenResponse>().await.map_err(|e| e.to_string())?;
    Ok(body)
}

pub async fn ensure_valid_token(
    session_id: &str,
    session: &mut Session,
) -> Result<(), String> {
    let now = get_current_timestamp();
    if now > session.expires_at {
        println!("Token expired, refreshing...");

        if let Some(refresh_token) = &session.refresh_token {
            let token = refresh_access_token(refresh_token).await?;
            session.access_token = token.access_token;
            session.expires_at = now + (token.expires_in as u64);

            save_session_with_user(session_id, session.clone());
        } else {
            return Err("No refresh token found".into());
        }
    }
    Ok(())
}
use crate::middleware::auth::{auth_middleware, AuthUser};
use crate::models::user::User;
use crate::oauth::client::exchange_code_for_token;
use crate::oauth::jwks_cache::JwksCache;
use crate::oauth::jwt::parse_id_token;
use crate::session::file::{save_session_with_user, Session};
use crate::utils::crypto::generate_session_id;
use axum::extract::{Query, State};
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use jsonwebtoken::get_current_timestamp;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub expected_state: Arc<Mutex<Option<String>>>,
    pub jwks_cache: Arc<Mutex<JwksCache>>,
    pub code_verifier: Arc<Mutex<Option<String>>>,
    pub expected_nonce: Arc<Mutex<Option<String>>>,
}

pub async fn start_server(state: AppState) {
    let public_routers = Router::new().route("/callback", get(callback_handler));

    let protected_routers = Router::new()
        .route("/me", get(protected_me_handler))
        .layer(middleware::from_fn(auth_middleware));

    let app = public_routers.merge(protected_routers).with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();

    println!("Listening on http://127.0.0.1:8081");

    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn callback_handler(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let code = match params.get("code") {
        Some(code) => code,
        None => {
            println!("No code received");
            return "No code received".to_string();
        }
    };
    let state = params.get("state");

    let expected_state = {
        let guard = app_state.expected_state.lock().unwrap();
        guard.clone()
    };
    match (state, expected_state.as_ref()) {
        (Some(received), Some(expected)) => {
            if received != expected {
                println!("Invalid state");
                return "Invalid state".to_string();
            }
        }
        _ => {
            println!("Invalid state");
            return "Invalid state".to_string();
        }
    }
    println!("State is valid");

    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
    let client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap();
    let redirect_uri = env::var("REDIRECT_URL").unwrap();
    let code_verifier = {
        let guard = app_state.code_verifier.lock().unwrap();
        guard.clone().unwrap()
    };
    match exchange_code_for_token(
        code,
        client_id.as_ref(),
        client_secret.as_ref(),
        redirect_uri.as_ref(),
        &code_verifier,
    )
    .await
    {
        Ok(token) => {
            println!("ID token: {}", token.id_token);
            let expected_nonce = {
                let guard = app_state.expected_nonce.lock().unwrap();
                guard.clone()
            };
            match parse_id_token(
                &token.id_token,
                &client_id,
                app_state.jwks_cache.clone(),
                expected_nonce,
            )
            .await
            {
                Ok(claims) => {
                    println!("ID token claims: {:#?}", claims);
                    let user = User {
                        id: claims.sub,
                        email: claims.email,
                        name: claims.name,
                    };

                    let session_id = generate_session_id();
                    println!("Session created: {}", session_id);
                    let now = get_current_timestamp();
                    let expires_at = now + token.expires_in as u64;
                    let session = Session {
                        user,
                        access_token: token.access_token,
                        refresh_token: token.refresh_token,
                        expires_at,
                    };

                    save_session_with_user(&session_id, session);
                    println!("User logged in.");

                    "Login successful. You can close this tab.".to_string()
                }
                Err(e) => {
                    println!("JWT parsing error: {}", e);
                    "Invalid token".to_string()
                }
            }
        }
        Err(e) => {
            println!("Token exchange error: {:?}", e);
            "Error during token exchange:".to_string()
        }
    }
}

async fn protected_me_handler(auth_user: AuthUser) -> impl IntoResponse {
    let user = &auth_user.session.user;
    format!(
        "Authenticated User:\nID: {}\nName {}\nEmail: {}\n",
        user.id, user.name, user.email
    )
}

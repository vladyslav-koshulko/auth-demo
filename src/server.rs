use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use crate::models::user::User;
use crate::oauth::claims::IdTokenClaims;
use crate::oauth::client::exchange_code_for_token;
use crate::oauth::jwt::parse_id_token;
use crate::session::file::{save_session_with_user};
use crate::utils::crypto::generate_session_id;

#[derive(Clone, Debug,)]
pub struct AppState {
    pub expected_state: Arc<Mutex<Option<String>>>,
}

pub async fn start_server(state: AppState) {
    let app = Router::new()
        .route("/callback", get(callback_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();

    println!("Listening on http://127.0.0.1:8081");

    axum::serve(listener, app).await.unwrap();
}


#[axum::debug_handler]
async fn callback_handler(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>
)  -> impl IntoResponse {
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

    match exchange_code_for_token(code, client_id.as_ref(), client_secret.as_ref(), redirect_uri.as_ref()).await {
        Ok(token) => {
            println!("ID token: {}", token.id_token);

            match parse_id_token(token.id_token.as_ref(), client_id.as_ref()) {
                Ok(claims) => {
                    println!("ID token claims: {:#?}", claims);
                    let user = User {
                        id: claims.sub,
                        email: claims.email,
                        name: claims.name,
                    };
                    
                    let session_id = generate_session_id();
                    println!("Session created: {}", session_id);
                    
                    save_session_with_user(&session_id, &user);
                    println!("User logged in: {:?}", user);
                    
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

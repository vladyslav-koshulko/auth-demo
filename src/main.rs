use std::env;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use clap::Parser;
use crate::oauth::client::ensure_valid_token;
use crate::oauth::google::build_authorization_url;
use crate::oauth::jwks_cache::JwksCache;
use crate::server::{start_server, AppState};
use crate::session::file::{clear_session, get_current_session_id, get_current_user};
use crate::utils::crypto::{generate_code_challenge, generate_code_verifier, generate_random_string};
use humantime::format_duration;

mod cli;
mod models;
mod session;
mod server;
mod utils;
mod oauth;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    dotenvy::dotenv().expect("Failed to load .env file");

    match cli.command {

        cli::Commands::Login =>{
            let state = generate_random_string(16);
            let nonce = generate_random_string(16);
            let code_verifier = generate_code_verifier();
            let code_challenge = generate_code_challenge(&code_verifier);

            let client_id = env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID");
            let redirect_uri = env::var("REDIRECT_URL").expect("Missing REDIRECT_URI");

            let url = build_authorization_url(
                client_id.as_ref(),
                redirect_uri.as_ref(),
                &state,
                &nonce,
                &code_challenge
            );

            let shared_state = Arc::new(Mutex::new(Some(state)));
            let expected_nonce = Arc::new(Mutex::new(Some(nonce)));

            let app_state = AppState {
                expected_state: shared_state.clone(),
                jwks_cache: Arc::new(Mutex::new(JwksCache::new())),
                code_verifier: Arc::new(Mutex::new(Some(code_verifier))),
                expected_nonce: expected_nonce.clone(),
            };

            println!("Opening browser ...");
            println!("URL: {}", url);

            webbrowser::open(&url).unwrap();

            start_server(app_state).await;
        },
        cli::Commands::Me => {
            match get_current_session_id() {
                None => {
                    println!("Not logged in");
                }
                Some(session_id) => {
                    let mut session = match get_current_user() {
                        Some(s) => s,
                        None => {
                            println!("Session not fount");
                            return;
                        }
                    };

                    if let Err(e) = ensure_valid_token(&session_id, &mut session).await {
                        println!("Failed to ensure valid token: {}", e);
                        println!("Please login again");
                        clear_session();
                        return;
                    }
                    let expiration_time = Duration::from_millis(session.expires_at.clone());
                    println!("User:");
                    println!("  ID: {}", session.user.id);
                    println!("  Name: {}", session.user.name);
                    println!("  Email: {}", session.user.email);
                    println!("  Token expires at: {}", format_duration(expiration_time))
                }
            }
        },
        cli::Commands::Logout => {
            clear_session();
            println!("Logged out");
        },
    }
}

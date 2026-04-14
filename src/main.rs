use std::env;
use std::sync::{Arc, Mutex};
use clap::Parser;
use crate::oauth::google::build_authorization_url;
use crate::oauth::jwks_cache::JwksCache;
use crate::server::{start_server, AppState};
use crate::session::file::{clear_session, get_current_user};
use crate::utils::crypto::{generate_code_challenge, generate_code_verifier, generate_random_string};

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

            let app_state = AppState {
                expected_state: shared_state.clone(),
                jwks_cache: Arc::new(Mutex::new(JwksCache::new())),
                code_verifier: Arc::new(Mutex::new(Some(code_verifier))),
            };

            println!("Opening browser ...");
            println!("URL: {}", url);

            webbrowser::open(&url).unwrap();

            start_server(app_state).await;
        },
        cli::Commands::Me => {
            match get_current_user() {
                None => {
                    println!("User not logged in");
                }
                Some(session) => {
                    println!("User:");
                    println!("ID: {}", session.user.id);
                    println!("Name: {}", session.user.name);
                    println!("Email: {}", session.user.email);
                }
            }
        },
        cli::Commands::Logout => {
            clear_session();
            println!("Logged out");
        },
    }
}

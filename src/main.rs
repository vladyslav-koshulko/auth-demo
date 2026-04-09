use std::env;
use std::sync::{Arc, Mutex};
use clap::Parser;
use crate::oauth::google::build_authorization_url;
use crate::server::{start_server, AppState};
use crate::session::file::{clear_session, load_session};
use crate::utils::crypto::generate_random_string;

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

            let client_id = env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID");
            let redirect_uri = env::var("REDIRECT_URL").expect("Missing REDIRECT_URI");

            let url = build_authorization_url(client_id.as_ref(), redirect_uri.as_ref(), &state, &nonce);

            let shared_state = Arc::new(Mutex::new(Some(state)));

            let app_state = AppState {
                expected_state: shared_state.clone(),
            };

            println!("Opening browser ...");
            println!("URL: {}", url);

            webbrowser::open(&url).unwrap();

            start_server(app_state).await;
        },
        cli::Commands::Me => {
            match load_session() {
                None => {
                    println!("User is not logged in (stub)");
                }
                Some(session_id) => {
                    println!("Session ID: {}", session_id);

                    print!("User is logged in (stub)");
                }
            }
        },
        cli::Commands::Logout => {
            clear_session();
            println!("Logged out");
        },
    }
}

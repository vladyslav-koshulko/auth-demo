use std::sync::{Arc, Mutex};
use clap::Parser;
use crate::oauth::google::build_authorization_url;
use crate::server::{start_server, AppState};
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


    match cli.command {
        cli::Commands::Login =>{
            let state = generate_random_string(16);
            let nonce = generate_random_string(16);

            let client_id = "YOUR_CLIENT_ID";
            let redirect_uri = "http://127.0.0.1:8081/callback";

            let url = build_authorization_url(client_id, redirect_uri, &state, &nonce);

            let shared_state = Arc::new(Mutex::new(Some(state)));

            let app_state = AppState {
                expected_state: shared_state.clone(),
            };

            println!("Opening browser ...");
            println!("URL: {}", url);

            webbrowser::open(&url).unwrap();

            start_server(app_state).await;
        },
        cli::Commands::Me => println!("Me"),
        cli::Commands::Logout => println!("Logout"),
    }
}

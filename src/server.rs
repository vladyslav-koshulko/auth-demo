use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use axum::extract::{Query, State};
use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;

#[derive(Clone)]
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

async fn callback_handler(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>
)  -> String {
    let code = params.get("code");
    let state = params.get("state");
    println!("Received code: {:?}", code);
    println!("Received state: {:?}", state);

    let expected_state = app_state.expected_state.lock().unwrap();
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


    "Login successful. You can close this tab.".to_string()
}

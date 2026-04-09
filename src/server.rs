use std::collections::HashMap;
use axum::extract::Query;
use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;

pub async fn start_server() {
    let app = Router::new()
        .route("/callback", get(callback_handler));

    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();

    println!("Listening on http://127.0.0.1:8081");

    axum::serve(listener, app).await.unwrap();
}

async fn callback_handler(Query(params): Query<HashMap<String, String>>)  -> String {
    let code = params.get("code").unwrap();
    let state = params.get("state").unwrap();
    println!("Received code: {}", code);
    println!("Received state: {}", state);

    "Login successful. You can close this tab.".to_string()
}

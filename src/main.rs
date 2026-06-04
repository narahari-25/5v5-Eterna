mod api;
mod matchmaker;
mod models;
mod state;

use api::{enqueue_player, get_matches, get_metrics};
use axum::{
    routing::{get, post},
    Router,
};
use matchmaker::run_matchmaker;
use state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    for _ in 0..4 {
        let matcher_state = state.clone();

        tokio::spawn(async move {
            run_matchmaker(matcher_state).await;
        });
    }

    let app = Router::new()
        .route("/enqueue", post(enqueue_player))
        .route("/metrics", get(get_metrics))
        .route("/matches", get(get_matches))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Matchmaker service running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
use crate::models::{EnqueueRequest, EnqueueResponse, Player};
use crate::state::AppState;
use axum::{extract::State, Json};
use std::time::Instant;
use uuid::Uuid;
use crate::models::{Match, MetricsResponse};

pub async fn enqueue_player(
    State(state): State<AppState>,
    Json(payload): Json<EnqueueRequest>,
) -> Json<EnqueueResponse> {
    let player = Player {
        id: Uuid::new_v4(),
        mmr: payload.mmr,
        region: payload.region,
        joined_at: Instant::now(),
    };

    let player_id = player.id;

    let mut waiting_players = state.waiting_players.lock().unwrap();
    waiting_players.push(player);

    Json(EnqueueResponse {
        player_id,
        message: "Player added to matchmaking queue".to_string(),
    })
}

pub async fn get_metrics(State(state): State<AppState>) -> Json<MetricsResponse> {
    use std::sync::atomic::Ordering;

    let waiting_players_guard = state.waiting_players.lock().unwrap();

    let waiting_players = waiting_players_guard.len();

    let oldest_waiting_player_ms = waiting_players_guard
        .iter()
        .map(|p| p.joined_at.elapsed().as_millis())
        .max()
        .unwrap_or(0);

    let matches_created = state.matches_created.load(Ordering::Relaxed);

    let total_wait_time_ms = state.total_wait_time_ms.load(Ordering::Relaxed);

    let total_mmr_diff = state.total_mmr_diff.load(Ordering::Relaxed);

    let average_wait_time_ms = if matches_created == 0 {
        0.0
    } else {
        total_wait_time_ms as f64 / matches_created as f64
    };

    let average_mmr_diff = if matches_created == 0 {
        0.0
    } else {
        total_mmr_diff as f64 / matches_created as f64
    };

    let elapsed_secs = state.started_at.elapsed().as_secs_f64();

    let matches_per_second = if elapsed_secs > 0.0 {
        matches_created as f64 / elapsed_secs
    } else {
        0.0
    };

    Json(MetricsResponse {
        waiting_players,
        matches_created,
        average_wait_time_ms,
        average_mmr_diff,

        oldest_waiting_player_ms,
        matches_per_second,
    })
}
pub async fn get_matches(State(state): State<AppState>) -> Json<Vec<Match>> {
    let matches = state.matches.lock().unwrap().clone();
    Json(matches)
}
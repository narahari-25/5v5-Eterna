use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone)]
pub struct Player {
    pub id: Uuid,
    pub mmr: i32,
    pub region: String,
    pub joined_at: Instant,
}

#[derive(Deserialize)]
pub struct EnqueueRequest {
    pub mmr: i32,
    pub region: String,
}

#[derive(Serialize)]
pub struct EnqueueResponse {
    pub player_id: Uuid,
    pub message: String,
}

#[derive(Serialize)]
pub struct MetricsResponse {
    pub waiting_players: usize,
    pub matches_created: usize,
    pub average_wait_time_ms: f64,
    pub average_mmr_diff: f64,

    // New metrics
    pub oldest_waiting_player_ms: u128,
    pub matches_per_second: f64,
}

#[derive(Clone, Serialize)]
pub struct Match {
    pub match_id: Uuid,
    pub team_a: Vec<PlayerSummary>,
    pub team_b: Vec<PlayerSummary>,
    pub avg_mmr_a: f64,
    pub avg_mmr_b: f64,
    pub mmr_diff: f64,
}

#[derive(Clone, Serialize)]
pub struct PlayerSummary {
    pub id: Uuid,
    pub mmr: i32,
    pub region: String,
}
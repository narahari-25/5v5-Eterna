use crate::models::{Match, Player};
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub waiting_players: Arc<Mutex<Vec<Player>>>,
    pub matches: Arc<Mutex<Vec<Match>>>,
    pub matches_created: Arc<AtomicUsize>,
    pub total_wait_time_ms: Arc<AtomicU64>,
    pub total_mmr_diff: Arc<AtomicU64>,

    // New field
    pub started_at: Arc<Instant>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            waiting_players: Arc::new(Mutex::new(Vec::new())),
            matches: Arc::new(Mutex::new(Vec::new())),
            matches_created: Arc::new(AtomicUsize::new(0)),
            total_wait_time_ms: Arc::new(AtomicU64::new(0)),
            total_mmr_diff: Arc::new(AtomicU64::new(0)),

            // New field
            started_at: Arc::new(Instant::now()),
        }
    }

    pub fn get_matches_created(&self) -> usize {
        self.matches_created.load(Ordering::Relaxed)
    }
}
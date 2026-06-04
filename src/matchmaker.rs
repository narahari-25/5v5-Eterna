use crate::models::{Match, Player, PlayerSummary};
use crate::state::AppState;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

const MATCH_SIZE: usize = 10;
const TEAM_SIZE: usize = 5;

pub async fn run_matchmaker(state: AppState) {
    loop {
        while try_create_match(&state) {}

        sleep(Duration::from_millis(20)).await;
    }
}

fn try_create_match(state: &AppState) -> bool {
    let selected_players = {
        let mut waiting_players = state.waiting_players.lock().unwrap();

        if waiting_players.len() < MATCH_SIZE {
            return false;
        }

        waiting_players.sort_by_key(|p| p.mmr);

        let selected_indices = find_best_candidate_window(&waiting_players);

        let Some(selected_indices) = selected_indices else {
            return false;
        };

        let mut selected_players = Vec::new();

        for index in selected_indices.into_iter().rev() {
            selected_players.push(waiting_players.remove(index));
        }

        selected_players
    };

    let average_wait_ms = selected_players
        .iter()
        .map(|p| p.joined_at.elapsed().as_millis() as u64)
        .sum::<u64>()
        / selected_players.len() as u64;

    let new_match = create_balanced_match(selected_players);

    state
        .total_wait_time_ms
        .fetch_add(average_wait_ms, Ordering::Relaxed);

    state
        .total_mmr_diff
        .fetch_add(new_match.mmr_diff.round() as u64, Ordering::Relaxed);

    state.matches.lock().unwrap().push(new_match);
    state.matches_created.fetch_add(1, Ordering::Relaxed);

    true
}

fn find_best_candidate_window(waiting_players: &[Player]) -> Option<Vec<usize>> {
    if waiting_players.len() < MATCH_SIZE {
        return None;
    }

    let mut best_indices: Option<Vec<usize>> = None;
    let mut best_spread = i32::MAX;

    for start in 0..=waiting_players.len() - MATCH_SIZE {
        let end = start + MATCH_SIZE - 1;

        let oldest_wait_secs = waiting_players[start]
            .joined_at
            .elapsed()
            .as_secs();

        let allowed_range = get_allowed_mmr_range(oldest_wait_secs);

        let spread = waiting_players[end].mmr - waiting_players[start].mmr;

        if spread <= allowed_range && spread < best_spread {
            best_spread = spread;
            best_indices = Some((start..=end).collect());
        }
    }

    best_indices
}

fn get_allowed_mmr_range(wait_time_secs: u64) -> i32 {
    if wait_time_secs < 1 {
        200
    } else if wait_time_secs < 3 {
        500
    } else if wait_time_secs < 5 {
        1000
    } else {
        3000
    }
}

fn create_balanced_match(players: Vec<Player>) -> Match {
    let n = players.len();
    let mut best_team_a_indices = Vec::new();
    let mut best_diff = f64::MAX;

    for mask in 0usize..(1usize << n) {
        if mask.count_ones() as usize == TEAM_SIZE {
            let mut team_a_sum = 0;
            let mut team_b_sum = 0;

            for i in 0..n {
                if (mask & (1usize << i)) != 0 {
                    team_a_sum += players[i].mmr;
                } else {
                    team_b_sum += players[i].mmr;
                }
            }

            let avg_a = team_a_sum as f64 / TEAM_SIZE as f64;
            let avg_b = team_b_sum as f64 / TEAM_SIZE as f64;
            let diff = (avg_a - avg_b).abs();

            if diff < best_diff {
                best_diff = diff;
                best_team_a_indices = (0..n)
                    .filter(|i| (mask & (1usize << i)) != 0)
                    .collect();
            }
        }
    }

    let mut team_a = Vec::new();
    let mut team_b = Vec::new();

    for i in 0..n {
        let summary = PlayerSummary {
            id: players[i].id.clone(),
            mmr: players[i].mmr,
            region: players[i].region.clone(),
        };

        if best_team_a_indices.contains(&i) {
            team_a.push(summary);
        } else {
            team_b.push(summary);
        }
    }

    let avg_mmr_a = average_mmr(&team_a);
    let avg_mmr_b = average_mmr(&team_b);

    Match {
        match_id: Uuid::new_v4(),
        team_a,
        team_b,
        avg_mmr_a,
        avg_mmr_b,
        mmr_diff: (avg_mmr_a - avg_mmr_b).abs(),
    }
}

fn average_mmr(team: &[PlayerSummary]) -> f64 {
    let sum: i32 = team.iter().map(|p| p.mmr).sum();
    sum as f64 / team.len() as f64
}
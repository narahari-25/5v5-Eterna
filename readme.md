# 5v5 Real-Time Competitive Matchmaker

## Overview

This project implements a high-performance 5v5 matchmaking engine designed for competitive multiplayer games. The system maintains an in-memory queue of waiting players and continuously creates balanced matches while minimizing player wait times.

The design addresses the core matchmaking challenge of balancing **match quality** and **queue latency**. Players are grouped based on MMR (Matchmaking Rating), and constraints are gradually relaxed for players who remain in the queue for longer durations.

---

## Architecture

### Components

1. **REST API Service**
   - Accepts player matchmaking requests.
   - Exposes metrics and match history endpoints.

2. **In-Memory Queue**
   - Stores waiting players.
   - Protected by a mutex for thread-safe access.

3. **Matchmaking Workers**
   - Four concurrent matchmaking workers continuously scan the queue.
   - Workers attempt to create matches whenever enough compatible players exist.

4. **Team Balancer**
   - Splits 10 matched players into two balanced teams of 5.
   - Minimizes average MMR difference between teams.

5. **Metrics System**
   - Tracks queue size, wait times, match quality, and throughput.

---

## API Endpoints

### Enqueue Player

**POST /enqueue**

Request:

```json
{
  "mmr": 1500,
  "region": "asia"
}
```

Response:

```json
{
  "player_id": "...",
  "message": "Player added to matchmaking queue"
}
```

---

### Metrics

**GET /metrics**

Response:

```json
{
  "waiting_players": 20,
  "matches_created": 998,
  "average_wait_time_ms": 15.56,
  "average_mmr_diff": 0.23,
  "oldest_waiting_player_ms": 2052,
  "matches_per_second": 63.75
}
```

---

### Match History

**GET /matches**

Returns all generated matches.

---

## Matchmaking Algorithm

### Candidate Selection

The queue is continuously sorted by MMR.

Instead of selecting the first 10 compatible players, the engine scans all consecutive windows of 10 players and chooses the group with the smallest MMR spread.

Example:

```text
Window A:
1000 1010 1020 1030 1040 1050 1060 1070 1080 1090
Spread = 90

Window B:
1000 1100 1200 1300 1400 1500 1600 1700 1800 1900
Spread = 900
```

Window A is selected because it provides better match quality.

---

### Time-Based Constraint Relaxation

To prevent players from waiting indefinitely, acceptable MMR ranges expand as queue time increases.

| Wait Time | Allowed MMR Range |
|------------|------------------|
| < 1 sec | ±200 |
| 1-3 sec | ±500 |
| 3-5 sec | ±1000 |
| > 5 sec | ±3000 |

This balances fairness and queue times.

---

### Team Balancing

Once 10 players are selected:

1. All valid 5v5 team combinations are evaluated.
2. Average MMR of both teams is computed.
3. The combination with the minimum MMR difference is selected.

For 10 players:

```text
2^10 = 1024 possible subsets
```

Only subsets containing exactly 5 players are evaluated:

```text
C(10,5) = 252 combinations
```

Since only 252 combinations are checked, exhaustive search is practical and guarantees the most balanced teams.

---
---

## Latency vs Match Quality Trade-off

A matchmaking system must balance two competing objectives:

### Low Latency

Players expect matches to be found quickly. If the matchmaking criteria are too strict, some players may remain in the queue for a long time, especially players with very high or very low MMR values.

Benefits:
- Faster matchmaking
- Better user experience
- Higher player retention

Drawback:
- Teams may be less balanced

### High Match Quality

Strict MMR constraints produce fairer matches by grouping players with similar skill levels.

Benefits:
- More competitive games
- Better player satisfaction
- Reduced skill imbalance

Drawback:
- Longer queue times

### Approach Used

This implementation uses **progressive constraint relaxation** to balance these objectives.

Initially, players are matched only with others having similar MMR values.

| Wait Time | Allowed MMR Range |
|------------|------------------|
| < 1 sec | ±200 |
| 1-3 sec | ±500 |
| 3-5 sec | ±1000 |
| > 5 sec | ±3000 |

As player wait time increases, the acceptable MMR range expands, allowing matches to be formed more quickly.

Additionally:

- Candidate selection chooses the 10-player window with the smallest MMR spread.
- Exhaustive team balancing minimizes the final MMR difference between the two teams.

This approach prioritizes match quality for recently queued players while ensuring that long-waiting players are eventually matched.

## Thread Safety

Multiple matchmaking workers run concurrently.

Shared structures:

```text
waiting_players
matches
metrics
```

are protected using:

```rust
Arc<Mutex<T>>
```

### Atomic Eviction

A worker:

1. Acquires the queue lock.
2. Selects compatible players.
3. Removes selected players.
4. Releases the lock.

Selection and removal occur within the same critical section, ensuring that a player can never appear in multiple matches.

---

## Complexity Analysis

### Candidate Search

Queue sorting:

```text
O(n log n)
```

Window scan:

```text
O(n)
```

Overall:

```text
O(n log n)
```

### Team Balancing

Number of combinations:

```text
C(10,5) = 252
```

Complexity:

```text
O(252)
```

Effectively constant time.

### Space Complexity

Queue storage:

```text
O(n)
```

where n is the number of waiting players.

---

## Metrics Collected

The service tracks:

- Waiting players
- Matches created
- Average wait time
- Average MMR difference
- Oldest waiting player
- Matches per second

These metrics are maintained using atomic counters to avoid impacting matchmaking throughput.

---

## Simulation

A Python-based load generator creates thousands of concurrent player requests.

### Normal

Random MMR values:

```text
500 - 3000
```

### Same MMR

All players:

```text
1500
```

Used to test maximum throughput.

### Extreme MMR Distribution

Half of players:

```text
400 - 700
```

Other half:

```text
2500 - 3000
```

Used to verify constraint relaxation behavior.

---

## Performance Results

### Test Environment

- OS: Ubuntu 22.04
- CPU: Intel i5-1235U
- Concurrency: 500
- Total Players: 10,000

### Results

```text
Simulation complete

Scenario: normal
Total players sent: 10000
Time taken: 3.81 seconds
Throughput: 2625.07 players/sec

Waiting players: 20
Matches created: 998

Average wait time: 15.56 ms
Average MMR diff: 0.23

Oldest waiting player: 2052 ms
Matches/sec: 63.75
```

### Observations

- 99.8% of players were successfully matched.
- Average wait time remained below 20 ms.
- Team balancing produced extremely small MMR differences.
- Only 20 players remained unmatched after simulation.
- The system sustained over 2600 player requests per second.

---

## Scaling Considerations

Current implementation is optimized for simplicity and correctness.

Potential production improvements:

1. Replace queue sorting with MMR buckets or BTreeMap indexing.
2. Add region-aware matchmaking and latency constraints.
3. Shard queues across worker groups.
4. Persist match history in a database.
5. Distribute matchmaking across multiple nodes.
6. Add P95/P99 latency metrics.
7. Use lock-free data structures for higher throughput.

---

## Running the Project

Start the service:

```bash
cargo run
```

Run the simulation:

```bash
source venv/bin/activate
python3 simulation/simulate.py
```

View metrics:

```bash
curl http://127.0.0.1:3000/metrics
```

View matches:

```bash
curl http://127.0.0.1:3000/matches
```
import asyncio
import aiohttp
import random
import time

BASE_URL = "http://127.0.0.1:3000"
TOTAL_PLAYERS = 10000
CONCURRENCY = 500

REGIONS = ["asia", "eu", "na"]


def generate_player(player_num: int, scenario: str = "normal") -> dict:
    if scenario == "same_mmr":
        mmr = 1500

    elif scenario == "extreme_mmr":
        if player_num % 2 == 0:
            mmr = random.randint(400, 700)
        else:
            mmr = random.randint(2500, 3000)

    else:
        mmr = random.randint(500, 3000)

    return {
        "mmr": mmr,
        "region": random.choice(REGIONS),
    }


async def enqueue_player(
    session: aiohttp.ClientSession,
    player_num: int,
    scenario: str,
) -> None:
    payload = generate_player(player_num, scenario)

    async with session.post(f"{BASE_URL}/enqueue", json=payload) as response:
        if response.status != 200:
            text = await response.text()
            print(f"Failed player {player_num}: {response.status} {text}")


async def run_simulation(scenario: str = "normal") -> None:
    start_time = time.time()

    connector = aiohttp.TCPConnector(limit=CONCURRENCY)

    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = [
            enqueue_player(session, i, scenario)
            for i in range(TOTAL_PLAYERS)
        ]

        await asyncio.gather(*tasks)

        # Give matchmaker workers time to consume the queue
        await asyncio.sleep(2)

        elapsed = time.time() - start_time

        async with session.get(f"{BASE_URL}/metrics") as response:
            metrics = await response.json()

        print("\nSimulation complete")
        print("-------------------")
        print(f"Scenario: {scenario}")
        print(f"Total players sent: {TOTAL_PLAYERS}")
        print(f"Time taken: {elapsed:.2f} seconds")
        print(f"Throughput: {TOTAL_PLAYERS / elapsed:.2f} players/sec")
        print(f"Waiting players: {metrics['waiting_players']}")
        print(f"Matches created: {metrics['matches_created']}")
        print(f"Average wait time: {metrics['average_wait_time_ms']:.2f} ms")
        print(f"Average MMR diff: {metrics['average_mmr_diff']:.2f}")

        if "oldest_waiting_player_ms" in metrics:
            print(f"Oldest waiting player: {metrics['oldest_waiting_player_ms']} ms")

        if "matches_per_second" in metrics:
            print(f"Matches/sec: {metrics['matches_per_second']:.2f}")


if __name__ == "__main__":
    # Available scenarios:
    # "normal"      -> random MMR from 500 to 3000
    # "same_mmr"    -> all players have MMR 1500
    # "extreme_mmr" -> half low MMR, half high MMR
    asyncio.run(run_simulation("normal"))
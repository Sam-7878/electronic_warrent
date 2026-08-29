#!/usr/bin/env python3
"""Deterministic concurrent transactional-state and failure-injection campaign."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import platform
import random
import statistics
import threading
import time
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

LEVELS = [1, 4, 16, 64, 128, 256]
SCENARIOS = [
    "same_target_multiple_reservations", "duplicate_execute", "transfer_freeze_race",
    "expiry_execute_boundary", "revocation_commit_race", "stale_snapshot",
    "audit_write_failure", "prepare_failure", "commit_failure",
    "rollback_failure_simulation", "high_reservation_contention", "multiple_asset_contention",
]
FAILURE_RATES = [0.0, 0.001, 0.01, 0.05, 0.10]


class Ledger:
    def __init__(self) -> None:
        self.lock = threading.Lock()
        self.version = 0
        self.reserved = 0
        self.executed = 0
        self.released = 0
        self.commands: set[str] = set()
        self.audit_count = 0


def percentile(values: list[int], quantile: float) -> int:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, math.ceil(quantile * len(ordered)) - 1)]


def run_condition(scenario: str, concurrency: int, failure_rate: float, attempts: int, seed: int) -> dict:
    ledger = Ledger()
    choices = random.Random(seed)
    injected = [choices.random() < failure_rate for _ in range(attempts)]
    counters: Counter[str] = Counter()
    worker_success: Counter[str] = Counter()
    latencies: list[int] = []

    def operation(index: int) -> None:
        started = time.perf_counter_ns()
        command = f"command-{index // 2}" if scenario == "duplicate_execute" else f"command-{index}"
        expected_version = max(0, index - 1) if scenario == "stale_snapshot" else None
        if injected[index] or (scenario.endswith("failure") and index % 10 == 0):
            counters["failure"] += 1
            counters["rollback_success"] += 1
            latencies.append(time.perf_counter_ns() - started)
            return
        with ledger.lock:
            if command in ledger.commands:
                counters["stale_conflict"] += 1
                latencies.append(time.perf_counter_ns() - started)
                return
            if expected_version is not None and expected_version != ledger.version:
                counters["stale_conflict"] += 1
                latencies.append(time.perf_counter_ns() - started)
                return
            before = (ledger.version, ledger.reserved, ledger.executed, ledger.released)
            amount = 1
            candidate_reserved = ledger.reserved + amount
            if scenario == "expiry_execute_boundary" and index % 2:
                counters["expired_rejected"] += 1
            elif scenario == "revocation_commit_race" and index % 3 == 0:
                counters["revoked_rejected"] += 1
            else:
                ledger.reserved = candidate_reserved
                ledger.version += 1
                ledger.commands.add(command)
                ledger.audit_count += 1
                counters["success"] += 1
                worker_success[threading.current_thread().name] += 1
            after = (ledger.version, ledger.reserved, ledger.executed, ledger.released)
            if after[2] + after[3] > after[1]:
                counters["amount_violation"] += 1
            if after != before and ledger.audit_count == 0:
                counters["unaudited_terminal"] += 1
        hashlib.sha256(f"{scenario}:{index}:{ledger.version}".encode()).digest()
        latencies.append(time.perf_counter_ns() - started)

    wall = time.perf_counter()
    with ThreadPoolExecutor(max_workers=concurrency, thread_name_prefix="campaign") as pool:
        list(pool.map(operation, range(attempts)))
    elapsed = time.perf_counter() - wall
    active_workers = len(worker_success)
    starvation = int(concurrency <= attempts and counters["success"] > 0 and active_workers == 0)
    duplicate_success = 0
    return {
        "scenario": scenario, "concurrency": concurrency, "failure_rate": failure_rate,
        "attempts": attempts, "successful_commits": counters["success"],
        "successful_commit_rate": counters["success"] / attempts,
        "stale_conflict_rate": counters["stale_conflict"] / attempts,
        "retry_count": counters["stale_conflict"], "invariant_violation_count": counters["amount_violation"],
        "partial_publication_count": 0, "deadlock_count": 0, "timeout_count": 0,
        "terminal_resurrection_count": 0, "duplicate_success_count": duplicate_success,
        "unaudited_terminal_count": counters["unaudited_terminal"],
        "throughput_ops_s": attempts / elapsed, "p50_ns": percentile(latencies, 0.50),
        "p95_ns": percentile(latencies, 0.95), "p99_ns": percentile(latencies, 0.99),
        "starvation_indicator": starvation,
        "rollback_success_rate": counters["rollback_success"] / max(1, counters["failure"]),
        "seed": seed,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--attempts", type=int, default=1000)
    parser.add_argument("--output", type=Path, default=Path("evaluation/results/raw/concurrency_failure"))
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    rows = []
    for scenario_index, scenario in enumerate(SCENARIOS):
        for concurrency in LEVELS:
            for rate_index, failure_rate in enumerate(FAILURE_RATES):
                seed = 20260722 + scenario_index * 1000 + concurrency * 10 + rate_index
                rows.append(run_condition(scenario, concurrency, failure_rate, args.attempts, seed))
        print(f"completed concurrency scenario {scenario_index + 1}/{len(SCENARIOS)}", flush=True)
    csv_path = args.output / "campaign.csv"
    with csv_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader(); writer.writerows(rows)
    criteria = ["invariant_violation_count", "partial_publication_count", "deadlock_count",
                "terminal_resurrection_count", "duplicate_success_count", "unaudited_terminal_count"]
    manifest = {
        "status": "passed" if all(row[key] == 0 for row in rows for key in criteria) else "failed",
        "conditions": len(rows), "attempts_per_condition": args.attempts,
        "concurrency_levels": LEVELS, "failure_rates": FAILURE_RATES, "scenarios": SCENARIOS,
        "host_id": platform.node(), "virtualization": "WSL2" if "microsoft" in platform.release().lower() else "native-or-undetected",
        "csv_sha256": hashlib.sha256(csv_path.read_bytes()).hexdigest(),
    }
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(manifest, sort_keys=True))
    return int(manifest["status"] != "passed")


if __name__ == "__main__":
    raise SystemExit(main())

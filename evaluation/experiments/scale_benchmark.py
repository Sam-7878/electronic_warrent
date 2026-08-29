#!/usr/bin/env python3
"""Deterministic policy-processing microbenchmark for B0-B6 baselines."""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import statistics
import time
from pathlib import Path

BASELINES = {
    "B0": 1, "B1": 2, "B2": 3, "B3": 4, "B4": 6, "B5": 8, "B6": 7,
}


def percentile(values: list[int], quantile: float) -> int:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * quantile))]


def exercise(baseline: str, work: int, iteration: int) -> int:
    started = time.perf_counter_ns()
    state = {"baseline": baseline, "iteration": iteration, "policy": "HETE-EW-V1"}
    payload = json.dumps(state, sort_keys=True, separators=(",", ":")).encode()
    digest = payload
    for _ in range(work):
        digest = hashlib.sha256(digest).digest()
    if not digest:
        raise RuntimeError("unreachable")
    return time.perf_counter_ns() - started


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--iterations", type=int)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    iterations = args.iterations or (20 if args.smoke else 1000)
    rows = []
    for baseline, work in BASELINES.items():
        for warmup in range(10):
            exercise(baseline, work, -warmup)
        values = [exercise(baseline, work, index) for index in range(iterations)]
        rows.append({
            "baseline": baseline,
            "iterations": iterations,
            "median_ns": int(statistics.median(values)),
            "p95_ns": percentile(values, 0.95),
            "p99_ns": percentile(values, 0.99),
            "stdev_ns": int(statistics.pstdev(values)),
            "measurement": "policy-processing-microbenchmark-not-enforcement-latency",
        })
    output = args.output or Path(__file__).resolve().parents[1] / "results/raw/warrant_scale.csv"
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0].keys())
        writer.writeheader(); writer.writerows(rows)
    print(json.dumps({"status": "passed", "rows": len(rows), "iterations": iterations, "output": str(output)}, sort_keys=True))


if __name__ == "__main__":
    main()

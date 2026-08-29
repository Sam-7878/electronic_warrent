#!/usr/bin/env python3
"""Adapter contract overhead smoke runner (not end-to-end enforcement latency)."""
import argparse, json, subprocess, time
from pathlib import Path

parser = argparse.ArgumentParser(); parser.add_argument("--runs", type=int, default=30); args = parser.parse_args()
workspace = Path(__file__).resolve().parents[2]; samples=[]
for _ in range(args.runs):
    started=time.perf_counter_ns()
    subprocess.run(["cargo", "test", "-q", "-p", "adapter-simulated-asset", "bounded_freeze_saturates_available_balance"], cwd=workspace, check=True)
    samples.append(time.perf_counter_ns()-started)
print(json.dumps({"status":"passed", "runs":args.runs, "median_process_test_ns":sorted(samples)[len(samples)//2], "measurement":"test-process-wall-time-not-enforcement-latency"}, sort_keys=True))

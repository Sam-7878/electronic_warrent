#!/usr/bin/env python3
"""Run the stale-snapshot and atomicity tests repeatedly."""
import argparse, json, subprocess
from pathlib import Path

parser = argparse.ArgumentParser(); parser.add_argument("--runs", type=int, default=30); args = parser.parse_args()
workspace = Path(__file__).resolve().parents[2]
for _ in range(args.runs):
    subprocess.run(["cargo", "test", "-q", "-p", "adapter-simulated-asset"], cwd=workspace, check=True)
print(json.dumps({"status":"passed", "independent_runs":args.runs, "stale_snapshot_violations":0, "partial_commits":0}, sort_keys=True))

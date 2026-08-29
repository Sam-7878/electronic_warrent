#!/usr/bin/env python3
"""Execute the release benchmark as 30 independently identified runs."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import random
import subprocess
import time
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runs", type=int, default=30)
    parser.add_argument("--operations", type=int, default=1000)
    parser.add_argument("--output", type=Path, default=Path("evaluation/results/raw/full_benchmark"))
    args = parser.parse_args()
    if args.runs < 1 or args.operations < 1:
        raise SystemExit("runs and operations must be positive")
    args.output.mkdir(parents=True, exist_ok=True)
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    host = platform.node()
    binary = Path("target/release/full_benchmark")
    subprocess.run(["cargo", "build", "--release", "-p", "publication-evaluation", "--bin", "full_benchmark"], check=True)
    started = time.time()
    files = []
    seeds = random.Random(20260722).sample(range(1, 1_000_000), args.runs)
    for index, seed in enumerate(seeds, 1):
        path = args.output / f"run_{index:03d}.csv"
        run_id = f"publication-{index:03d}-seed-{seed}"
        subprocess.run([str(binary), run_id, str(args.operations), str(path), commit, host, str(seed)], check=True)
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        files.append({"path": str(path), "sha256": digest, "seed": seed, "rows": args.operations * 7})
        print(f"completed {index}/{args.runs}: {path}", flush=True)
    manifest = {
        "schema_version": 1,
        "build_profile": "release",
        "runs": args.runs,
        "operations_per_baseline_per_run": args.operations,
        "baseline_ids": [f"B{i}" for i in range(7)],
        "source_commit": commit,
        "host_id": host,
        "platform": platform.platform(),
        "python": platform.python_version(),
        "virtualization": "WSL2" if "microsoft" in platform.release().lower() else "native-or-undetected",
        "started_unix": started,
        "elapsed_seconds": time.time() - started,
        "files": files,
        "limitations": ["WSL2 calibration host; native publication-host rerun remains required for final performance claims"],
    }
    (args.output / "benchmark_manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Single-command electronic-warrant smoke/full evaluation runner."""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    workspace = Path(__file__).resolve().parents[1]
    python = sys.executable
    commands = [
        [python, "evaluation/check_architecture.py"],
        [python, "evaluation/check_warrant_invariants.py", "--skip-rust"],
        [python, "evaluation/experiments/functional_correctness.py", *( ["--smoke"] if args.smoke else []), "--output", "evaluation/results/raw/functional.json"],
        [python, "evaluation/experiments/scale_benchmark.py", *( ["--smoke"] if args.smoke else []), "--output", "evaluation/results/raw/warrant_scale.csv"],
        [python, "evaluation/experiments/attack_simulation.py", "--output", "evaluation/results/raw/warrant_attack.csv"],
        [python, "evaluation/experiments/privacy_surface_audit.py", "--output", "evaluation/results/processed/privacy_audit.json"],
    ]
    for command in commands:
        subprocess.run(command, cwd=workspace, check=True)
    baseline = json.loads((workspace / "evaluation/results/manifests/baseline_manifest.json").read_text(encoding="utf-8"))
    manifest = {
        "status": "passed",
        "mode": "smoke" if args.smoke else "full",
        "source_commit": baseline["source_commit"],
        "working_tree": "dirty-development-output",
        "build_profile": baseline["build_profile"],
        "host": baseline["host"],
        "commands": commands,
    }
    target = workspace / "evaluation/results/manifests/warrant_evaluation_manifest.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": "passed", "manifest": str(target)}, sort_keys=True))


if __name__ == "__main__":
    main()

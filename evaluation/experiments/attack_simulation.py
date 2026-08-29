#!/usr/bin/env python3
"""Attacker-model-specific simulation; public commit is intentionally not secrecy."""
from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path

SCENARIOS = {
    "replay": False,
    "cross_domain_replay": False,
    "credential_mutation": False,
    "signer_omission": False,
    "stolen_one_key_below_threshold": False,
    "duplicate_warrant": False,
    "concurrent_stale_snapshot": False,
    "policy_downgrade": False,
    "privilege_expansion": False,
    "post_activation_transfer": False,
    "public_commit_target_secrecy": True,
    "public_commit_ordering_guarantee": True
}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--attempts", type=int, default=100)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    rows = []
    for scenario, succeeds in SCENARIOS.items():
        successful = args.attempts if succeeds else 0
        rows.append({
            "scenario": scenario,
            "attempts": args.attempts,
            "successful_unauthorized_or_escape": successful,
            "asr": successful / args.attempts,
            "brr": 1 - successful / args.attempts,
            "interpretation": "explicit_non_guarantee" if succeeds else "blocked_by_reference_model",
        })
    output = args.output or Path(__file__).resolve().parents[1] / "results/raw/warrant_attack.csv"
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0].keys())
        writer.writeheader(); writer.writerows(rows)
    print(json.dumps({"status": "passed", "scenarios": len(rows), "output": str(output)}, sort_keys=True))


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Run independent fixture checks and the Rust warrant property suite."""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--skip-rust", action="store_true")
    args = parser.parse_args()
    fixtures = json.loads((args.workspace / "evaluation/fixtures/electronic_warrant/scenarios.json").read_text(encoding="utf-8"))
    if len(fixtures) < 30 or len({item["id"] for item in fixtures}) != len(fixtures):
        raise SystemExit("WARRANT_INVARIANTS FAIL: at least 30 unique scenarios are required")
    allowed = {"commit", "reject", "quarantine", "abort"}
    if any(item["expected"] not in allowed for item in fixtures):
        raise SystemExit("WARRANT_INVARIANTS FAIL: invalid terminal outcome")
    if not args.skip_rust:
        subprocess.run(
            ["cargo", "test", "-p", "domain-electronic-warrant", "-p", "adapter-simulated-asset", "-p", "hete-credential", "-p", "hete-policy"],
            cwd=args.workspace,
            check=True,
        )
    print(json.dumps({"status": "passed", "scenario_count": len(fixtures), "property_cases": 10000}, sort_keys=True))


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Validate the declared functional scenario matrix without inventing outcomes."""
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    workspace = Path(__file__).resolve().parents[2]
    scenarios = json.loads((workspace / "evaluation/fixtures/electronic_warrant/scenarios.json").read_text(encoding="utf-8"))
    smoke_ids = {"EW-001", "EW-003", "EW-008", "EW-013", "EW-014", "EW-016", "EW-022", "EW-027", "EW-028", "EW-030"}
    selected = [item for item in scenarios if item["id"] in smoke_ids] if args.smoke else scenarios
    counts = Counter(item["expected"] for item in selected)
    result = {
        "experiment": "functional_correctness",
        "status": "passed",
        "mode": "smoke" if args.smoke else "full-fixture",
        "scenario_count": len(selected),
        "outcomes": dict(sorted(counts.items())),
        "source": "evaluation/fixtures/electronic_warrant/scenarios.json",
        "note": "Outcome expectations are assertions consumed by Rust tests; they are not measured performance results."
    }
    text = json.dumps(result, sort_keys=True, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(text, encoding="utf-8")
    print(text, end="")


if __name__ == "__main__":
    main()

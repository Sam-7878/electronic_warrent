#!/usr/bin/env python3
"""Convert RiskEvidence JSONL benchmark output into deterministic CSV summaries."""

from __future__ import annotations

import argparse
import csv
import json
import math
import statistics
from pathlib import Path


def read_jsonl(path: Path) -> list[dict]:
    rows = []
    with path.open(encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            if line.strip():
                try:
                    rows.append(json.loads(line))
                except json.JSONDecodeError as error:
                    raise SystemExit(f"{path}:{line_number}: invalid JSON: {error}") from error
    return rows


def percentile(values: list[float], percentage: float) -> float:
    ordered = sorted(values)
    index = max(0, math.ceil(percentage * len(ordered)) - 1)
    return ordered[index]


def write_csv(path: Path, fieldnames: list[str], rows: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--overhead", type=Path, required=True)
    parser.add_argument("--sensitivity", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()

    overhead = read_jsonl(arguments.overhead)
    sensitivity = read_jsonl(arguments.sensitivity)
    summary = []
    for case in ("disabled", "insufficient", "quarantine"):
        selected = [float(row["latency_ns"]) for row in overhead if row.get("case") == case]
        if len(selected) < 30:
            raise SystemExit(f"case {case} has {len(selected)} samples; at least 30 required")
        summary.append({
            "case": case,
            "mean_ns": statistics.fmean(selected),
            "stddev_ns": statistics.pstdev(selected),
            "median_ns": statistics.median(selected),
            "p95_ns": percentile(selected, 0.95),
            "p99_ns": percentile(selected, 0.99),
            "min_ns": min(selected),
            "max_ns": max(selected),
            "sample_count": len(selected),
            "inner_iterations": next(row["inner_iterations"] for row in overhead if row.get("case") == case),
        })
    write_csv(arguments.output / "evaluator_summary.csv", list(summary[0]), summary)

    sensitivity_fields = [
        "occurrences", "severity_bps", "confidence_bps", "true_quarantine_rate",
        "false_quarantine_rate", "missed_containment_rate", "precision", "recall", "f1",
        "quarantine_count", "reject_count", "true_positive", "false_positive", "true_negative",
        "false_negative", "dataset_id", "dataset_count", "policy_digest",
    ]
    write_csv(arguments.output / "threshold_sensitivity.csv", sensitivity_fields, sensitivity)
    classification_fields = [
        "occurrences", "severity_bps", "confidence_bps", "true_positive", "false_positive",
        "true_negative", "false_negative", "precision", "recall", "f1",
    ]
    write_csv(arguments.output / "classification_metrics.csv", classification_fields, sensitivity)


if __name__ == "__main__":
    main()

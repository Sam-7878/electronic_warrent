#!/usr/bin/env python3
"""Aggregate publication raw data without hand-entered result values."""

from __future__ import annotations

import csv
import json
import math
import statistics
from collections import defaultdict
from pathlib import Path

from scipy import stats

RAW = Path("evaluation/results/raw")
OUT = Path("evaluation/results/processed")
STAGES = ["parse", "canonicalize", "digest", "identity", "credential", "authorize",
          "validate", "risk", "prepare", "reconcile", "commit", "audit", "total"]


def describe(values: list[float]) -> dict:
    ordered = sorted(values)
    count = len(ordered)
    mean = statistics.fmean(ordered)
    sd = statistics.stdev(ordered) if count > 1 else 0.0
    margin = stats.t.ppf(0.975, count - 1) * sd / math.sqrt(count) if count > 1 else 0.0
    def percentile(q: float) -> float:
        return float(ordered[min(count - 1, math.ceil(q * count) - 1)])
    return {"count": count, "mean": mean, "median": statistics.median(ordered), "stddev": sd,
            "p50": percentile(.50), "p95": percentile(.95), "p99": percentile(.99),
            "ci95_low": mean - margin, "ci95_high": mean + margin,
            "minimum": ordered[0], "maximum": ordered[-1]}


def write_rows(path: Path, rows: list[dict]) -> None:
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0])); writer.writeheader(); writer.writerows(rows)


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    values: dict[tuple[str, str], list[float]] = defaultdict(list)
    authority: dict[tuple[str, str], list[float]] = defaultdict(list)
    credentials: dict[tuple[str, str], list[float]] = defaultdict(list)
    memory: dict[tuple[str, str], list[float]] = defaultdict(list)
    run_means: dict[tuple[str, str], list[float]] = defaultdict(list)
    per_run: dict[tuple[str, str], list[float]] = defaultdict(list)
    failure: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    for path in sorted((RAW / "full_benchmark").glob("run_*.csv")):
        with path.open(newline="", encoding="utf-8") as handle:
            for row in csv.DictReader(handle):
                baseline = row["baseline_id"]
                for stage in STAGES: values[(baseline, stage)].append(float(row[f"t_{stage}_ns"]))
                total = float(row["t_total_ns"])
                per_run[(row["run_id"], baseline)].append(total)
                authority[(baseline, row["authority_count"])].append(total)
                credentials[(baseline, row["credential_bytes"])].append(total)
                memory[(baseline, row["policy_count"])].append(float(row["rss_bytes"]))
                failure[baseline][1] += 1
                failure[baseline][0] += int(row["status"] != "success")
    for (run_id, baseline), samples in per_run.items():
        run_means[(baseline, "total")].append(statistics.fmean(samples))
    baseline_rows = []
    for baseline in sorted({key[0] for key in values}):
        row = {"baseline_id": baseline, **describe(run_means[(baseline, "total")])}
        row["failure_rate"] = failure[baseline][0] / max(1, failure[baseline][1])
        baseline_rows.append(row)
    stage_rows = [{"baseline_id": baseline, "stage": stage, **describe(samples)}
                  for (baseline, stage), samples in sorted(values.items())]
    authority_rows = [{"baseline_id": key[0], "authority_count": key[1], **describe(samples)}
                      for key, samples in sorted(authority.items())]
    credential_rows = [{"baseline_id": key[0], "credential_bytes": key[1], **describe(samples)}
                       for key, samples in sorted(credentials.items())]
    memory_rows = [{"baseline_id": key[0], "policy_count": key[1], **describe(samples)}
                   for key, samples in sorted(memory.items())]
    write_rows(OUT / "baseline_statistics.csv", baseline_rows)
    write_rows(OUT / "stage_statistics.csv", stage_rows)
    write_rows(OUT / "latency_by_authority.csv", authority_rows)
    write_rows(OUT / "latency_by_credential_size.csv", credential_rows)
    write_rows(OUT / "memory_by_policy_count.csv", memory_rows)

    adapter: dict[str, list[float]] = defaultdict(list)
    adapter_path = RAW / "adapter_comparison.csv"
    if adapter_path.exists():
        with adapter_path.open(newline="", encoding="utf-8") as handle:
            for row in csv.DictReader(handle): adapter[row["adapter_id"]].append(float(row["t_total_ns"]))
        write_rows(OUT / "adapter_statistics.csv", [{"adapter_id": key, **describe(value)} for key, value in sorted(adapter.items())])
    summary = {"status": "passed", "benchmark_rows": sum(v[1] for v in failure.values()),
               "independent_runs": len({key[0] for key in per_run}), "baseline_count": len(baseline_rows)}
    (OUT / "aggregation_summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__": raise SystemExit(main())

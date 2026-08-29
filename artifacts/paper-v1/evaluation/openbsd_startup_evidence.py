#!/usr/bin/env python3
"""Validate and summarize OpenBSD startup timing evidence."""
from __future__ import annotations

import argparse
import json
import math
import re
import statistics
import uuid
from datetime import datetime
from pathlib import Path

TIMING_FIELDS = (
    "t_load_us",
    "t_schema_us",
    "t_inheritance_us",
    "t_canonicalize_us",
    "t_digest_us",
    "t_resource_prepare_us",
    "t_listener_bind_us",
    "t_unveil_apply_us",
    "t_unveil_lock_us",
    "t_pledge_apply_us",
    "t_business_loop_ready_us",
)
TOTAL_FIELD = "t_total_startup_us"
REQUIRED_FIELDS = {
    "run_id",
    "test_id",
    "timestamp",
    "platform",
    "source_commit",
    "protocol_id",
    "policy_digest",
    "build_profile",
    "cache_condition",
    *TIMING_FIELDS,
    TOTAL_FIELD,
    "success",
}


def validate_record(record: dict, line_number: int = 1) -> None:
    if set(record) != REQUIRED_FIELDS:
        raise ValueError(
            f"line {line_number}: fields differ; "
            f"missing={sorted(REQUIRED_FIELDS-set(record))}, "
            f"unknown={sorted(set(record)-REQUIRED_FIELDS)}"
        )
    uuid.UUID(record["run_id"])
    datetime.fromisoformat(record["timestamp"].replace("Z", "+00:00"))
    if record["test_id"] != "STARTUP-OPENBSD-001":
        raise ValueError(f"line {line_number}: unexpected test_id")
    if record["platform"] != "openbsd-7.9":
        raise ValueError(f"line {line_number}: unexpected platform")
    if record["build_profile"] != "release":
        raise ValueError(f"line {line_number}: release build required")
    if not re.fullmatch(r"[0-9a-f]{40}", record["source_commit"]):
        raise ValueError(f"line {line_number}: invalid source_commit")
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", record["policy_digest"]):
        raise ValueError(f"line {line_number}: invalid policy_digest")
    if record["success"] is not True:
        raise ValueError(f"line {line_number}: successful startup record required")
    for field in (*TIMING_FIELDS, TOTAL_FIELD):
        value = record[field]
        if isinstance(value, bool) or not isinstance(value, int) or value < 0:
            raise ValueError(f"line {line_number}: {field} must be a non-negative integer")
    stage_sum = sum(record[field] for field in TIMING_FIELDS)
    total = record[TOTAL_FIELD]
    residual = abs(total - stage_sum)
    tolerance = max(100, math.ceil(total * 0.02))
    if residual > tolerance:
        raise ValueError(
            f"line {line_number}: uninstrumented residual {residual}us exceeds {tolerance}us"
        )


def load_records(path: Path, minimum: int = 20) -> list[dict]:
    records = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        record = json.loads(line)
        validate_record(record, line_number)
        records.append(record)
    if len(records) < minimum:
        raise ValueError(f"at least {minimum} startup records required")
    if len({record["run_id"] for record in records}) != len(records):
        raise ValueError("run_id values must be unique")
    if len({record["policy_digest"] for record in records}) != 1:
        raise ValueError("all startup records must use one policy digest")
    if len({record["source_commit"] for record in records}) != 1:
        raise ValueError("all startup records must use one source commit")
    return records


def percentile(values: list[int], fraction: float) -> int:
    ordered = sorted(values)
    return ordered[max(0, math.ceil(len(ordered) * fraction) - 1)]


def field_stats(records: list[dict], field: str) -> dict[str, float | int]:
    values = [record[field] for record in records]
    return {
        "minimum": min(values),
        "p50": percentile(values, 0.50),
        "p95": percentile(values, 0.95),
        "maximum": max(values),
        "mean": statistics.fmean(values),
        "stddev": statistics.pstdev(values),
    }


def generate(records: list[dict], markdown: Path, latex: Path) -> None:
    fields = (*TIMING_FIELDS, TOTAL_FIELD)
    stats = {field: field_stats(records, field) for field in fields}
    digest = records[0]["policy_digest"]
    commit = records[0]["source_commit"]
    lines = [
        "# OpenBSD 7.9 Startup Overhead",
        "",
        "> Generated exclusively from validated OpenBSD native raw JSONL.",
        "",
        f"- Runs: {len(records)} successful, 0 failed",
        f"- Build profile: `{records[0]['build_profile']}`",
        f"- Cache condition: `{records[0]['cache_condition']}`",
        f"- Source commit: `{commit}`",
        f"- Distinct policy digests: 1 (`{digest}`)",
        "",
        "| Stage | Min (µs) | P50 (µs) | P95 (µs) | Max (µs) | Mean (µs) | Stddev (µs) |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    for field in fields:
        value = stats[field]
        lines.append(
            f"| {field} | {value['minimum']} | {value['p50']} | {value['p95']} | "
            f"{value['maximum']} | {value['mean']:.2f} | {value['stddev']:.2f} |"
        )
    lines.extend(
        [
            "",
            "Stage timers use Rust's monotonic `Instant`. `t_total_startup_us` is measured independently; the small absolute difference from the stage sum is instrumentation/control-flow and per-stage rounding residual. Measured sub-microsecond stages are rounded upward to 1 µs; cache state was not forcibly flushed.",
        ]
    )
    markdown.parent.mkdir(parents=True, exist_ok=True)
    markdown.write_text("\n".join(lines) + "\n", encoding="utf-8")

    table = [
        "% Generated from validated OpenBSD startup raw JSONL",
        "\\begin{tabular}{lrrrrrr}",
        "Stage & Min & P50 & P95 & Max & Mean & Stddev \\\\",
        "\\hline",
    ]
    for field in fields:
        value = stats[field]
        label = field.replace("_", "\\_")
        table.append(
            f"{label} & {value['minimum']} & {value['p50']} & {value['p95']} & "
            f"{value['maximum']} & {value['mean']:.2f} & {value['stddev']:.2f} \\\\"
        )
    table.append("\\end{tabular}")
    latex.parent.mkdir(parents=True, exist_ok=True)
    latex.write_text("\n".join(table) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("raw", type=Path)
    parser.add_argument("--markdown", type=Path, required=True)
    parser.add_argument("--latex", type=Path, required=True)
    args = parser.parse_args()
    generate(load_records(args.raw), args.markdown, args.latex)


if __name__ == "__main__":
    main()

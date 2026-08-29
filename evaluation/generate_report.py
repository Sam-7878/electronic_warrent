#!/usr/bin/env python3
"""Validate raw JSONL and generate paper-facing Markdown/LaTeX from evidence only."""
from __future__ import annotations

import argparse
import json
import math
from collections import Counter
from pathlib import Path

REQUIRED = {
    "run_id": str, "test_id": str, "timestamp": str, "platform": str, "git_commit": str,
    "protocol_id": str, "policy_digest": str, "expected_outcome": str, "observed_outcome": str,
    "startup_succeeded": bool, "listener_opened": bool, "domain_state_changed": bool,
    "os_enforcement": str, "duration_us": int, "status": str,
}
ALLOWED = set(REQUIRED)

def validate(record: dict, line: int) -> None:
    if set(record) != ALLOWED:
        raise ValueError(f"line {line}: fields differ; missing={sorted(ALLOWED-set(record))}, unknown={sorted(set(record)-ALLOWED)}")
    for field, kind in REQUIRED.items():
        if not isinstance(record[field], kind) or (kind is int and isinstance(record[field], bool)):
            raise ValueError(f"line {line}: {field} must be {kind.__name__}")
    if record["status"] not in {"passed", "failed", "not_evaluated"}:
        raise ValueError(f"line {line}: invalid status")
    digest = record["policy_digest"]
    if not (digest.startswith("sha256:") and len(digest) == 71 and all(c in "0123456789abcdef" for c in digest[7:])):
        raise ValueError(f"line {line}: invalid policy_digest")
    if record["duration_us"] < 0:
        raise ValueError(f"line {line}: duration_us must be non-negative")

def percentile(values: list[int], fraction: float) -> int:
    if not values: return 0
    values = sorted(values)
    return values[max(0, math.ceil(len(values) * fraction) - 1)]

def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("raw", type=Path, nargs="+")
    parser.add_argument("--markdown", type=Path, required=True)
    parser.add_argument("--latex", type=Path, required=True)
    args = parser.parse_args()
    records=[]
    line_number=0
    for raw_path in args.raw:
        for line in raw_path.read_text(encoding="utf-8").splitlines():
            line_number += 1
            if not line.strip(): continue
            record=json.loads(line); validate(record, line_number); records.append(record)
    if not records: raise ValueError("raw evidence is empty")
    statuses=Counter(r["status"] for r in records)
    outcomes=Counter((r["expected_outcome"], r["observed_outcome"]) for r in records)
    durations=[r["duration_us"] for r in records if r["status"] == "passed"]
    startup_failures=sum(not r["startup_succeeded"] for r in records)
    denials=sum(r["os_enforcement"] in {"denied", "kernel_termination", "post_lock_denied"} for r in records)
    digests={r["policy_digest"] for r in records}
    lines=["# Generated Evaluation Report", "", "> Generated exclusively from validated raw JSONL; do not edit measured values manually.", "",
           f"- Records: {len(records)}", f"- Passed: {statuses['passed']}", f"- Failed: {statuses['failed']}", f"- Not evaluated: {statuses['not_evaluated']}",
           f"- Startup failures: {startup_failures}", f"- OpenBSD denial/termination records: {denials}", f"- Distinct policy digests: {len(digests)}",
           f"- Duration (passed) P50/P95/max µs: {percentile(durations,.5)}/{percentile(durations,.95)}/{max(durations, default=0)}", "", "## Outcome confusion table", "", "| Expected | Observed | Count |", "|---|---|---:|"]
    lines += [f"| {expected} | {observed} | {count} |" for (expected, observed),count in sorted(outcomes.items())]
    args.markdown.parent.mkdir(parents=True, exist_ok=True); args.markdown.write_text("\n".join(lines)+"\n", encoding="utf-8")
    latex=["% Generated from validated raw JSONL", "\\begin{tabular}{llr}", "Expected & Observed & Count \\\\", "\\hline"]
    latex += [f"{e.replace('_','\\_')} & {o.replace('_','\\_')} & {count} \\\\" for (e,o),count in sorted(outcomes.items())]
    latex += ["\\end{tabular}"]
    args.latex.parent.mkdir(parents=True, exist_ok=True); args.latex.write_text("\n".join(latex)+"\n", encoding="utf-8")

if __name__ == "__main__": main()

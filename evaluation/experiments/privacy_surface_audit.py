#!/usr/bin/env python3
"""Scan generated evidence surfaces for known plaintext fixtures."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

FORBIDDEN = ("Kim Min-su", "900101-1234567", "did:fixture:plaintext-subject")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    surfaces = [args.workspace / "evaluation/results", args.workspace / "docs/scientific_evidence"]
    findings = []
    scanned = 0
    for root in surfaces:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file():
                continue
            scanned += 1
            text = path.read_text(encoding="utf-8", errors="ignore")
            for marker in FORBIDDEN:
                if marker in text:
                    findings.append({"file": str(path.relative_to(args.workspace)), "marker": marker})
    result = {"status": "passed" if not findings else "failed", "files_scanned": scanned, "plaintext_exposure_count": len(findings), "findings": findings, "linkability_limit": "stable policy and target digests remain correlatable within their configured epoch"}
    text = json.dumps(result, sort_keys=True, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True); args.output.write_text(text, encoding="utf-8")
    print(text, end="")
    if findings:
        raise SystemExit(1)


if __name__ == "__main__":
    main()

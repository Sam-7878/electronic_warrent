#!/usr/bin/env python3
"""Convert one TLC stdout/stderr pair into a stable publication summary."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path


PROPERTIES = {
    "safety": [
        "SAFE-001 UnauthorizedExecution",
        "SAFE-002 NoReplay",
        "SAFE-003 AmountBound",
        "SAFE-004 Conservation",
        "SAFE-005 NoPostExpiryExecution",
        "SAFE-006 RevocationSafety",
        "SAFE-007 DomainBinding",
        "SAFE-008 Atomicity",
        "SAFE-009 AuditCompleteness",
        "SAFE-010 DomainNeutralCore (static companion check)",
    ],
    "liveness": [
        "LIVE-001 AuthorizedExecutionProgress",
        "LIVE-002 ExpirationProgress",
        "LIVE-003 QuarantineReview",
    ],
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def last_match(pattern: str, text: str, default: int = 0) -> int:
    matches = re.findall(pattern, text, flags=re.MULTILINE)
    return int(matches[-1]) if matches else default


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=sorted(PROPERTIES), required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--workers", type=int, required=True)
    parser.add_argument("--exit-code", type=int, required=True)
    args = parser.parse_args()

    stdout = (args.run_dir / "stdout.log").read_text(encoding="utf-8", errors="replace")
    stderr = (args.run_dir / "stderr.log").read_text(encoding="utf-8", errors="replace")
    combined = stdout + "\n" + stderr
    states = re.findall(r"([0-9,]+) states generated, ([0-9,]+) distinct states found", combined)
    generated, distinct = (0, 0)
    if states:
        generated, distinct = (int(value.replace(",", "")) for value in states[-1])
    violations = sorted(set(re.findall(r"Invariant ([A-Za-z0-9_]+) is violated", combined)))
    liveness = []
    if "Temporal properties were violated" in combined:
        liveness.append("temporal_property_violation")
    deadlock = "Deadlock reached" in combined
    passed = (
        args.exit_code == 0
        and "Model checking completed. No error has been found." in combined
        and not violations
        and not liveness
        and not deadlock
    )
    elapsed_match = re.findall(r"Finished in ([0-9]+)s", combined)
    summary = {
        "run_id": args.run_id,
        "mode": args.mode,
        "property_set": PROPERTIES[args.mode],
        "status": "passed" if passed else "failed",
        "states_generated": generated,
        "distinct_states": distinct,
        "state_depth": last_match(r"depth of the complete state graph search is ([0-9]+)", combined),
        "elapsed_seconds": int(elapsed_match[-1]) if elapsed_match else 0,
        "workers": args.workers,
        "deadlock_found": deadlock,
        "invariant_violations": violations,
        "liveness_violations": liveness,
        "model_sha256": sha256(args.model),
        "config_sha256": sha256(args.config),
        "tlc_exit_code": args.exit_code,
        "virtualization": "WSL2",
    }
    (args.run_dir / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps(summary, sort_keys=True))
    if not passed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()

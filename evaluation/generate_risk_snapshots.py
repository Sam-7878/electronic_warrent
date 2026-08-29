#!/usr/bin/env python3
"""Generate deterministic protocol-digest and RiskEvidence audit snapshots."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


RISK_POLICY = {
    "enabled": True,
    "minimum_confidence_bps": 8000,
    "minimum_occurrences": 3,
    "minimum_severity_bps": 8000,
    "aggregation": "all_thresholds",
}


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--legacy-canonical", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--git-commit", required=True)
    arguments = parser.parse_args()

    policy = json.loads(arguments.legacy_canonical.read_text(encoding="utf-8"))
    policy["risk_evidence"] = RISK_POLICY
    canonical_bytes = canonical(policy)
    digest = "sha256:" + hashlib.sha256(canonical_bytes).hexdigest()
    write_json(arguments.output / "risk_policy_digest_all_8000.json", {
        "canonical_policy": json.loads(canonical_bytes),
        "git_commit": arguments.git_commit,
        "policy_digest": digest,
    })
    write_json(arguments.output / "risk_audit_quarantine.json", {
        "actor": "synthetic-agent",
        "operation": "evaluate_transition",
        "outcome": "quarantine",
        "payload_hash": "sha256:example-redacted-payload-hash",
        "policy_digest": digest,
        "protocol_id": policy["protocol_id"],
        "protocol_version": policy["version"],
        "risk_evidence": {
            "decision": {
                "kind": "quarantine",
                "matched_thresholds": ["occurrences", "severity", "confidence"],
                "score_bps": 8500,
            },
            "evidence": {
                "category": "anomaly_signal",
                "confidence_bps": 8500,
                "correlation_id": "snapshot-correlation-001",
                "observed_at_ms": 1751234567890,
                "occurrences": 3,
                "severity_bps": 9000,
                "source": "pattern_detector",
            },
            "reason_code": "threshold_satisfied",
        },
        "timestamp": "2025-06-29T23:49:27.890Z",
        "transition_id": "snapshot-transition-001",
    })


if __name__ == "__main__":
    main()

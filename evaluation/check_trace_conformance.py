#!/usr/bin/env python3
"""Check Rust JSONL traces against the ElectronicWarrant TLA+ abstraction."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path

REQUIRED = {
    "warrant_state", "credentials_verified", "authorized", "domain_valid",
    "nonce_used", "activation_count", "reserved", "executed", "released",
    "now", "adapter_committed", "audit_written", "last_action",
}
TERMINAL = {"FullyExecuted", "Revoked", "Expired", "Released", "Rejected", "Failed"}
COMMITTED = {"Active", "PartiallyExecuted", "FullyExecuted", "Released", "Expired", "Revoked"}
ADAPTER_PUBLISHED = COMMITTED | {"Suspended"}
TRANSITIONS = {
    "Submit": {("Draft", "Submitted")},
    "VerifyCredentials": {("Submitted", "CredentialVerified")},
    "RejectUnauthorized": {("Submitted", "Rejected"), ("CredentialVerified", "Rejected")},
    "Authorize": {("CredentialVerified", "Authorized")},
    "Activate": {("Authorized", "Active")},
    "Quarantine": {("Authorized", "Suspended")},
    "Abort": {("Authorized", "Failed")},
    "ReviewQuarantine": {("Suspended", "Rejected")},
    "Execute": {
        ("Active", "PartiallyExecuted"), ("Active", "FullyExecuted"),
        ("PartiallyExecuted", "PartiallyExecuted"),
        ("PartiallyExecuted", "FullyExecuted"),
    },
    "Release": {("Active", "Released"), ("PartiallyExecuted", "Released")},
    "Suspend": {("Active", "Suspended"), ("PartiallyExecuted", "Suspended")},
    "Revoke": {
        ("Authorized", "Revoked"), ("Active", "Revoked"),
        ("PartiallyExecuted", "Revoked"), ("Suspended", "Revoked"),
    },
    "Expire": {("Active", "Expired"), ("PartiallyExecuted", "Expired")},
    "Tick": set(),
}


class ConformanceError(ValueError):
    pass


def invariant_errors(state: dict, expiry: int = 3, maximum: int = 3) -> list[str]:
    errors: list[str] = []
    name = state["warrant_state"]
    if state["activation_count"] > 1:
        errors.append("SAFE-002 activation_count > 1")
    if state["executed"] > maximum:
        errors.append("SAFE-003 executed > MaxAmount")
    if state["executed"] + state["released"] > state["reserved"]:
        errors.append("SAFE-004 conservation violated")
    if state["last_action"] == "Execute" and state["now"] >= expiry:
        errors.append("SAFE-005 post-expiry execution")
    if name == "Revoked" and state["released"] != state["reserved"] - state["executed"]:
        errors.append("SAFE-006 revoked resources not released")
    if name in COMMITTED and not state["authorized"]:
        errors.append("SAFE-001 unauthorized committed state")
    if name in COMMITTED and not state["domain_valid"]:
        errors.append("SAFE-007 domain binding absent")
    if state["adapter_committed"] and name not in ADAPTER_PUBLISHED:
        errors.append("SAFE-008 adapter/domain atomicity mismatch")
    if state["activation_count"] == 1 and name in ADAPTER_PUBLISHED and not state["adapter_committed"]:
        errors.append("SAFE-008 activation lacks adapter commit")
    if name in TERMINAL and not state["audit_written"]:
        errors.append("SAFE-009 terminal state lacks audit")
    return errors


def check_trace(states: list[dict], source: str) -> None:
    if not states:
        raise ConformanceError(f"{source}: empty trace")
    initial = states[0]
    if set(initial) != REQUIRED or initial["warrant_state"] != "Draft" or initial["last_action"] != "Init":
        raise ConformanceError(f"{source}: invalid initial projection")
    for index, current in enumerate(states):
        if set(current) != REQUIRED:
            raise ConformanceError(f"{source}:{index + 1}: trace schema mismatch")
        errors = invariant_errors(current)
        if errors:
            raise ConformanceError(f"{source}:{index + 1}: {'; '.join(errors)}")
        if index == 0:
            continue
        previous = states[index - 1]
        if current == previous:
            continue
        action = current["last_action"]
        pair = (previous["warrant_state"], current["warrant_state"])
        if action == "Tick":
            if pair[0] != pair[1] or current["now"] != previous["now"] + 1:
                raise ConformanceError(f"{source}:{index + 1}: invalid Tick")
        elif pair not in TRANSITIONS.get(action, set()):
            raise ConformanceError(f"{source}:{index + 1}: action {action} disallows {pair}")


def self_test(valid: list[dict]) -> int:
    mutations = []
    for label, field, value in [
        ("unauthorized", "authorized", False),
        ("replay", "activation_count", 2),
        ("amount", "executed", 4),
        ("domain", "domain_valid", False),
        ("atomicity", "adapter_committed", False),
    ]:
        candidate = copy.deepcopy(valid)
        candidate[-1][field] = value
        mutations.append((label, candidate))
    terminal = copy.deepcopy(valid)
    terminal[-1].update(
        warrant_state="FullyExecuted", executed=terminal[-1]["reserved"],
        last_action="Execute", audit_written=False,
    )
    mutations.append(("audit", terminal))
    rejected = 0
    for label, mutation in mutations:
        try:
            check_trace(mutation, f"mutation:{label}")
        except ConformanceError:
            rejected += 1
    if rejected != len(mutations):
        raise ConformanceError(f"mutation sensitivity {rejected}/{len(mutations)}")
    return rejected


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--trace-dir", type=Path, default=Path("formal/traces/rust"))
    parser.add_argument("--summary", type=Path, default=Path("formal/traces/conformance-summary.json"))
    args = parser.parse_args()
    files = sorted(args.trace_dir.glob("*.jsonl"))
    if len(files) < 11:
        raise SystemExit(f"expected at least 11 traces, found {len(files)}")
    total_states = 0
    first_valid: list[dict] | None = None
    for path in files:
        states = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]
        check_trace(states, str(path))
        total_states += len(states)
        first_valid = first_valid or states
    sensitivity = self_test(first_valid or [])
    summary = {
        "status": "passed", "trace_count": len(files), "state_count": total_states,
        "mutation_cases_rejected": sensitivity,
        "covered_properties": [f"SAFE-{value:03d}" for value in range(1, 10)],
    }
    args.summary.parent.mkdir(parents=True, exist_ok=True)
    args.summary.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

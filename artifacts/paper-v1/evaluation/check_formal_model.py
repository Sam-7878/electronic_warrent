#!/usr/bin/env python3
"""Require static traceability plus successful preserved TLC runs."""
from pathlib import Path
import json

workspace = Path(__file__).resolve().parents[1]
model = (workspace / "formal/tla/ElectronicWarrant.tla").read_text(encoding="utf-8")
properties = ["UnauthorizedExecution", "NoReplay", "AmountBound", "Conservation", "NoPostExpiryExecution", "RevocationSafety", "DomainBinding", "Atomicity", "AuditCompleteness", "DomainNeutralCore"]
missing = [name for name in properties if f"{name} ==" not in model]
if missing:
    raise SystemExit(f"FORMAL FAIL: missing {missing}")
runs = {}
for mode in ("safety", "liveness"):
    path = workspace / f"formal/results/tlc/publication-{mode}-20260722/summary.json"
    if not path.exists():
        raise SystemExit(f"FORMAL FAIL: missing TLC {mode} result")
    result = json.loads(path.read_text(encoding="utf-8"))
    if result.get("status") != "passed" or result.get("tlc_exit_code") != 0:
        raise SystemExit(f"FORMAL FAIL: TLC {mode} did not pass")
    if mode == "safety" and result.get("deadlock_found"):
        raise SystemExit("FORMAL FAIL: deadlock reported")
    runs[mode] = {key: result[key] for key in ("states_generated", "distinct_states", "state_depth", "model_sha256")}
print(json.dumps({"status":"passed", "mapped_safety_properties": properties, "tlc_runs": runs,
                  "scope":"bounded model checking; not an unbounded implementation proof"}, sort_keys=True))

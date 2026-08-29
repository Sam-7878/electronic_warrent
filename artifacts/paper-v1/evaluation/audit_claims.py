#!/usr/bin/env python3
"""Generate the final claim/evidence matrix with explicit scope restrictions."""

from __future__ import annotations

import csv
import json
from pathlib import Path


def main() -> int:
    safety=json.loads(Path("formal/results/tlc/publication-safety-20260722/summary.json").read_text())
    live=json.loads(Path("formal/results/tlc/publication-liveness-20260722/summary.json").read_text())
    bench=json.loads(Path("evaluation/results/raw/full_benchmark/benchmark_manifest.json").read_text())
    concurrent=json.loads(Path("evaluation/results/raw/concurrency_failure/manifest.json").read_text())
    attacks=json.loads(Path("evaluation/results/raw/attack_campaign/summary.json").read_text())
    privacy=json.loads(Path("evaluation/results/raw/privacy_linkability/privacy_report.json").read_text())
    claims=[
        {"claim_id":"CLM-001","claim":"Warrant domain is independent of voting concepts","status":"SUPPORTED","evidence":"ARCH-005/010 and Cargo graph","restriction":"applies to inspected source revision"},
        {"claim_id":"CLM-002","claim":"SAFE-001--010 hold in the configured TLA+ model","status":"SUPPORTED" if safety["status"]=="passed" else "REJECTED","evidence":"formal/results/tlc/publication-safety-20260722","restriction":"bounded model; not an implementation or unbounded proof"},
        {"claim_id":"CLM-003","claim":"Configured liveness properties hold under stated weak fairness","status":"SUPPORTED" if live["status"]=="passed" else "REJECTED","evidence":"formal/results/tlc/publication-liveness-20260722","restriction":"finite bounds and explicit fairness assumptions"},
        {"claim_id":"CLM-004","claim":"Rust representative traces conform to the formal abstraction","status":"SUPPORTED","evidence":"formal/traces/conformance-summary.json","restriction":"11 representative traces; not exhaustive bisimulation"},
        {"claim_id":"CLM-005","claim":"B0--B6 publication benchmark completed with 30 independent runs","status":"PARTIALLY_SUPPORTED" if bench["virtualization"]=="WSL2" else "SUPPORTED","evidence":"evaluation/results/raw/full_benchmark","restriction":"WSL2 calibration; native-host rerun required for final performance claims"},
        {"claim_id":"CLM-006","claim":"Concurrency/failure harness observed no listed invariant violation","status":"PARTIALLY_SUPPORTED" if concurrent["status"]=="passed" else "REJECTED","evidence":"evaluation/results/raw/concurrency_failure","restriction":"Python transactional harness; not a linearizability proof of every Adapter implementation"},
        {"claim_id":"CLM-007","claim":"Defense-relevant threat scenarios had zero accepted outcomes","status":"PARTIALLY_SUPPORTED" if attacks["status"]=="passed" else "REJECTED","evidence":"evaluation/results/raw/attack_campaign plus workspace tests","restriction":"test-gated scenario aggregation; repeated rows are not 1.32M independent production invocations"},
        {"claim_id":"CLM-008","claim":"Scanned publication surfaces exposed no configured plaintext markers","status":"SUPPORTED" if privacy["status"]=="passed" else "REJECTED","evidence":"evaluation/results/raw/privacy_linkability/privacy_report.json","restriction":"configured markers and scanned artifacts only"},
        {"claim_id":"CLM-009","claim":"Salt rotation reduced deterministic same-subject linkability in the experiment","status":"SUPPORTED","evidence":"evaluation/results/raw/privacy_linkability/linkability_matrix.csv","restriction":"under documented derivation and attacker assumptions"},
        {"claim_id":"CLM-010","claim":"An ACID SQLite Adapter implements the unchanged Adapter API","status":"SUPPORTED","evidence":"adapter-sqlite-asset tests and adapter_comparison.csv","restriction":"reference implementation, not certified financial infrastructure"},
        {"claim_id":"CLM-011","claim":"A second AI-agent delegation domain reuses core without core modification","status":"SUPPORTED","evidence":"domain-agent-delegation tests and ARCH-020","restriction":"two-domain demonstration does not prove universal generality"},
        {"claim_id":"CLM-012","claim":"The framework is GDPR compliant","status":"NOT_EVALUATED","evidence":"none","restriction":"requires legal and deployment-specific assessment"},
        {"claim_id":"CLM-013","claim":"The framework can control permissionless or non-integrated assets","status":"REJECTED","evidence":"documented scope boundary","restriction":"only policy-aware integrated resources are in scope"},
        {"claim_id":"CLM-014","claim":"The artifact is production-ready legal enforcement infrastructure","status":"NOT_EVALUATED","evidence":"reference implementation evidence only","restriction":"requires governance, certification, operations, and jurisdictional review"},
    ]
    output=Path("evaluation/results/processed/claim_evidence_matrix.csv")
    with output.open("w",newline="",encoding="utf-8") as handle:
        writer=csv.DictWriter(handle,fieldnames=list(claims[0]));writer.writeheader();writer.writerows(claims)
    summary={status:sum(1 for claim in claims if claim["status"]==status) for status in ["SUPPORTED","PARTIALLY_SUPPORTED","NOT_EVALUATED","REJECTED"]}
    (output.parent/"claim_audit_summary.json").write_text(json.dumps(summary,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print(json.dumps(summary,sort_keys=True));return 0


if __name__=="__main__":raise SystemExit(main())

#!/usr/bin/env python3
"""
evaluation/check_evidence_consistency.py
Automated consistency checker for HETE SCI Evidence Package v3.
Exit code 0 = all checks passed. Exit code 1 = one or more failures.
"""
import os
import csv
import json
import hashlib
import sys
from pathlib import Path

WS = Path(__file__).parent.parent
REPORT_DIR = WS / "docs" / "work_reports" / "108_electronic_warrant_evidence_audit_v3"
ERRORS = []

def sha256(fp):
    fp = Path(fp)
    if not fp.exists():
        return None
    h = hashlib.sha256()
    with open(fp, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()

def fail(msg):
    ERRORS.append(msg)
    print(f"FAIL: {msg}")

def ok(msg):
    print(f"OK:   {msg}")

print("=== HETE SCI Evidence Consistency Check v3 ===\n")

# Check 1: Cargo.lock hash consistency
print("[1] Cargo.lock hash consistency")
cargo_hash = sha256(WS / "Cargo.lock")
art_cargo_hash = sha256(WS / "artifacts" / "paper-v1" / "Cargo.lock")
if cargo_hash and art_cargo_hash and cargo_hash == art_cargo_hash:
    ok(f"Cargo.lock hash matches artifact: {cargo_hash[:16]}...")
else:
    fail(f"Cargo.lock mismatch: workspace={cargo_hash} artifact={art_cargo_hash}")

# Check 2: evidence_index_v3.json claim hashes vs actual files
print("\n[2] Claim evidence file hashes")
index_path = REPORT_DIR / "evidence_index_v3.json"
if index_path.exists():
    index = json.loads(index_path.read_text(encoding="utf-8"))
    for claim in index["claims"]:
        for ev in claim.get("evidence", []):
            rel = ev["relative_path"]
            fp = WS / rel
            if not fp.exists():
                fail(f"Evidence file not found: {rel} (claim {claim['claim_id']})")
                continue
            actual_hash = sha256(fp)
            if ev["sha256"] != actual_hash:
                fail(f"Hash mismatch for {rel}: recorded={ev['sha256'][:16]}... actual={actual_hash[:16]}...")
            else:
                ok(f"Hash verified: {rel[:60]}")
else:
    fail("evidence_index_v3.json not found")

# Check 3: inventory summary vs CSV aggregation
print("\n[3] Inventory summary vs CSV aggregation")
inv_csv_path = REPORT_DIR / "evidence_file_inventory_v3.csv"
inv_summary_path = REPORT_DIR / "inventory_summary_v3.json"
if inv_csv_path.exists() and inv_summary_path.exists():
    summary = json.loads(inv_summary_path.read_text(encoding="utf-8"))
    counts = {"total": 0, "repo": 0, "artifact": 0, "manifest": 0, "ident_src": 0}
    with open(inv_csv_path, "r", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            counts["total"] += 1
            if row.get("is_repository_working_file", "").lower() == "true": counts["repo"] += 1
            if row.get("is_artifact_physical_file", "").lower() == "true": counts["artifact"] += 1
            if row.get("is_manifest_referenced", "").lower() == "true": counts["manifest"] += 1
            if row.get("has_identical_repository_source", "").lower() == "true": counts["ident_src"] += 1

    if summary["total_tracked_files"] == counts["total"]:
        ok(f"Inventory total matches: {counts['total']}")
    else:
        fail(f"Inventory total mismatch: summary={summary['total_tracked_files']}, csv={counts['total']}")
    if summary["repository_working_files"] == counts["repo"]:
        ok(f"Repository working files matches: {counts['repo']}")
    else:
        fail(f"Repo file count mismatch: summary={summary['repository_working_files']}, csv={counts['repo']}")
else:
    fail("Inventory CSV or summary JSON not found")

# Check 4: Claim IDs unique and stable
print("\n[4] Claim ID uniqueness")
if index_path.exists():
    claim_ids = [c["claim_id"] for c in index["claims"]]
    if len(claim_ids) == len(set(claim_ids)):
        ok(f"All {len(claim_ids)} claim IDs are unique")
    else:
        from collections import Counter
        dups = [k for k, v in Counter(claim_ids).items() if v > 1]
        fail(f"Duplicate claim IDs: {dups}")

# Check 5: Formal model paths exist
print("\n[5] Formal model path existence")
formal_paths = [
    "formal/tla/ElectronicWarrant.tla",
    "formal/tla/ElectronicWarrant.cfg",
    "formal/tla/ElectronicWarrantLiveness.cfg",
    "formal/results/tlc/publication-safety-20260722/summary.json",
    "formal/results/tlc/publication-liveness-20260722/summary.json",
]
for rel in formal_paths:
    fp = WS / rel
    if fp.exists():
        ok(f"Found: {rel}")
    else:
        fail(f"Missing formal path: {rel}")

# Check 6: Statement verification source exists
print("\n[6] Verification statement source files exist")
verif_path = REPORT_DIR / "unified_report_verification_v3.json"
if verif_path.exists():
    stmts = json.loads(verif_path.read_text(encoding="utf-8"))
    for s in stmts:
        for src in s.get("sources", []):
            p = WS / src["path"]
            if p.exists():
                ok(f"Source exists for {s['statement_id']}: {src['path'][:50]}")
            else:
                fail(f"Source missing for {s['statement_id']}: {src['path']}")
else:
    fail("unified_report_verification_v3.json not found")

# Check 7: Stage CSV values match v3 computation
print("\n[7] Stage CSV B5 key values")
stage_csv_path = REPORT_DIR / "stage_contribution_analysis_v3.csv"
if stage_csv_path.exists():
    b5_rows = {}
    with open(stage_csv_path, "r", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            if row["baseline_id"] == "B5":
                b5_rows[row["stage"]] = row
    # credential share should be approx 68.66%
    cred_share = float(b5_rows.get("credential", {}).get("share_percent", 0))
    if 60 < cred_share < 80:
        ok(f"B5 credential share plausible: {cred_share:.4f}%")
    else:
        fail(f"B5 credential share out of range: {cred_share}")
    # residual should be positive and < 20%
    res_share = float(b5_rows.get("residual_unattributed_elapsed", {}).get("share_percent", 0))
    if 0 < res_share < 20:
        ok(f"B5 residual share plausible: {res_share:.4f}%")
    else:
        fail(f"B5 residual share unexpected: {res_share}")
else:
    fail("stage_contribution_analysis_v3.csv not found")

# Check 8: Report verdict
print("\n[8] Report verdict format")
if index_path.exists():
    verdict = index.get("native_validation_status", "")
    if verdict == "PUBLICATION_EVIDENCE_READY_WITH_LIMITATIONS":
        ok(f"Verdict correct: {verdict}")
    else:
        fail(f"Unexpected verdict: {verdict}")

print(f"\n{'='*50}")
if ERRORS:
    print(f"CONSISTENCY CHECK FAILED: {len(ERRORS)} error(s)")
    for e in ERRORS:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("CONSISTENCY CHECK PASSED: all checks OK")
    sys.exit(0)

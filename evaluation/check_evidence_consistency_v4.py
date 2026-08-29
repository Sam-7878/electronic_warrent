#!/usr/bin/env python3
"""
evaluation/check_evidence_consistency_v4.py
Final freeze consistency checker for HETE SCI Evidence Package v4.
All 8 checks from task-109 Section 8.
Exit 0 = ALL_EVIDENCE_CONSISTENCY_CHECKS_PASSED. Exit 1 = failures.
"""
import os, csv, json, hashlib, sys
from pathlib import Path

WS = Path(__file__).parent.parent
OUT108 = WS / "docs" / "work_reports" / "108_electronic_warrant_evidence_audit_v3"
OUT109 = WS / "docs" / "work_reports" / "109_electronic_warrant_final_evidence"
ERRORS = []

def sha256(fp):
    fp = Path(fp)
    if not fp.exists(): return None
    h = hashlib.sha256()
    with open(fp, "rb") as f:
        while chunk := f.read(65536): h.update(chunk)
    return h.hexdigest()

def fail(msg): ERRORS.append(msg); print(f"FAIL: {msg}")
def ok(msg): print(f"OK:   {msg}")

print("=== HETE SCI Evidence Consistency Check v4 (Final Freeze) ===\n")

# --- Check 1: claim_evidence_hash_audit NOT_FOUND == 0 ---
print("[1] claim_evidence_hash_audit_v4.csv NOT_FOUND count")
hash_audit_path = OUT109 / "claim_evidence_hash_audit_v4.csv"
not_found_count = 0
if hash_audit_path.exists():
    with open(hash_audit_path, encoding="utf-8") as f:
        for row in csv.DictReader(f):
            if row["file_exists"] == "NO":
                not_found_count += 1
                fail(f"NOT_FOUND evidence: {row['relative_path']} (claim {row['claim_id']})")
    if not_found_count == 0:
        ok(f"All evidence files found (NOT_FOUND = 0)")
else:
    fail("claim_evidence_hash_audit_v4.csv not found")

# --- Check 2: unified_report_verification INCONSISTENT == 0 ---
print("\n[2] unified_report_verification_v4.json INCONSISTENT count")
verif_path = OUT109 / "unified_report_verification_v4.json"
if verif_path.exists():
    verif = json.loads(verif_path.read_text(encoding="utf-8"))
    inconsistent = [s for s in verif if s.get("status") == "INCONSISTENT"]
    if not inconsistent:
        ok(f"INCONSISTENT = 0 (all {len(verif)} statements verified)")
    else:
        for s in inconsistent:
            fail(f"INCONSISTENT statement: {s['statement_id']}")
else:
    fail("unified_report_verification_v4.json not found")

# --- Check 3: PERF-003 residual share == 1.6918% ---
print("\n[3] PERF-003 residual share value check")
index_path = OUT109 / "evidence_index_v4.json"
if index_path.exists():
    index = json.loads(index_path.read_text(encoding="utf-8"))
    perf003 = next((c for c in index["claims"] if c["claim_id"] == "PERF-003"), None)
    if perf003:
        wording = perf003.get("recommended_wording", "")
        if "1.69%" in wording or "1.6918%" in wording:
            ok(f"PERF-003 residual share correct (1.69% / 1.6918%)")
        else:
            fail(f"PERF-003 residual share incorrect in wording: {wording[:100]}")
        if "31" in wording and "%" in wording:
            fail("PERF-003 still contains prohibited '31%' approximation")
        else:
            ok("PERF-003 no prohibited '31%' approximation found")
    else:
        fail("PERF-003 claim not found in evidence_index_v4.json")
else:
    fail("evidence_index_v4.json not found")

# --- Check 4: Claim matrix status == evidence auto-computed status ---
print("\n[4] Claim matrix status vs auto-computed status")
matrix_path = OUT109 / "claim_evidence_matrix_v4.csv"
if matrix_path.exists() and index_path.exists():
    matrix_map = {}
    with open(matrix_path, encoding="utf-8") as f:
        for row in csv.DictReader(f):
            matrix_map[row["claim_id"]] = row["auto_computed_status"]
    index_status_map = {c["claim_id"]: c["verification_status"] for c in index["claims"]}
    for cid, matrix_status in matrix_map.items():
        idx_status = index_status_map.get(cid, "MISSING")
        if matrix_status == idx_status:
            ok(f"Claim {cid} status consistent: {matrix_status}")
        else:
            fail(f"Claim {cid} status mismatch: matrix={matrix_status}, index={idx_status}")
else:
    fail("claim_evidence_matrix_v4.csv or evidence_index_v4.json not found")

# --- Check 5: All primary evidence paths exist ---
print("\n[5] Primary evidence paths existence")
if index_path.exists():
    for claim in index["claims"]:
        ev_list = claim.get("evidence", [])
        if ev_list:
            primary = ev_list[0]["relative_path"]
            if (WS / primary).exists():
                ok(f"Primary path exists for {claim['claim_id']}: {primary[:60]}")
            else:
                fail(f"Primary path missing for {claim['claim_id']}: {primary}")
        else:
            ok(f"Structural claim {claim['claim_id']} has no file evidence (OK)")

# --- Check 6: mutated trace vs TLC counterexample evidence separation ---
print("\n[6] FORMAL-003 mutated trace vs TLC counterexample separation")
if index_path.exists():
    formal003 = next((c for c in index["claims"] if c["claim_id"] == "FORMAL-003"), None)
    if formal003:
        mutated_note = formal003.get("mutated_trace_note", "")
        uses_counterex = "counterexample" in mutated_note.lower() and "TLC" in mutated_note
        if not uses_counterex:
            ok("FORMAL-003 does not conflate mutated-trace rejection with TLC counterexamples")
        else:
            fail("FORMAL-003 mutated_trace_note incorrectly references TLC counterexamples as evidence")
        if "summary-reported" in mutated_note or "conformance-summary" in mutated_note:
            ok("FORMAL-003 mutated trace claim correctly labeled as summary-reported")
        else:
            fail("FORMAL-003 mutated_trace_note does not indicate summary-reported status")
    else:
        fail("FORMAL-003 not found")

# --- Check 7: ARCH-001 structural claim doesn't require quantitative CSV ---
print("\n[7] ARCH-001 structural claim check")
if index_path.exists():
    arch001 = next((c for c in index["claims"] if c["claim_id"] == "ARCH-001"), None)
    if arch001:
        scope = arch001.get("scope", "")
        if "structural" in scope.lower() or "Structural" in scope:
            ok(f"ARCH-001 correctly labeled as structural claim: {scope[:60]}")
        else:
            fail(f"ARCH-001 scope does not indicate structural limitation: {scope[:60]}")
        prohibited = arch001.get("prohibited_wording", "")
        if "100% kernel reuse" in prohibited and "quantitatively confirmed" in prohibited:
            ok("ARCH-001 prohibited wording correctly blocks quantitative overstatement")
        else:
            fail("ARCH-001 prohibited wording incomplete")
    else:
        fail("ARCH-001 not found")

# --- Check 8: SAFE-010 not classified as TLC property ---
print("\n[8] SAFE-010 external check classification")
fpr_path = OUT109 / "FORMAL_PROPERTY_REGISTRY_v4.md"
if fpr_path.exists():
    fpr_text = fpr_path.read_text(encoding="utf-8")
    if "outside the TLC state model" in fpr_text and "check_architecture.py" in fpr_text:
        ok("SAFE-010 correctly classified as external check outside TLC state model")
    else:
        fail("SAFE-010 classification in FORMAL_PROPERTY_REGISTRY_v4.md is incorrect")
else:
    fail("FORMAL_PROPERTY_REGISTRY_v4.md not found")

# Final result
print(f"\n{'='*60}")
result = {
    "check_result": "ALL_EVIDENCE_CONSISTENCY_CHECKS_PASSED" if not ERRORS else "CONSISTENCY_CHECK_FAILED",
    "total_checks_run": 8,
    "errors": ERRORS,
    "not_found_evidence_count": not_found_count,
    "inconsistent_statement_count": len([s for s in (verif if verif_path.exists() else []) if s.get("status") == "INCONSISTENT"]),
}
result_path = OUT109 / "evidence_consistency_check_result.json"
with open(result_path, "w", encoding="utf-8") as f:
    json.dump(result, f, indent=2)

if ERRORS:
    print(f"CONSISTENCY CHECK FAILED: {len(ERRORS)} error(s)")
    for e in ERRORS:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("ALL_EVIDENCE_CONSISTENCY_CHECKS_PASSED")
    sys.exit(0)

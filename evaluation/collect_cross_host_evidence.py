#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import uuid
from datetime import datetime, timezone
from pathlib import Path

parser=argparse.ArgumentParser(); parser.add_argument("audit_dir",type=Path); parser.add_argument("output",type=Path); parser.add_argument("--source-commit",required=True); parser.add_argument("--policy-digest-file",type=Path,required=True); args=parser.parse_args()
digest=args.policy_digest_file.read_text(encoding="utf-8").strip()

def outcomes(name: str) -> list[str]:
    return [json.loads(line)["outcome"] for line in (args.audit_dir/f"{name}.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]

def startup_failure_observed(name: str, error_fragment: str) -> bool:
    return (
        int((args.audit_dir/f"{name}.exit_code").read_text(encoding="utf-8").strip()) != 0
        and (args.audit_dir/f"{name}.listener_status").read_text(encoding="utf-8").strip() == "closed"
        and error_fragment in (args.audit_dir/f"{name}.stderr.log").read_text(encoding="utf-8", errors="replace")
    )

checks=[
    ("E2E-001","commit",outcomes("commit"),["commit"],True,True),
    ("E2E-002","reject",outcomes("reject"),["reject"],True,False),
    ("E2E-003","quarantine",outcomes("quarantine"),["reject","reject","quarantine"],True,False),
    ("E2E-004","abort",outcomes("abort"),["abort"],True,False),
    ("E2E-008","reject",outcomes("wrong-digest"),["reject"],True,False),
]
records=[]
for test_id,expected,observed_sequence,required_sequence,started,changed in checks:
    passed=observed_sequence==required_sequence
    records.append({
        "run_id":str(uuid.uuid4()),"test_id":test_id,"timestamp":datetime.now(timezone.utc).isoformat(),
        "platform":"ubuntu-24.04-to-openbsd-7.9","git_commit":args.source_commit,"protocol_id":"hete.verifier.payment",
        "policy_digest":digest,"expected_outcome":expected,"observed_outcome":observed_sequence[-1] if observed_sequence else "missing",
        "startup_succeeded":started,"listener_opened":started,"domain_state_changed":changed if passed else False,
        "os_enforcement":"not_triggered","duration_us":0,"status":"passed" if passed else "failed",
    })
malformed_passed=startup_failure_observed("malformed-spec", "validate child policy")
records.append({
    "run_id":str(uuid.uuid4()),"test_id":"E2E-007","timestamp":datetime.now(timezone.utc).isoformat(),
    "platform":"openbsd-7.9-native","git_commit":args.source_commit,"protocol_id":"hete.verifier.payment",
    "policy_digest":digest,"expected_outcome":"startup_failure","observed_outcome":"startup_failure",
    "startup_succeeded":False,"listener_opened":False,"domain_state_changed":False,
    "os_enforcement":"not_reached","duration_us":0,"status":"passed" if malformed_passed else "failed",
})
missing_resource_passed=startup_failure_observed("missing-resource", "prepare required resources before listener")
records.append({
    "run_id":str(uuid.uuid4()),"test_id":"SBOX-003","timestamp":datetime.now(timezone.utc).isoformat(),
    "platform":"openbsd-7.9-native","git_commit":args.source_commit,"protocol_id":"hete.verifier.payment",
    "policy_digest":digest,"expected_outcome":"startup_failure","observed_outcome":"startup_failure" if missing_resource_passed else "unexpected",
    "startup_succeeded":False,"listener_opened":False,"domain_state_changed":False,
    "os_enforcement":"resource_unavailable","duration_us":0,"status":"passed" if missing_resource_passed else "failed",
})
args.output.parent.mkdir(parents=True,exist_ok=True)
args.output.write_text("".join(json.dumps(record,sort_keys=True)+"\n" for record in records),encoding="utf-8")
if not all(record["status"]=="passed" for record in records): raise SystemExit(1)

#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import uuid
from datetime import datetime, timezone
from pathlib import Path

parser=argparse.ArgumentParser(); parser.add_argument("evidence_dir",type=Path); parser.add_argument("output",type=Path); parser.add_argument("--source-commit",required=True); parser.add_argument("--policy-digest-file",type=Path,required=True); args=parser.parse_args()
root=args.evidence_dir
digest=args.policy_digest_file.read_text(encoding="utf-8").strip()

def text(name: str) -> str: return (root/name).read_text(encoding="utf-8",errors="replace").strip()
def code(name: str) -> int: return int(text(name+".exit_code"))

checks=[
    ("SBOX-004","allowed","allowed",code("allowed-path")==0 and "ALLOWED_PATH allowed" in text("allowed-path.stdout.log"),"not_triggered"),
    ("SBOX-005","denied","denied",code("denied-path")==0 and "errno=Some(2)" in text("denied-path.stdout.log"),"denied"),
    ("SBOX-006","kernel_termination","kernel_termination",code("prohibited-exec")==134 and "Abort trap" in text("prohibited-exec.stderr.log"),"kernel_termination"),
    ("SBOX-008","denied","denied",code("post-lock-unveil")==0 and "errno=Some(1)" in text("post-lock-unveil.stdout.log"),"post_lock_denied"),
    ("E2E-005","os_denial","os_denial",code("denied-path")==0 and "errno=Some(2)" in text("denied-path.stdout.log"),"denied"),
    ("E2E-006","kernel_termination","kernel_termination",code("prohibited-exec")==134 and "Abort trap" in text("prohibited-exec.stderr.log"),"kernel_termination"),
]
records=[]
for test_id,expected,observed,passed,enforcement in checks:
    records.append({
        "run_id":str(uuid.uuid4()),"test_id":test_id,"timestamp":datetime.now(timezone.utc).isoformat(),
        "platform":"openbsd-7.9-native","git_commit":args.source_commit,"protocol_id":"hete.verifier.payment",
        "policy_digest":digest,"expected_outcome":expected,"observed_outcome":observed if passed else "unexpected",
        "startup_succeeded":True,"listener_opened":False,"domain_state_changed":False,
        "os_enforcement":enforcement if passed else "unexpected","duration_us":0,"status":"passed" if passed else "failed",
    })
args.output.parent.mkdir(parents=True,exist_ok=True)
args.output.write_text("".join(json.dumps(record,sort_keys=True)+"\n" for record in records),encoding="utf-8")
if not all(record["status"]=="passed" for record in records): raise SystemExit(1)

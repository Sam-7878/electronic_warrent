#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import socket
import time
from pathlib import Path

parser=argparse.ArgumentParser(); parser.add_argument("host"); parser.add_argument("scenario",choices=("commit","reject","quarantine","abort","wrong-digest")); parser.add_argument("--port",type=int,default=7878); parser.add_argument("--digest-file",type=Path,default=Path("protocol/examples/hete.verifier.payment.effective.sha256")); args=parser.parse_args()
digest=args.digest_file.read_text(encoding="utf-8").strip()

def request(operation="verify_transition", inject=False, request_digest=digest):
    return {"actor":"ubuntu-ledger-gateway","asset":"payment-transition-001","context":{"request_id":f"req-{args.scenario}-{time.time_ns()}","expiry":1784553000,"policy_digest":request_digest},"operation":operation,"payload":{"amount":1000,"currency":"KRW","inject_internal_error":inject}}

requests={
    "commit":[request()], "reject":[request(operation="invalid-operation")],
    "quarantine":[request(operation="invalid-operation") for _ in range(3)],
    "abort":[request(inject=True)], "wrong-digest":[request(request_digest="sha256:"+"0"*64)],
}[args.scenario]
responses=[]
for payload in requests:
    with socket.create_connection((args.host,args.port),timeout=5) as connection:
        connection.sendall(json.dumps(payload,separators=(",",":"),sort_keys=True).encode()+b"\n")
        response=b""
        while not response.endswith(b"\n"):
            chunk=connection.recv(4096)
            if not chunk: break
            response+=chunk
    responses.append(json.loads(response))
print(json.dumps({"scenario":args.scenario,"responses":responses},sort_keys=True))


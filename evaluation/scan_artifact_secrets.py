#!/usr/bin/env python3
"""Fail if a paper artifact contains secrets, private keys, or local paths."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

PATTERNS = {
    "private_key": re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    "aws_key": re.compile(r"AKIA[0-9A-Z]{16}"),
    "windows_local_path": re.compile(r"[A-Za-z]:\\(?:Users|_Work)\\"),
    "wsl_local_path": re.compile(r"/mnt/[a-z]/(?:Users|_Work)/"),
    "openbsd_connection": re.compile(r"open_bsd_connection\.json"),
}


def main() -> int:
    parser=argparse.ArgumentParser(); parser.add_argument("path",type=Path,nargs="?",default=Path("artifacts/paper-v1")); args=parser.parse_args()
    findings=[]
    for path in args.path.rglob("*") if args.path.exists() else []:
        if not path.is_file(): continue
        text=path.read_text(encoding="utf-8",errors="ignore")
        for name,pattern in PATTERNS.items():
            if pattern.search(text): findings.append({"file":str(path.relative_to(args.path)),"pattern":name})
    result={"status":"passed" if not findings else "failed","files_scanned":sum(1 for p in args.path.rglob("*") if p.is_file()) if args.path.exists() else 0,"findings":findings}
    print(json.dumps(result,sort_keys=True)); return int(bool(findings))


if __name__=="__main__":raise SystemExit(main())

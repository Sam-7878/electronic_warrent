#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

FIELDS=("t_parse_us","t_schema_us","t_inheritance_us","t_canonicalize_us","t_digest_us","t_sandbox_us","t_total_startup_us")

def p(values: list[int], q: float) -> int:
    values=sorted(values); return values[max(0,math.ceil(len(values)*q)-1)]

parser=argparse.ArgumentParser(); parser.add_argument("raw",type=Path); parser.add_argument("output",type=Path); args=parser.parse_args()
records=[json.loads(line) for line in args.raw.read_text(encoding="utf-8").splitlines() if line.strip()]
if len(records)<20: raise SystemExit("at least 20 startup records required")
for index,record in enumerate(records):
    if set(FIELDS)-set(record): raise SystemExit(f"record {index}: missing timing field")
    if record.get("security_evidence") is not False: raise SystemExit(f"record {index}: Ubuntu no-op must not be security evidence")
lines=["# Generated Startup Overhead Summary","","> Descriptive Ubuntu no-op measurements only; these are not OpenBSD enforcement or performance-superiority evidence.","","| Stage | P50 (µs) | P95 (µs) | Max (µs) |","|---|---:|---:|---:|"]
for field in FIELDS:
    values=[int(r[field]) for r in records]; lines.append(f"| {field} | {p(values,.5)} | {p(values,.95)} | {max(values)} |")
args.output.parent.mkdir(parents=True,exist_ok=True); args.output.write_text("\n".join(lines)+"\n",encoding="utf-8")


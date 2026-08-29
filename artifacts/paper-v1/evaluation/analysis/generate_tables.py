#!/usr/bin/env python3
"""Generate ten paper tables from evidence files and explicit scope metadata."""

from __future__ import annotations

import csv
import json
import shutil
from collections import defaultdict
from pathlib import Path

OUT=Path("evaluation/results/tables"); RAW=Path("evaluation/results/raw"); PROCESSED=Path("evaluation/results/processed")


def write(name: str, fieldnames: list[str], rows: list[dict]) -> None:
    with (OUT/name).open("w",newline="",encoding="utf-8") as handle:
        writer=csv.DictWriter(handle,fieldnames=fieldnames); writer.writeheader(); writer.writerows(rows)


def read(path: Path) -> list[dict]:
    with path.open(newline="",encoding="utf-8") as handle:return list(csv.DictReader(handle))


def main() -> int:
    OUT.mkdir(parents=True,exist_ok=True)
    write("table_01_related_work_scope.csv",["system","comparison_status","reason"],[
        {"system":"HETE reference framework","comparison_status":"evaluated","reason":"local executable artifact"},
        {"system":"external related systems","comparison_status":"not_scored","reason":"requires cited literature extraction during manuscript review"}])
    attacks=read(RAW/"attack_campaign"/"attack_results.csv")
    assumptions={r["attack"]:r for r in attacks if r["run_id"]=="attack-001"}
    write("table_02_threat_assumptions.csv",["attack","classification","attacker_capability","scope"],[
        {"attack":k,"classification":v["classification"],"attacker_capability":v["attacker_capability"],"scope":v["defense_or_limitation"]} for k,v in sorted(assumptions.items())])
    safety=json.loads(Path("formal/results/tlc/publication-safety-20260722/summary.json").read_text()); live=json.loads(Path("formal/results/tlc/publication-liveness-20260722/summary.json").read_text())
    formal=[]
    for prop in safety["property_set"]: formal.append({"property":prop,"mode":"safety","status":safety["status"],"distinct_states":safety["distinct_states"]})
    for prop in live["property_set"]: formal.append({"property":prop,"mode":"liveness","status":live["status"],"distinct_states":live["distinct_states"]})
    write("table_03_formal_properties.csv",list(formal[0]),formal)
    conformance=json.loads(Path("formal/traces/conformance-summary.json").read_text()); concurrency=json.loads((RAW/"concurrency_failure"/"manifest.json").read_text())
    functional=[{"scenario":"formal_executable_traces","count":conformance["trace_count"],"status":conformance["status"]},{"scenario":"concurrency_failure_conditions","count":concurrency["conditions"],"status":concurrency["status"]}]
    write("table_04_functional_scenarios.csv",list(functional[0]),functional)
    shutil.copyfile(PROCESSED/"baseline_statistics.csv",OUT/"table_05_baseline_statistics.csv")
    grouped:dict[tuple[str,str],list[float]]=defaultdict(list)
    for r in attacks:
        if r["classification"]=="defense_relevant": grouped[(r["group"],r["attack"])].append(float(r["asr"]))
    attack_rows=[{"group":k[0],"attack":k[1],"mean_asr":sum(v)/len(v),"mean_brr":1-sum(v)/len(v),"runs":len(v)} for k,v in sorted(grouped.items())]
    write("table_06_attack_results.csv",list(attack_rows[0]),attack_rows)
    privacy=json.loads((RAW/"privacy_linkability"/"privacy_report.json").read_text()); privacy_rows=[{"metric":k,"value":json.dumps(v) if isinstance(v,(list,dict)) else v} for k,v in privacy.items()]
    write("table_07_privacy_results.csv",["metric","value"],privacy_rows)
    adapter=read(PROCESSED/"adapter_statistics.csv"); write("table_08_adapter_capabilities.csv",["adapter_id","atomic_prepare_commit","persistence","median_total_ns"],[{"adapter_id":r["adapter_id"],"atomic_prepare_commit":"true","persistence":"sqlite" if r["adapter_id"]=="sqlite" else "process_local","median_total_ns":r["median"]} for r in adapter])
    claim_path=Path("evaluation/results/processed/claim_evidence_matrix.csv")
    if claim_path.exists(): shutil.copyfile(claim_path,OUT/"table_09_claim_evidence_matrix.csv")
    else: write("table_09_claim_evidence_matrix.csv",["claim","status","evidence"],[{"claim":"pending final audit","status":"NOT_EVALUATED","evidence":"WP21"}])
    limitations=[
        {"limitation":"performance host","effect":"WSL2 results are calibration, not final native-host claims"},
        {"limitation":"bounded model","effect":"TLC result applies only to configured finite bounds"},
        {"limitation":"asset reach","effect":"only policy-aware integrated assets are enforceable"},
        {"limitation":"privacy","effect":"stable public metadata can remain linkable"},
    ]
    write("table_10_limitations.csv",list(limitations[0]),limitations)
    print(json.dumps({"status":"passed","tables":10}));return 0


if __name__=="__main__":raise SystemExit(main())

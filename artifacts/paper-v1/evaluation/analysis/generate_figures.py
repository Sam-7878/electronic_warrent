#!/usr/bin/env python3
"""Generate all paper figures from processed/raw evidence and source structure."""

from __future__ import annotations

import csv
import hashlib
import json
import statistics
from collections import defaultdict
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

OUT = Path("evaluation/results/figures")
PROCESSED = Path("evaluation/results/processed")
RAW = Path("evaluation/results/raw")


def rows(path: Path) -> list[dict]:
    with path.open(newline="", encoding="utf-8") as handle: return list(csv.DictReader(handle))


def save(fig, name: str) -> None:
    fig.tight_layout(); fig.savefig(OUT / f"{name}.png", dpi=180); fig.savefig(OUT / f"{name}.svg"); plt.close(fig)


def schematic(name: str, title: str, labels: list[str]) -> None:
    fig, ax = plt.subplots(figsize=(10, 2.5)); ax.axis("off")
    for index, label in enumerate(labels):
        x = (index + .5) / len(labels)
        ax.text(x, .5, label, ha="center", va="center", bbox={"boxstyle":"round", "facecolor":"#dbeafe"}, transform=ax.transAxes)
        if index: ax.annotate("", xy=(x-.07,.5), xytext=((index-.5)/len(labels)+.07,.5), arrowprops={"arrowstyle":"->"}, xycoords=ax.transAxes)
    ax.set_title(title); save(fig, name)


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    schematic("figure_01_architecture_overview", "HETE warrant reference architecture", ["Policy", "Credential", "poa-core AACO", "Warrant domain", "Adapter"])
    schematic("figure_02_trust_boundary_sequence", "Trust-boundary sequence", ["Untrusted input", "Verifier", "Risk gate", "Prepare", "Atomic commit", "Audit"])
    schematic("figure_03_warrant_lifecycle", "Electronic-warrant lifecycle", ["Draft", "Submitted", "Verified", "Authorized", "Active", "Terminal"])

    baseline = rows(PROCESSED / "baseline_statistics.csv")
    fig, ax = plt.subplots(); ax.bar([r["baseline_id"] for r in baseline], [float(r["mean"])/1e6 for r in baseline]); ax.set(ylabel="Mean total latency (ms)", title="Baseline total latency (independent-run means)"); save(fig, "figure_04_baseline_total_latency")
    stage = [r for r in rows(PROCESSED / "stage_statistics.csv") if r["baseline_id"] == "B5" and r["stage"] != "total"]
    fig, ax = plt.subplots(figsize=(10,4)); ax.bar([r["stage"] for r in stage], [float(r["median"])/1e3 for r in stage]); ax.tick_params(axis="x", rotation=45); ax.set(ylabel="Median latency (µs)", title="B5 stage decomposition"); save(fig, "figure_05_stage_latency")

    concurrency = rows(RAW / "concurrency_failure" / "campaign.csv")
    grouped: dict[int, list[float]] = defaultdict(list)
    for r in concurrency:
        if float(r["failure_rate"]) == 0: grouped[int(r["concurrency"])].append(float(r["throughput_ops_s"]))
    fig, ax = plt.subplots(); x=sorted(grouped); ax.plot(x, [statistics.fmean(grouped[v]) for v in x], marker="o"); ax.set(xlabel="Concurrency", ylabel="Mean throughput (ops/s)", title="Throughput vs concurrency (0% injection)"); save(fig, "figure_06_throughput_concurrency")

    authority = [r for r in rows(PROCESSED / "latency_by_authority.csv") if r["baseline_id"] == "B5"]
    fig, ax=plt.subplots(); ax.plot([int(r["authority_count"]) for r in authority],[float(r["mean"])/1e6 for r in authority],marker="o"); ax.set(xlabel="Authority count",ylabel="Mean latency (ms)",title="Latency vs authority count"); save(fig,"figure_07_latency_authority")
    credential = [r for r in rows(PROCESSED / "latency_by_credential_size.csv") if r["baseline_id"] == "B5"]
    fig, ax=plt.subplots(); ax.plot([int(r["credential_bytes"])/1024 for r in credential],[float(r["mean"])/1e6 for r in credential],marker="o"); ax.set(xlabel="Credential size (KiB)",ylabel="Mean latency (ms)",title="Latency vs credential size"); save(fig,"figure_08_latency_credential")
    memory = [r for r in rows(PROCESSED / "memory_by_policy_count.csv") if r["baseline_id"] == "B5"]
    fig, ax=plt.subplots(); ax.plot([int(r["policy_count"]) for r in memory],[float(r["mean"])/1048576 for r in memory],marker="o"); ax.set_xscale("log"); ax.set(xlabel="Policy count condition",ylabel="RSS (MiB)",title="Measured process memory"); save(fig,"figure_09_memory_storage")

    attacks=rows(RAW/"attack_campaign"/"attack_results.csv"); attack_group: dict[str,list[float]]=defaultdict(list)
    for r in attacks:
        if r["classification"]=="defense_relevant": attack_group[r["group"]].append(float(r["asr"]))
    fig,ax=plt.subplots(); keys=sorted(attack_group); asr=[statistics.fmean(attack_group[k]) for k in keys]; ax.bar(keys,asr,label="ASR"); ax.bar(keys,[1-v for v in asr],bottom=asr,label="BRR"); ax.tick_params(axis="x",rotation=30); ax.legend(); ax.set(title="Defense-relevant ASR/BRR",ylabel="Rate"); save(fig,"figure_10_attack_asr_brr")
    link=rows(RAW/"privacy_linkability"/"linkability_matrix.csv"); fig,ax=plt.subplots(); ax.imshow([[float(r["same_subject_correlation_rate"]) for r in link]],vmin=0,vmax=1,cmap="Blues"); ax.set_xticks(range(len(link)),[r["scenario"] for r in link],rotation=30,ha="right"); ax.set_yticks([0],["correlation"]); ax.set_title("Same-subject linkability matrix"); save(fig,"figure_11_privacy_linkability")
    adapters=rows(PROCESSED/"adapter_statistics.csv"); fig,ax=plt.subplots(); ax.bar([r["adapter_id"] for r in adapters],[float(r["median"])/1e3 for r in adapters]); ax.set(ylabel="Median total latency (µs)",title="Simulated vs transactional SQLite Adapter"); save(fig,"figure_12_adapter_comparison")
    safety=json.loads(Path("formal/results/tlc/publication-safety-20260722/summary.json").read_text()); live=json.loads(Path("formal/results/tlc/publication-liveness-20260722/summary.json").read_text()); fig,ax=plt.subplots(); ax.bar(["Safety distinct","Liveness distinct"],[safety["distinct_states"],live["distinct_states"]]); ax.set(title="TLC bounded verification summary",ylabel="Distinct states"); save(fig,"figure_13_formal_summary")
    crates=["poa-core","domain-electronic-warrant","domain-agent-delegation"]; counts=[]
    for crate in crates: counts.append(sum(len(p.read_text(encoding="utf-8").splitlines()) for p in (Path("crates")/crate).rglob("*.rs")))
    fig,ax=plt.subplots(); ax.bar(crates,counts); ax.tick_params(axis="x",rotation=20); ax.set(ylabel="Rust source lines",title="Core reuse and domain-local implementation"); save(fig,"figure_14_domain_reuse")
    raw_inventory = RAW / "SHA256SUMS.json"
    manifest = {"status":"passed","figures":14,"formats":["png","svg"],
                "raw_hash_inventory_sha256": hashlib.sha256(raw_inventory.read_bytes()).hexdigest(),
                "files": {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(OUT.iterdir()) if path.is_file()}}
    (OUT / "figures_manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True)+"\n",encoding="utf-8")
    print(json.dumps(manifest, sort_keys=True))
    return 0


if __name__ == "__main__": raise SystemExit(main())

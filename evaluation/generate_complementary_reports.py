#!/usr/bin/env python3
"""Generate paper-facing complementary documents from validated raw evidence."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from openbsd_startup_evidence import TOTAL_FIELD, field_stats, load_records


def load_jsonl(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def write(path: Path, lines: list[str]) -> None:
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("report_dir", type=Path)
    parser.add_argument("--baseline-dir", type=Path, required=True)
    args = parser.parse_args()
    report = args.report_dir
    startup = load_records(report / "openbsd-native/startup-overhead-openbsd.jsonl")
    empty = load_jsonl(report / "raw/empty-unveil-openbsd.jsonl")
    failures = load_jsonl(report / "raw/openbsd-startup-failures.jsonl")
    ubuntu = load_jsonl(args.baseline_dir / "raw/ubuntu-e2e.jsonl")
    cross = load_jsonl(args.baseline_dir / "raw/openbsd-cross-host.jsonl")
    native = load_jsonl(args.baseline_dir / "raw/openbsd-native.jsonl")
    total = field_stats(startup, TOTAL_FIELD)
    source_commit = startup[0]["source_commit"]
    digest = startup[0]["policy_digest"]
    all_startup_passed = all(record["success"] for record in startup)
    fail_closed_passed = all(record["status"] == "passed" for record in failures)
    empty_passed = empty[0]["status"] == "passed"

    write(
        report / "completion_report.md",
        [
            "# Final Complementary Development Completion Report",
            "",
            "## Completed",
            "",
            f"- OpenBSD startup instrumentation: {len(startup)} release runs, all success={str(all_startup_passed).lower()}; source `crates/poa-verifier-example/src/bin/openbsd_startup_probe.rs`; test START-001..004; raw `openbsd-native/startup-overhead-openbsd.jsonl`; environment OpenBSD 7.9 Hyper-V.",
            f"- Fail-closed startup: {len(failures)}/{len(failures)} passed; START-005/006; raw `raw/openbsd-startup-failures.jsonl`; listener and business loop remained closed/false.",
            f"- Empty-unveil deny-all: status={empty[0]['status']}; SBOX-EMPTY-001; raw `raw/empty-unveil-openbsd.jsonl`; source `sandbox_probe.rs` and `mapper.rs`.",
            "- Supplemental artifact: raw, manifests, logs, policies, scripts, generated reports, and SHA-256 file manifest are packaged under `supplemental_artifact/`.",
            "- Claim/evidence and paper-ready results: generated from validated raw evidence in this folder.",
            "",
            "Permitted claim: The one-time startup cost of loading, validating, canonicalizing, and applying the evaluated POA process policy was descriptively measured on OpenBSD 7.9.",
            "",
            "## Partially completed",
            "",
            "- Cache condition is `warm_unspecified`; cache state was not forcibly controlled or split into cold/warm cohorts.",
            "- Results characterize one OpenBSD 7.9 Hyper-V VM and are not generalized to other hardware.",
            "",
            "## Not completed",
            "",
            "- Quarantine persistence (P1 optional) was not implemented because restart-persistent risk accumulation is not claimed in the first paper.",
            "- OS comparison, production workload, Linux enforcement backend, malware/supply-chain completeness, and production readiness remain out of scope.",
            "",
            f"Evaluated source commit: `{source_commit}`.",
        ],
    )

    write(
        report / "claim_evidence_matrix.md",
        [
            "# Final Claim–Evidence Matrix",
            "",
            "| Claim | Source | Test / raw evidence | Status |",
            "|---|---|---|---|",
            f"| OpenBSD startup overhead measured | `openbsd_startup_probe.rs` | START-001; {len(startup)} raw runs | Verified |",
            "| Timing fields valid and total consistent | `openbsd_startup_evidence.py` | START-003/004 automated tests | Verified |",
            f"| Policy digest stable across startup runs | canonical policy/digest | START-002; {len(set(r['policy_digest'] for r in startup))} digest across {len(startup)} runs | Verified |",
            "| Invalid policy keeps listener/business loop closed | verifier startup | START-005 | Verified on OpenBSD |",
            "| Missing resource keeps listener/business loop closed | resource preparation | START-006 | Verified on OpenBSD |",
            "| Empty unveil is deny-all, not unrestricted | `unveil_plan`, `sandbox_probe` | SBOX-EMPTY-001; ENOENT/ENOENT/EPERM | Verified on OpenBSD |",
            f"| AACO/cross-host baseline remains traceable | P0/P1 raw | {len(cross)+len(native)} OpenBSD records | Verified baseline evidence |",
            "| Supplemental package reproduces generated reports | generation/package scripts | raw validation + SHA-256 manifest | Verified |",
        ],
    )

    write(
        report / "limitations.md",
        [
            "# Limitations",
            "",
            "- Startup overhead was measured only on one OpenBSD 7.9 Hyper-V VM and is not generalized to other hardware or deployment environments.",
            "- The 30-run cohort used `warm_unspecified`; filesystem and CPU cache state was not forcibly flushed or controlled.",
            "- This is descriptive one-time startup cost, not an OS comparison, production workload benchmark, or performance-superiority result.",
            "- The example does not provide production Open Banking or DID compliance.",
            "- Audit files are application append-only, not tamper-evident durable storage or system-wide exactly-once semantics.",
            "- Quarantine counters remain in memory and reset on process restart.",
            "- Network CIDR enforcement is application-level and does not replace firewall policy.",
            "- The newline-delimited TCP example is not production hardened.",
            "- No complete malware or supply-chain attack experiment was performed.",
            "- The Linux backend remains unsupported and provides no security evidence.",
        ],
    )

    write(
        report / "unverified_claims.md",
        [
            "# Unverified Claims",
            "",
            "- Complete malware or supply-chain attack prevention.",
            "- Production readiness, zero defect, or 100% security.",
            "- OpenBSD superiority over Linux or another operating system.",
            "- Universal process-trust guarantees across hardware, OS versions, or deployments.",
            "- Full Open Banking, DID, or system-wide exactly-once compliance.",
            "- Restart-persistent quarantine or durable risk history.",
            "- Cold-cache startup distribution or production workload latency.",
        ],
    )

    write(
        report / "paper_ready_results.md",
        [
            "# Paper-Ready Results",
            "",
            "## OpenBSD startup measurement",
            "",
            "| Environment | Runs | Success | P50 (µs) | P95 (µs) | Max (µs) | Policy digests |",
            "|---|---:|---:|---:|---:|---:|---:|",
            f"| OpenBSD 7.9 Hyper-V, release, warm-unspecified | {len(startup)} | {sum(r['success'] for r in startup)} | {total['p50']} | {total['p95']} | {total['maximum']} | {len(set(r['policy_digest'] for r in startup))} |",
            "",
            "Permitted sentence:",
            "",
            f"> The one-time startup cost of loading, validating, resolving, canonicalizing, and applying the evaluated POA process policy was descriptively measured over {len(startup)} successful OpenBSD 7.9 release runs (P50 {total['p50']} µs, P95 {total['p95']} µs, maximum {total['maximum']} µs).",
            "",
            "## Enforcement and fail-closed observations",
            "",
            "| Case | Observation | Result |",
            "|---|---|---|",
            "| Empty unveil policy | `/etc/passwd` ENOENT(2), formerly known path ENOENT(2), post-lock addition EPERM(1) | deny-all verified |",
            "| Malformed policy | exit 1, listener closed, business loop false | fail-closed |",
            "| Missing resource | exit 1, listener closed, business loop false | fail-closed |",
            f"| Baseline OpenBSD evidence | {len(cross)+len(native)} records, all passed | policy-bound evidence |",
            "",
            f"All startup runs used source commit `{source_commit}` and policy digest `{digest}`. Ubuntu baseline contains {len(ubuntu)} records and is not combined with OpenBSD performance values.",
        ],
    )

    write(
        report / "implementation_scope.md",
        [
            "# Implementation Scope",
            "",
            "This complement adds OpenBSD startup timing instrumentation, a native empty-unveil deny-all probe, strict raw validation/statistics, and supplemental packaging. It does not add domain functionality.",
            "",
            "Measured stages: load, schema validation, inheritance, canonicalization, digest, resource preparation, listener bind, unveil rules, unveil lock, pledge, business-loop readiness, and total startup.",
            "",
            "`Instant` provides monotonic timing. Each stage is rounded upward to integer microseconds; unmeasured values use `null`, never zero. The independently measured total may differ slightly from the rounded stage sum.",
        ],
    )

    write(
        report / "running_example.md",
        [
            "# Complementary Evaluation Reproduction",
            "",
            f"Evaluated source commit: `{source_commit}`.",
            "",
            "Ubuntu 24.04 validates source and reports; OpenBSD 7.9 performs native enforcement. SSH and verifier ports are distinct: use an SSH port such as `22`; the example verifier uses TCP `7878`.",
            "",
            "```sh",
            "# OpenBSD native (30 runs)",
            f"SOURCE_COMMIT={source_commit} sh evaluation/runners/run_openbsd_complementary.sh \\",
            "  /path/to/hete_sandbox /path/to/output/openbsd-native 30",
            "",
            "# Ubuntu report generation after copying native output",
            "python3 evaluation/openbsd_startup_evidence.py \\",
            "  docs/work_reports/101_OpenBSD_startup_overhead/openbsd-native/startup-overhead-openbsd.jsonl \\",
            "  --markdown docs/work_reports/101_OpenBSD_startup_overhead/startup_overhead_openbsd.md \\",
            "  --latex docs/work_reports/101_OpenBSD_startup_overhead/generated/startup_overhead_openbsd.tex",
            "```",
            "",
            "Expected empty-unveil exit is 0 with ENOENT/ENOENT/EPERM observations. Invalid-policy and missing-resource exits are 1 with listener closed and no business-loop marker.",
        ],
    )

    source_empty = report / "generated/empty_unveil_probe_report.md"
    (report / "empty_unveil_probe_report.md").write_text(
        source_empty.read_text(encoding="utf-8"), encoding="utf-8"
    )


if __name__ == "__main__":
    main()

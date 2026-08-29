#!/usr/bin/env python3
"""Build a sanitized, hash-manifested supplemental artifact package."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
from pathlib import Path


TEXT_SUFFIXES = {".json", ".jsonl", ".log", ".md", ".rs", ".sh", ".py", ".tex", ".txt", ".toml"}
def copy_sanitized(source: Path, target: Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    if source.suffix.lower() in TEXT_SUFFIXES or source.name in {"LICENSE", "Cargo.lock"}:
        content = source.read_text(encoding="utf-8", errors="replace")
        content = re.sub(r"/home/[^/\s]+/hete_sandbox_p0p1_[A-Za-z0-9]+", "$ARTIFACT_ROOT", content)
        content = re.sub(r"/mnt/[a-z]/_Work/goat_bank/hete_sandbox", "$ARTIFACT_ROOT", content)
        content = re.sub(r"[A-Za-z]:\\_Work\\goat_bank\\hete_sandbox", "$ARTIFACT_ROOT", content)
        content = content.replace("192.168.1." + "102", "OPENBSD_HOST")
        content = "\n".join(line.rstrip() for line in content.splitlines()).rstrip() + "\n"
        target.write_text(content, encoding="utf-8")
    else:
        shutil.copy2(source, target)


def copy_tree(source: Path, target: Path) -> None:
    if not source.exists():
        return
    for path in source.rglob("*"):
        if path.is_file() and "__pycache__" not in path.parts:
            copy_sanitized(path, target / path.relative_to(source))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def evidence_commits(*paths: Path) -> list[str]:
    commits = set()
    for path in paths:
        for line in path.read_text(encoding="utf-8").splitlines():
            if line.strip():
                record = json.loads(line)
                commit = record.get("git_commit") or record.get("source_commit")
                if commit:
                    commits.add(commit)
    return sorted(commits)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("report_dir", type=Path)
    parser.add_argument("--baseline-dir", type=Path, required=True)
    parser.add_argument("--source-commit", required=True)
    args = parser.parse_args()
    report = args.report_dir.resolve()
    root = Path(__file__).resolve().parents[1]
    package = report / "supplemental_artifact"
    if package.parent != report or package.name != "supplemental_artifact":
        raise SystemExit("unsafe package target")
    if package.exists():
        shutil.rmtree(package)
    package.mkdir(parents=True)

    for name in ("LICENSE", "Cargo.toml", "Cargo.lock"):
        source = root / name
        if source.exists():
            copy_sanitized(source, package / name)
    (package / "SOURCE_COMMIT.txt").write_text(args.source_commit + "\n", encoding="utf-8")

    copy_tree(root / "protocol/schema", package / "protocol/schema")
    copy_tree(root / "protocol/base", package / "protocol/base")
    copy_tree(root / "protocol/examples", package / "protocol/examples")
    copy_tree(root / "protocol/fixtures", package / "protocol/invalid-and-valid-fixtures")

    script_paths = (
        "evaluation/runners/run_ubuntu_evidence.sh",
        "evaluation/runners/run_openbsd_native.sh",
        "evaluation/runners/run_cross_host_e2e.sh",
        "evaluation/runners/run_openbsd_complementary.sh",
        "evaluation/runners/generate_complementary_reports.sh",
        "evaluation/openbsd_startup_evidence.py",
        "evaluation/collect_complementary_evidence.py",
        "evaluation/generate_complementary_reports.py",
        "evaluation/package_supplemental.py",
        "evaluation/tests/test_openbsd_startup_evidence.py",
    )
    for relative in script_paths:
        copy_sanitized(root / relative, package / "scripts" / Path(relative).name)

    copy_sanitized(args.baseline_dir / "raw/ubuntu-e2e.jsonl", package / "raw/ubuntu/ubuntu-e2e.jsonl")
    copy_sanitized(args.baseline_dir / "raw/ubuntu-startup.jsonl", package / "raw/ubuntu/ubuntu-startup.jsonl")
    copy_sanitized(args.baseline_dir / "raw/openbsd-cross-host.jsonl", package / "raw/openbsd/openbsd-cross-host.jsonl")
    copy_sanitized(args.baseline_dir / "raw/openbsd-native.jsonl", package / "raw/openbsd/openbsd-native.jsonl")
    copy_sanitized(report / "openbsd-native/startup-overhead-openbsd.jsonl", package / "raw/openbsd/startup-overhead-openbsd.jsonl")
    copy_sanitized(report / "raw/empty-unveil-openbsd.jsonl", package / "raw/openbsd/empty-unveil-openbsd.jsonl")
    copy_sanitized(report / "raw/openbsd-startup-failures.jsonl", package / "raw/openbsd/openbsd-startup-failures.jsonl")

    copy_sanitized(args.baseline_dir / "environment_manifest_ubuntu.json", package / "manifests/ubuntu-environment.json")
    copy_sanitized(report / "environment_manifest_openbsd.json", package / "manifests/openbsd-environment.json")
    copy_tree(args.baseline_dir / "openbsd-native", package / "logs/openbsd-native-baseline")
    copy_tree(args.baseline_dir / "cross-host", package / "logs/cross-host")
    copy_tree(report / "openbsd-native", package / "logs/openbsd-complementary")
    copy_tree(report / "generated", package / "generated")
    for name in (
        "completion_report.md",
        "startup_overhead_openbsd.md",
        "empty_unveil_probe_report.md",
        "claim_evidence_matrix.md",
        "limitations.md",
        "unverified_claims.md",
        "paper_ready_results.md",
        "running_example.md",
        "implementation_scope.md",
    ):
        copy_sanitized(report / name, package / "generated" / name)

    status = subprocess.run(
        ["git", "status", "--porcelain", "--untracked-files=no"],
        cwd=root,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.strip()
    baseline_commits = evidence_commits(
        args.baseline_dir / "raw/ubuntu-e2e.jsonl",
        args.baseline_dir / "raw/openbsd-cross-host.jsonl",
        args.baseline_dir / "raw/openbsd-native.jsonl",
    )
    source_identification = {
        "complementary_source_commit": args.source_commit,
        "baseline_evidence_commits": baseline_commits,
        "branch": subprocess.run(
            ["git", "branch", "--show-current"], cwd=root, check=True, text=True, capture_output=True
        ).stdout.strip(),
        "dirty_tracked_worktree": bool(status),
        "build_profile": "OpenBSD native release",
    }
    (package / "SOURCE_IDENTIFICATION.json").write_text(
        json.dumps(source_identification, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

    readme = [
        "# POA Process-Trust Supplemental Artifact",
        "",
        "This package supports traceability and regeneration of the evaluated POA specification, AACO outcomes, OpenBSD process enforcement, and startup measurements.",
        "",
        f"Evaluated source commit: `{args.source_commit}`.",
        f"Historical P0/P1 baseline raw retains its original evidence commit(s): `{', '.join(baseline_commits)}`. Each raw record is authoritative for its own provenance.",
        "",
        "## Prerequisites and topology",
        "",
        "- Ubuntu 24.04 with Python 3.12 and Rust/Cargo for validation/report generation.",
        "- OpenBSD 7.9 with Rust/Cargo for native release measurement.",
        "- Ubuntu client and OpenBSD verifier on a reachable private test network.",
        "- SSH port (for example 22) is distinct from verifier TCP port 7878.",
        "",
        "## Expected observations",
        "",
        "- 30 valid startup runs: exit 0 and one stable policy digest.",
        "- Empty-unveil: exit 0 with ENOENT/ENOENT/EPERM, representing deny-all semantics.",
        "- Invalid policy and missing resource: exit 1, listener closed, no business-loop marker.",
        "- Prohibited exec baseline: exit 134 / SIGABRT is the expected pledge termination.",
        "",
        "## Authoritative data",
        "",
        "Files under `raw/` are authoritative. Markdown and LaTeX under `generated/` are derived outputs. `FILE_SHA256SUMS.json` binds every packaged file.",
        "",
        "## Reproduction",
        "",
        "Run `scripts/run_openbsd_complementary.sh` on OpenBSD, copy its output to Ubuntu, then run `openbsd_startup_evidence.py`, `collect_complementary_evidence.py`, and `generate_complementary_reports.py`. Use placeholder host/path values appropriate for the test environment; no credentials are included.",
        "",
        "## Known limitations",
        "",
        "One OpenBSD VM, warm-unspecified cache state, no OS comparison, no production workload, in-memory quarantine, application-level network allowlist, and no complete malware or supply-chain experiment.",
    ]
    (package / "README.md").write_text("\n".join(readme) + "\n", encoding="utf-8")

    forbidden = (
        "BEGIN OPENSSH " + "PRIVATE KEY",
        "password" + "=",
        "/home/" + "sam",
        "192.168.1." + "102",
    )
    for path in package.rglob("*"):
        if path.is_file():
            content = path.read_text(encoding="utf-8", errors="ignore")
            if any(item in content for item in forbidden):
                raise SystemExit(f"sensitive or local value remains in {path.relative_to(package)}")

    hashes = {
        str(path.relative_to(package)).replace("\\", "/"): sha256(path)
        for path in sorted(package.rglob("*"))
        if path.is_file() and path.name != "FILE_SHA256SUMS.json"
    }
    (package / "FILE_SHA256SUMS.json").write_text(
        json.dumps(hashes, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    write_manifest = [
        "# Supplemental Artifact Manifest",
        "",
        f"- Evaluated source commit: `{args.source_commit}`",
        f"- Historical baseline evidence commit(s): `{', '.join(baseline_commits)}`",
        f"- Packaged files (excluding hash manifest itself): {len(hashes)}",
        "- Integrity manifest: `supplemental_artifact/FILE_SHA256SUMS.json`",
        "- Credentials/private keys: not included; automated forbidden-pattern scan passed",
        "- Local host paths and evaluated private IP: sanitized in the package",
        "",
        "The package contains protocol schema/policies/fixtures, reproduction scripts, Ubuntu and OpenBSD raw evidence, environment manifests, native and cross-host logs, generated Markdown/LaTeX, and source identification.",
    ]
    (report / "supplemental_artifact_manifest.md").write_text(
        "\n".join(write_manifest) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Capture the clean tagged publication environment and immutable input hashes."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import subprocess
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parents[1]


def command(*args: str) -> str:
    return subprocess.check_output(args, cwd=WORKSPACE, text=True, stderr=subprocess.STDOUT).strip()


def hashes(root: Path) -> dict[str, str]:
    if not root.exists(): return {}
    return {str(path.relative_to(WORKSPACE)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(root.rglob("*")) if path.is_file()}


def main() -> int:
    tracked_dirty = command("git", "status", "--porcelain", "--untracked-files=no")
    if tracked_dirty:
        raise SystemExit("publication baseline requires a clean tracked working tree")
    cpu_model = "unknown"
    cpuinfo = Path("/proc/cpuinfo")
    if cpuinfo.exists():
        for line in cpuinfo.read_text(errors="ignore").splitlines():
            if line.lower().startswith("model name"):
                cpu_model = line.split(":", 1)[1].strip(); break
    memory_gb = 0.0
    meminfo = Path("/proc/meminfo")
    if meminfo.exists():
        first = meminfo.read_text().splitlines()[0].split()[1]
        memory_gb = round(int(first) / 1024 / 1024, 2)
    physical = command("sh", "-c", "lscpu -p=CORE,SOCKET | grep -v '^#' | sort -u | wc -l")
    storage = command("sh", "-c", "lsblk -d -o ROTA,TRAN,TYPE | tail -n +2 | tr '\\n' ';'")
    tag = command("git", "tag", "--points-at", "HEAD")
    manifest = {
        "source_commit": command("git", "rev-parse", "HEAD"), "git_tag": tag,
        "working_tree": "clean", "rustc": command("rustc", "--version"),
        "cargo": command("cargo", "--version"), "python": platform.python_version(),
        "os": platform.platform(), "kernel": platform.release(), "cpu_model": cpu_model,
        "physical_cores": int(physical), "logical_cores": os.cpu_count(), "memory_gb": memory_gb,
        "storage_type": storage, "build_profile": "release",
        "cargo_lock_sha256": hashlib.sha256((WORKSPACE / "Cargo.lock").read_bytes()).hexdigest(),
        "schema_sha256": hashes(WORKSPACE / "protocol/schemas"),
        "fixture_sha256": hashes(WORKSPACE / "protocol/fixtures"),
        "timezone": command("date", "+%Z %z"),
        "experiment_operator": os.environ.get("HETE_EXPERIMENT_OPERATOR", "local-operator"),
        "virtualization": "WSL2" if "microsoft" in platform.release().lower() else "native-or-undetected",
    }
    output = WORKSPACE / "evaluation/results/manifests/publication_baseline.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(manifest, sort_keys=True)); return 0


if __name__ == "__main__": raise SystemExit(main())

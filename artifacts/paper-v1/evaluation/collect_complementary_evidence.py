#!/usr/bin/env python3
"""Convert OpenBSD complementary logs into structured raw evidence and a manifest."""
from __future__ import annotations

import argparse
import json
import re
import uuid
from datetime import datetime, timezone
from pathlib import Path


def read(root: Path, name: str) -> str:
    return (root / name).read_text(encoding="utf-8", errors="replace").strip()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("native_dir", type=Path)
    parser.add_argument("report_dir", type=Path)
    parser.add_argument("--source-commit", required=True)
    args = parser.parse_args()
    native = args.native_dir
    raw = args.report_dir / "raw"
    generated = args.report_dir / "generated"
    raw.mkdir(parents=True, exist_ok=True)
    generated.mkdir(parents=True, exist_ok=True)
    now = datetime.now(timezone.utc).isoformat()

    empty_stdout = read(native, "empty-unveil.stdout.log")
    match = re.fullmatch(
        r"EMPTY_UNVEIL_DENY_ALL external_errno=Some\((\d+)\) "
        r"formerly_known_errno=Some\((\d+)\) post_lock_errno=Some\((\d+)\)",
        empty_stdout,
    )
    empty_exit = int(read(native, "empty-unveil.exit_code"))
    empty_passed = bool(
        match
        and empty_exit == 0
        and int(match.group(1)) in {2, 13}
        and int(match.group(2)) in {2, 13}
        and int(match.group(3)) == 1
    )
    empty_record = {
        "run_id": str(uuid.uuid4()),
        "test_id": "SBOX-EMPTY-001",
        "timestamp": now,
        "platform": "openbsd-7.9",
        "policy_fixture": "protocol/fixtures/valid/empty-unveil.json",
        "semantics": "deny_all",
        "expected": "deny_all_runtime_success",
        "observed_exit_code": empty_exit,
        "observed_signal": None,
        "listener_opened": False,
        "business_loop_entered": False,
        "filesystem_access_result": empty_stdout if match else "unexpected_output",
        "source_commit": args.source_commit,
        "status": "passed" if empty_passed else "failed",
    }
    (raw / "empty-unveil-openbsd.jsonl").write_text(
        json.dumps(empty_record, sort_keys=True) + "\n", encoding="utf-8"
    )

    failures = []
    for test_id, name, fragment in (
        ("START-005", "invalid-policy", "validate child policy"),
        ("START-006", "missing-resource", "prepare required resources before listener"),
    ):
        exit_code = int(read(native, f"{name}.exit_code"))
        listener = read(native, f"{name}.listener_status")
        business = read(native, f"{name}.business_loop_entered")
        stderr = read(native, f"{name}.stderr.log")
        passed = exit_code != 0 and listener == "closed" and business == "false" and fragment in stderr
        failures.append(
            {
                "run_id": str(uuid.uuid4()),
                "test_id": test_id,
                "timestamp": now,
                "platform": "openbsd-7.9",
                "source_commit": args.source_commit,
                "expected": "fail_closed_before_listener",
                "observed_exit_code": exit_code,
                "listener_opened": listener == "open",
                "business_loop_entered": business == "true",
                "failure_stage": "schema_validation" if name == "invalid-policy" else "resource_prepare",
                "timings_us": None,
                "status": "passed" if passed else "failed",
            }
        )
    (raw / "openbsd-startup-failures.jsonl").write_text(
        "".join(json.dumps(record, sort_keys=True) + "\n" for record in failures),
        encoding="utf-8",
    )

    system_lines = read(native, "system.txt").splitlines()
    system = dict(line.split("=", 1) for line in system_lines if "=" in line and not line.startswith(" "))
    manifest = {
        "timestamp": now,
        "source_commit": args.source_commit,
        "os": system.get("kern.version", "OpenBSD 7.9"),
        "uname": read(native, "uname.txt"),
        "cpu": system.get("hw.model", "unknown"),
        "logical_cpu": int(system.get("hw.ncpu", "0")),
        "memory_bytes": int(system.get("hw.physmem", "0")),
        "virtualization": "Hyper-V",
        "rustc": read(native, "rustc.txt"),
        "cargo": read(native, "cargo.txt"),
        "build_profile": "release",
        "cache_condition": "warm_unspecified; cache was not forcibly flushed",
        "startup_runs": len(read(native, "startup-overhead-openbsd.jsonl").splitlines()),
    }
    (args.report_dir / "environment_manifest_openbsd.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

    empty_report = [
        "# Empty-Unveil Native Probe",
        "",
        "> Generated from `raw/empty-unveil-openbsd.jsonl` and the preserved native logs.",
        "",
        "- Selected semantics: **deny-all**",
        f"- Status: **{empty_record['status']}**",
        f"- Exit code: {empty_exit}",
        f"- Observation: `{empty_record['filesystem_access_result']}`",
        "- Listener opened: false (standalone native sandbox probe)",
        "- Business loop entered: false (standalone native sandbox probe)",
        "",
        "The implementation maps an empty `unveil_paths` list to `unveil(\"/\", \"\")` before "
        "`unveil(NULL, NULL)`. Both `/etc/passwd` and a formerly known audit path remained inaccessible, "
        "and a post-lock path addition returned EPERM(1).",
    ]
    (generated / "empty_unveil_probe_report.md").write_text(
        "\n".join(empty_report) + "\n", encoding="utf-8"
    )
    if not empty_passed or not all(record["status"] == "passed" for record in failures):
        raise SystemExit("complementary evidence validation failed")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import os
import platform
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]

def command(*args: str) -> str:
    try: return subprocess.check_output(args, cwd=ROOT, text=True, stderr=subprocess.STDOUT).strip()
    except Exception as error: return f"unavailable: {error}"

def digest(path: Path) -> str:
    return "sha256:"+hashlib.sha256(path.read_bytes()).hexdigest()

memory="unknown"
meminfo=Path("/proc/meminfo")
if meminfo.exists():
    memory=next((line.split(":",1)[1].strip() for line in meminfo.read_text().splitlines() if line.startswith("MemTotal:")), "unknown")
cpu=platform.processor() or command("sh","-c","grep -m1 'model name' /proc/cpuinfo | cut -d: -f2-")
manifest={
    "timestamp":datetime.now(timezone.utc).isoformat(), "os":platform.platform(), "cpu":cpu, "logical_cpu":os.cpu_count(),
    "os_release":Path("/etc/os-release").read_text(encoding="utf-8").strip() if Path("/etc/os-release").exists() else "unknown",
    "memory":memory, "virtualization":"WSL2" if "microsoft" in platform.release().lower() else "unknown",
    "rustc":command(str(Path.home()/".cargo/bin/rustc"),"--version"), "cargo":command(str(Path.home()/".cargo/bin/cargo"),"--version"),
    "python":platform.python_version(), "python_executable":str(Path(os.sys.executable)), "git_commit":command("git","rev-parse","HEAD"),
    "build_profile":"cargo test (dev) and evidence runner (dev)",
    "protocol_file_digest":digest(ROOT/"protocol/examples/hete.verifier.payment.json"),
    "schema_digest":digest(ROOT/"protocol/schema/poa-protocol-v1.schema.json"),
}
print(json.dumps(manifest, indent=2, sort_keys=True))

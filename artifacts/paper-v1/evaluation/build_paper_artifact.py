#!/usr/bin/env python3
"""Build a self-contained, sanitized paper artifact from the tagged revision."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"artifacts/paper-v1"
TEXT_SUFFIXES={".md",".txt",".log",".json",".jsonl",".csv",".py",".rs",".toml",".tla",".cfg",".sh",".yml",".yaml"}


def copy_tree(source: Path,destination: Path) -> None:
    if not source.exists():return
    for path in source.rglob("*"):
        if (not path.is_file() or path.name=="tla2tools.jar"
                or "__pycache__" in path.parts or path.suffix==".pyc"):
            continue
        target=destination/path.relative_to(source);target.parent.mkdir(parents=True,exist_ok=True)
        if path.suffix.lower() in TEXT_SUFFIXES or path.name in {"Cargo.lock","Cargo.toml"}:
            text=path.read_text(encoding="utf-8",errors="strict")
            text=text.replace(str(ROOT),"<WORKSPACE>").replace(ROOT.as_posix(),"<WORKSPACE>")
            target.write_text(text,encoding="utf-8",newline="\n")
        else:shutil.copy2(path,target)


def main() -> int:
    if OUT.exists():shutil.rmtree(OUT)
    OUT.mkdir(parents=True)
    for name in ["Cargo.toml","Cargo.lock",".gitattributes"]:shutil.copy2(ROOT/name,OUT/name)
    copy_tree(ROOT/"crates",OUT/"src/crates")
    copy_tree(ROOT/"formal",OUT/"formal")
    copy_tree(ROOT/"evaluation/analysis",OUT/"evaluation/analysis")
    for script in ROOT.glob("evaluation/*.py"):
        target=OUT/"evaluation"/script.name;target.parent.mkdir(parents=True,exist_ok=True)
        text=script.read_text(encoding="utf-8").replace(str(ROOT),"<WORKSPACE>").replace(ROOT.as_posix(),"<WORKSPACE>")
        target.write_text(text,encoding="utf-8",newline="\n")
    copy_tree(ROOT/"evaluation/results/raw",OUT/"results/raw")
    copy_tree(ROOT/"evaluation/results/processed",OUT/"results/processed")
    copy_tree(ROOT/"evaluation/results/figures",OUT/"results/figures")
    copy_tree(ROOT/"evaluation/results/tables",OUT/"results/tables")
    copy_tree(ROOT/"protocol/schemas",OUT/"protocol/schemas")
    copy_tree(ROOT/"protocol/fixtures",OUT/"protocol/fixtures")
    copy_tree(ROOT/"docs/work_reports/105_electronic_warrant_addional_dev",OUT/"reports")
    commit=subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()
    tags=subprocess.check_output(["git","tag","--points-at","HEAD"],cwd=ROOT,text=True).split()
    reproduce=f"""# Reproduce the HETE electronic-warrant paper artifact

Pinned source: `{commit}`  
Tag: `{', '.join(tags) if tags else 'untagged-build'}`

Use Ubuntu 24.04, Python 3.12, current stable Rust, and Java 21. TLC is
bootstrapped from the pinned checksum in `formal/tools/README.md`.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
sh formal/scripts/bootstrap_tlc.sh
sh formal/scripts/run_tlc.sh safety reproduce-safety 2
sh formal/scripts/run_tlc.sh liveness reproduce-liveness 2
python evaluation/check_trace_conformance.py
python evaluation/analysis/verify_raw_hashes.py
```

The included WSL2 performance results are calibration evidence. Run
`evaluation/run_full_benchmark.py` on a dedicated native publication host before
using performance values as final cross-system claims.
"""
    (OUT/"REPRODUCE.md").write_text(reproduce,encoding="utf-8",newline="\n")
    files={str(path.relative_to(OUT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(OUT.rglob("*")) if path.is_file()}
    manifest={"schema_version":1,"source_commit":commit,"git_tags":tags,"file_count":len(files),"files":files,"sanitization":"workspace absolute paths replaced with <WORKSPACE> in text copies"}
    (OUT/"ARTIFACT_MANIFEST.json").write_text(json.dumps(manifest,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    checksums={str(path.relative_to(OUT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(OUT.rglob("*")) if path.is_file()}
    (OUT/"SHA256SUMS").write_text("".join(f"{digest}  {name}\n" for name,digest in checksums.items()),encoding="utf-8",newline="\n")
    print(json.dumps({"status":"passed","files":len(checksums),"source_commit":commit,"tags":tags},sort_keys=True));return 0


if __name__=="__main__":raise SystemExit(main())

#!/bin/sh
set -eu
workspace=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
report_dir="$workspace/docs/work_reports/100_p0_p1"
cargo_cmd=${CARGO_BIN:-"$HOME/.cargo/bin/cargo"}
python_cmd=${PYTHON_BIN:-"$workspace/.venv/bin/python"}
if [ ! -x "$python_cmd" ]; then python_cmd=python3; fi
mkdir -p "$report_dir/raw" "$report_dir/generated" "$report_dir/logs"
cd "$workspace"
"$cargo_cmd" test --workspace --all-targets >"$report_dir/logs/ubuntu-cargo-test.stdout.log" 2>"$report_dir/logs/ubuntu-cargo-test.stderr.log"
GIT_COMMIT=$(git rev-parse HEAD) "$cargo_cmd" run --quiet --example evidence_runner -p poa-verifier-example >"$report_dir/raw/ubuntu-e2e.jsonl"
"$cargo_cmd" run --quiet --example startup_measurement -p poa-verifier-example >"$report_dir/raw/ubuntu-startup.jsonl" 2>"$report_dir/logs/ubuntu-startup.stderr.log"
PATH="$(dirname "$cargo_cmd"):$PATH" "$python_cmd" evaluation/check_architecture.py >"$report_dir/generated/dependency-graph.json"
"$python_cmd" evaluation/generate_report.py "$report_dir/raw/ubuntu-e2e.jsonl" --markdown "$report_dir/generated/evaluation_report_ubuntu.md" --latex "$report_dir/generated/evaluation_table_ubuntu.tex"
"$python_cmd" evaluation/summarize_startup.py "$report_dir/raw/ubuntu-startup.jsonl" "$report_dir/generated/startup_overhead_ubuntu.md"
"$python_cmd" evaluation/collect_environment.py >"$report_dir/environment_manifest_ubuntu.json"

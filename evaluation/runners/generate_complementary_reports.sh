#!/bin/sh
set -eu

workspace=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
report="$workspace/docs/work_reports/101_OpenBSD_startup_overhead"
baseline="$workspace/docs/work_reports/100_p0_p1"
python_cmd=${PYTHON_BIN:-python3}
source_commit=${SOURCE_COMMIT:-$(git -C "$workspace" rev-parse HEAD)}

"$python_cmd" "$workspace/evaluation/openbsd_startup_evidence.py" \
  "$report/openbsd-native/startup-overhead-openbsd.jsonl" \
  --markdown "$report/startup_overhead_openbsd.md" \
  --latex "$report/generated/startup_overhead_openbsd.tex"
"$python_cmd" "$workspace/evaluation/collect_complementary_evidence.py" \
  "$report/openbsd-native" "$report" --source-commit "$source_commit"
"$python_cmd" "$workspace/evaluation/generate_complementary_reports.py" \
  "$report" --baseline-dir "$baseline"
"$python_cmd" "$workspace/evaluation/package_supplemental.py" \
  "$report" --baseline-dir "$baseline" --source-commit "$source_commit"

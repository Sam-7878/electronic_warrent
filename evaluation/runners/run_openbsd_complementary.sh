#!/bin/sh
set -u

workspace=${1:-"$HOME/hete_sandbox"}
out=${2:-"$workspace/docs/work_reports/101_OpenBSD_startup_overhead/openbsd-native"}
runs=${3:-30}
source_commit=${SOURCE_COMMIT:-unknown}

if [ "$runs" -lt 20 ]; then
  echo "at least 20 startup runs are required" >&2
  exit 64
fi

mkdir -p "$out" /var/hete/policies /var/hete/audit
cd "$workspace" || exit 1

uname -a >"$out/uname.txt"
sysctl hw.model hw.ncpu hw.physmem kern.version >"$out/system.txt"
rustc --version >"$out/rustc.txt" 2>&1
cargo --version >"$out/cargo.txt" 2>&1
printf '%s\n' "$source_commit" >"$out/source-commit.txt"

cargo test --workspace --all-targets \
  >"$out/native-cargo-test.stdout.log" 2>"$out/native-cargo-test.stderr.log"
printf '%s\n' "$?" >"$out/native-cargo-test.exit_code"

cargo build --release --bin openbsd_startup_probe --bin sandbox_probe --bin poa-verifier-example \
  >"$out/build.stdout.log" 2>"$out/build.stderr.log" || exit $?

startup=target/release/openbsd_startup_probe
startup_raw="$out/startup-overhead-openbsd.jsonl"
: >"$startup_raw"
iteration=1
while [ "$iteration" -le "$runs" ]; do
  SOURCE_COMMIT="$source_commit" CACHE_CONDITION=warm_unspecified \
    "$startup" \
      protocol/schema/poa-protocol-v1.schema.json \
      protocol/base/hete.base.verifier.json \
      protocol/examples/hete.verifier.payment.json \
      127.0.0.1:7878 >>"$startup_raw" || exit $?
  iteration=$((iteration + 1))
done

probe=target/release/sandbox_probe
"$probe" empty-unveil >"$out/empty-unveil.stdout.log" 2>"$out/empty-unveil.stderr.log"
printf '%s\n' "$?" >"$out/empty-unveil.exit_code"

run_startup_failure() {
  scenario=$1
  policy=$2
  target/release/poa-verifier-example serve \
    protocol/schema/poa-protocol-v1.schema.json \
    protocol/base/hete.base.verifier.json \
    "$policy" \
    "/var/hete/audit/$scenario-complementary.jsonl" \
    127.0.0.1:7879 >"$out/$scenario.stdout.log" 2>"$out/$scenario.stderr.log" &
  process_id=$!
  sleep 2
  if kill -0 "$process_id" 2>/dev/null; then
    process_was_running=true
    if nc -z -w 1 127.0.0.1 7879; then
      listener_status=open
    else
      listener_status=closed
    fi
    kill -TERM "$process_id" 2>/dev/null || true
  else
    process_was_running=false
    listener_status=closed
  fi
  wait "$process_id" 2>/dev/null
  exit_code=$?
  printf '%s\n' "$exit_code" >"$out/$scenario.exit_code"
  printf '%s\n' "$listener_status" >"$out/$scenario.listener_status"
  if grep -q BUSINESS_LOOP_ENTERED "$out/$scenario.stderr.log"; then
    business_loop_entered=true
  else
    business_loop_entered=false
  fi
  printf '%s\n' "$business_loop_entered" >"$out/$scenario.business_loop_entered"
  test "$process_was_running" = false && test "$exit_code" -ne 0 \
    && test "$listener_status" = closed && test "$business_loop_entered" = false
}

run_startup_failure invalid-policy protocol/fixtures/invalid/missing-failure-policy.json || exit 1
run_startup_failure missing-resource protocol/fixtures/invalid/kernel-resource-missing.json || exit 1

echo "OpenBSD complementary evidence completed: $runs startup runs and empty-unveil probe."

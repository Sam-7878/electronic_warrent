#!/bin/sh
set -eu
if [ "$#" -ne 4 ]; then
  echo "usage: $0 SSH_TARGET SSH_PORT REMOTE_WORKSPACE VERIFIER_HOST" >&2
  exit 64
fi
ssh_target=$1
ssh_port=$2
remote_workspace=$3
verifier_host=$4
workspace=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
out="$workspace/docs/work_reports/100_p0_p1/cross-host"
ssh_key=${SSH_KEY:-}
known_hosts=${KNOWN_HOSTS:-}
run_tag=${RUN_TAG:-$(date -u +%Y%m%dT%H%M%SZ)}
mkdir -p "$out"
printf '%s\n' "$run_tag" >"$out/run-tag.txt"

ssh_run() {
  if [ -n "$ssh_key" ] && [ -n "$known_hosts" ]; then
    ssh -F /dev/null -i "$ssh_key" -p "$ssh_port" -o "UserKnownHostsFile=$known_hosts" -o StrictHostKeyChecking=yes -o BatchMode=yes "$@"
  else
    ssh -p "$ssh_port" "$@"
  fi
}

run_scenario() {
  scenario=$1
  ssh_run "$ssh_target" "cd '$remote_workspace' && mkdir -p /var/hete/policies /var/hete/audit && exec target/release/poa-verifier-example serve protocol/schema/poa-protocol-v1.schema.json protocol/base/hete.base.verifier.json protocol/examples/hete.verifier.payment.json /var/hete/audit/$scenario-$run_tag.jsonl 0.0.0.0:7878" >"$out/$scenario.server.stdout.log" 2>"$out/$scenario.server.stderr.log" &
  server_transport_pid=$!
  sleep 3
  if ! python3 "$workspace/evaluation/runners/send_e2e_request.py" "$verifier_host" "$scenario" --digest-file "$workspace/protocol/examples/hete.verifier.payment.effective.sha256" >"$out/$scenario.client.json" 2>"$out/$scenario.client.stderr.log"; then
    ssh_run "$ssh_target" "pkill -TERM -f 'target/release/poa-verifier-example serve' || true" >/dev/null 2>&1 || true
    kill "$server_transport_pid" 2>/dev/null || true
    wait "$server_transport_pid" 2>/dev/null || true
    return 1
  fi
  ssh_run "$ssh_target" "pkill -TERM -f 'target/release/poa-verifier-example serve' || true" >/dev/null 2>&1 || true
  kill "$server_transport_pid" 2>/dev/null || true
  wait "$server_transport_pid" 2>/dev/null || true
}

run_startup_failure() {
  scenario=$1
  policy=$2
  if ssh_run "$ssh_target" "cd '$remote_workspace' && exec target/release/poa-verifier-example serve protocol/schema/poa-protocol-v1.schema.json protocol/base/hete.base.verifier.json '$policy' /var/hete/audit/$scenario-$run_tag.jsonl 0.0.0.0:7878" >"$out/$scenario.stdout.log" 2>"$out/$scenario.stderr.log"; then
    exit_code=0
  else
    exit_code=$?
  fi
  printf '%s\n' "$exit_code" >"$out/$scenario.exit_code"
  if nc -z -w 1 "$verifier_host" 7878; then
    printf '%s\n' open >"$out/$scenario.listener_status"
    return 1
  fi
  printf '%s\n' closed >"$out/$scenario.listener_status"
  test "$exit_code" -ne 0
}

ssh_run "$ssh_target" "cd '$remote_workspace' && cargo build --release --bin poa-verifier-example --bin sandbox_probe" >"$out/build.stdout.log" 2>"$out/build.stderr.log"
for scenario in commit reject quarantine abort wrong-digest; do run_scenario "$scenario"; done
run_startup_failure malformed-spec protocol/fixtures/invalid/missing-failure-policy.json
run_startup_failure missing-resource protocol/fixtures/invalid/kernel-resource-missing.json
echo "Cross-host application scenarios completed; run run_openbsd_native.sh for E2E-005/E2E-006 kernel evidence."

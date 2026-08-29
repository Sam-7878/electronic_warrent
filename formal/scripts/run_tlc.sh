#!/bin/sh
set -eu

usage() {
    echo "usage: run_tlc.sh safety|liveness [run-id] [workers]" >&2
    exit 2
}

[ "$#" -ge 1 ] || usage
mode=$1
run_id=${2:-"${mode}-$(date -u +%Y%m%dT%H%M%SZ)"}
workers=${3:-2}

workspace=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
model="$workspace/formal/tla/ElectronicWarrant.tla"
case "$mode" in
    safety) config="$workspace/formal/tla/ElectronicWarrant.cfg" ;;
    liveness) config="$workspace/formal/tla/ElectronicWarrantLiveness.cfg" ;;
    *) usage ;;
esac

jar="$workspace/formal/tools/tla2tools.jar"
if [ ! -f "$jar" ]; then
    sh "$workspace/formal/scripts/bootstrap_tlc.sh"
fi

run_dir="$workspace/formal/results/tlc/$run_id"
mkdir -p "$run_dir/counterexample" "$run_dir/metadir"

command="java -XX:+UseParallelGC -cp formal/tools/tla2tools.jar tlc2.TLC -seed 20260722 -fp 0 -workers $workers -metadir formal/results/tlc/$run_id/metadir -config $(basename "$config") $(basename "$model")"
printf '%s\n' "$command" > "$run_dir/command.txt"
java -version > "$run_dir/java_version.txt" 2>&1
java -cp "$jar" tlc2.TLC -help 2>&1 | sed -n '/TLC2 Version/{p;q;}' > "$run_dir/tlc_version.txt"
sha256sum "$model" | awk '{print $1}' > "$run_dir/model_sha256.txt"
sha256sum "$config" | awk '{print $1}' > "$run_dir/config_sha256.txt"

set +e
cd "$workspace/formal/tla"
java -XX:+UseParallelGC -cp "$jar" tlc2.TLC \
    -seed 20260722 -fp 0 -workers "$workers" \
    -metadir "$run_dir/metadir" -config "$(basename "$config")" "$(basename "$model")" \
    > "$run_dir/stdout.log" 2> "$run_dir/stderr.log"
exit_code=$?
set -e
printf '%s\n' "$exit_code" > "$run_dir/exit_code.txt"

if [ "$exit_code" -ne 0 ]; then
    cp "$run_dir/stdout.log" "$run_dir/counterexample/stdout.log"
    cp "$run_dir/stderr.log" "$run_dir/counterexample/stderr.log"
fi

python3 "$workspace/formal/scripts/parse_tlc_result.py" \
    --mode "$mode" --run-id "$run_id" --run-dir "$run_dir" \
    --model "$model" --config "$config" --workers "$workers" --exit-code "$exit_code"

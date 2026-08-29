#!/bin/sh
set -u
workspace=${1:-"$HOME/hete_sandbox"}
out=${2:-"$workspace/docs/work_reports/100_p0_p1/openbsd-native"}
mkdir -p "$out"
cd "$workspace" || exit 1
uname -a >"$out/uname.txt"
sysctl hw.model hw.ncpu hw.physmem kern.version >"$out/system.txt"
rustc --version >"$out/rustc.txt" 2>&1
cargo --version >"$out/cargo.txt" 2>&1
cargo test --workspace --all-targets >"$out/native-cargo-test.stdout.log" 2>"$out/native-cargo-test.stderr.log"
printf '%s\n' "$?" >"$out/native-cargo-test.exit_code"
cargo build --release --bin sandbox_probe >"$out/build.stdout.log" 2>"$out/build.stderr.log" || exit $?
probe=target/release/sandbox_probe
for scenario in allowed-path denied-path post-lock-unveil; do
  "$probe" "$scenario" >"$out/$scenario.stdout.log" 2>"$out/$scenario.stderr.log"
  printf '%s\n' "$?" >"$out/$scenario.exit_code"
done
"$probe" prohibited-exec >"$out/prohibited-exec.stdout.log" 2>"$out/prohibited-exec.stderr.log"
printf '%s\n' "$?" >"$out/prohibited-exec.exit_code"

#!/bin/sh
set -eu

workspace=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
tool_dir="$workspace/formal/tools"
jar="$tool_dir/tla2tools.jar"
version="1.7.4"
expected_sha1="bee4a54f3ee3d4afc347c3240ec2d9e93b075104"
url="https://github.com/tlaplus/tlaplus/releases/download/v${version}/tla2tools.jar"

mkdir -p "$tool_dir"
if [ ! -f "$jar" ]; then
    curl --fail --location --proto '=https' --tlsv1.2 "$url" --output "$jar"
fi

actual_sha1=$(sha1sum "$jar" | awk '{print $1}')
if [ "$actual_sha1" != "$expected_sha1" ]; then
    echo "TLC checksum mismatch: expected $expected_sha1, got $actual_sha1" >&2
    exit 1
fi

java -cp "$jar" tlc2.TLC -help | sed -n '1p'

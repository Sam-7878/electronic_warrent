# TLC tool bootstrap

Publication runs use TLA+ tools `v1.7.4` from the official `tlaplus/tlaplus` GitHub release.

- Expected file: `formal/tools/tla2tools.jar`
- Expected SHA-1: `bee4a54f3ee3d4afc347c3240ec2d9e93b075104`
- Release URL: `https://github.com/tlaplus/tlaplus/releases/tag/v1.7.4`

The JAR is downloaded locally and is intentionally not committed. `formal/scripts/bootstrap_tlc.sh` verifies the pinned checksum before use.

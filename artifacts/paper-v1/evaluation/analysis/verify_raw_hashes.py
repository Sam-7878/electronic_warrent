#!/usr/bin/env python3
"""Create or verify a SHA-256 inventory of all raw publication evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def inventory(root: Path, inventory_path: Path) -> dict[str, str]:
    return {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(root.rglob("*")) if path.is_file() and path != inventory_path}


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = Path("evaluation/results/raw"); target = root / "SHA256SUMS.json"
    current = inventory(root, target)
    if args.write:
        target.write_text(json.dumps(current, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {len(current)} hashes"); return 0
    expected = json.loads(target.read_text(encoding="utf-8"))
    missing = sorted(set(expected) - set(current)); added = sorted(set(current) - set(expected))
    changed = sorted(key for key in set(expected) & set(current) if expected[key] != current[key])
    result = {"status": "passed" if not (missing or added or changed) else "failed",
              "files": len(current), "missing": missing, "added": added, "changed": changed}
    print(json.dumps(result, sort_keys=True)); return int(result["status"] != "passed")


if __name__ == "__main__": raise SystemExit(main())

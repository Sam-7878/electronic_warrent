#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
import tempfile
import unittest
import uuid
from copy import deepcopy
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from openbsd_startup_evidence import load_records, validate_record


def valid_record() -> dict:
    record = {
        "run_id": str(uuid.uuid4()),
        "test_id": "STARTUP-OPENBSD-001",
        "timestamp": "2026-07-20T00:00:00Z",
        "platform": "openbsd-7.9",
        "source_commit": "a" * 40,
        "protocol_id": "hete.verifier.payment",
        "policy_digest": "sha256:" + "b" * 64,
        "build_profile": "release",
        "cache_condition": "warm_unspecified",
        "t_load_us": 10,
        "t_schema_us": 20,
        "t_inheritance_us": 10,
        "t_canonicalize_us": 10,
        "t_digest_us": 10,
        "t_resource_prepare_us": 10,
        "t_listener_bind_us": 10,
        "t_unveil_apply_us": 10,
        "t_unveil_lock_us": 10,
        "t_pledge_apply_us": 10,
        "t_business_loop_ready_us": 1,
        "t_total_startup_us": 121,
        "success": True,
    }
    return record


class StartupEvidenceTests(unittest.TestCase):
    def test_start_001_and_002_accept_twenty_unique_same_digest_records(self) -> None:
        records = []
        for _ in range(20):
            record = valid_record()
            records.append(record)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "raw.jsonl"
            path.write_text(
                "".join(json.dumps(record) + "\n" for record in records), encoding="utf-8"
            )
            self.assertEqual(len(load_records(path)), 20)

    def test_start_003_rejects_missing_and_negative_timing(self) -> None:
        missing = valid_record()
        del missing["t_load_us"]
        with self.assertRaises(ValueError):
            validate_record(missing)
        negative = valid_record()
        negative["t_schema_us"] = -1
        with self.assertRaises(ValueError):
            validate_record(negative)

    def test_start_004_rejects_inconsistent_total(self) -> None:
        record = valid_record()
        record["t_total_startup_us"] = 10_000
        with self.assertRaises(ValueError):
            validate_record(record)

    def test_start_002_rejects_digest_drift(self) -> None:
        records = []
        for _ in range(20):
            records.append(valid_record())
        changed = deepcopy(records[-1])
        changed["policy_digest"] = "sha256:" + "c" * 64
        records[-1] = changed
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "raw.jsonl"
            path.write_text(
                "".join(json.dumps(record) + "\n" for record in records), encoding="utf-8"
            )
            with self.assertRaises(ValueError):
                load_records(path)


if __name__ == "__main__":
    unittest.main()

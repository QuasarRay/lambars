#!/usr/bin/env python3
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
path = ROOT / "verification" / "qualification.json"
data = json.loads(path.read_text(encoding="utf-8"))

expected = {f"SC-{i:03d}" for i in range(1, 41)}
rows = data.get("findings", [])
ids = [row.get("id") for row in rows]
if set(ids) != expected or len(ids) != len(expected):
    missing = sorted(expected - set(ids))
    extra = sorted(set(ids) - expected)
    print(f"qualification inventory mismatch: missing={missing} extra={extra} count={len(ids)}")
    sys.exit(1)

required = data["policy"]["release_requires_status"]
blocking = [
    row for row in rows
    if row.get("release_blocking", True) and row.get("status") != required
]
if blocking:
    print("release blocked by unqualified safety findings:")
    for row in blocking:
        print(f"  {row['id']}: {row['status']}")
    sys.exit(1)

print("all SC-001..SC-040 findings are release-qualified")

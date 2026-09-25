#!/usr/bin/env python3
"""Verify the exact surface that Cargo would publish for both public crates."""

from __future__ import annotations
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]

CASES = [
    (
        "lambars",
        ["cargo", "package", "--list", "--allow-dirty", "--locked"],
        {"Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "README.md", "CHANGELOG.md", "LICENSE", ".cargo_vcs_info.json"},
        ("src/",),
    ),
    (
        "lambars-derive",
        ["cargo", "package", "--list", "--allow-dirty", "--locked", "--manifest-path", "lambars-derive/Cargo.toml"],
        {"Cargo.toml", "Cargo.toml.orig", ".cargo_vcs_info.json"},
        ("src/",),
    ),
]

failed = False
for name, command, exact, prefixes in CASES:
    result = subprocess.run(command, cwd=ROOT, check=True, text=True, capture_output=True)
    entries = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    unexpected = [
        entry for entry in entries
        if entry not in exact and not any(entry.startswith(prefix) for prefix in prefixes)
    ]
    if unexpected:
        failed = True
        print(f"{name}: unexpected package entries:")
        for entry in unexpected:
            print(f"  - {entry}")
    else:
        print(f"{name}: package surface accepted ({len(entries)} entries)")

if failed:
    sys.exit(1)

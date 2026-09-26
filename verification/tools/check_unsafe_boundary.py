#!/usr/bin/env python3
"""Fail if runtime source contains unsafe Rust or attempts to weaken the unsafe lint."""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = ROOT / "src"
PATTERNS = [
    re.compile(r"#!\s*\[\s*allow\s*\(\s*unsafe_code\s*\)\s*\]"),
    re.compile(r"\bunsafe\s*\{"),
    re.compile(r"\bunsafe\s+impl\b"),
    re.compile(r"\bunsafe\s+fn\b"),
    re.compile(r"\bunsafe\s+extern\b"),
]

violations: list[str] = []
for path in sorted(SRC.rglob("*.rs")):
    rel = path.relative_to(ROOT)
    text = path.read_text(encoding="utf-8")
    for lineno, line in enumerate(text.splitlines(), 1):
        # Documentation/comments may discuss unsafe; enforce Rust syntax only.
        code = line.split("//", 1)[0]
        if any(pattern.search(code) for pattern in PATTERNS):
            violations.append(f"{rel}:{lineno}: {line.strip()}")

lib_text = (SRC / "lib.rs").read_text(encoding="utf-8")
if "#![forbid(unsafe_code)]" not in lib_text:
    violations.append("src/lib.rs: missing #![forbid(unsafe_code)]")

cargo_text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
if 'unsafe_code = "forbid"' not in cargo_text:
    violations.append('Cargo.toml: workspace unsafe_code lint is not "forbid"')

if violations:
    print("unsafe-code policy violation:")
    for violation in violations:
        print(f"  - {violation}")
    sys.exit(1)

print("runtime unsafe-code policy: zero unsafe syntax; crate and Cargo lints are forbid")

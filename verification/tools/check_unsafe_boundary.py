#!/usr/bin/env python3
"""Fail if unsafe code escapes the explicitly audited lazy implementation boundary."""

from __future__ import annotations
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = ROOT / "src"
ALLOWED = {
    pathlib.Path("src/control/lazy.rs"),
    pathlib.Path("src/control/concurrent_lazy.rs"),
}
PATTERNS = [
    re.compile(r"#!\s*\[\s*allow\s*\(\s*unsafe_code\s*\)\s*\]"),
    re.compile(r"\bunsafe\s*\{"),
    re.compile(r"\bunsafe\s+impl\b"),
    re.compile(r"\bunsafe\s+fn\b"),
]

violations: list[str] = []
observed_allowed: set[pathlib.Path] = set()
for path in SRC.rglob("*.rs"):
    rel = path.relative_to(ROOT)
    text = path.read_text(encoding="utf-8")
    if any(pattern.search(text) for pattern in PATTERNS):
        if rel not in ALLOWED:
            violations.append(str(rel))
        else:
            observed_allowed.add(rel)

missing = ALLOWED - observed_allowed
if missing:
    violations.append("audited unsafe boundary unexpectedly disappeared/moved: " + ", ".join(map(str, sorted(missing))))

if violations:
    print("unsafe-boundary policy violation:")
    for violation in violations:
        print(f"  - {violation}")
    sys.exit(1)

print("unsafe code remains confined to the audited lazy/concurrent-lazy boundary")

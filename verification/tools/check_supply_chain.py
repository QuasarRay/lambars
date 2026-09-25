#!/usr/bin/env python3
"""Reject mutable or unverified CI supply-chain execution inputs."""

from __future__ import annotations
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github" / "workflows"
SHA_REF = re.compile(r"^[0-9a-f]{40}$")
USE_RE = re.compile(r"\buses:\s*([^\s#]+)")
violations: list[str] = []

for path in sorted(WORKFLOWS.glob("*.yml")):
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()

    for lineno, line in enumerate(lines, 1):
        m = USE_RE.search(line)
        if m:
            value = m.group(1)
            if value.startswith("./"):
                pass
            elif "@" not in value:
                violations.append(f"{path.relative_to(ROOT)}:{lineno}: action has no ref: {value}")
            else:
                _, ref = value.rsplit("@", 1)
                if not SHA_REF.fullmatch(ref):
                    violations.append(
                        f"{path.relative_to(ROOT)}:{lineno}: action ref is not immutable SHA: {value}"
                    )

        lower = line.lower()
        if "releases/latest" in lower:
            violations.append(f"{path.relative_to(ROOT)}:{lineno}: mutable latest release URL")
        if re.search(r"\bgit\s+clone\b", line):
            violations.append(f"{path.relative_to(ROOT)}:{lineno}: git clone HEAD is forbidden; fetch an exact commit")
        if re.search(r"apt-get[^\n]*\|\|\s*true", line):
            violations.append(f"{path.relative_to(ROOT)}:{lineno}: ignored apt failure")
        if re.search(r"\bpip\s+install\b|python\s+-m\s+pip\s+install", line):
            if "==" not in line:
                violations.append(f"{path.relative_to(ROOT)}:{lineno}: unpinned pip install")
        if "cargo install " in line:
            if "--version" not in line or "--locked" not in line:
                violations.append(f"{path.relative_to(ROOT)}:{lineno}: cargo install must pin version and lockfile")
        if "git -C " in line and " fetch " in line and "--depth" in line:
            if "COMMIT" not in line:
                violations.append(f"{path.relative_to(ROOT)}:{lineno}: git fetch does not name a commit variable")

        if "curl --fail --location --output" in line or "wget " in line:
            # Executable/archive downloads must be digest checked before use.
            window = "\n".join(lines[lineno: min(len(lines), lineno + 6)])
            if "sha256sum -c -" not in window:
                violations.append(
                    f"{path.relative_to(ROOT)}:{lineno}: downloaded artifact lacks nearby SHA-256 verification"
                )

if violations:
    print("mutable CI supply-chain inputs detected:")
    for violation in violations:
        print(f"  - {violation}")
    sys.exit(1)

print("CI supply-chain policy: all third-party actions and executable downloads are pinned")

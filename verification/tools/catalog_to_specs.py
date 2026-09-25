#!/usr/bin/env python3
"""Lossless production-readiness catalog -> formal-specification generator."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

ENTRY = re.compile(r"- \[ \] `([^`]+)`")
EXPECTED_TOTAL = 2533


def classify(name: str) -> str:
    if any(x in name for x in ("law_holds", "associativity", "identity_law", "homomorphism", "interchange")):
        return "algebraic-law"
    if "matches_std_" in name or "reference_model" in name:
        return "reference-model-equivalence"
    if any(x in name for x in ("compile_fail", "compile_time", "rejects_", "diagnostic")):
        return "compile-time-contract"
    if any(x in name for x in ("concurrent", "race", "thread", "wakeup", "deadlock", "atomic", "poison")):
        return "concurrency-safety"
    if any(x in name for x in ("serde", "serialize", "deserialize")):
        return "serialization-roundtrip"
    if any(x in name for x in ("benchmark", "performance", "allocation", "regression", "latency")):
        return "performance-regression"
    if any(x in name for x in ("feature", "msrv", "package", "release", "docs", "clippy", "format")):
        return "build-release-gate"
    if any(x in name for x in ("panic", "error", "invalid", "out_of_bounds", "overflow")):
        return "negative-boundary"
    return "behavioral-contract"


def humanize(name: str) -> str:
    text = name.replace("_", " ")
    return text[:1].upper() + text[1:] + "."


def parse_catalog(path: Path) -> list[dict[str, str]]:
    entries: list[dict[str, str]] = []
    for source in sorted(path.glob("[0-9][0-9]_*.md")):
        if source.name == "00_INDEX.md":
            continue
        category = ""
        section = ""
        for line in source.read_text(encoding="utf-8").splitlines():
            if line.startswith("# "):
                category = line[2:].strip()
            elif line.startswith("## "):
                section = line[3:].strip()
            else:
                match = ENTRY.fullmatch(line)
                if match:
                    name = match.group(1)
                    entries.append(
                        {
                            "name": name,
                            "category": category,
                            "section": section,
                            "kind": classify(name),
                            "source": source.name,
                        }
                    )
    return entries


def render(entry: dict[str, str]) -> str:
    name = entry["name"]
    return "\n".join(
        (
            f"## `{name}`",
            "",
            f"- **Kind:** `{entry['kind']}`",
            f"- **Requirement:** {humanize(name)}",
            f"- **Verus obligation:** Prove the requirement represented by `{name}` for all inputs admitted by explicit specification preconditions. The conclusion must not be encoded with `assume`, an axiom, or a trusted escape hatch.",
            f"- **Kani obligation:** Model-check the requirement represented by `{name}` over the complete finite state space selected by the harness. Every bound named by the specification is mandatory and every additional bound must be explicit.",
            f"- **Dual-backend consistency:** Verus and Kani must reference the same semantic predicate and neither backend may weaken `{name}`.",
            f"- **Regression obligation:** Repeated implementation or proof structure required by `{name}` must be represented through shared metaprogramming/metaverification rather than copied proof logic.",
            "",
        )
    )


def generate(catalog: Path, output: Path) -> None:
    entries = parse_catalog(catalog)
    names = [entry["name"] for entry in entries]
    if len(entries) != EXPECTED_TOTAL:
        raise SystemExit(f"expected {EXPECTED_TOTAL} entries, found {len(entries)}")
    if len(set(names)) != EXPECTED_TOTAL:
        raise SystemExit("catalog contains duplicate canonical specification IDs")

    output.mkdir(parents=True, exist_ok=True)
    by_source: dict[str, list[dict[str, str]]] = {}
    for entry in entries:
        by_source.setdefault(entry["source"], []).append(entry)

    emitted: list[str] = []
    manifest_files = []
    for source_name, source_entries in sorted(by_source.items()):
        destination = output / source_name.replace(".md", ".spec.md")
        category = source_entries[0]["category"]
        body = [
            f"# Formal Specifications — {category}",
            "",
            f"Source catalog: `{source_name}`",
            "",
            "> Every specification below is normative. The canonical test name is the canonical specification ID.",
            "",
        ]
        for entry in source_entries:
            body.append(render(entry))
            emitted.append(entry["name"])
        destination.write_text("\n".join(body), encoding="utf-8")
        manifest_files.append({"file": destination.name, "count": len(source_entries)})

    if emitted != names:
        raise SystemExit("emitted specification order differs from catalog order")
    if set(emitted) != set(names):
        raise SystemExit("emitted specification set differs from catalog set")

    registry = {"schema_version": 1, "count": EXPECTED_TOTAL, "entries": entries}
    registry_bytes = json.dumps(registry, indent=2).encode()
    (output / "spec-registry.json").write_bytes(registry_bytes)
    (output / "MANIFEST.json").write_text(
        json.dumps(
            {
                "total": EXPECTED_TOTAL,
                "files": manifest_files,
                "registry_sha256": hashlib.sha256(registry_bytes).hexdigest(),
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("catalog", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    generate(args.catalog, args.output)


if __name__ == "__main__":
    main()

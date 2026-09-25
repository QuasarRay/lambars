#!/usr/bin/env python3
"""Detect repeated Rust implementation/proof token shapes.

This detector deliberately ignores identifier spelling and literal values so it
finds structural repetition that should become a macro, generic helper,
contract family, or delegated proof.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import re
from pathlib import Path

TOKEN = re.compile(
    r'(?P<comment>//[^\n]*|/\*.*?\*/)|'
    r'(?P<string>r#*"[^"]*"#*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\')|'
    r'(?P<number>\b(?:0x[0-9A-Fa-f_]+|0b[01_]+|\d[\d_]*)\b)|'
    r'(?P<ident>\b[A-Za-z_][A-Za-z0-9_]*\b)|'
    r'(?P<punct>::|=>|->|==|!=|<=|>=|&&|\|\||\.\.=|\.\.|[{}()\[\];,:.!?&|+*/%<>=\-])',
    re.S,
)

RUST_KEYWORDS = {
    "as", "async", "await", "break", "const", "continue", "crate", "dyn",
    "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
}


def normalized_tokens(text: str) -> list[str]:
    out: list[str] = []
    for match in TOKEN.finditer(text):
        kind = match.lastgroup
        value = match.group()
        if kind == "comment":
            continue
        if kind == "ident" and value not in RUST_KEYWORDS:
            out.append("IDENT")
        elif kind == "string":
            out.append("STRING")
        elif kind == "number":
            out.append("NUMBER")
        else:
            out.append(value)
    return out


def windows(path: Path, width: int, stride: int):
    tokens = normalized_tokens(path.read_text(encoding="utf-8", errors="ignore"))
    for start in range(0, max(0, len(tokens) - width + 1), stride):
        shape = tuple(tokens[start : start + width])
        digest = hashlib.sha256("\x1f".join(shape).encode()).hexdigest()
        yield digest, start, shape


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("roots", nargs="+", type=Path)
    parser.add_argument("--width", type=int, default=48)
    parser.add_argument("--stride", type=int, default=12)
    parser.add_argument("--min-occurrences", type=int, default=3)
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()

    groups: dict[str, list[dict[str, object]]] = collections.defaultdict(list)
    shapes: dict[str, tuple[str, ...]] = {}
    for root in args.roots:
        paths = [root] if root.is_file() else sorted(root.rglob("*.rs"))
        for path in paths:
            if any(part in {"target", ".git"} for part in path.parts):
                continue
            for digest, offset, shape in windows(path, args.width, args.stride):
                shapes[digest] = shape
                groups[digest].append({"path": str(path), "token_offset": offset})

    repeated = [
        {
            "fingerprint": digest,
            "occurrences": occurrences,
            "shape": list(shapes[digest]),
        }
        for digest, occurrences in groups.items()
        if len({(o["path"], o["token_offset"]) for o in occurrences})
        >= args.min_occurrences
    ]
    repeated.sort(key=lambda item: (-len(item["occurrences"]), item["fingerprint"]))

    payload = {
        "width": args.width,
        "stride": args.stride,
        "min_occurrences": args.min_occurrences,
        "clusters": repeated,
    }
    text = json.dumps(payload, indent=2)
    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(text + "\n", encoding="utf-8")
    else:
        print(text)


if __name__ == "__main__":
    main()

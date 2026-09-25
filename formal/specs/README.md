# Formal specification catalog

`catalog.json` contains exactly 2,533 machine-readable production-readiness specifications, one for every canonical test name in the audit catalog.

Each record carries a stable ID, canonical name, source catalog file/section, normative requirement sentence, specification kind, reusable proof-pattern family, and the requirement that both Verus and Kani discharge it.

Validate uniqueness, count, metadata, and dual-backend requirements with:

```console
cargo run -p xtask -- formal-catalog validate
```

Catalog registration is not semantic proof coverage. Backend-specific coverage manifests must distinguish generated wiring from discharged proof obligations.

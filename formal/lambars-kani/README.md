# Lambars Kani backend

The Kani backend is isolated from normal Lambars builds. Kani injects its own crate and sets `cfg(kani)`, so proof harnesses live behind that configuration as recommended by Kani.

Run the backend with:

```console
cargo kani -p lambars-kani
```

This crate separates three concepts:

- **catalog wiring**: all 2,533 specs exist and require Kani;
- **semantic coverage**: a proof harness actually reasons about the Lambars implementation or an executable model;
- **metaverification regression**: Kani proves reusable law/boundary/variant/delegate machinery equivalent to the duplicated reference shape it replaces.

The structural-repetition scanner in `xtask formal-patterns` finds recurring implementation/proof shapes. New repeated shapes should be extracted into reusable macros/models and then covered by Kani regression-equivalence harnesses.

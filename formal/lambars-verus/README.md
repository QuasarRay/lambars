# Lambars Verus backend

This verifier is intentionally a standalone Cargo workspace. Lambars declares Rust 1.92 as its MSRV, while current Verus releases track a newer Rust toolchain. Keeping this crate outside the normal workspace prevents verifier-toolchain churn from changing the library's consumer MSRV.

Pinned inputs:

- Verus release: `0.2026.09.20.aef82ed`
- vstd: `0.0.0-2026-09-20-0158`

Run:

```console
cd formal/lambars-verus
cargo verus verify
```

The backend separates **formal model coverage** from direct verification of Lambars executable code. Where current Verus can consume ordinary Rust directly, `#[verus_verify(dual_spec)]` is used to keep executable and spec logic single-sourced. Where Lambars uses unsupported or verifier-hostile Rust constructs, this crate proves a model/refinement obligation and leaves an explicit bridge obligation rather than silently claiming direct verification.

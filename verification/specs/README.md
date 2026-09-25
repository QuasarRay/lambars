# Specification registry

The production-readiness catalog contains exactly **2,533** unique canonical test names.

Run:

```bash
python verification/tools/catalog_to_specs.py <catalog-directory> verification/specs/generated
```

The generator rejects duplicate IDs, missing categories, a total other than 2,533, or any emitted set that differs from the source set.

Each generated specification contains:

- canonical ID;
- source category and section;
- normative requirement derived without weakening the original name;
- Verus obligation;
- Kani obligation;
- dual-backend consistency requirement;
- metaverification/repetition requirement.

The generated output is committed separately from the generator because it is reviewable specification material, not build output.

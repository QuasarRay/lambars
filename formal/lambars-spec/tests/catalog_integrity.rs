use std::collections::BTreeSet;

use lambars_spec::{BackendSet, SPEC_COUNT, all_specs, find_spec, validate_catalog};

#[test]
fn formal_catalog_contains_exactly_2533_unique_specifications() {
    assert_eq!(SPEC_COUNT, 2_533);
    assert_eq!(all_specs().len(), 2_533);
    assert_eq!(
        all_specs()
            .iter()
            .map(|s| s.name)
            .collect::<BTreeSet<_>>()
            .len(),
        2_533
    );
}

#[test]
fn every_formal_catalog_specification_requires_both_verus_and_kani_backends() {
    assert!(
        all_specs()
            .iter()
            .all(|s| s.required_backends.contains(BackendSet::BOTH))
    );
}

#[test]
fn formal_catalog_integrity_validation_reports_no_schema_or_uniqueness_violations() {
    assert_eq!(validate_catalog(), []);
}

#[test]
fn formal_catalog_lookup_resolves_first_and_last_specifications() {
    let first = &all_specs()[0];
    let last = &all_specs()[all_specs().len() - 1];
    assert_eq!(find_spec(first.name), Some(first));
    assert_eq!(find_spec(last.name), Some(last));
}

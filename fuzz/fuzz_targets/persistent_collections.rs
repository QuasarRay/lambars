#![no_main]

use lambars::persistent::{OrderedUniqueSet, PersistentVector};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let values: Vec<u8> = data.iter().copied().take(128).collect();

    let mut vector = PersistentVector::new();
    for value in &values {
        vector = vector.push_back(*value);
    }
    assert_eq!(vector.len(), values.len());
    for (index, expected) in values.iter().enumerate() {
        assert_eq!(vector.get(index), Some(expected));
    }

    let set = OrderedUniqueSet::from_sorted_vec(values.clone());
    let sorted = set.to_sorted_vec();
    assert!(sorted.windows(2).all(|window| window[0] < window[1]));
    for value in values {
        assert!(sorted.contains(&value));
    }
});

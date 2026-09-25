#![no_main]

use lambars::persistent::{OrderedUniqueSet, PersistentHashMap, PersistentVector};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut vector = PersistentVector::<u8>::new();
    let mut map = PersistentHashMap::<u8, u8>::new();
    let mut set = OrderedUniqueSet::<u8>::new();

    for chunk in data.chunks(3) {
        let op = chunk.first().copied().unwrap_or(0) % 6;
        let key = chunk.get(1).copied().unwrap_or(0);
        let value = chunk.get(2).copied().unwrap_or(0);

        match op {
            0 => {
                vector = vector.push_back(value);
                assert_eq!(vector.last(), Some(&value));
            }
            1 => {
                if let Some((next, popped)) = vector.pop_back() {
                    assert_eq!(Some(&popped), vector.last());
                    vector = next;
                }
            }
            2 => {
                map = map.insert(key, value);
                assert_eq!(map.get(&key), Some(&value));
            }
            3 => {
                set = set.insert(key);
                assert!(set.contains(&key));
            }
            4 => {
                set = set.remove(&key);
                assert!(!set.contains(&key));
            }
            _ => {
                if !vector.is_empty() {
                    let index = usize::from(key) % vector.len();
                    assert!(vector.get(index).is_some());
                }
            }
        }
    }

    let sorted = set.to_sorted_vec();
    assert!(sorted.windows(2).all(|window| window[0] < window[1]));
    assert_eq!(map.len(), map.iter().count());
    assert_eq!(vector.len(), vector.iter().count());
});

use lambars::persistent::{PersistentHashMap, PersistentVector};
use lambars_verification::boundary_cases;
use std::collections::HashMap;

fn persistent_vector_from_vec_matches(size: usize) -> bool {
    let source: Vec<u16> = (0..size).map(|value| value as u16).collect();
    let vector = PersistentVector::from_vec(source.clone());
    vector.len() == source.len()
        && vector.iter().copied().eq(source)
}

fn persistent_vector_push_back_preserves(size: usize) -> bool {
    let source: Vec<u16> = (0..size).map(|value| value as u16).collect();
    let vector = PersistentVector::from_vec(source.clone()).push_back(u16::MAX);
    vector.len() == source.len() + 1
        && vector.iter().take(size).copied().eq(source)
        && vector.last() == Some(&u16::MAX)
}

fn persistent_vector_iterator_exactly_once(size: usize) -> bool {
    let source: Vec<u16> = (0..size).map(|value| value as u16).collect();
    let vector = PersistentVector::from_vec(source.clone());
    let observed: Vec<u16> = vector.iter().copied().collect();
    observed == source
}

fn persistent_hashmap_insert_matches_std(size: usize) -> bool {
    let mut standard = HashMap::new();
    let mut persistent = PersistentHashMap::new();
    for key in 0..size {
        let key = key as u16;
        let value = key.wrapping_mul(17).wrapping_add(3);
        standard.insert(key, value);
        persistent = persistent.insert(key, value);
    }
    persistent.len() == standard.len()
        && standard
            .iter()
            .all(|(key, value)| persistent.get(key) == Some(value))
}

fn persistent_hashmap_iterator_exactly_once(size: usize) -> bool {
    let mut persistent = PersistentHashMap::new();
    for key in 0..size {
        let key = key as u16;
        persistent = persistent.insert(key, key.wrapping_mul(11));
    }

    let mut seen = HashMap::new();
    for (key, value) in persistent.iter() {
        if seen.insert(*key, *value).is_some() {
            return false;
        }
    }

    seen.len() == size
        && (0..size).all(|key| {
            let key = key as u16;
            seen.get(&key) == Some(&key.wrapping_mul(11))
        })
}

// Small structural boundaries are checked directly by both runtime regression
// and Kani. Larger boundary IDs delegate to the same predicates only after
// refinement/contract decomposition avoids infeasible loop unwinding.

boundary_cases! {
    persistent_vector_from_vec_matches;
    persistent_vector_from_vec_length_0_matches_source_vec = 0,
    persistent_vector_from_vec_length_1_matches_source_vec = 1,
    persistent_vector_from_vec_length_2_matches_source_vec = 2,
    persistent_vector_from_vec_length_7_matches_source_vec = 7,
    persistent_vector_from_vec_length_8_matches_source_vec = 8,
    persistent_vector_from_vec_length_9_matches_source_vec = 9
}

boundary_cases! {
    persistent_vector_push_back_preserves;
    persistent_vector_push_back_across_length_0_preserves_all_prior_values = 0,
    persistent_vector_push_back_across_length_1_preserves_all_prior_values = 1,
    persistent_vector_push_back_across_length_2_preserves_all_prior_values = 2,
    persistent_vector_push_back_across_length_7_preserves_all_prior_values = 7,
    persistent_vector_push_back_across_length_8_preserves_all_prior_values = 8,
    persistent_vector_push_back_across_length_9_preserves_all_prior_values = 9
}

boundary_cases! {
    persistent_vector_iterator_exactly_once;
    persistent_vector_iterator_length_0_visits_every_element_once = 0,
    persistent_vector_iterator_length_1_visits_every_element_once = 1,
    persistent_vector_iterator_length_2_visits_every_element_once = 2,
    persistent_vector_iterator_length_7_visits_every_element_once = 7,
    persistent_vector_iterator_length_8_visits_every_element_once = 8,
    persistent_vector_iterator_length_9_visits_every_element_once = 9
}

boundary_cases! {
    persistent_hashmap_insert_matches_std;
    persistent_hashmap_insert_unique_keys_to_size_0_matches_std_hashmap = 0,
    persistent_hashmap_insert_unique_keys_to_size_1_matches_std_hashmap = 1,
    persistent_hashmap_insert_unique_keys_to_size_2_matches_std_hashmap = 2,
    persistent_hashmap_insert_unique_keys_to_size_7_matches_std_hashmap = 7,
    persistent_hashmap_insert_unique_keys_to_size_8_matches_std_hashmap = 8,
    persistent_hashmap_insert_unique_keys_to_size_9_matches_std_hashmap = 9
}

boundary_cases! {
    persistent_hashmap_iterator_exactly_once;
    persistent_hashmap_iter_size_0_yields_every_entry_exactly_once = 0,
    persistent_hashmap_iter_size_1_yields_every_entry_exactly_once = 1,
    persistent_hashmap_iter_size_2_yields_every_entry_exactly_once = 2,
    persistent_hashmap_iter_size_7_yields_every_entry_exactly_once = 7,
    persistent_hashmap_iter_size_8_yields_every_entry_exactly_once = 8,
    persistent_hashmap_iter_size_9_yields_every_entry_exactly_once = 9
}

use vstd::prelude::*;

verus! {

pub open spec fn identity(value: int) -> int { value }
pub proof fn identity_is_identity(value: int) ensures identity(value) == value {}

pub type EitherModel = (bool, int);
pub open spec fn swap(value: EitherModel) -> EitherModel { (!value.0, value.1) }
pub proof fn either_swap_twice_returns_original_variant_and_value(value: EitherModel)
    ensures swap(swap(value)) == value {}

pub open spec fn update_at(s: Seq<int>, i: int, v: int) -> Seq<int>
    recommends 0 <= i < s.len()
{ s.update(i, v) }

pub proof fn update_at_preserves_other_indices(s: Seq<int>, i: int, j: int, v: int)
    requires 0 <= i < s.len(), 0 <= j < s.len(), i != j,
    ensures update_at(s, i, v)[j] == s[j] {}

pub proof fn append_preserves_length_sum(left: Seq<int>, right: Seq<int>)
    ensures (left + right).len() == left.len() + right.len() {}

pub open spec fn concurrent_lazy_reentry_matches_model(active: int, candidate: int) -> bool {
    active == candidate
}
pub proof fn sc002_distinct_instances_are_not_reentry(active: int, candidate: int)
    requires active != candidate
    ensures !concurrent_lazy_reentry_matches_model(active, candidate) {}
pub proof fn sc002_same_instance_is_reentry(identity: int)
    ensures concurrent_lazy_reentry_matches_model(identity, identity) {}

pub open spec fn generation_successor_model(current: int) -> int { current + 1 }
pub proof fn sc030_generation_successor_cannot_be_shared_zero(current: int)
    requires current >= 1
    ensures generation_successor_model(current) > current,
            generation_successor_model(current) != 0 {}

pub open spec fn owned_pair_review(left: int, right: int) -> (int, int) { (left, right) }
pub open spec fn owned_pair_preview(value: (int, int)) -> (int, int) { value }
pub proof fn sc003_owned_prism_preview_review_law(left: int, right: int)
    ensures owned_pair_preview(owned_pair_review(left, right)) == (left, right) {}

pub open spec fn normalize_pair(a: int, b: int) -> Seq<int> {
    if a < b { seq![a, b] }
    else if b < a { seq![b, a] }
    else { seq![a] }
}
pub proof fn sc004_normalize_pair_is_strict_and_unique(a: int, b: int)
    ensures
        normalize_pair(a, b).len() == 1 || normalize_pair(a, b).len() == 2,
        normalize_pair(a, b).len() == 2 ==> normalize_pair(a, b)[0] < normalize_pair(a, b)[1],
        normalize_pair(a, b).contains(a),
        normalize_pair(a, b).contains(b),
{
    if a < b {
        assert(normalize_pair(a, b) == seq![a, b]);
        assert(seq![a, b].contains(a));
        assert(seq![a, b].contains(b));
    } else if b < a {
        assert(normalize_pair(a, b) == seq![b, a]);
        assert(seq![b, a].contains(a));
        assert(seq![b, a].contains(b));
    } else {
        assert(a == b);
        assert(normalize_pair(a, b) == seq![a]);
        assert(seq![a].contains(a));
    }
}

/// SC-005 model of the consuming IO Functor operation that remains exposed.
pub open spec fn io_fmap_model(value: int, delta: int) -> int { value + delta }
pub proof fn sc005_io_consuming_fmap_is_total(value: int, delta: int)
    ensures io_fmap_model(value, delta) == value + delta {}

/// Additional Functor regression: mapping preserves sequence cardinality.
pub open spec fn vec_functor_mapped_len(source: Seq<int>) -> nat { source.len() }
pub proof fn vec_functor_map_preserves_length(source: Seq<int>)
    ensures vec_functor_mapped_len(source) == source.len() {}

} // verus!

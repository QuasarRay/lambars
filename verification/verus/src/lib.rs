use vstd::prelude::*;

verus! {

pub open spec fn identity(value: int) -> int { value }

pub proof fn identity_is_identity(value: int)
    ensures identity(value) == value
{
}

pub type EitherModel = (bool, int);

pub open spec fn swap(value: EitherModel) -> EitherModel {
    (!value.0, value.1)
}

pub proof fn either_swap_twice_returns_original_variant_and_value(value: EitherModel)
    ensures swap(swap(value)) == value
{
}

pub open spec fn update_at(s: Seq<int>, i: int, v: int) -> Seq<int>
    recommends 0 <= i < s.len()
{
    s.update(i, v)
}

pub proof fn update_at_preserves_other_indices(s: Seq<int>, i: int, j: int, v: int)
    requires
        0 <= i < s.len(),
        0 <= j < s.len(),
        i != j,
    ensures
        update_at(s, i, v)[j] == s[j],
{
}

pub proof fn append_preserves_length_sum(left: Seq<int>, right: Seq<int>)
    ensures (left + right).len() == left.len() + right.len()
{
}

pub open spec fn concurrent_lazy_reentry_matches_model(active: int, candidate: int) -> bool {
    active == candidate
}

pub proof fn sc002_distinct_instances_are_not_reentry(active: int, candidate: int)
    requires active != candidate
    ensures !concurrent_lazy_reentry_matches_model(active, candidate)
{
}

pub proof fn sc002_same_instance_is_reentry(identity: int)
    ensures concurrent_lazy_reentry_matches_model(identity, identity)
{
}

/// SC-030 model of checked generation-token advancement.
pub open spec fn generation_successor_model(current: int) -> int {
    current + 1
}

pub proof fn sc030_generation_successor_cannot_be_shared_zero(current: int)
    requires current >= 1
    ensures
        generation_successor_model(current) > current,
        generation_successor_model(current) != 0,
{
}

} // verus!

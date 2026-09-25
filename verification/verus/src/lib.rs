use vstd::prelude::*;

verus! {

/// Shared mathematical model for the identity law. Implementation refinement
/// to each Lambars instance is tracked separately and must not use an assumed
/// specification for the Lambars implementation.
pub open spec fn identity<T>(x: T) -> T {
    x
}

pub proof fn identity_is_identity<T>(x: T)
    ensures identity(x) == x
{
}

/// Model of Either swap used by both the implementation-refinement proof and
/// the Kani executable harness family.
pub enum EitherModel<L, R> {
    Left(L),
    Right(R),
}

pub open spec fn swap<L, R>(value: EitherModel<L, R>) -> EitherModel<R, L> {
    match value {
        EitherModel::Left(left) => EitherModel::Right(left),
        EitherModel::Right(right) => EitherModel::Left(right),
    }
}

pub proof fn either_swap_twice_returns_original_variant_and_value<L, R>(
    value: EitherModel<L, R>,
)
    ensures swap(swap(value)) == value
{
}

/// Generic sequence update model used by persistent vector/list refinement
/// proofs. This lemma is intentionally representation-independent.
pub open spec fn update_at<T>(s: Seq<T>, i: int, v: T) -> Seq<T>
    recommends 0 <= i < s.len()
{
    s.update(i, v)
}

pub proof fn update_at_preserves_other_indices<T>(s: Seq<T>, i: int, j: int, v: T)
    requires
        0 <= i < s.len(),
        0 <= j < s.len(),
        i != j,
    ensures
        update_at(s, i, v)[j] == s[j],
{
}

/// Generic append model for persistent sequence structures.
pub proof fn append_preserves_length_sum<T>(left: Seq<T>, right: Seq<T>)
    ensures
        (left + right).len() == left.len() + right.len(),
{
}

/// SC-002 model of the production `concurrent_lazy_reentry_matches` predicate.
/// The executable implementation uses identity equality exactly as modeled here.
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

/// These model lemmas are not counted as completed Lambars implementation
/// proofs until a refinement lemma connects the executable Lambars operation
/// to the model without an assumed implementation contract.
}

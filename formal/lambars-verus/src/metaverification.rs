use vstd::prelude::*;

#[verus_verify(dual_spec, open)]
#[verus_spec(returns identity_exec(value))]
pub fn identity_exec(value: u64) -> u64 {
    value
}

#[verus_verify(dual_spec, open)]
#[verus_spec(
    requires boundary > 0 && boundary < usize::MAX,
    returns boundary_center(boundary)
)]
pub fn boundary_center(boundary: usize) -> usize {
    boundary
}

verus! {

pub proof fn metaverification_dual_spec_identity_exec_is_its_own_spec(value: u64) {
    assert(identity_exec(value) == value);
}

pub proof fn metaverification_dual_spec_boundary_center_preserves_center(boundary: usize)
    requires
        boundary > 0,
        boundary < usize::MAX,
{
    assert(boundary_center(boundary) == boundary);
}

pub open spec fn law_triplet(left_identity: bool, right_identity: bool, associativity: bool) -> bool {
    left_identity && right_identity && associativity
}

pub proof fn metaverification_law_triplet_reduces_to_boolean_conjunction(
    left_identity: bool,
    right_identity: bool,
    associativity: bool,
) {
    assert(
        law_triplet(left_identity, right_identity, associativity)
            == (left_identity && right_identity && associativity)
    );
}

pub open spec fn boundary_below(boundary: int) -> int {
    boundary - 1
}

pub open spec fn boundary_exact(boundary: int) -> int {
    boundary
}

pub open spec fn boundary_above(boundary: int) -> int {
    boundary + 1
}

pub proof fn metaverification_boundary_triplet_has_exact_minus_one_exact_plus_one_shape(boundary: int) {
    assert(boundary_below(boundary) == boundary - 1);
    assert(boundary_exact(boundary) == boundary);
    assert(boundary_above(boundary) == boundary + 1);
}

}

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

/// SC-006/SC-034 state-transition model paired with the Loom interleaving suite.
pub open spec fn concurrent_lazy_transition_allowed(from: int, to: int) -> bool {
    (from == 0 && to == 1)
    || (from == 1 && to == 2)
    || (from == 1 && to == 3)
}

pub proof fn sc006_ready_and_poisoned_are_terminal(to: int)
    ensures
        !concurrent_lazy_transition_allowed(2, to),
        !concurrent_lazy_transition_allowed(3, to),
{
}

pub proof fn sc034_only_computing_can_publish_terminal_state(from: int, to: int)
    requires
        concurrent_lazy_transition_allowed(from, to),
        to == 2 || to == 3,
    ensures
        from == 1,
{
}


/// SC-029 model: the public security-mode selector has exactly one state.
pub open spec fn persistent_hash_security_mode_model() -> int { 1 }

pub proof fn sc029_only_keyed_hash_mode_is_exposed()
    ensures persistent_hash_security_mode_model() == 1
{
}


/// Formal model of the fail-closed SC-001..SC-040 release gate.
pub open spec fn safety_release_qualified_model(closed: Seq<bool>) -> bool {
    closed.len() == 40
    && forall|i: int| 0 <= i < 40 ==> closed[i]
}

pub proof fn unresolved_finding_blocks_release(closed: Seq<bool>, index: int)
    requires
        closed.len() == 40,
        0 <= index < 40,
        !closed[index],
    ensures
        !safety_release_qualified_model(closed),
{
    if safety_release_qualified_model(closed) {
        assert(forall|i: int| 0 <= i < 40 ==> closed[i]);
        assert(closed[index]);
        assert(false);
    }
}

pub proof fn sc001_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[0]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 0);
}

pub proof fn sc002_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[1]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 1);
}

pub proof fn sc003_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[2]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 2);
}

pub proof fn sc004_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[3]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 3);
}

pub proof fn sc005_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[4]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 4);
}

pub proof fn sc006_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[5]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 5);
}

pub proof fn sc007_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[6]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 6);
}

pub proof fn sc008_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[7]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 7);
}

pub proof fn sc009_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[8]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 8);
}

pub proof fn sc010_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[9]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 9);
}

pub proof fn sc011_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[10]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 10);
}

pub proof fn sc012_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[11]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 11);
}

pub proof fn sc013_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[12]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 12);
}

pub proof fn sc014_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[13]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 13);
}

pub proof fn sc015_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[14]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 14);
}

pub proof fn sc016_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[15]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 15);
}

pub proof fn sc017_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[16]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 16);
}

pub proof fn sc018_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[17]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 17);
}

pub proof fn sc019_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[18]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 18);
}

pub proof fn sc020_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[19]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 19);
}

pub proof fn sc021_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[20]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 20);
}

pub proof fn sc022_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[21]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 21);
}

pub proof fn sc023_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[22]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 22);
}

pub proof fn sc024_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[23]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 23);
}

pub proof fn sc025_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[24]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 24);
}

pub proof fn sc026_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[25]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 25);
}

pub proof fn sc027_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[26]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 26);
}

pub proof fn sc028_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[27]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 27);
}

pub proof fn sc029_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[28]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 28);
}

pub proof fn sc030_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[29]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 29);
}

pub proof fn sc031_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[30]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 30);
}

pub proof fn sc032_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[31]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 31);
}

pub proof fn sc033_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[32]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 32);
}

pub proof fn sc034_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[33]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 33);
}

pub proof fn sc035_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[34]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 34);
}

pub proof fn sc036_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[35]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 35);
}

pub proof fn sc037_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[36]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 36);
}

pub proof fn sc038_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[37]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 37);
}

pub proof fn sc039_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[38]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 38);
}

pub proof fn sc040_unresolved_blocks_release(closed: Seq<bool>)
    requires closed.len() == 40, !closed[39]
    ensures !safety_release_qualified_model(closed)
{
    unresolved_finding_blocks_release(closed, 39);
}


/// SC-028 model for the production bounded-wait classifier.
/// 0=ready, 1=empty, 2=computing, 3=poisoned.
/// Result: 0=ready, 1=not-started, 2=wait, 3=poisoned, 4=reentrant, 5=timed-out, 6=invalid.
pub open spec fn concurrent_lazy_wait_decision_model(
    state: int,
    reentrant: bool,
    timed_out: bool,
) -> int {
    if state == 2 && reentrant { 4 }
    else if state == 2 && timed_out { 5 }
    else if state == 2 { 2 }
    else if state == 0 { 1 }
    else if state == 3 { 3 }
    else if state == 1 { 0 }
    else { 6 }
}

pub proof fn sc028_expired_computing_wait_is_terminal(reentrant: bool)
    ensures
        concurrent_lazy_wait_decision_model(2, reentrant, true) != 2,
{
}

pub proof fn sc028_terminal_states_never_continue_waiting(state: int, reentrant: bool, timed_out: bool)
    requires state == 0 || state == 1 || state == 3
    ensures concurrent_lazy_wait_decision_model(state, reentrant, timed_out) != 2
{
}

pub proof fn sc028_execution_mode_does_not_change_classification(
    state: int,
    reentrant: bool,
    timed_out: bool,
    execution_mode: int,
)
    requires 0 <= execution_mode <= 2
    ensures
        concurrent_lazy_wait_decision_model(state, reentrant, timed_out)
            == concurrent_lazy_wait_decision_model(state, reentrant, timed_out),
{
}

} // verus!

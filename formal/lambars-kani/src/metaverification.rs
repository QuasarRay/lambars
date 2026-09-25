use lambars_formal_macros::{FormalModel, ProofDelegate, dual_backend_adapter, formal_delegate};
use lambars_spec::{FormalModel as FormalModelTrait, ProofDelegate as ProofDelegateTrait};

use crate::kani_proof;

#[derive(FormalModel, ProofDelegate)]
#[formal_model_name("BoundaryModel")]
#[proof_delegate_target("boundary_points")]
struct BoundaryModel;

#[dual_backend_adapter]
fn boundary_points(boundary: usize) -> [usize; 3] {
    [boundary - 1, boundary, boundary + 1]
}

formal_delegate!(
    fn delegated_boundary_points(boundary: usize) -> [usize; 3] => boundary_points(boundary);
);

fn handwritten_boundary_points(boundary: usize) -> [usize; 3] {
    let below = boundary - 1;
    let exact = boundary;
    let above = boundary + 1;
    [below, exact, above]
}

#[dual_backend_adapter]
fn law_triplet(left_identity: bool, right_identity: bool, associativity: bool) -> bool {
    left_identity && right_identity && associativity
}

formal_delegate!(
    fn delegated_law_triplet(
        left_identity: bool,
        right_identity: bool,
        associativity: bool,
    ) -> bool => law_triplet(left_identity, right_identity, associativity);
);

kani_proof!(
    metaverification_boundary_triplet_adapter_matches_handwritten_reference_shape,
    {
        let boundary: usize = kani::any();
        kani::assume(boundary > 0 && boundary < usize::MAX);
        assert_eq!(
            boundary_points(boundary),
            handwritten_boundary_points(boundary)
        );
    }
);

kani_proof!(
    metaverification_generated_kani_boundary_adapter_matches_shared_adapter,
    {
        let boundary: usize = kani::any();
        kani::assume(boundary > 0 && boundary < usize::MAX);
        assert_eq!(
            boundary_points__kani_adapter(boundary),
            boundary_points(boundary)
        );
    }
);

kani_proof!(
    metaverification_generated_verus_boundary_adapter_matches_shared_adapter,
    {
        let boundary: usize = kani::any();
        kani::assume(boundary > 0 && boundary < usize::MAX);
        assert_eq!(
            boundary_points__verus_adapter(boundary),
            boundary_points(boundary)
        );
    }
);

kani_proof!(
    metaverification_delegate_macro_boundary_function_matches_shared_target,
    {
        let boundary: usize = kani::any();
        kani::assume(boundary > 0 && boundary < usize::MAX);
        assert_eq!(
            delegated_boundary_points(boundary),
            boundary_points(boundary)
        );
    }
);

kani_proof!(
    metaverification_law_triplet_adapter_is_exact_boolean_conjunction,
    {
        let left_identity: bool = kani::any();
        let right_identity: bool = kani::any();
        let associativity: bool = kani::any();
        assert_eq!(
            law_triplet(left_identity, right_identity, associativity),
            left_identity && right_identity && associativity
        );
    }
);

kani_proof!(
    metaverification_delegate_macro_law_triplet_matches_shared_target,
    {
        let left_identity: bool = kani::any();
        let right_identity: bool = kani::any();
        let associativity: bool = kani::any();
        assert_eq!(
            delegated_law_triplet(left_identity, right_identity, associativity),
            law_triplet(left_identity, right_identity, associativity)
        );
    }
);

#[test]
fn derive_macros_expose_stable_metaverification_metadata() {
    assert_eq!(BoundaryModel::FORMAL_MODEL_NAME, "BoundaryModel");
    assert_eq!(BoundaryModel::PROOF_DELEGATE_TARGET, "boundary_points");
}

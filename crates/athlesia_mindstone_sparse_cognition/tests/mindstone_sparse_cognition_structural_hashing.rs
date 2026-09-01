use athlesia_mindstone_sparse_cognition::{
    CognitiveAdmissionClass, CognitiveBudget, CognitiveSignal, CognitiveStructure,
    MindstoneNoveltyGate, MindstoneSignalProfile, MindstoneStructuralNoveltyGate, NoveltyMemory,
    NoveltyStatus, SparseCognitionPolicy, StructuralHasher,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(children: Vec<CognitiveStructure>) -> CognitiveStructure {
    CognitiveStructure::ordered(children).unwrap()
}

fn unordered(children: Vec<CognitiveStructure>) -> CognitiveStructure {
    CognitiveStructure::unordered(children).unwrap()
}

fn memory(capacity: usize) -> NoveltyMemory {
    NoveltyMemory::new(capacity).unwrap()
}

fn profile(
    surprise: u16,
    uncertainty: u16,
    novelty: u16,
    learning_progress: u16,
    information_gain: u16,
) -> MindstoneSignalProfile {
    MindstoneSignalProfile::new(
        signal(surprise),
        signal(uncertainty),
        signal(novelty),
        signal(learning_progress),
        signal(information_gain),
    )
}

fn policy() -> SparseCognitionPolicy {
    SparseCognitionPolicy::new(signal(200), signal(600), 2, 8).unwrap()
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

#[test]
fn structural_compounds_require_nonempty_children() {
    assert_eq!(CognitiveStructure::ordered(Vec::new(),), None);

    assert_eq!(CognitiveStructure::unordered(Vec::new(),), None);

    assert!(atom(1,).is_atom());
}

#[test]
fn ordered_structure_preserves_order_and_hash_identity() {
    let left = ordered(vec![atom(1), atom(2)]);

    let right = ordered(vec![atom(2), atom(1)]);

    assert_ne!(left, right);

    assert_ne!(
        StructuralHasher::fingerprint(&left,),
        StructuralHasher::fingerprint(&right,)
    );
}

#[test]
fn unordered_structure_is_canonical_across_input_order() {
    let left = unordered(vec![atom(1), atom(2), atom(3)]);

    let right = unordered(vec![atom(3), atom(1), atom(2)]);

    assert_eq!(left, right);

    assert_eq!(
        StructuralHasher::fingerprint(&left,),
        StructuralHasher::fingerprint(&right,)
    );
}

#[test]
fn unordered_structure_preserves_multiplicity() {
    let single = unordered(vec![atom(1), atom(2)]);

    let repeated = unordered(vec![atom(1), atom(1), atom(2)]);

    assert_ne!(single, repeated);

    assert_ne!(
        StructuralHasher::fingerprint(&single,),
        StructuralHasher::fingerprint(&repeated,)
    );
}

#[test]
fn structural_kind_is_part_of_identity() {
    let ordered_form = ordered(vec![atom(7)]);

    let unordered_form = unordered(vec![atom(7)]);

    let atom_form = atom(7);

    assert_ne!(
        StructuralHasher::fingerprint(&ordered_form,),
        StructuralHasher::fingerprint(&unordered_form,)
    );

    assert_ne!(
        StructuralHasher::fingerprint(&ordered_form,),
        StructuralHasher::fingerprint(&atom_form,)
    );

    assert_ne!(
        StructuralHasher::fingerprint(&unordered_form,),
        StructuralHasher::fingerprint(&atom_form,)
    );
}

#[test]
fn nested_structure_boundaries_are_preserved() {
    let flat = ordered(vec![atom(1), atom(2), atom(3)]);

    let nested = ordered(vec![atom(1), ordered(vec![atom(2), atom(3)])]);

    assert_ne!(flat, nested);

    assert_ne!(
        StructuralHasher::fingerprint(&flat,),
        StructuralHasher::fingerprint(&nested,)
    );
}

#[test]
fn structural_hashing_is_exact_deterministic_and_non_mutating() {
    let structure = ordered(vec![atom(9), unordered(vec![atom(30), atom(10), atom(20)])]);

    let before = structure.clone();

    let first = StructuralHasher::fingerprint(&structure);

    let second = StructuralHasher::fingerprint(&structure);

    let cloned = StructuralHasher::fingerprint(&structure.clone());

    assert_eq!(first, second);

    assert_eq!(second, cloned);

    assert_eq!(structure, before);
}

#[test]
fn equal_structures_have_equal_fingerprints() {
    let first = ordered(vec![unordered(vec![atom(2), atom(1)]), atom(99)]);

    let second = ordered(vec![unordered(vec![atom(1), atom(2)]), atom(99)]);

    assert_eq!(first, second);

    assert_eq!(
        StructuralHasher::fingerprint(&first,),
        StructuralHasher::fingerprint(&second,)
    );
}

#[test]
fn first_structural_observation_is_novel_without_manual_fingerprint() {
    let structure = ordered(vec![atom(4), atom(8)]);

    let expected_fingerprint = StructuralHasher::fingerprint(&structure);

    let result = MindstoneStructuralNoveltyGate::evaluate(
        memory(4),
        structure.clone(),
        profile(0, 0, 0, 0, 0),
        policy(),
        budget(100),
    );

    assert_eq!(result.structure(), &structure);

    assert_eq!(result.fingerprint(), expected_fingerprint);

    assert!(result.is_novel());

    assert_eq!(result.admission().novelty().status(), NoveltyStatus::Novel);

    assert_eq!(
        result.decision().class(),
        CognitiveAdmissionClass::CheapUpdate
    );
}

#[test]
fn reordered_unordered_structure_is_recognized_as_known() {
    let first_structure = unordered(vec![atom(10), atom(20), atom(30)]);

    let second_structure = unordered(vec![atom(30), atom(10), atom(20)]);

    let first = MindstoneStructuralNoveltyGate::evaluate(
        memory(4),
        first_structure,
        profile(0, 0, 0, 0, 0),
        policy(),
        budget(100),
    );

    let second = MindstoneStructuralNoveltyGate::evaluate(
        first.admission().novelty().memory_after().clone(),
        second_structure,
        profile(0, 0, 1000, 0, 0),
        policy(),
        budget(100),
    );

    assert_eq!(second.admission().novelty().status(), NoveltyStatus::Known);

    assert_eq!(
        second.admission().profile().novelty(),
        CognitiveSignal::zero()
    );

    assert_eq!(second.decision().class(), CognitiveAdmissionClass::Ignore);
}

#[test]
fn reordered_ordered_structure_remains_novel() {
    let first = MindstoneStructuralNoveltyGate::evaluate(
        memory(4),
        ordered(vec![atom(10), atom(20)]),
        profile(0, 0, 0, 0, 0),
        policy(),
        budget(100),
    );

    let second = MindstoneStructuralNoveltyGate::evaluate(
        first.admission().novelty().memory_after().clone(),
        ordered(vec![atom(20), atom(10)]),
        profile(0, 0, 0, 0, 0),
        policy(),
        budget(100),
    );

    assert_eq!(second.admission().novelty().status(), NoveltyStatus::Novel);

    assert!(second.is_novel());
}

#[test]
fn structural_novelty_facade_matches_direct_hash_gate_and_preserves_budget() {
    let structure = ordered(vec![atom(50), unordered(vec![atom(3), atom(1), atom(2)])]);

    let structure_before = structure.clone();

    let input_memory = memory(5);

    let memory_before = input_memory.clone();

    let input_profile = profile(1000, 1000, 0, 1000, 1000);

    let profile_before = input_profile;

    let input_policy = policy();

    let input_budget = budget(3);

    let fingerprint = StructuralHasher::fingerprint(&structure);

    let direct = MindstoneNoveltyGate::evaluate(
        input_memory.clone(),
        fingerprint,
        input_profile,
        input_policy,
        input_budget,
    );

    let structural = MindstoneStructuralNoveltyGate::evaluate(
        input_memory.clone(),
        structure.clone(),
        input_profile,
        input_policy,
        input_budget,
    );

    let repeated = MindstoneStructuralNoveltyGate::evaluate(
        input_memory.clone(),
        structure.clone(),
        input_profile,
        input_policy,
        input_budget,
    );

    assert_eq!(structural.admission(), &direct);

    assert_eq!(structural, repeated);

    assert_eq!(
        structural.decision().class(),
        CognitiveAdmissionClass::Deliberate
    );

    assert_eq!(structural.decision().requested_units(), 8);

    assert_eq!(structural.decision().granted_units(), 3);

    assert!(structural.decision().is_budget_limited());

    assert_eq!(structure, structure_before);

    assert_eq!(input_memory, memory_before);

    assert_eq!(input_profile, profile_before);
}

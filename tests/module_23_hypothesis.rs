use athlesia::{
    Encoder, HypothesisInducer, PrimitiveDiscovery, PrimitiveSignature, RelationKind,
    RelationalStructure, StructuralHypothesis,
};

fn hypotheses<T>(values: &[T]) -> Vec<StructuralHypothesis>
where
    T: Eq + std::hash::Hash,
{
    let encoder = Encoder::new();
    let sequence = encoder.encode(values);

    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::default().discover(&relations);

    HypothesisInducer::default().induce(&primitives)
}

#[test]
fn repeated_primitive_induces_hypothesis() {
    let result = hypotheses(&[1, 2, 1, 2, 3]);

    assert_eq!(result.len(), 1);

    let hypothesis = &result[0];

    assert_eq!(hypothesis.evidence_count(), 2);
    assert_eq!(hypothesis.description_cost(), 1);
    assert_eq!(hypothesis.compression_gain(), 1);
    assert!(hypothesis.is_compressive());

    assert_eq!(
        hypothesis.signatures(),
        &[PrimitiveSignature::new(RelationKind::Equal, 2,)]
    );
}

#[test]
fn unsupported_structure_produces_no_hypothesis() {
    let result = hypotheses(&[1, 2, 1, 3]);

    assert!(result.is_empty());
}

#[test]
fn hypothesis_is_value_invariant() {
    let first = hypotheses(&[1, 2, 1, 2, 3]);

    let second = hypotheses(&[847, 13, 847, 13, 999]);

    assert_eq!(first, second);
}

#[test]
fn hypothesis_is_position_independent() {
    let first = hypotheses(&[1, 2, 1, 2, 9]);

    let second = hypotheses(&[9, 1, 2, 1, 2]);

    assert_eq!(first, second);
}

#[test]
fn hypothesis_induction_is_deterministic() {
    let first = hypotheses(&[1, 1, 1, 1]);

    let second = hypotheses(&[1, 1, 1, 1]);

    assert_eq!(first, second);
}

#[test]
fn multiple_primitives_create_competing_hypotheses() {
    let result = hypotheses(&[1, 1, 1, 1]);

    assert_eq!(result.len(), 3);

    assert_eq!(result[0].compression_gain(), 3);

    assert_eq!(result[0].description_cost(), 2);

    assert_eq!(result[0].evidence_count(), 5);
}

#[test]
fn composite_hypothesis_contains_multiple_signatures() {
    let result = hypotheses(&[1, 1, 1, 1]);

    let composite = &result[0];

    assert!(composite.contains(PrimitiveSignature::new(RelationKind::Equal, 1,)));

    assert!(composite.contains(PrimitiveSignature::new(RelationKind::Equal, 2,)));

    assert_eq!(composite.signatures().len(), 2);
}

#[test]
fn hypothesis_ranking_prefers_compression_gain() {
    let result = hypotheses(&[1, 1, 1, 1]);

    assert_eq!(result[0].compression_gain(), 3);

    assert_eq!(result[1].compression_gain(), 2);

    assert_eq!(result[2].compression_gain(), 1);
}

#[test]
fn stricter_gain_threshold_prunes_weak_hypothesis() {
    let encoder = Encoder::new();

    let sequence = encoder.encode(&[1, 1, 1, 1]);

    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::default().discover(&relations);

    let result = HypothesisInducer::new(2).induce(&primitives);

    assert_eq!(result.len(), 2);

    assert!(result
        .iter()
        .all(|hypothesis| { hypothesis.compression_gain() >= 2 }));
}

#[test]
fn gain_threshold_is_explicit() {
    let inducer = HypothesisInducer::new(4);

    assert_eq!(inducer.minimum_gain(), 4);
}

#[test]
fn empty_primitive_set_produces_no_hypothesis() {
    let inducer = HypothesisInducer::default();

    let result = inducer.induce(&[]);

    assert!(result.is_empty());
}

#[test]
fn hypothesis_identity_excludes_occurrence_locations() {
    let first = hypotheses(&[1, 2, 1, 2, 9]);

    let shifted = hypotheses(&[9, 1, 2, 1, 2]);

    assert_eq!(first, shifted);

    assert_eq!(first[0].signatures(), shifted[0].signatures());
}

#[test]
fn hypothesis_contains_structural_description_only() {
    let result = hypotheses(&[847, 13, 847, 13, 999]);

    let hypothesis = &result[0];

    assert_eq!(
        hypothesis.signatures(),
        &[PrimitiveSignature::new(RelationKind::Equal, 2,)]
    );

    assert_eq!(hypothesis.evidence_count(), 2);
}

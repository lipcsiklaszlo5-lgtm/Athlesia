use athlesia::{
    Encoder, PrimitiveDiscovery, PrimitiveOccurrence, RelationKind, RelationalStructure,
};

fn discover<T>(values: &[T]) -> Vec<athlesia::StructuralPrimitive>
where
    T: Eq + std::hash::Hash,
{
    let encoder = Encoder::new();
    let sequence = encoder.encode(values);
    let relations = RelationalStructure::from_sequence(&sequence);

    PrimitiveDiscovery::default().discover(&relations)
}

#[test]
fn repeated_relations_form_a_structural_primitive() {
    let primitives = discover(&[1, 2, 1, 2, 3]);

    assert_eq!(primitives.len(), 1);

    let primitive = &primitives[0];

    assert_eq!(primitive.kind(), RelationKind::Equal);
    assert_eq!(primitive.span(), 2);
    assert_eq!(primitive.support(), 2);
}

#[test]
fn primitive_occurrences_preserve_evidence_locations() {
    let primitives = discover(&[1, 2, 1, 2, 3]);

    assert_eq!(
        primitives[0].occurrences(),
        &[
            PrimitiveOccurrence::new(0, 2),
            PrimitiveOccurrence::new(1, 3),
        ]
    );
}

#[test]
fn singleton_relation_is_not_promoted_by_default() {
    let primitives = discover(&[1, 2, 1, 3]);

    assert!(primitives.is_empty());
}

#[test]
fn discovery_is_value_invariant() {
    let first = discover(&[1, 2, 1, 2, 3]);

    let second = discover(&[847, 13, 847, 13, 999]);

    assert_eq!(first, second);
}

#[test]
fn discovery_is_deterministic() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[1, 2, 1, 2, 1]);
    let relations = RelationalStructure::from_sequence(&sequence);

    let discovery = PrimitiveDiscovery::default();

    let first = discovery.discover(&relations);
    let second = discovery.discover(&relations);

    assert_eq!(first, second);
}

#[test]
fn different_spans_create_different_primitives() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[1, 1, 1, 1]);
    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::default().discover(&relations);

    assert_eq!(primitives.len(), 2);

    assert_eq!(primitives[0].span(), 1);
    assert_eq!(primitives[0].support(), 3);

    assert_eq!(primitives[1].span(), 2);
    assert_eq!(primitives[1].support(), 2);
}

#[test]
fn unsupported_span_is_rejected() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[1, 1, 1, 1]);
    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::new(3).discover(&relations);

    assert_eq!(primitives.len(), 1);
    assert_eq!(primitives[0].span(), 1);
    assert_eq!(primitives[0].support(), 3);
}

#[test]
fn minimum_support_is_explicit() {
    let discovery = PrimitiveDiscovery::new(4);

    assert_eq!(discovery.minimum_support(), 4);
}

#[test]
fn primitive_identity_is_independent_of_absolute_position() {
    let first = discover(&[1, 2, 1, 2, 9]);

    let second = discover(&[9, 1, 2, 1, 2]);

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);

    assert_eq!(first[0].kind(), second[0].kind());
    assert_eq!(first[0].span(), second[0].span());
    assert_eq!(first[0].support(), second[0].support());

    assert_ne!(first[0].occurrences(), second[0].occurrences());
}

#[test]
fn primitive_contains_structural_evidence_only() {
    let primitives = discover(&[847, 13, 847, 13, 999]);

    let primitive = &primitives[0];

    assert_eq!(primitive.kind(), RelationKind::Equal);
    assert_eq!(primitive.span(), 2);

    for occurrence in primitive.occurrences() {
        assert!(occurrence.left() < occurrence.right());
        assert_eq!(occurrence.span(), primitive.span());
    }
}

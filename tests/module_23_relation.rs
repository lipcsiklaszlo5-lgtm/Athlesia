use athlesia::{Encoder, RelationKind, RelationalStructure};

#[test]
fn equality_relations_are_discovered() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[1, 2, 1, 2, 3]);
    let structure = RelationalStructure::from_sequence(&sequence);

    assert_eq!(structure.relation_count(), 2);

    assert!(structure.has_relation(0, 2, RelationKind::Equal));
    assert!(structure.has_relation(1, 3, RelationKind::Equal));
}

#[test]
fn equality_relations_are_canonicalized() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[1, 2, 1, 2, 3]);
    let structure = RelationalStructure::from_sequence(&sequence);

    assert!(structure.has_relation(2, 0, RelationKind::Equal));
    assert!(structure.has_relation(3, 1, RelationKind::Equal));
}

#[test]
fn unequal_roles_do_not_create_equality_relations() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[1, 2, 3, 4]);
    let structure = RelationalStructure::from_sequence(&sequence);

    assert_eq!(structure.relation_count(), 0);
}

#[test]
fn constant_sequence_creates_all_pairwise_equalities() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[9, 9, 9, 9]);
    let structure = RelationalStructure::from_sequence(&sequence);

    assert_eq!(structure.relation_count(), 6);

    for left in 0..4 {
        for right in (left + 1)..4 {
            assert!(structure.has_relation(left, right, RelationKind::Equal));
        }
    }
}

#[test]
fn relation_structure_preserves_sequence_length() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[7, 8, 7, 8, 99]);
    let structure = RelationalStructure::from_sequence(&sequence);

    assert_eq!(structure.length(), 5);
}

#[test]
fn relation_structure_is_value_invariant() {
    let encoder = Encoder::new();

    let first = encoder.encode(&[1, 2, 1, 2, 3]);
    let second = encoder.encode(&[847, 13, 847, 13, 999]);

    let first_relations = RelationalStructure::from_sequence(&first);

    let second_relations = RelationalStructure::from_sequence(&second);

    assert_eq!(first_relations, second_relations);
}

#[test]
fn relation_structure_is_deterministic() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[10, 20, 10, 30, 20, 10]);

    let first = RelationalStructure::from_sequence(&sequence);

    let second = RelationalStructure::from_sequence(&sequence);

    assert_eq!(first, second);
}

#[test]
fn relation_order_is_stable() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[1, 2, 1, 2, 1]);

    let structure = RelationalStructure::from_sequence(&sequence);

    let expected = [
        (0usize, 2usize),
        (0usize, 4usize),
        (1usize, 3usize),
        (2usize, 4usize),
    ];

    let actual: Vec<(usize, usize)> = structure
        .relations()
        .iter()
        .map(|relation| (relation.left(), relation.right()))
        .collect();

    assert_eq!(actual, expected);
}

#[test]
fn relation_api_exposes_only_structural_information() {
    let encoder = Encoder::new();
    let sequence = encoder.encode(&[847, 13, 847, 13, 999]);

    let structure = RelationalStructure::from_sequence(&sequence);

    assert_eq!(structure.length(), 5);
    assert_eq!(structure.relation_count(), 2);

    for relation in structure.relations() {
        assert_eq!(relation.kind(), RelationKind::Equal);
        assert!(relation.left() < structure.length());
        assert!(relation.right() < structure.length());
        assert!(relation.left() < relation.right());
    }
}

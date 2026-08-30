use athlesia::{Role, StructuralSequence};

#[test]
fn role_identity_is_value_based() {
    assert_eq!(Role::new(7), Role::new(7));
    assert_ne!(Role::new(7), Role::new(8));
}

#[test]
fn structural_sequence_preserves_role_order() {
    let sequence = StructuralSequence::new(vec![
        Role::new(0),
        Role::new(1),
        Role::new(0),
        Role::new(1),
        Role::new(2),
    ]);

    let expected = [
        Role::new(0),
        Role::new(1),
        Role::new(0),
        Role::new(1),
        Role::new(2),
    ];

    assert_eq!(sequence.roles(), &expected);
}

#[test]
fn structural_sequence_supports_exact_position_access() {
    let sequence = StructuralSequence::new(vec![
        Role::new(0),
        Role::new(1),
        Role::new(0),
        Role::new(1),
        Role::new(2),
    ]);

    assert_eq!(sequence.role_at(0), Some(Role::new(0)));
    assert_eq!(sequence.role_at(2), Some(Role::new(0)));
    assert_eq!(sequence.role_at(4), Some(Role::new(2)));
    assert_eq!(sequence.role_at(5), None);
}

#[test]
fn empty_sequence_is_valid_structural_state() {
    let sequence = StructuralSequence::new(Vec::new());

    assert!(sequence.is_empty());
    assert_eq!(sequence.len(), 0);
    assert_eq!(sequence.roles(), &[]);
}

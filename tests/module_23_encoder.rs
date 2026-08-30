use athlesia::{Encoder, Role};

#[test]
fn encoder_is_deterministic() {
    let encoder = Encoder::new();
    let values = [1, 2, 1, 2, 3];

    let first = encoder.encode(&values);
    let second = encoder.encode(&values);

    assert_eq!(first, second);
}

#[test]
fn encoder_is_value_invariant() {
    let encoder = Encoder::new();

    let a = encoder.encode(&[1, 2, 1, 2, 3]);
    let b = encoder.encode(&[10, 20, 10, 20, 30]);
    let c = encoder.encode(&[847, 13, 847, 13, 999]);

    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn encoder_discovers_first_occurrence_roles() {
    let encoder = Encoder::new();

    let structure = encoder.encode(&[50, 80, 50, 90, 80]);

    let expected = [
        Role::new(0),
        Role::new(1),
        Role::new(0),
        Role::new(2),
        Role::new(1),
    ];

    assert_eq!(structure.roles(), &expected);
}

#[test]
fn encoder_handles_all_unique_values() {
    let encoder = Encoder::new();

    let structure = encoder.encode(&[10, 20, 30, 40]);

    let expected = [Role::new(0), Role::new(1), Role::new(2), Role::new(3)];

    assert_eq!(structure.roles(), &expected);
}

#[test]
fn encoder_handles_empty_input() {
    let encoder = Encoder::new();

    let structure = encoder.encode::<i32>(&[]);

    assert!(structure.is_empty());
}

#[test]
fn encoder_does_not_depend_on_numeric_value_magnitude() {
    let encoder = Encoder::new();

    let small = encoder.encode(&[1, 2, 1, 3]);
    let large = encoder.encode(&[999_999_999, -7, 999_999_999, 42]);

    assert_eq!(small, large);
}

#[test]
fn structural_output_contains_only_roles() {
    let encoder = Encoder::new();

    let structure = encoder.encode(&[847, 13, 847, 13, 999]);

    assert_eq!(
        structure.roles(),
        &[
            Role::new(0),
            Role::new(1),
            Role::new(0),
            Role::new(1),
            Role::new(2),
        ]
    );
}

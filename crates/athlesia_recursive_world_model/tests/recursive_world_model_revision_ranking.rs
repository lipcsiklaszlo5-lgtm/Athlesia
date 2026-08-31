use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldCostedRevision, RecursiveWorldMinimalRevision, RecursiveWorldModel,
    RecursiveWorldRevisionRanking, RecursiveWorldRule,
};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
    )
}

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(structural(span)))
}

fn rule(premises: &[usize], conclusions: &[usize]) -> RecursiveWorldRule {
    RecursiveWorldRule::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn costed(
    model: &RecursiveWorldModel,
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
) -> RecursiveWorldCostedRevision {
    let revision = RecursiveWorldMinimalRevision::apply(model, target, replacement).unwrap();

    RecursiveWorldCostedRevision::evaluate(revision)
}

#[test]
fn empty_ranking_is_empty() {
    let ranking = RecursiveWorldRevisionRanking::new(Vec::new());

    assert!(ranking.is_empty());

    assert_eq!(ranking.len(), 0);

    assert!(ranking.best().is_none());
}

#[test]
fn single_revision_is_best() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let revision = costed(&model, target, rule(&[1], &[3]));

    let ranking = RecursiveWorldRevisionRanking::new(vec![revision.clone()]);

    assert_eq!(ranking.best(), Some(&revision,));
}

#[test]
fn lower_total_cost_ranks_first() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let cheaper = costed(&model, target.clone(), rule(&[1], &[3]));

    let expensive = costed(&model, target, rule(&[4, 5], &[6, 7]));

    assert!(cheaper.total_cost() < expensive.total_cost());

    let ranking = RecursiveWorldRevisionRanking::new(vec![expensive, cheaper.clone()]);

    assert_eq!(ranking.best(), Some(&cheaper,));
}

#[test]
fn lower_dependency_impact_breaks_total_cost_tie() {
    let first = rule(&[1], &[2]);

    let first_dependent = rule(&[2], &[9]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), first_dependent, second.clone()]);

    let first_revision = costed(&model, first, rule(&[1], &[3]));

    let second_revision = costed(&model, second, rule(&[5, 7], &[8]));

    assert_eq!(first_revision.total_cost(), 4);

    assert_eq!(second_revision.total_cost(), 4);

    assert_eq!(first_revision.total_cost(), second_revision.total_cost());

    assert_eq!(first_revision.cost().dependency_impact_cost(), 1);

    assert_eq!(second_revision.cost().dependency_impact_cost(), 0);

    assert!(
        second_revision.cost().dependency_impact_cost()
            < first_revision.cost().dependency_impact_cost()
    );

    let ranking = RecursiveWorldRevisionRanking::new(vec![first_revision, second_revision.clone()]);

    assert_eq!(ranking.best(), Some(&second_revision,));
}

#[test]
fn lower_structural_delta_breaks_remaining_tie() {
    let first = rule(&[1], &[2]);

    let first_dependent = rule(&[2], &[9]);

    let second = rule(&[5], &[6]);

    let second_dependent = rule(&[6], &[10]);

    let model = RecursiveWorldModel::new(vec![
        first.clone(),
        first_dependent,
        second.clone(),
        second_dependent,
    ]);

    let compact = costed(&model, first, rule(&[1], &[3]));

    let larger = costed(&model, second, rule(&[5, 7], &[8]));

    assert_eq!(compact.cost().dependency_impact_cost(), 1);

    assert_eq!(larger.cost().dependency_impact_cost(), 1);

    assert!(compact.cost().structural_delta_cost() < larger.cost().structural_delta_cost());

    assert!(compact.total_cost() < larger.total_cost());

    let ranking = RecursiveWorldRevisionRanking::new(vec![larger, compact.clone()]);

    assert_eq!(ranking.best(), Some(&compact,));
}

#[test]
fn equal_cost_revisions_receive_deterministic_identity_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let first_revision = costed(&model, first, rule(&[1], &[3]));

    let second_revision = costed(&model, second, rule(&[5], &[7]));

    assert_eq!(first_revision.total_cost(), second_revision.total_cost());

    assert_eq!(
        first_revision.cost().dependency_impact_cost(),
        second_revision.cost().dependency_impact_cost()
    );

    assert_eq!(
        first_revision.cost().structural_delta_cost(),
        second_revision.cost().structural_delta_cost()
    );

    let ranking =
        RecursiveWorldRevisionRanking::new(vec![second_revision.clone(), first_revision.clone()]);

    let expected = if first_revision.revision().target() < second_revision.revision().target() {
        first_revision
    } else {
        second_revision
    };

    assert_eq!(ranking.best(), Some(&expected,));
}

#[test]
fn duplicate_costed_revisions_are_deduplicated() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let revision = costed(&model, target, rule(&[1], &[3]));

    let ranking = RecursiveWorldRevisionRanking::new(vec![revision.clone(), revision]);

    assert_eq!(ranking.len(), 1);
}

#[test]
fn ranking_preserves_all_distinct_revisions() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let left = costed(&model, first, rule(&[1], &[3]));

    let right = costed(&model, second, rule(&[5], &[7]));

    let ranking = RecursiveWorldRevisionRanking::new(vec![left.clone(), right.clone()]);

    assert_eq!(ranking.len(), 2);

    assert!(ranking.revisions().contains(&left,));

    assert!(ranking.revisions().contains(&right,));
}

#[test]
fn ranking_is_monotonic_by_total_cost() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let first = costed(&model, target.clone(), rule(&[1], &[3]));

    let second = costed(&model, target.clone(), rule(&[1, 4], &[5]));

    let third = costed(&model, target, rule(&[6, 7], &[8, 9]));

    let ranking = RecursiveWorldRevisionRanking::new(vec![third, first, second]);

    for pair in ranking.revisions().windows(2) {
        assert!(pair[0].total_cost() <= pair[1].total_cost());
    }
}

#[test]
fn ranking_is_deterministic_under_input_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let left_revision = costed(&model, first, rule(&[1], &[3]));

    let right_revision = costed(&model, second, rule(&[5], &[7]));

    let left =
        RecursiveWorldRevisionRanking::new(vec![left_revision.clone(), right_revision.clone()]);

    let right = RecursiveWorldRevisionRanking::new(vec![right_revision, left_revision]);

    assert_eq!(left, right);
}

#[test]
fn ranking_preserves_revision_identity() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let source = costed(&model, target, rule(&[1], &[3]));

    let expected = source.revision().clone();

    let ranking = RecursiveWorldRevisionRanking::new(vec![source]);

    assert_eq!(ranking.best().unwrap().revision(), &expected);
}

#[test]
fn ranking_does_not_mutate_source_vector() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let source = vec![
        costed(&model, first, rule(&[1], &[3])),
        costed(&model, second, rule(&[5], &[7])),
    ];

    let before = source.clone();

    let _ = RecursiveWorldRevisionRanking::new(source.clone());

    assert_eq!(source, before);
}

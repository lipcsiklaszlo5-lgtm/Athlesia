use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
    RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    RecursiveWorldRevisionAbstractionCompositionThreshold,
    RecursiveWorldRevisionAbstractionCompositionWitness,
    RecursiveWorldRevisionAbstractionCompositionWitnessSet,
};

use athlesia_recursive_world_model_revision_abstraction_composition_generalization::{
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationSource,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationValidator,
    RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(
        StructuralConcept::with_sequence_length(
            vec![PrimitiveSignature::new(RelationKind::Equal, span)],
            8,
        ),
    ))
}

fn class(members: &[usize]) -> RecursiveWorldRevisionAbstractionClass {
    RecursiveWorldRevisionAbstractionClass::new(members.iter().copied().map(unit).collect())
        .unwrap()
}

fn rule(premises: &[usize], conclusions: &[usize]) -> RecursiveWorldRule {
    RecursiveWorldRule::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn observation(
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionDiscoveryObservation {
    RecursiveWorldRevisionDiscoveryObservation::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn witness(
    from_members: &[usize],
    to_members: &[usize],
    premise_member: usize,
    conclusion_member: usize,
    noise: usize,
) -> RecursiveWorldRevisionAbstractionCompositionWitness {
    RecursiveWorldRevisionAbstractionCompositionWitness::new(
        class(from_members),
        class(to_members),
        observation(
            &[premise_member, 1000 + noise],
            &[conclusion_member, 2000 + noise],
        ),
    )
    .unwrap()
}

fn context(seed: usize) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    let relations = vec![(vec![1, 2], vec![10, 20]), (vec![10, 20], vec![100, 200])];

    let mut witnesses = Vec::new();

    for (edge_index, (from, to)) in relations.into_iter().enumerate() {
        for support_index in 0..2 {
            witnesses.push(witness(
                &from,
                &to,
                from[support_index % from.len()],
                to[support_index % to.len()],
                seed * 10000 + edge_index * 100 + support_index,
            ));
        }
    }

    let composition = RecursiveWorldRevisionAbstractionComposition::compose(
        RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap(),
        RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap(),
    )
    .unwrap();

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(composition).unwrap();

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports)
}

fn projected() -> RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        context(1),
        context(2),
    ])
    .unwrap();

    let generalized = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        source,
        RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    let resolution =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(generalized);

    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        resolution,
        context(10),
    )
    .projected_motifs()
    .first()
    .unwrap()
    .clone()
}

fn application() -> Vec<RecursiveWorldRevisionDiscoveryObservation> {
    vec![observation(&[1, 700], &[100, 800])]
}

#[test]
fn unavailable_discovery_is_validation_unavailable() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target,
        projected(),
        vec![observation(&[900], &[901])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::
            DiscoveryUnavailable
    );

    assert!(result.validation_result().is_none());
}

#[test]
fn noop_discovery_is_validation_unavailable() {
    let target = rule(&[1], &[100]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::
            DiscoveryUnavailable
    );
}

#[test]
fn missing_target_revision_is_rejected() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![rule(&[8], &[88])]),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::Rejected
    );

    assert!(result.is_rejected());

    assert!(result.rejected_hypothesis().is_some());
}

#[test]
fn replacement_collision_is_rejected() {
    let target = rule(&[9], &[99]);

    let replacement = rule(&[1], &[100]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![target.clone(), replacement]),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::Rejected
    );
}

#[test]
fn valid_generalized_composition_revision_is_accepted() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::Accepted
    );

    assert!(result.is_accepted());

    assert!(result.accepted_hypothesis().is_some());
}

#[test]
fn accepted_validation_preserves_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target,
        projected(),
        application(),
    );

    assert_eq!(result.accepted_hypothesis(), result.hypothesis());
}

#[test]
fn rejected_validation_preserves_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![rule(&[8], &[88])]),
        target,
        projected(),
        application(),
    );

    assert_eq!(result.rejected_hypothesis(), result.hypothesis());
}

#[test]
fn validation_preserves_target_and_replacement_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target.clone(),
        projected(),
        application(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.replacement(), Some(&rule(&[1], &[100],),));
}

#[test]
fn validation_preserves_realization_and_projection_provenance() {
    let target = rule(&[9], &[99]);

    let projected = projected();

    let before = projected.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target,
        projected,
        application(),
    );

    assert_eq!(result.projected_motif(), &before);

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );

    assert_eq!(result.support_count(), before.motif().support_count());
}

#[test]
fn validation_preserves_model_and_application_provenance() {
    let target = rule(&[9], &[99]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let model_before = model.clone();

    let application = application();

    let application_before = application.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        model,
        target,
        projected(),
        application,
    );

    assert_eq!(result.model(), &model_before);

    assert_eq!(
        result.application_observations(),
        application_before.as_slice()
    );
}

#[test]
fn generalized_composition_validator_facade_matches_direct_validation() {
    let target = rule(&[9], &[99]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let projected = projected();

    let application = application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidator::validate(
            model.clone(),
            target.clone(),
            projected.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
            model,
            target,
            projected,
            application,
        )
    );
}

#[test]
fn generalized_composition_validation_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let projected = projected();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let model_before = model.clone();

    let projected_before = projected.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        model.clone(),
        target.clone(),
        projected.clone(),
        vec![first.clone(), second.clone()],
    );

    let right = RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
        model.clone(),
        target,
        projected.clone(),
        vec![second, first],
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(projected, projected_before);

    assert!(left.is_accepted());
}

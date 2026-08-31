use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

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
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjector,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationSource,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
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

fn context(
    relations: Vec<(Vec<usize>, Vec<usize>)>,
    seed: usize,
) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
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

fn abc_relations() -> Vec<(Vec<usize>, Vec<usize>)> {
    vec![(vec![1, 2], vec![10, 20]), (vec![10, 20], vec![100, 200])]
}

fn abc_classes() -> Vec<RecursiveWorldRevisionAbstractionClass> {
    vec![class(&[1, 2]), class(&[10, 20]), class(&[100, 200])]
}

fn training_resolution() -> RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution {
    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        context(abc_relations(), 1),
        context(abc_relations(), 2),
    ])
    .unwrap();

    let generalized = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        source,
        RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(generalized)
}

#[test]
fn exact_application_motif_projects() {
    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        training_resolution(),
        context(abc_relations(), 10),
    );

    assert_eq!(
        projection.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus::Projected
    );

    assert_eq!(projection.len(), 1);

    assert!(projection.projected_motif(&abc_classes(),).is_some());
}

#[test]
fn missing_application_motif_does_not_project() {
    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        training_resolution(),
        context(
            vec![
                (vec![300, 301], vec![310, 311]),
                (vec![310, 311], vec![320, 321]),
            ],
            10,
        ),
    );

    assert_eq!(
        projection.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus::
            NoApplicationMatch
    );

    assert!(projection.is_empty());
}

#[test]
fn reversed_application_path_does_not_project() {
    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        training_resolution(),
        context(
            vec![(vec![100, 200], vec![10, 20]), (vec![10, 20], vec![1, 2])],
            10,
        ),
    );

    assert_eq!(
        projection.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus::
            NoApplicationMatch
    );
}

#[test]
fn noncontiguous_application_classes_do_not_project() {
    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        training_resolution(),
        context(
            vec![
                (vec![1, 2], vec![10, 20]),
                (vec![10, 20], vec![50, 60]),
                (vec![50, 60], vec![100, 200]),
            ],
            10,
        ),
    );

    assert!(projection.projected_motif(&abc_classes(),).is_none());
}

#[test]
fn longer_application_path_can_contain_exact_motif_window() {
    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        training_resolution(),
        context(
            vec![
                (vec![500, 600], vec![1, 2]),
                (vec![1, 2], vec![10, 20]),
                (vec![10, 20], vec![100, 200]),
                (vec![100, 200], vec![700, 800]),
            ],
            10,
        ),
    );

    assert!(projection.projected_motif(&abc_classes(),).is_some());
}

#[test]
fn application_match_preserves_matching_selection_identity() {
    let application = context(
        vec![
            (vec![1, 2], vec![10, 20]),
            (vec![10, 20], vec![100, 200]),
            (vec![100, 200], vec![700, 800]),
        ],
        10,
    );

    let before = application.clone();

    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        training_resolution(),
        application,
    );

    let projected = projection.projected_motif(&abc_classes()).unwrap();

    assert!(!projected.matching_selections().is_empty());

    for selection in projected.matching_selections() {
        assert!(before.selections().contains(selection,));
    }
}

#[test]
fn one_selection_counts_once_even_if_checked_through_projection() {
    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        training_resolution(),
        context(abc_relations(), 10),
    );

    let projected = projection.projected_motif(&abc_classes()).unwrap();

    assert_eq!(projected.match_count(), 1);
}

#[test]
fn conflicted_training_motif_is_not_projectable() {
    let relations = vec![
        (vec![1, 2], vec![10, 20]),
        (vec![10, 20], vec![100, 200]),
        (vec![10, 20], vec![300, 400]),
    ];

    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        context(relations.clone(), 1),
        context(relations, 2),
    ])
    .unwrap();

    let generalized = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        source,
        RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    let resolution =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(generalized);

    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        resolution,
        context(abc_relations(), 10),
    );

    assert_eq!(
        projection.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus::
            NoResolvedMotifs
    );

    assert!(projection.is_empty());
}

#[test]
fn projection_preserves_resolution_identity() {
    let resolution = training_resolution();

    let before = resolution.clone();

    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        resolution,
        context(abc_relations(), 10),
    );

    assert_eq!(projection.resolution(), &before);
}

#[test]
fn projection_preserves_application_identity() {
    let application = context(abc_relations(), 10);

    let before = application.clone();

    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        training_resolution(),
        application,
    );

    assert_eq!(projection.application(), &before);
}

#[test]
fn projector_facade_matches_direct_projection() {
    let resolution = training_resolution();

    let application = context(abc_relations(), 10);

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjector::project(
            resolution.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
            resolution,
            application,
        )
    );
}

#[test]
fn motif_projection_is_canonical_deterministic_and_non_mutating() {
    let resolution = training_resolution();

    let application = context(
        vec![
            (vec![500, 600], vec![1, 2]),
            (vec![1, 2], vec![10, 20]),
            (vec![10, 20], vec![100, 200]),
        ],
        10,
    );

    let resolution_before = resolution.clone();

    let application_before = application.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        resolution.clone(),
        application.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        resolution.clone(),
        application.clone(),
    );

    assert_eq!(left, right);

    assert_eq!(resolution, resolution_before);

    assert_eq!(application, application_before);

    assert_eq!(
        left.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus::Projected
    );
}

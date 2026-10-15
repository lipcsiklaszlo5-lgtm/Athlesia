//! Production-boundary test for R3E-C relational transfer through M51.
//!
//! The fixture is deliberately built from retained episodes, then queried only
//! through OnlinePersistentCognitiveState's public prediction/authority APIs.

use super::*;

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).expect("bounded test signal")
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(items: Vec<CognitiveStructure>) -> CognitiveStructure {
    CognitiveStructure::ordered(items).expect("non-empty ordered structure")
}

fn unordered(items: Vec<CognitiveStructure>) -> CognitiveStructure {
    CognitiveStructure::unordered(items).expect("non-empty unordered structure")
}

fn cell(index: u64, value: u64) -> CognitiveStructure {
    ordered(vec![atom(0xC311), atom(index), atom(0), atom(value)])
}

fn state(value: u64) -> athlesia_universal_domain_learning::GroundedStateSnapshot {
    let c0 = cell(0, value);
    let c1 = cell(1, value);
    let group = unordered(vec![c0.clone(), c1.clone()]);
    athlesia_universal_domain_learning::GroundedStateSnapshot::new(vec![
        c0,
        c1,
        group.clone(),
        unordered(vec![group]),
    ])
    .expect("grounded fixture state")
}

fn episode(
    before: u64,
    after: u64,
    action: u64,
) -> athlesia_universal_domain_learning::GroundedTransformationEpisode {
    athlesia_universal_domain_learning::GroundedTransformationEpisode::new(
        state(before),
        state(after),
        atom(action),
    )
}

fn retained_evidence() -> Vec<athlesia_universal_domain_learning::GroundedTransformationEpisode> {
    let mut episodes = Vec::new();
    for before in [1, 2, 3] {
        for _ in 0..3 {
            episodes.push(episode(before, before + 1, 1));
            episodes.push(episode(before, before, 2));
        }
    }
    episodes
}

fn schema_policy() -> athlesia_universal_domain_learning::TransitionSchemaPolicy {
    athlesia_universal_domain_learning::TransitionSchemaPolicy::new(
        1,
        signal(1000),
        signal(1),
        256,
        64,
    )
    .expect("schema policy")
}

fn model_policy() -> athlesia_universal_domain_learning::GroundedExecutableWorldModelPolicy {
    athlesia_universal_domain_learning::GroundedExecutableWorldModelPolicy::new(64)
        .expect("model policy")
}

fn owner_with_retained_evidence() -> OnlinePersistentCognitiveState {
    let mut owner = OnlinePersistentCognitiveState::new();
    let episodes = retained_evidence();
    owner.transition_schema_learning.episodes = episodes.clone();
    owner.transition_schema_learning.event_provenance = episodes
        .iter()
        .enumerate()
        .map(|(index, _)| {
            RetainedTransitionEventProvenance::new(
                (index + 1) as u64,
                (index + 1_000) as u64,
                (index + 2_000) as u64,
            )
        })
        .collect();
    owner
}

#[test]
fn m51_public_prediction_uses_relational_transfer_on_unseen_source() {
    let owner = owner_with_retained_evidence();
    let source = state(5);
    let action = atom(1);

    let prediction = owner
        .current_structural_prediction(&source, &action, schema_policy(), model_policy())
        .expect("three independent sources should produce a structural candidate");

    assert_eq!(
        prediction.origin(),
        athlesia_universal_domain_learning::GroundedStructuralPredictionOrigin::RelationalTransfer,
        "M51 must prefer the relational hypothesis over a stale literal schema"
    );
    assert!(prediction.predicted());
    assert_eq!(prediction.additions().len(), 4);
    assert_eq!(prediction.removals().len(), 4);

    let authorized = owner
        .current_empirically_authorized_structural_prediction(
            &source,
            &action,
            schema_policy(),
            model_policy(),
        )
        .expect("the same retained evidence must re-derive authority");

    assert_eq!(
        authorized.prediction().origin(),
        athlesia_universal_domain_learning::GroundedStructuralPredictionOrigin::RelationalTransfer
    );
    assert_eq!(authorized.prediction(), &prediction);
    assert_eq!(authorized.evidence_confidence(), signal(1000));
    assert!(authorized.controllability().value() > 0);
}

#[test]
fn m51_relational_transfer_abstains_when_structural_evidence_is_confounded() {
    let mut owner = OnlinePersistentCognitiveState::new();
    let mut episodes = retained_evidence();
    for _ in 0..3 {
        episodes.push(episode(1, 3, 1));
    }
    owner.transition_schema_learning.episodes = episodes.clone();
    owner.transition_schema_learning.event_provenance = episodes
        .iter()
        .enumerate()
        .map(|(index, _)| {
            RetainedTransitionEventProvenance::new(
                (index + 1) as u64,
                (index + 1_000) as u64,
                (index + 2_000) as u64,
            )
        })
        .collect();

    let prediction = owner.current_structural_prediction(
        &state(5),
        &atom(1),
        schema_policy(),
        model_policy(),
    );
    assert!(
        prediction.as_ref().map_or(true, |candidate| {
            candidate.origin()
                != athlesia_universal_domain_learning::GroundedStructuralPredictionOrigin::
                    RelationalTransfer
        }),
        "conflicting role effects must never be labelled as relational transfer"
    );
}

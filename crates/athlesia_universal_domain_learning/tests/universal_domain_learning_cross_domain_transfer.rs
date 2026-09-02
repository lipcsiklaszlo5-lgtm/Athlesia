use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    CalibratedRuleConfidence, CausalContrastInduction, CausalContrastPolicy,
    CausalContrastThresholds, ContextualTransitionEvidenceThresholds,
    ContextualTransitionRuleInduction, ContextualTransitionRulePolicy, CrossContextGeneralization,
    CrossContextGeneralizationPolicy, CrossContextGeneralizationThresholds, CrossDomainTransfer,
    CrossDomainTransferMap, CrossDomainTransferPolicy, CrossDomainTransferThresholds,
    GroundedInterventionalCausalHypothesis, GroundedStateSnapshot, GroundedTransferCorrespondence,
    GroundedTransformationEpisode, InterventionalCausalThresholds, InterventionalCausalValidation,
    InterventionalCausalValidationPolicy, InterventionalTransformationEpisode,
    RuleConfidenceCalibration, RuleConfidenceCalibrationPolicy, TransitionEffectKind,
    UniversalCrossDomainTransfer,
};

#[derive(Clone, Copy)]
struct TransferBounds {
    max_sources: usize,
    max_evaluations: usize,
    max_transferred: usize,
    full_support: u64,
}

#[derive(Clone, Copy)]
struct TransferEvidence {
    matched_states: usize,
    target_interventions: u64,
    contrast_interventions: u64,
    lift: u16,
    confidence: u16,
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn snapshot(facts: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts.iter().copied().map(atom).collect()).unwrap()
}

fn transition(
    before: &[u64],
    after: &[u64],
    transformation: CognitiveStructure,
) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(snapshot(before), snapshot(after), transformation)
}

fn controlled(
    before: &[u64],
    after: &[u64],
    transformation: CognitiveStructure,
) -> InterventionalTransformationEpisode {
    InterventionalTransformationEpisode::controlled(transition(before, after, transformation))
}

fn observed(
    before: &[u64],
    after: &[u64],
    transformation: CognitiveStructure,
) -> InterventionalTransformationEpisode {
    InterventionalTransformationEpisode::observed(transition(before, after, transformation))
}

fn transfer_policy(
    bounds: TransferBounds,
    evidence: TransferEvidence,
) -> CrossDomainTransferPolicy {
    let thresholds = CrossDomainTransferThresholds::new(
        evidence.matched_states,
        evidence.target_interventions,
        evidence.contrast_interventions,
        signal(evidence.lift),
        signal(evidence.confidence),
    )
    .unwrap();

    CrossDomainTransferPolicy::new(
        bounds.max_sources,
        bounds.max_evaluations,
        bounds.max_transferred,
        bounds.full_support,
        thresholds,
    )
    .unwrap()
}

fn default_transfer_policy() -> CrossDomainTransferPolicy {
    transfer_policy(
        TransferBounds {
            max_sources: 64,
            max_evaluations: 64,
            max_transferred: 32,
            full_support: 4,
        },
        TransferEvidence {
            matched_states: 2,
            target_interventions: 2,
            contrast_interventions: 2,
            lift: 400,
            confidence: 100,
        },
    )
}

fn source_training(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
    vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[1, 6, 9], &[1, 6, 9], transformation.clone()),
        transition(&[1, 6, 9], &[1, 6, 9], transformation),
    ]
}

fn calibrated_seed(transformation: CognitiveStructure) -> CalibratedRuleConfidence {
    let training = source_training(transformation.clone());

    let contextual_thresholds =
        ContextualTransitionEvidenceThresholds::new(2, signal(900), signal(300), signal(300))
            .unwrap();

    let contextual_policy =
        ContextualTransitionRulePolicy::new(2, 256, 8192, 256, contextual_thresholds).unwrap();

    let contextual = ContextualTransitionRuleInduction::induce(&training, &[], contextual_policy);

    let generalization_thresholds =
        CrossContextGeneralizationThresholds::new(2, 2, signal(600), signal(200)).unwrap();

    let generalization_policy =
        CrossContextGeneralizationPolicy::new(256, 2, 256, 128, generalization_thresholds).unwrap();

    let generalizations = CrossContextGeneralization::generalize(
        &training,
        contextual.selected(),
        generalization_policy,
    );

    let calibration_policy =
        RuleConfidenceCalibrationPolicy::new(2, 4, signal(1), 128, 128, 64).unwrap();

    RuleConfidenceCalibration::calibrate(
        &training,
        generalizations.selected(),
        &[],
        calibration_policy,
    )
    .selected()
    .iter()
    .find(|seed| {
        seed.transformation() == &transformation
            && seed.effect_kind() == TransitionEffectKind::Added
            && seed.effect_fact() == &atom(5)
    })
    .unwrap()
    .clone()
}

fn contrast_observations(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
) -> Vec<GroundedTransformationEpisode> {
    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9], [1, 8, 9]];

    let mut episodes = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(5);

        episodes.push(transition(&state, &target_after, target.clone()));

        episodes.push(transition(&state, &state, contrast.clone()));
    }

    episodes
}

fn source_interventional_evidence(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
) -> Vec<InterventionalTransformationEpisode> {
    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9], [1, 8, 9]];

    let mut evidence = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(5);

        evidence.push(controlled(&state, &target_after, target.clone()));

        evidence.push(controlled(&state, &state, contrast.clone()));
    }

    evidence
}

fn source_hypothesis(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
) -> GroundedInterventionalCausalHypothesis {
    let calibrated = calibrated_seed(target.clone());

    let observations = contrast_observations(target.clone(), contrast.clone());

    let contrast_thresholds =
        CausalContrastThresholds::new(2, 2, 2, signal(400), signal(100)).unwrap();

    let contrast_policy = CausalContrastPolicy::new(16, 16, 32, 16, contrast_thresholds).unwrap();

    let causal_contrasts = CausalContrastInduction::induce(
        &observations,
        std::slice::from_ref(&calibrated),
        contrast_policy,
    );

    let intervention_thresholds =
        InterventionalCausalThresholds::new(2, 2, 2, signal(400), signal(100)).unwrap();

    let intervention_policy =
        InterventionalCausalValidationPolicy::new(16, 16, 16, 4, intervention_thresholds).unwrap();

    let evidence = source_interventional_evidence(target, contrast);

    InterventionalCausalValidation::validate(
        &evidence,
        causal_contrasts.selected(),
        intervention_policy,
    )
    .selected()[0]
        .clone()
}

fn correspondence(
    source: CognitiveStructure,
    target: CognitiveStructure,
) -> GroundedTransferCorrespondence {
    GroundedTransferCorrespondence::new(source, target)
}

fn standard_map() -> CrossDomainTransferMap {
    CrossDomainTransferMap::new(
        atom(9000),
        atom(9001),
        vec![
            correspondence(atom(100), atom(1100)),
            correspondence(atom(200), atom(1200)),
            correspondence(atom(1), atom(11)),
            correspondence(atom(5), atom(15)),
        ],
    )
    .unwrap()
}

fn target_evidence(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
    effect_fact: u64,
) -> Vec<InterventionalTransformationEpisode> {
    let states = [[11, 12, 19], [11, 13, 19], [11, 14, 19], [11, 18, 19]];

    let mut evidence = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(effect_fact);

        evidence.push(controlled(&state, &target_after, target.clone()));

        evidence.push(controlled(&state, &state, contrast.clone()));
    }

    evidence
}

fn has_transfer(
    result: &athlesia_universal_domain_learning::CrossDomainTransferResult,
    target: &CognitiveStructure,
    contrast: &CognitiveStructure,
    effect: &CognitiveStructure,
) -> bool {
    result.selected().iter().any(|hypothesis| {
        hypothesis.target_transformation() == target
            && hypothesis.target_contrast_transformation() == contrast
            && hypothesis.target_effect_fact() == effect
    })
}

#[test]
fn transfer_map_requires_distinct_domains_nonempty_and_bijective_correspondence() {
    assert_eq!(
        CrossDomainTransferMap::new(
            atom(9000,),
            atom(9000,),
            vec![correspondence(atom(1,), atom(11,),),],
        ),
        None
    );

    assert_eq!(
        CrossDomainTransferMap::new(atom(9000,), atom(9001,), Vec::new(),),
        None
    );

    assert_eq!(
        CrossDomainTransferMap::new(
            atom(9000,),
            atom(9001,),
            vec![
                correspondence(atom(1,), atom(11,),),
                correspondence(atom(1,), atom(12,),),
            ],
        ),
        None
    );

    assert_eq!(
        CrossDomainTransferMap::new(
            atom(9000,),
            atom(9001,),
            vec![
                correspondence(atom(1,), atom(11,),),
                correspondence(atom(2,), atom(11,),),
            ],
        ),
        None
    );

    assert!(standard_map().correspondence_count() >= 4);
}

#[test]
fn exact_correspondence_identity_is_canonical_and_reordered_structures_remain_distinct() {
    let source = ordered(&[10, 20]);

    let reordered = ordered(&[20, 10]);

    let target = ordered(&[110, 120]);

    let map = CrossDomainTransferMap::new(
        atom(9000),
        atom(9001),
        vec![
            correspondence(source.clone(), target.clone()),
            correspondence(source.clone(), target.clone()),
        ],
    )
    .unwrap();

    assert_eq!(map.correspondence_count(), 1);

    assert_eq!(map.translate(&source,), Some(&target));

    assert_eq!(map.translate(&reordered,), None);

    assert_ne!(source, reordered);
}

#[test]
fn incomplete_correspondence_cannot_create_transfer_candidate() {
    let source = source_hypothesis(atom(100), atom(200));

    let incomplete = CrossDomainTransferMap::new(
        atom(9000),
        atom(9001),
        vec![
            correspondence(atom(100), atom(1100)),
            correspondence(atom(200), atom(1200)),
            correspondence(atom(1), atom(11)),
        ],
    )
    .unwrap();

    let evidence = target_evidence(atom(1100), atom(1200), 15);

    let result = CrossDomainTransfer::transfer(
        &evidence,
        std::slice::from_ref(&source),
        &incomplete,
        default_transfer_policy(),
    );

    assert_eq!(result.rejected_incomplete_mapping(), 1);

    assert_eq!(result.evaluated_candidate_count(), 0);

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn passive_target_evidence_cannot_validate_cross_domain_transfer() {
    let source = source_hypothesis(atom(100), atom(200));

    let map = standard_map();

    let controlled_evidence = target_evidence(atom(1100), atom(1200), 15);

    let passive = controlled_evidence
        .into_iter()
        .map(|item| InterventionalTransformationEpisode::observed(item.episode().clone()))
        .collect::<Vec<_>>();

    let result = CrossDomainTransfer::transfer(
        &passive,
        std::slice::from_ref(&source),
        &map,
        default_transfer_policy(),
    );

    assert_eq!(result.selected_count(), 0);

    assert_eq!(result.rejected_without_matched_target_interventions(), 1);
}

#[test]
fn complete_mapping_plus_target_interventions_transfers_validated_relation() {
    let source = source_hypothesis(atom(100), atom(200));

    let map = standard_map();

    let evidence = target_evidence(atom(1100), atom(1200), 15);

    let result = CrossDomainTransfer::transfer(
        &evidence,
        std::slice::from_ref(&source),
        &map,
        default_transfer_policy(),
    );

    assert!(has_transfer(
        &result,
        &atom(1100,),
        &atom(1200,),
        &atom(15,),
    ));

    let hypothesis = &result.selected()[0];

    assert_eq!(hypothesis.source_domain(), &atom(9000,));

    assert_eq!(hypothesis.target_domain(), &atom(9001,));

    assert_eq!(hypothesis.matched_target_state_count(), 4);

    assert_eq!(hypothesis.target_effect_rate().value(), 1000);

    assert_eq!(hypothesis.target_contrast_effect_rate().value(), 0);

    assert_eq!(hypothesis.target_interventional_lift().value(), 1000);

    assert_eq!(hypothesis.target_support_adequacy().value(), 1000);

    assert_eq!(
        hypothesis.transfer_confidence(),
        hypothesis.source_validated_confidence()
    );
}

#[test]
fn target_domain_common_effect_rejects_transfer_even_with_strong_source_causality() {
    let source = source_hypothesis(atom(100), atom(200));

    let map = standard_map();

    let states = [[11, 12, 19], [11, 13, 19]];

    let mut evidence = Vec::new();

    for state in states {
        let mut after = state.to_vec();

        after.push(15);

        evidence.push(controlled(&state, &after, atom(1100)));

        evidence.push(controlled(&state, &after, atom(1200)));
    }

    let result = CrossDomainTransfer::transfer(
        &evidence,
        std::slice::from_ref(&source),
        &map,
        transfer_policy(
            TransferBounds {
                max_sources: 16,
                max_evaluations: 16,
                max_transferred: 16,
                full_support: 2,
            },
            TransferEvidence {
                matched_states: 2,
                target_interventions: 2,
                contrast_interventions: 2,
                lift: 1,
                confidence: 1,
            },
        ),
    );

    assert_eq!(result.selected_count(), 0);

    assert_eq!(result.rejected_below_transfer_threshold(), 1);
}

#[test]
fn target_failures_and_control_successes_remain_explicit_transfer_counterevidence() {
    let source = source_hypothesis(atom(100), atom(200));

    let map = standard_map();

    let states = [[11, 12, 19], [11, 13, 19], [11, 14, 19], [11, 18, 19]];

    let mut evidence = Vec::new();

    for (index, state) in states.into_iter().enumerate() {
        let target_after = if index == 3 {
            state.to_vec()
        } else {
            let mut after = state.to_vec();

            after.push(15);

            after
        };

        let contrast_after = if index == 0 {
            let mut after = state.to_vec();

            after.push(15);

            after
        } else {
            state.to_vec()
        };

        evidence.push(controlled(&state, &target_after, atom(1100)));

        evidence.push(controlled(&state, &contrast_after, atom(1200)));
    }

    let result = CrossDomainTransfer::transfer(
        &evidence,
        std::slice::from_ref(&source),
        &map,
        transfer_policy(
            TransferBounds {
                max_sources: 16,
                max_evaluations: 16,
                max_transferred: 16,
                full_support: 4,
            },
            TransferEvidence {
                matched_states: 4,
                target_interventions: 4,
                contrast_interventions: 4,
                lift: 400,
                confidence: 100,
            },
        ),
    );

    let hypothesis = &result.selected()[0];

    assert_eq!(hypothesis.target_intervention_success_count(), 3);

    assert_eq!(hypothesis.target_intervention_failure_count(), 1);

    assert_eq!(hypothesis.contrast_intervention_success_count(), 1);

    assert_eq!(hypothesis.contrast_intervention_failure_count(), 3);

    assert_eq!(hypothesis.target_effect_rate().value(), 750);

    assert_eq!(hypothesis.target_contrast_effect_rate().value(), 250);

    assert_eq!(hypothesis.target_interventional_lift().value(), 500);
}

#[test]
fn small_target_sample_is_discounted_and_source_confidence_caps_transfer_confidence() {
    let source = source_hypothesis(atom(100), atom(200));

    let map = standard_map();

    let evidence = vec![
        controlled(&[11, 12, 19], &[11, 12, 15, 19], atom(1100)),
        controlled(&[11, 12, 19], &[11, 12, 19], atom(1200)),
    ];

    let result = CrossDomainTransfer::transfer(
        &evidence,
        std::slice::from_ref(&source),
        &map,
        transfer_policy(
            TransferBounds {
                max_sources: 16,
                max_evaluations: 16,
                max_transferred: 16,
                full_support: 4,
            },
            TransferEvidence {
                matched_states: 1,
                target_interventions: 1,
                contrast_interventions: 1,
                lift: 1,
                confidence: 1,
            },
        ),
    );

    let hypothesis = &result.selected()[0];

    assert_eq!(hypothesis.balanced_target_support(), 1);

    assert_eq!(hypothesis.target_support_adequacy().value(), 250);

    assert_eq!(hypothesis.target_evidence_confidence().value(), 250);

    let expected = (u32::from(source.validated_causal_confidence().value()) * 250) / 1000;

    assert_eq!(
        u32::from(hypothesis.transfer_confidence().value(),),
        expected
    );

    assert!(
        hypothesis.transfer_confidence().value() <= source.validated_causal_confidence().value()
    );
}

#[test]
fn passive_target_evidence_is_retained_without_inflating_transfer_score() {
    let source = source_hypothesis(atom(100), atom(200));

    let map = standard_map();

    let base_evidence = target_evidence(atom(1100), atom(1200), 15);

    let base = CrossDomainTransfer::transfer(
        &base_evidence,
        std::slice::from_ref(&source),
        &map,
        default_transfer_policy(),
    );

    let mut enriched = base_evidence.clone();

    enriched.push(observed(&[11, 21, 19], &[11, 15, 21, 19], atom(1100)));

    enriched.push(observed(&[11, 22, 19], &[11, 22, 19], atom(1200)));

    enriched.push(observed(&[11, 23, 19], &[11, 23, 19], atom(1100)));

    enriched.push(observed(&[11, 24, 19], &[11, 15, 24, 19], atom(1200)));

    let with_passive = CrossDomainTransfer::transfer(
        &enriched,
        std::slice::from_ref(&source),
        &map,
        default_transfer_policy(),
    );

    let first = &base.selected()[0];

    let second = &with_passive.selected()[0];

    assert_eq!(first.transfer_confidence(), second.transfer_confidence());

    assert_eq!(
        first.target_interventional_lift(),
        second.target_interventional_lift()
    );

    assert_eq!(second.passive_corroborating_count(), 2);

    assert_eq!(second.passive_counterevidence_count(), 2);
}

#[test]
fn exact_source_and_target_structure_identities_survive_transfer_without_collapsing_reordered_forms(
) {
    let source_target = ordered(&[100, 101]);

    let source_contrast = ordered(&[200, 201]);

    let target_target = ordered(&[1100, 1101]);

    let target_contrast = ordered(&[1200, 1201]);

    let source = source_hypothesis(source_target.clone(), source_contrast.clone());

    let map = CrossDomainTransferMap::new(
        atom(9000),
        atom(9001),
        vec![
            correspondence(source_target.clone(), target_target.clone()),
            correspondence(source_contrast.clone(), target_contrast.clone()),
            correspondence(atom(1), atom(11)),
            correspondence(atom(5), atom(15)),
        ],
    )
    .unwrap();

    let evidence = target_evidence(target_target.clone(), target_contrast.clone(), 15);

    let result = CrossDomainTransfer::transfer(
        &evidence,
        std::slice::from_ref(&source),
        &map,
        default_transfer_policy(),
    );

    assert!(has_transfer(
        &result,
        &target_target,
        &target_contrast,
        &atom(15,),
    ));

    assert_ne!(target_target, ordered(&[1101, 1100,],));
}

#[test]
fn hard_source_evaluation_and_final_frontiers_are_deterministic_and_input_order_invariant() {
    let source_a = source_hypothesis(atom(100), atom(200));

    let source_b = source_hypothesis(atom(300), atom(400));

    let map = CrossDomainTransferMap::new(
        atom(9000),
        atom(9001),
        vec![
            correspondence(atom(100), atom(1100)),
            correspondence(atom(200), atom(1200)),
            correspondence(atom(300), atom(1300)),
            correspondence(atom(400), atom(1400)),
            correspondence(atom(1), atom(11)),
            correspondence(atom(5), atom(15)),
        ],
    )
    .unwrap();

    let mut evidence = target_evidence(atom(1100), atom(1200), 15);

    evidence.extend(target_evidence(atom(1300), atom(1400), 15));

    let sources = vec![source_a, source_b];

    let source_limited = CrossDomainTransfer::transfer(
        &evidence,
        &sources,
        &map,
        transfer_policy(
            TransferBounds {
                max_sources: 1,
                max_evaluations: 1,
                max_transferred: 1,
                full_support: 4,
            },
            TransferEvidence {
                matched_states: 2,
                target_interventions: 2,
                contrast_interventions: 2,
                lift: 400,
                confidence: 100,
            },
        ),
    );

    assert_eq!(source_limited.considered_source_hypothesis_count(), 1);

    assert!(source_limited.source_frontier_truncated());

    let evaluation_limited = CrossDomainTransfer::transfer(
        &evidence,
        &sources,
        &map,
        transfer_policy(
            TransferBounds {
                max_sources: 2,
                max_evaluations: 1,
                max_transferred: 1,
                full_support: 4,
            },
            TransferEvidence {
                matched_states: 2,
                target_interventions: 2,
                contrast_interventions: 2,
                lift: 400,
                confidence: 100,
            },
        ),
    );

    assert_eq!(evaluation_limited.evaluated_candidate_count(), 1);

    assert!(evaluation_limited.evaluation_truncated());

    let final_policy = transfer_policy(
        TransferBounds {
            max_sources: 2,
            max_evaluations: 2,
            max_transferred: 1,
            full_support: 4,
        },
        TransferEvidence {
            matched_states: 2,
            target_interventions: 2,
            contrast_interventions: 2,
            lift: 400,
            confidence: 100,
        },
    );

    let first = CrossDomainTransfer::transfer(&evidence, &sources, &map, final_policy);

    let mut reversed_evidence = evidence.clone();

    reversed_evidence.reverse();

    let mut reversed_sources = sources.clone();

    reversed_sources.reverse();

    let second =
        CrossDomainTransfer::transfer(&reversed_evidence, &reversed_sources, &map, final_policy);

    assert_eq!(first, second);

    assert_eq!(first.selected_count(), 1);

    assert_eq!(first.admitted_before_frontier(), 2);
}

#[test]
fn cross_domain_transfer_is_deterministic_non_mutating_and_facade_equivalent() {
    let source = source_hypothesis(atom(100), atom(200));

    let sources = vec![source];

    let map = standard_map();

    let evidence = target_evidence(atom(1100), atom(1200), 15);

    let evidence_before = evidence.clone();

    let sources_before = sources.clone();

    let map_before = map.clone();

    let policy = default_transfer_policy();

    let direct = CrossDomainTransfer::transfer(&evidence, &sources, &map, policy);

    let facade = UniversalCrossDomainTransfer::evaluate(&evidence, &sources, &map, policy);

    let repeated = UniversalCrossDomainTransfer::evaluate(&evidence, &sources, &map, policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(evidence, evidence_before);

    assert_eq!(sources, sources_before);

    assert_eq!(map, map_before);

    assert_eq!(facade.input_source_hypothesis_count(), sources.len());

    assert!(facade.evaluated_candidate_count() <= facade.considered_source_hypothesis_count());
}

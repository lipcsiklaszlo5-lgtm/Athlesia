use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    CalibratedRuleConfidence, CausalContrastInduction, CausalContrastPolicy,
    CausalContrastThresholds, CompressedDomainModel, ContextPremiseSet,
    ContextualTransitionEvidenceThresholds, ContextualTransitionRuleInduction,
    ContextualTransitionRulePolicy, CrossContextGeneralization, CrossContextGeneralizationPolicy,
    CrossContextGeneralizationThresholds, CrossDomainTransfer, CrossDomainTransferMap,
    CrossDomainTransferPolicy, CrossDomainTransferThresholds, DomainModelCompression,
    DomainModelCompressionPolicy, GroundedInterventionalCausalHypothesis, GroundedStateSnapshot,
    GroundedTransferCorrespondence, GroundedTransformationEpisode, IntegratedDomainModel,
    IntegratedDomainModelPolicy, IntegratedDomainRelationAuthority, InterventionalCausalThresholds,
    InterventionalCausalValidation, InterventionalCausalValidationPolicy,
    InterventionalTransformationEpisode, RuleConfidenceCalibration,
    RuleConfidenceCalibrationPolicy, TransitionEffectKind, UniversalIntegratedDomainModel,
};

#[derive(Clone, Copy)]
enum EvidenceMode {
    Clean,
    Counterevidence,
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

fn training_history(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
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
    let training = training_history(transformation.clone());

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

fn intervention_evidence(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
    mode: EvidenceMode,
) -> Vec<InterventionalTransformationEpisode> {
    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9], [1, 8, 9]];

    let mut evidence = Vec::new();

    for (index, state) in states.into_iter().enumerate() {
        let target_occurs = matches!(mode, EvidenceMode::Clean) || index != 3;

        let contrast_occurs = matches!(mode, EvidenceMode::Counterevidence) && index == 0;

        let target_after = if target_occurs {
            let mut after = state.to_vec();

            after.push(5);

            after
        } else {
            state.to_vec()
        };

        let contrast_after = if contrast_occurs {
            let mut after = state.to_vec();

            after.push(5);

            after
        } else {
            state.to_vec()
        };

        evidence.push(controlled(&state, &target_after, target.clone()));

        evidence.push(controlled(&state, &contrast_after, contrast.clone()));
    }

    evidence
}

fn local_hypothesis(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
    mode: EvidenceMode,
) -> GroundedInterventionalCausalHypothesis {
    let calibrated = calibrated_seed(target.clone());

    let observations = contrast_observations(target.clone(), contrast.clone());

    let contrast_thresholds =
        CausalContrastThresholds::new(2, 2, 2, signal(400), signal(100)).unwrap();

    let contrast_policy = CausalContrastPolicy::new(16, 16, 32, 16, contrast_thresholds).unwrap();

    let contrasts = CausalContrastInduction::induce(
        &observations,
        std::slice::from_ref(&calibrated),
        contrast_policy,
    );

    let validation_thresholds =
        InterventionalCausalThresholds::new(2, 2, 2, signal(300), signal(1)).unwrap();

    let validation_policy =
        InterventionalCausalValidationPolicy::new(16, 16, 16, 4, validation_thresholds).unwrap();

    let evidence = intervention_evidence(target, contrast, mode);

    InterventionalCausalValidation::validate(&evidence, contrasts.selected(), validation_policy)
        .selected()[0]
        .clone()
}

fn correspondence(
    source: CognitiveStructure,
    target: CognitiveStructure,
) -> GroundedTransferCorrespondence {
    GroundedTransferCorrespondence::new(source, target)
}

fn transferred_model(
    source_domain: CognitiveStructure,
    target_domain: CognitiveStructure,
    source_transformation: CognitiveStructure,
    source_contrast: CognitiveStructure,
    target_transformation: CognitiveStructure,
    target_contrast: CognitiveStructure,
    mode: EvidenceMode,
) -> CompressedDomainModel {
    let source = local_hypothesis(
        source_transformation.clone(),
        source_contrast.clone(),
        EvidenceMode::Clean,
    );

    let transfer_map = CrossDomainTransferMap::new(
        source_domain,
        target_domain,
        vec![
            correspondence(source_transformation, target_transformation.clone()),
            correspondence(source_contrast, target_contrast.clone()),
            correspondence(atom(1), atom(1)),
            correspondence(atom(5), atom(5)),
        ],
    )
    .unwrap();

    let evidence = intervention_evidence(target_transformation, target_contrast, mode);

    let transfer_thresholds =
        CrossDomainTransferThresholds::new(1, 1, 1, signal(300), signal(1)).unwrap();

    let transfer_policy =
        CrossDomainTransferPolicy::new(16, 16, 16, 4, transfer_thresholds).unwrap();

    let transfer = CrossDomainTransfer::transfer(
        &evidence,
        std::slice::from_ref(&source),
        &transfer_map,
        transfer_policy,
    );

    let compression_policy = DomainModelCompressionPolicy::new(16, 16, 16).unwrap();

    let compression = DomainModelCompression::compress(transfer.selected(), compression_policy);

    compression.selected()[0].clone()
}

fn policy(
    max_local: usize,
    max_transferred: usize,
    max_relations: usize,
) -> IntegratedDomainModelPolicy {
    IntegratedDomainModelPolicy::new(max_local, max_transferred, max_relations).unwrap()
}

fn exact_context() -> ContextPremiseSet {
    ContextPremiseSet::new(vec![atom(1)]).unwrap()
}

#[test]
fn integrated_domain_model_policy_requires_positive_hard_bounds() {
    assert_eq!(IntegratedDomainModelPolicy::new(0, 10, 10,), None);

    assert_eq!(IntegratedDomainModelPolicy::new(10, 0, 10,), None);

    assert_eq!(IntegratedDomainModelPolicy::new(10, 10, 0,), None);

    assert!(IntegratedDomainModelPolicy::new(10, 10, 10,).is_some());
}

#[test]
fn local_interventional_hypothesis_is_normalized_without_losing_evidence() {
    let domain = atom(9001);

    let local = local_hypothesis(atom(1100), atom(1200), EvidenceMode::Clean);

    let result = IntegratedDomainModel::build(
        &domain,
        std::slice::from_ref(&local),
        &[],
        policy(16, 16, 16),
    );

    assert_eq!(result.relation_count(), 1);

    let relation = &result.relations()[0];

    assert_eq!(relation.domain(), &domain);

    assert_eq!(
        relation.authority(),
        IntegratedDomainRelationAuthority::LocalInterventional
    );

    assert_eq!(
        relation.confidence_ceiling(),
        local.validated_causal_confidence()
    );

    assert_eq!(
        relation.confidence_floor(),
        local.validated_causal_confidence()
    );

    assert_eq!(
        relation.target_success_count(),
        local.target_intervention_success_count()
    );

    assert_eq!(
        relation.contrast_success_count(),
        local.contrast_intervention_success_count()
    );

    assert_eq!(relation.provenance_count(), 0);
}

#[test]
fn compressed_transfer_model_is_normalized_with_provenance_and_confidence_range() {
    let domain = atom(9001);

    let transferred = transferred_model(
        atom(9000),
        domain.clone(),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Clean,
    );

    let result = IntegratedDomainModel::build(
        &domain,
        &[],
        std::slice::from_ref(&transferred),
        policy(16, 16, 16),
    );

    let relation = &result.relations()[0];

    assert_eq!(
        relation.authority(),
        IntegratedDomainRelationAuthority::TransferredCompressed
    );

    assert_eq!(
        relation.confidence_ceiling(),
        transferred.strongest_transfer_confidence()
    );

    assert_eq!(
        relation.confidence_floor(),
        transferred.weakest_transfer_confidence()
    );

    assert_eq!(relation.provenance_count(), transferred.provenance_count());

    assert_eq!(relation.source_member_count(), transferred.member_count());
}

#[test]
fn transferred_model_from_another_target_domain_is_rejected() {
    let active_domain = atom(9001);

    let transferred = transferred_model(
        atom(9000),
        atom(9002),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Clean,
    );

    let result = IntegratedDomainModel::build(
        &active_domain,
        &[],
        std::slice::from_ref(&transferred),
        policy(16, 16, 16),
    );

    assert_eq!(result.input_transferred_model_count(), 1);

    assert_eq!(result.matching_transferred_model_count(), 0);

    assert_eq!(result.rejected_target_domain_mismatch(), 1);

    assert_eq!(result.relation_count(), 0);
}

#[test]
fn exact_local_relation_has_authority_over_transferred_relation_with_same_semantic_key() {
    let domain = atom(9001);

    let local = local_hypothesis(atom(1100), atom(1200), EvidenceMode::Counterevidence);

    let transferred = transferred_model(
        atom(9000),
        domain.clone(),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Clean,
    );

    let result = IntegratedDomainModel::build(
        &domain,
        std::slice::from_ref(&local),
        std::slice::from_ref(&transferred),
        policy(16, 16, 16),
    );

    assert_eq!(result.relation_count(), 2);

    assert!(result.relations()[0].same_semantic_key(&result.relations()[1],));

    assert_eq!(
        result.relations()[0].authority(),
        IntegratedDomainRelationAuthority::LocalInterventional
    );

    let best = result
        .best_exact(
            &atom(1100),
            &atom(1200),
            &exact_context(),
            TransitionEffectKind::Added,
            &atom(5),
        )
        .unwrap();

    assert_eq!(
        best.authority(),
        IntegratedDomainRelationAuthority::LocalInterventional
    );
}

#[test]
fn transferred_evidence_variants_for_same_semantic_key_are_retained_not_lossily_merged() {
    let domain = atom(9001);

    let clean = transferred_model(
        atom(9000),
        domain.clone(),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Clean,
    );

    let counter = transferred_model(
        atom(9000),
        domain.clone(),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Counterevidence,
    );

    assert_ne!(
        clean.target_interventional_lift(),
        counter.target_interventional_lift()
    );

    let result =
        IntegratedDomainModel::build(&domain, &[], &[clean.clone(), counter], policy(16, 16, 16));

    assert_eq!(result.relation_count(), 2);

    assert!(result.relations()[0].same_semantic_key(&result.relations()[1],));

    assert_eq!(
        result.relations()[0].confidence_ceiling(),
        clean.strongest_transfer_confidence()
    );
}

#[test]
fn reordered_opaque_transformations_remain_distinct_integrated_relations() {
    let domain = atom(9001);

    let first_target = ordered(&[1100, 1101]);

    let second_target = ordered(&[1101, 1100]);

    let contrast = ordered(&[1200, 1201]);

    let first = local_hypothesis(first_target.clone(), contrast.clone(), EvidenceMode::Clean);

    let second = local_hypothesis(second_target.clone(), contrast, EvidenceMode::Clean);

    let result = IntegratedDomainModel::build(&domain, &[first, second], &[], policy(16, 16, 16));

    assert_ne!(first_target, second_target);

    assert_eq!(result.relation_count(), 2);

    assert_ne!(
        result.relations()[0].transformation(),
        result.relations()[1].transformation()
    );
}

#[test]
fn exact_query_never_matches_wrong_context_or_effect_identity() {
    let domain = atom(9001);

    let local = local_hypothesis(atom(1100), atom(1200), EvidenceMode::Clean);

    let result = IntegratedDomainModel::build(
        &domain,
        std::slice::from_ref(&local),
        &[],
        policy(16, 16, 16),
    );

    let wrong_context = ContextPremiseSet::new(vec![atom(2)]).unwrap();

    assert!(result
        .best_exact(
            &atom(1100,),
            &atom(1200,),
            &exact_context(),
            TransitionEffectKind::Added,
            &atom(5,),
        )
        .is_some());

    assert!(result
        .best_exact(
            &atom(1100,),
            &atom(1200,),
            &wrong_context,
            TransitionEffectKind::Added,
            &atom(5,),
        )
        .is_none());

    assert!(result
        .best_exact(
            &atom(1100,),
            &atom(1200,),
            &exact_context(),
            TransitionEffectKind::Added,
            &atom(6,),
        )
        .is_none());
}

#[test]
fn hard_local_frontier_prefers_stronger_local_interventional_evidence() {
    let domain = atom(9001);

    let strong = local_hypothesis(atom(1100), atom(1200), EvidenceMode::Clean);

    let weak = local_hypothesis(atom(1300), atom(1400), EvidenceMode::Counterevidence);

    assert!(
        strong.validated_causal_confidence().value() > weak.validated_causal_confidence().value()
    );

    let result =
        IntegratedDomainModel::build(&domain, &[weak, strong.clone()], &[], policy(1, 16, 16));

    assert_eq!(result.considered_local_hypothesis_count(), 1);

    assert!(result.local_frontier_truncated());

    assert_eq!(
        result.relations()[0].transformation(),
        strong.transformation()
    );
}

#[test]
fn hard_transferred_frontier_prefers_stronger_target_validated_transfer() {
    let domain = atom(9001);

    let strong = transferred_model(
        atom(9000),
        domain.clone(),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Clean,
    );

    let weak = transferred_model(
        atom(9000),
        domain.clone(),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Counterevidence,
    );

    assert!(
        strong.strongest_transfer_confidence().value()
            > weak.strongest_transfer_confidence().value()
    );

    let result =
        IntegratedDomainModel::build(&domain, &[], &[weak, strong.clone()], policy(16, 1, 16));

    assert_eq!(result.matching_transferred_model_count(), 2);

    assert_eq!(result.considered_transferred_model_count(), 1);

    assert!(result.transferred_frontier_truncated());

    assert_eq!(
        result.relations()[0].confidence_ceiling(),
        strong.strongest_transfer_confidence()
    );
}

#[test]
fn hard_final_frontier_preserves_local_authority_even_against_stronger_transfer_score() {
    let domain = atom(9001);

    let local = local_hypothesis(atom(1100), atom(1200), EvidenceMode::Counterevidence);

    let transferred = transferred_model(
        atom(9000),
        domain.clone(),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Clean,
    );

    assert!(
        transferred.strongest_transfer_confidence().value()
            >= local.validated_causal_confidence().value()
    );

    let result = IntegratedDomainModel::build(
        &domain,
        std::slice::from_ref(&local),
        std::slice::from_ref(&transferred),
        policy(16, 16, 1),
    );

    assert_eq!(result.admitted_before_frontier(), 2);

    assert_eq!(result.relation_count(), 1);

    assert_eq!(
        result.relations()[0].authority(),
        IntegratedDomainRelationAuthority::LocalInterventional
    );
}

#[test]
fn integrated_domain_model_is_deterministic_input_order_invariant_non_mutating_and_facade_equivalent(
) {
    let domain = atom(9001);

    let local_a = local_hypothesis(atom(1100), atom(1200), EvidenceMode::Clean);

    let local_b = local_hypothesis(atom(1300), atom(1400), EvidenceMode::Counterevidence);

    let transfer_a = transferred_model(
        atom(9000),
        domain.clone(),
        atom(100),
        atom(200),
        atom(1100),
        atom(1200),
        EvidenceMode::Clean,
    );

    let transfer_b = transferred_model(
        atom(9100),
        domain.clone(),
        atom(300),
        atom(400),
        atom(1500),
        atom(1600),
        EvidenceMode::Counterevidence,
    );

    let local = vec![local_a, local_b];

    let transferred = vec![transfer_a, transfer_b];

    let local_before = local.clone();

    let transferred_before = transferred.clone();

    let mut reversed_local = local.clone();

    reversed_local.reverse();

    let mut reversed_transferred = transferred.clone();

    reversed_transferred.reverse();

    let model_policy = policy(16, 16, 16);

    let direct = IntegratedDomainModel::build(&domain, &local, &transferred, model_policy);

    let reversed = IntegratedDomainModel::build(
        &domain,
        &reversed_local,
        &reversed_transferred,
        model_policy,
    );

    let facade =
        UniversalIntegratedDomainModel::evaluate(&domain, &local, &transferred, model_policy);

    let repeated =
        UniversalIntegratedDomainModel::evaluate(&domain, &local, &transferred, model_policy);

    assert_eq!(direct, reversed);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(local, local_before);

    assert_eq!(transferred, transferred_before);

    assert_eq!(facade.input_local_hypothesis_count(), local.len());

    assert_eq!(facade.input_transferred_model_count(), transferred.len());
}

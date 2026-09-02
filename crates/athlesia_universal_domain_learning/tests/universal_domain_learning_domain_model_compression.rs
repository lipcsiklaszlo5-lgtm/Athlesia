use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    CalibratedRuleConfidence, CausalContrastInduction, CausalContrastPolicy,
    CausalContrastThresholds, ContextualTransitionEvidenceThresholds,
    ContextualTransitionRuleInduction, ContextualTransitionRulePolicy, CrossContextGeneralization,
    CrossContextGeneralizationPolicy, CrossContextGeneralizationThresholds, CrossDomainTransfer,
    CrossDomainTransferMap, CrossDomainTransferPolicy, CrossDomainTransferThresholds,
    DomainModelCompression, DomainModelCompressionPolicy, GroundedCrossDomainTransferHypothesis,
    GroundedInterventionalCausalHypothesis, GroundedStateSnapshot, GroundedTransferCorrespondence,
    GroundedTransformationEpisode, InterventionalCausalThresholds, InterventionalCausalValidation,
    InterventionalCausalValidationPolicy, InterventionalTransformationEpisode,
    RuleConfidenceCalibration, RuleConfidenceCalibrationPolicy, TransitionEffectKind,
    UniversalDomainModelCompression,
};

#[derive(Clone)]
struct TransferSpec {
    source_domain: CognitiveStructure,
    target_domain: CognitiveStructure,
    source_transformation: CognitiveStructure,
    source_contrast: CognitiveStructure,
    target_transformation: CognitiveStructure,
    target_contrast: CognitiveStructure,
    target_context: u64,
    target_effect: u64,
}

#[derive(Clone, Copy)]
enum TargetEvidenceMode {
    Clean,
    Counterevidence,
    PassiveAugmented,
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

    let contrasts = CausalContrastInduction::induce(
        &observations,
        std::slice::from_ref(&calibrated),
        contrast_policy,
    );

    let validation_thresholds =
        InterventionalCausalThresholds::new(2, 2, 2, signal(400), signal(100)).unwrap();

    let validation_policy =
        InterventionalCausalValidationPolicy::new(16, 16, 16, 4, validation_thresholds).unwrap();

    let evidence = source_interventional_evidence(target, contrast);

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

fn target_evidence(
    spec: &TransferSpec,
    mode: TargetEvidenceMode,
) -> Vec<InterventionalTransformationEpisode> {
    let states = [
        [spec.target_context, 12, 19],
        [spec.target_context, 13, 19],
        [spec.target_context, 14, 19],
        [spec.target_context, 18, 19],
    ];

    let mut evidence = Vec::new();

    for (index, state) in states.into_iter().enumerate() {
        let target_occurs = !matches!(mode, TargetEvidenceMode::Counterevidence) || index != 3;

        let contrast_occurs = matches!(mode, TargetEvidenceMode::Counterevidence) && index == 0;

        let target_after = if target_occurs {
            let mut after = state.to_vec();

            after.push(spec.target_effect);

            after
        } else {
            state.to_vec()
        };

        let contrast_after = if contrast_occurs {
            let mut after = state.to_vec();

            after.push(spec.target_effect);

            after
        } else {
            state.to_vec()
        };

        evidence.push(controlled(
            &state,
            &target_after,
            spec.target_transformation.clone(),
        ));

        evidence.push(controlled(
            &state,
            &contrast_after,
            spec.target_contrast.clone(),
        ));
    }

    if matches!(mode, TargetEvidenceMode::PassiveAugmented) {
        let target_state = [spec.target_context, 21, 19];

        let mut target_after = target_state.to_vec();

        target_after.push(spec.target_effect);

        evidence.push(observed(
            &target_state,
            &target_after,
            spec.target_transformation.clone(),
        ));

        let contrast_state = [spec.target_context, 22, 19];

        evidence.push(observed(
            &contrast_state,
            &contrast_state,
            spec.target_contrast.clone(),
        ));
    }

    evidence
}

fn transferred_hypothesis(
    spec: TransferSpec,
    mode: TargetEvidenceMode,
) -> GroundedCrossDomainTransferHypothesis {
    let source = source_hypothesis(
        spec.source_transformation.clone(),
        spec.source_contrast.clone(),
    );

    let map = CrossDomainTransferMap::new(
        spec.source_domain.clone(),
        spec.target_domain.clone(),
        vec![
            correspondence(
                spec.source_transformation.clone(),
                spec.target_transformation.clone(),
            ),
            correspondence(spec.source_contrast.clone(), spec.target_contrast.clone()),
            correspondence(atom(1), atom(spec.target_context)),
            correspondence(atom(5), atom(spec.target_effect)),
        ],
    )
    .unwrap();

    let thresholds = CrossDomainTransferThresholds::new(1, 1, 1, signal(300), signal(1)).unwrap();

    let policy = CrossDomainTransferPolicy::new(16, 16, 16, 4, thresholds).unwrap();

    let evidence = target_evidence(&spec, mode);

    CrossDomainTransfer::transfer(&evidence, std::slice::from_ref(&source), &map, policy).selected()
        [0]
    .clone()
}

fn base_spec() -> TransferSpec {
    TransferSpec {
        source_domain: atom(9000),
        target_domain: atom(9001),
        source_transformation: atom(100),
        source_contrast: atom(200),
        target_transformation: atom(1100),
        target_contrast: atom(1200),
        target_context: 11,
        target_effect: 15,
    }
}

fn compression_policy(
    max_input: usize,
    max_groups: usize,
    max_output: usize,
) -> DomainModelCompressionPolicy {
    DomainModelCompressionPolicy::new(max_input, max_groups, max_output).unwrap()
}

#[test]
fn compression_policy_requires_positive_hard_bounds() {
    assert_eq!(DomainModelCompressionPolicy::new(0, 10, 10,), None);

    assert_eq!(DomainModelCompressionPolicy::new(10, 0, 10,), None);

    assert_eq!(DomainModelCompressionPolicy::new(10, 10, 0,), None);

    assert!(DomainModelCompressionPolicy::new(10, 10, 10,).is_some());
}

#[test]
fn exact_duplicate_transfer_models_collapse_without_duplicate_provenance() {
    let hypothesis = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let hypotheses = vec![hypothesis.clone(), hypothesis];

    let result = DomainModelCompression::compress(&hypotheses, compression_policy(16, 16, 16));

    assert_eq!(result.selected_count(), 1);

    let model = &result.selected()[0];

    assert_eq!(model.member_count(), 2);

    assert_eq!(model.provenance_count(), 1);

    assert_eq!(model.structurally_removed_member_count(), 1);

    assert_eq!(result.compression_gain().value(), 500);
}

#[test]
fn distinct_source_provenances_share_one_exact_target_model_without_confidence_inflation() {
    let first = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let mut second_spec = base_spec();

    second_spec.source_domain = atom(9100);

    second_spec.source_transformation = atom(300);

    second_spec.source_contrast = atom(400);

    let second = transferred_hypothesis(second_spec, TargetEvidenceMode::Clean);

    let strongest = first
        .transfer_confidence()
        .value()
        .max(second.transfer_confidence().value());

    let result = DomainModelCompression::compress(&[first, second], compression_policy(16, 16, 16));

    assert_eq!(result.selected_count(), 1);

    let model = &result.selected()[0];

    assert_eq!(model.member_count(), 2);

    assert_eq!(model.provenance_count(), 2);

    assert_eq!(model.strongest_transfer_confidence().value(), strongest);

    assert!(model.strongest_transfer_confidence().value() <= 1000);
}

#[test]
fn different_target_transformation_identity_cannot_be_compressed_together() {
    let first = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let mut second_spec = base_spec();

    second_spec.target_transformation = atom(1300);

    second_spec.target_contrast = atom(1400);

    let second = transferred_hypothesis(second_spec, TargetEvidenceMode::Clean);

    let result = DomainModelCompression::compress(&[first, second], compression_policy(16, 16, 16));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.compression_gain().value(), 0);
}

#[test]
fn different_target_context_identity_cannot_be_compressed_together() {
    let first = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let mut second_spec = base_spec();

    second_spec.target_context = 31;

    let second = transferred_hypothesis(second_spec, TargetEvidenceMode::Clean);

    let result = DomainModelCompression::compress(&[first, second], compression_policy(16, 16, 16));

    assert_eq!(result.selected_count(), 2);

    assert_ne!(
        result.selected()[0].target_context(),
        result.selected()[1].target_context()
    );
}

#[test]
fn different_target_evidence_statistics_are_not_lossily_merged() {
    let clean = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let counterevidence = transferred_hypothesis(base_spec(), TargetEvidenceMode::Counterevidence);

    assert_ne!(
        clean.target_interventional_lift(),
        counterevidence.target_interventional_lift()
    );

    let result =
        DomainModelCompression::compress(&[clean, counterevidence], compression_policy(16, 16, 16));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.structurally_removed_member_count(), 0);
}

#[test]
fn different_passive_evidence_summaries_remain_distinct_models() {
    let clean = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let passive = transferred_hypothesis(base_spec(), TargetEvidenceMode::PassiveAugmented);

    assert_ne!(
        clean.passive_corroborating_count(),
        passive.passive_corroborating_count()
    );

    let result =
        DomainModelCompression::compress(&[clean, passive], compression_policy(16, 16, 16));

    assert_eq!(result.selected_count(), 2);
}

#[test]
fn reordered_opaque_target_structures_remain_exactly_distinct_under_compression() {
    let mut first_spec = base_spec();

    first_spec.source_transformation = ordered(&[100, 101]);

    first_spec.source_contrast = ordered(&[200, 201]);

    first_spec.target_transformation = ordered(&[1100, 1101]);

    first_spec.target_contrast = ordered(&[1200, 1201]);

    let first = transferred_hypothesis(first_spec.clone(), TargetEvidenceMode::Clean);

    let mut second_spec = first_spec;

    second_spec.target_transformation = ordered(&[1101, 1100]);

    let second = transferred_hypothesis(second_spec, TargetEvidenceMode::Clean);

    assert_ne!(
        first.target_transformation(),
        second.target_transformation()
    );

    let result = DomainModelCompression::compress(&[first, second], compression_policy(16, 16, 16));

    assert_eq!(result.selected_count(), 2);
}

#[test]
fn hard_input_frontier_prefers_stronger_evidence_deterministically() {
    let strong = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let weak = transferred_hypothesis(base_spec(), TargetEvidenceMode::Counterevidence);

    assert!(strong.transfer_confidence().value() > weak.transfer_confidence().value());

    let first = DomainModelCompression::compress(
        &[weak.clone(), strong.clone()],
        compression_policy(1, 16, 16),
    );

    let second =
        DomainModelCompression::compress(&[strong.clone(), weak], compression_policy(1, 16, 16));

    assert_eq!(first, second);

    assert!(first.input_frontier_truncated());

    assert_eq!(first.considered_hypothesis_count(), 1);

    assert_eq!(
        first.selected()[0].strongest_transfer_confidence(),
        strong.transfer_confidence()
    );
}

#[test]
fn hard_group_generation_frontier_is_reported_without_cross_group_aliasing() {
    let first = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let mut second_spec = base_spec();

    second_spec.target_transformation = atom(1300);

    second_spec.target_contrast = atom(1400);

    let second = transferred_hypothesis(second_spec, TargetEvidenceMode::Clean);

    let result = DomainModelCompression::compress(&[first, second], compression_policy(16, 1, 16));

    assert_eq!(result.possible_model_group_count(), 2);

    assert_eq!(result.generated_model_group_count(), 1);

    assert!(result.group_generation_truncated());

    assert_eq!(result.selected_count(), 1);
}

#[test]
fn hard_output_frontier_prioritizes_greater_structural_redundancy() {
    let redundant = transferred_hypothesis(base_spec(), TargetEvidenceMode::Counterevidence);

    let mut second_spec = base_spec();

    second_spec.target_transformation = atom(1300);

    second_spec.target_contrast = atom(1400);

    let compact = transferred_hypothesis(second_spec, TargetEvidenceMode::Clean);

    let hypotheses = vec![
        redundant.clone(),
        redundant.clone(),
        redundant,
        compact.clone(),
        compact,
    ];

    let result = DomainModelCompression::compress(&hypotheses, compression_policy(16, 16, 1));

    assert_eq!(result.admitted_before_frontier(), 2);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].member_count(), 3);

    assert_eq!(result.selected()[0].structurally_removed_member_count(), 2);
}

#[test]
fn domain_model_compression_is_deterministic_non_mutating_and_facade_equivalent() {
    let first = transferred_hypothesis(base_spec(), TargetEvidenceMode::Clean);

    let mut second_spec = base_spec();

    second_spec.source_domain = atom(9100);

    second_spec.source_transformation = atom(300);

    second_spec.source_contrast = atom(400);

    let second = transferred_hypothesis(second_spec, TargetEvidenceMode::Clean);

    let hypotheses = vec![first, second];

    let before = hypotheses.clone();

    let policy = compression_policy(16, 16, 16);

    let direct = DomainModelCompression::compress(&hypotheses, policy);

    let facade = UniversalDomainModelCompression::evaluate(&hypotheses, policy);

    let repeated = UniversalDomainModelCompression::evaluate(&hypotheses, policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(hypotheses, before);

    assert_eq!(facade.input_hypothesis_count(), hypotheses.len());

    assert!(facade.generated_model_group_count() <= facade.possible_model_group_count());

    assert!(facade.selected_count() <= facade.admitted_before_frontier());
}

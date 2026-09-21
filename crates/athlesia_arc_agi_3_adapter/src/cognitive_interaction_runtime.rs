use crate::{
    interactive_session_runtime::{
        ArcAgi3CompletedTurn, ArcAgi3InteractiveSession, ArcAgi3InteractiveSessionError,
        ArcAgi3SessionCommand,
    },
    perceptual_ingestion_bridge::{
        ArcAgi3PerceptualBridgeError, ArcAgi3PerceptualIngestionBridge, ArcAgi3PerceptualProjection,
    },
    ArcAgi3Observation,
};
use athlesia_integrated_cognitive_agent::{
    CognitiveCycleStateTransitionRequest, EnvironmentActionDispatch,
    EnvironmentActionDispatchStatus, EnvironmentInteractionBoundary, IntegratedAgentPolicy,
    OnlineCognitiveOrchestration, OnlineCognitiveOrchestrationInput,
    OnlineCognitiveOrchestrationResult,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArcAgi3CognitiveInteractionError {
    DispatchRejected(EnvironmentActionDispatchStatus),
    ReadyDispatchMissing,
    Session(ArcAgi3InteractiveSessionError),
    Perception(ArcAgi3PerceptualBridgeError),
}

impl From<ArcAgi3InteractiveSessionError> for ArcAgi3CognitiveInteractionError {
    fn from(error: ArcAgi3InteractiveSessionError) -> Self {
        Self::Session(error)
    }
}

impl From<ArcAgi3PerceptualBridgeError> for ArcAgi3CognitiveInteractionError {
    fn from(error: ArcAgi3PerceptualBridgeError) -> Self {
        Self::Perception(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3CognitiveInteractionStep {
    orchestration: OnlineCognitiveOrchestrationResult,
    dispatch: EnvironmentActionDispatch,
    command: ArcAgi3SessionCommand,
}

impl ArcAgi3CognitiveInteractionStep {
    pub fn orchestration(&self) -> &OnlineCognitiveOrchestrationResult {
        &self.orchestration
    }

    pub fn dispatch(&self) -> &EnvironmentActionDispatch {
        &self.dispatch
    }

    pub fn command(&self) -> &ArcAgi3SessionCommand {
        &self.command
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3CognitiveInteractionCompletion {
    turn: ArcAgi3CompletedTurn,
    perception: ArcAgi3PerceptualProjection,
}

impl ArcAgi3CognitiveInteractionCompletion {
    pub fn turn(&self) -> &ArcAgi3CompletedTurn {
        &self.turn
    }

    pub fn perception(&self) -> &ArcAgi3PerceptualProjection {
        &self.perception
    }

    pub fn has_cognitive_feedback(&self) -> bool {
        self.turn.has_cognitive_feedback()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcAgi3ExperimentDispatchAuthority<'a> {
    result:
        &'a athlesia_autonomous_active_experimentation::IntegratedAutonomousExperimentationResult,
    expected_source_state: &'a athlesia_mindstone_sparse_cognition::CognitiveStructure,
}

impl<'a> ArcAgi3ExperimentDispatchAuthority<'a> {
    pub fn new(
        result:
            &'a athlesia_autonomous_active_experimentation::
                IntegratedAutonomousExperimentationResult,
        expected_source_state: &'a athlesia_mindstone_sparse_cognition::CognitiveStructure,
    ) -> Self {
        Self {
            result,
            expected_source_state,
        }
    }

    pub fn result(
        self,
    ) -> &'a athlesia_autonomous_active_experimentation::IntegratedAutonomousExperimentationResult
    {
        self.result
    }

    pub fn expected_source_state(
        self,
    ) -> &'a athlesia_mindstone_sparse_cognition::CognitiveStructure {
        self.expected_source_state
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'a> {
    pub exploitation_actions: &'a [crate::ArcAgi3Action],
    pub goal: &'a athlesia_executive_agency::ExecutiveGoal,
    pub goal_alignment: CognitiveSignal,
    pub exploitation_execution_cost: CognitiveSignal,

    /*
     * Exact native-M50 source identity expected at the ARC grounding
     * boundary.  This is intentionally not reconstructed by the adapter.
     */
    pub expected_experiment_source_state: &'a CognitiveStructure,

    /*
     * These are caller-native M50 inputs.
     *
     * The adapter does not manufacture competing predictions, beliefs,
     * controllability, confidence, EIG, or cost.
     */
    pub native_possibilities:
        &'a [
            athlesia_autonomous_active_experimentation::
                GroundedExperimentPossibility
        ],
    pub beliefs:
        &'a [
            athlesia_autonomous_active_experimentation::
                HypothesisBeliefState
        ],

    pub version_policy:
        athlesia_universal_domain_learning::
            GroundedExplanatoryVersionSpacePolicy,
    pub discrimination_policy:
        athlesia_autonomous_active_experimentation::
            EpistemicForecastDiscriminationPolicy,
    pub expectation_policy:
        athlesia_autonomous_active_experimentation::
            EmpiricalExpectedEpistemicProgressPolicy,
    pub priority_policy:
        athlesia_autonomous_active_experimentation::
            EmpiricalEpistemicActionPriorityPolicy,
    pub proposal_policy:
        athlesia_autonomous_active_experimentation::
            BeliefDrivenExperimentProposalPolicy,
    pub executive_policy:
        athlesia_executive_agency::
            ExecutiveAgencyPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3UnifiedExecutiveAuthority {
    source_state: CognitiveStructure,
    selected: crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate,
}

impl ArcAgi3UnifiedExecutiveAuthority {
    fn new(
        source_state: CognitiveStructure,
        selected: crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate,
    ) -> Self {
        Self {
            source_state,
            selected,
        }
    }

    pub fn action(&self) -> crate::ArcAgi3Action {
        self.selected.action()
    }

    pub fn cognitive_action(&self) -> &CognitiveStructure {
        self.selected.candidate().action()
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        self.selected.candidate().predicted_outcome()
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn candidate(&self) -> &athlesia_executive_agency::GroundedExecutiveActionCandidate {
        self.selected.candidate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3CognitiveInteractionRuntime {
    session: ArcAgi3InteractiveSession,
    perception: ArcAgi3PerceptualProjection,
    cognition: athlesia_integrated_cognitive_agent::OnlinePersistentCognitiveState,
}

impl ArcAgi3CognitiveInteractionRuntime {
    pub fn new(
        initial_observation: ArcAgi3Observation,
        first_perceptual_observation_index: u64,
    ) -> Result<Self, ArcAgi3CognitiveInteractionError> {
        let perception = ArcAgi3PerceptualIngestionBridge::project_observation(
            &initial_observation,
            first_perceptual_observation_index,
            None,
        )?;

        Ok(Self {
            session: ArcAgi3InteractiveSession::new(initial_observation),
            perception,
            cognition: athlesia_integrated_cognitive_agent::OnlinePersistentCognitiveState::new(),
        })
    }

    pub fn session(&self) -> &ArcAgi3InteractiveSession {
        &self.session
    }

    pub fn perception(&self) -> &ArcAgi3PerceptualProjection {
        &self.perception
    }

    pub fn cognition(
        &self,
    ) -> &athlesia_integrated_cognitive_agent::OnlinePersistentCognitiveState {
        &self.cognition
    }

    fn live_temporal_grouping_policy(
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidencePolicy
    {
        athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidencePolicy::new(
            2,
        )
        .expect("live temporal support threshold is positive")
    }

    fn live_grouping_generation_policy(
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingGenerationPolicy {
        athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingGenerationPolicy::new(
            256, 256,
        )
        .expect("live grouping frontier bounds are positive")
    }

    fn live_grouping_behavior_retention_policy(
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingBehaviorRetentionPolicy
    {
        athlesia_core_knowledge_perceptual_grounding::
            PerceptualGroupingBehaviorRetentionPolicy::new(2, 2)
            .expect("live grouping behavior thresholds are positive")
    }

    fn live_grouping_appearance_retention_policy(
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingAppearanceRetentionPolicy
    {
        athlesia_core_knowledge_perceptual_grounding::
            PerceptualGroupingAppearanceRetentionPolicy::new(2)
            .expect(
                "live appearance evidence requires a positive history threshold",
            )
    }

    pub fn current_perceptual_grouping_frontier(
        &self,
        temporal_policy:
            athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidencePolicy,
        grouping_policy:
            athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingGenerationPolicy,
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingGenerationResult {
        ArcAgi3PerceptualIngestionBridge::temporally_supported_grid_grouping_candidates(
            self.cognition.perceptual_temporal_evidence(),
            self.perception.latest_frame(),
            temporal_policy,
            grouping_policy,
        )
    }

    pub fn current_empirically_coherent_groupings(
        &self,
    ) -> Vec<
        athlesia_core_knowledge_perceptual_grounding::
            PerceptualGroupingCandidate
    > {
        self.cognition
            .current_empirically_coherent_groupings(
                self.perception.latest_frame(),
                Self::live_grouping_behavior_retention_policy(),
            )
    }

    pub fn current_objecthood_eligible_groupings(
        &self,
    ) -> Vec<
        athlesia_core_knowledge_perceptual_grounding::
            PerceptualGroupingCandidate
    > {
        use athlesia_core_knowledge_perceptual_grounding::
            PerceptualGroupingAppearanceObservationEvidence;

        let frame =
            self.perception.latest_frame();

        /*
         * ARC-specific responsibility ends here:
         *
         * decode the current grid and report raw visual observations for
         * groupings that have already passed the retained behavior frontier.
         *
         * No temporal-support interpretation and no object-promotion
         * decision belongs to the adapter.
         */
        let visual_observations =
            self
                .current_empirically_coherent_groupings()
                .into_iter()
                .filter_map(
                    |grouping| {
                        let (
                            appearance_cohesion,
                            contrast_boundary,
                        ) =
                            ArcAgi3PerceptualIngestionBridge::
                                grouping_visual_objecthood_evidence(
                                    frame,
                                    &grouping,
                                )?;

                        Some(
                            PerceptualGroupingAppearanceObservationEvidence::
                                new(
                                    grouping,
                                    appearance_cohesion,
                                    contrast_boundary,
                                ),
                        )
                    },
                )
                .collect::<Vec<_>>();

        self.cognition
            .current_objecthood_eligible_groupings_from_visual_observations(
                &visual_observations,
                Self::live_temporal_grouping_policy(),
            )
    }

    pub fn current_provisional_object_hypotheses(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::ObjectHypothesis> {
        let groupings =
            self.current_objecthood_eligible_groupings();

        self.cognition
            .current_provisional_object_hypotheses_from_groupings(
                self.perception.latest_frame(),
                &groupings,
                Self::live_grouping_appearance_retention_policy(),
            )
    }

    fn live_scene_grounding_policy(
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroundingPolicy {
        /*
         * This is a bounded runtime resource policy, not semantic evidence.
         *
         * Upstream live grouping already bounds provisional object hypotheses.
         * Scene search then remains finite while still allowing multiple
         * mutually incompatible explanations to compete.
         */
        athlesia_core_knowledge_perceptual_grounding::PerceptualGroundingPolicy::new(64, 32)
            .expect("live scene grounding bounds are positive")
    }

    fn provisional_object_scene_support(
        hypothesis: &athlesia_core_knowledge_perceptual_grounding::ObjectHypothesis,
    ) -> Option<athlesia_mindstone_sparse_cognition::CognitiveSignal> {
        let evidence = hypothesis.evidence();
        let zero = athlesia_mindstone_sparse_cognition::CognitiveSignal::zero();

        [
            evidence.cohesion(),
            evidence.persistence(),
            evidence.common_change(),
            evidence.boundary(),
            evidence.containment(),
            evidence.topology(),
        ]
        .into_iter()
        .filter(|signal| *signal > zero)
        .min()
    }

    fn hypotheses_overlap(
        left: &athlesia_core_knowledge_perceptual_grounding::ObjectHypothesis,
        right: &athlesia_core_knowledge_perceptual_grounding::ObjectHypothesis,
    ) -> bool {
        left.members()
            .iter()
            .copied()
            .any(|member| right.contains(member))
    }

    fn scene_explanatory_support(
        &self,
        hypotheses: &[athlesia_core_knowledge_perceptual_grounding::ObjectHypothesis],
    ) -> Option<athlesia_mindstone_sparse_cognition::CognitiveSignal> {
        use athlesia_core_knowledge_perceptual_grounding::EmpiricalObjecthoodSignalCalibration;

        if hypotheses.is_empty() {
            return None;
        }

        let reliability_floor = hypotheses
            .iter()
            .map(Self::provisional_object_scene_support)
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .min()?;

        let frame = self.perception.latest_frame();

        let perceptual_cell_count = frame
            .elements()
            .iter()
            .filter(|element| {
                ArcAgi3PerceptualIngestionBridge::decode_handle_coordinate(element.handle())
                    .is_some()
            })
            .count();

        if perceptual_cell_count == 0 {
            return None;
        }

        let mut covered = std::collections::BTreeSet::new();

        for hypothesis in hypotheses {
            for &member in hypothesis.members() {
                if ArcAgi3PerceptualIngestionBridge::decode_handle_coordinate(member).is_some() {
                    covered.insert(member);
                }
            }
        }

        let coverage = EmpiricalObjecthoodSignalCalibration::from_counts(
            covered.len(),
            perceptual_cell_count,
        )?;

        Some(reliability_floor.min(coverage))
    }

    fn current_scene_candidates(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::SceneInterpretation> {
        use athlesia_core_knowledge_perceptual_grounding::SceneInterpretation;

        let hypotheses = self.current_provisional_object_hypotheses();

        if hypotheses.is_empty() {
            return Vec::new();
        }

        let policy = Self::live_scene_grounding_policy();

        /*
         * Each provisional object seeds one alternative maximal,
         * non-overlapping explanation.
         *
         * This gives overlapping object hypotheses a chance to produce
         * genuinely competing scenes rather than silently merging them.
         *
         * The upstream object frontier is already bounded, and M46 applies
         * the final scene frontier.
         */
        let mut candidates = Vec::new();

        for seed_index in 0..hypotheses.len() {
            let mut scene_hypotheses = vec![hypotheses[seed_index].clone()];

            for (candidate_index, candidate) in hypotheses.iter().enumerate() {
                if candidate_index == seed_index {
                    continue;
                }

                if scene_hypotheses.len() >= policy.max_object_hypotheses_per_scene() {
                    break;
                }

                let overlaps_existing = scene_hypotheses
                    .iter()
                    .any(|existing| Self::hypotheses_overlap(existing, candidate));

                if !overlaps_existing {
                    scene_hypotheses.push(candidate.clone());
                }
            }

            let Some(explanatory_support) = self.scene_explanatory_support(&scene_hypotheses)
            else {
                continue;
            };

            let Some(scene) = SceneInterpretation::new(scene_hypotheses, explanatory_support)
            else {
                continue;
            };

            if scene.contains_overlapping_hypotheses() {
                continue;
            }

            candidates.push(scene);
        }

        candidates
    }

    pub fn current_competing_scene_interpretations(
        &self,
    ) -> athlesia_core_knowledge_perceptual_grounding::SceneCompetitionResult {
        let candidates = self.current_scene_candidates();

        athlesia_core_knowledge_perceptual_grounding::CoreKnowledgePerceptualGrounding::evaluate(
            self.perception.latest_frame(),
            &candidates,
            Self::live_scene_grounding_policy(),
        )
    }

    pub fn current_best_scene_interpretation(
        &self,
    ) -> Option<athlesia_core_knowledge_perceptual_grounding::SceneInterpretation> {
        self.current_competing_scene_interpretations()
            .selected()
            .first()
            .cloned()
    }

    fn live_transition_schema_policy() -> athlesia_universal_domain_learning::TransitionSchemaPolicy
    {
        let minimum_precision = athlesia_mindstone_sparse_cognition::CognitiveSignal::new(600)
            .expect("live transition precision threshold is positive and bounded");

        let minimum_association_lift = athlesia_mindstone_sparse_cognition::CognitiveSignal::new(1)
            .expect("live transition association threshold is positive and bounded");

        athlesia_universal_domain_learning::TransitionSchemaPolicy::new(
            2,
            minimum_precision,
            minimum_association_lift,
            256,
            64,
        )
        .expect("live transition-schema policy has positive bounded frontiers")
    }

    fn live_transition_schema_learning_policy(
    ) -> athlesia_integrated_cognitive_agent::EndogenousTransitionSchemaLearningPolicy {
        athlesia_integrated_cognitive_agent::EndogenousTransitionSchemaLearningPolicy::new(
            256,
            Self::live_transition_schema_policy(),
        )
        .expect("live transition learning has a positive evidence frontier")
    }

    fn live_perceptual_world_context(
    ) -> athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext {
        use athlesia_core_knowledge_perceptual_grounding::{
            ActionConsequencePolicy, IntegratedPerceptualWorldContext, PerceptualChangePolicy,
            PersistenceTrackingPolicy, TopologicalRelationPolicy,
        };

        IntegratedPerceptualWorldContext::new(
            Self::live_scene_grounding_policy(),
            PersistenceTrackingPolicy::new(64, 64, 128)
                .expect("live persistence bounds are positive"),
            TopologicalRelationPolicy::new(64, 128).expect("live topology bounds are positive"),
            PerceptualChangePolicy::new(64, 128).expect("live change bounds are positive"),
            ActionConsequencePolicy::new(64, 64, 128)
                .expect("live action-consequence bounds are positive"),
        )
    }

    fn live_executable_world_model_policy(
    ) -> athlesia_universal_domain_learning::GroundedExecutableWorldModelPolicy {
        athlesia_universal_domain_learning::GroundedExecutableWorldModelPolicy::new(64)
            .expect("live executable world-model schema frontier is positive")
    }

    pub fn current_grounded_world_state(
        &self,
    ) -> Option<athlesia_universal_domain_learning::GroundedStateSnapshot> {
        let current_scene = self.current_best_scene_interpretation()?;

        let current_facts =
            athlesia_core_knowledge_perceptual_grounding::
                GroundedPerceptualStateProjector::scene_facts(
                    self.perception.latest_frame(),
                    &current_scene,
                )?;

        athlesia_universal_domain_learning::GroundedStateSnapshot::new(current_facts)
    }

    pub fn current_action_qualified_empirical_successor_frequency(
        &self,
        action: &CognitiveStructure,
    ) -> Option<
        athlesia_integrated_cognitive_agent::
            ActionQualifiedEmpiricalSuccessorFrequency,
    > {
        /*
         * C16H-B2-C live query authority.
         *
         * The caller supplies only the contemplated action identity.
         *
         * The conditioning representation is obtained exclusively from
         * the frozen B0 current-state authority, which derives from the
         * latest grounded perception and fails closed when no current
         * grounding exists.
         *
         * Historical transition memory is not inspected here.
         * Frequency calculation remains exclusively B2-B authority.
         */
        let current_representation =
            self.current_grounded_world_state()?;

        Some(
            self.cognition()
                .transition_schema_learning()
                .action_qualified_empirical_successor_frequency(
                    &current_representation,
                    action,
                ),
        )
    }


    pub fn current_executable_world_model(
        &self,
    ) -> Option<athlesia_universal_domain_learning::GroundedExecutableWorldModel> {
        let episodes = self.cognition.transition_schema_learning().episodes();

        /*
         * One transition cannot self-confirm an executable causal model.
         */
        if episodes.len() < 2 {
            return None;
        }

        let induction =
            athlesia_universal_domain_learning::UniversalTransitionSchemaInduction::evaluate(
                episodes,
                &[],
                Self::live_transition_schema_policy(),
            );

        if induction.selected().is_empty() {
            return None;
        }

        Some(
            athlesia_universal_domain_learning::UniversalGroundedExecutableWorldModel::build(
                induction.selected(),
                Self::live_executable_world_model_policy(),
            ),
        )
    }

    pub fn current_structural_prediction_for_action(
        &self,
        action: crate::ArcAgi3Action,
    ) -> Option<athlesia_universal_domain_learning::GroundedStructuralPrediction> {
        let state = self.current_grounded_world_state()?;

        let model = self.current_executable_world_model()?;

        let transformation =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(action);

        Some(
            athlesia_universal_domain_learning::UniversalGroundedExecutableWorldModel::predict(
                &state,
                &transformation,
                &model,
            ),
        )
    }

    /*
     * These atoms are transport labels for the exact partial structural
     * prediction carried into M48's predicted_outcome field.
     *
     * They are not observations, confidence, utility, or world facts.
     */
    const MODEL_PREDICTION_TAG: u64 = 0x5034_4742_3200_0001;
    const MODEL_ADDITION_TAG: u64 = 0x5034_4742_3200_0002;
    const MODEL_REMOVAL_TAG: u64 = 0x5034_4742_3200_0003;

    fn model_prediction_structure(
        prediction: &athlesia_universal_domain_learning::GroundedStructuralPrediction,
    ) -> Option<athlesia_mindstone_sparse_cognition::CognitiveStructure> {
        use athlesia_mindstone_sparse_cognition::CognitiveStructure;

        if !prediction.predicted() {
            return None;
        }

        let mut terms = Vec::with_capacity(
            2_usize
                .saturating_add(prediction.additions().len())
                .saturating_add(prediction.removals().len()),
        );

        terms.push(CognitiveStructure::atom(Self::MODEL_PREDICTION_TAG));

        terms.push(prediction.transformation().clone());

        for fact in prediction.additions() {
            let effect = CognitiveStructure::ordered(vec![
                CognitiveStructure::atom(Self::MODEL_ADDITION_TAG),
                fact.clone(),
            ])?;

            terms.push(effect);
        }

        for fact in prediction.removals() {
            let effect = CognitiveStructure::ordered(vec![
                CognitiveStructure::atom(Self::MODEL_REMOVAL_TAG),
                fact.clone(),
            ])?;

            terms.push(effect);
        }

        CognitiveStructure::ordered(terms)
    }

    fn model_prediction_empirical_authority(
        model: &athlesia_universal_domain_learning::GroundedExecutableWorldModel,
        prediction: &athlesia_universal_domain_learning::GroundedStructuralPrediction,
    ) -> Option<(
        athlesia_mindstone_sparse_cognition::CognitiveSignal,
        athlesia_mindstone_sparse_cognition::CognitiveSignal,
    )> {
        use athlesia_mindstone_sparse_cognition::CognitiveSignal;
        use athlesia_universal_domain_learning::TransitionEffectKind;

        if !prediction.predicted() {
            return None;
        }

        let supporting = model
            .schemas()
            .iter()
            .filter(|schema| {
                if schema.transformation() != prediction.transformation() {
                    return false;
                }

                match schema.effect_kind() {
                    TransitionEffectKind::Added => prediction.predicts_addition(schema.fact()),

                    TransitionEffectKind::Removed => prediction.predicts_removal(schema.fact()),
                }
            })
            .collect::<Vec<_>>();

        if supporting.is_empty() {
            return None;
        }

        /*
         * Weakest-link authority:
         *
         * A multi-effect prediction cannot inherit more confidence or
         * controllability than its least-supported applicable component.
         */
        let evidence_confidence = supporting.iter().map(|schema| schema.precision()).min()?;

        let controllability = supporting
            .iter()
            .map(|schema| schema.association_lift())
            .min()?;

        if evidence_confidence == CognitiveSignal::zero()
            || controllability == CognitiveSignal::zero()
        {
            return None;
        }

        Some((evidence_confidence, controllability))
    }

    fn model_grounded_executive_candidate(
        model: &athlesia_universal_domain_learning::GroundedExecutableWorldModel,
        state: &athlesia_universal_domain_learning::GroundedStateSnapshot,
        action: crate::ArcAgi3Action,
        goal: &athlesia_executive_agency::ExecutiveGoal,
        goal_alignment: athlesia_mindstone_sparse_cognition::CognitiveSignal,
        execution_cost: athlesia_mindstone_sparse_cognition::CognitiveSignal,
    ) -> Option<athlesia_executive_agency::GroundedExecutiveActionCandidate> {
        use athlesia_mindstone_sparse_cognition::CognitiveSignal;

        let transformation =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(action);

        let prediction = model.predict(state, &transformation);

        if !prediction.predicted() {
            return None;
        }

        let predicted_outcome = Self::model_prediction_structure(&prediction)?;

        let (evidence_confidence, controllability) =
            Self::model_prediction_empirical_authority(model, &prediction)?;

        Some(
            athlesia_executive_agency::GroundedExecutiveActionCandidate::new(
                goal.identity().clone(),
                transformation,
                predicted_outcome,
                goal_alignment,
                controllability,
                evidence_confidence,
                /*
                 * This path exploits a learned causal model.
                 *
                 * It is not an epistemic experiment and therefore
                 * receives no fabricated information-gain signal.
                 */
                CognitiveSignal::zero(),
                execution_cost,
            ),
        )
    }

    fn current_model_grounded_authorized_candidates(
        &self,
        candidate_actions: &[crate::ArcAgi3Action],
        goal: &athlesia_executive_agency::ExecutiveGoal,
        goal_alignment: athlesia_mindstone_sparse_cognition::CognitiveSignal,
        execution_cost: athlesia_mindstone_sparse_cognition::CognitiveSignal,
    ) -> Vec<crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate> {
        let Some(state) = self.current_grounded_world_state() else {
            return Vec::new();
        };

        let Some(model) = self.current_executable_world_model() else {
            return Vec::new();
        };

        let mut authorized = Vec::new();

        for &action in candidate_actions {
            let Some(candidate) = Self::model_grounded_executive_candidate(
                &model,
                &state,
                action,
                goal,
                goal_alignment,
                execution_cost,
            ) else {
                continue;
            };

            let Ok(grounded) =
                crate::action_grounding_bridge::
                    ArcAgi3ActionGroundingBridge::
                    authorize_executive_candidate(
                        self.observation(),
                        &candidate,
                    )
            else {
                continue;
            };

            if authorized.iter().any(
                |existing: &crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate| {
                    existing.candidate() == grounded.candidate()
                },
            ) {
                continue;
            }

            authorized.push(grounded);
        }

        authorized
    }

    fn selected_authorized_executive_candidate(
        authorized: &[crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate],
        goal: &athlesia_executive_agency::ExecutiveGoal,
        policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
    ) -> Option<crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate> {
        if authorized.is_empty() {
            return None;
        }

        let candidates = authorized
            .iter()
            .map(|grounded| grounded.candidate().clone())
            .collect::<Vec<_>>();

        /*
         * Single final action/value authority.
         *
         * Neither the learned world-model path nor M50 performs a local
         * dispatch ranking here. Both merely contribute grounded candidates.
         */
        let executive = athlesia_executive_agency::UniversalExecutiveAgency::evaluate(
            std::slice::from_ref(goal),
            &candidates,
            policy,
        );

        let selected = executive.selected().first()?;

        authorized
            .iter()
            .find(|grounded| {
                grounded.candidate().action() == selected.action()
                    && grounded.candidate().predicted_outcome() == selected.predicted_outcome()
            })
            .cloned()
    }

    fn select_authorized_executive_candidate(
        authorized: &[crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate],
        goal: &athlesia_executive_agency::ExecutiveGoal,
        policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
    ) -> Option<crate::ArcAgi3Action> {
        Self::selected_authorized_executive_candidate(authorized, goal, policy)
            .map(|grounded| grounded.action())
    }

    pub fn current_model_grounded_action_selection(
        &self,
        candidate_actions: &[crate::ArcAgi3Action],
        goal: &athlesia_executive_agency::ExecutiveGoal,
        goal_alignment: athlesia_mindstone_sparse_cognition::CognitiveSignal,
        execution_cost: athlesia_mindstone_sparse_cognition::CognitiveSignal,
        policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
    ) -> Option<crate::ArcAgi3Action> {
        let authorized = self.current_model_grounded_authorized_candidates(
            candidate_actions,
            goal,
            goal_alignment,
            execution_cost,
        );

        Self::select_authorized_executive_candidate(&authorized, goal, policy)
    }

    pub fn current_successor_informed_unified_executive_authority(
        &self,
        request:
            ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'_>,
    ) -> Option<ArcAgi3UnifiedExecutiveAuthority> {
        let ArcAgi3SuccessorInformedUnifiedExecutiveRequest {
            exploitation_actions,
            goal,
            goal_alignment,
            exploitation_execution_cost,
            expected_experiment_source_state,
            native_possibilities,
            beliefs,
            version_policy,
            discrimination_policy,
            expectation_policy,
            priority_policy,
            proposal_policy,
            executive_policy,
        } = request;

        /*
         * B0 current-grounding authority is shared by both branches.
         *
         * Native M50 gating is therefore conditioned on the exact same
         * currently grounded representation that supplies exploitation
         * provenance.
         */
        let current_state =
            self.current_grounded_world_state()?;

        let mut authorized =
            self.current_model_grounded_authorized_candidates(
                exploitation_actions,
                goal,
                goal_alignment,
                exploitation_execution_cost,
            );

        let exploitation_source_state =
            CognitiveStructure::unordered(
                current_state
                    .facts()
                    .to_vec(),
            )
            .expect(
                "grounded current state contains at least one fact",
            );

        let mut provenance =
            authorized
                .iter()
                .cloned()
                .map(
                    |candidate| {
                        (
                            candidate,
                            exploitation_source_state
                                .clone(),
                        )
                    },
                )
                .collect::<Vec<_>>();

        /*
         * Preserve caller-native action identity and order.
         *
         * No sort, dedup, probability estimate, predicted outcome,
         * information gain, confidence, controllability, or utility is
         * synthesized here.
         */
        let native_actions =
            native_possibilities
                .iter()
                .map(
                    |possibility| {
                        possibility
                            .action()
                            .clone()
                    },
                )
                .collect::<Vec<_>>();

        let delegated =
            self
                .cognition()
                .current_successor_informed_native_m50_proposal_delegation(
                    athlesia_integrated_cognitive_agent::
                        SuccessorInformedNativeM50ProposalDelegationRequest {
                            native_input:
                                athlesia_integrated_cognitive_agent::
                                    SuccessorInformedNativeProposalInputRequest {
                                        state:
                                            &current_state,
                                        actions:
                                            &native_actions,
                                        version_policy,
                                        discrimination_policy,
                                        expectation_policy,
                                        priority_policy,
                                        native_possibilities,
                                    },
                            beliefs,
                            proposal_policy,
                        },
                );

        if let Some(result) =
            delegated
        {
            /*
             * C16I-E is the only native-M50 -> M48 grounding bridge.
             *
             * Its Result frontier is intentionally atomic: if any
             * generated proposal fails exact source identity or current
             * ARC availability, the complete experimental contribution
             * fails closed BEFORE M48 is invoked.
             */
            let experimental_candidates =
                crate::action_grounding_bridge::
                    ArcAgi3ActionGroundingBridge::
                        ground_belief_driven_proposal_frontier_for_goal(
                            self.observation(),
                            expected_experiment_source_state,
                            goal,
                            goal_alignment,
                            &result,
                        )
                        .ok()?;

            for candidate in
                experimental_candidates
            {
                /*
                 * Convert the already grounded M48 candidate into the
                 * runtime's authorized wrapper without altering any
                 * candidate field.
                 */
                let grounded =
                    crate::action_grounding_bridge::
                        ArcAgi3ActionGroundingBridge::
                            authorize_executive_candidate(
                                self.observation(),
                                &candidate,
                            )
                            .ok()?;

                /*
                 * Exact duplicate suppression only.
                 *
                 * No semantic deduplication, no action-only collapse,
                 * no re-ranking and no re-ordering.
                 */
                if authorized
                    .iter()
                    .any(
                        |existing| {
                            existing
                                .candidate()
                                == grounded
                                    .candidate()
                        },
                    )
                {
                    continue;
                }

                provenance.push(
                    (
                        grounded.clone(),
                        expected_experiment_source_state
                            .clone(),
                    ),
                );

                authorized.push(
                    grounded,
                );
            }
        }

        /*
         * C16I-F final authority:
         *
         * exploitation + native M50 candidates enter ONE common M48
         * evaluation.  There is no second selector and no local utility
         * authority in the ARC adapter.
         */
        let selected =
            Self::selected_authorized_executive_candidate(
                &authorized,
                goal,
                executive_policy,
            )?;

        let source_state =
            provenance
                .iter()
                .find_map(
                    |(
                        candidate,
                        source_state,
                    )| {
                        (
                            candidate
                                == &selected
                        )
                            .then(
                                || {
                                    source_state
                                        .clone()
                                },
                            )
                    },
                )?;

        Some(
            ArcAgi3UnifiedExecutiveAuthority::
                new(
                    source_state,
                    selected,
                ),
        )
    }

    pub fn current_successor_informed_unified_executive_action_selection(
        &self,
        request:
            ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'_>,
    ) -> Option<crate::ArcAgi3Action> {
        self
            .current_successor_informed_unified_executive_authority(
                request,
            )
            .map(
                |authority| {
                    authority.action()
                },
            )
    }

    pub fn current_unified_executive_authority(
        &self,
        exploitation_actions: &[crate::ArcAgi3Action],
        goal: &athlesia_executive_agency::ExecutiveGoal,
        goal_alignment: CognitiveSignal,
        exploitation_execution_cost: CognitiveSignal,
        experiment_authority: Option<ArcAgi3ExperimentDispatchAuthority<'_>>,
        policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
    ) -> Option<ArcAgi3UnifiedExecutiveAuthority> {
        let mut authorized = self.current_model_grounded_authorized_candidates(
            exploitation_actions,
            goal,
            goal_alignment,
            exploitation_execution_cost,
        );

        let mut provenance = Vec::<(
            crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate,
            CognitiveStructure,
        )>::new();

        /*
         * Exploitation provenance comes from the retained grounded
         * state itself. No caller supplies a cognitive source state.
         */
        if !authorized.is_empty() {
            let grounded_state = self.current_grounded_world_state()?;

            let source_state = CognitiveStructure::unordered(grounded_state.facts().to_vec())
                .expect("retained grounded world state contains facts");

            provenance.extend(
                authorized
                    .iter()
                    .cloned()
                    .map(|candidate| (candidate, source_state.clone())),
            );
        }

        /*
         * M50 remains experiment authority only.
         *
         * A continuing real experiment contributes its grounded
         * candidate to the SAME M48 frontier. Its exact source state
         * is retained separately so live feedback cannot reinterpret
         * M50 provenance after selection.
         */
        if let Some(experiment_authority) =
            experiment_authority.filter(|authority| authority.result().continuing())
        {
            let experimentation = experiment_authority.result();

            let expected_experiment_source_state = experiment_authority.expected_source_state();

            if let Some(proposal) = experimentation.next_experiment() {
                if let Ok(candidate) =
                    crate::action_grounding_bridge::
                        ArcAgi3ActionGroundingBridge::
                        ground_experiment_for_goal(
                            self.observation(),
                            expected_experiment_source_state,
                            goal,
                            goal_alignment,
                            proposal,
                        )
                {
                    if let Ok(grounded) =
                        crate::action_grounding_bridge::
                            ArcAgi3ActionGroundingBridge::
                            authorize_executive_candidate(
                                self.observation(),
                                &candidate,
                            )
                    {
                        if !authorized.iter().any(
                            |existing| {
                                existing.candidate()
                                    == grounded.candidate()
                            },
                        ) {
                            provenance.push((
                                grounded.clone(),
                                expected_experiment_source_state
                                    .clone(),
                            ));

                            authorized.push(
                                grounded,
                            );
                        }
                    }
                }
            }
        }

        /*
         * Frozen C1 authority:
         *
         * ONE M48 evaluation over exploitation + experimentation.
         * No live-layer ranking and no protocol-defined utility.
         */
        let selected = Self::selected_authorized_executive_candidate(&authorized, goal, policy)?;

        let source_state = provenance.iter().find_map(|(candidate, source_state)| {
            (candidate == &selected).then(|| source_state.clone())
        })?;

        Some(ArcAgi3UnifiedExecutiveAuthority::new(
            source_state,
            selected,
        ))
    }

    pub fn current_unified_executive_action_selection(
        &self,
        exploitation_actions: &[crate::ArcAgi3Action],
        goal: &athlesia_executive_agency::ExecutiveGoal,
        goal_alignment: CognitiveSignal,
        exploitation_execution_cost: CognitiveSignal,
        experiment_authority: Option<ArcAgi3ExperimentDispatchAuthority<'_>>,
        policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
    ) -> Option<crate::ArcAgi3Action> {
        self.current_unified_executive_authority(
            exploitation_actions,
            goal,
            goal_alignment,
            exploitation_execution_cost,
            experiment_authority,
            policy,
        )
        .map(|authority| authority.action())
    }

    pub fn observation(&self) -> &ArcAgi3Observation {
        self.session.observation()
    }

    pub fn next_perceptual_observation_index(&self) -> u64 {
        self.perception.next_observation_index()
    }

    pub fn begin_reset(
        &mut self,
    ) -> Result<ArcAgi3SessionCommand, ArcAgi3CognitiveInteractionError> {
        self.session.begin_reset().map_err(Into::into)
    }

    pub fn begin_unified_executive_authority(
        &mut self,
        authority: &ArcAgi3UnifiedExecutiveAuthority,
    ) -> Result<ArcAgi3SessionCommand, ArcAgi3CognitiveInteractionError> {
        let command = self.session.begin_unified_executive_action(
            authority.source_state(),
            authority.cognitive_action(),
        )?;

        debug_assert_eq!(command.action(), authority.action(),);

        Ok(command)
    }

    pub fn begin_from_orchestration_result(
        &mut self,
        orchestration: OnlineCognitiveOrchestrationResult,
    ) -> Result<ArcAgi3CognitiveInteractionStep, ArcAgi3CognitiveInteractionError> {
        let dispatch_result = EnvironmentInteractionBoundary::dispatch(&orchestration);

        if dispatch_result.status() != EnvironmentActionDispatchStatus::Ready {
            return Err(ArcAgi3CognitiveInteractionError::DispatchRejected(
                dispatch_result.status(),
            ));
        }

        let dispatch = dispatch_result
            .dispatch()
            .cloned()
            .ok_or(ArcAgi3CognitiveInteractionError::ReadyDispatchMissing)?;

        let command = self.session.begin_dispatch(&dispatch)?;

        Ok(ArcAgi3CognitiveInteractionStep {
            orchestration,
            dispatch,
            command,
        })
    }

    pub fn run_and_begin(
        &mut self,
        anchor_state: &CognitiveStructure,
        input: OnlineCognitiveOrchestrationInput<'_>,
        cycle_policy: IntegratedAgentPolicy,
        transition_request: &CognitiveCycleStateTransitionRequest,
    ) -> Result<ArcAgi3CognitiveInteractionStep, ArcAgi3CognitiveInteractionError> {
        if self.session.has_pending_command() {
            return Err(ArcAgi3InteractiveSessionError::PendingCommandExists.into());
        }

        let orchestration = OnlineCognitiveOrchestration::run(
            anchor_state,
            input,
            cycle_policy,
            transition_request,
        );

        self.begin_from_orchestration_result(orchestration)
    }

    pub fn complete_environment_turn(
        &mut self,
        observation: ArcAgi3Observation,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3CognitiveInteractionCompletion, ArcAgi3CognitiveInteractionError> {
        /*
         * Transactional rule:
         *
         * 1. clone session;
         * 2. validate/bind the real environment response on the clone;
         * 3. project the exact perceptual response;
         * 4. clone retained cognition;
         * 5. bind executive feedback only to the explicit causal
         *    previous-frame -> first-response-frame transition;
         * 6. update retained perceptual evidence on the cognitive clone;
         * 7. only then commit session + perception + cognition together.
         *
         * Protocol RESET turns carry no executive cognitive feedback and
         * therefore do not contaminate retained causal perceptual evidence.
         *
         * A failed environment response or failed perceptual projection
         * cannot partially advance any retained runtime state.
         */
        let previous_best_scene = self.current_best_scene_interpretation();

        let mut next_session = self.session.clone();

        let completed_turn = next_session.complete_turn(observation.clone(), confidence)?;

        let next_perception = ArcAgi3PerceptualIngestionBridge::project_observation(
            &observation,
            self.perception.next_observation_index(),
            Some(self.perception.latest_frame()),
        )?;

        let mut next_cognition = self.cognition.clone();

        if completed_turn.has_cognitive_feedback() {
            if let Some(causal_transition) = next_perception.causal_environment_transition() {
                let max_proposals_per_frame = causal_transition
                    .previous_frame()
                    .element_count()
                    .max(causal_transition.current_frame().element_count());

                if let Some(observation_result) =
                    ArcAgi3PerceptualIngestionBridge::atomic_transition_evidence(
                        causal_transition.previous_frame(),
                        causal_transition.current_frame(),
                        max_proposals_per_frame,
                    )
                {
                    /*
                     * Epistemic anti-self-confirmation rule:
                     *
                     * Grouping candidates are derived from temporal evidence
                     * retained BEFORE this environment consequence.
                     *
                     * The newly observed consequence may then validate or
                     * contradict those already-eligible candidates.
                     *
                     * Only after behavior evidence is retained do we admit
                     * this transition into atomic temporal history.
                     */
                    let grouping_frontier =
                        ArcAgi3PerceptualIngestionBridge::
                            temporally_supported_grid_grouping_candidates(
                                next_cognition
                                    .perceptual_temporal_evidence(),
                                causal_transition.current_frame(),
                                Self::live_temporal_grouping_policy(),
                                Self::live_grouping_generation_policy(),
                            );

                    let grouping_behavior =
                        athlesia_core_knowledge_perceptual_grounding::
                            PerceptualGroupingBehaviorObservation::observe(
                                grouping_frontier.candidates(),
                                &observation_result,
                            );

                    next_cognition.retain_perceptual_grouping_behavior_result(&grouping_behavior);

                    let grouping_appearance =
                        ArcAgi3PerceptualIngestionBridge::grouping_appearance_observation(
                            causal_transition.current_frame(),
                            grouping_frontier.candidates(),
                        );

                    next_cognition
                        .retain_perceptual_grouping_appearance_result(&grouping_appearance);

                    next_cognition.retain_perceptual_observation_result(&observation_result);

                    /*
                     * The current scene is derived from a temporary immutable
                     * view of the fully updated cognitive clone.
                     *
                     * Nothing is committed to self until all projection and
                     * learning work succeeds.
                     */
                    let current_best_scene = {
                        let next_runtime_view = Self {
                            session: next_session.clone(),
                            perception: next_perception.clone(),
                            cognition: next_cognition.clone(),
                        };

                        next_runtime_view.current_best_scene_interpretation()
                    };

                    if let (Some(previous_scene), Some(current_scene), Some(environment_evidence)) = (
                        previous_best_scene.as_ref().cloned(),
                        current_best_scene,
                        completed_turn.evidence(),
                    ) {
                        let candidates =
                            athlesia_core_knowledge_perceptual_grounding::
                                IntegratedPerceptualWorldCandidates::new(
                                    vec![previous_scene],
                                    vec![current_scene],
                                    Vec::new(),
                                    Vec::new(),
                                    Vec::new(),
                                    Vec::new(),
                                );

                        if let Some(world_input) =
                            athlesia_core_knowledge_perceptual_grounding::
                                IntegratedPerceptualWorldInput::new(
                                    causal_transition
                                        .previous_frame()
                                        .clone(),
                                    causal_transition
                                        .current_frame()
                                        .clone(),
                                    candidates,
                                )
                        {
                            /*
                             * C3E-B anti-retrocausality:
                             *
                             * Capture the epistemic question from the
                             * PRE-learning retained owner.  Only after that
                             * may this real transition update M47.
                             */
                            let progress_episode =
                                athlesia_integrated_cognitive_agent::
                                    PerceptualDomainLearningEvidenceBridge::
                                        derive(
                                            &world_input,
                                            Self::live_perceptual_world_context(),
                                            environment_evidence,
                                        )
                                        .episode()
                                        .cloned();

                            let pre_learning_possibility =
                                progress_episode
                                    .as_ref()
                                    .and_then(|episode| {
                                        next_cognition
                                            .current_m50_epistemic_possibility(
                                                episode.before(),
                                                episode.transformation(),
                                                athlesia_universal_domain_learning::
                                                    GroundedExplanatoryVersionSpacePolicy::
                                                        new(
                                                            1,
                                                            64,
                                                            512,
                                                            256,
                                                        )
                                                        .expect(
                                                            "live epistemic version bounds are positive",
                                                        ),
                                            )
                                    });

                            next_cognition
                                .observe_environment_transition(
                                    &world_input,
                                    Self::live_perceptual_world_context(),
                                    environment_evidence,
                                    Self::live_transition_schema_learning_policy(),
                                );

                            if let (
                                Some(episode),
                                Some(pre_learning),
                            ) = (
                                progress_episode.as_ref(),
                                pre_learning_possibility.as_ref(),
                            ) {
                                let discrimination_policy =
                                    athlesia_autonomous_active_experimentation::
                                        EpistemicForecastDiscriminationPolicy::
                                            new(
                                                512,
                                                512,
                                            )
                                            .expect(
                                                "live epistemic discrimination bounds are positive",
                                            );

                                let pre_discrimination =
                                    athlesia_autonomous_active_experimentation::
                                        AutonomousEpistemicForecastDiscrimination::
                                            evaluate(
                                                pre_learning,
                                                discrimination_policy,
                                            );

                                /*
                                 * Only genuinely unresolved pre-action
                                 * questions are empirical experimentation
                                 * progress evidence.
                                 */
                                if pre_discrimination.informative() {
                                    if let Some(post_learning) =
                                        next_cognition
                                            .current_m50_epistemic_possibility(
                                                episode.before(),
                                                episode.transformation(),
                                                athlesia_universal_domain_learning::
                                                    GroundedExplanatoryVersionSpacePolicy::
                                                        new(
                                                            1,
                                                            64,
                                                            512,
                                                            256,
                                                        )
                                                        .expect(
                                                            "live post-learning version bounds are positive",
                                                        ),
                                            )
                                    {
                                        if let Some(realized_outcome) =
                                            next_cognition
                                                .resolve_m50_epistemic_possibility_against_transition(
                                                    pre_learning,
                                                    episode.before(),
                                                    episode.after(),
                                                    episode.transformation(),
                                                    athlesia_autonomous_active_experimentation::
                                                        EpistemicOutcomeResolutionPolicy::
                                                            new(
                                                                512,
                                                                512,
                                                            )
                                                            .expect(
                                                                "live outcome-resolution bounds are positive",
                                                            ),
                                                )
                                        {
                                            let progress =
                                                athlesia_autonomous_active_experimentation::
                                                    AutonomousEpistemicResolutionProgress::
                                                        measure(
                                                            pre_learning,
                                                            &realized_outcome,
                                                            &post_learning,
                                                            discrimination_policy,
                                                        );

                                            if let Some(sample) =
                                                progress.sample().cloned()
                                            {
                                                next_cognition
                                                    .retain_epistemic_transfer_progress_event(
                                                        completed_turn.event_index(),
                                                        pre_learning,
                                                        sample.clone(),
                                                        athlesia_autonomous_active_experimentation::
                                                            EmpiricalEpistemicTransferIdentityPolicy::
                                                                new(512)
                                                                .expect(
                                                                    "positive live transfer identity frontier",
                                                                ),
                                                        athlesia_integrated_cognitive_agent::
                                                            RetainedEpistemicTransferProgressHistoryPolicy::
                                                                new(256)
                                                                .expect(
                                                                    "positive live transfer history frontier",
                                                                ),
                                                    );

                                                next_cognition
                                                    .retain_epistemic_progress_event(
                                                        completed_turn.event_index(),
                                                        sample,
                                                        athlesia_integrated_cognitive_agent::
                                                            RetainedEpistemicProgressHistoryPolicy::
                                                                new(
                                                                    256,
                                                                )
                                                                .expect(
                                                                    "live retained progress frontier is positive",
                                                                ),
                                                    );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.session = next_session;
        self.perception = next_perception.clone();
        self.cognition = next_cognition;

        Ok(ArcAgi3CognitiveInteractionCompletion {
            turn: completed_turn,
            perception: next_perception,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3CognitiveInteractionRuntime;

impl UniversalArcAgi3CognitiveInteractionRuntime {
    pub fn create_runtime(
        initial_observation: ArcAgi3Observation,
        first_perceptual_observation_index: u64,
    ) -> Result<ArcAgi3CognitiveInteractionRuntime, ArcAgi3CognitiveInteractionError> {
        ArcAgi3CognitiveInteractionRuntime::new(
            initial_observation,
            first_perceptual_observation_index,
        )
    }

    pub fn complete_environment_turn(
        runtime: &mut ArcAgi3CognitiveInteractionRuntime,
        observation: ArcAgi3Observation,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3CognitiveInteractionCompletion, ArcAgi3CognitiveInteractionError> {
        runtime.complete_environment_turn(observation, confidence)
    }
}


#[cfg(test)]
mod c16i_successor_informed_two_contract_e2e_tests {
    use super::*;
    mod m51_fixture {
        use crate as athlesia_arc_agi_3_adapter;

        include!(
            "../tests/support/m51_online_orchestration_fixture.rs"
        );
    }

    fn signal(
        value: u16,
    ) -> athlesia_mindstone_sparse_cognition::CognitiveSignal {
        athlesia_mindstone_sparse_cognition::
            CognitiveSignal::new(value)
            .unwrap()
    }

    fn atom(
        value: u64,
    ) -> athlesia_mindstone_sparse_cognition::CognitiveStructure {
        athlesia_mindstone_sparse_cognition::
            CognitiveStructure::atom(value)
    }

    fn action(
        id: crate::ArcAgi3ActionId,
    ) -> crate::ArcAgi3Action {
        crate::ArcAgi3Action::discrete(id)
            .unwrap()
    }

    fn object_grid(
        value: u8,
    ) -> crate::ArcAgi3Grid {
        crate::ArcAgi3Grid::from_rows(
            vec![
                vec![value, value],
                vec![8, 9],
            ],
        )
        .unwrap()
    }

    fn observation(
        game: &str,
        value: u8,
        last_action:
            Option<crate::ArcAgi3Action>,
    ) -> crate::ArcAgi3Observation {
        crate::ArcAgi3Observation::new(
            crate::ArcAgi3GameId::new(
                game.to_string(),
            )
            .unwrap(),
            crate::ArcAgi3GameState::NotFinished,
            crate::ArcAgi3FrameSequence::new(
                vec![
                    object_grid(value),
                ],
            )
            .unwrap(),
            0,
            3,
            crate::ArcAgi3AvailableActions::new(
                vec![
                    crate::ArcAgi3ActionId::Action1,
                    crate::ArcAgi3ActionId::Action2,
                ],
            )
            .unwrap(),
            last_action,
        )
    }

    fn training_turn(
        runtime:
            &mut ArcAgi3CognitiveInteractionRuntime,
        game: &str,
        selected_action: crate::ArcAgi3Action,
        value: u8,
    ) {
        let cognitive_action =
            crate::cognitive_protocol_bridge::
                ArcAgi3CognitiveProtocolBridge::
                    encode_action(
                        selected_action,
                    );

        m51_fixture::begin_arc(
            runtime,
            cognitive_action,
        )
        .expect(
            "C16I E2E training action must begin",
        );

        let completion =
            runtime
                .complete_environment_turn(
                    observation(
                        game,
                        value,
                        Some(selected_action),
                    ),
                    signal(900),
                )
                .expect(
                    "C16I E2E real consequence must commit",
                );

        assert!(
            completion.has_cognitive_feedback(),
            "training turn must be a genuine causal environment event",
        );
    }

    fn mature_runtime(
        runtime:
            &mut ArcAgi3CognitiveInteractionRuntime,
        game: &str,
    ) {
        let action_one =
            action(
                crate::ArcAgi3ActionId::Action1,
            );

        let action_two =
            action(
                crate::ArcAgi3ActionId::Action2,
            );

        for value in [
            2_u8,
            3,
            4,
            5,
        ] {
            training_turn(
                runtime,
                game,
                action_one,
                value,
            );
        }

        for (
            selected,
            value,
        ) in [
            (action_two, 5_u8),
            (action_one, 6_u8),
            (action_two, 6_u8),
            (action_one, 5_u8),
            (action_two, 5_u8),
            (action_one, 6_u8),
            (action_two, 6_u8),
            (action_one, 5_u8),
        ] {
            training_turn(
                runtime,
                game,
                selected,
                value,
            );
        }
    }

    fn version_policy(
    ) -> athlesia_universal_domain_learning::
        GroundedExplanatoryVersionSpacePolicy {
        athlesia_universal_domain_learning::
            GroundedExplanatoryVersionSpacePolicy::
                new(
                    1,
                    64,
                    512,
                    256,
                )
                .unwrap()
    }

    fn discrimination_policy(
    ) -> athlesia_autonomous_active_experimentation::
        EpistemicForecastDiscriminationPolicy {
        athlesia_autonomous_active_experimentation::
            EpistemicForecastDiscriminationPolicy::
                new(
                    512,
                    512,
                )
                .unwrap()
    }

    fn expectation_policy(
    ) -> athlesia_autonomous_active_experimentation::
        EmpiricalExpectedEpistemicProgressPolicy {
        athlesia_autonomous_active_experimentation::
            EmpiricalExpectedEpistemicProgressPolicy::
                new(
                    256,
                    256,
                    1,
                )
                .unwrap()
    }

    fn priority_policy(
    ) -> athlesia_autonomous_active_experimentation::
        EmpiricalEpistemicActionPriorityPolicy {
        athlesia_autonomous_active_experimentation::
            EmpiricalEpistemicActionPriorityPolicy::
                new(8)
                .unwrap()
    }

    fn proposal_policy(
    ) -> athlesia_autonomous_active_experimentation::
        BeliefDrivenExperimentProposalPolicy {
        use athlesia_autonomous_active_experimentation::{
            ActiveExperimentBounds,
            ActiveExperimentPolicy,
            ActiveExperimentThresholds,
            BeliefDrivenExperimentProposalBounds,
            BeliefDrivenExperimentProposalPolicy,
        };

        BeliefDrivenExperimentProposalPolicy::new(
            ActiveExperimentPolicy::new(
                ActiveExperimentBounds::new(
                    16,
                    16,
                    16,
                )
                .unwrap(),
                ActiveExperimentThresholds::new(
                    signal(500),
                    signal(500),
                    signal(500),
                    signal(500),
                )
                .unwrap(),
            ),
            BeliefDrivenExperimentProposalBounds::new(
                16,
                16,
                16,
                16,
            )
            .unwrap(),
            signal(500),
            signal(500),
        )
        .unwrap()
    }

    fn executive_policy(
    ) -> athlesia_executive_agency::
        ExecutiveAgencyPolicy {
        use athlesia_executive_agency::{
            ExecutiveAgencyPolicy,
            ExecutiveSelectionThresholds,
            ExecutiveUtilityWeights,
        };

        ExecutiveAgencyPolicy::new(
            1,
            8,
            16,
            1,
            ExecutiveUtilityWeights::new(
                0,
                0,
                0,
                1000,
                0,
            )
            .unwrap(),
            ExecutiveSelectionThresholds::new(
                signal(100),
                signal(100),
                signal(1),
                signal(600),
                signal(100),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn goal(
    ) -> athlesia_executive_agency::
        ExecutiveGoal {
        athlesia_executive_agency::
            ExecutiveGoal::new(
                atom(
                    0xC16F_0000_0000_0001,
                ),
                signal(900),
                athlesia_mindstone_sparse_cognition::
                    CognitiveSignal::zero(),
            )
    }

    #[derive(Debug)]
    struct Fixture {
        runtime:
            ArcAgi3CognitiveInteractionRuntime,
        arc_action:
            crate::ArcAgi3Action,
        cognitive_action:
            CognitiveStructure,
        native_source:
            CognitiveStructure,
        native_possibilities:
            Vec<
                athlesia_autonomous_active_experimentation::
                    GroundedExperimentPossibility
            >,
        beliefs:
            Vec<
                athlesia_autonomous_active_experimentation::
                    HypothesisBeliefState
            >,
    }

    fn fixture(
        game: &str,
        first_index: u64,
    ) -> Fixture {
        use athlesia_autonomous_active_experimentation::{
            AutonomousEpistemicForecastDiscrimination,
            AutonomousEpistemicResolutionProgress,
            CompetingHypothesisPrediction,
            GroundedEpistemicExperimentPossibility,
            GroundedExperimentPossibility,
            HypothesisBeliefState,
        };

        let action_one =
            action(
                crate::ArcAgi3ActionId::Action1,
            );

        let action_two =
            action(
                crate::ArcAgi3ActionId::Action2,
            );

        let mut runtime =
            ArcAgi3CognitiveInteractionRuntime::new(
                observation(
                    game,
                    1,
                    None,
                ),
                first_index,
            )
            .unwrap();

        mature_runtime(
            &mut runtime,
            game,
        );

        /*
         * Enter the established informative holdout.
         */
        training_turn(
            &mut runtime,
            game,
            action_two,
            7_u8,
        );

        /*
         * Real B2 sample:
         *
         *     state7 --ACTION1--> state6
         */
        training_turn(
            &mut runtime,
            game,
            action_one,
            6_u8,
        );

        /*
         * Return through a real causal turn to the exact state7
         * representation.  B2 now contains a genuine ACTION1 sample
         * from this source representation.
         */
        training_turn(
            &mut runtime,
            game,
            action_two,
            7_u8,
        );

        let current =
            runtime
                .current_grounded_world_state()
                .expect(
                    "C16I E2E current state7 must be grounded",
                );

        let cognitive_action =
            crate::cognitive_protocol_bridge::
                ArcAgi3CognitiveProtocolBridge::
                    encode_action(
                        action_one,
                    );

        let b2 =
            runtime
                .current_action_qualified_empirical_successor_frequency(
                    &cognitive_action,
                )
                .expect(
                    "B0 current grounding must permit B2 live query",
                );

        assert!(
            b2.is_qualified(),
            "C16I E2E requires genuine action-qualified B2 evidence",
        );

        assert!(
            b2.independent_action_event_count()
                > 0,
            "B2 authority must contain at least one real interaction event",
        );

        assert!(
            b2
                .successor_informed_proposal_eligibility()
                .eligible_as_supplemental_evidence(),
            "real B2 evidence must pass C16I-A integrity qualification",
        );

        /*
         * Current M50 question from the retained M47 owner.
         */
        let current_epistemic =
            runtime
                .cognition
                .current_m50_epistemic_possibility(
                    &current,
                    &cognitive_action,
                    version_policy(),
                )
                .expect(
                    "current state7 ACTION1 must expose an M50 epistemic possibility",
                );

        let current_discrimination =
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &current_epistemic,
                    discrimination_policy(),
                );

        assert!(
            current_discrimination
                .informative(),
            "C16I E2E requires a genuinely unresolved current M50 question",
        );

        assert!(
            current_discrimination
                .pairwise_separation_score()
                > 0,
            "informative M50 question must contain real pairwise separation",
        );

        /*
         * Ground outcome resolution in an actually observed B2
         * successor representation.  No target-occurrence booleans are
         * invented by this fixture.
         */
        let real_successor =
            b2
                .successor_frequencies()
                .first()
                .expect(
                    "qualified B2 result must expose a real successor",
                )
                .successor_representation()
                .clone();

        let realized_outcome =
            runtime
                .cognition
                .resolve_m50_epistemic_possibility_against_transition(
                    &current_epistemic,
                    &current,
                    &real_successor,
                    &cognitive_action,
                    athlesia_autonomous_active_experimentation::
                        EpistemicOutcomeResolutionPolicy::
                            new(
                                512,
                                512,
                            )
                            .unwrap(),
                )
                .expect(
                    "real B2 successor must resolve the current M50 question",
                );

        assert!(
            realized_outcome.resolved(),
            "C3D measurement requires a resolved empirical outcome",
        );

        /*
         * C3D measurement fixture:
         *
         * Preserve source/action identity and use a strict subset of the
         * already-existing forecast frontier as the post-learning
         * comparison.  The progress VALUE is therefore produced only by
         * AutonomousEpistemicResolutionProgress::measure.
         *
         * No expected progress/EIG number is manually inserted.
         */
        let post_learning =
            GroundedEpistemicExperimentPossibility::new(
                current_epistemic
                    .source_state()
                    .clone(),
                current_epistemic
                    .action()
                    .clone(),
                vec![
                    current_epistemic
                        .forecasts()
                        .first()
                        .expect(
                            "informative possibility must contain forecasts",
                        )
                        .clone(),
                ],
            )
            .unwrap();

        let progress =
            AutonomousEpistemicResolutionProgress::
                measure(
                    &current_epistemic,
                    &realized_outcome,
                    &post_learning,
                    discrimination_policy(),
                );

        assert!(
            progress.measured(),
            "C16I E2E C3F evidence must come from the real C3D measurement authority",
        );

        let sample =
            progress
                .sample()
                .expect(
                    "measured progress must contain one exact sample",
                )
                .clone();

        assert!(
            sample
                .realized_separation_reduction()
                > sample
                    .realized_separation_increase(),
            "fixture must provide positive measured epistemic progress for C3F",
        );

        /*
         * Retain through M51's existing bounded history owner.
         *
         * The high event id is fixture provenance only; it does not
         * participate in matching or priority.
         */
        let retained =
            runtime
                .cognition
                .retain_epistemic_progress_event(
                    u64::MAX - 16,
                    sample,
                    athlesia_integrated_cognitive_agent::
                        RetainedEpistemicProgressHistoryPolicy::
                            new(256)
                            .unwrap(),
                );

        assert!(
            retained.retained(),
            "measured C3D sample must be retained by the existing M51 owner",
        );

        let priority =
            runtime
                .cognition
                .current_empirical_epistemic_action_priority_frontier(
                    &current,
                    std::slice::from_ref(
                        &cognitive_action,
                    ),
                    version_policy(),
                    discrimination_policy(),
                    expectation_policy(),
                    priority_policy(),
                );

        assert_eq!(
            priority.status(),
            athlesia_autonomous_active_experimentation::
                EmpiricalEpistemicActionPriorityStatus::
                    Ranked,
            "C16I E2E requires measured positive C3F authority before native M50 gating",
        );

        let priority_candidate =
            priority
                .best()
                .expect(
                    "Ranked C3F frontier must expose one exact candidate",
                );

        assert_eq!(
            priority_candidate.action(),
            &cognitive_action,
        );

        /*
         * C16I-C requires caller-native M50 source/action identity to
         * match the already-ranked C3F binding exactly.
         */
        let native_source =
            priority_candidate
                .source_state()
                .clone();

        let hypothesis_one =
            atom(
                0xC16F_0000_0000_0101,
            );

        let hypothesis_two =
            atom(
                0xC16F_0000_0000_0102,
            );

        let native_possibilities =
            vec![
                GroundedExperimentPossibility::new(
                    native_source.clone(),
                    cognitive_action.clone(),
                    vec![
                        CompetingHypothesisPrediction::new(
                            hypothesis_one.clone(),
                            atom(
                                0xC16F_0000_0000_0201,
                            ),
                            signal(900),
                        )
                        .unwrap(),
                        CompetingHypothesisPrediction::new(
                            hypothesis_two.clone(),
                            atom(
                                0xC16F_0000_0000_0202,
                            ),
                            signal(900),
                        )
                        .unwrap(),
                    ],
                    signal(900),
                    signal(900),
                    signal(100),
                )
                .unwrap(),
            ];

        let beliefs =
            vec![
                HypothesisBeliefState::new(
                    hypothesis_one,
                    signal(820),
                )
                .unwrap(),
                HypothesisBeliefState::new(
                    hypothesis_two,
                    signal(760),
                )
                .unwrap(),
            ];

        /*
         * Anti-vacuity: prove A-D produces a real native M50 proposal
         * before testing F.
         */
        let native_result =
            runtime
                .cognition
                .current_successor_informed_native_m50_proposal_delegation(
                    athlesia_integrated_cognitive_agent::
                        SuccessorInformedNativeM50ProposalDelegationRequest {
                            native_input:
                                athlesia_integrated_cognitive_agent::
                                    SuccessorInformedNativeProposalInputRequest {
                                        state:
                                            &current,
                                        actions:
                                            std::slice::from_ref(
                                                &cognitive_action,
                                            ),
                                        version_policy:
                                            version_policy(),
                                        discrimination_policy:
                                            discrimination_policy(),
                                        expectation_policy:
                                            expectation_policy(),
                                        priority_policy:
                                            priority_policy(),
                                        native_possibilities:
                                            &native_possibilities,
                                    },
                            beliefs:
                                &beliefs,
                            proposal_policy:
                                proposal_policy(),
                        },
                )
                .expect(
                    "real B2 + measured C3F must reach native M50 delegation",
                );

        assert!(
            native_result.generated_count()
                > 0,
            "native M50 must actually generate a proposal before F is exercised",
        );

        Fixture {
            runtime,
            arc_action:
                action_one,
            cognitive_action,
            native_source,
            native_possibilities,
            beliefs,
        }
    }

    #[test]
    fn mismatched_expected_source_state_fails_closed_before_m48_authority(
    ) {
        let fixture =
            fixture(
                "c16i-f-source-mismatch",
                8_100_000,
            );

        let wrong_source =
            atom(
                0xC16F_FFFF_FFFF_FF01,
            );

        assert_ne!(
            wrong_source,
            fixture.native_source,
        );

        let selected =
            fixture
                .runtime
                .current_successor_informed_unified_executive_authority(
                    ArcAgi3SuccessorInformedUnifiedExecutiveRequest {
                        exploitation_actions:
                            &[],
                        goal:
                            &goal(),
                        goal_alignment:
                            signal(900),
                        exploitation_execution_cost:
                            signal(100),
                        expected_experiment_source_state:
                            &wrong_source,
                        native_possibilities:
                            &fixture
                                .native_possibilities,
                        beliefs:
                            &fixture.beliefs,
                        version_policy:
                            version_policy(),
                        discrimination_policy:
                            discrimination_policy(),
                        expectation_policy:
                            expectation_policy(),
                        priority_policy:
                            priority_policy(),
                        proposal_policy:
                            proposal_policy(),
                        executive_policy:
                            executive_policy(),
                    },
                );

        assert!(
            selected.is_none(),
            "exact E-bridge source mismatch must fail closed before any M48 authority can select the experiment",
        );
    }

    #[test]
    fn real_b2_measured_c3f_native_m50_reaches_action_only_through_common_m48(
    ) {
        let fixture =
            fixture(
                "c16i-f-common-m48",
                8_200_000,
            );

        let authority =
            fixture
                .runtime
                .current_successor_informed_unified_executive_authority(
                    ArcAgi3SuccessorInformedUnifiedExecutiveRequest {
                        exploitation_actions:
                            &[],
                        goal:
                            &goal(),
                        goal_alignment:
                            signal(900),
                        exploitation_execution_cost:
                            signal(100),
                        expected_experiment_source_state:
                            &fixture.native_source,
                        native_possibilities:
                            &fixture
                                .native_possibilities,
                        beliefs:
                            &fixture.beliefs,
                        version_policy:
                            version_policy(),
                        discrimination_policy:
                            discrimination_policy(),
                        expectation_policy:
                            expectation_policy(),
                        priority_policy:
                            priority_policy(),
                        proposal_policy:
                            proposal_policy(),
                        executive_policy:
                            executive_policy(),
                    },
                )
                .expect(
                    "eligible native M50 proposal must reach action authority only through the common M48 selector",
                );

        assert_eq!(
            authority.action(),
            fixture.arc_action,
        );

        assert_eq!(
            authority.cognitive_action(),
            &fixture.cognitive_action,
        );

        assert_eq!(
            authority.source_state(),
            &fixture.native_source,
            "selected experiment must preserve exact native M50 source provenance through common M48",
        );

        assert!(
            authority
                .candidate()
                .information_gain()
                > athlesia_mindstone_sparse_cognition::
                    CognitiveSignal::zero(),
            "selected experiment must carry native M50 information authority rather than fabricated adapter utility",
        );
    }
}

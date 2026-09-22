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
    BootstrapCoverageMissingFeedback,
    BootstrapCoverageActionMismatch,
    BootstrapCoverageRetentionRejected(
        athlesia_integrated_cognitive_agent::RetainedBootstrapActionCoverageStatus,
    ),
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
     * These are caller-native M50 inputs.
     *
     * The adapter does not manufacture competing predictions, beliefs,
     * controllability, confidence, EIG, or cost.
     */
    pub native_possibilities:
        &'a [athlesia_autonomous_active_experimentation::GroundedExperimentPossibility],
    pub beliefs: &'a [athlesia_autonomous_active_experimentation::HypothesisBeliefState],

    pub version_policy: athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy,
    pub discrimination_policy:
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy,
    pub expectation_policy:
        athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy,
    pub priority_policy:
        athlesia_autonomous_active_experimentation::EmpiricalEpistemicActionPriorityPolicy,
    pub proposal_policy:
        athlesia_autonomous_active_experimentation::BeliefDrivenExperimentProposalPolicy,
    pub executive_policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
}

// B3C-C1 EVIDENCE-FAITHFUL ARC AUTHORITY
//
// This request contains only:
//
// - a concrete ARC protocol action frontier;
// - exploitation context retained temporarily for the legacy exploitation
//   branch;
// - evidence/resource policies.
//
// It contains NO caller-native:
//
// - GroundedExperimentPossibility;
// - HypothesisBeliefState;
// - BeliefDrivenExperimentProposalPolicy;
// - predicted outcome;
// - confidence for hypotheses;
// - EIG;
// - controllability.
//
// `candidate_actions` are protocol action identities only. They are not
// cognitive predictions or beliefs.
#[derive(Clone, Copy, Debug)]
pub struct ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest<'a> {
    pub candidate_actions: &'a [crate::ArcAgi3Action],

    pub goal: &'a athlesia_executive_agency::ExecutiveGoal,
    pub goal_alignment: CognitiveSignal,
    pub exploitation_execution_cost: CognitiveSignal,

    pub version_policy: athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy,

    pub discrimination_policy:
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy,

    pub expectation_policy:
        athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy,

    pub priority_policy:
        athlesia_autonomous_active_experimentation::EmpiricalEpistemicActionPriorityPolicy,

    pub exploitation_policy: athlesia_executive_agency::ExecutiveAgencyPolicy,

    pub epistemic_policy: athlesia_executive_agency::EpistemicExecutiveSelectionPolicy,

    /*
     * Resource bound only.
     *
     * It does not assign value, confidence, EIG, probability,
     * controllability or predicted outcome.
     */
    pub ignorance_policy: athlesia_executive_agency::IgnoranceExplorationSelectionPolicy,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ArcAgi3UnifiedExecutiveAuthorityKind {
    LegacyGrounded,
    EvidenceFaithfulEpistemic,
    IgnoranceExploration,
    BootstrapIgnoranceExploration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3UnifiedExecutiveAuthority {
    kind: ArcAgi3UnifiedExecutiveAuthorityKind,
    source_state: CognitiveStructure,
    action: crate::ArcAgi3Action,
    cognitive_action: CognitiveStructure,

    legacy_selected: Option<crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate>,

    epistemic_selected:
        Option<athlesia_integrated_cognitive_agent::SelectedSuccessorInformedEpistemicActionIntent>,

    ignorance_selected: Option<athlesia_executive_agency::GroundedIgnoranceExplorationCandidate>,

    bootstrap_ignorance_selected:
        Option<athlesia_executive_agency::BootstrapIgnoranceExplorationCandidate>,
}

impl ArcAgi3UnifiedExecutiveAuthority {
    fn new(
        source_state: CognitiveStructure,
        selected: crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate,
    ) -> Self {
        let action = selected.action();

        let cognitive_action = selected.candidate().action().clone();

        Self {
            kind: ArcAgi3UnifiedExecutiveAuthorityKind::LegacyGrounded,
            source_state,
            action,
            cognitive_action,
            legacy_selected: Some(selected),
            epistemic_selected: None,
            ignorance_selected: None,
            bootstrap_ignorance_selected: None,
        }
    }

    fn new_epistemic(
        action: crate::ArcAgi3Action,
        selected:
            athlesia_integrated_cognitive_agent::
                SelectedSuccessorInformedEpistemicActionIntent,
    ) -> Self {
        let source_state = selected.source_state().clone();

        let cognitive_action = selected.action().clone();

        Self {
            kind: ArcAgi3UnifiedExecutiveAuthorityKind::EvidenceFaithfulEpistemic,
            source_state,
            action,
            cognitive_action,
            legacy_selected: None,
            epistemic_selected: Some(selected),
            ignorance_selected: None,
            bootstrap_ignorance_selected: None,
        }
    }

    fn new_ignorance(
        action: crate::ArcAgi3Action,
        selected: athlesia_executive_agency::GroundedIgnoranceExplorationCandidate,
    ) -> Self {
        let source_state = selected.source_state().clone();

        let cognitive_action = selected.action().clone();

        Self {
            kind: ArcAgi3UnifiedExecutiveAuthorityKind::IgnoranceExploration,
            source_state,
            action,
            cognitive_action,
            legacy_selected: None,
            epistemic_selected: None,
            ignorance_selected: Some(selected),
            bootstrap_ignorance_selected: None,
        }
    }

    fn new_bootstrap_ignorance(
        action: crate::ArcAgi3Action,
        selected: athlesia_executive_agency::BootstrapIgnoranceExplorationCandidate,
    ) -> Self {
        let source_state = selected.observation_identity().clone();

        let cognitive_action = selected.action().clone();

        Self {
            kind: ArcAgi3UnifiedExecutiveAuthorityKind::BootstrapIgnoranceExploration,
            source_state,
            action,
            cognitive_action,
            legacy_selected: None,
            epistemic_selected: None,
            ignorance_selected: None,
            bootstrap_ignorance_selected: Some(selected),
        }
    }

    pub fn kind(&self) -> ArcAgi3UnifiedExecutiveAuthorityKind {
        self.kind
    }

    pub fn action(&self) -> crate::ArcAgi3Action {
        self.action
    }

    pub fn cognitive_action(&self) -> &CognitiveStructure {
        &self.cognitive_action
    }

    /*
     * A concrete predicted outcome exists only on the frozen legacy
     * candidate representation.
     *
     * Evidence-faithful epistemic and ignorance-exploration authority
     * deliberately return None.
     */
    pub fn predicted_outcome(&self) -> Option<&CognitiveStructure> {
        self.legacy_selected
            .as_ref()
            .map(|selected| selected.candidate().predicted_outcome())
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn legacy_candidate(
        &self,
    ) -> Option<&athlesia_executive_agency::GroundedExecutiveActionCandidate> {
        self.legacy_selected
            .as_ref()
            .map(|selected| selected.candidate())
    }

    pub fn epistemic_selection(
        &self,
    ) -> Option<&athlesia_integrated_cognitive_agent::SelectedSuccessorInformedEpistemicActionIntent>
    {
        self.epistemic_selected.as_ref()
    }

    pub fn ignorance_selection(
        &self,
    ) -> Option<&athlesia_executive_agency::GroundedIgnoranceExplorationCandidate> {
        self.ignorance_selected.as_ref()
    }

    pub fn bootstrap_ignorance_selection(
        &self,
    ) -> Option<&athlesia_executive_agency::BootstrapIgnoranceExplorationCandidate> {
        self.bootstrap_ignorance_selected.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3CognitiveInteractionRuntime {
    session: ArcAgi3InteractiveSession,
    perception: ArcAgi3PerceptualProjection,
    cognition: athlesia_integrated_cognitive_agent::OnlinePersistentCognitiveState,

    pending_bootstrap_coverage_action: Option<CognitiveStructure>,
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
            pending_bootstrap_coverage_action: None,
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

    /*
     * Competing perceptual proposal families.
     *
     * 1. retained temporal adjacency proposals;
     * 2. current appearance-coherent maximal connected components.
     *
     * Neither family owns objecthood authority. Both merely provide
     * grounded candidates for later evidence gates.
     */
    /*
     * Proposal kind is discovery provenance, not physical grouping
     * identity.
     *
     * For the objecthood path, equal member sets require one stable
     * representative across time. Appearance-connected grouping is
     * preferred because it can exist before temporal relation evidence
     * matures; choosing it prevents retained appearance history from
     * changing identity when PairwiseRelation appears later.
     */
    fn compare_objecthood_grouping_candidates(
        left: &athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate,
        right: &athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate,
    ) -> std::cmp::Ordering {
        use athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidateKind;

        fn provenance_rank(kind: PerceptualGroupingCandidateKind) -> u8 {
            match kind {
                PerceptualGroupingCandidateKind::ConnectedComponent => 0,

                PerceptualGroupingCandidateKind::PairwiseRelation => 1,
            }
        }

        left.members()
            .cmp(right.members())
            .then_with(|| provenance_rank(left.kind()).cmp(&provenance_rank(right.kind())))
    }

    pub fn current_perceptual_grouping_candidates(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate> {
        let mut candidates = self
            .current_perceptual_grouping_frontier(
                Self::live_temporal_grouping_policy(),
                Self::live_grouping_generation_policy(),
            )
            .candidates()
            .to_vec();

        candidates.extend(
            ArcAgi3PerceptualIngestionBridge::appearance_coherent_grid_grouping_candidates(
                self.perception.latest_frame(),
            ),
        );

        /*
         * Proposal family is provenance, not perceptual identity.
         *
         * The same physical grouping may be discovered independently as
         * a temporal PairwiseRelation and as an appearance
         * ConnectedComponent.
         *
         * Downstream objecthood identity is the canonical member set, so
         * equivalent proposals must not become duplicate object candidates.
         *
         * Sorting remains deterministic. When both proposal families expose
         * the same member set, the canonical ordering selects one stable
         * representative without fabricating or discarding membership.
         */
        candidates.sort_by(Self::compare_objecthood_grouping_candidates);

        candidates.dedup_by(|left, right| left.members() == right.members());

        candidates
    }

    pub fn current_empirically_coherent_groupings(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate> {
        self.cognition.current_empirically_coherent_groupings(
            self.perception.latest_frame(),
            Self::live_grouping_behavior_retention_policy(),
        )
    }

    pub fn current_objecthood_eligible_groupings(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate> {
        use athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingAppearanceObservationEvidence;

        let frame = self.perception.latest_frame();

        /*
         * ARC-specific responsibility ends at reporting exact current
         * appearance observations for evidence-neutral grouping proposals.
         *
         * The adapter does not decide objecthood and does not fabricate
         * common-change support.
         */
        let visual_observations = self
            .current_perceptual_grouping_candidates()
            .into_iter()
            .filter_map(|grouping| {
                let (appearance_cohesion, contrast_boundary) =
                    ArcAgi3PerceptualIngestionBridge::grouping_visual_objecthood_evidence(
                        frame, &grouping,
                    )?;

                Some(PerceptualGroupingAppearanceObservationEvidence::new(
                    grouping,
                    appearance_cohesion,
                    contrast_boundary,
                ))
            })
            .collect::<Vec<_>>();

        self.cognition
            .current_objecthood_eligible_groupings_from_visual_observations(
                &visual_observations,
                Self::live_temporal_grouping_policy(),
                Self::live_grouping_behavior_retention_policy(),
            )
    }

    pub fn current_provisional_object_hypotheses(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::ObjectHypothesis> {
        let groupings = self.current_objecthood_eligible_groupings();

        self.cognition
            .current_provisional_object_hypotheses_from_groupings(
                self.perception.latest_frame(),
                &groupings,
                Self::live_grouping_appearance_retention_policy(),
                Self::live_grouping_behavior_retention_policy(),
            )
    }

    fn current_grouping_behavior_candidates(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate> {
        let mut candidates = self
            .current_perceptual_grouping_frontier(
                Self::live_temporal_grouping_policy(),
                Self::live_grouping_generation_policy(),
            )
            .candidates()
            .to_vec();
        candidates.extend(self.current_objecthood_eligible_groupings());

        // Observe each physical member-set once per action, even when both
        // temporal and appearance proposals already identify it.
        candidates.sort();
        candidates.dedup_by(|left, right| left.members() == right.members());

        for candidate in &mut candidates {
            // Continue one exact retained history when discovery provenance
            // changes. This does not merge counts or rewrite other histories.
            if let Some(record) = self
                .cognition
                .perceptual_grouping_behavior_evidence()
                .records()
                .iter()
                .filter(|record| record.candidate().members() == candidate.members())
                .max_by(|left, right| {
                    left.observation_count()
                        .cmp(&right.observation_count())
                        .then_with(|| right.candidate().cmp(left.candidate()))
                })
            {
                *candidate = record.candidate().clone();
            }
        }

        candidates
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

    fn current_scene_candidates(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::SceneInterpretation> {
        let hypotheses = self.current_provisional_object_hypotheses();

        /*
         * ARC-specific responsibility:
         *
         * the projected PerceptualFrame contains one protocol geometry
         * element in addition to the actual perceptual grid cells.
         *
         * Core scene construction remains domain-neutral and therefore
         * receives that non-scene handle explicitly rather than learning
         * ARC handle semantics.
         */
        athlesia_core_knowledge_perceptual_grounding::
            SceneInterpretationConstruction::
                evaluate_hypotheses(
                    self.perception.latest_frame(),
                    &hypotheses,
                    &[
                        ArcAgi3PerceptualIngestionBridge::
                            geometry_handle(),
                    ],
                    Self::live_scene_grounding_policy(),
                )
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
        let competition = self.current_competing_scene_interpretations();

        competition.unique_selected_scene().cloned()
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

    fn live_bootstrap_action_coverage_policy(
    ) -> athlesia_integrated_cognitive_agent::RetainedBootstrapActionCoveragePolicy {
        athlesia_integrated_cognitive_agent::RetainedBootstrapActionCoveragePolicy::new(
            crate::production_action_frontier::ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
        )
        .expect("production bootstrap coverage bound is positive")
    }

    fn current_evidence_neutral_bootstrap_observation_identity(
        &self,
    ) -> Option<CognitiveStructure> {
        /*
         * Absolute cold-start provenance.
         *
         * This is NOT a GroundedStateSnapshot and carries no object,
         * scene, causal, value, probability or predictive semantics.
         *
         * Cell signatures already preserve x/y/value exactly. The
         * explicit outer tag keeps this provenance identity structurally
         * distinct from every normal grounded execution-state identity.
         */
        const BOOTSTRAP_OBSERVATION_TAG: u64 = 0x4234_4135_414F_4253; // "B4A5AOBS"

        let facts =
            athlesia_core_knowledge_perceptual_grounding::
                GroundedPerceptualStateProjector::
                    directly_observed_frame_facts(
                        self.perception.latest_frame(),
                        &[
                            ArcAgi3PerceptualIngestionBridge::
                                geometry_handle(),
                        ],
                    )?;

        let observed = CognitiveStructure::unordered(facts)?;

        Some(CognitiveStructure::Ordered(vec![
            CognitiveStructure::atom(BOOTSTRAP_OBSERVATION_TAG),
            observed,
        ]))
    }

    pub fn current_grounded_world_state(
        &self,
    ) -> Option<athlesia_universal_domain_learning::GroundedStateSnapshot> {
        let competition = self.current_competing_scene_interpretations();

        let current_facts =
            athlesia_core_knowledge_perceptual_grounding::
                GroundedPerceptualStateProjector::
                    unique_selected_scene_facts(
                        self.perception.latest_frame(),
                        &competition,
                    )?;

        athlesia_universal_domain_learning::GroundedStateSnapshot::new(current_facts)
    }

    pub fn current_action_qualified_empirical_successor_frequency(
        &self,
        action: &CognitiveStructure,
    ) -> Option<athlesia_integrated_cognitive_agent::ActionQualifiedEmpiricalSuccessorFrequency>
    {
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
        let current_representation = self.current_grounded_world_state()?;

        Some(
            self.cognition()
                .transition_schema_learning()
                .action_qualified_empirical_successor_frequency(&current_representation, action),
        )
    }

    pub fn current_executable_world_model(
        &self,
    ) -> Option<athlesia_universal_domain_learning::GroundedExecutableWorldModel> {
        self.cognition.current_executable_world_model(
            Self::live_transition_schema_policy(),
            Self::live_executable_world_model_policy(),
        )
    }

    pub fn current_structural_prediction_for_action(
        &self,
        action: crate::ArcAgi3Action,
    ) -> Option<athlesia_universal_domain_learning::GroundedStructuralPrediction> {
        let state = self.current_grounded_world_state()?;

        let transformation =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(action);

        self.cognition.current_structural_prediction(
            &state,
            &transformation,
            Self::live_transition_schema_policy(),
            Self::live_executable_world_model_policy(),
        )
    }

    fn model_grounded_executive_candidate(
        &self,
        state: &athlesia_universal_domain_learning::GroundedStateSnapshot,
        action: crate::ArcAgi3Action,
        goal: &athlesia_executive_agency::ExecutiveGoal,
        goal_alignment: athlesia_mindstone_sparse_cognition::CognitiveSignal,
        execution_cost: athlesia_mindstone_sparse_cognition::CognitiveSignal,
    ) -> Option<athlesia_executive_agency::GroundedExecutiveActionCandidate> {
        /*
         * ARC-specific responsibility ends at exact action encoding.
         *
         * M51 owns:
         *
         * - executable prediction,
         * - empirical prediction authority,
         * - predicted-outcome cognitive identity,
         * - generic M48 candidate construction.
         */
        let transformation =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(action);

        self.cognition.current_model_grounded_executive_candidate(
            state,
            &transformation,
            goal,
            goal_alignment,
            execution_cost,
            Self::live_transition_schema_policy(),
            Self::live_executable_world_model_policy(),
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

        let mut authorized = Vec::new();

        for &action in candidate_actions {
            let Some(candidate) = self.model_grounded_executive_candidate(
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
        &self,
        authorized: &[crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate],
        goal: &athlesia_executive_agency::ExecutiveGoal,
        policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
    ) -> Option<crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate> {
        if authorized.is_empty() {
            return None;
        }

        /*
         * ARC protocol grounding has already constrained this frontier to
         * currently executable environment actions.
         *
         * The generic cognitive selection itself belongs exclusively to
         * M51 -> M48.
         */
        let candidates = authorized
            .iter()
            .map(|grounded| grounded.candidate().clone())
            .collect::<Vec<_>>();

        let selected =
            self.cognition
                .current_selected_executive_candidate(&candidates, goal, policy)?;

        /*
         * Exact full candidate identity is authoritative.
         *
         * Do not collapse by action identity or predicted outcome.
         */
        authorized
            .iter()
            .find(|grounded| grounded.candidate() == &selected)
            .cloned()
    }

    fn select_authorized_executive_candidate(
        &self,
        authorized: &[crate::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate],
        goal: &athlesia_executive_agency::ExecutiveGoal,
        policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
    ) -> Option<crate::ArcAgi3Action> {
        self.selected_authorized_executive_candidate(authorized, goal, policy)
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

        self.select_authorized_executive_candidate(&authorized, goal, policy)
    }

    // B3C-C1 EVIDENCE-FAITHFUL ARC AUTHORITY PATH
    //
    // The epistemic branch is now:
    //
    // current grounded state
    //   -> current M47/M50 forecast evidence
    //   -> retained C3F empirical progress
    //   -> retained B2 successor eligibility
    //   -> Cut16B intent
    //   -> M48 evidence-faithful final selection
    //   -> exact intent
    //   -> ARC protocol authorization.
    //
    // No native possibilities or belief states enter this path.
    //
    // Cross-kind exploitation/epistemic values are intentionally NOT
    // converted onto a fabricated common scale.
    //
    // If BOTH kinds independently produce authority, this transitional
    // path abstains fail-closed.
    pub fn current_evidence_faithful_successor_executive_authority(
        &self,
        request: ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest<'_>,
    ) -> Option<ArcAgi3UnifiedExecutiveAuthority> {
        let ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest {
            candidate_actions,
            goal,
            goal_alignment,
            exploitation_execution_cost,
            version_policy,
            discrimination_policy,
            expectation_policy,
            priority_policy,
            exploitation_policy,
            epistemic_policy,
            ignorance_policy,
        } = request;

        /*
         * Encode the concrete protocol frontier only as exact cognitive
         * action identities.
         *
         * No prediction or value semantics are added.
         */
        let cognitive_actions = candidate_actions
            .iter()
            .copied()
            .map(crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action)
            .collect::<Vec<_>>();

        /*
         * Normal authority requires the strict B0 scene-grounded state.
         *
         * At true cold start, however, the agent has not yet accumulated
         * enough temporal evidence to justify a unique scene.
         *
         * In that exact case ONLY, permit evidence-neutral ignorance
         * exploration from directly observed perceptual facts.
         *
         * No exploitation or epistemic authority is evaluated in this
         * bootstrap branch because no justified scene/world state exists.
         */
        let current_state = match self.current_grounded_world_state() {
            Some(state) => state,

            None => {
                let observation_identity =
                    self.current_evidence_neutral_bootstrap_observation_identity()?;

                let retention_policy = Self::live_bootstrap_action_coverage_policy();

                /*
                 * Never authorize a real bootstrap intervention that the
                 * retained owner cannot subsequently record.
                 */
                if self.cognition.bootstrap_action_coverage_event_count()
                    >= retention_policy.max_events()
                {
                    return None;
                }

                let selected = self.cognition.current_selected_bootstrap_ignorance_action(
                    &observation_identity,
                    &cognitive_actions,
                    ignorance_policy,
                )?;

                /*
                 * M48 selection is complete.
                 *
                 * ARC contributes only exact current protocol
                 * authorization.
                 */
                let action =
                    crate::action_grounding_bridge::
                        ArcAgi3ActionGroundingBridge::
                            authorize_environment_action(
                                self.observation(),
                                selected.action(),
                            )
                            .ok()?;

                return Some(ArcAgi3UnifiedExecutiveAuthority::new_bootstrap_ignorance(
                    action, selected,
                ));
            }
        };

        /*
         * Existing model-grounded exploitation remains frozen here.
         *
         * This branch is NOT reinterpreted as epistemic evidence.
         */
        let exploitation_authorized = self.current_model_grounded_authorized_candidates(
            candidate_actions,
            goal,
            goal_alignment,
            exploitation_execution_cost,
        );

        let exploitation_source_state =
            athlesia_integrated_cognitive_agent::
                OnlinePersistentCognitiveState::
                    grounded_execution_source_state_identity(
                        &current_state,
                    );

        let exploitation_authority = self
            .selected_authorized_executive_candidate(
                &exploitation_authorized,
                goal,
                exploitation_policy,
            )
            .map(|selected| {
                ArcAgi3UnifiedExecutiveAuthority::new(exploitation_source_state, selected)
            });

        /*
         * `cognitive_actions` were encoded once above so the exact same
         * identities feed either cold-start ignorance or the normal
         * scene-grounded epistemic/ignorance path.
         */
        let epistemic_selected = self
            .cognition
            .current_selected_successor_informed_epistemic_action_intent(
                &current_state,
                &cognitive_actions,
                version_policy,
                discrimination_policy,
                expectation_policy,
                priority_policy,
                epistemic_policy,
            );

        let epistemic_authority = match epistemic_selected {
            Some(selected) => {
                /*
                 * ARC performs only protocol decoding/availability
                 * authorization here.
                 *
                 * Cognitive selection already belongs to M51 -> M48.
                 */
                let action =
                        crate::action_grounding_bridge::
                            ArcAgi3ActionGroundingBridge::
                                authorize_environment_action(
                                    self.observation(),
                                    selected.action(),
                                )
                                .ok()?;

                Some(ArcAgi3UnifiedExecutiveAuthority::new_epistemic(
                    action, selected,
                ))
            }

            None => None,
        };

        // B3D-B2 IGNORANCE FALLBACK
        //
        // Learned authority always has precedence over ignorance coverage.
        //
        // IMPORTANT:
        //
        // ignorance fallback is permitted ONLY for genuine absence:
        //
        //     exploitation = None
        //     epistemic    = None
        //
        // It must never mask a conflict between two independently valid
        // learned authorities.
        match (exploitation_authority, epistemic_authority) {
            (Some(authority), None) | (None, Some(authority)) => Some(authority),

            /*
             * Distinct learned authority kinds currently have no
             * justified common metric.
             *
             * Ambiguity remains fail-closed.
             *
             * Ignorance coverage may NOT break this tie.
             */
            (Some(_), Some(_)) => None,

            (None, None) => {
                let selected = self
                    .cognition
                    .current_selected_ignorance_exploration_action(
                        &current_state,
                        &cognitive_actions,
                        ignorance_policy,
                    )?;

                /*
                 * Selection is already complete in M48.
                 *
                 * ARC contributes only protocol decoding and current
                 * availability authorization.
                 */
                let action =
                    crate::action_grounding_bridge::
                        ArcAgi3ActionGroundingBridge::
                            authorize_environment_action(
                                self.observation(),
                                selected.action(),
                            )
                            .ok()?;

                Some(ArcAgi3UnifiedExecutiveAuthority::new_ignorance(
                    action, selected,
                ))
            }
        }
    }

    pub fn current_evidence_faithful_successor_action_selection(
        &self,
        request: ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest<'_>,
    ) -> Option<crate::ArcAgi3Action> {
        self.current_evidence_faithful_successor_executive_authority(request)
            .map(|authority| authority.action())
    }

    pub fn current_successor_informed_unified_executive_authority(
        &self,
        request: ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'_>,
    ) -> Option<ArcAgi3UnifiedExecutiveAuthority> {
        let ArcAgi3SuccessorInformedUnifiedExecutiveRequest {
            exploitation_actions,
            goal,
            goal_alignment,
            exploitation_execution_cost,
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
        let current_state = self.current_grounded_world_state()?;

        let mut authorized = self.current_model_grounded_authorized_candidates(
            exploitation_actions,
            goal,
            goal_alignment,
            exploitation_execution_cost,
        );

        let exploitation_source_state =
            athlesia_integrated_cognitive_agent::
                OnlinePersistentCognitiveState::
                grounded_execution_source_state_identity(
                    &current_state,
                );

        let mut provenance = authorized
            .iter()
            .map(|candidate| {
                athlesia_integrated_cognitive_agent::ExecutiveCandidateProvenanceBinding::new(
                    exploitation_source_state.clone(),
                    candidate.candidate().clone(),
                )
            })
            .collect::<Vec<_>>();

        /*
         * Preserve caller-native action identity and order.
         *
         * No sort, dedup, probability estimate, predicted outcome,
         * information gain, confidence, controllability, or utility is
         * synthesized here.
         */
        let native_actions = native_possibilities
            .iter()
            .map(|possibility| possibility.action().clone())
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

        if let Some(delegation) = delegated {
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
                            delegation.source_state(),
                            goal,
                            goal_alignment,
                            delegation.result(),
                        )
                        .ok()?;

            for candidate in experimental_candidates {
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
                 * Preserve every exact candidate/source provenance binding.
                 *
                 * Candidate identity may be duplicate for M48 purposes,
                 * but a different source state is NOT disposable metadata.
                 * M51 resolves or rejects provenance ambiguity later.
                 */
                provenance.push(
                    athlesia_integrated_cognitive_agent::ExecutiveCandidateProvenanceBinding::new(
                        delegation.source_state().clone(),
                        grounded.candidate().clone(),
                    ),
                );

                /*
                 * The ARC wrapper frontier needs only one wrapper per exact
                 * candidate identity because identical candidates decode to
                 * the same ARC action. Provenance is retained separately.
                 */
                if !authorized
                    .iter()
                    .any(|existing| existing.candidate() == grounded.candidate())
                {
                    authorized.push(grounded);
                }
            }
        }

        /*
         * C16I-F final authority:
         *
         * M51 performs both:
         *
         * - the single common M48 selection; and
         * - exact fail-closed source provenance resolution.
         *
         * The ARC adapter only rebinds the selected generic candidate to
         * its already-authorized protocol wrapper.
         */
        let selected_provenance = self
            .cognition
            .current_selected_executive_candidate_with_provenance(
                &provenance,
                goal,
                executive_policy,
            )?;

        let selected = authorized
            .iter()
            .find(|grounded| grounded.candidate() == selected_provenance.candidate())
            .cloned()?;

        Some(ArcAgi3UnifiedExecutiveAuthority::new(
            selected_provenance.source_state().clone(),
            selected,
        ))
    }

    pub fn current_successor_informed_unified_executive_action_selection(
        &self,
        request: ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'_>,
    ) -> Option<crate::ArcAgi3Action> {
        self.current_successor_informed_unified_executive_authority(request)
            .map(|authority| authority.action())
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

        let mut provenance =
            Vec::<athlesia_integrated_cognitive_agent::ExecutiveCandidateProvenanceBinding>::new();

        /*
         * Exploitation provenance comes from the retained grounded
         * state itself. No caller supplies a cognitive source state.
         */
        if !authorized.is_empty() {
            let grounded_state = self.current_grounded_world_state()?;

            let source_state =
                athlesia_integrated_cognitive_agent::
                    OnlinePersistentCognitiveState::
                    grounded_execution_source_state_identity(
                        &grounded_state,
                    );

            provenance.extend(authorized.iter().map(|candidate| {
                athlesia_integrated_cognitive_agent::ExecutiveCandidateProvenanceBinding::new(
                    source_state.clone(),
                    candidate.candidate().clone(),
                )
            }));
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
                        provenance.push(
                            athlesia_integrated_cognitive_agent::
                                ExecutiveCandidateProvenanceBinding::
                                    new(
                                        expected_experiment_source_state
                                            .clone(),
                                        grounded
                                            .candidate()
                                            .clone(),
                                    ),
                        );

                        if !authorized.iter().any(
                            |existing| {
                                existing.candidate()
                                    == grounded.candidate()
                            },
                        ) {
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
         * ONE M48 evaluation over exploitation + experimentation, with
         * exact provenance resolution owned by M51.
         */
        let selected_provenance = self
            .cognition
            .current_selected_executive_candidate_with_provenance(&provenance, goal, policy)?;

        let selected = authorized
            .iter()
            .find(|grounded| grounded.candidate() == selected_provenance.candidate())
            .cloned()?;

        Some(ArcAgi3UnifiedExecutiveAuthority::new(
            selected_provenance.source_state().clone(),
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
        let pending_bootstrap_coverage_action = authority
            .bootstrap_ignorance_selection()
            .map(|selected| selected.action().clone());

        let command = self.session.begin_unified_executive_action(
            authority.source_state(),
            authority.cognitive_action(),
        )?;

        debug_assert_eq!(command.action(), authority.action(),);

        self.pending_bootstrap_coverage_action = pending_bootstrap_coverage_action;

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

        // Perception and retained cognition are unchanged while an action is
        // pending. Reconstruct its subjects from that pre-action state before
        // projecting or retaining any part of the consequence.
        let previous_grouping_candidates = self.current_grouping_behavior_candidates();

        let pending_bootstrap_coverage_action = self.pending_bootstrap_coverage_action.clone();

        let mut next_session = self.session.clone();

        let completed_turn = next_session.complete_turn(observation.clone(), confidence)?;

        let next_perception = ArcAgi3PerceptualIngestionBridge::project_observation(
            &observation,
            self.perception.next_observation_index(),
            Some(self.perception.latest_frame()),
        )?;

        let mut next_cognition = self.cognition.clone();

        if let Some(expected_action) = pending_bootstrap_coverage_action.as_ref() {
            let evidence = completed_turn
                .evidence()
                .ok_or(ArcAgi3CognitiveInteractionError::BootstrapCoverageMissingFeedback)?;

            if evidence.action_observation().descriptor() != expected_action {
                return Err(ArcAgi3CognitiveInteractionError::BootstrapCoverageActionMismatch);
            }

            let retention_status = next_cognition
                .retain_bootstrap_action_coverage_event(
                    evidence,
                    Self::live_bootstrap_action_coverage_policy(),
                )
                .status();

            if retention_status
                != athlesia_integrated_cognitive_agent::
                    RetainedBootstrapActionCoverageStatus::
                        Retained
            {
                return Err(
                    ArcAgi3CognitiveInteractionError::
                        BootstrapCoverageRetentionRejected(
                            retention_status,
                        ),
                );
            }
        }

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
                    // Validate or contradict prior subjects on the explicit
                    // action boundary, including members lost in the response.
                    // A unique scene is not required for behavioral observation.
                    let grouping_behavior =
                        athlesia_core_knowledge_perceptual_grounding::
                            PerceptualGroupingBehaviorObservation::observe(
                                &previous_grouping_candidates,
                                &observation_result,
                            );

                    next_cognition.retain_perceptual_grouping_behavior_result(&grouping_behavior);

                    /*
                     * Appearance proposals are allowed to originate from
                     * current visual organization without retroactively
                     * contaminating common-change evidence for this same
                     * transition.
                     *
                     * These post-action candidates receive only appearance
                     * evidence here; behavior used the pre-action subjects.
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

                    let mut appearance_candidates = grouping_frontier.candidates().to_vec();

                    appearance_candidates.extend(
                        ArcAgi3PerceptualIngestionBridge::
                            appearance_coherent_grid_grouping_candidates(
                                causal_transition.current_frame(),
                            ),
                    );

                    /*
                     * Retain one appearance observation per physical
                     * membership grouping, even when several proposal
                     * families independently discovered it.
                     *
                     * Otherwise proposal provenance would incorrectly
                     * multiply empirical evidence and later scene
                     * hypotheses.
                     */
                    appearance_candidates.sort_by(Self::compare_objecthood_grouping_candidates);

                    appearance_candidates.dedup_by(|left, right| left.members() == right.members());

                    let grouping_appearance =
                        ArcAgi3PerceptualIngestionBridge::grouping_appearance_observation(
                            causal_transition.current_frame(),
                            &appearance_candidates,
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
                            pending_bootstrap_coverage_action: pending_bootstrap_coverage_action
                                .clone(),
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
        self.pending_bootstrap_coverage_action = None;

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
pub(crate) mod c16i_successor_informed_two_contract_e2e_tests {
    use super::*;
    mod m51_fixture {
        use crate as athlesia_arc_agi_3_adapter;

        include!("../tests/support/m51_online_orchestration_fixture.rs");
    }

    fn signal(value: u16) -> athlesia_mindstone_sparse_cognition::CognitiveSignal {
        athlesia_mindstone_sparse_cognition::CognitiveSignal::new(value).unwrap()
    }

    fn atom(value: u64) -> athlesia_mindstone_sparse_cognition::CognitiveStructure {
        athlesia_mindstone_sparse_cognition::CognitiveStructure::atom(value)
    }

    fn action(id: crate::ArcAgi3ActionId) -> crate::ArcAgi3Action {
        crate::ArcAgi3Action::discrete(id).unwrap()
    }

    fn object_grid(value: u8) -> crate::ArcAgi3Grid {
        crate::ArcAgi3Grid::from_rows(vec![vec![value, value], vec![8, 9]]).unwrap()
    }

    fn observation(
        game: &str,
        value: u8,
        last_action: Option<crate::ArcAgi3Action>,
    ) -> crate::ArcAgi3Observation {
        crate::ArcAgi3Observation::new(
            crate::ArcAgi3GameId::new(game.to_string()).unwrap(),
            crate::ArcAgi3GameState::NotFinished,
            crate::ArcAgi3FrameSequence::new(vec![object_grid(value)]).unwrap(),
            0,
            3,
            crate::ArcAgi3AvailableActions::new(vec![
                crate::ArcAgi3ActionId::Action1,
                crate::ArcAgi3ActionId::Action2,
            ])
            .unwrap(),
            last_action,
        )
    }

    fn observation_with_available_actions(
        game: &str,
        value: u8,
        last_action: Option<crate::ArcAgi3Action>,
        available_actions: Vec<crate::ArcAgi3ActionId>,
    ) -> crate::ArcAgi3Observation {
        crate::ArcAgi3Observation::new(
            crate::ArcAgi3GameId::new(game.to_string()).unwrap(),
            crate::ArcAgi3GameState::NotFinished,
            crate::ArcAgi3FrameSequence::new(vec![object_grid(value)]).unwrap(),
            0,
            3,
            crate::ArcAgi3AvailableActions::new(available_actions).unwrap(),
            last_action,
        )
    }

    fn training_turn(
        runtime: &mut ArcAgi3CognitiveInteractionRuntime,
        game: &str,
        selected_action: crate::ArcAgi3Action,
        value: u8,
    ) {
        let cognitive_action =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
                selected_action,
            );

        m51_fixture::begin_arc(runtime, cognitive_action)
            .expect("C16I E2E training action must begin");

        let completion = runtime
            .complete_environment_turn(observation(game, value, Some(selected_action)), signal(900))
            .expect("C16I E2E real consequence must commit");

        assert!(
            completion.has_cognitive_feedback(),
            "training turn must be a genuine causal environment event",
        );
    }

    fn mature_runtime(runtime: &mut ArcAgi3CognitiveInteractionRuntime, game: &str) {
        let action_one = action(crate::ArcAgi3ActionId::Action1);

        let action_two = action(crate::ArcAgi3ActionId::Action2);

        for value in [2_u8, 3, 4, 5] {
            training_turn(runtime, game, action_one, value);
        }

        for (selected, value) in [
            (action_two, 5_u8),
            (action_one, 6_u8),
            (action_two, 6_u8),
            (action_one, 5_u8),
            (action_two, 5_u8),
            (action_one, 6_u8),
            (action_two, 6_u8),
            (action_one, 5_u8),
        ] {
            training_turn(runtime, game, selected, value);
        }
    }

    fn version_policy() -> athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy
    {
        athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
            1, 64, 512, 256,
        )
        .unwrap()
    }

    fn discrimination_policy(
    ) -> athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy {
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy::new(
            512, 512,
        )
        .unwrap()
    }

    fn expectation_policy(
    ) -> athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy {
        athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy::new(
            256, 256, 1,
        )
        .unwrap()
    }

    fn priority_policy(
    ) -> athlesia_autonomous_active_experimentation::EmpiricalEpistemicActionPriorityPolicy {
        athlesia_autonomous_active_experimentation::EmpiricalEpistemicActionPriorityPolicy::new(8)
            .unwrap()
    }

    fn proposal_policy(
    ) -> athlesia_autonomous_active_experimentation::BeliefDrivenExperimentProposalPolicy {
        use athlesia_autonomous_active_experimentation::{
            ActiveExperimentBounds, ActiveExperimentPolicy, ActiveExperimentThresholds,
            BeliefDrivenExperimentProposalBounds, BeliefDrivenExperimentProposalPolicy,
        };

        BeliefDrivenExperimentProposalPolicy::new(
            ActiveExperimentPolicy::new(
                ActiveExperimentBounds::new(16, 16, 16).unwrap(),
                ActiveExperimentThresholds::new(signal(500), signal(500), signal(500), signal(500))
                    .unwrap(),
            ),
            BeliefDrivenExperimentProposalBounds::new(16, 16, 16, 16).unwrap(),
            signal(500),
            signal(500),
        )
        .unwrap()
    }

    fn executive_policy() -> athlesia_executive_agency::ExecutiveAgencyPolicy {
        use athlesia_executive_agency::{
            ExecutiveAgencyPolicy, ExecutiveSelectionThresholds, ExecutiveUtilityWeights,
        };

        ExecutiveAgencyPolicy::new(
            1,
            8,
            16,
            1,
            ExecutiveUtilityWeights::new(0, 0, 0, 1000, 0).unwrap(),
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

    fn goal() -> athlesia_executive_agency::ExecutiveGoal {
        athlesia_executive_agency::ExecutiveGoal::new(
            atom(0xC16F_0000_0000_0001),
            signal(900),
            athlesia_mindstone_sparse_cognition::CognitiveSignal::zero(),
        )
    }

    #[derive(Debug)]
    pub(crate) struct Fixture {
        runtime: ArcAgi3CognitiveInteractionRuntime,
        arc_action: crate::ArcAgi3Action,
        cognitive_action: CognitiveStructure,
        native_source: CognitiveStructure,
        native_possibilities:
            Vec<athlesia_autonomous_active_experimentation::GroundedExperimentPossibility>,
        beliefs: Vec<athlesia_autonomous_active_experimentation::HypothesisBeliefState>,
    }

    impl Fixture {
        pub(crate) fn into_live_parts(
            self,
        ) -> (
            ArcAgi3CognitiveInteractionRuntime,
            crate::ArcAgi3Action,
            CognitiveStructure,
            CognitiveStructure,
            Vec<athlesia_autonomous_active_experimentation::GroundedExperimentPossibility>,
            Vec<athlesia_autonomous_active_experimentation::HypothesisBeliefState>,
        ) {
            (
                self.runtime,
                self.arc_action,
                self.cognitive_action,
                self.native_source,
                self.native_possibilities,
                self.beliefs,
            )
        }
    }

    pub(crate) fn live_goal() -> athlesia_executive_agency::ExecutiveGoal {
        goal()
    }

    /*
     * B4B4 TEST-ONLY helper.
     *
     * Seeds the retained bootstrap coverage owner through its real M51
     * evidence API. This does NOT create production mutable-cognition
     * authority and exists only inside this #[cfg(test)] module.
     */
    pub(crate) fn retain_bootstrap_coverage_for_test(
        runtime: &mut ArcAgi3CognitiveInteractionRuntime,
        event_index: u64,
        action: crate::ArcAgi3Action,
    ) {
        let cognitive_action =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(action);

        let observation =
            athlesia_integrated_cognitive_agent::EnvironmentInteractionObservation::new(
                event_index,
                CognitiveStructure::Ordered(vec![
                    CognitiveStructure::atom(0x4234_4234_5445_5354),
                    CognitiveStructure::atom(event_index),
                ]),
                signal(900),
            )
            .expect("B4B4 synthetic test observation is valid");

        let evidence =
            athlesia_integrated_cognitive_agent::EnvironmentInteractionEvidence::self_generated(
                &CognitiveStructure::atom(0x4234_4234_5352_4345),
                &cognitive_action,
                &observation,
            )
            .expect("B4B4 test evidence must be valid self-generated evidence");

        let result = runtime.cognition.retain_bootstrap_action_coverage_event(
            &evidence,
            athlesia_integrated_cognitive_agent::RetainedBootstrapActionCoveragePolicy::new(
                crate::production_action_frontier::ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
            )
            .unwrap(),
        );

        assert_eq!(
            result.status(),
            athlesia_integrated_cognitive_agent::RetainedBootstrapActionCoverageStatus::Retained,
            "B4B4 test bootstrap evidence must retain exactly once",
        );
    }

    pub(crate) fn live_response(
        game: &str,
        value: u8,
        last_action: Option<crate::ArcAgi3Action>,
    ) -> crate::ArcAgi3Observation {
        observation(game, value, last_action)
    }

    pub(crate) fn live_ignorance_fixture(
        game: &str,
        first_index: u64,
    ) -> (
        ArcAgi3CognitiveInteractionRuntime,
        [crate::ArcAgi3Action; 2],
        CognitiveStructure,
    ) {
        let action_two = action(crate::ArcAgi3ActionId::Action2);

        let action_three = action(crate::ArcAgi3ActionId::Action3);

        let action_four = action(crate::ArcAgi3ActionId::Action4);

        let mut runtime =
            ArcAgi3CognitiveInteractionRuntime::new(observation(game, 1, None), first_index)
                .unwrap();

        /*
         * Build enough perceptual/grounding history for a stable
         * current state using only ACTION1/ACTION2.
         *
         * ACTION3/ACTION4 remain interventionally unseen.
         */
        mature_runtime(&mut runtime, game);

        let cognitive_action_two =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
                action_two,
            );

        m51_fixture::begin_arc(&mut runtime, cognitive_action_two)
            .expect("final grounding turn must begin");

        let completion = runtime
            .complete_environment_turn(
                observation_with_available_actions(
                    game,
                    7,
                    Some(action_two),
                    vec![
                        crate::ArcAgi3ActionId::Action1,
                        crate::ArcAgi3ActionId::Action2,
                        crate::ArcAgi3ActionId::Action3,
                        crate::ArcAgi3ActionId::Action4,
                    ],
                ),
                signal(900),
            )
            .expect("final grounding turn must complete");

        assert!(completion.has_cognitive_feedback(),);

        let current = runtime
            .current_grounded_world_state()
            .expect("ignorance fixture must be grounded");

        let source =
            athlesia_integrated_cognitive_agent::
                OnlinePersistentCognitiveState::
                    grounded_execution_source_state_identity(
                        &current,
                    );

        for unseen in [action_three, action_four] {
            let cognitive =
                crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
                    unseen,
                );

            assert_eq!(
                runtime
                    .cognition()
                    .transition_schema_learning()
                    .exact_source_action_sample_count(&current, &cognitive,),
                Some(0),
                "ACTION3/ACTION4 must be genuinely unseen in the exact source state",
            );
        }

        (runtime, [action_three, action_four], source)
    }

    pub(crate) fn live_production_ignorance_fixture(
        game: &str,
        first_index: u64,
    ) -> (
        ArcAgi3CognitiveInteractionRuntime,
        [crate::ArcAgi3Action; 2],
        CognitiveStructure,
    ) {
        let action_two = action(crate::ArcAgi3ActionId::Action2);

        let action_three = action(crate::ArcAgi3ActionId::Action3);

        let action_four = action(crate::ArcAgi3ActionId::Action4);

        let mut runtime =
            ArcAgi3CognitiveInteractionRuntime::new(observation(game, 1, None), first_index)
                .unwrap();

        /*
         * Establish retained grounding/history without using the two
         * production cold-start interventions.
         */
        mature_runtime(&mut runtime, game);

        let cognitive_action_two =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
                action_two,
            );

        m51_fixture::begin_arc(&mut runtime, cognitive_action_two)
            .expect("production fixture grounding turn must begin");

        let completion = runtime
            .complete_environment_turn(
                observation_with_available_actions(
                    game,
                    7,
                    Some(action_two),
                    vec![
                        crate::ArcAgi3ActionId::Action3,
                        crate::ArcAgi3ActionId::Action4,
                    ],
                ),
                signal(900),
            )
            .expect("production fixture grounding turn must complete");

        assert!(completion.has_cognitive_feedback());

        let current = runtime
            .current_grounded_world_state()
            .expect("production ignorance fixture must be grounded");

        let source =
            athlesia_integrated_cognitive_agent::
                OnlinePersistentCognitiveState::
                    grounded_execution_source_state_identity(
                        &current,
                    );

        for unseen in [action_three, action_four] {
            let cognitive =
                crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
                    unseen,
                );

            assert_eq!(
                runtime
                    .cognition()
                    .transition_schema_learning()
                    .exact_source_action_sample_count(&current, &cognitive),
                Some(0),
                "production ACTION3/ACTION4 must begin unseen in the exact source state",
            );
        }

        (runtime, [action_three, action_four], source)
    }

    pub(crate) fn live_production_response(
        game: &str,
        value: u8,
        last_action: Option<crate::ArcAgi3Action>,
        available_action: crate::ArcAgi3ActionId,
    ) -> crate::ArcAgi3Observation {
        observation_with_available_actions(game, value, last_action, vec![available_action])
    }

    pub(crate) fn live_ignorance_response(
        game: &str,
        value: u8,
        last_action: Option<crate::ArcAgi3Action>,
    ) -> crate::ArcAgi3Observation {
        observation_with_available_actions(
            game,
            value,
            last_action,
            vec![
                crate::ArcAgi3ActionId::Action1,
                crate::ArcAgi3ActionId::Action2,
                crate::ArcAgi3ActionId::Action3,
                crate::ArcAgi3ActionId::Action4,
            ],
        )
    }

    pub(crate) fn live_request<'a>(
        exploitation_actions: &'a [crate::ArcAgi3Action],
        goal: &'a athlesia_executive_agency::ExecutiveGoal,
        native_possibilities:
            &'a [
                athlesia_autonomous_active_experimentation::
                    GroundedExperimentPossibility
            ],
        beliefs: &'a [athlesia_autonomous_active_experimentation::HypothesisBeliefState],
    ) -> ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'a> {
        ArcAgi3SuccessorInformedUnifiedExecutiveRequest {
            exploitation_actions,
            goal,
            goal_alignment: signal(900),
            exploitation_execution_cost: signal(100),
            native_possibilities,
            beliefs,
            version_policy: version_policy(),
            discrimination_policy: discrimination_policy(),
            expectation_policy: expectation_policy(),
            priority_policy: priority_policy(),
            proposal_policy: proposal_policy(),
            executive_policy: executive_policy(),
        }
    }

    pub(crate) fn live_evidence_faithful_request<'a>(
        candidate_actions: &'a [crate::ArcAgi3Action],
        goal: &'a athlesia_executive_agency::ExecutiveGoal,
    ) -> ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest<'a> {
        ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest {
            candidate_actions,
            goal,

            /*
             * Test fixture deliberately disables exploitation so the
             * evidence-faithful epistemic authority is isolated exactly.
             *
             * This value is never consumed by the epistemic branch.
             */
            goal_alignment: athlesia_mindstone_sparse_cognition::CognitiveSignal::zero(),

            exploitation_execution_cost: signal(100),

            version_policy: version_policy(),

            discrimination_policy: discrimination_policy(),

            expectation_policy: expectation_policy(),

            priority_policy: priority_policy(),

            exploitation_policy: executive_policy(),

            epistemic_policy: athlesia_executive_agency::EpistemicExecutiveSelectionPolicy::new(8)
                .unwrap(),

            ignorance_policy: athlesia_executive_agency::IgnoranceExplorationSelectionPolicy::new(
                8,
            )
            .unwrap(),
        }
    }

    pub(crate) fn fixture(game: &str, first_index: u64) -> Fixture {
        use athlesia_autonomous_active_experimentation::{
            AutonomousEpistemicForecastDiscrimination, AutonomousEpistemicResolutionProgress,
            CompetingHypothesisPrediction, GroundedEpistemicExperimentPossibility,
            GroundedExperimentPossibility, HypothesisBeliefState,
        };

        let action_one = action(crate::ArcAgi3ActionId::Action1);

        let action_two = action(crate::ArcAgi3ActionId::Action2);

        let mut runtime =
            ArcAgi3CognitiveInteractionRuntime::new(observation(game, 1, None), first_index)
                .unwrap();

        mature_runtime(&mut runtime, game);

        /*
         * Enter the established informative holdout.
         */
        training_turn(&mut runtime, game, action_two, 7_u8);

        /*
         * Real B2 sample:
         *
         *     state7 --ACTION1--> state6
         */
        training_turn(&mut runtime, game, action_one, 6_u8);

        /*
         * Return through a real causal turn to the exact state7
         * representation.  B2 now contains a genuine ACTION1 sample
         * from this source representation.
         */
        training_turn(&mut runtime, game, action_two, 7_u8);

        let current = runtime
            .current_grounded_world_state()
            .expect("C16I E2E current state7 must be grounded");

        let cognitive_action =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
                action_one,
            );

        let b2 = runtime
            .current_action_qualified_empirical_successor_frequency(&cognitive_action)
            .expect("B0 current grounding must permit B2 live query");

        assert!(
            b2.is_qualified(),
            "C16I E2E requires genuine action-qualified B2 evidence",
        );

        assert!(
            b2.independent_action_event_count() > 0,
            "B2 authority must contain at least one real interaction event",
        );

        assert!(
            b2.successor_informed_proposal_eligibility()
                .eligible_as_supplemental_evidence(),
            "real B2 evidence must pass C16I-A integrity qualification",
        );

        /*
         * Current M50 question from the retained M47 owner.
         */
        let current_epistemic = runtime
            .cognition
            .current_m50_epistemic_possibility(&current, &cognitive_action, version_policy())
            .expect("current state7 ACTION1 must expose an M50 epistemic possibility");

        let current_discrimination = AutonomousEpistemicForecastDiscrimination::evaluate(
            &current_epistemic,
            discrimination_policy(),
        );

        assert!(
            current_discrimination.informative(),
            "C16I E2E requires a genuinely unresolved current M50 question",
        );

        assert!(
            current_discrimination.pairwise_separation_score() > 0,
            "informative M50 question must contain real pairwise separation",
        );

        /*
         * Ground outcome resolution in an actually observed B2
         * successor representation.  No target-occurrence booleans are
         * invented by this fixture.
         */
        let real_successor = b2
            .successor_frequencies()
            .first()
            .expect("qualified B2 result must expose a real successor")
            .successor_representation()
            .clone();

        let realized_outcome = runtime
            .cognition
            .resolve_m50_epistemic_possibility_against_transition(
                &current_epistemic,
                &current,
                &real_successor,
                &cognitive_action,
                athlesia_autonomous_active_experimentation::EpistemicOutcomeResolutionPolicy::new(
                    512, 512,
                )
                .unwrap(),
            )
            .expect("real B2 successor must resolve the current M50 question");

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
        let post_learning = GroundedEpistemicExperimentPossibility::new(
            current_epistemic.source_state().clone(),
            current_epistemic.action().clone(),
            vec![current_epistemic
                .forecasts()
                .first()
                .expect("informative possibility must contain forecasts")
                .clone()],
        )
        .unwrap();

        let progress = AutonomousEpistemicResolutionProgress::measure(
            &current_epistemic,
            &realized_outcome,
            &post_learning,
            discrimination_policy(),
        );

        assert!(
            progress.measured(),
            "C16I E2E C3F evidence must come from the real C3D measurement authority",
        );

        let sample = progress
            .sample()
            .expect("measured progress must contain one exact sample")
            .clone();

        assert!(
            sample.realized_separation_reduction() > sample.realized_separation_increase(),
            "fixture must provide positive measured epistemic progress for C3F",
        );

        /*
         * Retain through M51's existing bounded history owner.
         *
         * The high event id is fixture provenance only; it does not
         * participate in matching or priority.
         */
        let retained = runtime.cognition.retain_epistemic_progress_event(
            u64::MAX - 16,
            sample,
            athlesia_integrated_cognitive_agent::RetainedEpistemicProgressHistoryPolicy::new(256)
                .unwrap(),
        );

        assert!(
            retained.retained(),
            "measured C3D sample must be retained by the existing M51 owner",
        );

        let priority = runtime
            .cognition
            .current_empirical_epistemic_action_priority_frontier(
                &current,
                std::slice::from_ref(&cognitive_action),
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

        let priority_candidate = priority
            .best()
            .expect("Ranked C3F frontier must expose one exact candidate");

        assert_eq!(priority_candidate.action(), &cognitive_action,);

        /*
         * C16I-C requires caller-native M50 source/action identity to
         * match the already-ranked C3F binding exactly.
         */
        let native_source = priority_candidate.source_state().clone();

        let mut native_hypotheses = Vec::<CognitiveStructure>::new();

        for forecast in current_epistemic.forecasts() {
            if !native_hypotheses
                .iter()
                .any(|existing| existing == forecast.hypothesis())
            {
                native_hypotheses.push(forecast.hypothesis().clone());
            }
        }

        assert!(
            native_hypotheses.len() >= 2,
            "C16I native fixture requires at least two exact current M50 hypothesis identities",
        );

        let hypothesis_one = native_hypotheses[0].clone();

        let hypothesis_two = native_hypotheses[1].clone();

        let native_possibilities = vec![GroundedExperimentPossibility::new(
            native_source.clone(),
            cognitive_action.clone(),
            vec![
                CompetingHypothesisPrediction::new(
                    hypothesis_one.clone(),
                    atom(0xC16F_0000_0000_0201),
                    signal(900),
                )
                .unwrap(),
                CompetingHypothesisPrediction::new(
                    hypothesis_two.clone(),
                    atom(0xC16F_0000_0000_0202),
                    signal(900),
                )
                .unwrap(),
            ],
            signal(900),
            signal(900),
            signal(100),
        )
        .unwrap()];

        let beliefs = vec![
            HypothesisBeliefState::new(hypothesis_one, signal(820)).unwrap(),
            HypothesisBeliefState::new(hypothesis_two, signal(760)).unwrap(),
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
            native_result.result().generated_count() > 0,
            "native M50 must actually generate a proposal before F is exercised",
        );

        Fixture {
            runtime,
            arc_action: action_one,
            cognitive_action,
            native_source,
            native_possibilities,
            beliefs,
        }
    }

    #[test]
    fn real_b2_measured_c3f_native_m50_reaches_action_only_through_common_m48() {
        let fixture = fixture("c16i-f-common-m48", 8_200_000);

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

        assert_eq!(authority.action(), fixture.arc_action,);

        assert_eq!(authority.cognitive_action(), &fixture.cognitive_action,);

        assert_eq!(
            authority.source_state(),
            &fixture.native_source,
            "selected experiment must preserve exact native M50 source provenance through common M48",
        );

        assert!(
            authority
                .legacy_candidate()
                .expect("legacy successor regression must expose legacy candidate")
                .information_gain()
                > athlesia_mindstone_sparse_cognition::
                    CognitiveSignal::zero(),
            "selected experiment must carry native M50 information authority rather than fabricated adapter utility",
        );
    }

    #[test]
    fn b3cc1_evidence_faithful_arc_path_uses_no_native_beliefs_or_possibilities() {
        let fixture = fixture("b3cc1-clean-epistemic", 8_400_000);

        let candidate_actions = [fixture.arc_action];

        let clean_request =
            ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest {
                candidate_actions: &candidate_actions,

                goal: &goal(),

                /*
                 * Deliberately suppress the frozen exploitation branch.
                 *
                 * This scalar is NOT consumed by the epistemic branch.
                 */
                goal_alignment: CognitiveSignal::zero(),

                exploitation_execution_cost: signal(100),

                version_policy: version_policy(),

                discrimination_policy: discrimination_policy(),

                expectation_policy: expectation_policy(),

                priority_policy: priority_policy(),

                exploitation_policy: executive_policy(),

                epistemic_policy:
                    athlesia_executive_agency::EpistemicExecutiveSelectionPolicy::new(8).unwrap(),

                ignorance_policy:
                    athlesia_executive_agency::IgnoranceExplorationSelectionPolicy::new(8).unwrap(),
            };

        let progress_before = fixture.runtime.cognition().epistemic_progress_event_count();

        let transition_before = fixture.runtime.cognition().transition_episode_count();

        let authority = fixture
            .runtime
            .current_evidence_faithful_successor_executive_authority(clean_request)
            .expect("real retained B2+C3F evidence must reach clean M48 authority");

        assert_eq!(
            authority.kind(),
            ArcAgi3UnifiedExecutiveAuthorityKind::EvidenceFaithfulEpistemic,
        );

        assert_eq!(authority.action(), fixture.arc_action,);

        assert_eq!(authority.cognitive_action(), &fixture.cognitive_action,);

        assert_eq!(authority.source_state(), &fixture.native_source,);

        assert!(
            authority.legacy_candidate().is_none(),
            "clean epistemic authority must not smuggle a legacy executive candidate",
        );

        let selected = authority
            .epistemic_selection()
            .expect("clean authority must retain exact M51-selected epistemic intent");

        assert_eq!(selected.action(), &fixture.cognitive_action,);

        assert_eq!(selected.source_state(), &fixture.native_source,);

        assert_eq!(
            authority.predicted_outcome(),
            None,
            "clean epistemic authority must not fabricate one concrete outcome",
        );

        assert_eq!(
            fixture.runtime.cognition().epistemic_progress_event_count(),
            progress_before,
            "selection itself must not manufacture learning evidence",
        );

        assert_eq!(
            fixture.runtime.cognition().transition_episode_count(),
            transition_before,
            "selection itself must not manufacture transition evidence",
        );
    }

    #[test]
    fn b3cc1_clean_path_abstains_without_any_candidate_action() {
        let runtime = ArcAgi3CognitiveInteractionRuntime::new(
            observation("b3cc1-no-evidence", 1, None),
            8_500_000,
        )
        .unwrap();

        let candidate_actions: [crate::ArcAgi3Action; 0] = [];

        let request =
            ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest {
                candidate_actions: &candidate_actions,
                goal: &goal(),
                goal_alignment: CognitiveSignal::zero(),
                exploitation_execution_cost: signal(100),
                version_policy: version_policy(),
                discrimination_policy: discrimination_policy(),
                expectation_policy: expectation_policy(),
                priority_policy: priority_policy(),
                exploitation_policy: executive_policy(),
                epistemic_policy:
                    athlesia_executive_agency::EpistemicExecutiveSelectionPolicy::new(8).unwrap(),

                ignorance_policy:
                    athlesia_executive_agency::IgnoranceExplorationSelectionPolicy::new(8).unwrap(),
            };

        assert_eq!(
            runtime.current_evidence_faithful_successor_executive_authority(request,),
            None,
            "an empty action frontier must remain abstention",
        );
    }

    #[test]
    fn b3db2_absent_learned_authority_uses_exact_m48_ignorance_coverage() {
        let (runtime, candidate_actions, expected_source) =
            live_ignorance_fixture("b3db2-cognitive", 9_000_000);

        let goal = goal();

        let request = live_evidence_faithful_request(&candidate_actions, &goal);

        let authority = runtime
            .current_evidence_faithful_successor_executive_authority(request)
            .expect("grounded unseen actions must produce ignorance coverage authority");

        assert_eq!(
            authority.kind(),
            ArcAgi3UnifiedExecutiveAuthorityKind::IgnoranceExploration,
        );

        assert_eq!(authority.source_state(), &expected_source,);

        assert!(candidate_actions.contains(&authority.action(),),);

        assert!(authority.legacy_candidate().is_none(),);

        assert!(authority.epistemic_selection().is_none(),);

        let ignorance = authority
            .ignorance_selection()
            .expect("ignorance authority must retain exact M48 coverage candidate");

        assert_eq!(ignorance.exact_source_action_sample_count(), 0,);

        assert_eq!(ignorance.action(), authority.cognitive_action(),);

        assert_eq!(
            authority.predicted_outcome(),
            None,
            "ignorance exploration cannot fabricate a concrete outcome",
        );
    }
}

use crate::{
    ArcAgi3Observation,
    interactive_session_runtime::{
        ArcAgi3CompletedTurn, ArcAgi3InteractiveSession, ArcAgi3InteractiveSessionError,
        ArcAgi3SessionCommand,
    },
    perceptual_ingestion_bridge::{
        ArcAgi3PerceptualBridgeError, ArcAgi3PerceptualIngestionBridge, ArcAgi3PerceptualProjection,
    },
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

    fn live_temporal_grouping_policy()
    -> athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidencePolicy {
        athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidencePolicy::new(
            2,
        )
        .expect("live temporal support threshold is positive")
    }

    fn live_grouping_generation_policy()
    -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingGenerationPolicy {
        athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingGenerationPolicy::new(
            256, 256,
        )
        .expect("live grouping frontier bounds are positive")
    }

    fn live_grouping_behavior_retention_policy()
    -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingBehaviorRetentionPolicy {
        athlesia_core_knowledge_perceptual_grounding::
            PerceptualGroupingBehaviorRetentionPolicy::new(2, 2)
            .expect("live grouping behavior thresholds are positive")
    }

    fn live_grouping_appearance_retention_policy()
    -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingAppearanceRetentionPolicy
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
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate> {
        let policy = Self::live_grouping_behavior_retention_policy();

        let mut candidates = self
            .cognition
            .perceptual_grouping_behavior_evidence()
            .supported_records(policy)
            .into_iter()
            .map(|record| record.candidate().clone())
            .filter(|candidate| candidate.is_grounded_in(self.perception.latest_frame()))
            .collect::<Vec<_>>();

        candidates.sort();
        candidates.dedup();

        candidates
    }

    pub fn current_objecthood_eligible_groupings(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate> {
        use athlesia_core_knowledge_perceptual_grounding::{
            PerceptualObjectPromotionEvidence, PerceptualObjectPromotionGate,
            PerceptualObjectProposal, PerceptualProposalTemporalSupportStatus,
        };

        let frame = self.perception.latest_frame();

        let mut eligible = Vec::new();

        for grouping in self.current_empirically_coherent_groupings() {
            let temporal_persistence = grouping.members().iter().all(|handle| {
                let proposal = PerceptualObjectProposal::new(vec![*handle])
                    .expect("one grouping member forms one valid atomic proposal");

                self.cognition
                    .perceptual_temporal_evidence()
                    .support_status(&proposal, Self::live_temporal_grouping_policy())
                    == PerceptualProposalTemporalSupportStatus::Supported
            });

            let Some((appearance_cohesion, contrast_boundary)) =
                ArcAgi3PerceptualIngestionBridge::grouping_visual_objecthood_evidence(
                    frame, &grouping,
                )
            else {
                continue;
            };

            /*
             * The source collection is already restricted to retained
             * empirically coherent groupings. Therefore common_change=true
             * here is not inferred from appearance or geometry.
             */
            let evidence = PerceptualObjectPromotionEvidence::new(
                temporal_persistence,
                true,
                appearance_cohesion,
                contrast_boundary,
            );

            if let Some(candidate) = PerceptualObjectPromotionGate::evaluate(grouping, evidence) {
                eligible.push(candidate.grouping().clone());
            }
        }

        eligible.sort();
        eligible.dedup();

        eligible
    }

    pub fn current_provisional_object_hypotheses(
        &self,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::ObjectHypothesis> {
        use athlesia_core_knowledge_perceptual_grounding::{
            EmpiricalObjecthoodSignalCalibration, ObjectHypothesis, ObjecthoodEvidence,
            PerceptualGroupingAppearanceSupportStatus, PerceptualObjectProposal,
        };

        let mut hypotheses = Vec::new();

        for grouping in self.current_objecthood_eligible_groupings() {
            if self
                .cognition
                .perceptual_grouping_appearance_evidence()
                .support_status(&grouping, Self::live_grouping_appearance_retention_policy())
                != PerceptualGroupingAppearanceSupportStatus::Supported
            {
                continue;
            }

            let Some(appearance_record) = self
                .cognition
                .perceptual_grouping_appearance_evidence()
                .record(&grouping)
            else {
                continue;
            };

            let Some(behavior_record) = self
                .cognition
                .perceptual_grouping_behavior_evidence()
                .record(&grouping)
            else {
                continue;
            };

            let behavioral_opportunities = behavior_record
                .uniform_changed_count()
                .saturating_add(behavior_record.mixed_count());

            let Some(common_change) = EmpiricalObjecthoodSignalCalibration::from_counts(
                behavior_record.uniform_changed_count(),
                behavioral_opportunities,
            ) else {
                continue;
            };

            let Some(cohesion) = EmpiricalObjecthoodSignalCalibration::from_counts(
                appearance_record.appearance_cohesion_support_count(),
                appearance_record.observation_count(),
            ) else {
                continue;
            };

            let Some(boundary) = EmpiricalObjecthoodSignalCalibration::from_counts(
                appearance_record.contrast_boundary_support_count(),
                appearance_record.observation_count(),
            ) else {
                continue;
            };

            let mut persistence: Option<athlesia_mindstone_sparse_cognition::CognitiveSignal> =
                None;

            let mut valid_members = true;

            for handle in grouping.members() {
                let proposal = PerceptualObjectProposal::new(vec![*handle])
                    .expect("one grouping member is one valid atomic proposal");

                let Some(record) = self
                    .cognition
                    .perceptual_temporal_evidence()
                    .record(&proposal)
                else {
                    valid_members = false;
                    break;
                };

                let Some(member_signal) = EmpiricalObjecthoodSignalCalibration::from_counts(
                    record.cross_frame_presence_count(),
                    record.observation_count(),
                ) else {
                    valid_members = false;
                    break;
                };

                persistence = Some(match persistence {
                    Some(current) => current.min(member_signal),
                    None => member_signal,
                });
            }

            if !valid_members {
                continue;
            }

            let Some(persistence) = persistence else {
                continue;
            };

            let evidence = ObjecthoodEvidence::new(
                cohesion,
                persistence,
                common_change,
                boundary,
                athlesia_mindstone_sparse_cognition::CognitiveSignal::zero(),
                athlesia_mindstone_sparse_cognition::CognitiveSignal::zero(),
            );

            let Some(hypothesis) = ObjectHypothesis::new(grouping.members().to_vec(), evidence)
            else {
                continue;
            };

            if hypothesis.is_grounded_in(self.perception.latest_frame()) {
                hypotheses.push(hypothesis);
            }
        }

        hypotheses.sort_by(|left, right| left.members().cmp(right.members()));

        hypotheses.dedup_by(|left, right| left.members() == right.members());

        hypotheses
    }

    fn live_scene_grounding_policy()
    -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroundingPolicy {
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

    fn live_transition_schema_learning_policy()
    -> athlesia_integrated_cognitive_agent::EndogenousTransitionSchemaLearningPolicy {
        athlesia_integrated_cognitive_agent::EndogenousTransitionSchemaLearningPolicy::new(
            256,
            Self::live_transition_schema_policy(),
        )
        .expect("live transition learning has a positive evidence frontier")
    }

    fn live_perceptual_world_context()
    -> athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext {
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

    fn live_executable_world_model_policy()
    -> athlesia_universal_domain_learning::GroundedExecutableWorldModelPolicy {
        athlesia_universal_domain_learning::GroundedExecutableWorldModelPolicy::new(64)
            .expect("live executable world-model schema frontier is positive")
    }

    pub fn current_grounded_world_state(
        &self,
    ) -> Option<&athlesia_universal_domain_learning::GroundedStateSnapshot> {
        self.cognition
            .transition_schema_learning()
            .episodes()
            .last()
            .map(athlesia_universal_domain_learning::GroundedTransformationEpisode::after)
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
                state,
                &transformation,
                &model,
            ),
        )
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
                            next_cognition
                                .observe_environment_transition(
                                    &world_input,
                                    Self::live_perceptual_world_context(),
                                    environment_evidence,
                                    Self::live_transition_schema_learning_policy(),
                                );
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

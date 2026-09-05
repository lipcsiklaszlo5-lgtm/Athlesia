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

                    next_cognition.retain_perceptual_observation_result(&observation_result);
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

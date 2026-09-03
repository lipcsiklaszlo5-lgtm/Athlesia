use athlesia_autonomous_active_experimentation::*;
use athlesia_core_knowledge_perceptual_grounding::*;
use athlesia_executive_agency::*;
use athlesia_integrated_cognitive_agent::*;
use athlesia_meta_learning_skill_memory::*;
use athlesia_mindstone_sparse_cognition::*;
use athlesia_universal_domain_learning::*;

mod fx48 {
    use super::*;
    fn signal(value: u16) -> CognitiveSignal {
        CognitiveSignal::new(value).unwrap()
    }

    fn atom(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn goal(identity: CognitiveStructure, satisfaction: u16) -> ExecutiveGoal {
        ExecutiveGoal::new(identity, signal(1000), signal(satisfaction))
    }

    fn agency_policy() -> ExecutiveAgencyPolicy {
        ExecutiveAgencyPolicy::new(
            32,
            32,
            128,
            128,
            ExecutiveUtilityWeights::new(1, 0, 0, 0, 0).unwrap(),
            ExecutiveSelectionThresholds::new(
                signal(1),
                signal(1),
                signal(1),
                signal(1),
                signal(1),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn arbitration_policy() -> GoalConflictArbitrationPolicy {
        GoalConflictArbitrationPolicy::new(
            32,
            32,
            128,
            32,
            signal(0),
            GoalConflictArbitrationThresholds::new(signal(1), signal(1)).unwrap(),
        )
        .unwrap()
    }

    fn intention_policy() -> MultiStepIntentionPolicy {
        MultiStepIntentionPolicy::new(
            32,
            32,
            16,
            256,
            32,
            MultiStepIntentionThresholds::new(signal(1), signal(1), signal(1), signal(1)).unwrap(),
        )
        .unwrap()
    }

    fn source(
        goal_identity: CognitiveStructure,
        action: CognitiveStructure,
        outcome: CognitiveStructure,
    ) -> Vec<ArbitratedExecutiveIntent> {
        let current_goal = goal(goal_identity.clone(), 0);

        let candidate = GroundedExecutiveActionCandidate::new(
            goal_identity,
            action,
            outcome,
            signal(1000),
            signal(1000),
            signal(1000),
            signal(0),
            signal(0),
        );

        let executive = ExecutiveAgency::select(
            std::slice::from_ref(&current_goal),
            std::slice::from_ref(&candidate),
            agency_policy(),
        );

        GoalConflictArbitration::arbitrate(executive.selected(), &[], None, arbitration_policy())
            .selected()
            .to_vec()
    }

    fn plan_with(
        goal_identity: CognitiveStructure,
        required_state: CognitiveStructure,
        first_action: CognitiveStructure,
        first_outcome: CognitiveStructure,
        second_action: CognitiveStructure,
        second_outcome: CognitiveStructure,
    ) -> ExecutableMultiStepIntention {
        let sources = source(
            goal_identity.clone(),
            first_action.clone(),
            first_outcome.clone(),
        );

        let candidate = MultiStepIntentionCandidate::new(
            goal_identity,
            vec![
                GroundedIntentionStep::new(
                    required_state,
                    first_action,
                    first_outcome.clone(),
                    signal(1000),
                    signal(1000),
                    signal(0),
                ),
                GroundedIntentionStep::new(
                    first_outcome,
                    second_action,
                    second_outcome,
                    signal(1000),
                    signal(1000),
                    signal(0),
                ),
            ],
            signal(1000),
        )
        .unwrap();

        MultiStepIntention::select(
            &sources,
            std::slice::from_ref(&candidate),
            intention_policy(),
        )
        .selected()[0]
            .clone()
    }

    fn default_plan() -> ExecutableMultiStepIntention {
        plan_with(atom(1), atom(500), atom(10), atom(110), atom(11), atom(111))
    }

    fn monitoring_policy() -> IntentionExecutionMonitoringPolicy {
        IntentionExecutionMonitoringPolicy::new(16, 16, signal(500)).unwrap()
    }

    fn pending_monitoring(
        intention: &ExecutableMultiStepIntention,
    ) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
        IntentionExecutionMonitor::monitor(intention, &[], monitoring_policy())
    }

    fn continuation(
        goal_identity: CognitiveStructure,
        progress: u16,
        cost: u16,
    ) -> ContinuationAssessment {
        ContinuationAssessment::new(
            goal_identity,
            signal(progress),
            signal(1000),
            signal(1000),
            signal(cost),
        )
    }

    fn stop_policy(minimum_value: u16) -> StopReconsiderationPolicy {
        StopReconsiderationPolicy::new(3, signal(500), signal(500), signal(minimum_value)).unwrap()
    }

    fn exploration_policy() -> ExplorationExploitationPolicy {
        ExplorationExploitationPolicy::new(
            16,
            16,
            ExplorationExploitationThresholds::new(
                signal(500),
                signal(500),
                signal(100),
                signal(100),
                signal(100),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn integrated_policy(minimum_continuation_value: u16) -> IntegratedExecutiveControlPolicy {
        IntegratedExecutiveControlPolicy::new(
            stop_policy(minimum_continuation_value),
            exploration_policy(),
        )
    }
    pub fn g() -> ExecutiveGoal {
        goal(atom(1), 0)
    }
    pub fn i() -> ExecutableMultiStepIntention {
        default_plan()
    }
    pub fn m(i: &ExecutableMultiStepIntention) -> IntentionExecutionMonitoringResult {
        pending_monitoring(i)
    }
    pub fn c() -> ContinuationAssessment {
        continuation(atom(1), 600, 0)
    }
    pub fn p() -> IntegratedExecutiveControlPolicy {
        integrated_policy(100)
    }
}

mod fx49r {
    use super::*;
    struct Trace {
        base: u64,
        fixed: u64,
        terminal: u64,
    }
    struct Abs {
        left: u64,
        right: u64,
        fixed: u64,
        terminal: u64,
        support: usize,
        success: u16,
        confidence: u16,
    }
    fn s(v: u16) -> CognitiveSignal {
        CognitiveSignal::new(v).unwrap()
    }

    fn a(v: u64) -> CognitiveStructure {
        CognitiveStructure::atom(v)
    }

    fn step(
        state: CognitiveStructure,
        action: CognitiveStructure,
        outcome: CognitiveStructure,
        confidence: u16,
    ) -> GroundedSkillStep {
        GroundedSkillStep::new(state, action, outcome, s(confidence)).unwrap()
    }

    fn candidate(
        t: &Trace,
        support: usize,
        success: u16,
        confidence: u16,
    ) -> RepeatedSkillCandidate {
        let episodes: Vec<_> = (0..support)
            .map(|_| {
                GroundedSkillEpisode::new(
                    a(t.base),
                    a(t.fixed),
                    vec![
                        step(a(t.base), a(t.base + 10), a(t.base + 110), confidence),
                        step(a(t.base + 110), a(t.fixed), a(t.terminal), confidence),
                    ],
                    s(success),
                )
                .unwrap()
            })
            .collect();

        let memory = SkillMemoryFoundation::build(
            &episodes,
            SkillMemoryPolicy::new(64, 16, 64, 64, s(1), s(1)).unwrap(),
        );

        RepeatedSkillCandidateDiscovery::discover(
            memory.entries(),
            RepeatedSkillCandidatePolicy::new(64, 64, 16, 64, 2, s(1), s(1)).unwrap(),
        )
        .candidates()[0]
            .clone()
    }

    fn abstraction(spec: &Abs) -> StructuralSkillAbstractionEvidence {
        StructuralSkillAbstractionInduction::induce(
            &[
                candidate(
                    &Trace {
                        base: spec.left,
                        fixed: spec.fixed,
                        terminal: spec.terminal,
                    },
                    spec.support,
                    spec.success,
                    spec.confidence,
                ),
                candidate(
                    &Trace {
                        base: spec.right,
                        fixed: spec.fixed,
                        terminal: spec.terminal,
                    },
                    spec.support,
                    spec.success,
                    spec.confidence,
                ),
            ],
            StructuralSkillAbstractionPolicy::new(16, 16, 16, 16, 2, s(1), s(1)).unwrap(),
        )
        .abstractions()[0]
            .clone()
    }

    fn generalization(left: Abs, right: Abs) -> CrossContextSkillGeneralizationEvidence {
        CrossContextSkillGeneralization::generalize(
            &[abstraction(&left), abstraction(&right)],
            CrossContextSkillGeneralizationPolicy::new(16, 16, 16, 16, 1, s(1), s(1)).unwrap(),
        )
        .generalizations()[0]
            .clone()
    }

    fn record(fixed_left: u64, fixed_right: u64, support: usize) -> CompressedSkillRecord {
        let g = generalization(
            Abs {
                left: 100,
                right: 200,
                fixed: fixed_left,
                terminal: 70,
                support,
                success: 1000,
                confidence: 1000,
            },
            Abs {
                left: 300,
                right: 400,
                fixed: fixed_right,
                terminal: 80,
                support,
                success: 1000,
                confidence: 1000,
            },
        );

        LossControlledSkillCompression::compress_all(
            std::slice::from_ref(&g),
            SkillCompressionPolicy::new(
                SkillCompressionBounds::new(16, 16, 16, 16).unwrap(),
                SkillCompressionThresholds::new(1, s(1), s(1), 0).unwrap(),
            ),
        )
        .records()[0]
            .clone()
    }
    pub fn r() -> CompressedSkillRecord {
        record(7, 7, 2)
    }
}

mod fx49 {
    use super::*;
    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn binding(id: usize, value: u64) -> GroundedSkillSlotBinding {
        GroundedSkillSlotBinding::new(SkillReuseSlotKind::Structural, id, a(value), s(900)).unwrap()
    }

    fn request() -> GroundedSkillReuseRequest {
        GroundedSkillReuseRequest::new(a(900), a(7), vec![binding(1, 910), binding(2, 1010)])
    }

    fn observation(
        state: u64,
        action: u64,
        outcome: u64,
        confidence: u16,
    ) -> SkillExecutionObservation {
        SkillExecutionObservation::new(a(state), a(action), a(outcome), s(confidence)).unwrap()
    }

    fn exact_observations() -> Vec<SkillExecutionObservation> {
        vec![
            observation(900, 910, 1010, 900),
            observation(1010, 7, 7, 900),
        ]
    }

    fn use_evidence(accesses: usize, successes: usize, failures: usize) -> SkillMemoryUseEvidence {
        SkillMemoryUseEvidence::new(accesses, successes, failures, s(900)).unwrap()
    }

    fn consolidation_thresholds() -> SkillMemoryConsolidationThresholds {
        SkillMemoryConsolidationThresholds::new(s(700), s(500), s(200), 3, 5).unwrap()
    }

    fn policy() -> IntegratedSkillLearningCyclePolicy {
        IntegratedSkillLearningCyclePolicy::new(
            SkillReusePolicy::new(
                SkillReuseBounds::new(16, 16, 16, 16, 1).unwrap(),
                SkillReuseThresholds::new(1, 1, s(500), s(500), s(500)).unwrap(),
            ),
            SkillOutcomeFeedbackPolicy::new(16, 16, s(500)).unwrap(),
            SkillRevisionMemoryPolicy::new(16, 200, 100, s(500), s(500)).unwrap(),
            SkillMemoryConsolidationPolicy::new(
                SkillMemoryConsolidationBounds::new(16, 16, 16, 16, 16, 16).unwrap(),
                consolidation_thresholds(),
            ),
        )
    }

    fn input(
        memory: SkillRevisionMemoryEntry,
        observations: Vec<SkillExecutionObservation>,
        usage: SkillMemoryUseEvidence,
    ) -> IntegratedSkillLearningCycleInput {
        IntegratedSkillLearningCycleInput::new(memory, request(), observations, usage)
    }
    pub fn i() -> IntegratedSkillLearningCycleInput {
        input(
            SkillRevisionMemoryEntry::new(super::fx49r::r()),
            exact_observations(),
            use_evidence(5, 4, 0),
        )
    }
    pub fn p() -> IntegratedSkillLearningCyclePolicy {
        policy()
    }
}

mod fx50 {
    use super::*;
    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn belief(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        HypothesisBeliefState::new(a(hypothesis), s(confidence)).unwrap()
    }

    fn foundation_policy() -> ActiveExperimentPolicy {
        ActiveExperimentPolicy::new(
            ActiveExperimentBounds::new(32, 32, 32).unwrap(),
            ActiveExperimentThresholds::new(s(500), s(500), s(500), s(500)).unwrap(),
        )
    }

    fn proposal_policy(max_possibilities: usize) -> BeliefDrivenExperimentProposalPolicy {
        BeliefDrivenExperimentProposalPolicy::new(
            foundation_policy(),
            BeliefDrivenExperimentProposalBounds::new(16, max_possibilities, 16, 16).unwrap(),
            s(500),
            s(500),
        )
        .unwrap()
    }

    fn learning_policy() -> LearningProgressPolicy {
        LearningProgressPolicy::new(
            LearningProgressBounds::new(32, 16, 8).unwrap(),
            LearningProgressThresholds::new(s(500), 2, s(50)).unwrap(),
        )
        .unwrap()
    }

    fn sequence_policy(max_expansions: usize) -> ExperimentSequencePlanningPolicy {
        ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(16, 16, 4, max_expansions, 8).unwrap(),
            s(500),
        )
        .unwrap()
    }

    fn control_policy(minimum_information: u16) -> StopContinueExperimentationPolicy {
        StopContinueExperimentationPolicy::new(
            StopContinueExperimentationBounds::new(16, 8, 8).unwrap(),
            StopContinueExperimentationThresholds::new(
                s(500),
                s(850),
                s(250),
                s(100),
                s(minimum_information),
                s(500),
            )
            .unwrap(),
        )
    }

    fn policy() -> IntegratedAutonomousExperimentationPolicy {
        IntegratedAutonomousExperimentationPolicy::new(
            proposal_policy(16),
            learning_policy(),
            sequence_policy(64),
            control_policy(600),
        )
        .unwrap()
    }
    pub fn b() -> Vec<HypothesisBeliefState> {
        vec![belief(1, 700)]
    }
    pub fn p() -> IntegratedAutonomousExperimentationPolicy {
        policy()
    }
}

fn s(v: u16) -> CognitiveSignal {
    CognitiveSignal::new(v).unwrap()
}

fn a(v: u64) -> CognitiveStructure {
    CognitiveStructure::atom(v)
}

fn frame(id: u64, xs: &[(u64, u64)]) -> PerceptualFrame {
    PerceptualFrame::new(
        id,
        xs.iter()
            .map(|(h, v)| PerceptualElement::new(PerceptualElementHandle::new(*h), a(*v)))
            .collect(),
    )
    .unwrap()
}

fn perception() -> IntegratedPerceptualWorldInput {
    IntegratedPerceptualWorldInput::new(
        frame(1, &[(1, 10)]),
        frame(2, &[(1, 10), (2, 20)]),
        IntegratedPerceptualWorldCandidates::new(vec![], vec![], vec![], vec![], vec![], vec![]),
    )
    .unwrap()
}

fn perceptual_context() -> IntegratedPerceptualWorldContext {
    IntegratedPerceptualWorldContext::new(
        PerceptualGroundingPolicy::new(8, 8).unwrap(),
        PersistenceTrackingPolicy::new(8, 8, 16).unwrap(),
        TopologicalRelationPolicy::new(8, 16).unwrap(),
        PerceptualChangePolicy::new(8, 16).unwrap(),
        ActionConsequencePolicy::new(8, 8, 16).unwrap(),
    )
}

fn perceptual_policy() -> PerceptualGroundingIngestionPolicy {
    PerceptualGroundingIngestionPolicy::new(
        PerceptualGroundingIngestionBounds::new(8, 8, 32, 32).unwrap(),
    )
}

fn domain_policy() -> IntegratedDomainModelPolicy {
    IntegratedDomainModelPolicy::new(8, 8, 16).unwrap()
}

fn domain_ingestion() -> UniversalDomainLearningIngestionPolicy {
    UniversalDomainLearningIngestionPolicy::new(
        UniversalDomainLearningIngestionBounds::new(8, 8, 16, 8).unwrap(),
    )
}

fn agent_policy() -> IntegratedAgentPolicy {
    IntegratedAgentPolicy::new(
        IntegratedAgentBounds::new(5, 2000).unwrap(),
        IntegratedAgentThresholds::new(s(500)).unwrap(),
    )
}

fn loop_policy(max_steps: usize) -> BoundedRecurrentAgentLoopPolicy {
    BoundedRecurrentAgentLoopPolicy::new(max_steps).unwrap()
}

struct StepFixture {
    pg: PerceptualGroundingIngestionRequest,
    domain: UniversalDomainLearningIngestionRequest,
    executive: ExecutiveAgencyIngestionRequest,
    skill: MetaLearningSkillMemoryIngestionRequest,
    experiment: AutonomousExperimentationIngestionRequest,
    perceptual_input: IntegratedPerceptualWorldInput,
    goal: ExecutiveGoal,
    intention: ExecutableMultiStepIntention,
    monitoring: IntentionExecutionMonitoringResult,
    continuation: ContinuationAssessment,
    skill_input: IntegratedSkillLearningCycleInput,
    beliefs: Vec<HypothesisBeliefState>,
    transition: CognitiveCycleStateTransitionRequest,
}

impl StepFixture {
    fn new(
        anchor_value: u64,
        result_base: u64,
        provenance_base: u64,
        preserve_anchor: bool,
    ) -> Self {
        let anchor = a(anchor_value);

        let pg = PerceptualGroundingIngestionRequest::new(
            anchor.clone(),
            a(result_base + 1),
            a(provenance_base),
            s(900),
            s(200),
        )
        .unwrap();

        let domain = UniversalDomainLearningIngestionRequest::new(
            a(7000 + result_base),
            anchor.clone(),
            a(result_base + 2),
            a(provenance_base + 100),
            s(900),
            s(250),
        )
        .unwrap();

        let executive = ExecutiveAgencyIngestionRequest::new(
            anchor.clone(),
            a(result_base + 3),
            a(provenance_base + 200),
            s(900),
            s(225),
        )
        .unwrap();

        let skill = MetaLearningSkillMemoryIngestionRequest::new(
            anchor.clone(),
            a(result_base + 4),
            a(provenance_base + 300),
            s(900),
            s(200),
        )
        .unwrap();

        let experiment = AutonomousExperimentationIngestionRequest::new(
            anchor.clone(),
            a(result_base + 5),
            a(provenance_base + 400),
            s(900),
            s(225),
        )
        .unwrap();

        let perceptual_input = perception();

        let goal = fx48::g();
        let intention = fx48::i();
        let monitoring = fx48::m(&intention);
        let continuation = fx48::c();

        let skill_input = fx49::i();
        let beliefs = fx50::b();

        let transition = if preserve_anchor {
            CognitiveCycleStateTransitionRequest::new(
                anchor,
                CognitiveCycleTransitionAuthority::PreserveAnchor,
                None,
            )
            .unwrap()
        } else {
            CognitiveCycleStateTransitionRequest::new(
                anchor,
                CognitiveCycleTransitionAuthority::AdoptLayer(
                    IntegratedCognitiveLayer::ExecutiveAgency,
                ),
                Some(a(provenance_base + 200)),
            )
            .unwrap()
        };

        Self {
            pg,
            domain,
            executive,
            skill,
            experiment,
            perceptual_input,
            goal,
            intention,
            monitoring,
            continuation,
            skill_input,
            beliefs,
            transition,
        }
    }

    fn input<'a>(
        &'a self,
        feedback: Option<RecurrentFeedbackEvidence>,
    ) -> OnlineRecurrentCognitiveStepInput<'a> {
        let executive_context = IntegratedExecutiveControlContext::new(
            &self.goal,
            Some(&self.intention),
            &self.monitoring,
            None,
            Some(&self.continuation),
            ReconsiderationState::default(),
            &[],
        );

        let runtime = OnlineCognitiveOrchestrationInput::new(
            OnlinePerceptualGroundingRuntime::new(
                &self.pg,
                &self.perceptual_input,
                perceptual_context(),
                perceptual_policy(),
            ),
            OnlineUniversalDomainRuntime::new(
                &self.domain,
                &[],
                &[],
                domain_policy(),
                domain_ingestion(),
            ),
            OnlineExecutiveAgencyRuntime::new(
                &self.executive,
                executive_context,
                fx48::p(),
            ),
            OnlineSkillMemoryRuntime::new(
                &self.skill,
                &self.skill_input,
                fx49::p(),
            ),
            OnlineAutonomousExperimentationRuntime::new(
                &self.experiment,
                &self.beliefs,
                &[],
                &[],
                0,
                fx50::p(),
            ),
        );

        OnlineRecurrentCognitiveStepInput::new(
            runtime,
            &self.transition,
            feedback,
        )
    }
}

fn fb(
    predecessor_step_index: usize,
    predecessor_anchor: u64,
    predecessor_provenance: Option<u64>,
) -> RecurrentFeedbackEvidence {
    RecurrentFeedbackEvidence::new(
        predecessor_step_index,
        a(predecessor_anchor),
        predecessor_provenance.map(a),
    )
}

fn valid_two_step(facade: bool) -> OnlineRecurrentCognitiveLoopResult {
    let first = StepFixture::new(1000, 1000, 9000, false);
    let second = StepFixture::new(1003, 2000, 10000, false);

    let inputs = vec![
        first.input(None),
        second.input(Some(fb(0, 1003, Some(9200)))),
    ];

    if facade {
        UniversalOnlineRecurrentCognitiveLoop::evaluate(
            &a(1000),
            inputs,
            agent_policy(),
            loop_policy(2),
        )
    } else {
        OnlineRecurrentCognitiveLoop::run(
            &a(1000),
            inputs,
            agent_policy(),
            loop_policy(2),
        )
    }
}

#[test]
fn executable_online_step_dispatches_exact_m48_selection() {
    let recurrent = valid_two_step(false);

    assert_eq!(
        recurrent.status(),
        OnlineRecurrentCognitiveLoopStatus::Completed
    );

    let online = &recurrent.steps()[0];

    assert_eq!(
        online.executive_decision(),
        Some(
            athlesia_executive_agency::IntegratedExecutiveControlDecision::ExecuteCurrent
        )
    );

    let selection = online
        .executive_selection()
        .expect("executable online step must retain executive selection");

    assert_eq!(
        selection.source(),
        athlesia_executive_agency::IntegratedExecutiveSelectionSource::CurrentIntention
    );

    let dispatch = EnvironmentInteractionBoundary::dispatch(online);

    assert_eq!(dispatch.status(), EnvironmentActionDispatchStatus::Ready);
    assert!(dispatch.ready());
    assert!(!dispatch.rejected());

    let dispatch = dispatch
        .dispatch()
        .expect("ready boundary result must contain dispatch");

    assert_eq!(dispatch.source_anchor_state(), &a(1000));
    assert_eq!(dispatch.selection(), selection);
    assert_eq!(dispatch.action(), selection.action());
    assert_eq!(dispatch.predicted_outcome(), selection.predicted_outcome());
}

#[test]
fn second_recurrent_step_dispatches_from_exact_previous_anchor() {
    let recurrent = valid_two_step(false);

    let online = &recurrent.steps()[1];

    assert_eq!(online.next_anchor_state(), Some(&a(2003)));

    let dispatch = EnvironmentInteractionBoundary::dispatch(online);

    assert_eq!(dispatch.status(), EnvironmentActionDispatchStatus::Ready);

    let dispatch = dispatch
        .dispatch()
        .expect("ready boundary result must contain dispatch");

    assert_eq!(dispatch.source_anchor_state(), &a(1003));
    assert_ne!(dispatch.source_anchor_state(), &a(2003));
}

#[test]
fn environment_observation_binds_exact_canonical_feedback_evidence() {
    let recurrent = valid_two_step(false);
    let online = &recurrent.steps()[0];

    let dispatch_result = EnvironmentInteractionBoundary::dispatch(online);
    let dispatch = dispatch_result
        .dispatch()
        .expect("accepted executable step must dispatch");

    let observation = EnvironmentInteractionObservation::new(
        41,
        a(7777),
        s(850),
    )
    .expect("positive confidence observation must be valid");

    let evidence =
        EnvironmentInteractionBoundary::bind_observation(dispatch, &observation)
            .expect("valid environment observation must bind");

    assert_eq!(evidence.action_observation().event_index(), 41);
    assert_eq!(
        evidence.action_observation().source(),
        athlesia_core_knowledge_perceptual_grounding::ActionSource::SelfGenerated
    );
    assert_eq!(
        evidence.action_observation().descriptor(),
        dispatch.action()
    );

    assert_eq!(
        evidence.execution_observation().observed_state(),
        dispatch.source_anchor_state()
    );
    assert_eq!(
        evidence.execution_observation().observed_action(),
        dispatch.action()
    );
    assert_eq!(
        evidence.execution_observation().observed_outcome(),
        &a(7777)
    );
    assert_eq!(
        evidence.execution_observation().observation_confidence(),
        s(850)
    );

    assert_eq!(
        evidence.experiment_observation().source_state(),
        dispatch.source_anchor_state()
    );
    assert_eq!(
        evidence.experiment_observation().action(),
        dispatch.action()
    );
    assert_eq!(
        evidence.experiment_observation().observed_outcome(),
        &a(7777)
    );
    assert_eq!(
        evidence.experiment_observation().confidence(),
        s(850)
    );
}

#[test]
fn zero_confidence_environment_observation_is_rejected() {
    assert!(
        EnvironmentInteractionObservation::new(
            7,
            a(7000),
            s(0),
        )
        .is_none()
    );
}

#[test]
fn rejected_online_step_cannot_dispatch_environment_action() {
    let stale = StepFixture::new(
        999,
        1000,
        9000,
        false,
    );

    let recurrent = OnlineRecurrentCognitiveLoop::run(
        &a(1000),
        vec![stale.input(None)],
        agent_policy(),
        loop_policy(1),
    );

    assert_eq!(
        recurrent.status(),
        OnlineRecurrentCognitiveLoopStatus::OnlineStepRejected
    );

    assert_eq!(recurrent.executed_step_count(), 1);
    assert_eq!(recurrent.completed_step_count(), 0);

    let rejected_online = &recurrent.steps()[0];

    assert!(rejected_online.rejected());

    let dispatch =
        EnvironmentInteractionBoundary::dispatch(rejected_online);

    assert_eq!(
        dispatch.status(),
        EnvironmentActionDispatchStatus::OnlineStepRejected
    );
    assert!(dispatch.rejected());
    assert!(dispatch.dispatch().is_none());
}

#[test]
fn environment_boundary_is_deterministic_and_facade_equivalent() {
    let recurrent = valid_two_step(false);
    let online = &recurrent.steps()[0];

    let direct_a = EnvironmentInteractionBoundary::dispatch(online);
    let direct_b = EnvironmentInteractionBoundary::dispatch(online);
    let facade = UniversalEnvironmentInteractionBoundary::dispatch(online);

    assert_eq!(direct_a, direct_b);
    assert_eq!(direct_a, facade);

    let dispatch = direct_a
        .dispatch()
        .expect("accepted executable step must dispatch");

    let observation = EnvironmentInteractionObservation::new(
        99,
        a(9900),
        s(900),
    )
    .expect("positive confidence observation must be valid");

    let direct_evidence =
        EnvironmentInteractionBoundary::bind_observation(
            dispatch,
            &observation,
        );

    let repeated_evidence =
        EnvironmentInteractionBoundary::bind_observation(
            dispatch,
            &observation,
        );

    let facade_evidence =
        UniversalEnvironmentInteractionBoundary::bind_observation(
            dispatch,
            &observation,
        );

    assert_eq!(direct_evidence, repeated_evidence);
    assert_eq!(direct_evidence, facade_evidence);
}


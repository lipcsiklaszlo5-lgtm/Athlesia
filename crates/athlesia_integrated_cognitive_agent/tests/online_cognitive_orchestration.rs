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

#[derive(Clone, Copy)]
struct Config {
    domain_anchor: u64,
    experiment_provenance: u64,
    confidence: u16,
    transition_provenance: u64,
    preserve: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            domain_anchor: 1000,
            experiment_provenance: 9400,
            confidence: 900,
            transition_provenance: 9200,
            preserve: false,
        }
    }
}

fn run(config: Config, facade: bool) -> OnlineCognitiveOrchestrationResult {
    let anchor = a(1000);

    let pg = PerceptualGroundingIngestionRequest::new(
        anchor.clone(),
        a(1001),
        a(9000),
        s(config.confidence),
        s(200),
    )
    .unwrap();

    let domain = UniversalDomainLearningIngestionRequest::new(
        a(7000),
        a(config.domain_anchor),
        a(1002),
        a(9100),
        s(config.confidence),
        s(250),
    )
    .unwrap();

    let executive = ExecutiveAgencyIngestionRequest::new(
        anchor.clone(),
        a(1003),
        a(9200),
        s(config.confidence),
        s(225),
    )
    .unwrap();

    let skill = MetaLearningSkillMemoryIngestionRequest::new(
        anchor.clone(),
        a(1004),
        a(9300),
        s(config.confidence),
        s(200),
    )
    .unwrap();

    let experiment = AutonomousExperimentationIngestionRequest::new(
        anchor.clone(),
        a(1005),
        a(config.experiment_provenance),
        s(config.confidence),
        s(225),
    )
    .unwrap();

    let perceptual_input = perception();

    let goal = fx48::g();
    let intention = fx48::i();
    let monitoring = fx48::m(&intention);
    let continuation = fx48::c();

    let executive_context = IntegratedExecutiveControlContext::new(
        &goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        ReconsiderationState::default(),
        &[],
    );

    let skill_input = fx49::i();
    let beliefs = fx50::b();

    let transition = if config.preserve {
        CognitiveCycleStateTransitionRequest::new(
            anchor.clone(),
            CognitiveCycleTransitionAuthority::PreserveAnchor,
            None,
        )
        .unwrap()
    } else {
        CognitiveCycleStateTransitionRequest::new(
            anchor.clone(),
            CognitiveCycleTransitionAuthority::AdoptLayer(
                IntegratedCognitiveLayer::ExecutiveAgency,
            ),
            Some(a(config.transition_provenance)),
        )
        .unwrap()
    };

    let input = OnlineCognitiveOrchestrationInput::new(
        OnlinePerceptualGroundingRuntime::new(
            &pg,
            &perceptual_input,
            perceptual_context(),
            perceptual_policy(),
        ),
        OnlineUniversalDomainRuntime::new(&domain, &[], &[], domain_policy(), domain_ingestion()),
        OnlineExecutiveAgencyRuntime::new(&executive, executive_context, fx48::p()),
        OnlineSkillMemoryRuntime::new(&skill, &skill_input, fx49::p()),
        OnlineAutonomousExperimentationRuntime::new(&experiment, &beliefs, &[], &[], 0, fx50::p()),
    );

    if facade {
        UniversalOnlineCognitiveOrchestration::evaluate(&anchor, input, agent_policy(), &transition)
    } else {
        OnlineCognitiveOrchestration::run(&anchor, input, agent_policy(), &transition)
    }
}

#[test]
fn real_five_layer_runtime_advances() {
    let r = run(Config::default(), false);
    assert!(r.advanced());
    assert_eq!(r.next_anchor_state(), Some(&a(1003)));
    assert_eq!(r.contributions().len(), 5);
}

#[test]
fn runtime_contributions_are_canonical() {
    let r = run(Config::default(), false);
    assert_eq!(
        r.contributions()
            .iter()
            .map(|x| x.layer())
            .collect::<Vec<_>>(),
        IntegratedCognitiveCycle::required_layers().to_vec(),
    );
}

#[test]
fn runtime_preserves_exact_states_and_provenance() {
    let r = run(Config::default(), false);

    for (layer, state, provenance) in [
        (IntegratedCognitiveLayer::PerceptualGrounding, 1001, 9000),
        (
            IntegratedCognitiveLayer::UniversalDomainLearning,
            1002,
            9100,
        ),
        (IntegratedCognitiveLayer::ExecutiveAgency, 1003, 9200),
        (
            IntegratedCognitiveLayer::MetaLearningSkillMemory,
            1004,
            9300,
        ),
        (
            IntegratedCognitiveLayer::AutonomousExperimentation,
            1005,
            9400,
        ),
    ] {
        let c = r.contribution(layer).unwrap();
        assert_eq!(c.result_state(), &a(state));
        assert_eq!(c.provenance(), &a(provenance));
    }
}

#[test]
fn real_m48_decision_survives_ingestion() {
    assert_eq!(
        run(Config::default(), false).executive_decision(),
        Some(IntegratedExecutiveControlDecision::ExecuteCurrent),
    );
}

#[test]
fn real_m49_integrated_cycle_executes() {
    let r = run(Config::default(), false);
    assert!(r.skill_reused().is_some());
    assert!(r
        .contribution(IntegratedCognitiveLayer::MetaLearningSkillMemory)
        .is_some());
}

#[test]
fn real_m50_cycle_stops_resolved() {
    assert_eq!(
        run(Config::default(), false).experimentation_status(),
        Some(IntegratedAutonomousExperimentationStatus::StopResolved),
    );
}

#[test]
fn stale_request_anchor_rejects_before_execution() {
    let r = run(
        Config {
            domain_anchor: 9999,
            ..Config::default()
        },
        false,
    );

    assert_eq!(
        r.status(),
        OnlineCognitiveOrchestrationStatus::RequestAnchorMismatch(
            IntegratedCognitiveLayer::UniversalDomainLearning,
        ),
    );

    assert!(r.contributions().is_empty());
}

#[test]
fn low_confidence_cannot_manufacture_cycle() {
    let r = run(
        Config {
            confidence: 400,
            ..Config::default()
        },
        false,
    );

    assert_eq!(
        r.status(),
        OnlineCognitiveOrchestrationStatus::CognitiveCycleRejected,
    );
    assert!(r.next_anchor_state().is_none());
}

#[test]
fn cross_layer_provenance_collision_is_atomic() {
    let r = run(
        Config {
            experiment_provenance: 9000,
            ..Config::default()
        },
        false,
    );

    assert_eq!(
        r.status(),
        OnlineCognitiveOrchestrationStatus::CognitiveCycleRejected,
    );
    assert!(r.next_anchor_state().is_none());
}

#[test]
fn stale_transition_provenance_rejects() {
    let r = run(
        Config {
            transition_provenance: 9999,
            ..Config::default()
        },
        false,
    );

    assert_eq!(
        r.status(),
        OnlineCognitiveOrchestrationStatus::StateTransitionRejected,
    );
    assert!(r.next_anchor_state().is_none());
}

#[test]
fn explicit_preserve_authority_preserves_anchor() {
    let r = run(
        Config {
            preserve: true,
            ..Config::default()
        },
        false,
    );

    assert!(r.preserved());
    assert_eq!(r.next_anchor_state(), Some(&a(1000)));
}

#[test]
fn runtime_is_deterministic_and_facade_equivalent() {
    let direct = run(Config::default(), false);
    let facade = run(Config::default(), true);
    let repeated = run(Config::default(), true);

    assert_eq!(direct, facade);
    assert_eq!(facade, repeated);
}

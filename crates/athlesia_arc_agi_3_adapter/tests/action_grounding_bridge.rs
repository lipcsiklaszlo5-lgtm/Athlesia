use athlesia_arc_agi_3_adapter::{
    action_grounding_bridge::{
        ArcAgi3ActionGroundingBridge, ArcAgi3ActionGroundingError,
        UniversalArcAgi3ActionGroundingBridge,
    },
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
};
use athlesia_autonomous_active_experimentation::{
    AutonomousExperimentProposal, ExperimentEvidence,
};
use athlesia_executive_agency::{ExecutiveGoal, GroundedExecutiveActionCandidate};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn observation(state: ArcAgi3GameState, actions: Vec<ArcAgi3ActionId>) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new("action-grounding-test".to_string()).unwrap(),
        state,
        ArcAgi3FrameSequence::new(vec![
            ArcAgi3Grid::from_rows(vec![vec![1, 2], vec![3, 4]]).unwrap()
        ])
        .unwrap(),
        0,
        3,
        ArcAgi3AvailableActions::new(actions).unwrap(),
        None,
    )
}

fn action_structure(action: ArcAgi3Action) -> CognitiveStructure {
    ArcAgi3CognitiveProtocolBridge::encode_action(action)
}

fn candidate(
    action: ArcAgi3Action,
    predicted_outcome: CognitiveStructure,
) -> GroundedExecutiveActionCandidate {
    GroundedExecutiveActionCandidate::new(
        CognitiveStructure::atom(100),
        action_structure(action),
        predicted_outcome,
        signal(610),
        signal(720),
        signal(830),
        signal(540),
        signal(90),
    )
}

fn evidence() -> ExperimentEvidence {
    ExperimentEvidence::new(
        signal(450),
        signal(730),
        signal(820),
        signal(910),
        signal(70),
    )
    .unwrap()
}

fn proposal(
    source_state: CognitiveStructure,
    action: ArcAgi3Action,
    predicted_outcome: CognitiveStructure,
) -> AutonomousExperimentProposal {
    AutonomousExperimentProposal::new(
        source_state,
        action_structure(action),
        predicted_outcome,
        evidence(),
    )
}

#[test]
fn available_action_authorizes_existing_m48_candidate_without_rewriting_evidence() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1, ArcAgi3ActionId::Action6],
    );

    let original = candidate(
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap(),
        CognitiveStructure::ordered(vec![
            CognitiveStructure::atom(7),
            CognitiveStructure::atom(8),
        ])
        .unwrap(),
    );

    let authorized =
        ArcAgi3ActionGroundingBridge::authorize_executive_candidate(&observation, &original)
            .unwrap();

    assert_eq!(authorized.candidate(), &original);

    assert_eq!(
        authorized.action(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1,).unwrap(),
    );

    assert_eq!(
        authorized.candidate().predicted_outcome(),
        original.predicted_outcome(),
    );

    assert_eq!(
        authorized.candidate().goal_alignment(),
        original.goal_alignment(),
    );

    assert_eq!(
        authorized.candidate().controllability(),
        original.controllability(),
    );

    assert_eq!(
        authorized.candidate().evidence_confidence(),
        original.evidence_confidence(),
    );

    assert_eq!(
        authorized.candidate().information_gain(),
        original.information_gain(),
    );

    assert_eq!(
        authorized.candidate().execution_cost(),
        original.execution_cost(),
    );
}

#[test]
fn unavailable_action_rejects_grounded_candidate_without_reinterpretation() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let original = candidate(
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap(),
        CognitiveStructure::atom(333),
    );

    assert_eq!(
        ArcAgi3ActionGroundingBridge::authorize_executive_candidate(&observation, &original,),
        Err(ArcAgi3ActionGroundingError::ActionUnavailable),
    );
}

#[test]
fn malformed_cognitive_action_is_rejected_instead_of_mapped_to_arc_action() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let malformed = GroundedExecutiveActionCandidate::new(
        CognitiveStructure::atom(1),
        CognitiveStructure::atom(999_999),
        CognitiveStructure::atom(2),
        signal(100),
        signal(200),
        signal(300),
        signal(400),
        signal(50),
    );

    assert!(matches!(
        ArcAgi3ActionGroundingBridge::authorize_executive_candidate(&observation, &malformed,),
        Err(ArcAgi3ActionGroundingError::Codec(_))
    ));
}

#[test]
fn executive_candidate_cannot_smuggle_protocol_reset() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let reset_candidate = candidate(ArcAgi3Action::reset(), CognitiveStructure::atom(5));

    assert_eq!(
        ArcAgi3ActionGroundingBridge::authorize_executive_candidate(&observation, &reset_candidate,),
        Err(ArcAgi3ActionGroundingError::ExecutiveResetForbidden),
    );
}

#[test]
fn terminal_game_rejects_non_reset_executive_candidate() {
    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    for state in [
        ArcAgi3GameState::NotPlayed,
        ArcAgi3GameState::Win,
        ArcAgi3GameState::GameOver,
    ] {
        let observation = observation(state, vec![ArcAgi3ActionId::Action1]);

        let grounded = candidate(action, CognitiveStructure::atom(10));

        assert_eq!(
            ArcAgi3ActionGroundingBridge::authorize_executive_candidate(&observation, &grounded,),
            Err(ArcAgi3ActionGroundingError::GameNotActive),
        );
    }
}

#[test]
fn coordinate_action_preserves_exact_action_identity() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action6],
    );

    let action = ArcAgi3Action::coordinate(17, 42).unwrap();

    let grounded = candidate(action, CognitiveStructure::atom(11));

    let authorized =
        ArcAgi3ActionGroundingBridge::authorize_executive_candidate(&observation, &grounded)
            .unwrap();

    assert_eq!(authorized.action(), action);

    assert_eq!(authorized.candidate().action(), grounded.action(),);
}

#[test]
fn m50_proposal_requires_exact_cognitive_source_state() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let expected_source = CognitiveStructure::ordered(vec![
        CognitiveStructure::atom(51),
        CognitiveStructure::atom(52),
    ])
    .unwrap();

    let stale_source = CognitiveStructure::ordered(vec![
        CognitiveStructure::atom(51),
        CognitiveStructure::atom(99),
    ])
    .unwrap();

    let proposal = proposal(
        stale_source,
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap(),
        CognitiveStructure::atom(700),
    );

    assert_eq!(
        ArcAgi3ActionGroundingBridge::authorize_experiment_proposal(
            &observation,
            &expected_source,
            &proposal,
        ),
        Err(ArcAgi3ActionGroundingError::SourceStateMismatch),
    );
}

#[test]
fn m50_proposal_authorization_preserves_prediction_and_experiment_evidence() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let source = CognitiveStructure::atom(1234);

    let original = proposal(
        source.clone(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap(),
        CognitiveStructure::ordered(vec![
            CognitiveStructure::atom(88),
            CognitiveStructure::atom(89),
        ])
        .unwrap(),
    );

    let authorized = ArcAgi3ActionGroundingBridge::authorize_experiment_proposal(
        &observation,
        &source,
        &original,
    )
    .unwrap();

    assert_eq!(authorized.proposal(), &original);

    assert_eq!(
        authorized.proposal().predicted_outcome(),
        original.predicted_outcome(),
    );

    assert_eq!(authorized.proposal().evidence(), original.evidence(),);
}

#[test]
fn experiment_proposal_becomes_m48_candidate_only_from_explicit_real_evidence() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let source = CognitiveStructure::atom(500);

    let goal = ExecutiveGoal::new(CognitiveStructure::atom(600), signal(900), signal(100));

    let predicted = CognitiveStructure::ordered(vec![
        CognitiveStructure::atom(701),
        CognitiveStructure::atom(702),
    ])
    .unwrap();

    let proposal = proposal(
        source.clone(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap(),
        predicted.clone(),
    );

    let explicit_goal_alignment = signal(640);

    let grounded = ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        explicit_goal_alignment,
        &proposal,
    )
    .unwrap();

    let source_evidence = proposal.evidence();

    assert_eq!(grounded.goal_identity(), goal.identity(),);

    assert_eq!(grounded.action(), proposal.action(),);

    assert_eq!(grounded.predicted_outcome(), &predicted,);

    assert_eq!(grounded.goal_alignment(), explicit_goal_alignment,);

    assert_eq!(
        grounded.controllability(),
        source_evidence.controllability(),
    );

    assert_eq!(
        grounded.evidence_confidence(),
        source_evidence.grounding_confidence(),
    );

    assert_eq!(
        grounded.information_gain(),
        source_evidence.expected_information_gain(),
    );

    assert_eq!(grounded.execution_cost(), source_evidence.execution_cost(),);
}

#[test]
fn arc_availability_never_determines_predicted_outcome() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let source = CognitiveStructure::atom(800);

    let goal = ExecutiveGoal::new(CognitiveStructure::atom(801), signal(900), signal(200));

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let first_prediction = CognitiveStructure::atom(901);

    let second_prediction = CognitiveStructure::atom(902);

    let first = proposal(source.clone(), action, first_prediction.clone());

    let second = proposal(source.clone(), action, second_prediction.clone());

    let first_grounded = ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        signal(500),
        &first,
    )
    .unwrap();

    let second_grounded = ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        signal(500),
        &second,
    )
    .unwrap();

    assert_eq!(first_grounded.action(), second_grounded.action(),);

    assert_eq!(first_grounded.predicted_outcome(), &first_prediction,);

    assert_eq!(second_grounded.predicted_outcome(), &second_prediction,);

    assert_ne!(
        first_grounded.predicted_outcome(),
        second_grounded.predicted_outcome(),
    );
}

#[test]
fn goal_alignment_is_explicit_authority_not_derived_from_arc_protocol() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let source = CognitiveStructure::atom(1000);

    let goal = ExecutiveGoal::new(CognitiveStructure::atom(1001), signal(950), signal(100));

    let proposal = proposal(
        source.clone(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap(),
        CognitiveStructure::atom(1002),
    );

    let low = ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        signal(200),
        &proposal,
    )
    .unwrap();

    let high = ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        signal(900),
        &proposal,
    )
    .unwrap();

    assert_eq!(low.goal_alignment(), signal(200),);

    assert_eq!(high.goal_alignment(), signal(900),);

    assert_eq!(low.predicted_outcome(), high.predicted_outcome(),);

    assert_eq!(low.action(), high.action(),);
}

#[test]
fn grounding_is_deterministic_and_non_mutating() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let original_observation = observation.clone();

    let source = CognitiveStructure::atom(44);

    let goal = ExecutiveGoal::new(CognitiveStructure::atom(55), signal(900), signal(100));

    let proposal = proposal(
        source.clone(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap(),
        CognitiveStructure::atom(66),
    );

    let original_proposal = proposal.clone();

    let left = ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        signal(500),
        &proposal,
    );

    let right = ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        signal(500),
        &proposal,
    );

    assert_eq!(left, right);

    assert_eq!(observation, original_observation,);

    assert_eq!(proposal, original_proposal,);
}

#[test]
fn universal_facade_matches_direct_grounding() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1],
    );

    let source = CognitiveStructure::atom(90);

    let goal = ExecutiveGoal::new(CognitiveStructure::atom(91), signal(900), signal(0));

    let proposal = proposal(
        source.clone(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap(),
        CognitiveStructure::atom(92),
    );

    let direct = ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        signal(700),
        &proposal,
    );

    let facade = UniversalArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
        &observation,
        &source,
        &goal,
        signal(700),
        &proposal,
    );

    assert_eq!(direct, facade);
}

#[test]
fn bridge_is_compile_time_bound_to_real_m48_and_m50_contracts() {
    let _authorize: fn(
        &ArcAgi3Observation,
        &GroundedExecutiveActionCandidate,
    ) -> Result<
        athlesia_arc_agi_3_adapter::action_grounding_bridge::ArcAgi3AuthorizedExecutiveCandidate,
        ArcAgi3ActionGroundingError,
    > = ArcAgi3ActionGroundingBridge::authorize_executive_candidate;

    let _ground: fn(
        &ArcAgi3Observation,
        &CognitiveStructure,
        &ExecutiveGoal,
        CognitiveSignal,
        &AutonomousExperimentProposal,
    ) -> Result<GroundedExecutiveActionCandidate, ArcAgi3ActionGroundingError> =
        ArcAgi3ActionGroundingBridge::ground_experiment_for_goal;

    let _evidence: ExperimentEvidence = evidence();
}

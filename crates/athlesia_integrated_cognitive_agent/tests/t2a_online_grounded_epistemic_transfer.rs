use athlesia_executive_agency::{
    EpistemicExecutableIntentionStep, EpistemicExecutiveAuthorizationStatus,
    EpistemicExecutiveControl, EpistemicExecutiveControlPolicy,
};
use athlesia_integrated_cognitive_agent::{
    EnvironmentInteractionBoundary, EnvironmentInteractionObservation,
    EpistemicEnvironmentActionDispatchStatus, OnlineGroundedEpisodicTransferMemory,
    OnlineGroundedEpisodicTransferPolicy, OnlineGroundedEpisodicTransferRuntime,
    OnlineGroundedEpisodicTransferStatus, UniversalOnlineGroundedEpisodicTransferRuntime,
};
use athlesia_meta_learning_skill_memory::{
    GroundedEpisodicAnalogyPolicy, GroundedSkillEpisode, GroundedSkillStep,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn s(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).expect("positive bounded signal")
}

fn a(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn source_episode() -> GroundedSkillEpisode {
    GroundedSkillEpisode::new(
        a(100),
        a(700),
        vec![
            GroundedSkillStep::new(a(100), a(110), a(210), s(1000)).expect("valid source prefix"),
            GroundedSkillStep::new(a(210), a(700), a(999), s(1000))
                .expect("valid source continuation"),
        ],
        s(1000),
    )
    .expect("valid source episode")
}

fn executive_policy(minimum: u16) -> EpistemicExecutiveControlPolicy {
    EpistemicExecutiveControlPolicy::new(s(minimum)).expect("valid executive policy")
}

fn transfer_policy() -> OnlineGroundedEpisodicTransferPolicy {
    OnlineGroundedEpisodicTransferPolicy::new(
        GroundedEpisodicAnalogyPolicy::new(16, s(500)).expect("valid analogy policy"),
        executive_policy(500),
    )
}

fn environment_evidence(
    state: u64,
    action: u64,
    outcome: u64,
    confidence: u16,
    event_index: u64,
) -> athlesia_integrated_cognitive_agent::EnvironmentInteractionEvidence {
    let step =
        EpistemicExecutableIntentionStep::new(a(state), a(action), Some(a(outcome)), s(confidence))
            .expect("positive evidence step");

    let authorization =
        EpistemicExecutiveControl::authorize(&a(20_000), &a(state), step, executive_policy(1));

    assert!(authorization.authorized());

    let dispatch = EnvironmentInteractionBoundary::dispatch_epistemic(&a(state), &authorization);

    assert_eq!(
        dispatch.status(),
        EpistemicEnvironmentActionDispatchStatus::Ready
    );

    let dispatch = dispatch
        .dispatch()
        .expect("authorized epistemic selection must dispatch");

    let observation =
        EnvironmentInteractionObservation::new(event_index, a(outcome), s(confidence))
            .expect("positive environment observation");

    EnvironmentInteractionBoundary::bind_epistemic_observation(dispatch, &observation)
        .expect("epistemic dispatch must bind observation")
}

#[test]
fn partial_executive_authority_preserves_unknown_future_without_fabrication() {
    let step = EpistemicExecutableIntentionStep::new(a(1010), a(20_000), None, s(900))
        .expect("valid partial executable step");

    let result =
        EpistemicExecutiveControl::authorize(&a(20_000), &a(1010), step, executive_policy(500));

    assert_eq!(
        result.status(),
        EpistemicExecutiveAuthorizationStatus::Authorized
    );

    let selection = result
        .selection()
        .expect("authorized result retains selection");

    assert_eq!(selection.required_state(), &a(1010));
    assert_eq!(selection.action(), &a(20_000));
    assert_eq!(selection.predicted_outcome(), None);
    assert_eq!(selection.evidence_confidence(), s(900));
}

#[test]
fn epistemic_environment_boundary_dispatches_action_without_requiring_fake_outcome() {
    let step = EpistemicExecutableIntentionStep::new(a(1010), a(20_000), None, s(900))
        .expect("valid partial executable step");

    let authorization =
        EpistemicExecutiveControl::authorize(&a(20_000), &a(1010), step, executive_policy(500));

    let dispatch = EnvironmentInteractionBoundary::dispatch_epistemic(&a(1010), &authorization);

    assert!(dispatch.ready());

    let dispatch = dispatch
        .dispatch()
        .expect("authorized epistemic selection must dispatch");

    assert_eq!(dispatch.source_anchor_state(), &a(1010));
    assert_eq!(dispatch.action(), &a(20_000));
    assert_eq!(dispatch.predicted_outcome(), None);

    let observation = EnvironmentInteractionObservation::new(9, a(30_000), s(850))
        .expect("valid environment observation");

    let evidence =
        EnvironmentInteractionBoundary::bind_epistemic_observation(dispatch, &observation)
            .expect("observation must bind");

    assert_eq!(evidence.execution_observation().observed_state(), &a(1010));
    assert_eq!(
        evidence.execution_observation().observed_action(),
        &a(20_000)
    );
    assert_eq!(
        evidence.execution_observation().observed_outcome(),
        &a(30_000)
    );
    assert_eq!(
        evidence.execution_observation().observation_confidence(),
        s(850)
    );
}

#[test]
fn m51_memory_turns_environment_feedback_into_grounded_episodic_transfer_and_authority() {
    let mut memory = OnlineGroundedEpisodicTransferMemory::new(a(900), a(20_000));

    assert!(memory.remember_source_episode(source_episode()));

    let evidence = environment_evidence(900, 910, 1010, 900, 1);

    assert!(memory.record_environment_evidence(&evidence));
    assert_eq!(memory.observation_count(), 1);

    let result =
        OnlineGroundedEpisodicTransferRuntime::evaluate(&memory, &a(1010), transfer_policy());

    assert_eq!(
        result.status(),
        OnlineGroundedEpisodicTransferStatus::Authorized
    );
    assert!(result.authorized());

    let analogy = result
        .analogy()
        .expect("authorized transfer retains analogy evidence");

    assert_eq!(analogy.candidate_count(), 1);
    assert!(!analogy.conflicting_evidence());

    let selection = result
        .selection()
        .expect("authorized transfer retains executive selection");

    assert_eq!(selection.required_state(), &a(1010));
    assert_eq!(selection.action(), &a(20_000));
    assert_eq!(selection.predicted_outcome(), None);

    let facade = UniversalOnlineGroundedEpisodicTransferRuntime::evaluate(
        &memory,
        &a(1010),
        transfer_policy(),
    );

    assert_eq!(result, facade);
}

#[test]
fn conflicting_target_evidence_causes_runtime_abstention_before_executive_authority() {
    let mut memory = OnlineGroundedEpisodicTransferMemory::new(a(900), a(20_000));

    assert!(memory.remember_source_episode(source_episode()));
    assert!(memory.record_environment_evidence(&environment_evidence(900, 910, 1010, 900, 1)));
    assert!(memory.record_environment_evidence(&environment_evidence(900, 920, 1020, 900, 2)));

    let result =
        OnlineGroundedEpisodicTransferRuntime::evaluate(&memory, &a(1010), transfer_policy());

    assert_eq!(
        result.status(),
        OnlineGroundedEpisodicTransferStatus::AnalogyAbstained
    );
    assert!(result.abstained());
    assert!(result.authorization().is_none());
    assert!(result.selection().is_none());

    let analogy = result
        .analogy()
        .expect("abstention retains epistemic diagnosis");

    assert!(analogy.conflicting_evidence());
    assert!(!analogy.observation_frontier_exceeded());
}

#[test]
fn executive_authority_rejects_transferred_action_when_current_state_is_not_grounded_requirement() {
    let mut memory = OnlineGroundedEpisodicTransferMemory::new(a(900), a(20_000));

    assert!(memory.remember_source_episode(source_episode()));
    assert!(memory.record_environment_evidence(&environment_evidence(900, 910, 1010, 900, 1)));

    let result =
        OnlineGroundedEpisodicTransferRuntime::evaluate(&memory, &a(5555), transfer_policy());

    assert_eq!(
        result.status(),
        OnlineGroundedEpisodicTransferStatus::ExecutiveRejected
    );

    let authorization = result
        .authorization()
        .expect("executive rejection retains authorization diagnosis");

    assert_eq!(
        authorization.status(),
        EpistemicExecutiveAuthorizationStatus::RequiredStateMismatch
    );
    assert!(authorization.rejected());
    assert!(result.selection().is_none());
}

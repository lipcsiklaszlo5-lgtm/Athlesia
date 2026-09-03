use athlesia_integrated_cognitive_agent::{
    autonomous_cognitive_self_bootstrap::{
        BootstrapFeedback, BootstrapSignal, OutcomeHypothesis, SelfBootstrapBounds,
        SelfBootstrapInput, SelfBootstrapPolicy, SelfBootstrapStatus, SelfBootstrapThresholds,
    },
    OnlineAutonomousSelfBootstrapDigest, UniversalOnlineAutonomousCognitiveSelfBootstrap,
};
use athlesia_mindstone_sparse_cognition::CognitiveStructure;

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn signal(value: u16) -> BootstrapSignal {
    BootstrapSignal::new(value).unwrap()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OpaqueTransition {
    source: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
}

impl OpaqueTransition {
    fn new(
        source: CognitiveStructure,
        action: CognitiveStructure,
        outcome: CognitiveStructure,
    ) -> Self {
        Self {
            source,
            action,
            outcome,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OpaqueMicroWorld {
    current_state: CognitiveStructure,
    affordances: Vec<CognitiveStructure>,
    transitions: Vec<OpaqueTransition>,
}

impl OpaqueMicroWorld {
    fn new(
        current_state: CognitiveStructure,
        affordances: Vec<CognitiveStructure>,
        transitions: Vec<OpaqueTransition>,
    ) -> Self {
        Self {
            current_state,
            affordances,
            transitions,
        }
    }

    fn current_state(&self) -> &CognitiveStructure {
        &self.current_state
    }

    fn affordances(&self) -> &[CognitiveStructure] {
        &self.affordances
    }

    fn transition(&self, action: &CognitiveStructure) -> Option<&CognitiveStructure> {
        self.transitions
            .iter()
            .find(|transition| {
                transition.source == self.current_state && transition.action == *action
            })
            .map(|transition| &transition.outcome)
    }
}

fn policy() -> SelfBootstrapPolicy {
    SelfBootstrapPolicy::new(
        SelfBootstrapBounds::new(16, 32, 8).unwrap(),
        SelfBootstrapThresholds::new(signal(500), signal(500), signal(500), signal(500)),
    )
}

fn world_a() -> OpaqueMicroWorld {
    OpaqueMicroWorld::new(
        atom(1001),
        vec![atom(1101), atom(1102)],
        vec![
            OpaqueTransition::new(atom(1001), atom(1101), atom(1201)),
            OpaqueTransition::new(atom(1001), atom(1102), atom(1202)),
        ],
    )
}

fn world_b() -> OpaqueMicroWorld {
    OpaqueMicroWorld::new(
        atom(7001),
        vec![atom(8101), atom(9302)],
        vec![
            OpaqueTransition::new(atom(7001), atom(8101), atom(4401)),
            OpaqueTransition::new(atom(7001), atom(9302), atom(5502)),
        ],
    )
}

fn blind_digest(world: &OpaqueMicroWorld) -> OnlineAutonomousSelfBootstrapDigest {
    UniversalOnlineAutonomousCognitiveSelfBootstrap::evaluate(
        &SelfBootstrapInput::new(
            world.current_state().clone(),
            None,
            world.affordances().to_vec(),
            Vec::new(),
            BootstrapFeedback::Unspecified,
        ),
        policy(),
    )
    .unwrap()
}

fn controlled_digest(
    world: &OpaqueMicroWorld,
    action: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
) -> OnlineAutonomousSelfBootstrapDigest {
    UniversalOnlineAutonomousCognitiveSelfBootstrap::evaluate(
        &SelfBootstrapInput::new(
            world.current_state().clone(),
            None,
            world.affordances().to_vec(),
            vec![OutcomeHypothesis::new(
                world.current_state().clone(),
                action,
                predicted_outcome,
                signal(900),
                signal(900),
                signal(900),
                signal(100),
            )],
            BootstrapFeedback::Unspecified,
        ),
        policy(),
    )
    .unwrap()
}

#[test]
fn world_a_blind_condition_has_no_host_supplied_hypothesis() {
    let world = world_a();
    let result = blind_digest(&world);

    assert_eq!(result.status(), SelfBootstrapStatus::ModelExpansionRequired,);

    assert_eq!(result.selected_action(), None,);

    assert_eq!(result.predicted_outcome(), None,);
}

#[test]
fn surface_remapped_world_b_blind_condition_has_same_capability_boundary() {
    let world = world_b();
    let result = blind_digest(&world);

    assert_eq!(result.status(), SelfBootstrapStatus::ModelExpansionRequired,);

    assert_eq!(result.selected_action(), None,);

    assert_eq!(result.predicted_outcome(), None,);
}

#[test]
fn blind_affordances_alone_never_fabricate_outcome_predictions() {
    for world in [world_a(), world_b()] {
        let result = blind_digest(&world);

        assert!(result.selected_action().is_none());

        assert!(result.predicted_outcome().is_none());
    }
}

#[test]
fn grounded_hypothesis_control_world_a_reaches_downstream_selection() {
    let world = world_a();

    let action = atom(1102);
    let outcome = atom(1202);

    assert_eq!(world.transition(&action), Some(&outcome),);

    let result = controlled_digest(&world, action.clone(), outcome.clone());

    assert_eq!(result.status(), SelfBootstrapStatus::Selected,);

    assert_eq!(result.selected_action(), Some(&action),);

    assert_eq!(result.predicted_outcome(), Some(&outcome),);
}

#[test]
fn grounded_hypothesis_control_world_b_reaches_downstream_selection() {
    let world = world_b();

    let action = atom(9302);
    let outcome = atom(5502);

    assert_eq!(world.transition(&action), Some(&outcome),);

    let result = controlled_digest(&world, action.clone(), outcome.clone());

    assert_eq!(result.status(), SelfBootstrapStatus::Selected,);

    assert_eq!(result.selected_action(), Some(&action),);

    assert_eq!(result.predicted_outcome(), Some(&outcome),);
}

#[test]
fn worlds_are_surface_distinct_but_share_the_same_hidden_transition_arity() {
    let first = world_a();
    let second = world_b();

    assert_ne!(first.current_state(), second.current_state(),);

    assert_ne!(first.affordances(), second.affordances(),);

    assert_eq!(first.affordances().len(), 2,);

    assert_eq!(second.affordances().len(), 2,);

    assert_eq!(first.transitions.len(), second.transitions.len(),);
}

#[test]
fn e5a_empirical_capability_verdict_is_reproducible() {
    let first_a = blind_digest(&world_a());

    let second_a = blind_digest(&world_a());

    let first_b = blind_digest(&world_b());

    let second_b = blind_digest(&world_b());

    assert_eq!(first_a, second_a,);

    assert_eq!(first_b, second_b,);

    println!();
    println!("=== E5A EMPIRICAL CAPABILITY VERDICT ===");
    println!("WORLD A / NO HOST HYPOTHESES: {:?}", first_a.status(),);
    println!(
        "WORLD B / SURFACE REMAP / NO HOST HYPOTHESES: {:?}",
        first_b.status(),
    );
    println!("HOST-HYPOTHESIS POSITIVE CONTROL: SELECTED");
    println!("ENDOGENOUS HYPOTHESIS GENERATION OBSERVED: NO");
    println!("DOWNSTREAM GROUNDED SELECTION OBSERVED: YES");
    println!("CURRENT BOTTLENECK: MODEL EXPANSION / HYPOTHESIS GENERATION");
}

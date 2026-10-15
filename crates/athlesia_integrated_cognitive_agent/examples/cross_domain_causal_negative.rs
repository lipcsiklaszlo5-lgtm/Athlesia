//! Negative control for the bounded cross-domain causal core.
//!
//! The target has the same node inventory but one causal edge has the wrong
//! relation kind. A structural transfer must refuse the mapping rather than
//! installing a target model.

use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{
    CausalEdge, CausalNode, CanonicalCausalGraph, CrossDomainCausalCore, EdgeKind, Literal,
    MappingBudget, NodeKind, State, TransferStatus, TransitionExample,
};

fn graph(offset: u32, write_kind: EdgeKind) -> CanonicalCausalGraph {
    CanonicalCausalGraph::new(
        vec![
            CausalNode {
                id: offset,
                kind: NodeKind::Operation,
                type_tag: 7,
            },
            CausalNode {
                id: offset + 1,
                kind: NodeKind::StateField,
                type_tag: 1,
            },
            CausalNode {
                id: offset + 2,
                kind: NodeKind::StateField,
                type_tag: 2,
            },
        ],
        vec![
            CausalEdge {
                from: offset,
                to: offset + 1,
                kind: EdgeKind::Requires,
            },
            CausalEdge {
                from: offset,
                to: offset + 2,
                kind: write_kind,
            },
        ],
    )
    .expect("negative-control graph is structurally valid")
}

fn source_state(power: bool, safety: bool, gate: bool) -> State {
    State::new([
        Literal::new(1, power),
        Literal::new(2, safety),
        Literal::new(3, gate),
    ])
}

fn main() {
    let mut core = CrossDomainCausalCore::new();
    core.learn_source(&[
        TransitionExample::new(
            1,
            9,
            source_state(true, true, false),
            source_state(true, true, true),
        ),
        TransitionExample::new(
            2,
            9,
            source_state(true, true, false),
            source_state(true, true, true),
        ),
        TransitionExample::new(
            3,
            9,
            source_state(true, true, false),
            source_state(true, true, true),
        ),
    ])
    .expect("source fixture must be learnable");

    let status = core.transfer(
        &graph(1, EdgeKind::Writes),
        &graph(10, EdgeKind::Emits),
        19,
        MappingBudget::new(128).expect("positive mapping budget"),
    );

    assert_eq!(status, TransferStatus::Refused);
    assert!(core.target().is_none());
    assert!(core.ledger().verify());
    assert_eq!(core.ledger().events().len(), 2);

    println!(
        r#"{{"record":"negative_control","case":"wrong_topology","status":"REFUSED","target_installed":false,"ledger_valid":true,"events":2}}"#
    );
}

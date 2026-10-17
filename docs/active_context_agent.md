# A5 — Active context and fail-closed perception

The A5 layer adds a small capability above the existing causal learner: an
observation may contain `True`, `False`, or `Unknown`. Unknown is preserved
through the API and cannot silently become `false`.

When a required field is unknown, the kernel chooses a host-approved,
resettable inspection action. The host returns the value through a command and
token-bound receipt. Only after the state is complete does the existing
`AutonomousCausalAgent` receive control. The causal learner, its evidence
ledger, revision epochs and transfer checks remain the authority for actions.

The scope is deliberately bounded: Boolean fields, a finite inspection
catalogue and a complete-state handoff. It is an active-perception contract,
not a general vision system or a physical robot safety controller.

Run:

```bash
cargo test --locked --manifest-path crates/athlesia_integrated_cognitive_agent/Cargo.toml --test active_context_agent -- --test-threads=1
cargo run --locked --manifest-path crates/athlesia_integrated_cognitive_agent/Cargo.toml --example active_context_bench
```

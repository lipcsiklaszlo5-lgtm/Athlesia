# Evidence and limitations

This document separates tested *local mechanisms* from an integrated or external benchmark claim. It is not a scorecard.

| Mechanism | Tracked source evidence | Allowed conclusion | Not demonstrated |
| --- | --- | --- | --- |
| Deterministic symbolic abstraction | `src/`, historical crate tests | Structural identity and revision baselines in restricted representations | Arbitrary domain intelligence |
| Grounded explanatory models | M47 `src/lib.rs`, `tests/` | Version spaces, bounded contexts, counterexample/relevance handling | Causality established from unconstrained real-world data |
| Episodic analogy | M49 `tests/t1b_empirical_one_shot_acceleration.rs` | On its synthetic two-step worlds, 6 cold vs. 4 transferred interactions | General zero/one-shot learning or 500x speedups |
| Noisy transfer and conflict abstention | M49 T1C/T1D tests | Specific noisy-world advantage and poisoned-correspondence abstention | Universal robustness to drift or adversarial agents |
| Online revision | `experiments/e6_online_revision_transfer/` | Python reference rules and tests; see E6 evidence reports | Rust DCR end-to-end equivalence |
| Pixel affordance discovery | E7–E9 experiment READMEs | Bounded evaluation within their defined environment grammars | General visual perception |
| Out-of-class falsification | E10 `README.md` and `E10_EVIDENCE.md` | Known late-drift blind spot and stress tests | Perfect rejection of unseen semantics |
| E11 monitoring | Local working tree only in audited snapshot | No tracked-source claim until committed/reviewed | Published or independently repeated outcome |
| ARC-AGI-3 | ARC adapter and blind benchmark crates | Protocol, runner and tests exist | Verified official score or general task success |
| Fast native/WASM runtime | Proposed DCR work | A target to benchmark | 0.2 ms bound, memory footprint guarantee or certified safety |

## Reproducibility order

1. `./tools/verify_repo.sh` — exact 33-manifest Rust test inventory, no official ARC transport.
2. Run each Python experiment separately using **its own** pinned documented invocation, input fixtures and boundary conditions.
3. Inspect actual saved traces, errors, and output artifacts. Counts in documentation are historical evidence claims until reproduced on a named revision.
4. Require independent, hidden-to-agent test scenarios before making external product claims.

## Important provenance distinctions

- A deterministic policy over a finite hypothesis set is not proof that the hypothesis set contains the world's actual transition law.
- Falsification only occurs after a contradictory observation. Previously committed external side effects may not be reversible.
- One successful source episode can ground an analogical proposal without proving the target outcome; unknown predictions must remain unknown.
- A successful `cargo test` is code-regression evidence, not ARC-AGI-3 completion evidence.
- The repository's tracked license is MIT; any future AGPL/component-specific licensing requires an explicit change and rights review.

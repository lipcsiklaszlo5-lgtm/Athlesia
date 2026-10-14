# Repository map — read this before changing a crate

Athlesia uses independent Cargo roots. **Do not turn the project into a Cargo workspace as a cosmetic cleanup.** The canonical list is [`tools/repo_manifests.txt`](../tools/repo_manifests.txt). The canonical test runner is [`tools/verify_repo.sh`](../tools/verify_repo.sh).

## Active line (preserve and integrate)

- **M45**: `crates/athlesia_mindstone_sparse_cognition` — structural state and bounded cognition.
- **M46**: `crates/athlesia_core_knowledge_perceptual_grounding` — perceptual hypothesis/evidence.
- **M47**: `crates/athlesia_universal_domain_learning` — causal transition and explanatory models.
- **M48**: `crates/athlesia_executive_agency` — executive action authority.
- **M49**: `crates/athlesia_meta_learning_skill_memory` — episodic analogy and transfer.
- **M50**: `crates/athlesia_autonomous_active_experimentation` — exploration and hypothesis discrimination.
- **M51**: `crates/athlesia_integrated_cognitive_agent` — active orchestration/feedback.
- **Bootstrap**: `crates/athlesia_autonomous_cognitive_self_bootstrap` — evidence-to-goal synthesis.
- **ARC**: `crates/athlesia_arc_agi_3_adapter` — real adapter/runtime/protocol.
- **Blind benchmark**: `crates/athlesia_arc_agi_3_blind_benchmark` — bounded evaluation harness.

These ten modules are the currently designated active line; that does not establish every source function is invoked by the default runner.

## Historical or frozen Rust research (retain)

- Root `src/` baseline (`athlesia`).
- `crates/athlesia_revision`, `crates/athlesia_hierarchy`, `crates/athlesia_cross_level`, `crates/athlesia_recursive`.
- `crates/athlesia_recursive_*`: historical reasoning / world-model revision variants, including abstraction, induction, composition and generalization.
- `experiments/e5_empirical_abstraction_micro_world`.

**Preservation rule:** do not remove or relocate the frozen code until there is a verified consumer graph and compatible contract replacement. Legacy does not mean useless; it means not authoritative for the present runtime.

## Python research and falsification

`experiments/e6_online_revision_transfer` contains Python causal revision/transfer reference code; E7 through E9 move toward raw-grid and pixel grounding; E10 is the adversarial falsification pilot. Their local tests should be kept together with their SHA manifests and evidence reports. The checked Git revision does not include E11: an untracked local E11 tree needs a separate provenance review and deliberate commit.

## Historical notes

- [`REPOSITORY_STATUS.md`](REPOSITORY_STATUS.md) — September build-infrastructure checkpoint; old branch and commit references are historical.
- [`COGNITIVE_TRACE.md`](COGNITIVE_TRACE.md) — passive recording contract.
- [`SUCCESSOR_EPISODE_RUNTIME.md`](SUCCESSOR_EPISODE_RUNTIME.md) — bounded ARC execution contract.
- [`checkpoints/`](checkpoints/) — dated checkpoints.
- [`archive/`](archive/) — large read-only audit documents if safely archived by the repository cleanup.

For a local manifest/dependency summary and untracked-file warnings, run `python3 tools/audit_repository.py`.

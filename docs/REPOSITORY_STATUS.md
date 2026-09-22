# Repository status

This build infrastructure checkpoint follows commit
`87730815ef2ffc886c3cb675066af0446b028954` on
`recovery-laptop-20260921`. It preserves the existing crate architecture and
cognitive behavior.

## Canonical build verification

Run `./tools/verify_repo.sh` from the checkout. The script resolves the repository
root itself and verifies exactly the 33 paths in `tools/repo_manifests.txt`, in
inventory order. GitHub Actions invokes the same command on Linux.

`rust-toolchain.toml` pins Rust 1.98.1 with rustfmt and clippy. Rustup must be
available; initial toolchain and dependency downloads require registry access.
For each manifest, verification runs locked metadata, formatting checks,
locked checks of all targets, and locked tests. Clippy is installed but is not
an additional verification stage.

Builds run sequentially with one job, incremental compilation disabled, and
debug information disabled. Build artifacts use a temporary directory outside
the checkout, removed on exit. The script does not rewrite source, formatting,
or lockfiles. It stops at the first failure and reports the manifest and stage.
It does not invoke ARC network benchmark execution or the stale
`tools/verify_state.py`. Both that verifier and `state/project_state.json` await
a separate reconciliation task and are not the canonical build truth surface.

The repository retains 33 independently buildable manifests, including crates
that intentionally serve as independent Cargo workspace roots. The root remains
an `athlesia` package; no repository-wide Cargo workspace is introduced.

## Active cognitive line

- `athlesia_mindstone_sparse_cognition`
- `athlesia_core_knowledge_perceptual_grounding`
- `athlesia_universal_domain_learning`
- `athlesia_autonomous_active_experimentation`
- `athlesia_executive_agency`
- `athlesia_meta_learning_skill_memory`
- `athlesia_autonomous_cognitive_self_bootstrap`
- `athlesia_integrated_cognitive_agent`
- `athlesia_arc_agi_3_adapter`
- `athlesia_arc_agi_3_blind_benchmark`

## Legacy/frozen research line

- Root `athlesia` package.
- `athlesia_revision`, `athlesia_hierarchy`, and `athlesia_cross_level`.
- The `athlesia_recursive*` crates.
- `experiments/e5_empirical_abstraction_micro_world` (E5) remains research
  validation infrastructure.

These components are preserved and verified. They are not currently the
authoritative production cognitive execution line.

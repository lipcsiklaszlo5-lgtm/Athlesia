# Contributing to Athlesia

Athlesia is research software. Contributions must preserve the distinction between an empirical observation, a hypothesis and an authorized execution.

## Before submitting

1. Identify the existing module that owns the concept. Do **not** introduce a new Cargo crate for a single test, feature flag, proof sketch or tiny helper.
2. Include a negative case: structurally similar but incorrect inputs, conflicting evidence, exhausted bounds or incomplete observations.
3. Preserve exact identities and the original evidence trail; never manufacture missing observations, candidate sets, scorecard values or effects.
4. If production behavior changes, provide a focused failing test first and a replayable result from the real active path.
5. Run `./tools/verify_repo.sh` and the affected Python experiment suite. The root `cargo test` is not the full repository check.
6. Keep research archives, manifests and historical negative results; changes to frozen contracts require explicit evidence and a migration note.

## Scope discipline

- Active Rust line: M45–M51, ARC adapter, blind benchmark and bootstrap.
- Legacy crates: retained for historical integrity, **not** automatic production integration points.
- Python E6–E10: reference experiments, **not** silent production runtime dependencies.
- New DCR: planned canonical model/evidence store and execution gate, not yet implemented.

## Safe release process

Keep algorithm changes separate from documentation and formatting changes. Never auto-stage `git add -A` from a dirty Codespaces tree with untracked experiments, artifacts or personal scratch scripts. Review `git status --short`, stage explicit paths, commit and let GitHub Actions verify the exact revision.

# Athlesia

**Research software for bounded, evidence-grounded structural learning and causal model revision.**

Athlesia investigates whether a small deterministic cognitive core can form reusable abstractions from observations, choose informative actions, revise falsified models and transfer experience across unfamiliar environments. The long-term benchmark target includes interactive ARC-AGI-3; the immediate engineering target is an integrated, independently testable causal learning runtime.

This is **experimental research software**, not a general AGI, a production safety controller or a validated replacement for language models.

## Start here

| What you want | Where to look |
| --- | --- |
| The authoritative Rust build and regression suite | `./tools/verify_repo.sh` |
| Read-only repository inventory and dependency map | `python3 tools/audit_repository.py` |
| Active cognitive modules vs historical research | [Repository map](docs/REPO_MAP.md) |
| Current and missing integration contracts | [Architecture](docs/ARCHITECTURE.md) |
| Reproducible claims, limitations and experiments | [Evidence](docs/EVIDENCE_AND_LIMITS.md) |
| Near-term implementation order | [Roadmap](docs/ROADMAP.md) |
| Contribution and preservation rules | [Contributing](CONTRIBUTING.md) |

## Research mechanisms

```text
observations / external actions
               |
        perceptual evidence (M46)
               |
        causal hypotheses (M47) <--- counterexamples
               |
          executive choice (M48) <--> bounded experiments (M50)
               |
          observed outcomes
               |
       episodic skill evidence (M49)
               |
     correspondence / conditional transfer

  M45: finite compute and sparse processing
  M51 + ARC adapter: orchestration, grounding, execution feedback
  E6–E10: separate Python reference and falsification experiments
```

**A diagram is not an end-to-end performance claim.** The existing modules contain working, separately tested mechanisms; the unified DCR knowledge state and execution authority are still planned. See [Architecture](docs/ARCHITECTURE.md) for the integration gaps.

## Reproduce the Rust baseline

Requirements: Rust as pinned in `rust-toolchain.toml` and a Linux environment. Dependency downloads may require network access.

```bash
./tools/verify_repo.sh
```

This is the canonical verification path. It checks the **33 manifest entries** in `tools/repo_manifests.txt`, in sequence. It does **not** run the Python E6–E10 experiments or an official ARC-AGI-3 scorecard. `cargo test` from the repository root only tests the root package.

For E6:

```bash
cd experiments/e6_online_revision_transfer
python3 -m unittest discover -s tests -v
```

The other Python experiments have their own readmes and invocation instructions; several require explicit `PYTHONPATH` entries. Do not treat their results as official ARC scores.

## What has been demonstrated — and what has not

- **M49**: a documented deterministic two-step microworld shows one-episode analogical acceleration (6 cold interactions versus 4 transferred). This is **not** general one-shot learning.
- **M47**: bounded explanatory hypotheses, observation-conditioned rule assessment, counterexamples and representation insufficiency detection have dedicated tests.
- **E6**: a Python causal-revision and transfer reference environment with separately documented tests and evidence.
- **E10**: adversarial affordance falsification includes an intentionally detectable failure mode: a late semantic change cannot be guaranteed absent by early calibration.
- **ARC-AGI-3**: an adapter and bounded runner exist, but this README makes **no claim** of a verified general or official competitive score.
- **DCR**: an architectural direction, **not yet** a completed integrated runtime.

Details and experiment-specific caveats: [Evidence and limits](docs/EVIDENCE_AND_LIMITS.md).

## Repository policy

- The active cognition line is **M45–M51, executive/experiment modules and the ARC adapter**, not every historical crate.
- M23–M44 and their recursive crates remain in the verification inventory as **preserved research history**.
- E5–E10 are research environments and test oracles, not interchangeable production cognitive engines.
- Untracked work such as E11 must be reviewed and committed explicitly before being cited as a published component.
- Avoid new crates for each micro-feature; keep one canonical owner for execution authority and evidence provenance.

The code in this repository is currently released under the [MIT License](LICENSE). Do not assume an AGPL relicensing has already occurred. Third-party code and future commercial distribution must honor applicable license obligations.

## Research principles

**Exact identity before abstraction. Evidence before promotion. Preserve negative cases. Bound computation. Abstain rather than fabricate a conclusion.**

[Research background](docs/RESEARCH.md) · [Design principles](docs/DESIGN_PRINCIPLES.md) · [Historical infrastructure note](docs/REPOSITORY_STATUS.md)

# Architecture — current research inventory and next integration boundary

Athlesia is a collection of separately versioned Cargo packages, **not one root Cargo workspace**. The canonical build inventory is `tools/repo_manifests.txt` (33 manifests). The root `athlesia` crate is an early structural-learning baseline, not the active production agent.

## Active research line

| Layer | Rust package | Responsibility |
| --- | --- | --- |
| Compute allocation | `athlesia_mindstone_sparse_cognition` (M45) | Bounded cognitive structures, sparse/evidence-sensitive processing |
| Perception | `athlesia_core_knowledge_perceptual_grounding` (M46) | Provisional objecthood and frame/behavior evidence |
| World model | `athlesia_universal_domain_learning` (M47) | Grounded transition episodes, explanatory version spaces, causal contrasts, insufficiency detection |
| Executive | `athlesia_executive_agency` (M48) | Bounded selection, abstention, replan decisions |
| Skill / transfer | `athlesia_meta_learning_skill_memory` (M49) | Grounded episodic analogy, candidate reuse, conflicting-correspondence rejection |
| Experimentation | `athlesia_autonomous_active_experimentation` (M50) | Epistemic candidate choice and progress estimation |
| Bootstrap | `athlesia_autonomous_cognitive_self_bootstrap` | Evidence-based objective and candidate evaluation |
| Orchestration | `athlesia_integrated_cognitive_agent` (M51) | Cognitive integration APIs and feedback bindings |
| Environment | `athlesia_arc_agi_3_adapter` | ARC observation/action contracts, bounded live execution and transport |
| Benchmark protocol | `athlesia_arc_agi_3_blind_benchmark` | Blind benchmark input/output and trace contracts |

This table describes **source ownership**, not proof that all modules share a persistent world model during a live episode. See `ATHLESIA_CODEX_ARCHITECTURE_AUDIT.txt` in the original history and the source-level integration tests for actual call paths.

## Python research references

| Experiment | Role |
| --- | --- |
| E5 | Baseline blind-affordance limitation |
| E6 | Online rule revision, guarded transitions and transfer reference |
| E7 | Raw-grid and motion evidence |
| E8 | Pixel-to-causal bridge |
| E9 | Unlabelled pixel affordance discovery |
| E10 | Adversarial out-of-class falsification and documented late-drift weakness |
| E11 | Python continuous monitoring, provisional validity, contradiction-triggered quarantine; not integrated into Rust DCR |

Each experiment has its own evidence scope. They must not be silently collapsed into a claimed end-to-end result.

## Intended DCR integration (not yet implemented)

```text
independently observed state
         |
 M46 grounding + M47 hypothesis synthesis
         |
 canonical versioned K = (mapping, guards, transitions,
                            positive/negative evidence, validity, budget)
         ^                                  |
         |                           M49 skill proposal
         |                                  |
         |                         SINGLE EXECUTION GATE
         |                        /          |           \
         |                     permit       probe        abstain
         |                        \          |           /
         +---------------- actual outcome / exact provenance
                                  |
                         falsification / E6-style revision
```

**Required invariants before this becomes a supported runtime:**

1. One canonical owner of learned state and versioned evidence; no parallel authoritative E6 and M47 stores.
2. Observation/action provenance bound to exactly one completed environment event; transport failure cannot fabricate success.
3. An action proposal is not permission. No bypass around the final gate.
4. `Unknown`, conflict and exhausted resource bounds do not become positive predictions.
5. Drift is detected by **observed disagreement**, not magically before an unchanged-schema tool has acted.
6. Distinguish bounded hypotheses from a complete hypothesis class. Retained facts, gaps and counterexamples remain queryable.
7. Active tests must be restricted to explicitly authorized, low-risk test environments or reversible operations.
8. No claims of real-time timing or hardware safety without measurements and system-level validation.

Existing detailed contracts: [Cognitive trace](COGNITIVE_TRACE.md), [Successor episode](SUCCESSOR_EPISODE_RUNTIME.md). Historical audit reports have been retained, not treated as current runtime documentation.

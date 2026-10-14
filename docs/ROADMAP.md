# Roadmap — one end-to-end capability before new modules

## Non-goals

- No more one-feature-one-crate proliferation without an explicit architectural owner.
- No manual ARC game solution, hidden oracle data or invented benchmark successes.
- No claim of zero-shot AGI, hardware safety certification or sub-millisecond worst-case timing without proof.
- Do not delete frozen research modules just to make the codebase appear smaller.

## Ordered engineering gates

| Gate | Work | Evidence to release |
| --- | --- | --- |
| R0 — stable baseline | Reconcile local source edits and preserve green 33/33 Cargo manifest verification | Committed clean change, green GitHub Actions run |
| R1 — canonical knowledge state | Define versioned K, evidence provenance, bounded candidate frontiers | Positive/negative Rust tests and deterministic serialization |
| R2 — M47 + M49 | Bind analogical proposals to grounded explanatory hypotheses | No action inferred from missing outcome; false transfer rejected |
| R3 — execution authority | One pre-dispatch gate, epoch/version checks, no hidden retries | Stale-authority, ambiguous-mapping and transport-indeterminate tests |
| R4 — falsification and revision | Port E6-style version-space revision with E10-style negative oracle | Documented model quarantine and bounded, authorized probes |
| R5 — real tool boundary | Independently authored tool drift evaluator with unknown cases | Cold-start/transfer/negative-transfer comparison, full raw traces |
| R6 — small hardware / browser | Native CPU and WASM packaging of the same core | Real latency distribution, peak memory, CPU and binary-size data |
| R7 — external claim validation | Independent runs, reproducible scripts, limitations | Only then public Show HN/tech launch claims |

The original ARC-AGI-3 objective remains a separate external benchmark; DCR work should reuse the same abstraction and feedback semantics instead of introducing benchmark-specific rules.

## Engineering acceptance rule

A gate is complete only when its production code, independent failure cases, and reproducible commands exist in the same revision. Tests asserting a hoped-for result are not independent evidence of real-world performance.

# Cross-domain causal trace

The `cross_domain_causal_trace` example is a bounded, deterministic demonstration of
Athlesia's causal cycle. It uses the production `CrossDomainCausalCore` API and emits
machine-readable JSONL; the example is not a scripted success flag.

## Reproduce and verify

From the repository root:

```bash
bash tools/verify_cross_domain_causal_trace.sh
```

The verifier writes `artifacts/causal/cross_domain_causal_trace.jsonl` and checks:

- the complete `OBSERVE -> ABSTRACT -> TRANSFER -> UNKNOWN -> TARGET_TRIAL -> FALSIFY -> QUARANTINE -> ACTIVE_PROBE -> REVISE` sequence;
- seven ordered ledger records;
- continuity of the previous-record hash links;
- a final `REVISED` status at epoch 1;
- a positive runtime measurement from the current process.

## What this establishes

The fixture demonstrates that a typed structural mapping can produce a target
hypothesis, that target evidence is required before authority is granted, and that a
contradicting target outcome revokes the previous permit. The core then selects a
bounded probe and commits a local revision while preserving the ledger chain.

The target sandbox is fixed and deterministic. The example demonstrates the complete
runtime protocol within its declared grammar; it does not claim arbitrary graph
understanding, universal causal identification, or a guaranteed runtime bound on
unbounded input sizes. The reported nanoseconds are measurements of the current local
run and must be reported with the hardware and build mode.

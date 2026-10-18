# Athlesia Causal Theater — live P9 proof

This extends the existing P9 checkpoint (`6ef218b`) and the P10.1 light frontend.
The main three-act journey now calls the **existing P9 Rust runtime**, through
three isolated WASM Workers. Mock mode is available only with `?engine=mock`.
There is no automatic fallback.

## Build and open

From the repository root:

```bash
node --test demo/causal-theater/tests/*.test.mjs
CARGO_BUILD_JOBS=1 cargo test --manifest-path crates/athlesia_causal_wasm/Cargo.toml --jobs 1 --example theater_p9_replay
bash tools/build_causal_theater.sh
python3 tools/serve_causal_theater.py --port 8081
```

Open port 8081 in Codespaces and use `/causal-theater/`. The new server also
redirects `/` and the retired `/wasm/` entrypoint to the current frontend. The
WASM binary stays under `demo/wasm/pkg`. If wasm-pack is missing, install it once
with `cargo install wasm-pack --locked`. The build adds Ed25519 dependencies to
this crate's lockfile; review and commit that updated lockfile with the sources.

## What the visitor can test

1. Enter a seed and initialize. Two local origin Workers create independent
   ephemeral Ed25519 keys with the browser CSPRNG. Each learns a source output
   effect. A third Worker receives only public keys and signed receipt-backed
   CKS/1 packets. A modified signed packet is tried and rejected first.
2. Sabotage the software plant. A real Rust prediction commitment resolves as
   `Contradicted`. The origin signs an endorsement withdrawal. The receiver
   applies it to its existing trust mesh and attempts the exact prepared lease.
   `authorize_and_dispatch_trusted` rejects it as `StaleExecutionLease`. No
   executable receiver actions escape the Rust gate; no database write occurs.
3. Revalidation is an explicit host-controlled restore. A newer signed
   endorsement restores quorum. Rust verifies that the old lease remains stale
   and issues a fresh one. Revalidation itself dispatches nothing.
4. Bind a database domain through typed roles. The kernel requires a unique
   structural mapping and imports the frozen source evidence. There are no
   target-domain learner updates or trial database writes.
5. Execute. A denied host-policy trial is checked first. A subsequent admitted
   P9 dispatch exposes the action exactly once. The browser performs an actual
   IndexedDB transaction, then checks the stored record in a separate read
   transaction. Rust resolves the observed after-state against its commitment.
6. Try replay. The request reaches Rust and is rejected; the database driver is
   not called again. The record can be read independently in DevTools under
   `Application → IndexedDB → athlesia-p9-proof → dispatches`.

The database record is local demonstration data, not a production transaction.
No network service or model API is used during the run. OS CPU utilization,
temperature and power are not fabricated. Linear memory is the sum of the
three WASM instances; it is not total process or system memory.

## Reproduce without JavaScript

Download **native replay proof** after completing the journey. Place the JSON
in the repository root, then run:

```bash
cargo run --manifest-path crates/athlesia_causal_wasm/Cargo.toml --example theater_p9_replay -- --verify athlesia-p9-proof.json
```

This creates a fresh native receiver and replays every original request,
including rejected requests, checking every Rust result. Raw signed JSONL
packets remain strings so JavaScript never rounds their u64 fields. Signing
secrets are not exported. Modified verdicts, modified packets, an incomplete
journey or a different native result cause a nonzero exit.

To generate a deterministic native-only trial with documented test keys:

```bash
cargo run --manifest-path crates/athlesia_causal_wasm/Cargo.toml --example theater_p9_replay -- 42 > native-p9-proof.json
cargo run --manifest-path crates/athlesia_causal_wasm/Cargo.toml --example theater_p9_replay -- --verify native-p9-proof.json
```

The browser uses fresh private keys in origin Workers, not these test keys.
For an additional **Node / SQLite** host integration trial after building WASM
(Node >=22 with `node:sqlite`):

```bash
node tools/test_theater_wasm.mjs 42 node-p9-proof.json
cargo run --manifest-path crates/athlesia_causal_wasm/Cargo.toml --example theater_p9_replay -- --verify node-p9-proof.json
```

This uses the actual browser orchestration and WASM, with a real SQLite host
write instead of browser IndexedDB. It is not a screenshot or browser-layout
check. Browser storage and render checks still require a browser.

## Precise scope

- Source: a bounded, explicitly simulated actuator, not a physical industrial
  robot. Industrial validation requires a separate physical or HIL trial with
  independently observed actuator calls and sensor outcomes.
- Learning: the current receipt exporter transfers an **observed effect**.
  Resource/interlock conditions are explicit host policy. Positive receipts
  alone do not discover or preserve a complete guarded causal law. The UI
  does not label them as learned guards.
- Trust: two local origins, two ephemeral keys and isolated WASM memories,
  with actual Ed25519 verification. They are not independent organizations or
  independently truthful physical sensors. Trust roots are host-provisioned.
- Reproduction: the native verifier independently recomputes the kernel
  verdicts. It does not remotely attest the original browser, its database,
  or physical events. An outside person can rebuild and repeat the trial;
  an actual external reproduction must still be performed by that person.
- Zero-shot: no target observations train the model. Replaying source evidence
  during CKS import is allowed. Typed graph roles are supplied by the host;
  arbitrary schema discovery is outside this bounded demonstration.
- Execution: the host database driver obeys Rust dispatches. A hostile page
  owner can bypass their own client-side JS; this page is not a security
  boundary protecting arbitrary third-party databases.

## Files and existing kernel boundaries

`theater.rs` lives in the existing WASM crate. It calls `CausalLearner`,
`ExecutionKnowledgeSync`, `ExecutionKnowledgeReceiver`, `CausalTrustMesh`,
`CausalHierarchy`, and `EndToEndCausalRuntime`. It introduces no replacement
learner, trust mesh or causal engine. The P1–P9 core source files are unchanged.

`wasm-bridge.js` orchestrates messages; `p9-worker.js` owns Rust instances;
`load-kernel.js` fingerprints and executes the same bytes; `database-driver.js`
is the browser host adapter. The old opaque challenge files remain as source
but are no longer loaded by the main page. Existing mock regression tests
remain available and clearly identified as mock tests.

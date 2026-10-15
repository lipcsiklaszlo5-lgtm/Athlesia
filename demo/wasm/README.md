# Athlesia Fresh World Challenge — WASM demo

This is a static browser UI over the Rust `FreshWorldSession` state machine and
the user-supplied mechanism runner. It has no Python server and no model/API
dependency.

## Build

From the repository root:

```bash
cargo install wasm-pack --locked
bash tools/build_causal_wasm.sh
```

## Run locally

```bash
python3 -m http.server 8080 --directory demo/wasm
```

Open `http://localhost:8080`.

The page drives three actual Rust transitions:

```text
GENERATED → SABOTAGED → HOLDOUT_VERIFIED
```

The browser receives the guard commitment, not the hidden guard itself. This is
solver-interface isolation, not a claim of cryptographic secrecy: client-side
WASM can be inspected by the person running it.

## Bring your own mechanism

The **RUN OPAQUE-PORT HOLDOUT** panel accepts a JSON mechanism with:

- a typed source, target and holdout graph;
- one or more source transition traces;
- an optional target and holdout environment guard used only to answer probes;
  when omitted, the fixture derives the guard from the seed and structural
  type tags.

The Rust core does not receive the world specification as a learner input. A
private environment adapter emits opaque observation frames; the solver receives
only `before → action → after` logs and reconstructs its bounded structural
representation from those frames. It does not receive guard membership as a
learner fact. A structurally ambiguous or incompatible graph returns
`ABSTAIN`/`REFUSED`; it is never counted as a success.

The current contract is intentionally bounded: at most 32 nodes, 96 edges, 32
source examples and one or two hidden guard literals. This is a reproducible
research boundary, not unrestricted domain understanding.

The same contract can be replayed without a browser:

```bash
cargo run \
  --manifest-path crates/athlesia_integrated_cognitive_agent/Cargo.toml \
  --example custom_mechanism -- path/to/challenge.json
```

The command emits the same JSONL evidence format used by the WASM UI and exits
non-zero for `ABSTAIN`, `REFUSED` or invalid input.

To exercise the same challenge through the strict black-box boundary:

```bash
cargo run \
  --manifest-path crates/athlesia_integrated_cognitive_agent/Cargo.toml \
  --example opaque_action_port -- path/to/challenge.json
```

# Athlesia Fresh World Challenge — WASM demo

This is a static browser UI over the Rust `FreshWorldSession` state machine.
It has no Python server and no model/API dependency.

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

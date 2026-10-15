# Athlesia causal trace demo

This is a thin Gradio presentation layer over the compiled Rust examples. It does
not reimplement the learner in Python. The two controls expose the same positive
causal cycle and wrong-topology negative control used by the repository verifiers.

## Local run

From the repository root:

```bash
cargo build --release \
  --manifest-path crates/athlesia_integrated_cognitive_agent/Cargo.toml \
  --examples

python3 -m pip install -r demo/requirements.txt
python3 demo/app.py
```

Open the printed local URL. `ATHLESIA_BIN_DIR` can point at another directory
containing the two compiled example binaries.

The current UI intentionally exposes the bounded verified fixtures. Arbitrary user
graph ingestion and Wasm packaging are separate milestones; the interface must not
pretend that those capabilities already exist.

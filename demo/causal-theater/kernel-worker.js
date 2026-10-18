// This worker calls the existing compiled Rust session directly. No mock adapter
// is imported here. Errors cross the worker boundary as failures.
import { checkedSeed, checkedSummary } from "./native-proof-core.js";

const PKG = new URL("../wasm/pkg/", import.meta.url);
let module;
let exports;
let identity;
let loading;
let session;

async function loadKernel() {
  if (!loading) loading = (async () => {
    module = await import(new URL("athlesia_causal_wasm.js", PKG).href);
    if (typeof module.WasmOpaqueInteractionSession !== "function") throw new Error("This WASM build lacks WasmOpaqueInteractionSession. Rebuild from the current repository.");
    const response = await fetch(new URL("athlesia_causal_wasm_bg.wasm", PKG));
    if (!response.ok) throw new Error(`WASM executable unavailable (HTTP ${response.status}).`);
    const bytes = await response.arrayBuffer();
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    const hash = [...new Uint8Array(digest)].map(value => value.toString(16).padStart(2, "0")).join("");
    // Hash and execute the SAME bytes. Current wasm-bindgen web initialization.
    exports = await module.default({ module_or_path: bytes });
    if (!(exports?.memory instanceof WebAssembly.Memory)) throw new Error("The WASM build did not expose linear memory.");
    let build = null;
    const manifest = await fetch(new URL("theater-build.json", PKG));
    if (manifest.ok) {
      build = await manifest.json();
      if (build.wasm_sha256 !== hash) throw new Error("WASM artifact and build manifest disagree. Rebuild before presenting this executable.");
    }
    identity = { wasm_sha256: hash, wasm_bytes: bytes.byteLength, build, boundary: "OpaqueInteractionSession", p9_trust_lease_exercised: false };
  })();
  return loading;
}

self.onmessage = async event => {
  const { id, operation, seed } = event.data || {};
  try {
    if (!Number.isSafeInteger(id) || id < 1) throw new Error("Invalid worker request identity.");
    await loadKernel();
    const started = performance.now();
    let replay = null;
    if (operation === "start") {
      const value = checkedSeed(seed);
      session?.free();
      session = new module.WasmOpaqueInteractionSession(value);
    } else {
      if (!session) throw new Error("No Rust session has been created.");
      if (operation === "act") session.send_action(session.action_token());
      else if (operation === "sabotage") session.sabotage();
      else if (operation === "probe") session.probe();
      else if (operation === "holdout") session.verify_holdout();
      else if (operation === "replay") {
        const before = session.phase();
        if (before !== "HOLDOUT_VERIFIED") throw new Error("Replay challenge requires a verified holdout first.");
        let rejection;
        try { session.verify_holdout(); } catch (error) { rejection = String(error); }
        if (!rejection || session.phase() !== before) throw new Error("Rust replay was not rejected cleanly.");
        replay = { rejected_by_rust: true, phase_unchanged: true, error: rejection, scope: "session phase replay; not P9 permit replay" };
      } else throw new Error("Unknown Rust operation.");
    }
    const jsonl = session.snapshot();
    const elapsed = performance.now() - started;
    const phase = session.phase();
    const { summary, count } = checkedSummary(jsonl, phase);
    self.postMessage({ id, ok: true, phase, jsonl, summary, count, replay, identity, metrics: { call_ms: elapsed, memory_bytes: exports.memory.buffer.byteLength, scope: "WASM bridge call including snapshot serialization; not OS CPU usage" } });
  } catch (error) {
    self.postMessage({ id, ok: false, error: String(error?.message || error), phase: session?.phase() || "ERROR" });
  }
};

// P13 deliberately has no mock import and no fallback success path.
import { loadCausalKernel } from "./byod-load-kernel.js";

self.onmessage = async event => {
  const { id, operation, source_csv, target_csv, bundle_json } = event.data || {};
  try {
    if (!Number.isSafeInteger(id) || id < 1) throw new Error("Invalid worker request identity.");
    const kernel = await loadCausalKernel();
    const started = performance.now();
    let raw;
    if (operation === "run") {
      if (typeof source_csv !== "string" || typeof target_csv !== "string") throw new Error("Both evidence domains are required.");
      raw = kernel.module.run_byod_break_test_json(source_csv, target_csv);
    } else if (operation === "verify") {
      if (typeof bundle_json !== "string" || bundle_json.length === 0) throw new Error("A raw Rust proof bundle is required.");
      raw = kernel.module.verify_byod_break_test_json(bundle_json);
    } else throw new Error("Unknown BYOD worker operation.");
    const parsed = JSON.parse(raw);
    self.postMessage({
      id,
      ok: true,
      operation,
      raw,
      parsed,
      identity: kernel.identity,
      metrics: {
        call_ms: performance.now() - started,
        memory_bytes: kernel.exports.memory.buffer.byteLength,
        scope: "local WASM call including Rust JSON serialization; not OS CPU time",
      },
    });
  } catch (error) {
    self.postMessage({ id, ok: false, operation, error: String(error?.message || error) });
  }
};

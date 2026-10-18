const PKG = new URL("../wasm/pkg/", import.meta.url);
let loading;

function hex(bytes) {
  return [...new Uint8Array(bytes)].map(value => value.toString(16).padStart(2, "0")).join("");
}

export function loadCausalKernel() {
  if (!loading) loading = (async () => {
    const module = await import(new URL("athlesia_causal_wasm.js", PKG).href);
    for (const name of ["run_byod_transfer_json", "run_byod_break_test_json", "verify_byod_break_test_json"]) {
      if (typeof module[name] !== "function") throw new Error(`WASM build lacks ${name}. Rebuild the current repository.`);
    }
    const response = await fetch(new URL("athlesia_causal_wasm_bg.wasm", PKG), { cache: "no-store" });
    if (!response.ok) throw new Error(`WASM executable unavailable (HTTP ${response.status}).`);
    const bytes = await response.arrayBuffer();
    const hash = hex(await crypto.subtle.digest("SHA-256", bytes));
    const exports = await module.default({ module_or_path: bytes });
    if (!(exports?.memory instanceof WebAssembly.Memory)) throw new Error("WASM linear memory was not exposed.");
    let build = null;
    const manifest = await fetch(new URL("theater-build.json", PKG), { cache: "no-store" });
    if (manifest.ok) {
      build = await manifest.json();
      if (build.wasm_sha256 !== hash) throw new Error("Build manifest and loaded executable differ.");
    } else if (manifest.status !== 404) throw new Error("Could not check the build manifest.");
    return {
      module,
      exports,
      identity: {
        wasm_sha256: hash,
        wasm_bytes: bytes.byteLength,
        build,
        boundary: "byod_causal_transfer / CrossDomainCausalCore",
        p13_break_test: true,
        p9_execution_lease_issued: false,
      },
    };
  })();
  return loading;
}

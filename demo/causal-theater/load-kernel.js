export async function loadKernel() {
  const module = await import("../wasm/pkg/athlesia_causal_wasm.js");
  const response = await fetch(new URL("../wasm/pkg/athlesia_causal_wasm_bg.wasm", import.meta.url), { cache: "no-store" });
  if (!response.ok) throw new Error(`WASM download failed: HTTP ${response.status}`);
  const bytes = await response.arrayBuffer();
  const hash = [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map(b => b.toString(16).padStart(2, "0")).join("");
  let build = null;
  const manifest = await fetch(new URL("../wasm/pkg/theater-build.json", import.meta.url), { cache: "no-store" });
  if (manifest.ok) {
    build = await manifest.json();
    if (build.wasm_sha256 !== hash) throw new Error("Build manifest and loaded executable differ.");
  } else if (manifest.status !== 404) throw new Error("Could not check the build manifest.");
  const instance = await module.default({ module_or_path: bytes });
  if (!module.WasmTheaterOrigin || !module.WasmTheaterReceiver) throw new Error("This binary predates the P9 theater bridge. Rebuild with bash tools/build_causal_theater.sh.");
  return { module, instance, identity: { wasm_sha256: hash, build } };
}

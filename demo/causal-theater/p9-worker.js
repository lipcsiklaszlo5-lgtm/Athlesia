import { loadKernel } from "./load-kernel.js";
let kernel, session, role;
let busy = false;
self.onmessage = async ({ data }) => {
  const { id, op } = data;
  if (busy) { self.postMessage({ id, error: "One Rust call at a time" }); return; }
  busy = true;
  try {
    if (op === "init") {
      if (session) throw new Error("Worker already initialized");
      kernel = await loadKernel(); role = data.role;
      if (role === "origin") {
        const secret = crypto.getRandomValues(new Uint8Array(32));
        try { session = new kernel.module.WasmTheaterOrigin(data.seed, data.origin, secret); }
        finally { secret.fill(0); }
      } else if (role === "receiver") {
        const decode = text => Uint8Array.from(text.match(/.{2}/g) || [], byte => Number.parseInt(byte, 16));
        session = new kernel.module.WasmTheaterReceiver(data.seed, decode(data.public_a), decode(data.public_b));
      } else throw new Error("Unknown Worker role");
      self.postMessage({ id, result: { public_key: role === "origin" ? session.public_key() : null,
        identity: kernel.identity, memory_bytes: kernel.instance.memory.buffer.byteLength } });
    } else {
      if (!session) throw new Error("Rust session not initialized");
      const started = performance.now();
      let result;
      if (role === "origin" && op === "bundle") result = session.bundle(data.sequence);
      else if (role === "origin" && op === "sabotage") result = JSON.parse(session.sabotage());
      else if (role === "origin" && op === "restore") result = session.restore();
      else if (role === "receiver" && op === "step") result = JSON.parse(session.step(data.request));
      else throw new Error("Operation not allowed in this Worker role");
      self.postMessage({ id, result, call_ms: performance.now() - started,
        memory_bytes: kernel.instance.memory.buffer.byteLength });
    }
  } catch (error) { self.postMessage({ id, error: String(error?.message || error) }); }
  finally { busy = false; }
};

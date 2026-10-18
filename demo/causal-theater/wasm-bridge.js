import { DatabaseDriver } from "./database-driver.js";
import { checkedSeed } from "./native-proof-core.js";

export async function createWasmBridge({ getSeed = () => 42, onUpdate = () => {}, driverFactory = () => new DatabaseDriver() } = {}) {
  if (typeof Worker === "undefined") throw new Error("P9 WASM Workers unavailable. No mock fallback.");
  let channels = [], receiver, origins, identity, seed, driver, publicKeys;
  let transcript = [], checks = {}, nextId = 0, generation = 0;
  const active = (run, signal) => { if (run !== generation || signal?.aborted) throw new DOMException("Run cancelled", "AbortError"); };
  const publish = () => onUpdate({ identity, seed, writes: driver?.writes ?? 0,
    recordKey: driver?.recordKey, checks: { ...checks }, transcript: [...transcript] });
  function channel() {
    const worker = new Worker(new URL("./p9-worker.js", import.meta.url), { type: "module" });
    let pending;
    const stop = error => {
      worker.terminate();
      if (pending) { clearTimeout(pending.timer); pending.reject(error); pending = null; }
    };
    worker.onerror = () => stop(new Error("P9 Worker failed. Build the new WASM binary and serve the demo directory."));
    worker.onmessage = ({ data }) => {
      if (!pending || data.id !== pending.id) return;
      const call = pending; pending = null; clearTimeout(call.timer);
      if (data.error) call.reject(new Error(data.error));
      else {
        current.memory = data.memory_bytes || data.result?.memory_bytes || current.memory;
        if (Number.isFinite(data.call_ms)) window.dispatchEvent(new CustomEvent("athlesia:wasm-metrics", {
          detail: { call_ms: data.call_ms, memory_bytes: channels.reduce((n, item) => n + item.memory, 0) }
        }));
        call.resolve(data.result);
      }
    };
    const current = { memory: 0, stop, call(data, signal) {
      if (pending) return Promise.reject(new Error("Rust call already in progress"));
      if (signal?.aborted) return Promise.reject(new DOMException("Cancelled", "AbortError"));
      return new Promise((resolve, reject) => {
        const id = ++nextId;
        const abort = () => stop(new DOMException("Cancelled", "AbortError"));
        const done = fn => value => { signal?.removeEventListener("abort", abort); fn(value); };
        pending = { id, resolve: done(resolve), reject: done(reject),
          timer: setTimeout(() => stop(new Error("P9 Worker deadline exceeded")), 30000) };
        signal?.addEventListener("abort", abort, { once: true }); worker.postMessage({ id, ...data });
      });
    } };
    channels.push(current); return current;
  }
  async function step(request, signal) {
    const run = generation;
    const target = receiver;
    const bytes = JSON.stringify(request);
    try {
      const response = await target.call({ op: "step", request: bytes }, signal);
      active(run, signal);
      transcript.push({ request: bytes, response }); publish(); return response;
    } catch (error) {
      active(run, signal);
      transcript.push({ request: bytes, error: error.message }); publish(); throw error;
    }
  }
  async function rejectStep(request, expected, signal) {
    try { await step(request, signal); }
    catch (error) { if (expected.test(error.message)) return true; throw error; }
    throw new Error("A forbidden operation was accepted by Rust");
  }
  const bundle = (index, sequence, signal) => origins[index].call({ op: "bundle", sequence }, signal);
  return {
    async end_to_end_causal_runtime({ op }, { signal } = {}) {
      const run = generation;
      if (op === "learn") {
        seed = checkedSeed(getSeed()); transcript = []; checks = {};
        driver = driverFactory(); origins = [channel(), channel()];
        const hello = await Promise.all(origins.map((origin, index) => origin.call({ op: "init", role: "origin", seed, origin: 7001 + index }, signal)));
        active(run, signal);
        publicKeys = hello.map(item => item.public_key); identity = hello[0].identity;
        receiver = channel();
        const receiving = await receiver.call({ op: "init", role: "receiver", seed,
          public_a: publicKeys[0], public_b: publicKeys[1] }, signal);
        active(run, signal);
        if (receiving.identity.wasm_sha256 !== identity.wasm_sha256 || hello[1].identity.wasm_sha256 !== identity.wasm_sha256) throw new Error("Worker executable identities differ");
        const [a, b] = await Promise.all([bundle(0, 1, signal), bundle(1, 1, signal)]);
        const forged = a.replace('"origin_sequence":1', '"origin_sequence":2');
        if (forged === a) throw new Error("Could not construct the negative authentication trial");
        checks.forged_packet_rejected = await rejectStep({ op: "learn", a: forged, b }, /Authentication|Integrity|Signature/, signal);
        const result = await step({ op: "learn", a, b }, signal); publish(); return result;
      }
      if (!receiver) throw new Error("Initialize the P9 receiver first");
      if (op === "sabotage") {
        const fault = await origins[0].call({ op: "sabotage" }, signal);
        active(run, signal);
        if (fault.source_trial_result !== "CONTRADICTION") throw new Error("Source fault was not confirmed by Rust");
        const result = await step({ op: "sabotage", revocation: fault.revocation }, signal);
        if (driver.writes !== 0 || result.receiver_dispatches !== 0) throw new Error("An operation escaped the revoked boundary");
        checks.stale_lease_blocked = true; publish(); return result;
      }
      if (op === "revalidate") {
        const restored = await origins[0].call({ op: "restore" }, signal);
        active(run, signal);
        return step({ op: "remap", a: restored }, signal);
      }
      throw new Error("Unknown runtime operation");
    },
    async frozen_transfer({ op }, { signal } = {}) {
      const run = generation;
      if (!receiver) throw new Error("No P9 receiver");
      if (op === "generate_domain") return step({ op: "alien", a: await bundle(0, 3, signal), b: await bundle(1, 1, signal) }, signal);
      if (op === "execute") {
        checks.host_policy_denial = await rejectStep({ op: "dispatch", host_allows: false }, /PolicyDenied/, signal);
        const dispatch = await step({ op: "dispatch", host_allows: true }, signal);
        active(run, signal);
        const targetDriver = driver;
        let observed;
        try { observed = await targetDriver.execute(dispatch); }
        catch (error) {
          active(run, signal);
          await step({ op: "missing" }, signal);
          throw error;
        }
        active(run, signal);
        const result = await step({ op: "settle", observed: observed.committed === true }, signal);
        checks.replay_rejected = await rejectStep({ op: "dispatch", host_allows: true }, /requires a fresh target lease/, signal);
        if (driver.writes !== 1) throw new Error("Expected exactly one committed database write");
        checks.real_database_readback = true; publish(); return result;
      }
      throw new Error("Unknown transfer operation");
    },
    async inspectDatabase() { if (!driver) throw new Error("No database run yet"); return driver.read(); },
    async replayAttempt() {
      if (!checks.replay_rejected) throw new Error("Finish the P9 run first");
      await rejectStep({ op: "dispatch", host_allows: true }, /requires a fresh target lease/);
      publish(); return driver.read();
    },
    proof() {
      if (!checks.real_database_readback || !checks.replay_rejected) throw new Error("Proof is incomplete");
      return { schema: "athlesia/p9-reproduction/1", seed, public_a: publicKeys[0], public_b: publicKeys[1],
        transcript, executable: identity, database: { key: driver.recordKey, writes: driver.writes }, checks,
        scope: "Native P9 replay verifies kernel outcomes and signatures. IndexedDB readback is local evidence; no physical robot or external operator is attested." };
    },
    reset() {
      generation += 1;
      for (const item of channels) item.stop(new DOMException("Run reset", "AbortError"));
      channels = []; receiver = null; origins = null; transcript = []; checks = {}; driver = null; publish();
    }
  };
}

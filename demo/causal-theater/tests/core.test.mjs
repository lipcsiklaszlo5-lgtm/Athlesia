import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { StateMachine, Journey, validateResult, boundedCall, delay, SCHEMA } from "../core.js";
import { MockTheaterAdapter, WasmTheaterAdapter, createAdapter } from "../adapters.js";

async function fullRun(options = {}) {
  const adapter = new MockTheaterAdapter();
  const states = [];
  const results = [];
  const journey = new Journey(adapter, { pace: 0, onChange: value => states.push(value), onResult: (operation, value) => results.push([operation, value]), ...options });
  for (let i = 0; i < 4; i += 1) assert.equal(await journey.act(), true);
  return { journey, adapter, states, results };
}

test("FSM rejects skipping chapters and unknown phases", () => {
  const machine = new StateMachine();
  assert.equal(machine.state, "ready");
  assert.throws(() => machine.transition("complete"), /Illegal/);
  assert.throws(() => machine.transition("alien-ready"), /Illegal/);
  assert.throws(() => machine.transition("anything"), /Illegal/);
  machine.transition("learning");
  assert.throws(() => machine.transition("learning"), /Illegal/);
  machine.transition("error");
  assert.throws(() => machine.transition("learned"), /Illegal/);
  machine.reset();
  assert.equal(machine.state, "ready");
});

test("the complete journey has exactly the required ordered states", async () => {
  const { journey, states, results } = await fullRun();
  assert.equal(journey.state, "complete");
  assert.deepEqual(states, ["learning", "learned", "sabotaging", "blocked", "remapping", "recovered", "generating", "alien-ready", "transferring", "complete"]);
  assert.deepEqual(results.map(([op]) => op), ["learn", "sabotage", "remap", "alien", "transfer"]);
  assert.equal(await journey.act(), false);
});

test("mock derives a unique bounded conjunction from four interventions", async () => {
  const result = await new MockTheaterAdapter().learn();
  assert.equal(result.source_interventions, 4);
  assert.deepEqual(result.guards, ["resource", "valve"]);
  assert.equal(result.kernel_us, null);
  assert.equal(result.neural_weights, 0);
  assert.equal(result.evidence[1].candidates_remaining, 1);
});

test("revocation blocks dispatch, recovery never revives the old lease", async () => {
  const { results } = await fullRun();
  const blocked = results.find(([op]) => op === "sabotage")[1];
  const recovered = results.find(([op]) => op === "remap")[1];
  assert.equal(blocked.dispatch_issued, false);
  assert.equal(blocked.lease_revoked, true);
  assert.ok(blocked.epoch_after > blocked.epoch_before);
  assert.equal(recovered.old_lease_valid, false);
  assert.equal(recovered.fresh_lease, true);
  assert.equal(recovered.dispatch_issued, false);
});

test("transfer keeps the frozen structure, explicit map, zero updates and honest provenance", async () => {
  const { results } = await fullRun();
  const learned = results[0][1];
  const alien = results[3][1];
  const transfer = results[4][1];
  assert.equal(transfer.structure_digest, learned.structure_digest);
  assert.equal(alien.mapping_policy, "EXPLICIT_TYPED_ROLES");
  assert.deepEqual(alien.mapping.map(item => item.to), ["Replica", "Quorum", "Commit"]);
  assert.equal(transfer.target_training_updates, 0);
  assert.equal(transfer.training_epochs, 0);
  assert.equal(transfer.protocol_verified, false);
  assert.equal(transfer.lease_consumed, true);
  assert.equal(transfer.evidence.at(-1).rust_proof, false);
});

test("double clicks cannot launch a second backend operation", async () => {
  const adapter = new MockTheaterAdapter();
  let calls = 0;
  const original = adapter.learn.bind(adapter);
  adapter.learn = async () => { calls += 1; await delay(10); return original(); };
  const journey = new Journey(adapter, { pace: 0 });
  const first = journey.act();
  assert.equal(journey.busy, true);
  assert.equal(await journey.act(), false);
  assert.equal(await first, true);
  assert.equal(calls, 1);
});

test("reset cancels in-flight presentation and rejects its late result", async () => {
  const adapter = new MockTheaterAdapter();
  const original = adapter.learn.bind(adapter);
  let resolve;
  const pending = new Promise(yes => { resolve = yes; });
  adapter.learn = () => pending;
  const results = [];
  const journey = new Journey(adapter, { pace: 0, onResult: value => results.push(value) });
  const run = journey.act();
  await Promise.resolve();
  journey.reset();
  resolve(await original());
  assert.equal(await run, false);
  assert.equal(journey.state, "ready");
  assert.equal(journey.busy, false);
  assert.equal(results.length, 0);
});

test("reset cancels an animation pause without advancing a new run", async () => {
  const journey = new Journey(new MockTheaterAdapter(), { pace: 1 });
  const run = journey.act();
  await delay(5);
  journey.reset();
  assert.equal(await run, false);
  assert.equal(journey.state, "ready");
});

test("backend deadline produces a terminal fail-closed error", async () => {
  const adapter = new MockTheaterAdapter();
  adapter.learn = () => new Promise(() => {});
  const journey = new Journey(adapter, { pace: 0, timeoutMs: 10 });
  assert.equal(await journey.act(), false);
  assert.equal(journey.state, "error");
  assert.match(journey.error.message, /deadline/);
  assert.equal(await journey.act(), false);
});

test("result errors cannot render a complete state", async () => {
  const adapter = new MockTheaterAdapter();
  adapter.learn = async () => ({ schema: SCHEMA, engine: "mock", status: "CONFIRMED" });
  const journey = new Journey(adapter, { pace: 0 });
  assert.equal(await journey.act(), false);
  assert.equal(journey.state, "error");
  assert.match(journey.error.message, /contract rejected/);
});

test("changed frozen structure fails closed during revalidation", async () => {
  const adapter = new MockTheaterAdapter();
  const remap = adapter.remap.bind(adapter);
  adapter.remap = async () => ({ ...await remap(), structure_digest: "changed" });
  const journey = new Journey(adapter, { pace: 0 });
  assert.equal(await journey.act(), true);
  assert.equal(await journey.act(), false);
  assert.equal(journey.state, "error");
  assert.match(journey.error.message, /structure changed/);
});

test("adapter itself rejects out-of-order calls and transfer replay", async () => {
  const adapter = new MockTheaterAdapter();
  await assert.rejects(() => adapter.transfer(), /No compatible/);
  await assert.rejects(() => adapter.sabotage(), /No learned/);
  await adapter.learn();
  await assert.rejects(() => adapter.learn(), /not ready/);
  await adapter.sabotage();
  await adapter.remap();
  await adapter.alien();
  await adapter.transfer();
  await assert.rejects(() => adapter.transfer(), /No compatible/);
});

test("result contract rejects fake mock Rust latency", async () => {
  const result = await new MockTheaterAdapter().learn();
  assert.throws(() => validateResult("learn", { ...result, kernel_us: 200 }, "mock"), /Rust timing/);
});

test("result contract rejects false provenance, oversized or empty evidence", async () => {
  const result = await new MockTheaterAdapter().learn();
  for (const changed of [{ ...result, engine: "wasm" }, { ...result, evidence: [] }, { ...result, evidence: Array(33).fill({ event: "x" }) }, { ...result, padding: "x".repeat(66000) }]) {
    assert.throws(() => validateResult("learn", changed, "mock"), /contract rejected/);
  }
  assert.throws(() => validateResult("unknown", result, "mock"), /unknown operation/);
});

test("revocation contract refuses any dispatch and non-monotonic epochs", async () => {
  const adapter = new MockTheaterAdapter();
  await adapter.learn();
  const result = await adapter.sabotage();
  for (const changed of [{ ...result, dispatch_issued: true }, { ...result, epoch_after: result.epoch_before }, { ...result, epoch_after: Infinity }]) {
    assert.throws(() => validateResult("sabotage", changed, "mock"), /contract rejected/);
  }
});

test("mapping contract rejects duplicate target roles", async () => {
  const { results } = await fullRun();
  const alien = structuredClone(results[3][1]);
  alien.mapping[1].to = alien.mapping[0].to;
  assert.throws(() => validateResult("alien", alien, "mock"), /non-bijective/);
});

test("zero-shot contract rejects training, unconfirmed output and fake CKS verification", async () => {
  const { results } = await fullRun();
  const result = results[4][1];
  for (const changed of [{ ...result, target_training_updates: 1 }, { ...result, training_epochs: 1 }, { ...result, proof_result: "UNKNOWN" }, { ...result, protocol_verified: true }, { ...result, lease_consumed: false }]) {
    assert.throws(() => validateResult("transfer", changed, "mock"), /contract rejected/);
  }
  assert.throws(() => validateResult("transfer", { ...result, engine: "wasm", protocol_verified: false }, "wasm"), /unverified WASM/);
});

test("bounded calls and waits honor abort", async () => {
  const controller = new AbortController();
  const waiting = delay(1000, controller.signal);
  controller.abort();
  await assert.rejects(waiting, { name: "AbortError" });
  await assert.rejects(boundedCall(() => 1, controller.signal), { name: "AbortError" });
  await assert.rejects(delay(1, controller.signal), { name: "AbortError" });
});

test("WASM mode without Worker support never falls back to mock", async () => {
  await assert.rejects(() => createAdapter("wasm"), /Workers unavailable/);
  await assert.rejects(() => createAdapter("magic"), /Unknown engine/);
  assert.throws(() => new WasmTheaterAdapter({}), /incomplete/);
});

test("WASM adapter uses the runtime and frozen-transfer seams, with no added fake results", async () => {
  const calls = [];
  const bridge = {
    end_to_end_causal_runtime: async request => { calls.push(request.op); return { raw: true }; },
    frozen_transfer: async request => { calls.push(request.op); return { raw: true }; },
    reset: () => calls.push("reset"),
  };
  const adapter = await createAdapter("wasm", bridge);
  assert.equal(adapter.engine, "wasm");
  assert.deepEqual(await adapter.learn(), { raw: true });
  await adapter.sabotage();
  await adapter.remap();
  await adapter.alien();
  await adapter.transfer();
  adapter.reset();
  assert.deepEqual(calls, ["learn", "sabotage", "revalidate", "generate_domain", "execute", "reset"]);
});

test("two reset runs are deterministic and do not carry authority across runs", async () => {
  const { journey, results } = await fullRun();
  const original = structuredClone(results);
  results.length = 0;
  journey.reset();
  assert.equal(journey.state, "ready");
  for (let i = 0; i < 4; i += 1) await journey.act();
  assert.deepEqual(results, original);
});

test("HTML has unique IDs and all app lookup targets; no external dependencies", async () => {
  const base = new URL("../", import.meta.url);
  const html = await readFile(new URL("index.html", base), "utf8");
  const app = await readFile(new URL("app.js", base), "utf8");
  const css = await readFile(new URL("style.css", base), "utf8");
  const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]);
  assert.equal(ids.length, new Set(ids).size);
  for (const [, id] of app.matchAll(/\$\("([^"]+)"\)/g)) assert.ok(ids.includes(id), `Missing #${id}`);
  for (const [, id] of html.matchAll(/url\(#([^)]*)\)/g)) assert.ok(ids.includes(id), `Missing SVG #${id}`);
  assert.match(html, /Live P9 Rust\/WASM/);
  assert.match(html, /source machine is a software plant/);
  assert.match(html, /<dialog/);
  assert.match(css, /prefers-reduced-motion/);
  assert.doesNotMatch(html, /(?:src|href)="https?:\/\//);
  assert.doesNotMatch(app, /\.innerHTML\s*=/);
});

import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { checkedSeed, checkedSummary, nextNativeAction } from "../native-proof-core.js";
import { browserMetrics } from "../telemetry.js";

test("visitor seeds preserve the full WASM u32 range and reject malformed input", () => {
  for (const seed of [0, 42, 4294967295]) assert.equal(checkedSeed(String(seed)), seed);
  for (const seed of ["", " ", "nan", -1, 4294967296, 2.5]) assert.throws(() => checkedSeed(seed));
});

test("a green Rust holdout requires all five kernel checks", () => {
  const summary = { record: "summary", status: "HOLDOUT_VERIFIED", holdout_positive: true, holdout_negative: true, ledger_verified: true, source_model_unchanged: true, first_attempt_transfer: true };
  assert.equal(checkedSummary(JSON.stringify(summary), "HOLDOUT_VERIFIED").summary.status, "HOLDOUT_VERIFIED");
  for (const key of ["holdout_positive", "holdout_negative", "ledger_verified", "source_model_unchanged", "first_attempt_transfer"]) {
    assert.throws(() => checkedSummary(JSON.stringify({ ...summary, [key]: false }), "HOLDOUT_VERIFIED"));
  }
  assert.throws(() => checkedSummary(JSON.stringify(summary), "REVISED"));
  assert.throws(() => checkedSummary("not json", "REVISED"));
  assert.throws(() => checkedSummary("x".repeat(1048577), "REVISED"));
});

test("native challenge actions depend on the Rust phase, including unresolved probes", () => {
  assert.equal(nextNativeAction("CONTRADICTED"), "probe");
  assert.equal(nextNativeAction("PROBING"), "probe");
  assert.equal(nextNativeAction("REVISED"), "holdout");
  assert.equal(nextNativeAction("HOLDOUT_VERIFIED"), "replay");
  assert.equal(nextNativeAction("ABSTAIN"), "start");
  assert.throws(() => nextNativeAction("invented success"));
});

test("hardware values remain unavailable when browser APIs are absent", () => {
  assert.deepEqual(browserMetrics({}, {}), { logical_cores_reported: null, device_ram_gib_estimate: null, browser_js_heap_bytes: null });
  assert.deepEqual(browserMetrics({ hardwareConcurrency: 8, deviceMemory: 4 }, { memory: { usedJSHeapSize: 1234 } }), { logical_cores_reported: 8, device_ram_gib_estimate: 4, browser_js_heap_bytes: 1234 });
});

test("the primary journey uses P9 Workers and preserves the physical validation boundary", async () => {
  const base = new URL("../", import.meta.url);
  const worker = await readFile(new URL("p9-worker.js", base), "utf8");
  const html = await readFile(new URL("index.html", base), "utf8");
  const lab = await readFile(new URL("app.js", base), "utf8");
  const loader = await readFile(new URL("load-kernel.js", base), "utf8");
  const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(item => item[1]);
  assert.equal(ids.length, new Set(ids).size);
  for (const [, id] of lab.matchAll(/\$\("([^"]+)"\)/g)) assert.ok(ids.includes(id), `Missing #${id}`);
  assert.match(worker, /new kernel\.module\.WasmTheaterReceiver/);
  assert.match(loader, /SHA-256/);
  assert.doesNotMatch(worker, /import.*adapters/);
  assert.match(html, /Physical robot validation requires a hardware trial/);
  assert.match(html, /color-scheme" content="light/);
  assert.doesNotMatch(html, /src="\.\/proof-lab\.js"/);
  assert.match(lab, /params.get\("engine"\) \|\| "wasm"/);
});

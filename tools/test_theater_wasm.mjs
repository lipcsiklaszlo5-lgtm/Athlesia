// Executes the actual browser bridge and three real WASM instances in Node.
// The host driver here is real SQLite; the browser host uses IndexedDB.
// Requires Node >=22 with node:sqlite, and an already generated WASM package.
import { Worker as NodeWorker } from "node:worker_threads";
import { DatabaseSync } from "node:sqlite";
import { writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import assert from "node:assert/strict";
import { Journey } from "../demo/causal-theater/core.js";
import { WasmTheaterAdapter } from "../demo/causal-theater/adapters.js";
import { createWasmBridge } from "../demo/causal-theater/wasm-bridge.js";
import { checkedDispatch } from "../demo/causal-theater/database-driver.js";

class WorkerHost {
  constructor(url) {
    this.worker = new NodeWorker(new URL("./theater_node_worker.mjs", import.meta.url), { workerData: { entry: url.href } });
    this.worker.on("message", data => this.onmessage?.({ data }));
    this.worker.on("error", error => this.onerror?.(error));
  }
  postMessage(data) { this.worker.postMessage(data); }
  terminate() { void this.worker.terminate(); }
}
globalThis.Worker = WorkerHost;
globalThis.window = new EventTarget();

class SQLiteDriver {
  #writes = 0;
  #attempted = false;
  constructor() {
    this.recordKey = crypto.randomUUID();
    this.db = new DatabaseSync(":memory:");
    this.db.exec("CREATE TABLE dispatches (id TEXT PRIMARY KEY, committed INTEGER, dispatch_digest TEXT, commitment_digest TEXT)");
  }
  get writes() { return this.#writes; }
  async execute(dispatch) {
    checkedDispatch(dispatch);
    assert.equal(this.#attempted, false); this.#attempted = true;
    this.db.prepare("INSERT INTO dispatches VALUES (?, 1, ?, ?)").run(this.recordKey, dispatch.dispatch_digest, dispatch.commitment_digest);
    this.#writes += 1;
    return this.read();
  }
  async read() {
    const row = this.db.prepare("SELECT * FROM dispatches WHERE id = ?").get(this.recordKey);
    return row ? { ...row, committed: row.committed === 1 } : null;
  }
}
const seed = Number(process.argv[2] ?? 42);
const output = resolve(process.argv[3] ?? "athlesia-p9-proof.json");
let driver, latest;
const bridge = await createWasmBridge({ getSeed: () => seed,
  driverFactory: () => (driver = new SQLiteDriver()), onUpdate: run => { latest = run; } });
const journey = new Journey(new WasmTheaterAdapter(bridge), { pace: 0, timeoutMs: 90000 });
try {
  for (let step = 0; step < 4; step += 1) {
    assert.equal(await journey.act(), true, journey.error?.message);
    if (step < 3) assert.equal(driver.writes, 0);
  }
  assert.equal(journey.state, "complete", journey.error?.message);
  assert.equal(driver.writes, 1);
  for (const value of Object.values(latest.checks)) assert.equal(value, true);
  const record = await bridge.replayAttempt();
  assert.equal(record.committed, true); assert.equal(driver.writes, 1);
  await writeFile(output, JSON.stringify({ ...bridge.proof(), host_driver: "Node SQLite integration trial" }, null, 2) + "\n");
  console.log(`PASS: three real WASM instances / signatures / stale lease / fresh admission / SQLite commit / readback / replay. Proof: ${output}`);
} finally { bridge.reset(); driver?.db.close(); }

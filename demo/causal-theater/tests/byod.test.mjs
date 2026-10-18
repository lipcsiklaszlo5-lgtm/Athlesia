import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const base = new URL("../", import.meta.url);
const read = name => readFile(new URL(name, base), "utf8");

test("P13 makes editable evidence the primary visible experience", async () => {
  const [html, css] = await Promise.all([read("index.html"), read("byod.css")]);
  for (const id of ["byod-source-table", "byod-target-table", "byod-run", "byod-download", "byod-verify"]) assert.match(html, new RegExp(`id="${id}"`));
  assert.match(html, /Edit one row/);
  assert.match(html, /Upload CSV/g);
  assert.match(css, /\.break-test/);
  assert.match(css, /#experience > \.theater/);
});

test("P13 browser path invokes proof and replay exports with no mock fallback", async () => {
  const [loader, worker] = await Promise.all([read("byod-load-kernel.js"), read("byod-worker.js")]);
  assert.match(loader, /run_byod_break_test_json/);
  assert.match(loader, /verify_byod_break_test_json/);
  assert.match(worker, /module\.run_byod_break_test_json/);
  assert.match(worker, /module\.verify_byod_break_test_json/);
  assert.doesNotMatch(worker, /import.*adapters|MockTheaterAdapter/);
});

test("suggested edits change real target bytes and always re-enter Rust", async () => {
  const ui = await read("byod-ui.js");
  assert.match(ui, /applySuggestedEdit/);
  assert.match(ui, /await runProof\(\)/);
  assert.match(ui, /holdout\[effect\.after\] = holdout\[effect\.before\]/);
  assert.match(ui, /rows\.splice\(2\)/);
  assert.match(ui, /bridge\.run\(\$\("byod-source-raw"\)\.value, \$\("byod-target-raw"\)\.value\)/);
});

test("proof download preserves raw Rust JSON so u64 values are not rounded", async () => {
  const [ui, worker] = await Promise.all([read("byod-ui.js"), read("byod-worker.js")]);
  assert.match(ui, /rawBundle = result\.raw/);
  assert.match(ui, /new Blob\(\[rawBundle\]/);
  assert.match(worker, /raw,/);
  assert.match(ui, /predicted_after_id/);
});

test("the declared bounded contract and limitations remain visible", async () => {
  const html = await read("index.html");
  assert.match(html, /≤16 fields/);
  assert.match(html, /≤128 episodes/);
  assert.match(html, /≤256 KiB/);
  assert.match(html, /does not claim general AGI/);
  assert.match(html, /does not.*issue a P9 execution lease/);
});

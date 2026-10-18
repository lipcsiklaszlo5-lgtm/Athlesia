import { ByodBridge } from "./byod-bridge.js";

const $ = id => document.getElementById(id);
const MAX_FILE_BYTES = 256 * 1024;
const SOURCE_SAMPLE = `episode,action,before.power,before.coolant,before.open,after.power,after.coolant,after.open
1,press,1,0,0,1,0,1
2,press,0,1,0,0,1,1
3,press,1,1,0,1,1,1
`;
const TARGET_SAMPLE = `episode,action,after.commit,before.replica,after.quorum,before.commit,after.replica,before.quorum
1,write,1,0,1,0,0,1
2,write,1,1,0,0,1,0
3,write,1,0,0,0,0,0
`;
const VERDICTS = new Set(["VERIFIED", "CONTRADICTED", "ABSTAIN", "REFUSED"]);
const VERDICT_COPY = {
  VERIFIED: ["PROOF VERIFIED", "The rule survived your unseen domain."],
  CONTRADICTED: ["CONTRADICTION DETECTED", "You broke the claim. Athlesia stopped."],
  ABSTAIN: ["EVIDENCE INSUFFICIENT", "There isn't enough evidence to know."],
  REFUSED: ["INPUT REFUSED", "The evidence contract was rejected."],
};

const bridge = new ByodBridge();
let busy = false;
let rawBundle = "";
let currentBundle = null;
let sourceMatrix;
let targetMatrix;

function matrix(csv) {
  const normalized = csv.replaceAll("\r\n", "\n").replace(/\n+$/, "");
  if (!normalized) throw new Error("CSV is empty.");
  const rows = normalized.split("\n").map(line => line.split(",").map(value => value.trim()));
  if (rows.length < 2 || rows.some(row => row.length !== rows[0].length)) throw new Error("Rows must have the same number of comma-separated cells.");
  return rows;
}

function csv(rows) {
  return `${rows.map(row => row.join(",")).join("\n")}\n`;
}

function displayTable(kind, rows) {
  const host = $(`byod-${kind}-table`);
  host.replaceChildren();
  const table = document.createElement("table");
  const head = document.createElement("thead");
  const headRow = document.createElement("tr");
  rows[0].forEach(label => {
    const cell = document.createElement("th");
    cell.textContent = label;
    headRow.append(cell);
  });
  head.append(headRow);
  const body = document.createElement("tbody");
  rows.slice(1).forEach((values, rowIndex) => {
    const row = document.createElement("tr");
    values.forEach((value, columnIndex) => {
      const cell = document.createElement("td");
      const header = rows[0][columnIndex];
      let editor;
      const normalizedBoolean = value.toLowerCase();
      if ((header.startsWith("before.") || header.startsWith("after.")) && ["0", "1", "true", "false"].includes(normalizedBoolean)) {
        editor = document.createElement("select");
        for (const optionValue of ["0", "1"]) {
          const option = document.createElement("option");
          option.value = optionValue;
          option.textContent = optionValue;
          editor.append(option);
        }
        editor.value = ["1", "true"].includes(normalizedBoolean) ? "1" : "0";
      } else {
        editor = document.createElement("input");
        editor.value = value;
        editor.inputMode = header === "episode" ? "numeric" : "text";
      }
      editor.setAttribute("aria-label", `${kind} row ${rowIndex + 1} ${header}`);
      editor.addEventListener("change", () => {
        rows[rowIndex + 1][columnIndex] = editor.value;
        $(`byod-${kind}-raw`).value = csv(rows);
        markDirty();
      });
      cell.append(editor);
      row.append(cell);
    });
    body.append(row);
  });
  table.append(head, body);
  host.append(table);
}

function loadTable(kind, text) {
  const host = $(`byod-${kind}-table`);
  try {
    const rows = matrix(text);
    if (kind === "source") sourceMatrix = rows;
    else targetMatrix = rows;
    $(`byod-${kind}-raw`).value = text;
    displayTable(kind, rows);
  } catch (error) {
    host.replaceChildren();
    const message = document.createElement("p");
    message.className = "table-error";
    message.textContent = error.message;
    host.append(message);
  }
}

function setBusy(value) {
  busy = value;
  $("byod-run").disabled = value;
  $("byod-run").setAttribute("aria-busy", String(value));
  document.querySelectorAll("#break-test input, #break-test textarea, #break-test select, #break-test button:not(#byod-run)").forEach(control => {
    control.disabled = value || (["byod-download", "byod-verify"].includes(control.id) && !rawBundle);
  });
  $("byod-run-label").textContent = value ? "RUNNING COMPILED RUST" : "RUN THE PROOF";
}

function markDirty() {
  rawBundle = "";
  currentBundle = null;
  $("byod-download").disabled = true;
  $("byod-verify").disabled = true;
  $("byod-verdict-panel").dataset.verdict = "IDLE";
  $("byod-verdict-code").textContent = "EVIDENCE CHANGED";
  $("byod-verdict-title").textContent = "The previous proof no longer applies.";
  $("byod-verdict-detail").textContent = "Run the Rust kernel again. The UI will not carry authority across edited bytes.";
  $("byod-proof-id").textContent = "—";
  $("byod-holdouts").textContent = "—";
  $("byod-updates").textContent = "—";
  $("byod-latency").textContent = "—";
  $("byod-ledger").textContent = "—";
  $("byod-mapping").replaceChildren();
  $("byod-episodes").replaceChildren();
  $("byod-replay-result").textContent = "Changed evidence has no proof yet.";
}

function checkBundle(bundle) {
  if (bundle?.schema !== "athlesia/byod-break-test-proof/1" || !bundle.report || !VERDICTS.has(bundle.report.status)) throw new Error("Rust returned an unsupported P13 proof contract.");
  if (!/^[0-9a-f]{16}$/.test(bundle.proof_id) || bundle.report.target_training_updates !== 0 || bundle.report.execution_authority_issued !== false) throw new Error("Rust proof violated the zero-training or authority boundary.");
  return bundle;
}

function drawMapping(report) {
  const graph = $("byod-mapping");
  graph.replaceChildren();
  for (const [index, item] of (report.mapping || []).entries()) {
    const node = document.createElement("div");
    node.className = "byod-node";
    node.style.setProperty("--node-delay", `${index * 60}ms`);
    const number = document.createElement("span");
    number.className = "byod-node-index";
    number.textContent = `${item.source_node} → ${item.target_node}`;
    const roles = document.createElement("strong");
    roles.textContent = `${item.source_role} → ${item.target_role}`;
    const type = document.createElement("small");
    type.textContent = "typed structural role";
    node.append(number, roles, type);
    graph.append(node);
  }
}

function drawEpisodes(report) {
  const host = $("byod-episodes");
  host.replaceChildren();
  if (!report.episodes?.length) {
    const empty = document.createElement("p");
    empty.textContent = "No independent holdout result was admitted.";
    host.append(empty);
    return;
  }
  const table = document.createElement("table");
  const head = document.createElement("thead");
  const headRow = document.createElement("tr");
  ["holdout", "predicted digest", "observed digest", "result"].forEach(value => {
    const cell = document.createElement("th");
    cell.textContent = value;
    headRow.append(cell);
  });
  head.append(headRow);
  const body = document.createElement("tbody");
  for (const item of report.episodes) {
    const row = document.createElement("tr");
    [`episode ${item.episode}`, item.predicted_after_id ?? "unavailable", item.observed_after_id, item.matched ? "MATCH" : "MISMATCH"].forEach((value, index) => {
      const cell = document.createElement("td");
      cell.textContent = String(value);
      if (index === 3) cell.className = item.matched ? "match" : "mismatch";
      row.append(cell);
    });
    body.append(row);
  }
  table.append(head, body);
  host.append(table);
}

function render(bundle, metrics, identity) {
  const report = bundle.report;
  const copy = VERDICT_COPY[report.status];
  $("byod-verdict-panel").dataset.verdict = report.status;
  $("byod-verdict-code").textContent = `${copy[0]} / ${report.reason_code}`;
  $("byod-verdict-title").textContent = copy[1];
  $("byod-verdict-detail").textContent = report.reason;
  $("byod-proof-id").textContent = bundle.proof_id;
  $("byod-holdouts").textContent = `${report.target_verified_records}/${report.target_holdout_records}`;
  $("byod-updates").textContent = String(report.target_training_updates);
  $("byod-latency").textContent = `${metrics.call_ms.toFixed(2)} ms`;
  $("byod-ledger").textContent = report.ledger_verified ? "CHAIN VERIFIED" : "NOT ADMITTED";
  $("byod-authority").textContent = report.execution_authority_issued ? "ISSUED" : "NOT ISSUED";
  $("byod-engine").className = "break-engine live";
  $("byod-engine").querySelector("strong").textContent = "LIVE RUST / LOCAL WASM";
  $("byod-engine").querySelector("span").textContent = `${(identity.wasm_bytes / 1048576).toFixed(2)} MiB executable · SHA ${identity.wasm_sha256.slice(0, 12)}…`;
  $("byod-hardware").textContent = `${navigator.hardwareConcurrency || "unavailable"} logical cores reported · ${navigator.deviceMemory || "unavailable"} GiB RAM estimate · ${(metrics.memory_bytes / 1048576).toFixed(1)} MiB WASM linear memory`;
  $("byod-native-command").textContent = `cargo run --manifest-path crates/athlesia_causal_wasm/Cargo.toml --example byod_break_test_verify -- --verify athlesia-break-test-${bundle.proof_id}.json`;
  $("byod-replay-result").textContent = `Rust issued proof ${bundle.proof_id}. Download it or change the evidence.`;
  drawMapping(report);
  drawEpisodes(report);
}

async function runProof() {
  if (busy) return;
  setBusy(true);
  try {
    const result = await bridge.run($("byod-source-raw").value, $("byod-target-raw").value);
    const bundle = checkBundle(result.parsed);
    rawBundle = result.raw; // Preserve Rust JSON bytes; never parse and reserialize for proof download.
    currentBundle = bundle;
    render(bundle, result.metrics, result.identity);
    $("byod-download").disabled = false;
    $("byod-verify").disabled = false;
  } catch (error) {
    rawBundle = "";
    currentBundle = null;
    $("byod-verdict-panel").dataset.verdict = "ERROR";
    $("byod-verdict-code").textContent = "NO RUST VERDICT";
    $("byod-verdict-title").textContent = "The executable must answer.";
    $("byod-verdict-detail").textContent = `${error.message} Build the WASM package before presenting.`;
    $("byod-engine").className = "break-engine failed";
    $("byod-engine").querySelector("strong").textContent = "RUST / WASM UNAVAILABLE";
  } finally {
    setBusy(false);
  }
}

function effectColumns(rows) {
  if (!rows || rows.length < 2) throw new Error("A mapping witness is required first.");
  const header = rows[0];
  const witness = rows[1];
  const candidates = [];
  for (const [index, name] of header.entries()) {
    if (!name.startsWith("before.")) continue;
    const label = name.slice(7);
    const after = header.indexOf(`after.${label}`);
    if (after >= 0 && witness[index] !== witness[after]) candidates.push({ label, before: index, after });
  }
  if (candidates.length !== 1) throw new Error("The mapping witness must expose exactly one changed field.");
  return candidates[0];
}

function applySuggestedEdit(kind) {
  const rows = targetMatrix.map(row => [...row]);
  const effect = effectColumns(rows);
  if (rows.length < 3) throw new Error("This edit requires at least one held-out target row.");
  if (kind === "starvation") rows.splice(2);
  else if (kind === "contradiction") {
    const holdout = rows.at(-1);
    holdout[effect.after] = holdout[effect.before];
  } else if (kind === "harmless") {
    const header = rows[0];
    const holdout = rows[2];
    const before = header.findIndex(name => name.startsWith("before.") && name.slice(7) !== effect.label);
    if (before < 0) throw new Error("No non-effect context field is available.");
    const after = header.indexOf(`after.${header[before].slice(7)}`);
    const toggled = ["1", "true"].includes(holdout[before].toLowerCase()) ? "0" : "1";
    holdout[before] = toggled;
    holdout[after] = toggled;
  } else throw new Error("Unknown break-test edit.");
  const text = csv(rows);
  loadTable("target", text);
  markDirty();
}

async function loadFile(kind, file) {
  if (!file) return;
  if (file.size > MAX_FILE_BYTES) throw new Error("CSV exceeds the 256 KiB domain budget.");
  const text = await file.text();
  loadTable(kind, text);
  markDirty();
}

function download() {
  if (!rawBundle || !currentBundle) return;
  const url = URL.createObjectURL(new Blob([rawBundle], { type: "application/json" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = `athlesia-break-test-${currentBundle.proof_id}.json`;
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

async function verify() {
  if (!rawBundle || busy) return;
  setBusy(true);
  try {
    const result = await bridge.verify(rawBundle);
    if (result.parsed?.verified !== true || result.parsed.proof_id !== currentBundle.proof_id) throw new Error("Rust rejected the proof replay.");
    $("byod-replay-result").textContent = `REPLAY VERIFIED IN RUST / ${result.parsed.proof_id} / exact input bytes reproduced`;
  } catch (error) {
    $("byod-replay-result").textContent = `REPLAY REJECTED / ${error.message}`;
  } finally {
    setBusy(false);
  }
}

function reset() {
  loadTable("source", SOURCE_SAMPLE);
  loadTable("target", TARGET_SAMPLE);
  markDirty();
  $("byod-verdict-code").textContent = "REFERENCE EVIDENCE RESTORED";
  $("byod-verdict-title").textContent = "Now make the kernel earn the result.";
}

loadTable("source", SOURCE_SAMPLE);
loadTable("target", TARGET_SAMPLE);
$("byod-hardware").textContent = `${navigator.hardwareConcurrency || "unavailable"} logical cores reported · ${navigator.deviceMemory || "unavailable"} GiB RAM estimate · WASM not loaded yet`;
$("byod-run").disabled = false;
$("byod-run").addEventListener("click", runProof);
$("byod-download").addEventListener("click", download);
$("byod-verify").addEventListener("click", verify);
$("byod-reset").addEventListener("click", reset);
for (const kind of ["source", "target"]) {
  $(`byod-${kind}-raw`).addEventListener("input", markDirty);
  $(`byod-${kind}-apply`).addEventListener("click", () => {
    loadTable(kind, $(`byod-${kind}-raw`).value);
    markDirty();
  });
  $(`byod-${kind}-file`).addEventListener("change", event => loadFile(kind, event.target.files[0]).catch(error => {
    $("byod-run-note").textContent = error.message;
  }));
}
document.querySelectorAll("[data-byod-edit]").forEach(button => button.addEventListener("click", async () => {
  try {
    applySuggestedEdit(button.dataset.byodEdit);
    await runProof();
  } catch (error) {
    $("byod-run-note").textContent = error.message;
  }
}));
window.addEventListener("pagehide", () => bridge.close(), { once: true });

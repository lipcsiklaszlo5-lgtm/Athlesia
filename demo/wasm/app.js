import init, { WasmFreshWorldSession } from "./pkg/athlesia_causal_wasm.js";

const $ = (id) => document.getElementById(id);
const STORAGE_KEY = "athlesia.fresh-world.session.v1";
let session = null;
let latestJsonl = "";
let lastCallMs = 0;

function recordsFrom(jsonl) {
  return jsonl.trim().split(/\n+/).filter(Boolean).map((line) => JSON.parse(line));
}

function recordOf(records, name) {
  return records.find((record) => record.record === name);
}

function latestSummary(records) {
  return [...records].reverse().find((record) => record.record === "summary") || null;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function setStatus(message, error = false) {
  $("status").textContent = message;
  $("status").classList.toggle("error", error);
}

function persistSession() {
  if (!session) return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      version: 1,
      seed: session.seed(),
      phase: session.phase(),
      lastCallMs,
      updatedAt: new Date().toISOString(),
    }));
  } catch (error) {
    // Private browsing or a restrictive embedded frame may disable storage.
    // The live session still works; persistence is an enhancement only.
    console.warn("Athlesia session persistence unavailable", error);
  }
}

function clearPersistedSession() {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (error) {
    console.warn("Athlesia session storage could not be cleared", error);
  }
}

function restoreSession() {
  let saved;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return false;
    saved = JSON.parse(raw);
  } catch (error) {
    clearPersistedSession();
    return false;
  }

  if (!saved || saved.version !== 1 || !Number.isSafeInteger(saved.seed) || saved.seed < 1) {
    clearPersistedSession();
    return false;
  }

  const expectedPhase = String(saved.phase || "");
  if (!["GENERATED", "SABOTAGED", "HOLDOUT_VERIFIED"].includes(expectedPhase)) {
    clearPersistedSession();
    return false;
  }

  try {
    session = new WasmFreshWorldSession(saved.seed);
    let jsonl = session.world_jsonl();
    if (expectedPhase === "SABOTAGED" || expectedPhase === "HOLDOUT_VERIFIED") {
      jsonl = session.sabotage();
    }
    if (expectedPhase === "HOLDOUT_VERIFIED") {
      jsonl = session.transfer_to_unknown();
    }
    if (session.phase() !== expectedPhase) throw new Error("restored phase mismatch");

    latestJsonl = jsonl;
    lastCallMs = Number.isFinite(Number(saved.lastCallMs)) ? Number(saved.lastCallMs) : 0;
    render(jsonl, lastCallMs);
    $("sabotage").disabled = expectedPhase !== "GENERATED";
    $("transfer").disabled = expectedPhase !== "SABOTAGED";
    setStatus(`Restored local session ${saved.seed} at ${expectedPhase}.`);
    return true;
  } catch (error) {
    clearPersistedSession();
    session = null;
    latestJsonl = "";
    lastCallMs = 0;
    console.warn("Athlesia saved session was discarded", error);
    return false;
  }
}

function render(jsonl, callMs) {
  latestJsonl = jsonl;
  const records = recordsFrom(jsonl);
  const world = recordOf(records, "fresh_world");
  const summary = latestSummary(records);
  const stages = records.filter((record) => record.record === "stage");
  const graphs = records.filter((record) => record.record === "graph");
  const tokens = recordOf(records, "tokens");

  $("phase").textContent = world?.phase || session?.phase() || "—";
  $("seed").textContent = world?.seed ?? session?.seed() ?? "—";
  $("latency").textContent = `${callMs.toFixed(2)} ms`;
  $("digest").textContent = summary?.report_digest ? String(summary.report_digest) : "pending";

  $("worlds").classList.remove("empty");
  const tokenCards = tokens ? Object.entries(tokens).map(([worldName, values]) => {
    const items = Object.entries(values).map(([role, token]) => `<code>${escapeHtml(role)}: ${escapeHtml(token)}</code>`).join("");
    return `<div class="world-card"><h3>${escapeHtml(worldName.toUpperCase())}</h3>${items}</div>`;
  }).join("") : "<div class=\"empty\">Token records appear after generation.</div>";
  $("worlds").innerHTML = tokenCards;

  $("timeline").classList.remove("empty");
  $("timeline").innerHTML = stages.map((stage) => {
    const name = String(stage.stage || "UNKNOWN");
    const detail = String(stage.detail || "");
    let color = "";
    if (name === "FALSIFY" || detail.includes("CONTRADICTION")) color = "red";
    if (name === "REVISE" || name === "HOLDOUT" || name === "TARGET_TRIAL") color = "green";
    if (name === "TRANSFER" || name === "ACTIVE_PROBE") color = "blue";
    return `<div class="event ${color}"><strong>${escapeHtml(name)}</strong><span>${escapeHtml(detail)}</span></div>`;
  }).join("");

  if (summary) {
    const baseline = summary.naive_baseline_negative_failure ? "FAILED negative holdout" : "not run";
    const athlesia = summary.holdout_positive && summary.holdout_negative ? "PASSED both holdouts" : "pending";
    $("comparison").innerHTML = `
      <div class="comparison-card"><span>NAIVE BASELINE</span><strong>${escapeHtml(baseline)}</strong><small>No guard revision; negative case is mishandled.</small></div>
      <div class="comparison-card"><span>ATHLESIA</span><strong>${escapeHtml(athlesia)}</strong><small>positive=${summary.holdout_positive} · negative=${summary.holdout_negative} · probes=${summary.probes}</small></div>`;
  }

  $("raw").textContent = jsonl;
  $("download").disabled = false;
}

function randomSeed() {
  const values = new Uint32Array(1);
  crypto.getRandomValues(values);
  return values[0] || 1;
}

function run(label, callback) {
  try {
    const start = performance.now();
    const jsonl = callback();
    const elapsed = performance.now() - start;
    lastCallMs = elapsed;
    render(jsonl, elapsed);
    persistSession();
    setStatus(`${label} completed in ${elapsed.toFixed(2)} ms of browser-observed WASM time.`);
    return true;
  } catch (error) {
    setStatus(`${label} failed: ${error}`, true);
    return false;
  }
}

$("generate").addEventListener("click", () => {
  const seed = randomSeed();
  session = new WasmFreshWorldSession(seed);
  lastCallMs = 0;
  render(session.world_jsonl(), 0);
  persistSession();
  $("sabotage").disabled = false;
  $("transfer").disabled = true;
  setStatus(`Fresh world ${seed} generated. The solver has not acted.`);
});

$("sabotage").addEventListener("click", () => {
  if (!session) return;
  if (run("Sabotage", () => session.sabotage())) $("transfer").disabled = false;
});

$("transfer").addEventListener("click", () => {
  if (!session) return;
  if (run("Transfer and holdout", () => session.transfer_to_unknown())) {
    $("sabotage").disabled = true;
    $("transfer").disabled = true;
  }
});

$("download").addEventListener("click", () => {
  const blob = new Blob([latestJsonl], { type: "application/jsonl" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `athlesia-fresh-world-${session?.seed() ?? "unknown"}.jsonl`;
  link.click();
  URL.revokeObjectURL(url);
});

try {
  await init();
  $("generate").disabled = false;
  if (!restoreSession()) setStatus("Rust/WASM kernel loaded. Generate a fresh world to begin.");
} catch (error) {
  setStatus(`WASM load failed: ${error}`, true);
}

window.addEventListener("pagehide", persistSession);

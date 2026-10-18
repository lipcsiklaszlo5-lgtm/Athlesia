import { Journey } from "./core.js";
import { createAdapter } from "./adapters.js";
import { createWasmBridge } from "./wasm-bridge.js";
import { checkedSeed } from "./native-proof-core.js";

const $ = id => document.getElementById(id);
const params = new URLSearchParams(location.search);
const requestedEngine = params.get("engine") || "wasm";
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
const evidence = [];
let journey;
let engine = requestedEngine;
let learnedResult;
let transferResult;
let bootstrapError;
let bridge;

function updateProof(run) {
  $("proof-records").textContent = `${run.transcript.length} original P9 requests / verdicts`;
  $("proof-record-key").textContent = run.recordKey || "Not created.";
  $("proof-outcome").textContent = `${run.writes} actual database ${run.writes === 1 ? "write" : "writes"} committed.`;
  if (run.identity) {
    $("proof-hash").textContent = run.identity.wasm_sha256;
    const build = run.identity.build;
    $("proof-build").textContent = build ? `${build.source_commit}${build.source_dirty ? " + local changes" : ""} · ${build.profile}` : "No manifest; executable bytes were fingerprinted locally.";
    $("proof-engine").textContent = "LIVE P9 / ED25519 / RUST";
    $("proof-engine").classList.add("live");
  }
  const names = { signature: "forged_packet_rejected", lease: "stale_lease_blocked", policy: "host_policy_denial", database: "real_database_readback", replay: "replay_rejected" };
  for (const [name, key] of Object.entries(names)) $("check-" + name).classList.toggle("done", run.checks[key] === true);
  const complete = Object.values(names).every(key => run.checks[key] === true);
  $("proof-download").disabled = !complete;
  $("proof-readback").disabled = !complete;
  $("proof-attack").disabled = !complete;
  $("proof-phase").textContent = complete ? "P9 GATE + DATABASE OUTCOME VERIFIED" : run.checks.stale_lease_blocked ? "REVOKED LEASE BLOCKED BY RUST" : "P9 CHALLENGE IN PROGRESS";
  $("proof-detail").textContent = complete
    ? "The database commit completed, a separate read matched the exact dispatch, and Rust rejected replay. Download this run for a native P9 reproduction."
    : run.checks.stale_lease_blocked ? "Rust withheld all protected commands. Database writes so far: 0. Recovery requires a new endorsement and a fresh lease."
      : "Two local origins exchange signed receipts with an isolated Rust receiver. Target data is not used for model training.";
}

$("proof-download").addEventListener("click", () => {
  try {
    const url = URL.createObjectURL(new Blob([JSON.stringify(bridge.proof(), null, 2) + "\n"], { type: "application/json" }));
    const link = document.createElement("a"); link.href = url; link.download = "athlesia-p9-proof.json";
    document.body.append(link); link.click(); link.remove(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  } catch (error) { $("proof-detail").textContent = error.message; }
});
$("proof-readback").addEventListener("click", async () => {
  try {
    const record = await bridge.inspectDatabase();
    if (!record || record.committed !== true) throw new Error("The database record is missing.");
    $("proof-detail").textContent = `A fresh database read found record ${record.id}. Dispatch ${record.dispatch_digest}. You can inspect it in DevTools → Application → IndexedDB → athlesia-p9-proof.`;
  } catch (error) { $("proof-detail").textContent = error.message; }
});
$("proof-attack").addEventListener("click", async () => {
  $("proof-attack").disabled = true;
  try {
    const record = await bridge.replayAttempt();
    if (!record) throw new Error("Original database record disappeared.");
    $("proof-detail").textContent = "Your replay request reached Rust and was rejected. The database driver was not called again. The original committed record is still present.";
  } catch (error) { $("proof-detail").textContent = error.message; }
  finally { $("proof-attack").disabled = true; }
});

const COPY = Object.freeze({
  ready: {
    chapter: 1, kicker: "01 — THE PHYSICIST", title: "First, discover what makes it work.",
    description: "Observe an actuator's effect in a software plant. Then carry its frozen structure across an execution boundary.",
    button: "INITIALIZE CAUSAL KERNEL", hint: "One mechanism. Three moments of understanding.", stage: "AWAITING INTERACTION", authority: "Not issued",
  },
  learning: {
    chapter: 1, kicker: "01 — EXTRACTING THE STRUCTURE", title: "The machine becomes a rule.",
    description: "Two local Rust origins learn an output effect, sign receipt-backed packets, and send them to a separate receiver.",
    button: "EXTRACTING CAUSAL STRUCTURE", hint: "The animation is presentation time, not a benchmark.", stage: "SOURCE INTERVENTIONS", authority: "Learning", busy: true,
  },
  learned: {
    chapter: 1, kicker: "01 — STRUCTURE LEARNED", title: "Not the pixels. The cause.",
    description: "The source output effect is frozen. Resource and interlock constraints remain explicit host policy. The receiver has prepared a live Trust Lease.",
    button: "SABOTAGE ENVIRONMENT", hint: "Break an assumption. Watch what happens to authority.", stage: "CAUSAL STRUCTURE FROZEN", authority: "Source scoped", danger: true,
  },
  sabotaging: {
    chapter: 2, kicker: "02 — THE ASSUMPTION BREAKS", title: "What if the world stops agreeing?",
    description: "Jam a source software-plant trial. Rust confirms the contradiction; its origin signs an endorsement withdrawal before the receiver dispatches.",
    button: "CHECKING LIVE TRUST FRONTIER", hint: "A prediction is not permission to act.", stage: "CONTRADICTION INJECTED", authority: "Checking", busy: true,
  },
  blocked: {
    chapter: 2, kicker: "02 — FAIL-CLOSED", title: "Contradiction detected. Permission revoked.",
    description: "The prepared lease is stale. No dispatch is issued. The safest decision is not to act.",
    button: "TRUST LEASE REVOKED", hint: "ABSTAIN · zero protected database writes.", stage: "EXECUTION WITHHELD", authority: "ABSTAIN", busy: true,
  },
  remapping: {
    chapter: 2, kicker: "02 — REVALIDATE, NEVER ASSUME", title: "Recover the evidence. Not the old permit.",
    description: "The host restores the original source effect and supplies a fresh signed endorsement. Rust rechecks admission; the old lease stays invalid.",
    button: "REVALIDATING SOURCE SCOPE", hint: "Recovery is not automatic authority to dispatch.", stage: "REVALIDATING SOURCE", authority: "Withheld", busy: true,
  },
  recovered: {
    chapter: 2, kicker: "02 — SOURCE REVALIDATED", title: "A fresh lease. The same hard boundary.",
    description: "Source evidence is revalidated. The old lease cannot be replayed. Now change everything except the causal structure.",
    button: "GENERATE ALIEN DOMAIN", hint: "Different names. Different geometry. Compatible roles.", stage: "FRESH FRONTIER AVAILABLE", authority: "Fresh scope",
  },
  generating: {
    chapter: 3, kicker: "03 — ANOTHER WORLD", title: "The software plant becomes a database.",
    description: "Resource becomes replica. Valve becomes quorum. Door becomes commit. The host supplies an explicit typed role map.",
    button: "GENERATING ALIEN DOMAIN", hint: "Compatible structure is required; appearance is irrelevant.", stage: "TYPED DOMAIN BINDING", authority: "Not dispatched", busy: true,
  },
  "alien-ready": {
    chapter: 3, kicker: "03 — THE ZERO-SHOT MOMENT", title: "New world. No new training.",
    description: "Apply the frozen rule through the compatible role map. No model update or target-domain training is allowed.",
    button: "EXECUTE ZERO-SHOT TRANSFER", hint: "One frozen structure. Zero target training updates.", stage: "TARGET MODEL UPDATES: 0", authority: "Awaiting lease",
  },
  transferring: {
    chapter: 3, kicker: "03 — STRUCTURE IN MOTION", title: "Transfer the logic. Keep the safeguards.",
    description: "Map the frozen structure, check the current lease and host policy, then compare the outcome with the commitment.",
    button: "MAPPING FROZEN KNOWLEDGE", hint: "Compatibility → admission → dispatch → observation.", stage: "APPLYING FROZEN STRUCTURE", authority: "Checking lease", busy: true,
  },
  complete: {
    chapter: 3, kicker: "03 — TRANSFER COMPLETE", title: "Different world. Same intelligence.",
    description: "One actual database record was committed and read back. Rust confirmed the outcome and rejected an attempt to dispatch the same lease again.",
    button: "EXPERIENCE AGAIN", hint: "Inspect the ledger. Every claim has a provenance.", stage: "OUTCOME CONFIRMED", authority: "Lease consumed", outline: true,
  },
  error: {
    chapter: 1, kicker: "EXECUTION WITHHELD", title: "No evidence. No green light.",
    description: "The backend did not return an acceptable result. Nothing is presented as a verified execution.",
    button: "RESTART EXPERIENCE", hint: "The causal ledger retains the failure details.", stage: "FAIL-CLOSED / BACKEND ERROR", authority: "WITHHELD", outline: true,
  },
});

const LEDGER_COPY = Object.freeze({
  learn: ["A causal rule was frozen", "Four source observations establish the output effect. Interlocks remain host policy; positive receipts do not identify them."],
  sabotage: ["Dispatch was withheld", "A software-plant fault contradicts the source prediction. A signed revocation makes the prepared lease fail the live P9 check."],
  remap: ["Source scope was revalidated", "New admissible source evidence produces a fresh lease. The old lease remains invalid; no action is dispatched by recovery."],
  alien: ["A compatible target was bound", "The fixture host supplies an explicit, bijective typed role map. This is not arbitrary schema understanding."],
  transfer: ["Frozen structure reached the target", "A compatible mapped rule produces the expected target outcome with zero target training updates."],
});

function render(state) {
  const view = COPY[state];
  $("theater-seed").disabled = state !== "ready";
  $("restart").disabled = state === "transferring";
  document.body.dataset.phase = state;
  $("scene-kicker").textContent = view.kicker;
  $("scene-title").textContent = view.title;
  $("scene-description").textContent = view.description;
  $("action-label").textContent = view.button;
  $("action-hint").textContent = view.hint;
  $("stage-state").textContent = view.stage;
  $("authority-value").textContent = view.authority;
  $("primary-action").disabled = Boolean(view.busy);
  $("primary-action").classList.toggle("danger", Boolean(view.danger));
  $("primary-action").classList.toggle("outline", Boolean(view.outline));
  $("primary-action").setAttribute("aria-busy", String(Boolean(view.busy)));
  const alien = ["generating", "alien-ready", "transferring", "complete"].includes(state);
  $("world-label").textContent = alien ? "ALIEN DOMAIN" : "SOURCE DOMAIN";
  $("world-name").textContent = alien ? "/ DATABASE 02" : "/ REACTOR 01";
  $("world-coordinate").textContent = alien ? "DOMAIN 02 · LOCAL INDEXEDDB" : "DOMAIN 01 · SOFTWARE PLANT";
  $("world-dot").style.background = state === "blocked" ? "var(--red)" : alien ? "var(--orange)" : "var(--green)";
  $("training-value").textContent = alien ? "0" : "—";
  document.querySelectorAll("[data-chapter]").forEach(item => {
    const chapter = Number(item.dataset.chapter);
    item.classList.toggle("current", chapter === view.chapter);
    item.classList.toggle("done", chapter < view.chapter || state === "complete");
    if (chapter === view.chapter) item.setAttribute("aria-current", "step");
    else item.removeAttribute("aria-current");
  });
  $("stage-viewport").setAttribute("aria-label", `${alien ? "Replica, quorum and commit in a database world." : "Resource, valve and door in a reactor world."} ${view.title} ${view.description}`);
  if (state === "learned" && learnedResult) {
    $("action-hint").textContent = learnedResult.kernel_us === null
      ? `${learnedResult.source_interventions} observations / origin · two signed local origins · timings below.`
      : `0 neural weights · Rust kernel: ${(learnedResult.kernel_us / 1000).toFixed(3)} ms.`;
  }
  if (state === "complete" && transferResult) {
    $("stage-state").textContent = engine === "wasm" && transferResult.protocol_verified
      ? "CKS/1 VERIFIED · OUTCOME CONFIRMED"
      : "MOCK TRANSFER COMPLETE · NOT RUST PROOF";
    $("action-hint").textContent = engine === "wasm"
      ? "Training epochs: 0 · CKS/1 protocol verified."
      : "Training epochs: 0 · CKS/1 verification awaits WASM.";
  }
  if (state === "error") {
    const message = journey?.error?.message || bootstrapError?.message || "Backend unavailable.";
    $("scene-description").textContent = message;
    recordFailure(message);
  }
}

function updateLedger() {
  $("ledger-count").textContent = String(evidence.length);
  $("ledger-download").disabled = evidence.length === 0;
  $("ledger-summary").textContent = evidence.length
    ? `${String(evidence.length).padStart(2, "0")} records · ${engine === "mock" ? "illustrative JS fixture" : "WASM bridge output"}`
    : "Waiting for the first interaction";
  $("ledger-raw").textContent = evidence.length ? evidence.map(item => JSON.stringify(item, null, 2)).join("\n\n") : "No records yet.";
}

function ledgerRow(operation, title, description, tag) {
  const row = document.createElement("li");
  const number = document.createElement("span");
  number.className = "ledger-event-number";
  number.textContent = String(evidence.length).padStart(2, "0");
  const content = document.createElement("div");
  const heading = document.createElement("strong");
  heading.textContent = title;
  const detail = document.createElement("p");
  detail.textContent = description;
  content.append(heading, detail);
  const badge = document.createElement("span");
  badge.className = "ledger-tag";
  badge.textContent = tag;
  row.dataset.operation = operation;
  row.append(number, content, badge);
  $("ledger-events").append(row);
}

function recordResult(operation, result, browserCallMs) {
  if (operation === "learn") learnedResult = result;
  if (operation === "transfer") transferResult = result;
  evidence.push({
    record: "athlesia/theater-observation/1",
    sequence: evidence.length + 1,
    provenance: engine === "mock" ? "illustrative-js-fixture" : "wasm-bridge-output",
    operation,
    browser_call_ms: Number(browserCallMs.toFixed(3)),
    timing_note: "Browser call duration, not Rust kernel latency. Animation delays excluded.",
    result,
  });
  const [title, description] = LEDGER_COPY[operation];
  ledgerRow(operation, title, description, engine === "mock" ? "MOCK" : "WASM");
  updateLedger();
}

function recordFailure(message) {
  // Only one error transition per run is allowed by the FSM.
  evidence.push({ record: "athlesia/theater-observation/1", sequence: evidence.length + 1, operation: "error", provenance: engine, message: String(message).slice(0, 2000), dispatch_issued_by_ui: false });
  ledgerRow("error", "Execution withheld", message, "REJECTED");
  updateLedger();
}

function restart() {
  evidence.length = 0;
  learnedResult = null;
  transferResult = null;
  $("ledger-events").replaceChildren();
  updateLedger();
  try {
    if (!journey) return location.reload();
    journey.reset();
    $("primary-action").focus({ preventScroll: true });
  } catch (error) {
    $("scene-description").textContent = error.message;
    $("primary-action").disabled = true;
    recordFailure(error.message);
  }
}

$("primary-action").addEventListener("click", () => {
  if (!journey || ["complete", "error"].includes(journey.state)) return restart();
  void journey.act();
});
$("restart").addEventListener("click", restart);
$("ledger-open").addEventListener("click", () => $("ledger-dialog").showModal());
$("ledger-close").addEventListener("click", () => $("ledger-dialog").close());
$("ledger-dialog").addEventListener("click", event => {
  if (event.target !== event.currentTarget) return;
  const rect = event.currentTarget.getBoundingClientRect();
  if (event.clientY < rect.top || event.clientX < rect.left || event.clientX > rect.right) event.currentTarget.close();
});
$("ledger-download").addEventListener("click", () => {
  if (!evidence.length) return;
  const url = URL.createObjectURL(new Blob([evidence.map(item => JSON.stringify(item)).join("\n") + "\n"], { type: "application/x-ndjson" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = `athlesia-theater-${engine}-ledger.jsonl`;
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
});
reducedMotion.addEventListener("change", () => { if (journey) journey.pace = reducedMotion.matches ? 0.2 : 1; });

try {
  bridge = requestedEngine === "wasm" ? await createWasmBridge({ getSeed: () => checkedSeed($("theater-seed").value), onUpdate: updateProof }) : null;
  const adapter = await createAdapter(requestedEngine, bridge);
  engine = adapter.engine;
  document.body.dataset.engine = engine;
  $("engine-label").textContent = engine === "mock" ? "INTERACTIVE MOCK" : "LIVE WASM BRIDGE";
  if (engine === "wasm") {
    $("provenance").textContent = "P9 Rust/WASM · software plant → real local IndexedDB operation.";
    $("ledger-description").textContent = "Backend-supplied evidence envelopes. Browser and animation durations are not kernel latency. Explicit typed role mappings remain required. Inspect raw records for scope and protocol evidence.";
  }
  if (engine === "mock") document.querySelector(".proof-lab").hidden = true;
  journey = new Journey(adapter, { onChange: render, onResult: recordResult, pace: reducedMotion.matches ? 0.2 : 1, timeoutMs: 90000 });
  render("ready");
} catch (error) {
  bootstrapError = error;
  document.body.dataset.phase = "error";
  document.body.dataset.engine = "unavailable";
  $("engine-label").textContent = "BACKEND UNAVAILABLE";
  $("provenance").textContent = "No P9 WASM bridge connected. No mock fallback was used.";
  render("error");
  $("scene-description").textContent = error.message;
  $("action-hint").textContent = "Rebuild the WASM package with bash tools/build_causal_theater.sh, then reload.";
  $("action-label").textContent = "RETRY CONNECTION";
}

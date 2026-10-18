import { checkedSeed, checkedSummary, nextNativeAction } from "./native-proof-core.js";

const $ = id => document.getElementById(id);
let worker;
let pending;
let requestId = 0;
let phase = "EMPTY";
let rawTrace = "";
let runSeed;
let identity;
let measurements = [];

const VIEWS = {
  READY_FOR_ACTION: ["SOURCE OBSERVED", "A real Rust session is waiting for your action.", "The hidden mechanism stays inside the Rust environment. The learner received opaque observation traces.", "EXECUTE OPAQUE ACTION", 1],
  TARGET_CONFIRMED: ["TARGET CONFIRMED", "The transferred effect survived its first trial.", "Now change the environment. The next result comes from a fresh Rust interaction, not from the presentation timeline.", "SABOTAGE THIS RUN", 1],
  CONTRADICTED: ["MODEL QUARANTINED", "You broke the prediction.", "Rust detected an outcome contradiction and quarantined the previous target model. Ask the kernel to select one discriminating experiment.", "RUN ONE ACTIVE PROBE", 2],
  PROBING: ["AMBIGUITY REMAINS", "One experiment was not enough.", "The Rust hypothesis frontier still contains alternatives. Run another bounded experiment; the UI cannot force a successful revision.", "RUN NEXT ACTIVE PROBE", 3],
  REVISED: ["LOCAL REVISION COMMITTED", "The kernel earned a revised rule.", "Transfer that revised structure into a previously untested holdout. Both a positive and a negative holdout must pass.", "VERIFY UNSEEN HOLDOUT", 3],
  HOLDOUT_VERIFIED: ["BOTH HOLDOUTS VERIFIED", "The Rust result agrees with the new world.", "Positive and negative holdouts passed; the source model stayed unchanged and its evidence chain verified. Try repeating the completed operation.", "TRY REPLAYING LAST STEP", 4],
  REPLAY_REJECTED: ["REPLAY REJECTED BY RUST", "A finished phase cannot be played twice.", "The repeated holdout call returned a Rust error and left the session unchanged. This checks session phase ordering; the P9 permit gate is a separate contract.", "RUN ANOTHER CHALLENGE", 5],
  ABSTAIN: ["RUST ABSTAINED", "The evidence did not resolve the world.", "No success is inferred from the animation. The original Rust trace records this unresolved result.", "TRY ANOTHER CHALLENGE", 2],
};

function setBusy(value) {
  $("proof-action").disabled = value;
  $("proof-action").setAttribute("aria-busy", String(value));
  $("proof-seed").disabled = value || !["EMPTY", "ERROR", "ABSTAIN", "REPLAY_REJECTED"].includes(phase);
  $("proof-random").disabled = $("proof-seed").disabled;
}

function terminate(error) {
  worker?.terminate();
  worker = null;
  if (pending) {
    clearTimeout(pending.timer);
    const reject = pending.reject;
    pending = null;
    reject(error);
  }
}

function nativeCall(operation, seed) {
  if (pending) return Promise.reject(new Error("A Rust call is already in flight."));
  if (!worker) {
    worker = new Worker(new URL("./kernel-worker.js", import.meta.url), { type: "module" });
    worker.onmessage = event => {
      if (!pending || event.data?.id !== pending.id) return;
      const request = pending;
      pending = null;
      clearTimeout(request.timer);
      if (event.data.ok) request.resolve(event.data);
      else request.reject(new Error(event.data.error || "Rust worker rejected the operation."));
    };
    worker.onerror = () => terminate(new Error("The Rust worker failed to load. Build the WASM package and serve the demo directory as documented."));
  }
  return new Promise((resolve, reject) => {
    const id = ++requestId;
    pending = { id, resolve, reject, timer: setTimeout(() => terminate(new Error("Rust call deadline exceeded. No success was reported.")), 30000) };
    worker.postMessage({ id, operation, seed });
  });
}

function download(name, contents, type) {
  const url = URL.createObjectURL(new Blob([contents], { type }));
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

$("proof-action").addEventListener("click", async () => {
  const operation = nextNativeAction(phase);
  let seed;
  try { seed = checkedSeed($("proof-seed").value); }
  catch (error) { $("proof-detail").textContent = error.message; return; }
  setBusy(true);
  $("proof-action-label").textContent = operation === "start" ? "LOADING REAL RUST / WASM" : "CALLING THE RUST KERNEL";
  try {
    if (operation === "start") {
      rawTrace = "";
      measurements = [];
      $("proof-download").disabled = true;
      $("proof-measurements").disabled = true;
      document.querySelectorAll("[data-proof-step]").forEach(item => item.classList.remove("done"));
    }
    const result = await nativeCall(operation, seed);
    const checked = checkedSummary(result.jsonl, result.phase);
    rawTrace = result.jsonl;
    identity = result.identity;
    runSeed = seed;
    phase = result.replay?.rejected_by_rust === true ? "REPLAY_REJECTED" : result.phase;
    const view = VIEWS[phase];
    if (!view) throw new Error("Unsupported Rust session phase.");
    $("proof-phase").textContent = view[0];
    $("proof-outcome").textContent = view[1];
    $("proof-detail").textContent = result.replay ? `${view[2]} Rust returned: ${result.replay.error}` : view[2];
    $("proof-action-label").textContent = view[3];
    $("proof-engine").textContent = "LIVE RUST / LOCAL WASM";
    $("proof-engine").classList.add("live");
    document.querySelector(".proof-lab").classList.remove("failed");
    $("proof-hash").textContent = identity.wasm_sha256;
    $("proof-build").textContent = identity.build ? `${identity.build.source_commit}${identity.build.source_dirty ? " + local changes" : ""} · ${identity.build.profile} · rustc ${identity.build.rustc}` : "No build manifest provided. Executable SHA-256 is measured from loaded bytes.";
    $("proof-replay-command").textContent = `cargo run --manifest-path crates/athlesia_integrated_cognitive_agent/Cargo.toml --example causal_theater_replay -- ${runSeed}`;
    $("proof-records").textContent = `${checked.count} original Rust records · ${checked.summary.probes} active probes`;
    $("proof-download").disabled = false;
    $("proof-measurements").disabled = false;
    document.querySelectorAll("[data-proof-step]").forEach(item => item.classList.toggle("done", Number(item.dataset.proofStep) <= view[4]));
    measurements.push({ operation, rust_phase: result.phase, replay: result.replay, ...result.metrics });
    window.dispatchEvent(new CustomEvent("athlesia:wasm-metrics", { detail: result.metrics }));
  } catch (error) {
    terminate(error);
    phase = "ERROR";
    document.querySelector(".proof-lab").classList.add("failed");
    $("proof-phase").textContent = "NO VERIFIED RESULT";
    $("proof-outcome").textContent = "The executable must answer.";
    $("proof-detail").textContent = `${error.message} Run bash tools/build_causal_theater.sh, then serve --directory demo and open /causal-theater/.`;
    $("proof-engine").textContent = "RUST RUN UNAVAILABLE";
    $("proof-engine").classList.remove("live");
    $("proof-action-label").textContent = "RETRY REAL RUST RUN";
  } finally { setBusy(false); }
});

$("proof-random").addEventListener("click", () => {
  $("proof-seed").value = String(crypto.getRandomValues(new Uint32Array(1))[0]);
});
$("proof-download").addEventListener("click", () => {
  if (rawTrace) download(`athlesia-rust-seed-${runSeed}.jsonl`, rawTrace, "application/x-ndjson");
});
$("proof-measurements").addEventListener("click", () => {
  if (identity) download(`athlesia-rust-seed-${runSeed}-measurements.json`, JSON.stringify({ schema: "athlesia/browser-run/1", seed: runSeed, executable: identity, timings: measurements, note: "WASM calls include snapshot serialization. Linear memory is reserved WASM memory, not total machine RAM or OS CPU load. The live opaque challenge does not exercise the P9 Trust Lease gate." }, null, 2) + "\n", "application/json");
});
window.addEventListener("pagehide", () => terminate(new Error("Page closed.")), { once: true });

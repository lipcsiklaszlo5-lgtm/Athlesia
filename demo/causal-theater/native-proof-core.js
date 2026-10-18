const PHASES = new Set(["READY_FOR_ACTION", "TARGET_CONFIRMED", "CONTRADICTED", "PROBING", "REVISED", "HOLDOUT_VERIFIED", "ABSTAIN"]);

export function checkedSeed(value) {
  if (String(value).trim() === "") throw new Error("Enter a challenge seed.");
  const number = Number(value);
  if (!Number.isInteger(number) || number < 0 || number > 4294967295) throw new Error("Seed must be an integer from 0 to 4294967295.");
  return number;
}

/** Parse only display facts; preserve the original Rust JSONL string byte for byte. */
export function checkedSummary(jsonl, phase) {
  if (typeof jsonl !== "string" || jsonl.length === 0 || jsonl.length > 1048576 || !PHASES.has(phase)) throw new Error("Rust trace is absent or exceeds the display budget.");
  const lines = jsonl.trim().split("\n");
  if (lines.length > 4096) throw new Error("Rust trace record budget exceeded.");
  const records = lines.map(line => JSON.parse(line));
  const summary = records.findLast(item => item.record === "summary");
  if (!summary || summary.status !== phase) throw new Error("Rust session phase and trace disagree.");
  if (phase === "HOLDOUT_VERIFIED" && !["holdout_positive", "holdout_negative", "ledger_verified", "source_model_unchanged", "first_attempt_transfer"].every(key => summary[key] === true)) {
    throw new Error("Rust did not confirm both holdouts, the frozen source and the evidence chain.");
  }
  return { summary, count: records.length };
}

export function nextNativeAction(phase) {
  const actions = {
    EMPTY: "start",
    READY_FOR_ACTION: "act",
    TARGET_CONFIRMED: "sabotage",
    CONTRADICTED: "probe",
    PROBING: "probe",
    REVISED: "holdout",
    HOLDOUT_VERIFIED: "replay",
    REPLAY_REJECTED: "start",
    ABSTAIN: "start",
    ERROR: "start",
  };
  if (!Object.hasOwn(actions, phase)) throw new Error("Unknown Rust challenge phase.");
  return actions[phase];
}

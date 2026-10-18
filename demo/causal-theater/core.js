/** Presentation contract. None of these browser types grant execution authority. */
export const SCHEMA = "athlesia/causal-theater/1";
export const PHASES = Object.freeze({
  ready: ["learning"],
  learning: ["learned", "error"],
  learned: ["sabotaging"],
  sabotaging: ["blocked", "error"],
  blocked: ["remapping", "error"],
  remapping: ["recovered", "error"],
  recovered: ["generating"],
  generating: ["alien-ready", "error"],
  "alien-ready": ["transferring"],
  transferring: ["complete", "error"],
  complete: [],
  error: [],
});

export class StateMachine {
  #state = "ready";
  get state() { return this.#state; }
  transition(next) {
    if (!PHASES[this.#state]?.includes(next)) {
      throw new Error(`Illegal journey transition: ${this.#state} → ${next}`);
    }
    this.#state = next;
    return next;
  }
  reset() { this.#state = "ready"; }
}

export function abortError() {
  return new DOMException("Journey cancelled", "AbortError");
}

export function delay(ms, signal) {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) return reject(abortError());
    const stop = () => {
      clearTimeout(timer);
      signal?.removeEventListener("abort", stop);
      reject(abortError());
    };
    const timer = setTimeout(() => {
      signal?.removeEventListener("abort", stop);
      resolve();
    }, ms);
    signal?.addEventListener("abort", stop, { once: true });
  });
}

export async function boundedCall(operation, signal, timeoutMs = 12000) {
  if (signal?.aborted) throw abortError();
  let timer;
  let stop;
  try {
    return await Promise.race([
      Promise.resolve().then(() => {
        if (signal?.aborted) throw abortError();
        return operation();
      }),
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error("Backend deadline exceeded; execution withheld.")), timeoutMs);
        stop = () => reject(abortError());
        signal?.addEventListener("abort", stop, { once: true });
      }),
    ]);
  } finally {
    clearTimeout(timer);
    signal?.removeEventListener("abort", stop);
  }
}

const EXPECTED = Object.freeze({
  learn: "LEARNED",
  sabotage: "ABSTAIN",
  remap: "REVALIDATED",
  alien: "DOMAIN_READY",
  transfer: "CONFIRMED",
});

function requireContract(condition, message) {
  if (!condition) throw new Error(`Backend contract rejected: ${message}`);
}

function nonempty(value) {
  return typeof value === "string" && value.length > 0 && value.length <= 512;
}

/** Reject missing, contradictory, oversized or misleading results before rendering success. */
export function validateResult(operation, value, engine) {
  requireContract(Object.hasOwn(EXPECTED, operation), "unknown operation");
  requireContract(value && typeof value === "object", "missing result");
  requireContract(value.schema === SCHEMA && value.engine === engine, "schema / provenance");
  requireContract(value.status === EXPECTED[operation], "unexpected status");
  requireContract(Array.isArray(value.evidence) && value.evidence.length > 0 && value.evidence.length <= 32, "evidence budget");
  requireContract(JSON.stringify(value).length <= 65536, "result budget");
  requireContract(value.evidence.every(event => event && nonempty(event.event)), "invalid evidence record");
  requireContract(engine === "mock" || engine === "wasm", "unknown engine");
  if (operation === "learn") {
    requireContract(nonempty(value.model_id) && nonempty(value.structure_digest), "missing frozen model");
    requireContract(value.neural_weights === 0, "non-symbolic model");
    requireContract(Number.isInteger(value.source_interventions) && value.source_interventions > 0, "source evidence");
    requireContract(Array.isArray(value.guards) && value.guards.every(nonempty), "invalid learned guards");
    if (engine === "mock") requireContract(value.guards.length === 2 && new Set(value.guards).size === 2, "bounded two-guard fixture");
    if (engine === "wasm") requireContract(value.guards.length === 0 &&
      Array.isArray(value.host_policy_guards) && value.host_policy_guards.length === 2 &&
      value.rule_scope === "observed source effect; interlocks are explicit host policy, not learned guards",
      "source effect and explicit host interlocks must remain distinct");
    requireContract(value.kernel_us === null || (Number.isFinite(value.kernel_us) && value.kernel_us >= 0), "invalid kernel timing");
    requireContract(engine !== "mock" || value.kernel_us === null, "mock cannot report Rust timing");
  }
  if (operation === "sabotage") {
    requireContract(value.lease_revoked === true && value.dispatch_issued === false, "revocation did not withhold dispatch");
    requireContract(value.reason === "STALE_EXECUTION_LEASE" && nonempty(value.prepared_commitment), "missing stale commitment");
    requireContract(Number.isSafeInteger(value.epoch_before) && Number.isSafeInteger(value.epoch_after) && value.epoch_after > value.epoch_before, "non-monotonic trust epoch");
  }
  if (operation === "remap") {
    requireContract(value.fresh_lease === true && value.old_lease_valid === false, "old lease revived");
    requireContract(value.dispatch_issued === false && nonempty(value.structure_digest), "revalidation is not dispatch");
  }
  if (operation === "alien") {
    requireContract(value.target_training_updates === 0 && value.mapping_policy === "EXPLICIT_TYPED_ROLES", "target mapping policy");
    requireContract(Array.isArray(value.mapping) && value.mapping.length === 3, "mapping size");
    requireContract(value.mapping.every(item => nonempty(item?.from) && nonempty(item?.to)), "mapping labels");
    requireContract(new Set(value.mapping.map(item => item.from)).size === 3 && new Set(value.mapping.map(item => item.to)).size === 3, "non-bijective mapping");
    requireContract(["effect.target", "guard.interlock", "guard.resource"].every(role => value.mapping.some(item => item.role === role)), "incompatible typed roles");
  }
  if (operation === "transfer") {
    requireContract(value.target_training_updates === 0 && value.training_epochs === 0, "target training occurred");
    requireContract(value.proof_result === "CONFIRMED" && value.dispatch_issued === true, "unconfirmed outcome");
    requireContract(nonempty(value.commitment_digest) && nonempty(value.plan_digest), "missing exact commitment");
    requireContract(nonempty(value.structure_digest) && value.lease_consumed === true, "missing lease consumption");
    requireContract(value.protocol === "CKS/1" && typeof value.protocol_verified === "boolean", "sync provenance");
    requireContract(engine !== "mock" || value.protocol_verified === false, "mock cannot verify CKS");
    requireContract(engine !== "wasm" || value.protocol_verified === true, "unverified WASM transfer");
  }
  return value;
}

/** Strict, cancellable orchestration. Only one action is allowed in each stable phase. */
export class Journey {
  #machine = new StateMachine();
  #controller = new AbortController();
  #busy = false;
  #generation = 0;
  #modelDigest = null;

  constructor(adapter, { onChange = () => {}, onResult = () => {}, pace = 1, timeoutMs = 12000 } = {}) {
    this.adapter = adapter;
    this.onChange = onChange;
    this.onResult = onResult;
    this.pace = pace;
    this.timeoutMs = timeoutMs;
  }
  get state() { return this.#machine.state; }
  get busy() { return this.#busy; }

  #advance(next) {
    this.#machine.transition(next);
    this.onChange(next);
  }

  async #result(operation, generation, signal) {
    const started = performance.now();
    const raw = await boundedCall(() => this.adapter[operation]({ signal }), signal, this.timeoutMs);
    if (signal.aborted || generation !== this.#generation) throw abortError();
    const result = validateResult(operation, raw, this.adapter.engine);
    if (operation === "learn") this.#modelDigest = result.structure_digest;
    if (operation === "remap" || operation === "transfer") {
      requireContract(result.structure_digest === this.#modelDigest, "frozen structure changed across domains");
    }
    this.onResult(operation, result, performance.now() - started);
    return result;
  }

  async act() {
    if (this.#busy || ["complete", "error"].includes(this.state)) return false;
    const initial = this.state;
    if (!["ready", "learned", "recovered", "alien-ready"].includes(initial)) return false;
    this.#busy = true;
    const generation = this.#generation;
    const signal = this.#controller.signal;
    const pause = ms => delay(ms * this.pace, signal);
    try {
      if (initial === "ready") {
        this.#advance("learning");
        await this.#result("learn", generation, signal);
        await pause(1800);
        this.#advance("learned");
      } else if (initial === "learned") {
        this.#advance("sabotaging");
        await this.#result("sabotage", generation, signal);
        this.#advance("blocked");
        await pause(2300);
        this.#advance("remapping");
        await this.#result("remap", generation, signal);
        await pause(1700);
        this.#advance("recovered");
      } else if (initial === "recovered") {
        this.#advance("generating");
        await this.#result("alien", generation, signal);
        await pause(1300);
        this.#advance("alien-ready");
      } else {
        this.#advance("transferring");
        await this.#result("transfer", generation, signal);
        await pause(2300);
        this.#advance("complete");
      }
      return true;
    } catch (error) {
      if (error.name === "AbortError" || generation !== this.#generation) return false;
      this.error = error;
      this.#advance("error");
      return false;
    } finally {
      if (generation === this.#generation) this.#busy = false;
    }
  }

  reset() {
    this.#controller.abort();
    this.#controller = new AbortController();
    this.#generation += 1;
    this.#busy = false;
    this.#modelDigest = null;
    this.error = null;
    this.adapter.reset();
    this.#machine.reset();
    this.onChange("ready");
  }
}

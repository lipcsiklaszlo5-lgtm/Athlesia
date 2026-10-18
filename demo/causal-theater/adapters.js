import { SCHEMA } from "./core.js";

const ROLE_MAP = Object.freeze([
  { from: "Resource", to: "Replica", role: "guard.resource" },
  { from: "Valve", to: "Quorum", role: "guard.interlock" },
  { from: "Door", to: "Commit", role: "effect.target" },
]);

/** Display fingerprint only. Not a cryptographic digest or a signature. */
function mockFingerprint(value) {
  let result = 2166136261;
  for (const char of JSON.stringify(value)) result = Math.imul(result ^ char.charCodeAt(0), 16777619);
  return `mock-fnv32:${(result >>> 0).toString(16).padStart(8, "0")}`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

/**
 * ILLUSTRATIVE JS FIXTURE — deliberately not Rust evidence.
 * A bounded truth table selects a conjunction; explicit role mapping reuses it.
 * Timing, signatures and CKS verification are NOT emulated as verified facts.
 */
export class MockTheaterAdapter {
  engine = "mock";
  #phase;
  #epoch;
  #rule;
  #structure;
  #prepared;
  #leaseEpoch;
  #mapping;
  constructor() { this.reset(); }

  reset() {
    this.#phase = "ready";
    this.#epoch = 1;
    this.#rule = null;
    this.#structure = null;
    this.#prepared = null;
    this.#leaseEpoch = null;
    this.#mapping = null;
  }

  #envelope(status, detail, evidence) {
    return { schema: SCHEMA, engine: this.engine, status, ...detail, evidence };
  }

  async learn() {
    assert(this.#phase === "ready", "Fixture is not ready to learn.");
    const interventions = [
      { resource: false, valve: false, door: false },
      { resource: true, valve: false, door: false },
      { resource: false, valve: true, door: false },
      { resource: true, valve: true, door: true },
    ];
    const candidates = [
      [], ["resource"], ["valve"], ["resource", "valve"],
    ].filter(guards => interventions.every(row => guards.every(key => row[key]) === row.door));
    assert(candidates.length === 1, "Ambiguous source evidence: ABSTAIN.");
    this.#rule = candidates[0];
    this.#structure = mockFingerprint({ guards: ["guard.resource", "guard.interlock"], effect: "effect.target" });
    this.#phase = "learned";
    return this.#envelope("LEARNED", {
      model_id: "mock:source-frozen-v1",
      structure_digest: this.#structure,
      neural_weights: 0,
      source_interventions: interventions.length,
      guards: [...this.#rule],
      kernel_us: null,
    }, [
      { event: "SOURCE_INTERVENTIONS", samples: interventions, provenance: "illustrative-js-fixture" },
      { event: "BOUNDED_CONJUNCTION_SELECTED", candidates_remaining: candidates.length, target_training_updates: 0 },
      { event: "FROZEN_STRUCTURE", digest: this.#structure, crypto_verified: false },
    ]);
  }

  async sabotage() {
    assert(this.#phase === "learned", "No learned source to sabotage.");
    const before = this.#epoch;
    this.#prepared = mockFingerprint({ model: this.#structure, command: 500, epoch: before });
    this.#leaseEpoch = before;
    // The fixture deliberately injects BOTH an outcome contradiction and a
    // trust-frontier revocation. Production must supply each actual evidence
    // event; a visual contradiction alone is not a signed P8 revocation.
    const predicted = this.#rule.every(() => true);
    const observed = false;
    assert(predicted !== observed, "Fixture failed to create a contradiction.");
    this.#epoch += 1;
    const allowed = this.#leaseEpoch === this.#epoch;
    assert(!allowed, "Revoked fixture lease must not authorize dispatch.");
    this.#phase = "blocked";
    return this.#envelope("ABSTAIN", {
      lease_revoked: true,
      dispatch_issued: false,
      reason: "STALE_EXECUTION_LEASE",
      prepared_commitment: this.#prepared,
      epoch_before: before,
      epoch_after: this.#epoch,
    }, [
      { event: "OUTCOME_CONTRADICTION", predicted, observed },
      { event: "REVOCATION_INJECTED", signature_verified: false, fixture_only: true },
      { event: "TRUST_FRONTIER_CHANGED", epoch_before: before, epoch_after: this.#epoch },
      { event: "DISPATCH_WITHHELD", decision: "ABSTAIN", physical_actions: 0 },
    ]);
  }

  async remap() {
    assert(this.#phase === "blocked", "Revalidation requires a blocked fixture.");
    const oldLeaseEpoch = this.#leaseEpoch;
    // Source-only recovery. This is NOT target training or autonomous discovery
    // of a new law. In Rust, require new admissible evidence + a fresh lease.
    this.#epoch += 1;
    this.#leaseEpoch = this.#epoch;
    this.#phase = "recovered";
    return this.#envelope("REVALIDATED", {
      fresh_lease: true,
      old_lease_valid: oldLeaseEpoch === this.#epoch,
      dispatch_issued: false,
      structure_digest: this.#structure,
    }, [
      { event: "SOURCE_REVALIDATED", scope: "restored-source-fixture", new_trust_evidence: "illustrative" },
      { event: "FRESH_LEASE_ISSUED", epoch: this.#epoch, old_lease_valid: false, dispatch_issued: false },
    ]);
  }

  async alien() {
    assert(this.#phase === "recovered", "Source revalidation is required first.");
    this.#mapping = ROLE_MAP.map(item => ({ ...item }));
    this.#phase = "alien-ready";
    return this.#envelope("DOMAIN_READY", {
      domain_id: "mock:replicated-database",
      target_training_updates: 0,
      mapping_policy: "EXPLICIT_TYPED_ROLES",
      mapping: this.#mapping,
    }, [{ event: "TYPED_DOMAIN_CREATED", mapping: this.#mapping, mapping_provided_by: "fixture-host", arbitrary_schema_discovery: false }]);
  }

  async transfer() {
    assert(this.#phase === "alien-ready", "No compatible alien domain.");
    assert(this.#leaseEpoch === this.#epoch, "Stale lease: ABSTAIN.");
    const structure = mockFingerprint({ guards: ["guard.resource", "guard.interlock"], effect: "effect.target" });
    assert(structure === this.#structure && this.#mapping.length === 3, "Incompatible typed structure: ABSTAIN.");
    // No target model update. Apply the frozen conjunction to explicitly bound
    // replica/quorum symbols, then compare prediction with fixture observation.
    const target = { resource: true, valve: true };
    const predicted = this.#rule.every(key => target[key]);
    const observed = target.resource && target.valve;
    assert(predicted === observed, "Target outcome contradiction: ABSTAIN.");
    this.#phase = "complete";
    return this.#envelope("CONFIRMED", {
      target_training_updates: 0,
      training_epochs: 0,
      proof_result: "CONFIRMED",
      dispatch_issued: true,
      commitment_digest: mockFingerprint({ target, structure, epoch: this.#epoch }),
      plan_digest: mockFingerprint({ steps: [199], structure }),
      structure_digest: structure,
      lease_consumed: true,
      protocol: "CKS/1",
      protocol_verified: false,
    }, [
      { event: "FROZEN_TYPED_MAPPING", structure_digest: structure, target_training_updates: 0 },
      { event: "MOCK_PROTOCOL_ENVELOPE", protocol: "CKS/1", protocol_verified: false },
      { event: "FIXTURE_DISPATCH", lease_epoch: this.#epoch, lease_consumed: true },
      { event: "FIXTURE_OUTCOME_CONFIRMED", predicted, observed, rust_proof: false },
    ]);
  }
}

/**
 * FUTURE WASM BOUNDARY. These are bridge contract methods, NOT claims that
 * crates/athlesia_causal_wasm currently exports these names. See wasm-bridge.js.
 * Missing/malformed/rejected WASM results never fall back to a mock success.
 */
export class WasmTheaterAdapter {
  engine = "wasm";
  constructor(bridge) {
    assert(bridge && ["end_to_end_causal_runtime", "frozen_transfer", "reset"].every(key => typeof bridge[key] === "function"), "P9 WASM theater bridge is incomplete.");
    this.bridge = bridge;
  }
  learn({ signal } = {}) {
    // Hook: actual source interaction / bounded causal learner / freeze.
    return this.bridge.end_to_end_causal_runtime({ op: "learn" }, { signal });
  }
  sabotage({ signal } = {}) {
    // Hook: real contradiction + authenticated revocation; validate stale lease
    // against the LIVE mesh before authorize_and_dispatch_trusted can dispatch.
    return this.bridge.end_to_end_causal_runtime({ op: "sabotage" }, { signal });
  }
  remap({ signal } = {}) {
    // Hook: restored admissible evidence + freshly issued lease, never old lease.
    return this.bridge.end_to_end_causal_runtime({ op: "revalidate" }, { signal });
  }
  alien({ signal } = {}) {
    // Hook: generate a bounded, typed target and expose its explicit role map.
    return this.bridge.frozen_transfer({ op: "generate_domain" }, { signal });
  }
  transfer({ signal } = {}) {
    // Hook: CKS decode/verification -> frozen candidate -> hierarchy -> trusted
    // dispatch -> observed outcome receipt; do not learn in the target domain.
    return this.bridge.frozen_transfer({ op: "execute" }, { signal });
  }
  reset() { this.bridge.reset(); }
}

export async function createAdapter(engine = "mock", bridge = null) {
  if (engine === "mock") return new MockTheaterAdapter();
  assert(engine === "wasm", "Unknown engine. Use ?engine=mock or ?engine=wasm.");
  if (!bridge) {
    const module = await import("./wasm-bridge.js");
    bridge = await module.createWasmBridge();
  }
  return new WasmTheaterAdapter(bridge);
}

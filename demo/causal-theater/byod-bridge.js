export class ByodBridge {
  #worker;
  #pending;
  #sequence = 0;
  #deadlineMs;

  constructor({ deadlineMs = 30000 } = {}) {
    this.#deadlineMs = deadlineMs;
  }

  #ensureWorker() {
    if (this.#worker) return;
    this.#worker = new Worker(new URL("./byod-worker.js", import.meta.url), { type: "module" });
    this.#worker.onmessage = event => {
      if (!this.#pending || event.data?.id !== this.#pending.id) return;
      const pending = this.#pending;
      this.#pending = null;
      clearTimeout(pending.timer);
      if (event.data.ok) pending.resolve(event.data);
      else pending.reject(new Error(event.data.error || "Rust worker rejected the operation."));
    };
    this.#worker.onerror = () => this.#fail(new Error("The P13 Rust worker failed to load."));
  }

  #fail(error) {
    if (this.#pending) {
      clearTimeout(this.#pending.timer);
      this.#pending.reject(error);
      this.#pending = null;
    }
    this.#worker?.terminate();
    this.#worker = null;
  }

  #call(payload) {
    if (this.#pending) return Promise.reject(new Error("A kernel operation is already in flight."));
    this.#ensureWorker();
    return new Promise((resolve, reject) => {
      const id = ++this.#sequence;
      const timer = setTimeout(() => this.#fail(new Error("Rust/WASM deadline exceeded. No verdict was reported.")), this.#deadlineMs);
      this.#pending = { id, resolve, reject, timer };
      this.#worker.postMessage({ id, ...payload });
    });
  }

  run(sourceCsv, targetCsv) {
    return this.#call({ operation: "run", source_csv: sourceCsv, target_csv: targetCsv });
  }

  verify(bundleJson) {
    return this.#call({ operation: "verify", bundle_json: bundleJson });
  }

  close() {
    this.#fail(new Error("Bridge closed."));
  }
}

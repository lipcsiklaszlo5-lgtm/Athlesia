// A real local database write after Rust authorization; never a production DB.
export function checkedDispatch(dispatch) {
  if (dispatch?.status !== "AUTHORIZED" || dispatch.lease_consumed !== true ||
      !Array.isArray(dispatch.actions) || dispatch.actions.length !== 1 || dispatch.actions[0] !== 199 ||
      !/^\d+$/.test(dispatch.dispatch_digest) || !/^\d+$/.test(dispatch.commitment_digest)) {
    throw new Error("No exact P9-authorized database operation");
  }
  return dispatch;
}
export class DatabaseDriver {
  #writes = 0;
  #attempted = false;
  constructor(recordKey = crypto.randomUUID()) { this.recordKey = recordKey; }
  get writes() { return this.#writes; }
  async #open() {
    if (!globalThis.indexedDB) throw new Error("IndexedDB unavailable; no simulated write will be substituted.");
    return new Promise((resolve, reject) => {
      const request = indexedDB.open("athlesia-p9-proof", 1);
      request.onupgradeneeded = () => request.result.createObjectStore("dispatches", { keyPath: "id" });
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
      request.onblocked = () => reject(new Error("Database upgrade is blocked by another tab."));
    });
  }
  async read() {
    const db = await this.#open();
    try {
      return await new Promise((resolve, reject) => {
        const tx = db.transaction("dispatches", "readonly");
        const request = tx.objectStore("dispatches").get(this.recordKey);
        tx.oncomplete = () => resolve(request.result || null);
        tx.onabort = () => reject(tx.error || new Error("Database read aborted"));
        tx.onerror = () => reject(tx.error);
      });
    } finally { db.close(); }
  }
  async execute(dispatch) {
    checkedDispatch(dispatch);
    if (this.#attempted) throw new Error("The database driver already attempted this run; retry is withheld.");
    this.#attempted = true;
    const db = await this.#open();
    const record = { id: this.recordKey, committed: true, dispatch_digest: dispatch.dispatch_digest,
      commitment_digest: dispatch.commitment_digest, engine: "real IndexedDB transaction" };
    try {
      await new Promise((resolve, reject) => {
        const tx = db.transaction("dispatches", "readwrite");
        tx.objectStore("dispatches").add(record);
        tx.oncomplete = () => { this.#writes += 1; resolve(); };
        tx.onabort = () => reject(tx.error || new Error("Database write aborted"));
        tx.onerror = () => reject(tx.error);
      });
    } finally { db.close(); }
    const observed = await this.read();
    if (!observed || observed.dispatch_digest !== dispatch.dispatch_digest || observed.committed !== true) {
      throw new Error("Database readback did not confirm the exact authorized write.");
    }
    return observed;
  }
}

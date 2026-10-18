// Node-only integration harness, not loaded by the web page.
import { parentPort, workerData } from "node:worker_threads";
import { readFile } from "node:fs/promises";
const originalFetch = globalThis.fetch;
globalThis.fetch = async (input, options) => {
  const url = new URL(input);
  if (url.protocol !== "file:") return originalFetch(input, options);
  try { return new Response(await readFile(url), { status: 200 }); }
  catch (error) { if (error.code === "ENOENT") return new Response("", { status: 404 }); throw error; }
};
globalThis.self = { postMessage: message => parentPort.postMessage(message) };
await import(workerData.entry);
parentPort.on("message", data => self.onmessage({ data }));

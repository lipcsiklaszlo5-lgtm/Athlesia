// Browser-visible quantities only. No made-up CPU%, watts, temperature or OS RAM use.
export function browserMetrics(nav, perf) {
  const cores = Number.isInteger(nav.hardwareConcurrency) && nav.hardwareConcurrency > 0 ? nav.hardwareConcurrency : null;
  const ram = Number.isFinite(nav.deviceMemory) && nav.deviceMemory > 0 ? nav.deviceMemory : null;
  const heap = Number.isFinite(perf.memory?.usedJSHeapSize) ? perf.memory.usedJSHeapSize : null;
  return { logical_cores_reported: cores, device_ram_gib_estimate: ram, browser_js_heap_bytes: heap };
}

if (typeof document !== "undefined") {
  const $ = id => document.getElementById(id);
  let lastRuntime = null;
  const update = () => {
    if (document.hidden) return;
    const data = browserMetrics(navigator, performance);
    $("hw-cores").textContent = data.logical_cores_reported ?? "Unavailable";
    $("hw-ram").textContent = data.device_ram_gib_estimate === null ? "Unavailable" : `~${data.device_ram_gib_estimate} GiB`;
    $("hw-heap").textContent = data.browser_js_heap_bytes === null ? "Unavailable" : `${(data.browser_js_heap_bytes / 1048576).toFixed(1)} MiB`;
    if (lastRuntime) {
      $("hw-wasm").textContent = `${(lastRuntime.memory_bytes / 1048576).toFixed(2)} MiB`;
      $("hw-call").textContent = `${lastRuntime.call_ms.toFixed(3)} ms`;
    }
  };
  window.addEventListener("athlesia:wasm-metrics", event => {
    const detail = event.detail;
    if (!Number.isFinite(detail?.call_ms) || !Number.isFinite(detail?.memory_bytes) || detail.call_ms < 0 || detail.memory_bytes < 0) return;
    lastRuntime = detail;
    update();
  });
  document.addEventListener("visibilitychange", update);
  const timer = setInterval(update, 1500);
  window.addEventListener("pagehide", () => clearInterval(timer), { once: true });
  update();
}

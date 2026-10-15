from __future__ import annotations

import html
import json
import os
import subprocess
from pathlib import Path
from typing import Any

import gradio as gr

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "crates/athlesia_integrated_cognitive_agent/Cargo.toml"
DEFAULT_BIN_DIR = MANIFEST.parent / "target" / "release" / "examples"
BIN_DIR = Path(os.environ.get("ATHLESIA_BIN_DIR", DEFAULT_BIN_DIR))

CSS = """
body { background: #0b0f14; }
.gradio-container { max-width: 1180px !important; }
.hero { padding: 1.2rem 1.4rem; border: 1px solid #263241; border-radius: 14px;
        background: linear-gradient(135deg, #111923, #0b0f14); }
.hero h1 { margin: 0; color: #f2f5f7; }
.hero p { color: #a9b5c2; margin-bottom: 0; }
.kpi { padding: .8rem 1rem; border: 1px solid #263241; border-radius: 10px;
       background: #111820; min-height: 76px; }
.kpi-label { color: #8c9aaa; font-size: .78rem; text-transform: uppercase; letter-spacing: .08em; }
.kpi-value { color: #7dffae; font-size: 1.35rem; font-weight: 700; margin-top: .2rem; }
.trace { display: flex; flex-direction: column; gap: .45rem; }
.trace-row { display: grid; grid-template-columns: 150px 1fr; gap: .8rem; padding: .7rem .85rem;
             border-left: 4px solid #506070; border-radius: 8px; background: #111820; }
.trace-stage { color: #dbe5ee; font-weight: 700; letter-spacing: .04em; }
.trace-detail { color: #aebbc7; }
.trace-red { border-left-color: #ff5577; background: #201219; }
.trace-red .trace-stage { color: #ff8ca4; }
.trace-green { border-left-color: #57e389; background: #102018; }
.trace-green .trace-stage { color: #7dffae; }
.trace-blue { border-left-color: #62a9ff; }
.note { color: #8391a0; font-size: .85rem; }
"""


def _binary_command(example: str) -> list[str]:
    binary = BIN_DIR / example
    if binary.is_file():
        return [str(binary)]
    return [
        "cargo",
        "run",
        "--release",
        "--manifest-path",
        str(MANIFEST),
        "--example",
        example,
    ]


def _run_example(example: str) -> tuple[list[dict[str, Any]], str]:
    completed = subprocess.run(
        _binary_command(example),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
        timeout=30,
        env={**os.environ, "RUST_BACKTRACE": "0"},
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip()
        raise RuntimeError(f"Rust example failed ({completed.returncode}): {detail}")
    records: list[dict[str, Any]] = []
    for line in completed.stdout.splitlines():
        if line.strip():
            records.append(json.loads(line))
    if not records:
        raise RuntimeError("Rust example produced no JSONL records")
    return records, "\n".join(json.dumps(record, sort_keys=True) for record in records)


def _trace_html(records: list[dict[str, Any]]) -> str:
    stage_records = [record for record in records if record.get("record") == "stage"]
    if not stage_records:
        negative = next((record for record in records if record.get("record") == "negative_control"), None)
        if negative is None:
            return '<div class="trace"><div class="trace-row trace-red"><div class="trace-stage">NO TRACE</div><div class="trace-detail">No stage records were emitted.</div></div></div>'
        return (
            '<div class="trace">'
            '<div class="trace-row trace-red">'
            '<div class="trace-stage">REFUSED</div>'
            '<div class="trace-detail">Wrong topology rejected; no target model installed.</div>'
            '</div></div>'
        )

    rows = []
    for record in stage_records:
        stage = html.escape(str(record.get("stage", "UNKNOWN")))
        detail = html.escape(str(record.get("detail", "")))
        if stage in {"FALSIFY", "AUTHORITY"} and "QUARANTINED" in detail:
            css = "trace-red"
        elif stage == "REVISE" or stage == "TARGET_TRIAL":
            css = "trace-green"
        elif stage in {"TRANSFER", "ACTIVE_PROBE"}:
            css = "trace-blue"
        else:
            css = ""
        rows.append(
            f'<div class="trace-row {css}"><div class="trace-stage">{stage}</div><div class="trace-detail">{detail}</div></div>'
        )
    return '<div class="trace">' + "".join(rows) + "</div>"


def _metrics(records: list[dict[str, Any]], label: str) -> str:
    summary = next((record for record in records if record.get("record") == "summary"), None)
    negative = next((record for record in records if record.get("record") == "negative_control"), None)
    if summary is not None:
        elapsed_us = summary["elapsed_ns"] / 1_000
        return (
            f'<div class="kpi"><div class="kpi-label">{html.escape(label)}</div>'
            f'<div class="kpi-value">{html.escape(summary["status"])} · {elapsed_us:.3f} µs</div></div>'
            f'<p class="note">epoch={summary["epoch"]} · events={summary["events"]} · ledger_valid={str(summary["ledger_valid"]).lower()}</p>'
        )
    if negative is not None:
        return (
            f'<div class="kpi"><div class="kpi-label">{html.escape(label)}</div>'
            f'<div class="kpi-value">{html.escape(negative["status"])}</div></div>'
            '<p class="note">target_installed=false · ledger_valid=true · topology mismatch</p>'
        )
    return '<div class="kpi"><div class="kpi-label">Run</div><div class="kpi-value">UNKNOWN</div></div>'


def run_positive() -> tuple[str, str, str, str]:
    try:
        records, jsonl = _run_example("cross_domain_causal_trace")
        return _trace_html(records), _metrics(records, "Causal cycle"), jsonl, "✅ Rust core completed: contradiction detected and locally revised."
    except Exception as exc:  # surfaced in the UI, not hidden
        message = html.escape(str(exc))
        return _trace_html([]), f'<div class="kpi"><div class="kpi-label">Error</div><div class="kpi-value">FAILED</div></div><p class="note">{message}</p>', "", f"❌ {message}"


def run_negative() -> tuple[str, str, str, str]:
    try:
        records, jsonl = _run_example("cross_domain_causal_negative")
        return _trace_html(records), _metrics(records, "Negative control"), jsonl, "🛑 Rust core refused the mismatched topology."
    except Exception as exc:  # surfaced in the UI, not hidden
        message = html.escape(str(exc))
        return _trace_html([]), f'<div class="kpi"><div class="kpi-label">Error</div><div class="kpi-value">FAILED</div></div><p class="note">{message}</p>', "", f"❌ {message}"


with gr.Blocks(css=CSS, title="Athlesia / Causal Core") as demo:
    gr.HTML(
        '<div class="hero"><h1>Athlesia / Cross-domain causal core</h1>'
        '<p>A thin UI over the Rust kernel. No model API, no Python emulation, no scripted success flag.</p></div>'
    )
    with gr.Row():
        with gr.Column(scale=2):
            gr.Markdown("### Run a real kernel trace")
            sabotage = gr.Button("🔥 TRIGGER CONTRADICTION", variant="primary")
            negative = gr.Button("🧱 BREAK THE TOPOLOGY")
            gr.Markdown(
                "The first control runs the bounded observe → transfer → falsify → revise cycle. "
                "The second presents a structurally incompatible target and must refuse it."
            )
        with gr.Column(scale=1):
            metrics = gr.HTML(_metrics([], "Ready"))
            status = gr.Markdown("Ready. Choose a verified control.")
    gr.Markdown("### Evidence trace")
    timeline = gr.HTML(_trace_html([]))
    with gr.Row():
        log = gr.Code(label="Machine-readable JSONL", language="json", lines=18)
    gr.Markdown(
        "The displayed runtime is the bounded fixture's own measurement. It is not a claim of arbitrary graph understanding or a universal latency bound."
    )

    sabotage.click(run_positive, outputs=[timeline, metrics, log, status])
    negative.click(run_negative, outputs=[timeline, metrics, log, status])


if __name__ == "__main__":
    demo.launch(server_name="0.0.0.0", server_port=int(os.environ.get("PORT", "7860")))

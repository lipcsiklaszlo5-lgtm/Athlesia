#!/usr/bin/env python3
"""Build the Open Intelligence proposal from the frozen v4 result JSON."""

from __future__ import annotations

import json
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    KeepTogether,
    PageBreak,
    Paragraph,
    SimpleDocTemplate,
    Spacer,
    Table,
    TableStyle,
)

ROOT = Path(__file__).resolve().parents[1]
RESULT_PATH = ROOT / "artifacts/open_intelligence/result.json"
OUTPUT_PATH = ROOT / "output/pdf/Athlesia_Open_Intelligence_Proposal.pdf"

INK = colors.HexColor("#172238")
MUTED = colors.HexColor("#5A6475")
ACCENT = colors.HexColor("#007E68")
PALE = colors.HexColor("#E8F5F1")
LINE = colors.HexColor("#D6DEE8")
WHITE = colors.white


def register_fonts() -> None:
    pdfmetrics.registerFont(
        TTFont("Athlesia", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf")
    )
    pdfmetrics.registerFont(
        TTFont("AthlesiaBold", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf")
    )


def build() -> Path:
    result = json.loads(RESULT_PATH.read_text(encoding="utf-8"))
    if result.get("benchmark") != "athlesia-open-intelligence-core-v4":
        raise ValueError("Proposal requires the frozen v4 result")
    if result.get("protocol_status") != "final held-out run":
        raise ValueError("Result is not marked as the final held-out run")
    if not result.get("validation", {}).get("passed"):
        raise ValueError("Refusing to generate a proposal from a failed benchmark")

    trained = result["heldout_trained_core"]
    one_shot = result["heldout_one_example_core"]
    cold = result["heldout_cold_core"]
    random = result["heldout_random"]
    ood = result["ood_reversed_roles"]
    ood_cold = result["ood_reversed_roles_cold"]
    paired = result["paired_trained_vs_cold"]
    paired_ci = paired["paired_mean_steps_saved_95pct_bootstrap_ci"]
    ood_extra = -result["paired_ood_trained_vs_cold"]["paired_mean_steps_saved"]
    effects = result["transfer_effect"]
    protocol = result["protocol"]

    register_fonts()
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)

    document = SimpleDocTemplate(
        str(OUTPUT_PATH),
        pagesize=A4,
        leftMargin=18 * mm,
        rightMargin=18 * mm,
        topMargin=17 * mm,
        bottomMargin=18 * mm,
        title="Athlesia - Open Intelligence Research Milestone",
        author="Laszlo Lipcsik",
        subject="Interaction-learned role transfer in procedural grid worlds",
    )
    base = getSampleStyleSheet()
    styles = {
        "kicker": ParagraphStyle(
            "Kicker", fontName="AthlesiaBold", fontSize=8.5, leading=11,
            textColor=ACCENT, spaceAfter=3 * mm
        ),
        "title": ParagraphStyle(
            "MainTitle", fontName="AthlesiaBold", fontSize=27, leading=31,
            textColor=INK, spaceAfter=3 * mm
        ),
        "subtitle": ParagraphStyle(
            "Subtitle", fontName="Athlesia", fontSize=11, leading=16,
            textColor=MUTED, spaceAfter=6 * mm
        ),
        "h1": ParagraphStyle(
            "Section", fontName="AthlesiaBold", fontSize=16, leading=20,
            textColor=INK, spaceBefore=3 * mm, spaceAfter=2.5 * mm
        ),
        "h2": ParagraphStyle(
            "Subsection", fontName="AthlesiaBold", fontSize=10.5, leading=14,
            textColor=INK, spaceBefore=2.5 * mm, spaceAfter=1.2 * mm
        ),
        "body": ParagraphStyle(
            "BodyText", fontName="Athlesia", fontSize=8.8, leading=13,
            textColor=INK, spaceAfter=2 * mm
        ),
        "small": ParagraphStyle(
            "SmallText", fontName="Athlesia", fontSize=7.5, leading=10,
            textColor=MUTED
        ),
        "metric": ParagraphStyle(
            "Metric", fontName="AthlesiaBold", fontSize=17, leading=20,
            textColor=ACCENT, alignment=TA_LEFT
        ),
        "metric_label": ParagraphStyle(
            "MetricLabel", fontName="Athlesia", fontSize=7.2, leading=9.5,
            textColor=MUTED, alignment=TA_LEFT
        ),
        "cell": ParagraphStyle(
            "CellText", fontName="Athlesia", fontSize=7.8, leading=10.5,
            textColor=INK
        ),
        "cell_bold": ParagraphStyle(
            "CellBold", fontName="AthlesiaBold", fontSize=7.8, leading=10.5,
            textColor=INK
        ),
        "cell_head": ParagraphStyle(
            "CellHead", fontName="AthlesiaBold", fontSize=7.8, leading=10.5,
            textColor=WHITE
        ),
        "callout": ParagraphStyle(
            "Callout", fontName="AthlesiaBold", fontSize=10, leading=14,
            textColor=INK, leftIndent=3 * mm, rightIndent=3 * mm
        ),
    }

    def p(value: str, style: str = "body") -> Paragraph:
        return Paragraph(value, styles[style])

    def footer(canvas, doc) -> None:  # type: ignore[no-untyped-def]
        canvas.saveState()
        width, _ = A4
        canvas.setStrokeColor(LINE)
        canvas.line(18 * mm, 13 * mm, width - 18 * mm, 13 * mm)
        canvas.setFont("Athlesia", 7)
        canvas.setFillColor(MUTED)
        canvas.drawString(18 * mm, 8.5 * mm, "ATHLESIA  /  OPEN INTELLIGENCE  /  V4")
        canvas.drawRightString(width - 18 * mm, 8.5 * mm, str(doc.page))
        canvas.restoreState()

    metrics = Table(
        [
            [
                [p(f"{trained['successes']}/{trained['episodes']}", "metric"),
                 p("held-out worlds solved", "metric_label")],
                [p(f"{effects['step_reduction_vs_cold_pct']:.2f}%", "metric"),
                 p("fewer actions than cold", "metric_label")],
                [p(f"{paired['paired_mean_steps_saved']:.3f}", "metric"),
                 p("actions saved per world", "metric_label")],
                [p(str(result["external_api_calls"]), "metric"),
                 p("external API calls", "metric_label")],
            ]
        ],
        colWidths=[42.5 * mm] * 4,
    )
    metrics.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, -1), PALE),
        ("BOX", (0, 0), (-1, -1), 0.7, ACCENT),
        ("INNERGRID", (0, 0), (-1, -1), 0.35, colors.HexColor("#B8DCD2")),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("LEFTPADDING", (0, 0), (-1, -1), 3 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 2 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 3 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 3 * mm),
    ]))

    evidence = [
        [p("Condition", "cell_head"), p("Success", "cell_head"),
         p("Mean actions", "cell_head"), p("Purpose", "cell_head")],
        [p("Trained, 16 examples/family", "cell_bold"),
         p(f"{trained['successes']}/{trained['episodes']} ({trained['success_rate'] * 100:.2f}%)", "cell"),
         p(f"{trained['mean_steps']:.2f}", "cell"),
         p("Retained role evidence", "cell")],
        [p("One example/family", "cell_bold"),
         p(f"{one_shot['successes']}/{one_shot['episodes']} ({one_shot['success_rate'] * 100:.2f}%)", "cell"),
         p(f"{one_shot['mean_steps']:.2f}", "cell"),
         p("Low-data comparison", "cell")],
        [p("Cold model", "cell_bold"),
         p(f"{cold['successes']}/{cold['episodes']} ({cold['success_rate'] * 100:.2f}%)", "cell"),
         p(f"{cold['mean_steps']:.2f}", "cell"),
         p("Same priors, no retained evidence", "cell")],
        [p("Random actions", "cell_bold"),
         p(f"{random['successes']}/{random['episodes']} ({random['success_rate'] * 100:.2f}%)", "cell"),
         p(f"{random['mean_steps']:.2f}", "cell"),
         p("Unguided reference", "cell")],
    ]
    evidence_table = Table(
        evidence, colWidths=[47 * mm, 39 * mm, 27 * mm, 57 * mm], repeatRows=1
    )
    evidence_table.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, 0), INK),
        ("GRID", (0, 0), (-1, -1), 0.35, LINE),
        ("ROWBACKGROUNDS", (0, 1), (-1, -1), [WHITE, colors.HexColor("#F6F8FB")]),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("LEFTPADDING", (0, 0), (-1, -1), 2 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 1.5 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 1.6 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 1.6 * mm),
    ]))

    story = [
        Spacer(1, 5 * mm),
        p("OPEN INTELLIGENCE FOUNDATION  /  RESEARCH MILESTONE", "kicker"),
        p("Learn the role.<br/>Solve the world.", "title"),
        p(
            "A small agent learns geometric roles through interaction, then uses "
            "that evidence in unseen procedural worlds.",
            "subtitle",
        ),
        metrics,
        Spacer(1, 4 * mm),
        p("The result", "h1"),
        p(
            f"In the frozen v4 run, five separately trained models solved "
            f"{trained['successes']} of {trained['episodes']} held-out worlds "
            f"after 16 training examples per family. They averaged "
            f"{trained['mean_steps']:.2f} actions per world, versus "
            f"{cold['mean_steps']:.2f} for the same fixed priors without retained "
            f"evidence. The paired difference was "
            f"{paired['paired_mean_steps_saved']:.3f} actions "
            f"(95% stratified paired-bootstrap interval "
            f"{paired_ci[0]:.3f} to {paired_ci[1]:.3f}).",
        ),
        evidence_table,
        Spacer(1, 3 * mm),
        p(
            "The world is a 13×13 grid with opaque tile IDs, remapped actions, "
            "twelve decoys, and up to eighteen obstacles. Training, held-out, "
            "and shift-probe seeds and token ranges are disjoint.",
            "small",
        ),
        p("A result with a visible boundary", "h2"),
        p(
            f"Under reversed key/exit roles, the trained model solves "
            f"{ood['successes']}/{ood['episodes']} worlds at "
            f"{ood['mean_steps']:.2f} mean actions; cold solves "
            f"{ood_cold['successes']}/{ood_cold['episodes']} at "
            f"{ood_cold['mean_steps']:.2f}. The transferred rule causes about "
            f"{ood_extra:.2f} extra actions per world. This negative transfer "
            f"is the next research problem, not a result hidden from the proposal.",
        ),
        p(
            "Scope: synthetic grid worlds, five declared geometric features, "
            "and one independently trained model per family. This does not "
            "demonstrate unknown-family discovery, raw visual perception, "
            "general intelligence, Rust integration, or ARC-AGI performance.",
            "small",
        ),
        PageBreak(),
        p("THE RESEARCH MILESTONE", "kicker"),
        p("One learner.<br/>Five families. Safer transfer.", "title"),
        p(
            "The $10,000 request funds the unresolved step exposed by the result: "
            "a shared learner that can select and revise role models from "
            "interaction without receiving a task-family label.",
            "subtitle",
        ),
        p("Six-week plan", "h1"),
    ]

    milestone_rows = [
        [p("Period", "cell_head"), p("Work", "cell_head"), p("Evidence", "cell_head")],
        [p("Week 1", "cell_bold"), p("Freeze the v4 reference", "cell"),
         p("Clean-run reproduction, protocol, and checksums", "cell")],
        [p("Weeks 2-3", "cell_bold"), p("Build a shared learner", "cell"),
         p("No family label; interaction traces and development benchmark", "cell")],
        [p("Weeks 4-5", "cell_bold"), p("Test model selection and shift recovery", "cell"),
         p("New frozen split; paired cold, reset, and adaptive comparisons", "cell")],
        [p("Week 6", "cell_bold"), p("Release code and report", "cell"),
         p("One-command reproduction, results, and failures", "cell")],
    ]
    milestone_table = Table(
        milestone_rows, colWidths=[24 * mm, 58 * mm, 88 * mm], repeatRows=1
    )
    milestone_table.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, 0), INK),
        ("GRID", (0, 0), (-1, -1), 0.35, LINE),
        ("ROWBACKGROUNDS", (0, 1), (-1, -1), [WHITE, colors.HexColor("#F6F8FB")]),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 2 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 2 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 2 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 2 * mm),
    ]))

    budget_rows = [
        [p("Use", "cell_head"), p("Amount", "cell_head"), p("Deliverable", "cell_head")],
        [p("Researcher time", "cell_bold"), p("$7,000", "cell"),
         p("Shared learner and experiments", "cell")],
        [p("CPU evaluation and storage", "cell_bold"), p("$2,000", "cell"),
         p("Frozen splits, ablations, preserved results", "cell")],
        [p("Documentation and release", "cell_bold"), p("$1,000", "cell"),
         p("Reproduction guide and technical report", "cell")],
        [p("Total", "cell_bold"), p("$10,000", "cell_bold"),
         p("Open research release", "cell_bold")],
    ]
    budget_table = Table(budget_rows, colWidths=[55 * mm, 28 * mm, 87 * mm], repeatRows=1)
    budget_table.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, 0), INK),
        ("BACKGROUND", (0, -1), (-1, -1), PALE),
        ("GRID", (0, 0), (-1, -1), 0.35, LINE),
        ("ROWBACKGROUNDS", (0, 1), (-1, -2), [WHITE, colors.HexColor("#F6F8FB")]),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("LEFTPADDING", (0, 0), (-1, -1), 2 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 2 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 1.8 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 1.8 * mm),
    ]))

    story.extend([
        milestone_table,
        p("Predeclared evaluation gate", "h2"),
        p(
            "Keep the v4 reference unchanged. Test at least 1,000 new held-out "
            "worlds, keep family labels and hidden state out of the policy, "
            "report every family, and compare against paired cold, reset, and "
            "random baselines. A Rust port starts only after the shared learner "
            "passes; if it does not, publish the failure and revise the claim.",
        ),
        p("Proposed use of funds", "h2"),
        budget_table,
        Spacer(1, 3 * mm),
        p(
            "The current benchmark uses the Python standard library and makes "
            f"zero external API calls. Its recorded full-run evaluation time was "
            f"{result['elapsed_ms'] / 1000:.0f} seconds; hardware affects runtime.",
            "small",
        ),
        KeepTogether([
            p("Reproduce the current evidence", "h2"),
            p(
                "Showcase: bash tools/run_open_intelligence_showcase.sh<br/>"
                "Full benchmark: bash tools/run_open_intelligence_demo.sh<br/>"
                "Repository: github.com/lipcsiklaszlo5-lgtm/Athlesia",
                "body",
            ),
        ]),
        p(
            "Independent research. Open code. Results tied to a frozen evaluator.",
            "callout",
        ),
        Spacer(1, 2 * mm),
        p(
            f"Benchmark: {result['benchmark']} | "
            f"Worlds: {protocol['total_heldout_episodes']} held-out + "
            f"{protocol['ood_episodes_per_family'] * len(protocol['rule_families'])} shift probe | "
            "Prepared 24 September 2026",
            "small",
        ),
    ])

    document.build(story, onFirstPage=footer, onLaterPages=footer)
    return OUTPUT_PATH


if __name__ == "__main__":
    print(build())

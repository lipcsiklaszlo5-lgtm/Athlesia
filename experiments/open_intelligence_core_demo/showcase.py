#!/usr/bin/env python3
"""Small live transfer demonstration; the scored benchmark is separate."""

from __future__ import annotations

import statistics

from core_demo import (
    CoreAgent,
    EpisodeResult,
    ObservationActionEnvironment,
    ROLE_RULES,
    RoleSchema,
    WorldSpec,
    generate_world,
)


TRAIN_BASE = 5_400_000
HELDOUT_BASE = 5_500_000
TRAIN_PER_FAMILY = 16
HELDOUT_PER_FAMILY = 8
ACTION_BUDGET = 128


class TracedEnvironment:
    """Records positions for the after-action display, outside the agent API."""

    def __init__(self, spec: WorldSpec):
        self._environment = ObservationActionEnvironment(spec)
        self.positions = [self._environment.observe().agent]

    def observe(self):
        return self._environment.observe()

    def step(self, action: int):
        transition = self._environment.step(action)
        self.positions.append(transition.observation.agent)
        return transition


def _run(
    schema: RoleSchema,
    token_memory: dict[int, str],
    spec: WorldSpec,
    traced: bool = False,
) -> tuple[EpisodeResult, list[tuple[int, int]]]:
    environment = TracedEnvironment(spec) if traced else ObservationActionEnvironment(spec)
    result = CoreAgent(schema.clone(), dict(token_memory)).run(
        environment, budget=ACTION_BUDGET
    )
    positions = environment.positions if traced else []
    return result, positions


def _draw(spec: WorldSpec, positions: list[tuple[int, int]]) -> list[str]:
    visited = set(positions)
    decoys = set(spec.decoys)
    rows: list[str] = []
    for y in range(spec.height):
        row = []
        for x in range(spec.width):
            position = (x, y)
            if position == spec.start:
                cell = "S"
            elif position == spec.key:
                cell = "K"
            elif position == spec.exit:
                cell = "E"
            elif position in decoys:
                cell = "?"
            elif position in spec.walls:
                cell = "#"
            elif position in visited:
                cell = "·"
            else:
                cell = " "
            row.append(cell)
        rows.append("".join(row))
    return rows


def run_showcase() -> int:
    print("ATHLESIA — LIVE TRANSFER SHOWCASE")
    print("One learned role model per task family; eight fresh worlds per family.")
    print("These 40 showcase worlds are separate from the scored 640-world run.\n")

    totals: dict[str, list[EpisodeResult]] = {"learned": [], "cold": []}
    trace_data: tuple[WorldSpec, list[tuple[int, int]], list[tuple[int, int]], EpisodeResult, EpisodeResult] | None = None

    for rule_id, rule_name in enumerate(ROLE_RULES):
        learner = CoreAgent(RoleSchema(), {})
        seen_train_tokens: set[int] = set()
        for exposure in range(1, TRAIN_PER_FAMILY + 1):
            spec = generate_world(
                TRAIN_BASE + rule_id * 10_000 + exposure,
                rule_id=rule_id,
            )
            seen_train_tokens.update(spec.surface_tokens)
            learner.run(ObservationActionEnvironment(spec), budget=ACTION_BUDGET)

        inferred = learner.schema.selected_feature()
        family_learned: list[EpisodeResult] = []
        family_cold: list[EpisodeResult] = []
        family_wins = 0
        for index in range(HELDOUT_PER_FAMILY):
            spec = generate_world(
                HELDOUT_BASE + rule_id * 10_000 + index,
                rule_id=rule_id,
            )
            if not seen_train_tokens.isdisjoint(spec.surface_tokens):
                raise RuntimeError("showcase token namespaces overlap")

            is_trace = rule_id == 0 and index == 0
            trained, trained_path = _run(
                learner.schema, learner.token_memory, spec, traced=is_trace
            )
            cold, cold_path = _run(RoleSchema(), {}, spec, traced=is_trace)
            family_learned.append(trained)
            family_cold.append(cold)
            family_wins += trained.steps < cold.steps
            if is_trace:
                trace_data = (spec, trained_path, cold_path, trained, cold)

        totals["learned"].extend(family_learned)
        totals["cold"].extend(family_cold)
        learned_mean = statistics.mean(result.steps for result in family_learned)
        cold_mean = statistics.mean(result.steps for result in family_cold)
        learned_successes = sum(result.success for result in family_learned)
        cold_successes = sum(result.success for result in family_cold)
        print(
            f"{rule_name:14} inferred={inferred or 'none':14} "
            f"learned={learned_successes}/{HELDOUT_PER_FAMILY} "
            f"{learned_mean:5.1f} actions | cold={cold_successes}/{HELDOUT_PER_FAMILY} "
            f"{cold_mean:5.1f} actions | wins={family_wins}/{HELDOUT_PER_FAMILY}"
        )

    learned = totals["learned"]
    cold = totals["cold"]
    learned_successes = sum(result.success for result in learned)
    cold_successes = sum(result.success for result in cold)
    learned_mean = statistics.mean(result.steps for result in learned)
    cold_mean = statistics.mean(result.steps for result in cold)
    wins = sum(a.steps < b.steps for a, b in zip(learned, cold))
    print("\nSHOWCASE SUMMARY (illustrative; not the benchmark score)")
    print(
        f"40 unseen worlds: learned {learned_successes}/40, "
        f"{learned_mean:.2f} actions; cold {cold_successes}/40, "
        f"{cold_mean:.2f} actions; learned wins {wins}/40."
    )

    if trace_data is not None:
        spec, trained_path, cold_path, trained, cold = trace_data
        trained_map = _draw(spec, trained_path)
        cold_map = _draw(spec, cold_path)
        print("\nFIRST HELD-OUT WORLD — edge family; same map, no labels shown to either agent")
        print(f"Learned: {'SOLVED' if trained.success else 'FAILED'} in {trained.steps} actions")
        print(f"Cold:    {'SOLVED' if cold.success else 'FAILED'} in {cold.steps} actions")
        print("Legend: S start, # wall, ? unknown object, K key, E exit, · visited")
        for left, right in zip(trained_map, cold_map):
            print(f"{left}   {right}")

    return 0


if __name__ == "__main__":
    raise SystemExit(run_showcase())

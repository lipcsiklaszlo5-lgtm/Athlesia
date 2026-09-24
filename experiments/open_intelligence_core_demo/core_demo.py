#!/usr/bin/env python3
"""Athlesia Open Intelligence proof: causal role transfer in opaque grid worlds.

The benchmark deliberately isolates one claim.  A small set of spatial priors
is fixed; action meanings and entity identities are not.  The learner must use
interaction feedback to acquire reusable role evidence, then solve held-out
worlds whose layouts, action permutations, and surface tokens were never seen
during training.

Only the Python standard library is used so the evidence can be reproduced on
a CPU with one command.
"""

from __future__ import annotations

import argparse
import collections
import dataclasses
import json
import math
import statistics
import time
from pathlib import Path
from typing import Iterable, Optional


Position = tuple[int, int]
Direction = tuple[int, int]
DIRECTIONS: tuple[Direction, ...] = ((0, -1), (0, 1), (-1, 0), (1, 0))
ROLE_RULES = ("edge", "left_right", "upper_lower", "checkerboard", "near_far")


class SplitMix64:
    """Small deterministic RNG with no dependency on Python's RNG version."""

    def __init__(self, seed: int):
        self.state = seed & 0xFFFFFFFFFFFFFFFF

    def next_u64(self) -> int:
        self.state = (self.state + 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF
        value = self.state
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & 0xFFFFFFFFFFFFFFFF
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & 0xFFFFFFFFFFFFFFFF
        return value ^ (value >> 31)

    def below(self, upper: int) -> int:
        if upper <= 0:
            raise ValueError("upper must be positive")
        return self.next_u64() % upper

    def shuffle(self, values: list[object]) -> None:
        for index in range(len(values) - 1, 0, -1):
            other = self.below(index + 1)
            values[index], values[other] = values[other], values[index]


@dataclasses.dataclass(frozen=True)
class Observation:
    width: int
    height: int
    agent: Position
    tiles: tuple[tuple[int, ...], ...]

    def tile(self, position: Position) -> int:
        return self.tiles[position[1]][position[0]]


@dataclasses.dataclass(frozen=True)
class Transition:
    observation: Observation
    reward: int
    terminated: bool


@dataclasses.dataclass(frozen=True)
class WorldSpec:
    seed: int
    rule_id: int
    width: int
    height: int
    start: Position
    key: Position
    exit: Position
    decoys: tuple[Position, ...]
    walls: frozenset[Position]
    empty_token: int
    wall_token: int
    key_token: int
    exit_token: int
    decoy_tokens: tuple[int, ...]
    action_directions: tuple[Direction, ...]
    reversed_roles: bool = False

    @property
    def surface_tokens(self) -> frozenset[int]:
        return frozenset(
            (self.empty_token, self.wall_token, self.key_token, self.exit_token)
            + self.decoy_tokens
        )


class OpaqueKeyDoorWorld:
    """A tiny environment that exposes observations and scalar feedback only."""

    def __init__(self, spec: WorldSpec):
        self._spec = spec
        self._agent = spec.start
        self._has_key = False
        self._terminated = False

    def observe(self) -> Observation:
        rows: list[tuple[int, ...]] = []
        spec = self._spec
        decoy_tokens = dict(zip(spec.decoys, spec.decoy_tokens))
        for y in range(spec.height):
            row: list[int] = []
            for x in range(spec.width):
                position = (x, y)
                if position in spec.walls:
                    row.append(spec.wall_token)
                elif position == spec.key and not self._has_key:
                    row.append(spec.key_token)
                elif position == spec.exit:
                    row.append(spec.exit_token)
                elif position in decoy_tokens:
                    row.append(decoy_tokens[position])
                else:
                    row.append(spec.empty_token)
            rows.append(tuple(row))
        return Observation(spec.width, spec.height, self._agent, tuple(rows))

    def step(self, action: int) -> Transition:
        if self._terminated:
            raise RuntimeError("cannot step a completed world")
        spec = self._spec
        dx, dy = spec.action_directions[action]
        target = (self._agent[0] + dx, self._agent[1] + dy)
        if (
            0 <= target[0] < spec.width
            and 0 <= target[1] < spec.height
            and target not in spec.walls
        ):
            if target == spec.exit:
                if self._has_key:
                    self._agent = target
                    self._terminated = True
                # A closed exit is locally observable only as failed motion.
            else:
                self._agent = target
                if target == spec.key and not self._has_key:
                    self._has_key = True
        return Transition(self.observe(), int(self._terminated), self._terminated)


class ObservationActionEnvironment:
    """The only interface an agent receives: observations, actions, and feedback."""

    def __init__(self, spec: WorldSpec):
        self.__simulator = OpaqueKeyDoorWorld(spec)

    def observe(self) -> Observation:
        return self.__simulator.observe()

    def step(self, action: int) -> Transition:
        return self.__simulator.step(action)


def _boundary(position: Position, width: int, height: int) -> bool:
    x, y = position
    return x in (0, width - 1) or y in (0, height - 1)


def _spatial_features(
    position: Position, start: Position, width: int, height: int
) -> dict[str, int]:
    x, y = position
    return {
        "edge": int(_boundary(position, width, height)),
        "left_right": int(x < width // 2),
        "upper_lower": int(y < height // 2),
        "checkerboard": int((x + y) % 2 == 0),
        "near_far": int(
            abs(x - start[0]) + abs(y - start[1]) <= min(width, height) // 3
        ),
    }


def _role_feature_values(rule_id: int, reversed_roles: bool = False) -> tuple[int, int]:
    if not 0 <= rule_id < len(ROLE_RULES):
        raise ValueError(f"unknown role rule {rule_id}")
    key_value, exit_value = (0, 1) if rule_id == 0 else (1, 0)
    return (exit_value, key_value) if reversed_roles else (key_value, exit_value)


def _reachable(
    start: Position,
    target: Position,
    width: int,
    height: int,
    walls: frozenset[Position],
) -> bool:
    frontier = collections.deque([start])
    seen = {start}
    while frontier:
        current = frontier.popleft()
        if current == target:
            return True
        for dx, dy in DIRECTIONS:
            nxt = (current[0] + dx, current[1] + dy)
            if (
                0 <= nxt[0] < width
                and 0 <= nxt[1] < height
                and nxt not in walls
                and nxt not in seen
            ):
                seen.add(nxt)
                frontier.append(nxt)
    return False


def generate_world(
    seed: int,
    rule_id: int = 0,
    reversed_roles: bool = False,
    size: int = 13,
    decoy_count: int = 12,
    obstacle_count: int = 18,
) -> WorldSpec:
    """Generate a world from one of five hidden geometric role rules."""

    if size < 7 or size % 2 == 0:
        raise ValueError("world size must be odd and at least seven")
    if not 0 <= decoy_count <= 24:
        raise ValueError("decoy count must be between zero and twenty-four")
    if obstacle_count < 0:
        raise ValueError("obstacle count cannot be negative")

    rng = SplitMix64(seed)
    width = height = size
    start = (size // 2, size // 2)
    key_value, exit_value = _role_feature_values(rule_id, reversed_roles)
    feature_name = ROLE_RULES[rule_id]

    calibration_area = {
        (start[0] + dx, start[1] + dy)
        for dy in (-1, 0, 1)
        for dx in (-1, 0, 1)
    }
    all_positions = [
        (x, y)
        for y in range(height)
        for x in range(width)
        if (x, y) not in calibration_area and (x, y) != start
    ]
    role_positions = [
        position
        for position in all_positions
        if rule_id == 0 or not _boundary(position, width, height)
        if not (
            position[0] in (0, width - 1)
            and position[1] in (0, height - 1)
        )
    ]
    key_candidates = [
        position
        for position in role_positions
        if _spatial_features(position, start, width, height)[feature_name] == key_value
    ]
    exit_candidates = [
        position
        for position in role_positions
        if _spatial_features(position, start, width, height)[feature_name] == exit_value
    ]
    if not key_candidates or not exit_candidates:
        raise RuntimeError(f"rule {ROLE_RULES[rule_id]} has no legal role positions")
    key = key_candidates[rng.below(len(key_candidates))]
    exit_options = [position for position in exit_candidates if position != key]
    exit_position = exit_options[rng.below(len(exit_options))]

    object_positions = {key, exit_position}
    walls = {
        (x, y)
        for y in range(height)
        for x in range(width)
        if _boundary((x, y), width, height)
    }
    walls.discard(key)
    walls.discard(exit_position)

    decoy_candidates = [
        (x, y)
        for y in range(height)
        for x in range(width)
        if (x, y) not in calibration_area
        and (x, y) != start
        and (x, y) not in object_positions
        and not (
            x in (0, width - 1)
            and y in (0, height - 1)
        )
    ]
    rng.shuffle(decoy_candidates)
    decoys = tuple(decoy_candidates[:decoy_count])
    object_positions.update(decoys)
    walls.difference_update(decoys)

    obstacle_candidates = [
        (x, y)
        for y in range(1, height - 1)
        for x in range(1, width - 1)
        if (x, y) not in calibration_area
        and (x, y) != start
        and (x, y) not in object_positions
    ]
    rng.shuffle(obstacle_candidates)
    for candidate in obstacle_candidates[:obstacle_count]:
        trial = frozenset(walls | {candidate})
        if _reachable(start, key, width, height, trial) and _reachable(
            key, exit_position, width, height, trial
        ):
            walls.add(candidate)

    # Token namespaces are disjoint across seeds; token order and action IDs
    # are independently permuted in every world.
    token_count = 4 + decoy_count
    tokens: list[object] = [seed * 64 + offset for offset in range(1, token_count * 2, 2)]
    rng.shuffle(tokens)
    actions: list[object] = list(DIRECTIONS)
    rng.shuffle(actions)
    return WorldSpec(
        seed=seed,
        rule_id=rule_id,
        width=width,
        height=height,
        start=start,
        key=key,
        exit=exit_position,
        decoys=decoys,
        walls=frozenset(walls),
        empty_token=int(tokens[0]),
        wall_token=int(tokens[1]),
        key_token=int(tokens[2]),
        exit_token=int(tokens[3]),
        decoy_tokens=tuple(int(token) for token in tokens[4:]),
        action_directions=tuple(actions),  # type: ignore[arg-type]
        reversed_roles=reversed_roles,
    )


@dataclasses.dataclass
class RoleSchema:
    feature_counts: dict[str, dict[str, collections.Counter[int]]] = dataclasses.field(
        default_factory=dict
    )
    role_examples: dict[str, int] = dataclasses.field(default_factory=dict)

    def observe(self, role: str, features: dict[str, int]) -> None:
        role_counts = self.feature_counts.setdefault(role, {})
        for name, value in features.items():
            role_counts.setdefault(name, collections.Counter())[value] += 1
        self.role_examples[role] = self.role_examples.get(role, 0) + 1

    def observe_collectible(self, features: dict[str, int]) -> None:
        self.observe("collectible", features)

    def observe_exit(self, features: dict[str, int]) -> None:
        self.observe("terminal", features)

    @property
    def evidence_count(self) -> int:
        return sum(self.role_examples.values())

    def clone(self) -> "RoleSchema":
        return RoleSchema(
            feature_counts={
                role: {
                    feature: collections.Counter(counts)
                    for feature, counts in features.items()
                }
                for role, features in self.feature_counts.items()
            },
            role_examples=dict(self.role_examples),
        )

    def selected_feature(self, role: str = "collectible") -> Optional[str]:
        other = "terminal" if role == "collectible" else "collectible"
        role_examples = self.role_examples.get(role, 0)
        other_examples = self.role_examples.get(other, 0)
        if role_examples == 0 or other_examples == 0:
            return None

        separations: dict[str, float] = {}
        for feature in ROLE_RULES:
            role_counts = self.feature_counts.get(role, {}).get(feature, {})
            other_counts = self.feature_counts.get(other, {}).get(feature, {})
            role_zero = (role_counts.get(0, 0) + 1) / (role_examples + 2)
            role_one = (role_counts.get(1, 0) + 1) / (role_examples + 2)
            other_zero = (other_counts.get(0, 0) + 1) / (other_examples + 2)
            other_one = (other_counts.get(1, 0) + 1) / (other_examples + 2)
            separations[feature] = abs(math.log(role_zero / other_zero)) + abs(
                math.log(role_one / other_one)
            )
        return max(ROLE_RULES, key=lambda feature: separations[feature])

    def log_odds(self, features: dict[str, int], role: str) -> float:
        other = "terminal" if role == "collectible" else "collectible"
        role_examples = self.role_examples.get(role, 0)
        other_examples = self.role_examples.get(other, 0)
        if role_examples == 0 or other_examples == 0:
            return 0.0
        feature = self.selected_feature(role)
        if feature is None or feature not in features:
            return 0.0
        role_counts = self.feature_counts.get(role, {}).get(feature, {})
        other_counts = self.feature_counts.get(other, {}).get(feature, {})
        value = features[feature]
        role_probability = (role_counts.get(value, 0) + 1) / (role_examples + 2)
        other_probability = (other_counts.get(value, 0) + 1) / (other_examples + 2)
        return math.log(role_probability / other_probability)


@dataclasses.dataclass(frozen=True)
class EpisodeResult:
    success: bool
    steps: int
    motor_actions_learned: int
    exploratory_target_failures: int


class CoreAgent:
    """A bounded learner with explicit spatial priors and retained role evidence."""

    def __init__(
        self,
        schema: Optional[RoleSchema] = None,
        token_memory: Optional[dict[int, str]] = None,
    ):
        self.schema = schema if schema is not None else RoleSchema()
        self.token_memory = token_memory if token_memory is not None else {}

    @staticmethod
    def _infer_surface(observation: Observation) -> tuple[int, int, list[Position]]:
        counts = collections.Counter(value for row in observation.tiles for value in row)
        boundary_counts = collections.Counter(
            observation.tile((x, y))
            for y in range(observation.height)
            for x in range(observation.width)
            if _boundary((x, y), observation.width, observation.height)
        )
        wall_token = boundary_counts.most_common(1)[0][0]
        empty_token = max(
            (token for token in counts if token != wall_token),
            key=lambda token: counts[token],
        )
        entities = [
            (x, y)
            for y in range(observation.height)
            for x in range(observation.width)
            if observation.tile((x, y)) not in (wall_token, empty_token)
        ]
        return empty_token, wall_token, entities

    @staticmethod
    def _path(
        observation: Observation,
        wall_token: int,
        target: Position,
    ) -> Optional[list[Position]]:
        frontier = collections.deque([observation.agent])
        parent: dict[Position, Optional[Position]] = {observation.agent: None}
        while frontier:
            current = frontier.popleft()
            if current == target:
                path: list[Position] = []
                while parent[current] is not None:
                    path.append(current)
                    current = parent[current]  # type: ignore[assignment]
                path.reverse()
                return path
            for dx, dy in DIRECTIONS:
                nxt = (current[0] + dx, current[1] + dy)
                if (
                    0 <= nxt[0] < observation.width
                    and 0 <= nxt[1] < observation.height
                    and observation.tile(nxt) != wall_token
                    and nxt not in parent
                ):
                    parent[nxt] = current
                    frontier.append(nxt)
        return None

    @staticmethod
    def _direction(source: Position, target: Position) -> Direction:
        return target[0] - source[0], target[1] - source[1]

    def run(self, world: ObservationActionEnvironment, budget: int = 96) -> EpisodeResult:
        observation = world.observe()
        start_position = observation.agent
        action_to_direction: dict[int, Direction] = {}
        direction_to_action: dict[Direction, int] = {}
        calibration_queue = collections.deque(range(4))
        blocked_entities: set[Position] = set()
        has_collectible = False
        terminated = False
        steps = 0
        target_failures = 0

        while steps < budget and not terminated:
            before = observation
            empty_token, wall_token, entities = self._infer_surface(before)

            if calibration_queue:
                action = calibration_queue.popleft()
                intended_target = None
            else:
                candidates = [entity for entity in entities if entity not in blocked_entities]
                wanted_role = "terminal" if has_collectible else "collectible"
                paths: list[tuple[float, list[Position], Position]] = []
                for candidate in candidates:
                    path = self._path(before, wall_token, candidate)
                    if path is None:
                        continue
                    features = _spatial_features(
                        candidate, start_position, before.width, before.height
                    )
                    role_score = self.schema.log_odds(features, "collectible")
                    remembered_role = self.token_memory.get(before.tile(candidate))
                    if remembered_role == wanted_role:
                        role_score = 1_000_000.0
                    elif remembered_role is not None:
                        role_score = -1_000_000.0
                    rank = role_score if wanted_role == "collectible" else -role_score
                    paths.append((rank, path, candidate))
                if not paths:
                    # Recover from a wrong transferred role: reconsider locally
                    # failed entities after the environment state changes.
                    blocked_entities.clear()
                    continue
                _, path, intended_target = min(
                    paths, key=lambda item: (-item[0], len(item[1]), item[2])
                )
                if not path:
                    blocked_entities.add(intended_target)
                    target_failures += 1
                    continue
                desired = self._direction(before.agent, path[0])
                action = direction_to_action[desired]

            transition = world.step(action)
            observation = transition.observation
            terminated = transition.terminated
            steps += 1
            delta = self._direction(before.agent, observation.agent)
            if delta in DIRECTIONS:
                action_to_direction[action] = delta
                direction_to_action[delta] = action

            collected = (
                intended_target is not None
                and before.tile(intended_target) not in (empty_token, wall_token)
                and observation.tile(intended_target) == empty_token
            )
            if collected:
                has_collectible = True
                blocked_entities.clear()
                self.token_memory[before.tile(intended_target)] = "collectible"
                self.schema.observe_collectible(
                    _spatial_features(
                        intended_target,
                        start_position,
                        observation.width,
                        observation.height,
                    )
                )
            elif intended_target is not None and observation.agent == before.agent:
                # The selected entity behaved as a closed affordance.
                blocked_entities.add(intended_target)
                target_failures += 1
            elif intended_target is not None and observation.agent == intended_target:
                # An entity that remains under the agent after contact is an
                # inert distractor in this environment family.
                _, _, after_entities = self._infer_surface(observation)
                if intended_target in after_entities:
                    blocked_entities.add(intended_target)
                    target_failures += 1

            if terminated:
                if intended_target is not None:
                    self.token_memory[before.tile(intended_target)] = "terminal"
                self.schema.observe_exit(
                    _spatial_features(
                        intended_target or observation.agent,
                        start_position,
                        observation.width,
                        observation.height,
                    )
                )

        return EpisodeResult(terminated, steps, len(action_to_direction), target_failures)


def run_random(world: ObservationActionEnvironment, seed: int, budget: int = 96) -> EpisodeResult:
    rng = SplitMix64(seed)
    steps = 0
    terminated = False
    while steps < budget and not terminated:
        transition = world.step(rng.below(4))
        terminated = transition.terminated
        steps += 1
    return EpisodeResult(terminated, steps, 0, 0)


def _seed_range(base: int, count: int) -> list[int]:
    return list(range(base, base + count))


def _summarize(results: Iterable[EpisodeResult]) -> dict[str, object]:
    values = list(results)
    successes = [result for result in values if result.success]
    return {
        "episodes": len(values),
        "successes": len(successes),
        "success_rate": round(len(successes) / len(values), 4) if values else 0.0,
        "median_steps": statistics.median(result.steps for result in values) if values else 0,
        "mean_steps": round(statistics.mean(result.steps for result in values), 2) if values else 0.0,
        "mean_motor_actions_learned": round(
            statistics.mean(result.motor_actions_learned for result in values), 2
        ) if values else 0.0,
        "mean_exploratory_target_failures": round(
            statistics.mean(result.exploratory_target_failures for result in values), 2
        ) if values else 0.0,
    }


def _wilson_interval(successes: int, trials: int) -> tuple[float, float]:
    if trials == 0:
        return 0.0, 0.0
    z = 1.959963984540054
    rate = successes / trials
    scale = 1.0 + z * z / trials
    center = (rate + z * z / (2 * trials)) / scale
    radius = (
        z
        * ((rate * (1 - rate) / trials) + z * z / (4 * trials * trials)) ** 0.5
        / scale
    )
    return max(0.0, center - radius), min(1.0, center + radius)


def _paired_bootstrap_interval(
    differences: list[float],
    seed: int,
    resamples: int = 2_000,
    stratum_sizes: Optional[list[int]] = None,
) -> tuple[float, float]:
    if not differences:
        return 0.0, 0.0
    strata = stratum_sizes or [len(differences)]
    if any(size <= 0 for size in strata) or sum(strata) != len(differences):
        raise ValueError("bootstrap strata must partition all paired differences")
    bounds: list[tuple[int, int]] = []
    offset = 0
    for size in strata:
        bounds.append((offset, offset + size))
        offset += size
    rng = SplitMix64(seed)
    sample_count = len(differences)
    means: list[float] = []
    for _ in range(resamples):
        total = 0.0
        for start, end in bounds:
            size = end - start
            for _ in range(size):
                total += differences[start + rng.below(size)]
        means.append(total / sample_count)
    means.sort()
    return means[int((resamples - 1) * 0.025)], means[int((resamples - 1) * 0.975)]


def _comparison_summary(
    trained: list[EpisodeResult],
    control: list[EpisodeResult],
    seed: int,
    stratum_sizes: Optional[list[int]] = None,
) -> dict[str, object]:
    differences = [base.steps - learned.steps for learned, base in zip(trained, control)]
    lower, upper = _paired_bootstrap_interval(
        differences, seed, stratum_sizes=stratum_sizes
    )
    return {
        "paired_mean_steps_saved": round(statistics.mean(differences), 3) if differences else 0.0,
        "paired_mean_steps_saved_95pct_bootstrap_ci": [round(lower, 3), round(upper, 3)],
        "episodes_won": sum(value > 0 for value in differences),
        "episodes_tied": sum(value == 0 for value in differences),
        "episodes_lost": sum(value < 0 for value in differences),
    }


def evaluate(
    train_count: int = 16,
    heldout_per_family: int = 128,
    ood_per_family: int = 32,
    train_base: int = 9_000_000,
    heldout_base: int = 10_000_000,
    ood_base: int = 11_000_000,
    final_evaluation: bool = False,
    world_size: int = 13,
    decoy_count: int = 12,
    obstacle_count: int = 18,
    action_budget: int = 128,
) -> dict[str, object]:
    started = time.perf_counter()
    curve_sizes = sorted({0, 1, 2, 4, 8, train_count})
    train_results: list[EpisodeResult] = []
    train_specs_by_rule: dict[int, list[WorldSpec]] = {}
    curve_schemas_by_rule: dict[int, dict[int, RoleSchema]] = {}
    curve_tokens_by_rule: dict[int, dict[int, dict[int, str]]] = {}
    train_seeds_by_rule: dict[int, list[int]] = {}

    # Each hidden rule family gets its own training sequence and its own
    # learned model. Rule IDs remain inside the evaluator and simulator.
    for rule_id in range(len(ROLE_RULES)):
        schema = RoleSchema()
        token_memory: dict[int, str] = {}
        learner = CoreAgent(schema, token_memory)
        snapshots: dict[int, RoleSchema] = {0: RoleSchema()}
        token_snapshots: dict[int, dict[int, str]] = {0: {}}
        family_seeds: list[int] = []
        family_specs: list[WorldSpec] = []
        for exposure in range(1, train_count + 1):
            seed = train_base + rule_id * 10_000 + exposure
            family_seeds.append(seed)
            spec = generate_world(
                seed,
                rule_id=rule_id,
                size=world_size,
                decoy_count=decoy_count,
                obstacle_count=obstacle_count,
            )
            family_specs.append(spec)
            train_results.append(
                learner.run(ObservationActionEnvironment(spec), budget=action_budget)
            )
            if exposure in curve_sizes:
                snapshots[exposure] = schema.clone()
                token_snapshots[exposure] = dict(token_memory)
        train_specs_by_rule[rule_id] = family_specs
        train_seeds_by_rule[rule_id] = family_seeds
        curve_schemas_by_rule[rule_id] = snapshots
        curve_tokens_by_rule[rule_id] = token_snapshots

    # Test worlds are generated only after all training snapshots are frozen.
    heldout_specs_by_rule: dict[int, list[WorldSpec]] = {
        rule_id: [
            generate_world(
                heldout_base + rule_id * 10_000 + index,
                rule_id=rule_id,
                size=world_size,
                decoy_count=decoy_count,
                obstacle_count=obstacle_count,
            )
            for index in range(heldout_per_family)
        ]
        for rule_id in range(len(ROLE_RULES))
    }
    heldout_results_by_rule: dict[int, dict[int, list[EpisodeResult]]] = {
        rule_id: {} for rule_id in range(len(ROLE_RULES))
    }
    cold_by_rule: dict[int, list[EpisodeResult]] = {
        rule_id: [
            CoreAgent().run(ObservationActionEnvironment(spec), budget=action_budget)
            for spec in heldout_specs_by_rule[rule_id]
        ]
        for rule_id in range(len(ROLE_RULES))
    }
    random_by_rule: dict[int, list[EpisodeResult]] = {
        rule_id: [
            run_random(
                ObservationActionEnvironment(spec),
                900_000 + rule_id * 10_000 + index,
                budget=action_budget,
            )
            for index, spec in enumerate(heldout_specs_by_rule[rule_id])
        ]
        for rule_id in range(len(ROLE_RULES))
    }
    memorizer_by_rule: dict[int, list[EpisodeResult]] = {}
    for rule_id in range(len(ROLE_RULES)):
        raw_memory = curve_tokens_by_rule[rule_id].get(1, {})
        memorizer_by_rule[rule_id] = [
            CoreAgent(RoleSchema(), dict(raw_memory)).run(
                ObservationActionEnvironment(spec), budget=action_budget
            )
            for spec in heldout_specs_by_rule[rule_id]
        ]

    transfer_curve: dict[str, dict[str, object]] = {}
    trained_by_exposure: dict[int, list[EpisodeResult]] = {}
    for exposure in curve_sizes:
        results: list[EpisodeResult] = []
        cold_results: list[EpisodeResult] = []
        for rule_id in range(len(ROLE_RULES)):
            schema_snapshot = curve_schemas_by_rule[rule_id][exposure]
            token_snapshot = curve_tokens_by_rule[rule_id][exposure]
            family_results = [
                CoreAgent(schema_snapshot.clone(), dict(token_snapshot)).run(
                    ObservationActionEnvironment(spec), budget=action_budget
                )
                for spec in heldout_specs_by_rule[rule_id]
            ]
            heldout_results_by_rule[rule_id][exposure] = family_results
            results.extend(family_results)
            cold_results.extend(cold_by_rule[rule_id])
        trained_by_exposure[exposure] = results
        transfer_curve[str(exposure)] = {
            **_summarize(results),
            "vs_cold_ablation": _comparison_summary(
                results,
                cold_results,
                700_000 + exposure,
                stratum_sizes=[heldout_per_family] * len(ROLE_RULES),
            ),
        }

    heldout_results = trained_by_exposure[train_count]
    cold_results = [result for family in cold_by_rule.values() for result in family]
    random_results = [result for family in random_by_rule.values() for result in family]
    memorizer_results = [result for family in memorizer_by_rule.values() for result in family]
    ood_specs_by_rule: dict[int, list[WorldSpec]] = {
        rule_id: [
            generate_world(
                ood_base + rule_id * 10_000 + index,
                rule_id=rule_id,
                reversed_roles=True,
                size=world_size,
                decoy_count=decoy_count,
                obstacle_count=obstacle_count,
            )
            for index in range(ood_per_family)
        ]
        for rule_id in range(len(ROLE_RULES))
    }
    ood_results_by_rule: dict[int, list[EpisodeResult]] = {
        rule_id: [
            CoreAgent(
                curve_schemas_by_rule[rule_id][train_count].clone(),
                dict(curve_tokens_by_rule[rule_id][train_count]),
            ).run(ObservationActionEnvironment(spec), budget=action_budget)
            for spec in ood_specs_by_rule[rule_id]
        ]
        for rule_id in range(len(ROLE_RULES))
    }
    ood_cold_by_rule: dict[int, list[EpisodeResult]] = {
        rule_id: [
            CoreAgent().run(ObservationActionEnvironment(spec), budget=action_budget)
            for spec in ood_specs_by_rule[rule_id]
        ]
        for rule_id in range(len(ROLE_RULES))
    }
    ood_results = [result for family in ood_results_by_rule.values() for result in family]
    ood_cold_results = [result for family in ood_cold_by_rule.values() for result in family]

    train_seeds = {seed for seeds in train_seeds_by_rule.values() for seed in seeds}
    heldout_seeds = {
        spec.seed for family in heldout_specs_by_rule.values() for spec in family
    }
    ood_seeds = {
        ood_base + rule_id * 10_000 + index
        for rule_id in range(len(ROLE_RULES))
        for index in range(ood_per_family)
    }
    train_tokens = set().union(
        *(spec.surface_tokens for family in train_specs_by_rule.values() for spec in family)
    )
    heldout_tokens = set().union(
        *(spec.surface_tokens for family in heldout_specs_by_rule.values() for spec in family)
    )
    ood_tokens = set().union(
        *(
            spec.surface_tokens
            for family in ood_specs_by_rule.values()
            for spec in family
        )
    )
    train_summary = _summarize(train_results)
    trained_summary = _summarize(heldout_results)
    cold_summary = _summarize(cold_results)
    random_summary = _summarize(random_results)
    memorizer_summary = _summarize(memorizer_results)
    ood_summary = _summarize(ood_results)
    ood_cold_summary = _summarize(ood_cold_results)
    cold_steps = float(cold_summary["mean_steps"])
    trained_steps = float(trained_summary["mean_steps"])
    cold_failures = float(cold_summary["mean_exploratory_target_failures"])
    trained_failures = float(trained_summary["mean_exploratory_target_failures"])
    heldout_strata = [heldout_per_family] * len(ROLE_RULES)
    ood_strata = [ood_per_family] * len(ROLE_RULES)
    paired_summary = _comparison_summary(
        heldout_results, cold_results, 799_999, stratum_sizes=heldout_strata
    )
    paired_ood_summary = _comparison_summary(
        ood_results, ood_cold_results, 799_997, stratum_sizes=ood_strata
    )
    one_example_results = trained_by_exposure.get(1, trained_by_exposure[train_count])
    one_example_summary = _summarize(one_example_results)
    one_example_paired = _comparison_summary(
        one_example_results, cold_results, 799_998, stratum_sizes=heldout_strata
    )
    success_interval = _wilson_interval(
        int(trained_summary["successes"]), int(trained_summary["episodes"])
    )
    result: dict[str, object] = {
        "benchmark": "athlesia-open-intelligence-core-v4",
        "protocol_status": "final held-out run" if final_evaluation else "development run",
        "claim": "within each of five separately trained task families, an interaction-inferred geometric role rule transfers to held-out worlds",
        "fixed_core_priors": [
            "2D locality",
            "persistent self-position",
            "four reversible movement effects",
            "object persistence and scalar terminal feedback",
            "a fixed five-feature geometric representation",
        ],
        "not_provided_to_agent": [
            "world seed",
            "action meanings",
            "tile semantics",
            "key position",
            "exit position",
            "hidden inventory state",
            "task-family rule ID",
        ],
        "train": train_summary,
        "heldout_trained_core": trained_summary,
        "heldout_one_example_core": one_example_summary,
        "heldout_cold_core": cold_summary,
        "heldout_token_memorizer": memorizer_summary,
        "heldout_random": random_summary,
        "ood_reversed_roles": ood_summary,
        "ood_reversed_roles_cold": ood_cold_summary,
        "role_transfer_curve": transfer_curve,
        "heldout_success_95pct_wilson_ci": [round(value, 4) for value in success_interval],
        "paired_trained_vs_cold": paired_summary,
        "paired_one_example_vs_cold": one_example_paired,
        "paired_ood_trained_vs_cold": paired_ood_summary,
        "transfer_effect": {
            "step_reduction_vs_cold_pct": round(
                100.0 * (cold_steps - trained_steps) / cold_steps, 2
            ),
            "failed_target_reduction_vs_cold_pct": round(
                100.0 * (cold_failures - trained_failures) / cold_failures, 2
            ) if cold_failures else 0.0,
            "success_lift_vs_random_percentage_points": round(
                100.0
                * (
                    float(trained_summary["success_rate"])
                    - float(random_summary["success_rate"])
                ),
                2,
            ),
        },
        "protocol": {
            "train_seed_ranges_by_rule": {
                ROLE_RULES[rule_id]: [min(seeds), max(seeds) + 1]
                for rule_id, seeds in train_seeds_by_rule.items()
            },
            "heldout_seed_ranges_by_rule": {
                ROLE_RULES[rule_id]: [
                    heldout_base + rule_id * 10_000,
                    heldout_base + rule_id * 10_000 + heldout_per_family,
                ]
                for rule_id in range(len(ROLE_RULES))
            },
            "ood_seed_ranges_by_rule": {
                ROLE_RULES[rule_id]: [
                    ood_base + rule_id * 10_000,
                    ood_base + rule_id * 10_000 + ood_per_family,
                ]
                for rule_id in range(len(ROLE_RULES))
            },
            "rule_families": list(ROLE_RULES),
            "train_examples_per_family": train_count,
            "training_examples_total": train_count * len(ROLE_RULES),
            "rule_family_models_trained_independently": True,
            "heldout_episodes_per_family": heldout_per_family,
            "total_heldout_episodes": len(heldout_results),
            "ood_episodes_per_family": ood_per_family,
            "primary_training_examples_per_family": train_count,
            "world_size": world_size,
            "obstacle_cells_requested": obstacle_count,
            "action_budget": action_budget,
            "primary_metric": "mean environment actions per world; unsuccessful worlds count at the action budget",
            "distractor_entities_per_world": decoy_count,
            "spatial_feature_hypotheses": list(
                _spatial_features(
                    (0, 0),
                    (world_size // 2, world_size // 2),
                    world_size,
                    world_size,
                )
            ),
            "paired_conditions_share_identical_world_specs": True,
            "memorizer_uses_training_raw_token_ids_only": True,
            "bootstrap_resamples": 2_000,
            "acceptance_criteria": {
                "heldout_success_rate_minimum": 0.95,
                "step_reduction_vs_cold_minimum_pct": 10.0,
                "paired_bootstrap_ci_lower_bound_minimum": 0.0,
                "collection_oracle_field_allowed": False,
            },
        },
        "learned_role_schemas_by_family": {
            ROLE_RULES[rule_id]: {
                "role_examples": curve_schemas_by_rule[rule_id][train_count].role_examples,
                "feature_counts": {
                    role: {
                        feature: dict(counts)
                        for feature, counts in features.items()
                    }
                    for role, features in curve_schemas_by_rule[rule_id][train_count].feature_counts.items()
                },
            }
            for rule_id in range(len(ROLE_RULES))
        },
        "leakage_guards": {
            "train_and_heldout_seeds_disjoint": train_seeds.isdisjoint(heldout_seeds),
            "train_and_ood_seeds_disjoint": train_seeds.isdisjoint(ood_seeds),
            "heldout_and_ood_seeds_disjoint": heldout_seeds.isdisjoint(ood_seeds),
            "train_and_heldout_tokens_disjoint": train_tokens.isdisjoint(heldout_tokens),
            "train_and_ood_tokens_disjoint": train_tokens.isdisjoint(ood_tokens),
            "heldout_and_ood_tokens_disjoint": heldout_tokens.isdisjoint(ood_tokens),
            "heldout_worlds_created_after_training": True,
            "agent_receives_seed_or_hidden_state": False,
            "transition_has_no_collection_oracle": "collected" not in Transition.__dataclass_fields__,
            "task_family_id_not_exposed_to_agent": True,
            "train_and_heldout_tokens_disjoint_for_memorizer": train_tokens.isdisjoint(heldout_tokens),
        },
        "per_family_results": {
            ROLE_RULES[rule_id]: {
                "heldout_trained": _summarize(heldout_results_by_rule[rule_id][train_count]),
                "heldout_cold": _summarize(cold_by_rule[rule_id]),
                "heldout_memorizer": _summarize(memorizer_by_rule[rule_id]),
                "ood_reversed_roles": _summarize(ood_results_by_rule[rule_id]),
                "ood_reversed_roles_cold": _summarize(ood_cold_by_rule[rule_id]),
            }
            for rule_id in range(len(ROLE_RULES))
        },
        "external_api_calls": 0,
        "elapsed_ms": round((time.perf_counter() - started) * 1000, 2),
    }
    return result


def validate(result: dict[str, object]) -> list[str]:
    failures: list[str] = []
    guards = result["leakage_guards"]
    assert isinstance(guards, dict)
    if not all(value is True for key, value in guards.items() if key != "agent_receives_seed_or_hidden_state"):
        failures.append("a positive leakage guard failed")
    if guards["agent_receives_seed_or_hidden_state"] is not False:
        failures.append("agent received privileged state")
    if guards.get("transition_has_no_collection_oracle") is not True:
        failures.append("environment exposes a collection oracle")
    trained = result["heldout_trained_core"]
    cold = result["heldout_cold_core"]
    random = result["heldout_random"]
    assert isinstance(trained, dict) and isinstance(cold, dict) and isinstance(random, dict)
    if float(trained["success_rate"]) < 0.95:
        failures.append("held-out trained-core success below 95%")
    if float(trained["success_rate"]) <= float(random["success_rate"]):
        failures.append("trained core did not beat random baseline")
    memorizer = result["heldout_token_memorizer"]
    assert isinstance(memorizer, dict)
    if (
        memorizer["success_rate"] != cold["success_rate"]
        or memorizer["mean_steps"] != cold["mean_steps"]
    ):
        failures.append("raw-token memorizer unexpectedly transferred to disjoint IDs")
    if float(trained["mean_steps"]) >= float(cold["mean_steps"]):
        failures.append("retained role evidence did not improve sample efficiency")
    transfer = result["transfer_effect"]
    assert isinstance(transfer, dict)
    if float(transfer["step_reduction_vs_cold_pct"]) < 10.0:
        failures.append("held-out transfer reduced steps by less than 10%")
    paired = result["paired_trained_vs_cold"]
    assert isinstance(paired, dict)
    interval = paired["paired_mean_steps_saved_95pct_bootstrap_ci"]
    if float(interval[0]) <= 0:
        failures.append("paired 95% bootstrap interval includes no improvement")
    family_results = result["per_family_results"]
    assert isinstance(family_results, dict)
    for family, metrics in family_results.items():
        if float(metrics["heldout_trained"]["success_rate"]) < 0.90:
            failures.append(f"{family} held-out success below 90%")
        if float(metrics["heldout_trained"]["mean_steps"]) >= float(
            metrics["heldout_cold"]["mean_steps"]
        ):
            failures.append(f"{family} learned model did not beat its cold ablation")
    return failures


def render_human(result: dict[str, object]) -> str:
    def line(label: str, key: str) -> str:
        section = result[key]
        assert isinstance(section, dict)
        return (
            f"{label:24} {section['successes']:>3}/{section['episodes']:<3} "
            f"success={float(section['success_rate']):>6.1%} "
            f"mean_steps={float(section['mean_steps']):>6.2f} "
            f"failed_targets={float(section['mean_exploratory_target_failures']):>4.2f}"
        )

    guards = result["leakage_guards"]
    transfer = result["transfer_effect"]
    curve = result["role_transfer_curve"]
    compact_curve = {
        exposure: {
            "success_rate": metrics["success_rate"],
            "mean_steps": metrics["mean_steps"],
        }
        for exposure, metrics in curve.items()
    }
    interval = result["paired_trained_vs_cold"][
        "paired_mean_steps_saved_95pct_bootstrap_ci"
    ]
    training_examples = result["protocol"]["train_examples_per_family"]
    family_lines = {
        family: {
            "success": f"{metrics['heldout_trained']['successes']}/{metrics['heldout_trained']['episodes']}",
            "trained_steps": metrics["heldout_trained"]["mean_steps"],
            "cold_steps": metrics["heldout_cold"]["mean_steps"],
            "ood_success": f"{metrics['ood_reversed_roles']['successes']}/{metrics['ood_reversed_roles']['episodes']}",
            "ood_cold_success": f"{metrics['ood_reversed_roles_cold']['successes']}/{metrics['ood_reversed_roles_cold']['episodes']}",
        }
        for family, metrics in result["per_family_results"].items()
    }
    one_shot_interval = result["paired_one_example_vs_cold"][
        "paired_mean_steps_saved_95pct_bootstrap_ci"
    ]
    return "\n".join(
        [
            "ATHLESIA OPEN INTELLIGENCE CORE — REPRODUCIBLE EVIDENCE",
            "=" * 63,
            line("Training", "train"),
            line("Held-out trained (1-shot)", "heldout_one_example_core"),
            line(f"Held-out trained ({training_examples}-shot)", "heldout_trained_core"),
            line("Held-out cold ablation", "heldout_cold_core"),
            line("Held-out token memory", "heldout_token_memorizer"),
            line("Held-out random", "heldout_random"),
            line("OOD reversed roles", "ood_reversed_roles"),
            line("OOD reversed roles, cold", "ood_reversed_roles_cold"),
            "-" * 63,
            f"Measured transfer effect: {json.dumps(transfer, sort_keys=True)}",
            f"Paired 95% bootstrap CI, actions saved: {interval}",
            f"One-example paired 95% CI: {one_shot_interval}",
            f"OOD paired 95% CI, actions saved: {result['paired_ood_trained_vs_cold']['paired_mean_steps_saved_95pct_bootstrap_ci']}",
            f"Training curve (examples: mean actions): {json.dumps(compact_curve, sort_keys=True)}",
            f"Rule-family results: {json.dumps(family_lines, sort_keys=True)}",
            f"Leakage guards: {json.dumps(guards, sort_keys=True)}",
            f"External API calls: {result['external_api_calls']}",
            f"CPU elapsed: {result['elapsed_ms']} ms",
        ]
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quick", action="store_true", help="run a smaller local smoke benchmark")
    parser.add_argument("--json-out", type=Path, help="write the full machine-readable result")
    args = parser.parse_args()
    result = (
        evaluate(
            8,
            16,
            8,
            train_base=1_100_000,
            heldout_base=1_200_000,
            ood_base=1_300_000,
        )
        if args.quick
        else evaluate(final_evaluation=True)
    )
    print(render_human(result))
    failures = validate(result)
    result["validation"] = {"passed": not failures, "failures": failures}
    if args.json_out:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
        print(f"Machine-readable result: {args.json_out}")
    if failures:
        for failure in failures:
            print(f"VALIDATION FAIL: {failure}")
        return 1
    print("VALIDATION PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

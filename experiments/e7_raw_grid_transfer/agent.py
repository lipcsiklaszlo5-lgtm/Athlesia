"""E7 visual-grid dynamics learner, intentionally not an ARC-AGI-3 agent.

Input is a raw 2-D array of opaque colour integers, opaque action IDs, and
an externally declared target coordinate. It learns action translations from
real observed before/after pairs, plus an obstacle colour from a failed action.
No simulator import, hidden coordinates, palette metadata or action labels.
"""
from __future__ import annotations

from collections import Counter, deque
from dataclasses import dataclass
from typing import Protocol

Frame = tuple[tuple[int, ...], ...]
Position = tuple[int, int]


class CannotIdentify(RuntimeError):
    pass


@dataclass(frozen=True)
class Demonstration:
    before: Frame
    action: str
    after: Frame


class PublicPort(Protocol):
    def observe(self) -> Frame: ...
    def send_action(self, action: str) -> Frame: ...


def validate(frame: Frame) -> tuple[int, int]:
    if not isinstance(frame, tuple) or len(frame) < 3:
        raise CannotIdentify('Invalid grid height')
    if any(not isinstance(row, tuple) for row in frame):
        raise CannotIdentify('Invalid grid row type')
    widths = {len(row) for row in frame}
    if len(widths) != 1 or next(iter(widths), 0) < 3:
        raise CannotIdentify('Invalid grid width')
    w = next(iter(widths))
    if any(type(v) is not int or not 0 <= v <= 255 for row in frame for v in row):
        raise CannotIdentify('Invalid grid pixel')
    return len(frame), w


def positions(frame: Frame):
    for r, row in enumerate(frame):
        for c, value in enumerate(row):
            yield (r, c), value


def find_one(frame: Frame, colour: int) -> Position:
    matches = [pos for pos, v in positions(frame) if v == colour]
    if len(matches) != 1:
        raise CannotIdentify('Moving colour must occur exactly once')
    return matches[0]


class Agent:
    def __init__(self) -> None:
        self.actor_colour: int | None = None
        self.floor_colour: int | None = None
        self.obstacle_colours: frozenset[int] = frozenset()
        self.action_delta: dict[str, Position] = {}

    def fit(self, traces: tuple[Demonstration, ...]) -> None:
        if self.actor_colour is not None or not 5 <= len(traces) <= 32:
            raise CannotIdentify('A fresh learner and >=5 demonstrations required')
        moved: dict[str, Position] = {}
        actor: int | None = None
        floor: int | None = None
        blocked: list[Demonstration] = []
        for trace in traces:
            h, w = validate(trace.before)
            if validate(trace.after) != (h, w):
                raise CannotIdentify('Dimension change')
            if not trace.action or not isinstance(trace.action, str):
                raise CannotIdentify('Invalid action ID')
            change = [(r, c) for r in range(h) for c in range(w)
                      if trace.before[r][c] != trace.after[r][c]]
            if not change:
                blocked.append(trace)
                continue
            if len(change) != 2:
                raise CannotIdentify('Outside one-moving-pixel model')
            most_common, count = Counter(v for _, v in positions(trace.before)).most_common(1)[0]
            if count <= h * w // 2:
                raise CannotIdentify('No identifiable dominant background')
            old_places = [p for p in change
                          if trace.before[p[0]][p[1]] != most_common
                          and trace.after[p[0]][p[1]] == most_common]
            if len(old_places) != 1:
                raise CannotIdentify('Cannot ground moving pixel')
            old = old_places[0]
            new = next(p for p in change if p != old)
            old_colour = trace.before[old[0]][old[1]]
            if (trace.before[new[0]][new[1]] != most_common
                    or trace.after[new[0]][new[1]] != old_colour):
                raise CannotIdentify('Non-translational visual transition')
            delta = (new[0] - old[0], new[1] - old[1])
            if abs(delta[0]) + abs(delta[1]) != 1:
                raise CannotIdentify('Nonlocal movement outside grammar')
            if actor is not None and (actor != old_colour or floor != most_common):
                raise CannotIdentify('Inconsistent visual roles')
            if trace.action in moved and moved[trace.action] != delta:
                raise CannotIdentify('Nonstationary action semantics')
            actor, floor = old_colour, most_common
            moved[trace.action] = delta
        if actor is None or floor is None:
            raise CannotIdentify('No positive movement witness')
        if len(moved) != 4 or len(set(moved.values())) != 4:
            raise CannotIdentify('Four distinct cardinal actions required')
        obstacles: set[int] = set()
        for sample in blocked:
            if sample.action not in moved:
                raise CannotIdentify('No-op action not previously identified')
            old = find_one(sample.before, actor)
            dr, dc = moved[sample.action]
            destination = (old[0] + dr, old[1] + dc)
            h, w = validate(sample.before)
            if not (0 <= destination[0] < h and 0 <= destination[1] < w):
                raise CannotIdentify('Boundary vs obstacle cannot be distinguished')
            colour = sample.before[destination[0]][destination[1]]
            if colour in (actor, floor):
                raise CannotIdentify('No-op cannot identify obstacle')
            obstacles.add(colour)
        if not obstacles:
            raise CannotIdentify('No obstacle-colour grounding witness')
        if len(obstacles) != 1:
            raise CannotIdentify('Ambiguous obstacle model')
        self.actor_colour = actor
        self.floor_colour = floor
        self.action_delta = moved
        self.obstacle_colours = frozenset(obstacles)

    def plan(self, frame: Frame, target: Position) -> tuple[str, ...]:
        h, w = validate(frame)
        if self.actor_colour is None or self.floor_colour is None or not self.action_delta:
            raise CannotIdentify('Visual dynamics not learned')
        if (not isinstance(target, tuple) or len(target) != 2
                or any(type(x) is not int for x in target)
                or not 0 <= target[0] < h or not 0 <= target[1] < w):
            raise CannotIdentify('Invalid externally declared target')
        start = find_one(frame, self.actor_colour)
        known = {self.actor_colour, self.floor_colour, *self.obstacle_colours}
        for (r, c), col in positions(frame):
            if (r, c) != target and col not in known:
                raise CannotIdentify('Unclassified visual pixel; refuse')
        if frame[target[0]][target[1]] in self.obstacle_colours:
            raise CannotIdentify('Target is a classified obstacle')
        queue = deque([start])
        parent: dict[Position, tuple[Position, str] | None] = {start: None}
        while queue:
            curr = queue.popleft()
            if curr == target:
                output: list[str] = []
                while parent[curr] is not None:
                    step = parent[curr]
                    assert step is not None
                    previous, action = step
                    output.append(action)
                    curr = previous
                return tuple(reversed(output))
            for action, (dr, dc) in sorted(self.action_delta.items()):
                nxt = (curr[0] + dr, curr[1] + dc)
                if (0 <= nxt[0] < h and 0 <= nxt[1] < w
                        and nxt not in parent and (nxt == target or
                        frame[nxt[0]][nxt[1]] == self.floor_colour)):
                    parent[nxt] = (curr, action)
                    queue.append(nxt)
        raise CannotIdentify('No certified path through known floor pixels')

    def execute(self, port: PublicPort, target: Position, *, max_actions: int = 60) -> dict:
        before = port.observe()
        path = self.plan(before, target)
        if len(path) > max_actions:
            raise CannotIdentify('Action budget exceeded before execution')
        for token in path:
            old = find_one(before, self.actor_colour)  # type: ignore[arg-type]
            dr, dc = self.action_delta[token]
            expected = (old[0] + dr, old[1] + dc)
            after = port.send_action(token)
            if validate(after) != validate(before):
                raise CannotIdentify('Unexpected grid size after action')
            actual = find_one(after, self.actor_colour)  # type: ignore[arg-type]
            if actual != expected:
                raise CannotIdentify('Visual model falsified during execution')
            before = after
        reached = find_one(before, self.actor_colour) == target  # type: ignore[arg-type]
        return {'reported_reached': reached, 'actions': len(path)}

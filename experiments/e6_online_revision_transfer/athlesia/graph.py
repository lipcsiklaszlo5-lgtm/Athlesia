"""Typed exact graph isomorphism: joint color refinement + bounded CSP.

Refinement is only a pruning tool. Every returned mapping is edge-verified.
Symmetries are refused rather than broken using opaque token names.
"""
from __future__ import annotations

from collections import Counter, deque
from dataclasses import dataclass
from enum import Enum

from .protocol import Edge, Scene


class MatchStatus(str, Enum):
    UNIQUE = "unique"
    AMBIGUOUS = "ambiguous"
    NO_MATCH = "no_match"
    BUDGET_EXCEEDED = "budget_exceeded"


@dataclass(frozen=True, slots=True)
class Match:
    status: MatchStatus
    pairs: tuple[tuple[str, str], ...]
    search_states: int

    def unique(self) -> dict[str, str]:
        if self.status != MatchStatus.UNIQUE:
            raise ValueError(f"Mapping is not identifiable: {self.status.value}")
        return dict(self.pairs)


def distances(scene: Scene, source: str) -> dict[str, int]:
    out = {source: 0}
    queue = deque([source])
    while queue:
        n = queue.popleft()
        for e in scene.edges:
            other = e.target if e.source == n else e.source if e.target == n else None
            if other is not None and other not in out:
                out[other] = out[n] + 1
                queue.append(other)
    return out


def isomorphism(left: Scene, right: Scene, budget: int = 10_000) -> Match:
    if len(left.nodes) != len(right.nodes) or len(left.edges) != len(right.edges):
        return Match(MatchStatus.NO_MATCH, (), 0)
    scenes = (left, right)
    kinds = sorted({n.kind.value for s in scenes for n in s.nodes})
    colors = [{n.id: kinds.index(n.kind.value) for n in s.nodes} for s in scenes]
    # Joint interning ensures colors are comparable across the two graphs.
    for _ in range(len(left.nodes) + 1):
        signatures: list[dict[str, tuple[object, ...]]] = []
        for scene, color in zip(scenes, colors):
            signatures.append({n.id: (
                color[n.id],
                tuple(sorted((e.relation.value, color[e.target]) for e in scene.edges if e.source == n.id)),
                tuple(sorted((e.relation.value, color[e.source]) for e in scene.edges if e.target == n.id)),
            ) for n in scene.nodes})
        vocabulary = sorted(set(signatures[0].values()) | set(signatures[1].values()), key=repr)
        ids = {sig: i for i, sig in enumerate(vocabulary)}
        new = [{n: ids[sig] for n, sig in ss.items()} for ss in signatures]
        if Counter(new[0].values()) != Counter(new[1].values()):
            return Match(MatchStatus.NO_MATCH, (), 0)
        # A non-splitting refinement has reached a stable partition.
        stable = len(set(new[0].values())) == len(set(colors[0].values()))
        colors = new
        if stable:
            break
    candidates = {u: tuple(v for v, c in colors[1].items() if c == colors[0][u]) for u in colors[0]}
    target_edges = set(right.edges)
    source_edges = set(left.edges)
    solutions: list[dict[str, str]] = []
    used: set[str] = set()
    mapping: dict[str, str] = {}
    states = 0
    exceeded = False

    def visit() -> None:
        nonlocal states, exceeded
        if len(solutions) >= 2 or exceeded:
            return
        states += 1
        if states > budget:
            exceeded = True
            return
        if len(mapping) == len(left.nodes):
            mapped = {Edge(mapping[e.source], e.relation, mapping[e.target]) for e in left.edges}
            if mapped == target_edges:
                solutions.append(mapping.copy())
            return
        remaining = [u for u in candidates if u not in mapping]
        u = min(remaining, key=lambda x: (sum(v not in used for v in candidates[x]), colors[0][x]))
        for v in candidates[u]:
            if v in used:
                continue
            ok = True
            for a, b in mapping.items():
                for relation in {e.relation for e in (*left.edges, *right.edges)}:
                    if ((Edge(u, relation, a) in source_edges) != (Edge(v, relation, b) in target_edges)
                        or (Edge(a, relation, u) in source_edges) != (Edge(b, relation, v) in target_edges)):
                        ok = False
                        break
                if not ok:
                    break
            if ok:
                mapping[u] = v
                used.add(v)
                visit()
                used.remove(v)
                del mapping[u]
    visit()
    if len(solutions) >= 2:
        return Match(MatchStatus.AMBIGUOUS, (), states)
    if exceeded:
        return Match(MatchStatus.BUDGET_EXCEEDED, (), states)
    if not solutions:
        return Match(MatchStatus.NO_MATCH, (), states)
    return Match(MatchStatus.UNIQUE, tuple(sorted(solutions[0].items())), states)

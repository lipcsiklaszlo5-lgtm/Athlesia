"""Public wire contract. Contains no simulator rules, seeds, or solution labels.

The observations are instrumented symbolic snapshots, NOT raw image perception.
Edge labels define the public interface vocabulary; object/action IDs are opaque.
"""
from __future__ import annotations

import json
from dataclasses import dataclass
from enum import Enum
from typing import Protocol, TypeAlias, cast

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
MAX_BYTES = 262_144
MAX_NODES = 128
MAX_EDGES = 512
MAX_TRACES = 256
OBSERVE = "@observe"


class ContractError(ValueError):
    pass


class Kind(str, Enum):
    OBJECT = "object"
    FLUENT = "fluent"
    ACTION = "action"
    GOAL = "goal"


class Relation(str, Enum):
    AT = "at"                  # A state channel belongs to an object.
    LINK = "link"              # An externally visible physical connection.
    ACTS_ON = "acts_on"        # Public action/actuator binding, not its effect.
    SET_TRUE = "set_true"      # Trusted calibration/intervention capability.
    SET_FALSE = "set_false"
    CLEAR = "clear"            # Re-arm the observable outcome between trials.


def record(value: object, keys: set[str]) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != keys:
        raise ContractError(f"Unexpected object schema; expected {sorted(keys)}")
    if any(not isinstance(k, str) for k in value):
        raise ContractError("Non-string object key")
    return cast(dict[str, object], value)


def array(value: object, maximum: int) -> list[object]:
    if not isinstance(value, list) or len(value) > maximum:
        raise ContractError("Array type/size violation")
    return cast(list[object], value)


def token(value: object) -> str:
    if not isinstance(value, str) or not 1 <= len(value) <= 96:
        raise ContractError("Invalid token length/type")
    if any(not (ch.isascii() and (ch.isalnum() or ch in "_@:-")) for ch in value):
        raise ContractError("Invalid token alphabet")
    return value


def integer(value: object, lo: int, hi: int) -> int:
    if type(value) is not int or not lo <= value <= hi:
        raise ContractError("Integer type/range violation")
    return value


def boolean(value: object) -> bool:
    if type(value) is not bool:
        raise ContractError("Expected explicit Boolean (unknown is not false)")
    return value


@dataclass(frozen=True, slots=True)
class Node:
    id: str
    kind: Kind


@dataclass(frozen=True, slots=True, order=True)
class Edge:
    source: str
    relation: Relation
    target: str


@dataclass(frozen=True, slots=True)
class Scene:
    nodes: tuple[Node, ...]
    edges: tuple[Edge, ...]
    # Rendering-only fields. Neither induction nor graph matching uses these.
    width: int
    height: int
    layout: tuple[tuple[str, int, int], ...]

    def kind_of(self, node: str) -> Kind:
        for item in self.nodes:
            if item.id == node:
                return item.kind
        raise ContractError("Unknown node")

    def targets(self, node: str, relation: Relation) -> tuple[str, ...]:
        return tuple(e.target for e in self.edges if e.source == node and e.relation == relation)

    def to_wire(self) -> dict[str, Json]:
        return {
            "nodes": [{"id": n.id, "kind": n.kind.value} for n in self.nodes],
            "edges": [{"source": e.source, "relation": e.relation.value, "target": e.target} for e in self.edges],
            "width": self.width, "height": self.height,
            "layout": [[n, x, y] for n, x, y in self.layout],
        }

    @classmethod
    def from_wire(cls, value: object) -> Scene:
        p = record(value, {"nodes", "edges", "width", "height", "layout"})
        nodes: list[Node] = []
        for raw in array(p["nodes"], MAX_NODES):
            n = record(raw, {"id", "kind"})
            try:
                nodes.append(Node(token(n["id"]), Kind(token(n["kind"]))))
            except ValueError as ex:
                raise ContractError("Invalid node kind") from ex
        ids = {n.id for n in nodes}
        if not nodes or len(ids) != len(nodes) or OBSERVE in ids:
            raise ContractError("Empty/duplicate/reserved node ID")
        edges: list[Edge] = []
        for raw in array(p["edges"], MAX_EDGES):
            raw_edge = record(raw, {"source", "relation", "target"})
            try:
                edges.append(Edge(token(raw_edge["source"]), Relation(token(raw_edge["relation"])), token(raw_edge["target"])))
            except ValueError as ex:
                raise ContractError("Invalid edge relation") from ex
        if len(set(edges)) != len(edges) or any(e.source not in ids or e.target not in ids for e in edges):
            raise ContractError("Duplicate/dangling edge")
        width, height = integer(p["width"], 1, 100_000), integer(p["height"], 1, 100_000)
        layout: list[tuple[str, int, int]] = []
        for raw in array(p["layout"], MAX_NODES):
            xyz = array(raw, 3)
            if len(xyz) != 3:
                raise ContractError("Invalid layout entry")
            layout.append((token(xyz[0]), integer(xyz[1], 0, width - 1), integer(xyz[2], 0, height - 1)))
        if {x[0] for x in layout} != ids or len(layout) != len(ids):
            raise ContractError("Layout must cover nodes once")
        scene = cls(tuple(nodes), tuple(edges), width, height, tuple(layout))
        for e in edges:
            kinds = (scene.kind_of(e.source), scene.kind_of(e.target))
            valid = {
                Relation.AT: kinds[0] in (Kind.FLUENT, Kind.GOAL) and kinds[1] == Kind.OBJECT,
                Relation.LINK: kinds == (Kind.OBJECT, Kind.OBJECT),
                Relation.ACTS_ON: kinds == (Kind.ACTION, Kind.OBJECT),
                Relation.SET_TRUE: kinds == (Kind.ACTION, Kind.FLUENT),
                Relation.SET_FALSE: kinds == (Kind.ACTION, Kind.FLUENT),
                Relation.CLEAR: kinds == (Kind.ACTION, Kind.GOAL),
            }[e.relation]
            if not valid:
                raise ContractError("Invalid typed edge")
        return scene


@dataclass(frozen=True, slots=True)
class Observation:
    episode: str
    sequence: int
    scene: Scene
    values: tuple[tuple[str, bool], ...]

    def value(self, node: str) -> bool:
        for n, v in self.values:
            if node == n:
                return v
        raise ContractError("Unobserved fluent; cannot treat missing as false")

    def to_wire(self) -> dict[str, Json]:
        return {"episode": self.episode, "sequence": self.sequence,
                "scene": self.scene.to_wire(), "values": {n: v for n, v in self.values}}

    @classmethod
    def from_wire(cls, value: object) -> Observation:
        p = record(value, {"episode", "sequence", "scene", "values"})
        scene = Scene.from_wire(p["scene"])
        required = {n.id for n in scene.nodes if n.kind in (Kind.FLUENT, Kind.GOAL)}
        states = record(p["values"], required)
        return cls(token(p["episode"]), integer(p["sequence"], 0, 1_000_000), scene,
                   tuple(sorted((n, boolean(v)) for n, v in states.items())))


@dataclass(frozen=True, slots=True)
class Trace:
    before: Observation
    action: str
    after: Observation

    def validate(self) -> None:
        if self.before.episode != self.after.episode:
            raise ContractError("Cross-episode trace")
        if self.after.sequence != self.before.sequence + 1:
            raise ContractError("Replayed/out-of-order transition")
        if self.before.scene != self.after.scene:
            raise ContractError("Structural change within a trace requires a new context")
        if self.before.scene.kind_of(self.action) != Kind.ACTION:
            raise ContractError("Not an action")

    def to_wire(self) -> dict[str, Json]:
        return {"before": self.before.to_wire(), "action": self.action, "after": self.after.to_wire()}

    @classmethod
    def from_wire(cls, value: object) -> Trace:
        p = record(value, {"before", "action", "after"})
        result = cls(Observation.from_wire(p["before"]), token(p["action"]), Observation.from_wire(p["after"]))
        result.validate()
        return result


class ActionPort(Protocol):
    def send_action(self, action_token: str) -> Observation: ...


def dumps(value: Json) -> str:
    result = json.dumps(value, separators=(",", ":"), ensure_ascii=True, allow_nan=False)
    if len(result.encode()) > MAX_BYTES:
        raise ContractError("Message exceeds limit")
    return result


def loads(text: str) -> object:
    if len(text.encode()) > MAX_BYTES:
        raise ContractError("Message exceeds limit")
    def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
        out: dict[str, object] = {}
        for key, val in items:
            if key in out:
                raise ContractError("Duplicate JSON key")
            out[key] = val
        return out
    def invalid_constant(_: str) -> None:
        raise ContractError("Non-finite JSON number")
    try:
        return json.loads(text, object_pairs_hook=pairs, parse_constant=invalid_constant)
    except (ValueError, RecursionError) as ex:
        raise ContractError("Invalid JSON") from ex

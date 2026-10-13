"""PRIVATE evaluator fixture. Never copied into the isolated solver directory.

This deliberately small simulator is a validation fixture, not a robotics engine.
The unknown law is a positive conjunction of at most two instrumented channels.
"""
from __future__ import annotations

import random
from dataclasses import dataclass

from .protocol import ContractError, Edge, Kind, Node, Observation, Relation, Scene, OBSERVE


@dataclass(frozen=True, slots=True)
class WorldConfig:
    seed: int
    guards: tuple[int, ...] = (0,)
    flags: tuple[bool, ...] = (True, True)
    episode_index: int = 0


class World:
    def __init__(self, config: WorldConfig) -> None:
        if not 2 <= len(config.flags) <= 6:
            raise ValueError("Fixture supports 2..6 instrumented channels")
        if not config.guards or len(set(config.guards)) != len(config.guards):
            raise ValueError("Invalid hidden guards")
        if any(i not in range(len(config.flags)) for i in config.guards):
            raise ValueError("Invalid hidden guard index")
        rng = random.Random(config.seed)
        count = len(config.flags)
        # Token generation is independent of the hidden law. The seed is private.
        def identifier() -> str:
            return "tok_" + format(rng.getrandbits(128), "032x")
        self.objects = tuple(identifier() for _ in range(count))
        self.fluents = tuple(identifier() for _ in range(count))
        self.goal = identifier()
        self.primary = identifier()
        self.clear = identifier()
        self.up = tuple(identifier() for _ in range(count))
        self.down = tuple(identifier() for _ in range(count))
        nodes = [Node(n, Kind.OBJECT) for n in self.objects]
        nodes += [Node(n, Kind.FLUENT) for n in self.fluents]
        nodes += [Node(self.goal, Kind.GOAL)]
        nodes += [Node(n, Kind.ACTION) for n in (self.primary, self.clear, *self.up, *self.down)]
        edges = [Edge(self.objects[i], Relation.LINK, self.objects[i + 1]) for i in range(count - 1)]
        edges += [Edge(f, Relation.AT, o) for f, o in zip(self.fluents, self.objects)]
        edges += [Edge(self.goal, Relation.AT, self.objects[-1]), Edge(self.primary, Relation.ACTS_ON, self.objects[0])]
        edges += [Edge(self.clear, Relation.CLEAR, self.goal)]
        edges += [Edge(a, Relation.SET_TRUE, f) for a, f in zip(self.up, self.fluents)]
        edges += [Edge(a, Relation.SET_FALSE, f) for a, f in zip(self.down, self.fluents)]
        width, height = rng.randrange(12, 4096), rng.randrange(12, 4096)
        layout = [(n.id, rng.randrange(width), rng.randrange(height)) for n in nodes]
        rng.shuffle(nodes)
        rng.shuffle(edges)
        rng.shuffle(layout)
        self.scene = Scene(tuple(nodes), tuple(edges), width, height, tuple(layout))
        self.episode = f"ep_{rng.getrandbits(96):024x}_{config.episode_index}"
        self._guard_ids = tuple(self.fluents[i] for i in config.guards)
        self._values = dict(zip(self.fluents, config.flags)) | {self.goal: False}
        self._sequence = 0

    def send_action(self, action_token: str) -> Observation:
        if action_token == OBSERVE:
            return self._observation()
        if action_token == self.primary:
            if all(self._values[x] for x in self._guard_ids):
                self._values[self.goal] = True
        elif action_token == self.clear:
            self._values[self.goal] = False
        elif action_token in self.up:
            self._values[self.fluents[self.up.index(action_token)]] = True
        elif action_token in self.down:
            self._values[self.fluents[self.down.index(action_token)]] = False
        else:
            raise ContractError("Unknown action token")
        self._sequence += 1
        return self._observation()

    def _observation(self) -> Observation:
        return Observation(self.episode, self._sequence, self.scene, tuple(sorted(self._values.items())))

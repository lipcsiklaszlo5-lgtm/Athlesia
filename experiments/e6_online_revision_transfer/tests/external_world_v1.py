"""A separate, deterministic Boolean-world IMPLEMENTATION for Day6b.

Uses Athlesia's documented PUBLIC protocol only. Does NOT import the E6
private fixture (athlesia.environment) and does not expose hidden guard truth
in Scene/Observation. Still uses the SAME public symbolic ontology, so it is
not an independently commissioned evaluator or evidence of ARC-AGI-3 transfer.
"""
from __future__ import annotations

import hashlib
import random

from athlesia.protocol import (ContractError, Edge, Kind, Node, Observation,
                               Relation, Scene, OBSERVE)


def _opaque(seed: int, label: str) -> str:
    payload = f'external-v1|{seed}|{label}'.encode('ascii')
    return 'x_' + hashlib.blake2s(payload, digest_size=12).hexdigest()


class ExternalWorld:
    """Externally visible action/fluents, private one/two-literal guard law."""

    def __init__(self, seed: int, channels: int, guard: tuple[tuple[int, bool], ...],
                 flags: tuple[bool, ...], episode: int):
        if not 3 <= channels <= 5 or len(flags) != channels or any(type(x) is not bool for x in flags):
            raise ValueError('Expected 3..5 Boolean channels')
        if not 1 <= len(guard) <= 2 or len({idx for idx, _ in guard}) != len(guard):
            raise ValueError('Expected one or two unique guard literals')
        if any(type(idx) is not int or not 0 <= idx < channels or type(pol) is not bool
               for idx, pol in guard):
            raise ValueError('Invalid hidden guard')
        self._law = tuple(guard)  # Hidden evaluator state; never serialized.
        self.seed = seed
        self.channels = channels
        self.episode_index = episode
        self.objects = tuple(_opaque(seed, f'obj-{i}') for i in range(channels))
        self.fluents = tuple(_opaque(seed, f'bool-{i}') for i in range(channels))
        self.goal = _opaque(seed, 'success-register')
        self.primary = _opaque(seed, 'opaque-actuator')
        self.clear = _opaque(seed, 'clear-register')
        self.up = tuple(_opaque(seed, f'calibrate-high-{i}') for i in range(channels))
        self.down = tuple(_opaque(seed, f'calibrate-low-{i}') for i in range(channels))
        self.episode = _opaque(seed, f'episode-{episode}')
        self._registers = dict(zip(self.fluents, flags)) | {self.goal: False}
        self._sequence = 0
        nodes = [Node(n, Kind.OBJECT) for n in self.objects]
        nodes.extend(Node(n, Kind.FLUENT) for n in self.fluents)
        nodes.append(Node(self.goal, Kind.GOAL))
        nodes.extend(Node(n, Kind.ACTION) for n in
                     (self.primary, self.clear, *self.up, *self.down))
        edges = [Edge(self.objects[i], Relation.LINK, self.objects[i + 1])
                 for i in range(channels - 1)]
        edges += [Edge(self.fluents[i], Relation.AT, self.objects[i]) for i in range(channels)]
        edges += [Edge(self.goal, Relation.AT, self.objects[-1]),
                  Edge(self.primary, Relation.ACTS_ON, self.objects[0]),
                  Edge(self.clear, Relation.CLEAR, self.goal)]
        edges += [Edge(self.up[i], Relation.SET_TRUE, self.fluents[i]) for i in range(channels)]
        edges += [Edge(self.down[i], Relation.SET_FALSE, self.fluents[i]) for i in range(channels)]
        # Independent layout/order generator; every episode keeps a fixed
        # scene for the trace, and the topology stays isomorphic across epochs.
        rnd = random.Random(int(hashlib.sha256(f'{seed}/{episode}/geometry'.encode()).hexdigest(), 16))
        rnd.shuffle(nodes)
        rnd.shuffle(edges)
        width, height = 613, 757
        layout = [(n.id, rnd.randrange(width), rnd.randrange(height)) for n in nodes]
        rnd.shuffle(layout)
        self.scene = Scene(tuple(nodes), tuple(edges), width, height, tuple(layout))

    def send_action(self, token: str) -> Observation:
        if token == OBSERVE:
            return self._snapshot()
        if token == self.primary:
            if all(self._registers[self.fluents[idx]] is wanted for idx, wanted in self._law):
                self._registers[self.goal] = True
        elif token == self.clear:
            self._registers[self.goal] = False
        elif token in self.up:
            self._registers[self.fluents[self.up.index(token)]] = True
        elif token in self.down:
            self._registers[self.fluents[self.down.index(token)]] = False
        else:
            raise ContractError('Unknown action token')
        self._sequence += 1
        return self._snapshot()

    def _snapshot(self) -> Observation:
        return Observation(self.episode, self._sequence, self.scene,
                           tuple(sorted(self._registers.items())))

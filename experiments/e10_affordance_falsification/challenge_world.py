"""PRIVATE E10 adversarial public-interface harness, same-author.

No solver/bridge import of this file is permitted. Out-of-grammar situations
are deliberately injected behind the normal E9 public observe/action API.
"""
from __future__ import annotations
from pixel_world import World

SUPPORTED = 'supported'
KINDS = ('supported', 'inventory_missing', 'inventory_extra', 'dead_control',
         'multipixel_effect', 'aliased_controls', 'ambiguous_gate',
         'irreversible_latch', 'late_semantic_drift', 'nonstationary_control')

class ChallengeWorld(World):
    def __init__(self, spec, episode: int, kind: str):
        if kind not in KINDS:
            raise ValueError('Invalid challenge variant')
        super().__init__(spec, episode)
        self.kind = kind
        self._latched = False
        self._decoy = False
        self._used: dict[str, int] = {}
        self._extra_colour = next(x for x in range(1, 256) if
                                  x not in set(spec.case.palette) |
                                  {c for pair in spec.colours for c in pair} |
                                  {v for _, v in spec.decorations})

    def available_actions(self):
        tokens = super().available_actions()
        if self.kind == 'inventory_missing':
            return tokens[:-1]
        if self.kind == 'inventory_extra':
            return tokens + ('unknown_extra_action',)
        return tokens

    def observe(self):
        frame = super().observe()
        if self.kind not in ('multipixel_effect', 'ambiguous_gate'):
            return frame
        rows = [list(row) for row in frame]
        if self.kind == 'ambiguous_gate':
            # Unique unknown neighbour prior is deliberately violated.
            r, c = self._spec.start
            rows[r][c-1] = self._extra_colour
        elif self._decoy:
            rows[7][7] = self._extra_colour
        return tuple(tuple(row) for row in rows)

    def _noop(self):
        self.real_actions += 1
        self.sequence += 1
        return self.observe()

    def send_action(self, token):
        pair0, pair1 = self._spec.control_pairs
        if self.kind == 'dead_control' and token == pair0[1]:
            return self._noop()
        if self.kind == 'aliased_controls' and token in pair1:
            self._flags[0] = token == pair1[1]
            return self._noop()
        if self.kind == 'irreversible_latch' and token == pair0[0] and self._latched:
            return self._noop()
        if self.kind == 'late_semantic_drift' and self.real_actions >= 8 and token == pair0[0]:
            return self._noop()
        if self.kind == 'nonstationary_control' and token == pair0[0] and self._used.get(token,0) >= 1:
            return self._noop()
        previous = self.observe()
        result = super().send_action(token)
        self._used[token] = self._used.get(token,0) + 1
        if self.kind == 'irreversible_latch' and token == pair0[1]:
            self._latched = True
        if self.kind == 'multipixel_effect' and token in (pair0 + pair1) and result != previous:
            self._decoy = not self._decoy
            result = self.observe()
        return result

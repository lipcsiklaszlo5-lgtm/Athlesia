"""A small observational effect-epoch experiment; not autonomous AGI.

External environment changes one previously learned action's effect role.
The learner sees only Trace(before, action, after); no hidden target is supplied.
"""
from __future__ import annotations

import argparse
from dataclasses import replace
import itertools
import json
from pathlib import Path
import random
import time
import unittest

from athlesia.environment import World, WorldConfig
from athlesia.learner import Agent, CannotIdentify, Literal
from athlesia.protocol import OBSERVE, Trace


class AlteredAuxWorld(World):
    """Private test fixture, never imported by the solver."""
    def send_action(self, token):
        if token == self.auxiliary[0]:
            self._values[self.fluents[2]] = False
            self._sequence += 1
            return self._observation()
        return super().send_action(token)


def trace(world, action):
    before = world.send_action(OBSERVE)
    return Trace(before, action, world.send_action(action))


def train_old(seed: int, channels: int = 3):
    demos = []
    for i in range(3):
        w = World(WorldConfig(seed, (0,), (True,) * channels, i, True))
        demos.append(trace(w, w.primary))
    # Identify unguarded auxiliary effect: vary ALL other feature states during
    # positive witnesses. No source rule or effect target is given to the agent.
    for i, assignment in enumerate(itertools.product((False, True), repeat=channels - 1), 10):
        flags = list(assignment)
        flags.insert(1, False)
        w = World(WorldConfig(seed, (0,), tuple(flags), i, True))
        demos.append(trace(w, w.auxiliary[0]))
    w = World(WorldConfig(seed, (0,), (True,) * channels, 40, True))
    demos.append(trace(w, w.auxiliary[0]))
    for j, flags in enumerate(((True,) * channels, (False,) * channels), 41):
        w = World(WorldConfig(seed, (0,), flags, j, True))
        demos.append(trace(w, w.auxiliary[1]))
    agent = Agent()
    agent.fit(tuple(demos))
    role = agent.roles[w.auxiliary[0]]
    assert agent.rules[role].guards == () and len(agent.version_spaces[role]) == 1
    return agent, role


def shifted_trace(seed, flags, episode):
    w = AlteredAuxWorld(WorldConfig(seed, (0,), flags, episode, True))
    return trace(w, w.auxiliary[0]), w


def run_shift(agent, seed, channels):
    f0 = (False, False, True) + (False,) * (channels - 3)
    f1 = (True, False, True) + (False,) * (channels - 3)
    f2 = (False, False, False) + (False,) * (channels - 3)
    t0, world = shifted_trace(seed, f0, 100)
    s0 = agent.observe_effect_shift(t0)
    t1, _ = shifted_trace(seed, f1, 101)
    s1 = agent.observe_effect_shift(t1)
    t2, _ = shifted_trace(seed, f2, 102)
    s2 = agent.observe_effect_shift(t2)
    return (s0, s1, s2), world


class EffectEpochTests(unittest.TestCase):
    def test_new_effect_learned_and_old_rule_archived(self):
        seed = 8101
        a, role = train_old(seed)
        previous = a.rules[role]
        other = {r: (rule, a.version_spaces[r], dict(a.witness_store(r)))
                 for r, rule in a.rules.items() if r != role}
        outcomes, world = run_shift(a, seed, 3)
        self.assertEqual([s['status'] for s in outcomes],
                         ['quarantined', 'quarantined', 'committed'])
        self.assertEqual(a.commits, 1)
        self.assertEqual(a.rules[role].effects,
                         (Literal(a.roles[world.fluents[2]], False),))
        self.assertEqual(a.rules[role].guards, ())
        self.assertEqual(a.mechanism_archive[role][0][0], previous)
        self.assertNotIn(role, a.pending_actions)
        self.assertNotIn(role, a.effect_candidates)
        for r, (rule, space, witnesses) in other.items():
            self.assertEqual(a.rules[r], rule)
            self.assertEqual(a.version_spaces[r], space)
            self.assertEqual(a.witness_store(r), witnesses)
        prospective, w = shifted_trace(seed, (True, False, True), 200)
        self.assertEqual(dict(a.predict_public(prospective.before, w.auxiliary[0])),
                         dict(prospective.after.values))

    def test_first_anomaly_quarantines_and_blocks_prediction(self):
        seed = 8103
        a, role = train_old(seed)
        old = a.rules[role]
        t, w = shifted_trace(seed, (False, False, True), 100)
        self.assertEqual(a.observe_effect_shift(t)['status'], 'quarantined')
        self.assertEqual(a.rules[role], old)
        self.assertEqual(a.commits, 0)
        with self.assertRaises(CannotIdentify):
            a.predict_public(t.before, w.auxiliary[0])
        with self.assertRaises(CannotIdentify):
            a.observe_transition(t)

    def test_two_positives_cannot_commit_without_noop(self):
        seed = 8107
        a, role = train_old(seed)
        for i, flags in enumerate(((False, False, True), (True, False, True)), 100):
            t, _ = shifted_trace(seed, flags, i)
            self.assertFalse(a.observe_effect_shift(t)['local_commit'])
        self.assertEqual(a.commits, 0)
        self.assertIn(role, a.effect_candidates)

    def test_unchanged_old_effect_is_not_an_epoch(self):
        seed = 8109
        a, role = train_old(seed)
        w = World(WorldConfig(seed, (0,), (False, False, True), 100, True))
        observed = trace(w, w.auxiliary[0])
        before, old_events = a.memory_digest(), len(a.events)
        result = a.observe_effect_shift(observed)
        self.assertEqual(result['status'], 'unchanged')
        self.assertEqual(a.rules[role].effects,
                         (Literal(a.roles[w.fluents[1]], True),))
        self.assertEqual(a.memory_digest(), before)
        self.assertEqual(len(a.events), old_events)

    def test_multi_effect_anomaly_rejected_atomically(self):
        seed = 8111
        a, _ = train_old(seed)
        t, w = shifted_trace(seed, (False, False, True), 100)
        changed = dict(t.after.values)
        changed[w.fluents[0]] = True
        forged = replace(t, after=replace(t.after,
                         values=tuple(sorted(changed.items()))))
        digest, n = a.memory_digest(), len(a.events)
        with self.assertRaisesRegex(CannotIdentify, 'singly identifiable'):
            a.observe_effect_shift(forged)
        self.assertEqual(a.memory_digest(), digest)
        self.assertEqual(len(a.events), n)

    def test_conflicting_followup_and_duplicate_episode_atomic(self):
        seed = 8117
        a, _ = train_old(seed)
        t, _ = shifted_trace(seed, (False, False, True), 100)
        a.observe_effect_shift(t)
        digest, events = a.memory_digest(), len(a.events)
        with self.assertRaisesRegex(CannotIdentify, 'Repeated effect episode'):
            a.observe_effect_shift(t)
        self.assertEqual(a.memory_digest(), digest)
        followup = World(WorldConfig(seed, (0,), (False, False, True), 101, True))
        normal = trace(followup, followup.auxiliary[0])
        with self.assertRaisesRegex(CannotIdentify, 'Contradictory effect-epoch'):
            a.observe_effect_shift(normal)
        self.assertEqual(a.memory_digest(), digest)
        self.assertEqual(len(a.events), events)


def stress(cases):
    if not 1 <= cases <= 200:
        raise ValueError('cases must be in 1..200')
    random_gen = random.Random(0xD5B2026)
    checks = {'quarantined_first': 0, 'effect_relearned': 0,
              'archive_preserved': 0, 'unrelated_action_preserved': 0}
    started = time.perf_counter_ns()
    for i in range(cases):
        seed = random_gen.randrange(1, 2 ** 42)
        channels = 3 + i % 2
        a, role = train_old(seed, channels)
        old = a.rules[role]
        unrelated = {x: a.rules[x] for x in a.rules if x != role}
        states, world = run_shift(a, seed, channels)
        if [s['status'] for s in states] != ['quarantined','quarantined','committed']:
            raise AssertionError('Unexpected shift sequence')
        checks['quarantined_first'] += 1
        if a.rules[role].effects != (Literal(a.roles[world.fluents[2]], False),):
            raise AssertionError('Incorrect inferred new effect')
        checks['effect_relearned'] += 1
        if a.mechanism_archive[role][0][0] != old:
            raise AssertionError('Old epoch missing')
        checks['archive_preserved'] += 1
        if any(a.rules[x] != r for x, r in unrelated.items()):
            raise AssertionError('Unrelated model changed')
        checks['unrelated_action_preserved'] += 1
        if (i + 1) % 10 == 0:
            print(f'DAY5B_EFFECTS {i+1}/{cases}', flush=True)
    return {'benchmark':'Athlesia E6 Day5b observed-single-effect drift pilot v0.1',
            'cases':cases,'checks':checks,'official_arc_agi_3':False,
            'scope':'existing identified unguarded action, one changed Boolean effect',
            'independent_generator':False,'autonomous_interventions':False,
            'new_features_discovered':False,
            'wall_ms_total':(time.perf_counter_ns()-started)/1e6}


if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    ap.add_argument('--cases', type=int, default=100)
    ap.add_argument('--output', type=Path,
                    default=Path('results/day5b_effects100.json'))
    args = ap.parse_args()
    result = stress(args.cases)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(result, indent=2))
    if any(v != args.cases for v in result['checks'].values()):
        raise SystemExit('DAY5B_FAIL')
    print('DAY5B_EFFECT_PILOT_PASSED')

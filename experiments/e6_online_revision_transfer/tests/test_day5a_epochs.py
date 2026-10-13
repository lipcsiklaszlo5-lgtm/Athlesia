"""Guard-mechanism replacement after observational contradictions.

LIMITATIONS: The environment is author-defined and the effect signature is
fixed; this pilot does not prove AGI, new-effect discovery or noisy drift
identification. Multiple observation episodes are not independent generators.
"""
from __future__ import annotations
import argparse
import itertools
import json
from pathlib import Path
import random
import time
import unittest

from athlesia.environment import World, WorldConfig
from athlesia.learner import Agent, CannotIdentify, Literal
from athlesia.protocol import OBSERVE, Trace


def make_trace(seed: int, channels: int, guards: tuple[int, ...],
               flags: tuple[bool, ...], episode: int):
    w = World(WorldConfig(seed, guards, flags, episode))
    before = w.send_action(OBSERVE)
    return Trace(before, w.primary, w.send_action(w.primary)), w


def trained_identified(seed: int, channels: int = 3, old: tuple[int, ...] = (0,)):
    traces = []
    for i, flags in enumerate(itertools.product((False, True), repeat=channels)):
        trace, _ = make_trace(seed, channels, old, flags, i)
        traces.append(trace)
    agent = Agent()
    agent.fit(tuple(traces))
    assert len(agent.version_space) == 1
    return agent


def move_epoch(agent, seed, channels, new_guard):
    start = (True, False) + (False,) * (channels - 2)
    initial, source = make_trace(seed, channels, new_guard, start, 100)
    first = agent.observe_mechanism_shift(initial)
    assert first['status'] == 'quarantined'
    assert not first['local_commit']
    observed = 1
    terminal = None
    for i, flags in enumerate(itertools.product((False, True), repeat=channels), 101):
        if flags == start:
            continue
        trace, _ = make_trace(seed, channels, new_guard, flags, i)
        terminal = agent.observe_mechanism_shift(trace)
        observed += 1
        if terminal['status'] == 'committed':
            break
    assert terminal is not None and terminal['status'] == 'committed', terminal
    return observed, source


class MechanismEpochTests(unittest.TestCase):
    def test_detect_quarantine_relearn_and_keep_archive(self):
        seed = 711
        a = trained_identified(seed)
        old = a.rule
        old_witnesses = dict(a.witnesses)
        start, world = make_trace(seed, 3, (1,), (True, False, False), 100)
        initial = a.observe_mechanism_shift(start)
        self.assertEqual(initial['status'], 'quarantined')
        self.assertEqual(a.rule, old)
        self.assertEqual(a.commits, 0)
        self.assertIn(a.primary, a.pending_actions)
        with self.assertRaises(CannotIdentify):
            a.predict_public(start.before, world.primary)
        with self.assertRaises(CannotIdentify):
            a.observe_transition(start)
        observed = 1
        for episode, flags in enumerate(itertools.product((False, True), repeat=3), 101):
            if flags == (True, False, False):
                continue
            transition, _ = make_trace(seed, 3, (1,), flags, episode)
            result = a.observe_mechanism_shift(transition)
            observed += 1
            if result['status'] == 'committed':
                break
        self.assertEqual(result['status'], 'committed')
        self.assertGreaterEqual(observed, 3)
        self.assertEqual(a.rule.guards, (Literal(a.roles[world.fluents[1]], True),))
        self.assertEqual(a.commits, 1)
        self.assertNotIn(a.primary, a.pending_actions)
        snapshot = a.mechanism_archive[a.primary][0]
        self.assertEqual(snapshot[0], old)
        self.assertEqual(snapshot[2], old_witnesses)
        self.assertEqual(len(a.version_space), 1)
        self.assertTrue(all(a.consistent(h) for h in a.version_space))

    def test_single_anomaly_cannot_commit(self):
        seed=719
        a=trained_identified(seed)
        first,_=make_trace(seed,3,(1,),(True,False,False),100)
        a.observe_mechanism_shift(first)
        self.assertEqual(a.commits,0)
        self.assertEqual(a.rule.guards,(Literal(a.roles[World(WorldConfig(seed,(0,),(True,)*3)).fluents[0]],True),))
        self.assertEqual(len(a.mechanism_archive),0)

    def test_contradictory_new_epoch_trace_atomic_rejection(self):
        seed=727
        a=trained_identified(seed)
        first,_=make_trace(seed,3,(1,),(True,False,False),100)
        a.observe_mechanism_shift(first)
        digest=a.memory_digest()
        event_count=len(a.events)
        contradictory,_=make_trace(seed,3,(0,),(True,False,False),101)
        with self.assertRaisesRegex(CannotIdentify,'Conflicting new-epoch'):
            a.observe_mechanism_shift(contradictory)
        self.assertEqual(a.memory_digest(),digest)
        self.assertEqual(len(a.events),event_count)
        self.assertEqual(a.commits,0)

    def test_unknown_effect_is_atomic_rejection(self):
        from dataclasses import replace
        seed=733
        a=trained_identified(seed)
        transition,world=make_trace(seed,3,(1,),(False,False,False),100)
        altered=dict(transition.after.values)
        altered[world.fluents[0]]=True
        forged=replace(transition,after=replace(transition.after,
                         values=tuple(sorted(altered.items()))))
        digest=a.memory_digest()
        with self.assertRaisesRegex(CannotIdentify,'Unmodelled changed fluent'):
            a.observe_mechanism_shift(forged)
        self.assertEqual(a.memory_digest(),digest)
        self.assertEqual(a.commits,0)

    def test_no_false_positive_if_mechanism_unchanged(self):
        seed=743
        a=trained_identified(seed)
        transition,_=make_trace(seed,3,(0,),(True,False,False),100)
        digest=a.memory_digest()
        result=a.observe_mechanism_shift(transition)
        self.assertEqual(result['status'],'unchanged')
        self.assertEqual(a.memory_digest(),digest)


def run_stress(cases: int):
    if not 1<=cases<=300:
        raise ValueError('cases must be in 1..300')
    rng=random.Random(0xE60A5A)
    counts={'old_epoch_identified':0,'single_trace_quarantined':0,
            'new_guard_reidentified':0,'old_epoch_archived':0}
    start=time.perf_counter_ns()
    for i in range(cases):
        seed=rng.randrange(1,2**40)
        channels=3+i%2
        new_guard=(1+(i%(channels-1)),)
        a=trained_identified(seed,channels)
        old=a.rule
        counts['old_epoch_identified']+=1
        n,world=move_epoch(a,seed,channels,new_guard)
        assert n>=3 and a.commits==1
        counts['single_trace_quarantined']+=1
        expected=(Literal(a.roles[world.fluents[new_guard[0]]],True),)
        assert a.rule.guards==expected,(i,a.rule.guards,expected)
        counts['new_guard_reidentified']+=1
        assert a.mechanism_archive[a.primary][0][0]==old
        counts['old_epoch_archived']+=1
        if (i+1)%10==0:
            print(f'DAY5A_EPOCHS {i+1}/{cases}',flush=True)
    return {'benchmark':'Athlesia E6 Day5a mechanism epoch pilot v0.1',
            'cases':cases,'checks':counts,
            'test_family':'same author-developed Boolean generator, unchanged effect signature',
            'drift_identification':'requires independent episode observations; assumes stationary new law',
            'autonomous_intervention_selection':False,
            'new_effect_signature_learning':False,
            'noisy_mechanism_distinction':False,
            'official_arc_agi_3':False,
            'wall_ms_total':(time.perf_counter_ns()-start)/1e6}


if __name__=='__main__':
    p=argparse.ArgumentParser()
    p.add_argument('--cases',type=int,default=100)
    p.add_argument('--output',type=Path,default=Path('results/day5a_epochs100.json'))
    args=p.parse_args()
    report=run_stress(args.cases)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,indent=2))
    if any(v!=args.cases for v in report['checks'].values()):
        raise SystemExit('DAY5A_FAIL: incomplete category')
    print('DAY5A_EPOCH_PILOT_PASSED')

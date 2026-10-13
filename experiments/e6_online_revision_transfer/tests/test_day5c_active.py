"""Day5c active guarded-mechanism identification in a fresh real environment.

The hidden law is fixture-private; solver chooses settings based only on
public state, current version space and advertised calibration actions.
This does NOT reset episodes autonomously or establish ARC-AGI-3 performance.
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
from athlesia.learner import CannotIdentify, Literal
from athlesia.protocol import OBSERVE
from test_day5a_epochs import trained_identified, make_trace


def start_epoch(seed, channels, new_guards):
    agent = trained_identified(seed,channels)
    flags=(True,) + (False,) * (channels - 1)
    trace,world=make_trace(seed,channels,new_guards,flags,100)
    r=agent.observe_mechanism_shift(trace)
    assert r['status']=='quarantined'
    return agent,world


def finish_active(agent, seed, channels, new_guards, budget=20):
    attempts=0
    preps=0
    scores=[]
    for episode in range(101,101+budget):
        # Fresh episode configuration is supplied by the test broker, NOT a
        # hidden-rule-setting/reset command inside the learner.
        w=World(WorldConfig(seed,new_guards,(False,)*channels,episode))
        r=agent.run_active_guard_probe(w,w.primary)
        assert r['active_selection'] and r['probe_actions']==1
        attempts+=1
        preps+=r['preparation_actions']
        scores.append(r['worst_case_eliminated'])
        if r['status']=='committed':
            return attempts,preps,scores,w
    raise AssertionError('Active probe did not identify new bounded mechanism')


class ActiveGuardTests(unittest.TestCase):
    def test_planning_does_not_touch_memory_or_environment(self):
        agent,_=start_epoch(20111,3,(1,))
        world=World(WorldConfig(20111,(1,),(False,)*3,101))
        obs=world.send_action(OBSERVE)
        digest=agent.memory_digest()
        plan=agent.choose_active_guard_probe(obs,world.primary)
        self.assertGreater(plan['worst_case_eliminated'],0)
        self.assertEqual(agent.memory_digest(),digest)
        self.assertEqual(world.send_action(OBSERVE).sequence,obs.sequence)

    def test_auto_probe_reidentifies_unknown_guard(self):
        agent,old_world=start_epoch(20113,3,(1,))
        old=agent.rule
        steps,preps,_,world=finish_active(agent,20113,3,(1,))
        self.assertGreaterEqual(steps,2)
        self.assertGreaterEqual(preps,0)
        self.assertEqual(agent.rule.guards,(Literal(agent.roles[world.fluents[1]],True),))
        self.assertEqual(agent.mechanism_archive[agent.primary][0][0],old)
        self.assertNotIn(agent.primary,agent.pending_actions)

    def test_same_episode_rejected_before_action(self):
        seed=20117
        agent,_=start_epoch(seed,3,(1,))
        w=World(WorldConfig(seed,(1,),(False,)*3,100))
        before=w.send_action(OBSERVE)
        digest=agent.memory_digest()
        with self.assertRaisesRegex(CannotIdentify,'fresh episode'):
            agent.run_active_guard_probe(w,w.primary)
        self.assertEqual(w.send_action(OBSERVE).sequence,before.sequence)
        self.assertEqual(agent.memory_digest(),digest)

    def test_unquarantined_action_cannot_probe(self):
        seed=20119
        agent=trained_identified(seed,3)
        w=World(WorldConfig(seed,(0,),(False,)*3,101))
        before=w.send_action(OBSERVE)
        with self.assertRaisesRegex(CannotIdentify,'No quarantined'):
            agent.run_active_guard_probe(w,w.primary)
        self.assertEqual(w.send_action(OBSERVE).sequence,before.sequence)

    def test_new_action_not_mapped_cannot_probe(self):
        seed=20123
        agent,_=start_epoch(seed,3,(1,))
        w=World(WorldConfig(seed,(1,),(False,)*3,101))
        digest=agent.memory_digest()
        with self.assertRaises(CannotIdentify):
            agent.run_active_guard_probe(w,w.up[0])
        self.assertEqual(agent.memory_digest(),digest)


def stress(cases):
    if not 1<=cases<=200:
        raise ValueError('cases must be in 1..200')
    rng=random.Random(0xD5C2026)
    counts={'quarantine':0,'active_reidentified':0,'archive_preserved':0,
            'unrelated_actions_preserved':0}
    probes=[]
    interventions=[]
    tic=time.perf_counter_ns()
    for i in range(cases):
        seed=rng.randrange(1,2**40)
        channels=3+(i%2)
        new=(1+(i%(channels-1)),)
        agent,old_world=start_epoch(seed,channels,new)
        old_rule=agent.rule
        others={r:agent.rules[r] for r in agent.rules if r !=agent.primary}
        counts['quarantine']+=1
        n,prep,_,world=finish_active(agent,seed,channels,new)
        probes.append(n)
        interventions.append(prep)
        assert agent.rule.guards==(Literal(agent.roles[world.fluents[new[0]]],True),)
        counts['active_reidentified']+=1
        assert agent.mechanism_archive[agent.primary][0][0]==old_rule
        counts['archive_preserved']+=1
        assert all(agent.rules[r]==x for r,x in others.items())
        counts['unrelated_actions_preserved']+=1
        if (i+1)%10==0:
            print(f'DAY5C_ACTIVE {i+1}/{cases}',flush=True)
    ordered=sorted(probes)
    return {'benchmark':'Athlesia E6 Day5c active bounded guard probe v0.1',
            'cases':cases,'checks':counts,
            'probe_actions_total':sum(probes),
            'probe_actions_p50':ordered[len(ordered)//2],
            'probe_actions_max':max(probes),
            'calibration_actions_total':sum(interventions),
            'fixture':'author-developed, Boolean guard shift; externally fresh episodes',
            'independent_generator':False,'autonomous_environment_reset':False,
            'official_arc_agi_3':False,
            'wall_ms_total':(time.perf_counter_ns()-tic)/1e6}


if __name__=='__main__':
    ap=argparse.ArgumentParser()
    ap.add_argument('--cases',type=int,default=100)
    ap.add_argument('--output',type=Path,default=Path('results/day5c_active100.json'))
    args=ap.parse_args()
    report=stress(args.cases)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,indent=2))
    assert all(n==args.cases for n in report['checks'].values())
    print('DAY5C_ACTIVE_PILOT_PASSED')

"""Day 4b: runtime-targeted goal switching from previously learned effects.

Not autonomous goal invention, mechanism-change relearning, or independent
held-out ARC-AGI-3 evaluation. Tests include genuine environment actions.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import random
import time
import unittest

from athlesia.environment import World, WorldConfig
from athlesia.learner import Agent, CannotIdentify
from athlesia.protocol import Trace, OBSERVE
from test_day4_topology import ExpandedWorld


def train_mixed(seed: int = 31, channels: int = 3) -> Agent:
    traces = []
    for episode in range(3):
        w=World(WorldConfig(seed,(0,),(True,)*channels,episode,True))
        before=w.send_action(OBSERVE)
        traces.append(Trace(before,w.primary,w.send_action(w.primary)))
    for episode,aux,flags in ((20,0,(False,)*channels),
                              (21,0,(True,)*channels),
                              (22,1,(True,)*channels),
                              (23,1,(False,)*channels)):
        w=World(WorldConfig(seed,(0,),flags,episode,True))
        before=w.send_action(OBSERVE)
        traces.append(Trace(before,w.auxiliary[aux],w.send_action(w.auxiliary[aux])))
    agent=Agent()
    agent.fit(tuple(traces))
    if len(agent.rules)!=3:
        raise AssertionError('Must first induce exactly three independent action schemas')
    return agent


class DriftingAuxWorld(World):
    """Evaluator-only action drift; never supplied as a solver-side law."""
    def send_action(self, token: str):
        result=super().send_action(token)
        if token==self.auxiliary[0]:
            self._values[self.fluents[1]]=False
            return self._observation()
        return result


class Day4bGoalTests(unittest.TestCase):
    def test_switch_to_positive_auxiliary_goal(self):
        agent=train_mixed()
        w=World(WorldConfig(95,(0,),(False,False,False),40,True))
        prior=agent.memory_digest()
        result=agent.solve_public_goal(w,w.fluents[1],True)
        self.assertEqual(result['chosen_action'],w.auxiliary[0])
        self.assertEqual(result['actions'],1)
        self.assertTrue(w._values[w.fluents[1]])
        self.assertEqual(agent.memory_digest(),prior)

    def test_switch_to_negative_auxiliary_goal(self):
        agent=train_mixed()
        w=World(WorldConfig(97,(0,),(True,True,True),41,True))
        result=agent.solve_public_goal(w,w.fluents[-1],False)
        self.assertEqual(result['chosen_action'],w.auxiliary[1])
        self.assertFalse(w._values[w.fluents[-1]])

    def test_already_satisfied_has_zero_actions(self):
        agent=train_mixed()
        w=World(WorldConfig(99,(0,),(False,True,False),42,True))
        before=w.send_action(OBSERVE)
        result=agent.solve_public_goal(w,w.fluents[1],True)
        self.assertEqual(result['status'],'already_satisfied')
        self.assertEqual(result['actions'],0)
        self.assertEqual(w.send_action(OBSERVE).sequence,before.sequence)

    def test_uncertain_state_refuses_without_experiment(self):
        agent=train_mixed()
        w=World(WorldConfig(101,(0,),(True,False,True),43,True))
        before=w.send_action(OBSERVE)
        prior=agent.memory_digest()
        with self.assertRaisesRegex(CannotIdentify,'no unique'):
            agent.solve_public_goal(w,w.fluents[1],True)
        self.assertEqual(w.send_action(OBSERVE).sequence,before.sequence)
        self.assertEqual(prior,agent.memory_digest())

    def test_unlearned_new_goal_is_not_hallucinated(self):
        agent=train_mixed()
        w=ExpandedWorld(World(WorldConfig(103,(0,),(False,)*3,44,True)))
        before=w.send_action(OBSERVE)
        with self.assertRaisesRegex(CannotIdentify,'not grounded'):
            agent.solve_public_goal(w,w.extra_fluent,True)
        self.assertEqual(w.send_action(OBSERVE).sequence,before.sequence)

    def test_expanded_topology_goal_with_known_effect(self):
        agent=train_mixed()
        w=ExpandedWorld(World(WorldConfig(105,(0,),(False,)*3,45,True)))
        prior=agent.memory_digest()
        result=agent.solve_public_goal(w,w.world.fluents[1],True)
        self.assertEqual(result['chosen_action'],w.world.auxiliary[0])
        self.assertEqual(result['status'],'achieved')
        self.assertEqual(agent.memory_digest(),prior)

    def test_changed_mechanism_does_not_claim_goal_success(self):
        agent=train_mixed()
        w=DriftingAuxWorld(WorldConfig(107,(0,),(False,)*3,46,True))
        before=w.send_action(OBSERVE)
        prior=agent.memory_digest()
        with self.assertRaisesRegex(CannotIdentify,'falsified learned transition'):
            agent.solve_public_goal(w,w.fluents[1],True)
        self.assertEqual(w.send_action(OBSERVE).sequence,before.sequence+1)
        self.assertEqual(agent.memory_digest(),prior)


def stress(cases: int):
    if not 1<=cases<=1000:
        raise ValueError('cases must be in 1..1000')
    rng=random.Random(0xD4B2026)
    counts={'goal_up':0,'goal_down':0,'expanded_goal':0,'uncertain_refused':0}
    start=time.perf_counter_ns()
    for i in range(cases):
        seed=rng.randrange(1,2**40)
        channels=3+(i%3)
        agent=train_mixed(seed,channels)
        prior=agent.memory_digest()
        if i%4 == 0:
            w=World(WorldConfig(seed+1_000_003,(0,),(False,)*channels,52,True))
            res=agent.solve_public_goal(w,w.fluents[1],True)
            assert res['status']=='achieved' and res['chosen_action']==w.auxiliary[0]
            counts['goal_up']+=1
        elif i%4 == 1:
            w=World(WorldConfig(seed+1_000_003,(0,),(True,)*channels,53,True))
            res=agent.solve_public_goal(w,w.fluents[-1],False)
            assert res['status']=='achieved' and res['chosen_action']==w.auxiliary[1]
            counts['goal_down']+=1
        elif i%4 == 2:
            w=ExpandedWorld(World(WorldConfig(seed+1_000_003,(0,),(False,)*channels,54,True)))
            res=agent.solve_public_goal(w,w.world.fluents[1],True)
            assert res['status']=='achieved' and res['chosen_action']==w.world.auxiliary[0]
            counts['expanded_goal']+=1
        else:
            w=World(WorldConfig(seed+1_000_003,(0,),
                                (True,False)+((True,)*(channels-2)),55,True))
            first=w.send_action(OBSERVE)
            try:
                agent.solve_public_goal(w,w.fluents[1],True)
            except CannotIdentify:
                pass
            else:
                raise AssertionError('Uncertain goal was improperly executed')
            assert w.send_action(OBSERVE).sequence==first.sequence
            counts['uncertain_refused']+=1
        assert prior==agent.memory_digest(), 'Goal pilot mutated learned memory'
        if (i+1)%10==0:
            print(f'DAY4B_GOALS {i+1}/{cases}',flush=True)
    return {'benchmark':'Athlesia E6 Day4b declared-goal learned-action pilot v0.1',
            'cases':cases,'checks':counts,
            'test_family':'author-developed in-process Boolean fixture',
            'official_arc_agi_3':False,
            'autonomous_goal_invention':False,
            'mechanism_relearning':False,
            'wall_ms_total':(time.perf_counter_ns()-start)/1e6}


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--cases',type=int,default=100)
    parser.add_argument('--output',type=Path,default=Path('results/day4b_goals100.json'))
    args=parser.parse_args()
    report=stress(args.cases)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,indent=2))
    assert sum(report['checks'].values())==args.cases
    print('DAY4B_GOAL_PILOT_PASSED')

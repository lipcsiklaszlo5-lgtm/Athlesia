"""Executable raw-grid E7 pilot, same-author test engine (not ARC-AGI-3)."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import random
import time
import unittest
from collections import Counter

from agent import Agent, CannotIdentify, Demonstration, find_one
from evaluator import World, new_case, training_demonstrations

BUDGET = 60
SCHEDULE = 0xE7A_2026


def manhattan(a, b):
    return abs(a[0]-b[0])+abs(a[1]-b[1])


def naive_rollout(agent: Agent, world: World, goal, budget: int, mode: str, rng: random.Random) -> dict:
    """Public-frame-only greedy and uniformly random control baselines."""
    visited: Counter = Counter()
    actions = tuple(sorted(agent.action_delta))
    for _ in range(budget):
        if world.success:
            break
        frame = world.observe()
        loc = find_one(frame, agent.actor_colour)
        if mode == 'random':
            choice = rng.choice(actions)
        else:
            scored = []
            for token in actions:
                dr, dc = agent.action_delta[token]
                nr, nc = loc[0]+dr, loc[1]+dc
                if not (0 <= nr < len(frame) and 0 <= nc < len(frame[0])):
                    continue
                if ((nr,nc) != goal and frame[nr][nc] != agent.floor_colour):
                    continue
                scored.append((manhattan((nr,nc),goal), visited[(nr,nc)], rng.random(), token))
            if not scored:
                return {'status':'refused_no_safe_local_move', 'success': False, 'actions':world.calls}
            choice=min(scored)[-1]
        world.send_action(choice)
        visited[find_one(world.observe(), agent.actor_colour)] += 1
    return {'status':'reached' if world.success else 'budget_exhausted',
            'success':world.success, 'actions':world.calls}


def one_case(seed: int, index: int, budget: int = BUDGET):
    case,optimal = new_case(seed)
    traces=training_demonstrations(case)
    agent=Agent()
    agent.fit(traces)
    learned_dynamics=(set(agent.action_delta.values()) == {(1,0),(-1,0),(0,1),(0,-1)})
    assert learned_dynamics
    world=World(case)
    try:
        reported=agent.execute(world,case.goal,max_actions=budget)
        status='reached' if world.success else 'false_claim'
    except CannotIdentify as ex:
        status='refused:'+str(ex)[:110]
    active={'success':world.success,'status':status,'actions':world.calls,
            'optimal_actions':optimal}
    if active['success']:
        assert active['actions'] == optimal, 'Visual planner did not find shortest path'
    g=World(case)
    greedy=naive_rollout(agent,g,case.goal,budget,'greedy',random.Random(seed^0x1199))
    r=World(case)
    random_result=naive_rollout(agent,r,case.goal,budget,'random',random.Random(seed^0x5511))
    return {'case':index,'visual_planner':active,'greedy_local':greedy,
            'uniform_random':random_result}


def benchmark(n: int, budget: int = BUDGET):
    if not 1<=n<=250 or not 1<=budget<=150:
        raise ValueError('Invalid case count or budget')
    rng=random.Random(SCHEDULE)
    seeds=[rng.randrange(1,2**55) for _ in range(n)]
    start=time.perf_counter()
    cases=[]
    for i,seed in enumerate(seeds):
        cases.append(one_case(seed,i,budget))
        if (i+1)%10 == 0:
            print(f'E7_RAW_GRID_COMPLETED {i+1}/{n}',flush=True)
    def result(key):
        records=[x[key] for x in cases]
        actions=[x['actions'] for x in records]
        successes=[x for x in records if x['success']]
        return {'successes':len(successes),'failures':len(records)-len(successes),
                'real_actions_all_cases':sum(actions),
                'real_actions_median_success_only':(sorted(x['actions'] for x in successes)[len(successes)//2]
                                                      if successes else None),
                'status_counts':dict(sorted(Counter(x['status'] for x in records).items()))}
    return {
        'benchmark':'Athlesia E7 raw pixel grid transition-grounding pilot v0.1',
        'cases':n,'action_budget_per_arm':budget,
        'fixed_seed_schedule_sha256':hashlib.sha256(json.dumps(seeds,separators=(',',':')).encode()).hexdigest(),
        'visual_planner':result('visual_planner'),
        'greedy_local':result('greedy_local'),
        'uniform_random':result('uniform_random'),
        'shortest_path_optimal_successes':sum(x['visual_planner']['success'] and
            x['visual_planner']['actions']==x['visual_planner']['optimal_actions'] for x in cases),
        'training_demonstrations_per_case':5,
        'external_goal_coordinate_provided':True,
        'action_identifiers_per_case_remapped':True,
        'pixel_palette_per_case_remapped':True,
        'public_visual_ontology':'opaque integer grids only; tutorial move effects and wall collision',
        'solver_imports_simulator':False,
        'same_author_evaluator':True,
        'official_arc_agi_3':False,
        'autonomous_goal_invention':False,
        'novel_action_grammar_learned':False,
        'partial_observation_solved':False,
        'wall_ms_total':round(1000*(time.perf_counter()-start),3),
        'sample_cases':cases[:3],
    }


class VisualGroundingTests(unittest.TestCase):
    def test_learn_dynamics_without_semantic_labels(self):
        for seed in (10,11,12,13,14):
            case,_=new_case(seed)
            agent=Agent()
            agent.fit(training_demonstrations(case))
            self.assertEqual(set(agent.action_delta.values()), {(1,0),(-1,0),(0,-1),(0,1)})
            self.assertEqual(agent.obstacle_colours,{case.palette[1]})
            self.assertEqual(agent.actor_colour,case.palette[2])

    def test_new_heldout_maze_solved_from_opaque_pixels(self):
        for seed in (110,111,112,113):
            result=one_case(seed,0)
            self.assertTrue(result['visual_planner']['success'],result)
            self.assertEqual(result['visual_planner']['actions'],result['visual_planner']['optimal_actions'])

    def test_unknown_visual_role_fails_closed(self):
        case,_=new_case(211)
        agent=Agent();agent.fit(training_demonstrations(case))
        frame=[list(x) for x in World(case).observe()]
        unknown=next(c for c in range(256) if c not in case.palette)
        # Replace an ordinary floor cell with an unclassified pixel.
        p=next((r,c) for r,row in enumerate(frame) for c,v in enumerate(row)
               if v == case.palette[0] and (r,c)!=case.goal)
        frame[p[0]][p[1]]=unknown
        with self.assertRaisesRegex(CannotIdentify,'Unclassified'):
            agent.plan(tuple(map(tuple,frame)),case.goal)

    def test_action_semantics_change_falsifies_execution(self):
        case,_=new_case(313)
        agent=Agent();agent.fit(training_demonstrations(case))
        class SabotagedPort:
            def __init__(self,world):
                self.world=world
            def observe(self):
                return self.world.observe()
            def send_action(self,token):
                # Unexpected no-op; refuses rather than claim success.
                return self.world.observe()
        with self.assertRaisesRegex(CannotIdentify,'falsified'):
            agent.execute(SabotagedPort(World(case)),case.goal)

    def test_ambiguous_training_cannot_identify(self):
        case,_=new_case(415)
        traces=list(training_demonstrations(case))
        agent=Agent()
        with self.assertRaises(CannotIdentify):
            agent.fit(tuple(traces[:4]))

    def test_seed_schedule_reproducible(self):
        a=benchmark(3,budget=30)
        b=benchmark(3,budget=30)
        for key in ('fixed_seed_schedule_sha256','visual_planner','greedy_local','uniform_random'):
            self.assertEqual(a[key],b[key])

    def test_agent_has_no_evaluator_import(self):
        source=Path(__file__).with_name('agent.py').read_text()
        self.assertNotIn('import evaluator',source)
        self.assertNotIn('from evaluator import',source)


if __name__=='__main__':
    ap=argparse.ArgumentParser()
    ap.add_argument('--cases',type=int,default=100)
    ap.add_argument('--budget',type=int,default=BUDGET)
    ap.add_argument('--output',type=Path,default=Path('results/e7_raw_grid100.json'))
    args=ap.parse_args()
    result=benchmark(args.cases,args.budget)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf8')
    print(json.dumps(result,indent=2))
    print('E7_RAW_GRID_PILOT_TESTED_NOT_COMMITTED')

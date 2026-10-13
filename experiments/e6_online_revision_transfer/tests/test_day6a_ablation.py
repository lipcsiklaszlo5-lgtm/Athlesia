"""Athlesia E6 Day 6a -- paired active-vs-uniform-random intervention ablation.

Experiments share the SAME author-developed World fixture and the SAME initial
public evidence. This is NOT an independent environment generator, nor an
official ARC-AGI-3 score. Experimental-law truth is used by the evaluator
only to score the final inferred model, never for selecting random probes.
Random may fail; failures always remain in the denominator.
"""
from __future__ import annotations

import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import statistics
import time
import unittest

from athlesia.environment import World, WorldConfig
from athlesia.learner import Agent, CannotIdentify, Literal
from athlesia.protocol import OBSERVE, Trace
from test_day5a_epochs import trained_identified, make_trace

# Budget is specified BEFORE inspection of outcomes. Both arms receive it.
TRIAL_BUDGET = 12
SEED_SCHEDULE = 0xD6A_2026
BASELINE_RANDOM_SEED = 0xA61A_C0DE


def start_paired_agents(seed: int, channels: int, hidden_new_guard: tuple[int, ...]):
    if hidden_new_guard == (0,) or not 3 <= channels <= 5:
        raise ValueError('Invalid paired experiment specification')
    first, _ = make_trace(
        seed, channels, hidden_new_guard,
        (True,) + (False,) * (channels - 1), 100
    )
    a = trained_identified(seed, channels, (0,))
    b = trained_identified(seed, channels, (0,))
    ra = a.observe_mechanism_shift(first)
    rb = b.observe_mechanism_shift(first)
    assert ra['status'] == rb['status'] == 'quarantined'
    assert a.memory_digest() == b.memory_digest(), 'arms must start from identical memory'
    return a, b


def uniform_random_probe(agent: Agent, world: World, rng: random.Random) -> dict:
    """Same real calibration/probe interface as active selection, random policy.

    This policy uses only the public scene, current learned action's effects,
    and its known Boolean feature-role vocabulary. It never reads world.guards,
    world._guard_ids or any other private environment law.
    """
    obs = world.send_action(OBSERVE)
    binding = agent.binding(obs.scene)
    action = next((role for role, node in binding.items() if node == world.primary), None)
    candidate = agent.mechanism_candidates.get(action)
    if candidate is None or action not in agent.pending_actions:
        raise CannotIdentify('Random baseline has no quarantined mechanism')
    if any(episode == obs.episode for episode, _ in candidate.evidence):
        raise CannotIdentify('Random baseline requires a fresh episode')
    effect_roles = {item.role for item in agent.rules[action].effects}
    toggles = tuple(role for role in agent.features if role not in effect_roles)
    proposed = tuple(Literal(role, bool(rng.getrandbits(1))) for role in toggles)
    prepared = 0
    for literal in proposed:
        before_sequence = obs.sequence
        obs = agent.set_literal(world, obs, binding, literal)
        prepared += obs.sequence - before_sequence
    after = agent.checked_action(world, obs, world.primary)
    update = agent.observe_mechanism_shift(Trace(obs, world.primary, after))
    return {**update, 'probe_actions': 1, 'preparation_actions': prepared}


def run_arm(agent: Agent, seed: int, channels: int, hidden_new_guard: tuple[int, ...],
            policy: str, random_seed: int, budget: int = TRIAL_BUDGET) -> dict:
    rng = random.Random(random_seed)
    probes = 0
    calibrations = 0
    status = 'budget_exhausted'
    last_world = None
    error = None
    for episode in range(101, 101 + budget):
        world = World(WorldConfig(seed, hidden_new_guard, (False,) * channels, episode))
        last_world = world
        try:
            result = (agent.run_active_guard_probe(world, world.primary)
                      if policy == 'active'
                      else uniform_random_probe(agent, world, rng))
        except CannotIdentify as exc:
            status = 'refused'
            error = type(exc).__name__ + ': ' + str(exc)[:160]
            break
        probes += int(result['probe_actions'])
        calibrations += int(result['preparation_actions'])
        if result['status'] == 'committed':
            status = 'committed'
            break
    correct = bool(status == 'committed' and last_world is not None and
                   agent.rule.guards == (
                       Literal(agent.roles[last_world.fluents[hidden_new_guard[0]]], True),
                   ))
    # No successful result is awarded just for reporting commit.
    if status == 'committed' and not correct:
        status = 'wrong_commit'
    return {
        'status': status, 'correct': correct,
        'probes': probes, 'calibration_actions': calibrations,
        'total_real_actions': probes + calibrations, 'error': error,
    }


def one_pair(seed: int, channels: int, guard: tuple[int, ...], index: int,
             budget: int = TRIAL_BUDGET) -> dict:
    active_agent, random_agent = start_paired_agents(seed, channels, guard)
    a = run_arm(active_agent, seed, channels, guard, 'active',
                BASELINE_RANDOM_SEED ^ seed ^ index, budget)
    b = run_arm(random_agent, seed, channels, guard, 'random',
                BASELINE_RANDOM_SEED ^ seed ^ index, budget)
    return {'index': index, 'channels': channels, 'active': a, 'random': b}


def percentile(data: list[int], pct: int):
    if not data:
        return None
    seq = sorted(data)
    return seq[(len(seq)-1) * pct // 100]


def summarize_arm(data: list[dict], budget: int) -> dict:
    won = [item for item in data if item['correct']]
    costs = [item['probes'] for item in won]
    actions = [item['total_real_actions'] for item in won]
    # Capped failures are included in full-population metrics, NOT silently
    # excluded to make a slower method appear artificially efficient.
    capped_probes = [item['probes'] if item['correct'] else budget + 1 for item in data]
    return {
        'successes': len(won), 'failures': len(data)-len(won),
        'probe_p50_success_only': percentile(costs, 50),
        'probe_p95_success_only': percentile(costs, 95),
        'actions_p50_success_only': percentile(actions, 50),
        'actions_p95_success_only': percentile(actions, 95),
        'capped_probe_mean_all_cases': round(statistics.mean(capped_probes), 4),
        'probe_actions_all_cases': sum(item['probes'] for item in data),
        'calibration_actions_all_cases': sum(item['calibration_actions'] for item in data),
        'total_real_actions_all_cases': sum(item['total_real_actions'] for item in data),
        'failure_status_counts': {
            status: sum(item['status'] == status for item in data)
            for status in sorted(set(item['status'] for item in data if not item['correct']))
        },
    }


def experiment(cases: int, budget: int = TRIAL_BUDGET) -> dict:
    if not 1 <= cases <= 250 or not 1 <= budget <= 20:
        raise ValueError('cases must be 1..250 and budget 1..20')
    rng = random.Random(SEED_SCHEDULE)
    pairs=[]
    seeds=[]
    start=time.perf_counter_ns()
    for i in range(cases):
        seed=rng.randrange(1, 2**40)
        channels=3+(i % 2)
        guard=(1 + ((i // 2) % (channels-1)),)
        seeds.append([seed, channels, *guard])
        pair=one_pair(seed,channels,guard,i,budget)
        pairs.append(pair)
        if (i+1) % 10 == 0:
            print(f'DAY6A_ABLATION {i+1}/{cases}',flush=True)
    a=[item['active'] for item in pairs]
    b=[item['random'] for item in pairs]
    both=[p for p in pairs if p['active']['correct'] and p['random']['correct']]
    comparison={
        'both_success': len(both),
        'active_only_success': sum(p['active']['correct'] and not p['random']['correct'] for p in pairs),
        'random_only_success': sum(p['random']['correct'] and not p['active']['correct'] for p in pairs),
        'both_failed': sum(not p['active']['correct'] and not p['random']['correct'] for p in pairs),
        'paired_success_probe_wins_active': sum(p['active']['probes'] < p['random']['probes'] for p in both),
        'paired_success_probe_ties': sum(p['active']['probes'] == p['random']['probes'] for p in both),
        'paired_success_probe_wins_random': sum(p['active']['probes'] > p['random']['probes'] for p in both),
        'paired_success_total_action_wins_active': sum(p['active']['total_real_actions'] < p['random']['total_real_actions'] for p in both),
        'paired_success_total_action_ties': sum(p['active']['total_real_actions'] == p['random']['total_real_actions'] for p in both),
        'paired_success_total_action_wins_random': sum(p['active']['total_real_actions'] > p['random']['total_real_actions'] for p in both),
    }
    seed_hash=hashlib.sha256(json.dumps(seeds,separators=(',',':')).encode()).hexdigest()
    return {
        'benchmark': 'Athlesia E6 Day6a paired active-vs-random ablation v0.1',
        'cases':cases, 'probe_budget_per_arm':budget,
        'fixed_seed_schedule_sha256':seed_hash,
        'active':summarize_arm(a,budget),'uniform_random':summarize_arm(b,budget),
        'paired_comparison':comparison,
        'sample_cases':pairs[:3],
        'wall_ms_total':round((time.perf_counter_ns()-start)/1e6,4),
        'evaluation_type':'paired in-process deterministic fixture; same author-developed World',
        'independent_generator':False,'independent_scorer':False,
        'official_arc_agi_3':False,'autonomous_episode_reset':False,
        'notes':'All failed/refused/wrong-commit cases remain in denominator; active superiority is NOT asserted.'
    }


class Day6aTests(unittest.TestCase):
    def test_identical_initial_state_both_arms(self):
        a,b=start_paired_agents(90101,3,(1,))
        self.assertEqual(a.memory_digest(),b.memory_digest())
        self.assertEqual(len(a.mechanism_archive),0)
        self.assertEqual(len(b.mechanism_archive),0)

    def test_random_policy_does_not_consult_hidden_guards(self):
        # A private guard is never read by random_probe. Only advertised
        # interventions and actual state transitions are admitted.
        a,b=start_paired_agents(90103,3,(1,))
        world=World(WorldConfig(90103,(1,),(False,)*3,101))
        before=world.send_action(OBSERVE)
        result=uniform_random_probe(b,world,random.Random(22))
        self.assertEqual(result['probe_actions'],1)
        self.assertEqual(world.send_action(OBSERVE).sequence,
                         before.sequence+result['preparation_actions']+1)
        self.assertIn(result['status'],('quarantined','committed'))

    def test_both_policies_account_for_actions_and_failures(self):
        pair=one_pair(90107,3,(1,),0,budget=1)
        for key in ('active','random'):
            x=pair[key]
            self.assertLessEqual(x['probes'],1)
            self.assertEqual(x['total_real_actions'],x['probes']+x['calibration_actions'])
            self.assertFalse(x['correct'])  # Day5a requires >=3 distinct episodes.

    def test_reproducible_schedule_and_outcomes(self):
        first=experiment(3, budget=9)
        second=experiment(3, budget=9)
        for key in ('fixed_seed_schedule_sha256','active','uniform_random','paired_comparison'):
            self.assertEqual(first[key],second[key])


if __name__ == '__main__':
    ap=argparse.ArgumentParser()
    ap.add_argument('--cases',type=int,default=100)
    ap.add_argument('--budget',type=int,default=TRIAL_BUDGET)
    ap.add_argument('--output',type=Path,default=Path('results/day6a_ablation100.json'))
    args=ap.parse_args()
    report=experiment(args.cases,args.budget)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,indent=2))
    assert report['cases']==args.cases
    assert report['active']['successes']+report['active']['failures']==args.cases
    assert report['uniform_random']['successes']+report['uniform_random']['failures']==args.cases
    assert sum(report['paired_comparison'][k] for k in
               ('both_success','active_only_success','random_only_success','both_failed'))==args.cases
    print('DAY6A_PAIRED_ABLATION_COMPLETED')

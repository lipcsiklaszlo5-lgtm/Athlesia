"""Deterministic Day-4 *in-process fixture* stress and falsifying controls.

This is NOT a new independent task generator or an ARC-AGI-3 evaluation.
The private fixture is only used by this evaluator/test module, not the solver.
Run as: python3 tests/test_day4_stress.py --cases 100 --output results/day4_stress100.json
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import random
import resource
import statistics
import time
import unittest

from athlesia.environment import World, WorldConfig
from athlesia.graph import induced_embedding, MatchStatus
from athlesia.learner import Agent, CannotIdentify
from athlesia.protocol import OBSERVE, Trace
from test_day4_topology import ExpandedWorld, DuplicateWorld


def percentile(series: list[float], q: float) -> float:
    ordered = sorted(series)
    return ordered[max(0, min(len(ordered)-1, int(__import__('math').ceil(len(ordered)*q))-1))]


def train(seed: int, channels: int, guards: tuple[int, ...]) -> Agent:
    traces = []
    for episode in range(3):
        world = World(WorldConfig(seed, guards, (True,)*channels, episode, True))
        before = world.send_action(OBSERVE)
        traces.append(Trace(before, world.primary, world.send_action(world.primary)))
    agent = Agent()
    agent.fit(tuple(traces))
    trap = World(WorldConfig(seed, guards, (False,)*channels, 3, True))
    outcome = agent.revise(trap)
    if not outcome['identified_within_declared_class'] or len(agent.version_space) != 1:
        raise AssertionError('Training/revision did not identify one model')
    return agent


def expected_rejection(agent: Agent, target: object, label: str) -> None:
    prior = agent.memory_digest()
    try:
        agent.transfer(target)  # type: ignore[arg-type]
    except CannotIdentify:
        if agent.memory_digest() != prior:
            raise AssertionError(f'{label}: rejection mutated learned memory')
    else:
        raise AssertionError(f'{label}: invalid target was incorrectly accepted')


def run_stress(cases: int = 100) -> dict[str, object]:
    if not 1 <= cases <= 1000:
        raise ValueError('cases must be 1..1000')
    start_cpu = time.process_time_ns()
    start_wall = time.perf_counter_ns()
    elapsed: list[float] = []
    counts = dict(expanded_transfer=0, changed_mechanism_rejected=0,
                  unexpected_effect_rejected=0, ambiguous_embedding_rejected=0)
    samples: list[dict[str, object]] = []
    generator = random.Random(0xA7D4_2040)
    for i in range(cases):
        begin = time.perf_counter_ns()
        seed = generator.randrange(1, 2**40)
        channels = 3 + (i % 3)
        guards = ((i % channels,), (0,channels-1))[i % 4 == 0]
        other = next(j for j in range(channels) if j not in guards)
        agent = train(seed, channels, guards)
        old_memory = agent.memory_digest()

        # Positive: new object, fluent, goal and reset action added to the graph.
        target_world = World(WorldConfig(seed + 1_000_003, guards, (False,)*channels, 50, True))
        target = ExpandedWorld(target_world)
        match = induced_embedding(agent.template, target.scene)
        if match.status != MatchStatus.UNIQUE:
            raise AssertionError(f'case {i}: embedding was {match.status}')
        result = agent.transfer(target)
        if result['first_task_attempt_success'] is not True or not target_world._values[target_world.goal]:
            raise AssertionError(f'case {i}: false positive goal result')
        if agent.memory_digest() != old_memory:
            raise AssertionError(f'case {i}: positive transfer mutated learned memory')
        counts['expanded_transfer'] += 1

        # Negative: identical graph family, different private mechanism.
        changed = ExpandedWorld(World(WorldConfig(seed + 1_000_004, (other,),
                              (False,)*channels, 51, True)))
        expected_rejection(agent, changed, 'changed mechanism')
        counts['changed_mechanism_rejected'] += 1

        # Negative: previously unseen goal-space fluent changes as a side effect.
        changed_extra = ExpandedWorld(World(WorldConfig(seed + 1_000_005, guards,
                                      (False,)*channels, 52, True)), unexpected=True)
        expected_rejection(agent, changed_extra, 'unmapped side effect')
        counts['unexpected_effect_rejected'] += 1

        # Negative: two structurally identical copies; no privileged token tie-break.
        duplicate = DuplicateWorld(World(WorldConfig(seed + 1_000_006, guards,
                                   (False,)*channels, 53, True)))
        if induced_embedding(agent.template, duplicate.scene).status != MatchStatus.AMBIGUOUS:
            raise AssertionError(f'case {i}: duplicate was not ambiguous')
        expected_rejection(agent, duplicate, 'ambiguous topology')
        counts['ambiguous_embedding_rejected'] += 1
        elapsed.append((time.perf_counter_ns()-begin)/1_000_000)
        if i < 5:
            samples.append({'case':i, 'channels':channels, 'guards':list(guards),
                            'template_nodes':len(agent.template.nodes),
                            'expanded_nodes':len(target.scene.nodes),
                            'embedding_search_states':match.search_states,
                            'preparation_actions':result['preparation_actions']})
        if (i+1) % 10 == 0:
            print(f'DAY4_STRESS_COMPLETED {i+1}/{cases}', flush=True)
    base = Path(__file__).parents[1]
    digest = hashlib.sha256()
    for path in ('athlesia/graph.py','athlesia/learner.py',
                 'tests/test_day4_topology.py','tests/test_day4_stress.py'):
        digest.update(path.encode()+b'\0'+(base/path).read_bytes())
    return {
        'benchmark':'Athlesia E6 Day 4 topology-extension fixture stress v0.1',
        'test_family':'instrumented Boolean fixture; 3..5 channels, 1..2 positive guards',
        'evaluator':'in-process deterministic fixture, not independently developed',
        'not_official_arc_agi_3':True,
        'solver_source_sha256_subset':digest.hexdigest(),
        'cases':cases, 'checks':counts, 'sample_cases':samples,
        'total_cpu_ms':(time.process_time_ns()-start_cpu)/1e6,
        'total_wall_ms':(time.perf_counter_ns()-start_wall)/1e6,
        'wall_ms_per_case_p50':percentile(elapsed,0.5),
        'wall_ms_per_case_p95':percentile(elapsed,0.95),
        'process_peak_rss_bytes_upper':resource.getrusage(resource.RUSAGE_SELF).ru_maxrss*1024,
        'limitations':'Does not test novel goals, new rule grammars, missing observation, subprocess isolation, or unseen non-fixture mechanisms.'
    }


class Day4StressSmoke(unittest.TestCase):
    def test_small_seeded_topology_stress(self) -> None:
        summary = run_stress(6)
        self.assertEqual(summary['checks']['expanded_transfer'],6)
        self.assertEqual(summary['checks']['changed_mechanism_rejected'],6)
        self.assertEqual(summary['checks']['unexpected_effect_rejected'],6)
        self.assertEqual(summary['checks']['ambiguous_embedding_rejected'],6)


if __name__ == '__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--cases',type=int,default=100)
    parser.add_argument('--output',type=Path,default=Path('results/day4_stress100.json'))
    args=parser.parse_args()
    report=run_stress(args.cases)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps(report,indent=2,ensure_ascii=False))
    if any(v!=args.cases for v in report['checks'].values()):
        raise SystemExit('FAIL: incomplete stress coverage')
    print('DAY4_STRESS_PASSED')

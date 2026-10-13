"""Held-out, separately implemented symbolic environment: active vs random.

Tests guard polarity/conjunction shifts and evaluates exact learned rules.
Same PUBLIC affordance schema as Day6a; separate simulator/oracle, not a
truly third-party author or new ARC-AGI-3 domain. No World import here.
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

from external_world_v1 import ExternalWorld
from athlesia.learner import Agent, CannotIdentify, Literal
from athlesia.protocol import OBSERVE, Trace

TRIAL_BUDGET = 12
SCHEDULE_SEED = 0x6B20_2026
RANDOM_ARM_SEED = 0x18_5D_6B
OLD_LAW = ((0, True),)


def new_law(family: int) -> tuple[tuple[int, bool], ...]:
    return (
        ((1, True),),
        ((1, False),),
        ((1, True), (2, False)),
        ((1, False), (2, True)),
    )[family % 4]


def trace(world: ExternalWorld) -> Trace:
    before = world.send_action(OBSERVE)
    return Trace(before, world.primary, world.send_action(world.primary))


def old_training(seed: int, channels: int) -> Agent:
    demos = []
    for episode, bits in enumerate(itertools.product((False, True), repeat=channels)):
        w = ExternalWorld(seed, channels, OLD_LAW, bits, episode)
        demos.append(trace(w))
    agent = Agent()
    agent.fit(tuple(demos))
    if len(agent.version_space) != 1 or agent.rule.guards != (Literal(agent.roles[w.fluents[0]], True),):
        raise AssertionError('Old mechanism not exactly identified from independent world')
    return agent


def mismatch_flags(channels: int, law: tuple[tuple[int, bool], ...]) -> tuple[bool, ...]:
    """Evaluator chooses a contradiction, never passes the law to the agent."""
    for flags in itertools.product((False, True), repeat=channels):
        if flags[0] and not all(flags[i] is polarity for i, polarity in law):
            return flags
    raise ValueError('No old/new law separating witness')


def pair_agents(seed: int, channels: int, law: tuple[tuple[int, bool], ...]):
    a, b = old_training(seed, channels), old_training(seed, channels)
    first_world = ExternalWorld(seed, channels, law, mismatch_flags(channels, law), 100)
    anomalous = trace(first_world)
    x = a.observe_mechanism_shift(anomalous)
    y = b.observe_mechanism_shift(anomalous)
    if x['status'] != 'quarantined' or y['status'] != 'quarantined':
        raise AssertionError('Both arms must be quarantined from the same evidence')
    if a.memory_digest() != b.memory_digest():
        raise AssertionError('Policy arms started from different memory states')
    return a, b


def uniform_probe(agent: Agent, world: ExternalWorld, rng: random.Random) -> dict:
    """Frozen Day6a uniform-bit baseline: public state/actions only."""
    obs = world.send_action(OBSERVE)
    binding = agent.binding(obs.scene)
    role = next((r for r, node in binding.items() if node == world.primary), None)
    candidate = agent.mechanism_candidates.get(role)
    if candidate is None or role not in agent.pending_actions:
        raise CannotIdentify('No quarantined mechanism for random probe')
    if any(episode == obs.episode for episode, _ in candidate.evidence):
        raise CannotIdentify('Repeated environment episode')
    effect_roles = {e.role for e in agent.rules[role].effects}
    toggles = tuple(r for r in agent.features if r not in effect_roles)
    proposed = tuple(Literal(r, bool(rng.getrandbits(1))) for r in toggles)
    preps = 0
    for lit in proposed:
        last = obs.sequence
        obs = agent.set_literal(world, obs, binding, lit)
        preps += obs.sequence - last
    after = agent.checked_action(world, obs, world.primary)
    result = agent.observe_mechanism_shift(Trace(obs, world.primary, after))
    return {**result, 'probe_actions': 1, 'preparation_actions': preps}


def evaluate_arm(agent: Agent, seed: int, channels: int,
                 law: tuple[tuple[int, bool], ...], mode: str,
                 rng_seed: int, budget: int = TRIAL_BUDGET) -> dict:
    rng = random.Random(rng_seed)
    probes = 0
    calibration = 0
    status = 'budget_exhausted'
    error = None
    w = None
    for episode in range(101, 101 + budget):
        w = ExternalWorld(seed, channels, law, (False,) * channels, episode)
        try:
            result = (agent.run_active_guard_probe(w, w.primary)
                      if mode == 'active' else uniform_probe(agent, w, rng))
        except CannotIdentify as exc:
            status = 'refused'
            error = str(exc)[:160]
            break
        probes += int(result['probe_actions'])
        calibration += int(result['preparation_actions'])
        if result['status'] == 'committed':
            status = 'committed'
            break
    expected = frozenset((agent.roles[w.fluents[i]], polarity) for i, polarity in law)
    actual = frozenset((g.role, g.value) for g in agent.rule.guards)
    correct = bool(status == 'committed' and agent.rule.effects and actual == expected)
    if status == 'committed' and not correct:
        status = 'wrong_commit'
    return {'status': status, 'correct': correct, 'probes': probes,
            'calibration_actions': calibration, 'real_actions': probes + calibration,
            'error': error}


def pct(items: list[int], p: int):
    if not items:
        return None
    seq = sorted(items)
    return seq[(len(seq) - 1) * p // 100]


def summarize_arm(items: list[dict], budget: int) -> dict:
    good = [x for x in items if x['correct']]
    return {
        'successes': len(good), 'failures': len(items) - len(good),
        'probe_p50_success_only': pct([x['probes'] for x in good], 50),
        'probe_p95_success_only': pct([x['probes'] for x in good], 95),
        'probes_executed_all': sum(x['probes'] for x in items),
        'calibrations_executed_all': sum(x['calibration_actions'] for x in items),
        'real_actions_executed_all': sum(x['real_actions'] for x in items),
        'failure_penalized_probe_mean': round(statistics.mean(
            [x['probes'] if x['correct'] else budget + 1 for x in items]), 4),
        'failure_reasons': {key: sum(x['status'] == key for x in items)
                            for key in sorted({x['status'] for x in items if not x['correct']})},
    }


def paired(cases: int = 100, budget: int = TRIAL_BUDGET) -> dict:
    if not 1 <= cases <= 200 or not 1 <= budget <= 20:
        raise ValueError('Expected cases 1..200, budget 1..20')
    rng = random.Random(SCHEDULE_SEED)
    rows = []
    schedule = []
    start = time.perf_counter_ns()
    for i in range(cases):
        seed = rng.randrange(1, 2**40)
        channels = 3 + ((i // 4) % 3)
        family = i % 4
        law = new_law(family)
        schedule.append([seed, channels, [[idx, pol] for idx, pol in law]])
        a, b = pair_agents(seed, channels, law)
        active = evaluate_arm(a, seed, channels, law, 'active', RANDOM_ARM_SEED ^ seed ^ i, budget)
        control = evaluate_arm(b, seed, channels, law, 'random', RANDOM_ARM_SEED ^ seed ^ i, budget)
        rows.append({'family': family, 'channels': channels, 'active': active, 'random': control})
        if (i + 1) % 10 == 0:
            print(f'DAY6B_EXTERNAL {i+1}/{cases}', flush=True)
    both = [row for row in rows if row['active']['correct'] and row['random']['correct']]
    win = lambda k: {
        'active': sum(row['active'][k] < row['random'][k] for row in both),
        'tie': sum(row['active'][k] == row['random'][k] for row in both),
        'random': sum(row['active'][k] > row['random'][k] for row in both),
    }
    family_rows = {}
    for family in range(4):
        subset = [row for row in rows if row['family'] == family]
        family_rows[str(family)] = {
            'law_shape': ['single_positive', 'single_negative', 'two_mixed_positive_negative', 'two_mixed_negative_positive'][family],
            'cases': len(subset),
            'active_successes': sum(x['active']['correct'] for x in subset),
            'random_successes': sum(x['random']['correct'] for x in subset),
        }
    return {
        'benchmark': 'Athlesia E6 Day6b separately implemented Boolean evaluator v0.1',
        'cases': cases, 'budget_probes_per_arm': budget,
        'frozen_schedule_sha256': hashlib.sha256(json.dumps(schedule, separators=(',', ':')).encode()).hexdigest(),
        'active': summarize_arm([x['active'] for x in rows], budget),
        'uniform_random': summarize_arm([x['random'] for x in rows], budget),
        'pair_counts': {
            'both_success': len(both),
            'active_only': sum(x['active']['correct'] and not x['random']['correct'] for x in rows),
            'random_only': sum(x['random']['correct'] and not x['active']['correct'] for x in rows),
            'both_failed': sum(not x['active']['correct'] and not x['random']['correct'] for x in rows),
        },
        'paired_success_probe_comparison': win('probes'),
        'paired_success_total_action_comparison': win('real_actions'),
        'by_guard_family': family_rows,
        'sample_cases': rows[:3],
        'wall_ms_total': round((time.perf_counter_ns() - start) / 1e6, 3),
        'environment_imports_original_private_fixture': False,
        'environment_implementation_separate': True,
        'independent_authorship_or_external_audit': False,
        'public_ontology_shared_with_training': True,
        'hidden_laws_new_polarities_and_conjunctions': True,
        'official_arc_agi_3': False,
        'autonomous_episode_reset': False,
        'note': 'All failures and incorrect commits remain in the denominator; no success threshold enforced.'
    }


class Day6bExternalTests(unittest.TestCase):
    def test_external_hidden_law_not_in_public_observation(self):
        a = ExternalWorld(90701, 3, ((1, True),), (True, False, True), 100)
        b = ExternalWorld(90701, 3, ((1, False),), (True, False, True), 100)
        self.assertEqual(a.send_action(OBSERVE).to_wire(), b.send_action(OBSERVE).to_wire())

    def test_public_instrumentation_actions(self):
        w = ExternalWorld(90703, 3, ((1, False), (2, True)), (False, True, False), 10)
        before = w.send_action(OBSERVE)
        self.assertFalse(before.value(w.goal))
        w.send_action(w.down[1])
        w.send_action(w.up[2])
        self.assertTrue(w.send_action(w.primary).value(w.goal))
        self.assertFalse(w.send_action(w.clear).value(w.goal))
        self.assertEqual(w.send_action(OBSERVE).sequence, 4)

    def test_old_model_identified_without_original_generator(self):
        a = old_training(90707, 4)
        self.assertEqual(len(a.version_space), 1)

    def test_paired_arms_identical_start(self):
        a, b = pair_agents(90711, 3, new_law(2))
        self.assertEqual(a.memory_digest(), b.memory_digest())
        self.assertEqual(len(a.mechanism_archive), 0)

    def test_undersized_budget_cannot_claim_success(self):
        a, b = pair_agents(90713, 3, new_law(1))
        x = evaluate_arm(a, 90713, 3, new_law(1), 'active', 3, budget=1)
        y = evaluate_arm(b, 90713, 3, new_law(1), 'random', 3, budget=1)
        self.assertEqual(x['status'], 'budget_exhausted')
        self.assertEqual(y['status'], 'budget_exhausted')
        self.assertFalse(x['correct'])
        self.assertFalse(y['correct'])

    def test_engine_import_boundary(self):
        from pathlib import Path
        source = Path(__file__).with_name('external_world_v1.py').read_text()
        self.assertNotIn('from athlesia.environment import', source)
        self.assertNotIn('import athlesia.environment', source)

    def test_both_policy_schedules_reproducible(self):
        a = paired(3, 12)
        b = paired(3, 12)
        for k in ('frozen_schedule_sha256', 'active', 'uniform_random', 'pair_counts', 'by_guard_family'):
            self.assertEqual(a[k], b[k])


if __name__ == '__main__':
    arg = argparse.ArgumentParser()
    arg.add_argument('--cases', type=int, default=100)
    arg.add_argument('--budget', type=int, default=TRIAL_BUDGET)
    arg.add_argument('--output', type=Path, default=Path('results/day6b_external100.json'))
    opts = arg.parse_args()
    report = paired(opts.cases, opts.budget)
    opts.output.parent.mkdir(parents=True, exist_ok=True)
    opts.output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    print(json.dumps(report, indent=2))
    assert report['active']['successes'] + report['active']['failures'] == opts.cases
    assert report['uniform_random']['successes'] + report['uniform_random']['failures'] == opts.cases
    assert sum(report['pair_counts'].values()) == opts.cases
    print('DAY6B_EXTERNAL_ABLATION_COMPLETED')

"""E10 falsification: does E9 safely refuse worlds outside its assumed class?

Does not modify E6/E7/E8/E9; records unsupported ACCEPTANCES as failures,
including cases that look sound over a short audit (no completeness claim).
"""
from __future__ import annotations
import argparse
import hashlib
import json
import random
import time
import unittest
from pathlib import Path

from evaluator import new_case, training_demonstrations
from discovery import DiscoveredBridge, DiscoveryError
from challenge_world import ChallengeWorld, KINDS, SUPPORTED

SCHEDULE = 0xE10A2026
PER_KIND = 10
SUPPORTED_COUNT = 20
TOTAL = SUPPORTED_COUNT + PER_KIND*(len(KINDS)-1)


def validate_post_discovery(bridge: DiscoveredBridge, world: ChallengeWorld) -> tuple[bool, str | None, int]:
    """External auditor exercises learned effects AFTER E9 claims discovery.

    Uses ONLY the E9 public action/observe API and bridge's learned mapping.
    Scoring does not trust a returned bridge object as proof of model validity.
    This is a bounded behavioural probe, not proof of future stationarity.
    """
    actions = 0
    try:
        before = world.observe()
        if dict(bridge.pixels_to_values(before))['crossed']:
            return False, 'unexpected_actor_position', actions
        for i, ctrl in enumerate(bridge.switches):
            for tok, target in ((ctrl.off_token, False), (ctrl.on_token, True),
                                (ctrl.off_token, False)):
                previous = world.observe()
                old = dict(bridge.pixels_to_values(previous))
                after = world.send_action(tok)
                actions += 1
                now = dict(bridge.pixels_to_values(after))
                expected = dict(old)
                expected[f'switch{i}'] = target
                if now != expected:
                    return False, 'incorrect_learned_successor', actions
                # Whole frame must not mutate any unrelated visual cell.
                changed = [(r, c) for r, row in enumerate(previous)
                           for c, value in enumerate(row) if after[r][c] != value]
                if changed not in ([], [ctrl.cell]):
                    return False, 'unexplained_visual_side_effect', actions
    except (ValueError, KeyError, IndexError, RuntimeError) as exc:
        return False, type(exc).__name__, actions
    return True, None, actions


def one_case(seed: int, kind: str, idx: int) -> dict:
    case, _ = new_case(seed)
    # The fixed E9 problem law is valid; only the interface dynamics change.
    from pixel_world import HiddenSpec
    spec = HiddenSpec.create(case, ((1, True),), seed)
    world = ChallengeWorld(spec, -20, kind)
    bridge = DiscoveredBridge(training_demonstrations(case))
    accepted = False
    error = None
    audited = None
    audit_reason = None
    audit_actions = 0
    try:
        bridge.discover(world)
        accepted = True
        audited, audit_reason, audit_actions = validate_post_discovery(bridge, world)
    except (DiscoveryError, ValueError, RuntimeError, KeyError, IndexError) as exc:
        error = type(exc).__name__ + ': ' + str(exc)[:130]
    # These are independent scorer labels, never available to E9.
    supported = kind == SUPPORTED
    return {'case': idx, 'kind': kind, 'in_declared_class': supported,
            'discovery_accepted': accepted, 'post_discovery_audit_pass': audited,
            'acceptance_of_out_of_class': accepted and not supported,
            'accepted_unsound_at_audit': accepted and audited is False,
            'discovery_actions': world.real_actions - audit_actions,
            'post_audit_actions': audit_actions,
            'reason': error if error is not None else audit_reason}


def benchmark(n: int = TOTAL) -> dict:
    if not 1 <= n <= TOTAL:
        raise ValueError('Case count must be in 1..' + str(TOTAL))
    kinds = [SUPPORTED] * SUPPORTED_COUNT
    for k in KINDS[1:]:
        kinds += [k] * PER_KIND
    rng = random.Random(SCHEDULE)
    order = list(range(len(kinds)))
    rng.shuffle(order)
    schedule = [(kinds[i], rng.randrange(1, 2**49)) for i in order[:n]]
    schedule_hash = hashlib.sha256(json.dumps(schedule, separators=(',', ':')).encode()).hexdigest()
    start = time.perf_counter_ns()
    records = []
    for j, (kind, seed) in enumerate(schedule):
        records.append(one_case(seed, kind, j))
        if (j + 1) % 10 == 0:
            print(f'E10_FALSIFICATION {j+1}/{n}', flush=True)
    accepted = [r for r in records if r['discovery_accepted']]
    in_class = [r for r in records if r['in_declared_class']]
    out_class = [r for r in records if not r['in_declared_class']]
    by_type = []
    for k in KINDS:
        subset = [r for r in records if r['kind'] == k]
        by_type.append({'variant':k, 'cases':len(subset),
                        'accepted':sum(r['discovery_accepted'] for r in subset),
                        'refused':sum(not r['discovery_accepted'] for r in subset),
                        'accepted_unsound_at_audit':sum(r['accepted_unsound_at_audit'] for r in subset),
                        'discovery_actions':sum(r['discovery_actions'] for r in subset)})
    return {'benchmark':'Athlesia E10 E9 out-of-grammar affordance falsification v0.1',
            'cases':n,'schedule_sha256':schedule_hash,
            'supported_cases':len(in_class),
            'supported_accepted':sum(r['discovery_accepted'] for r in in_class),
            'supported_refused':sum(not r['discovery_accepted'] for r in in_class),
            'unsupported_cases':len(out_class),
            'unsupported_refused':sum(not r['discovery_accepted'] for r in out_class),
            'unsupported_accepted_false_positive':sum(r['discovery_accepted'] for r in out_class),
            'false_accepts_falsified_by_post_audit':sum(r['accepted_unsound_at_audit'] for r in out_class),
            'all_accepted_audited_pass':sum(r['post_discovery_audit_pass'] is True for r in accepted),
            'all_accepted_audited_fail':sum(r['post_discovery_audit_pass'] is False for r in accepted),
            'actual_discovery_actions':sum(r['discovery_actions'] for r in records),
            'post_audit_actions':sum(r['post_audit_actions'] for r in records),
            'by_variant':by_type,
            'sample_false_accepts':[r for r in records if r['acceptance_of_out_of_class']][:5],
            'sample_refusals':[r for r in records if not r['discovery_accepted']][:3],
            'unchanged_e9_discovery':True,'same_author_challenger':True,
            'e9_public_ontology_shared':True,'no_external_episode_reset_claim':True,
            'official_arc_agi_3':False,
            'wall_ms_total':round((time.perf_counter_ns()-start)/1e6,3)}

class E10Tests(unittest.TestCase):
    def test_supported_cases_remain_accepted_and_sound(self):
        for seed in range(99100,99110):
            r = one_case(seed, SUPPORTED, seed)
            self.assertTrue(r['discovery_accepted'],r)
            self.assertTrue(r['post_discovery_audit_pass'],r)

    def test_external_variant_not_visible_to_discovery(self):
        import inspect, discovery
        source = inspect.getsource(discovery)
        self.assertNotIn('challenge_world',source)
        self.assertNotIn('from pixel_world import',source)
        self.assertNotIn('import pixel_world',source)

    def test_delayed_drift_is_not_silently_scored_successful(self):
        r = one_case(98750,'late_semantic_drift',0)
        # This variant should expose a real flaw: eight training interactions
        # cannot certify a ninth action's stationarity.
        self.assertTrue(r['discovery_accepted'],r)
        self.assertFalse(r['post_discovery_audit_pass'],r)
        self.assertTrue(r['accepted_unsound_at_audit'])

    def test_bad_action_inventory_refused_without_interactions(self):
        for kind in ('inventory_missing','inventory_extra'):
            r = one_case(99123,kind,0)
            self.assertFalse(r['discovery_accepted'],r)
            self.assertEqual(r['discovery_actions'],0)

    def test_deterministic_summary(self):
        a,b=benchmark(12),benchmark(12)
        for k in ('schedule_sha256','supported_cases','supported_accepted',
                  'unsupported_refused','unsupported_accepted_false_positive','by_variant'):
            self.assertEqual(a[k],b[k])

    def test_accounting_includes_every_acceptance(self):
        out=benchmark(TOTAL)
        self.assertEqual(out['cases'],TOTAL)
        self.assertEqual(out['supported_cases']+out['unsupported_cases'],TOTAL)
        self.assertEqual(out['supported_accepted']+out['supported_refused'],out['supported_cases'])
        self.assertEqual(out['unsupported_refused']+out['unsupported_accepted_false_positive'],out['unsupported_cases'])
        self.assertEqual(out['all_accepted_audited_fail']+out['all_accepted_audited_pass'],
                         out['supported_accepted']+out['unsupported_accepted_false_positive'])
        self.assertEqual(sum(x['cases'] for x in out['by_variant']),TOTAL)

if __name__=='__main__':
    p=argparse.ArgumentParser()
    p.add_argument('--cases',type=int,default=TOTAL)
    p.add_argument('--output',type=Path,default=Path('results/e10_falsification100.json'))
    args=p.parse_args()
    report=benchmark(args.cases)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,indent=2))
    assert report['supported_accepted']+report['supported_refused']==report['supported_cases']
    assert report['unsupported_accepted_false_positive']+report['unsupported_refused']==report['unsupported_cases']
    print('E10_FALSIFICATION_PILOT_COMPLETED_WITH_FAILURES_RECORDED')

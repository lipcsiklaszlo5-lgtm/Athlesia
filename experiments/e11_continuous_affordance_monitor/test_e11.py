"""E11: continuously revalidate E9 claims against the untouched E10 adversarial suite.

This is a same-author monitored-operations research pilot. The monitor does not
pretend that a finite successful history guarantees future stability.
"""
from __future__ import annotations
import argparse
import hashlib
import inspect
import json
from pathlib import Path
import random
import time
import unittest

from evaluator import new_case, training_demonstrations
from discovery import DiscoveredBridge, DiscoveryError
from pixel_world import HiddenSpec, World
from challenge_world import ChallengeWorld, KINDS, SUPPORTED
from monitor import ContinuousAffordanceMonitor, ModelQuarantined

SCHEDULE = 0xE10A2026  # Reuse E10's frozen adversarial schedule, not cherry-picked.
PER_KIND = 10
SUPPORTED_COUNT = 20
TOTAL = SUPPORTED_COUNT + PER_KIND * (len(KINDS)-1)


def frozen_schedule():
    kinds = [SUPPORTED] * SUPPORTED_COUNT
    for kind in KINDS[1:]:
        kinds.extend([kind] * PER_KIND)
    rng = random.Random(SCHEDULE)
    order = list(range(len(kinds)))
    rng.shuffle(order)
    return [(kinds[i], rng.randrange(1,2**49)) for i in order]


def setup_world(seed, world_class, kind=None):
    case,_ = new_case(seed)
    spec = HiddenSpec.create(case,((1,True),),seed)
    world = world_class(spec,-20,kind) if kind is not None else world_class(spec,-20)
    bridge = DiscoveredBridge(training_demonstrations(case))
    return bridge,world


def one_case(kind, seed, index):
    bridge,world = setup_world(seed,ChallengeWorld,kind)
    initially_accepted = False
    caught = False
    actions = 0
    reason = None
    status = 'discovery_refused'
    discovery_actions = 0
    try:
        bridge.discover(world)
        initially_accepted = True
    except (DiscoveryError,ValueError,RuntimeError,IndexError,KeyError) as exc:
        reason = type(exc).__name__
    discovery_actions = world.real_actions
    if initially_accepted:
        monitor = ContinuousAffordanceMonitor(bridge,world)
        try:
            monitor.exercise(3)  # 12 subsequent actions, independent of challenge labels.
            status = monitor.status # always 'provisional' despite consistent witnesses
        except ModelQuarantined as exc:
            caught = True
            status = monitor.status
            reason = monitor.counterexample.reason if monitor.counterexample else str(exc)
        actions = monitor.actions
        assert monitor.confirmed_actions <= actions
    return {'case':index,'kind':kind,'in_declared_class':kind==SUPPORTED,
            'discovery_accepted':initially_accepted,
            'monitor_quarantined':caught,'status':status,'discovery_actions':discovery_actions,
            'monitor_actions':actions,'reason':reason}


class LateAfterTwentyEight(World):
    """Private NEW challenge: law changes long after E9's eight interactions.

    Explicitly outside E9 class. The monitor has no access to this type or
    to the internal real-actions count. The benchmark checks actual behavior.
    """
    def send_action(self, token):
        if (self.real_actions >= 28 and
                token == self._spec.control_pairs[0][0]):
            self.real_actions += 1
            self.sequence += 1
            return self.observe()  # physical off action silently stops working
        return super().send_action(token)


def long_delayed_case(seed):
    bridge,world = setup_world(seed,LateAfterTwentyEight)
    bridge.discover(world)
    monitor = ContinuousAffordanceMonitor(bridge,world)
    # First 8 monitored actions are still good, but only PROVISIONAL.
    monitor.exercise(2)
    provisional = (monitor.status=='provisional' and monitor.actions==8)
    try:
        monitor.exercise(12)
    except ModelQuarantined:
        pass
    return {'provisional_before_late_change':provisional,
            'quarantined_after_violation':monitor.status=='quarantined',
            'monitor_actions':monitor.actions,
            'reason':monitor.counterexample.reason if monitor.counterexample else None}


def benchmark(n=TOTAL, long_cases=20):
    if not 1<=n<=TOTAL or not 1<=long_cases<=25:
        raise ValueError('bounded benchmark ranges required')
    schedule = frozen_schedule()[:n]
    schedule_hash = hashlib.sha256(json.dumps(schedule,separators=(',',':')).encode()).hexdigest()
    started = time.perf_counter_ns()
    rows = []
    for i,(kind,seed) in enumerate(schedule):
        rows.append(one_case(kind,seed,i))
        if (i+1)%10==0:
            print(f'E11_MONITORED {i+1}/{n}',flush=True)
    long_samples = [long_delayed_case(115000+i) for i in range(long_cases)]
    supported=[x for x in rows if x['in_declared_class']]
    unsupported=[x for x in rows if not x['in_declared_class']]
    initially_accepted=[x for x in unsupported if x['discovery_accepted']]
    by_kind=[]
    for kind in KINDS:
        subset=[x for x in rows if x['kind']==kind]
        by_kind.append({'variant':kind,'cases':len(subset),
            'initially_accepted':sum(x['discovery_accepted'] for x in subset),
            'quarantined_during_monitoring':sum(x['monitor_quarantined'] for x in subset),
            'discovery_actions':sum(x['discovery_actions'] for x in subset),
            'monitor_actions':sum(x['monitor_actions'] for x in subset)})
    return {'benchmark':'Athlesia E11 ongoing affordance hypothesis monitoring v0.1',
        'cases':n,'e10_frozen_schedule_sha256':schedule_hash,
        'supported_cases':len(supported),
        'supported_initially_accepted':sum(x['discovery_accepted'] for x in supported),
        'supported_quarantined':sum(x['monitor_quarantined'] for x in supported),
        'supported_remained_provisional':sum(x['status']=='provisional' for x in supported),
        'unsupported_cases':len(unsupported),
        'unsupported_refused_at_discovery':sum(not x['discovery_accepted'] for x in unsupported),
        'unsupported_accepted_provisionally':len(initially_accepted),
        'unsupported_quarantined_during_monitoring':sum(x['monitor_quarantined'] for x in initially_accepted),
        'unsupported_remained_provisional_within_window':sum(x['status']=='provisional' for x in initially_accepted),
        'discovery_actions_all_cases':sum(x['discovery_actions'] for x in rows),
        'post_discovery_monitored_actions_all_cases':sum(x['monitor_actions'] for x in rows),
        'by_variant':by_kind,
        'new_delayed_change_after_28':{
          'cases':len(long_samples),
          'provisional_before_change':sum(x['provisional_before_late_change'] for x in long_samples),
          'subsequently_quarantined':sum(x['quarantined_after_violation'] for x in long_samples),
          'monitored_actions':sum(x['monitor_actions'] for x in long_samples),
          'sample':long_samples[:3]},
        'historical_e10_false_accepts_unchanged':13,
        'new_certifications_of_future_stationarity':0,
        'private_variant_read_by_monitor':False,
        'any_finite_history_proves_stationarity':False,
        'unchanged_e6_e7_e8_e9_e10':True,
        'same_author_test':True,'official_arc_agi_3':False,
        'sample_initial_false_accepts':[x for x in initially_accepted][:4],
        'wall_ms_total':round((time.perf_counter_ns()-started)/1e6,3)}


class E11Tests(unittest.TestCase):
    def test_private_variant_not_imported_by_monitor(self):
        import monitor
        src=inspect.getsource(monitor)
        for forbidden in ('import challenge_world','import pixel_world','import test_e10'):
            self.assertNotIn(forbidden,src)

    def test_monitor_stays_provisional_on_supported_world(self):
        bridge,world=setup_world(115100,World)
        bridge.discover(world)
        m=ContinuousAffordanceMonitor(bridge,world)
        m.exercise(4)
        self.assertEqual(m.status,'provisional')
        self.assertEqual(m.confirmed_actions,16)
        self.assertIsNone(m.counterexample)

    def test_delayed_variant_falsified_and_quarantined(self):
        for seed in (115102,115103,115104):
            r=long_delayed_case(seed)
            self.assertTrue(r['provisional_before_late_change'],r)
            self.assertTrue(r['quarantined_after_violation'],r)

    def test_quarantine_prevents_further_actions(self):
        bridge,world=setup_world(115105,ChallengeWorld,'late_semantic_drift')
        bridge.discover(world)
        m=ContinuousAffordanceMonitor(bridge,world)
        with self.assertRaises(ModelQuarantined):
            m.exercise(5)
        count=world.real_actions
        with self.assertRaises(ModelQuarantined):
            m.execute(bridge.switches[0].on_token)
        self.assertEqual(world.real_actions,count)

    def test_e10_false_accept_not_hidden(self):
        r=benchmark(110,long_cases=3)
        self.assertEqual(r['unsupported_accepted_provisionally'],13)
        self.assertEqual(r['supported_initially_accepted'],20)
        self.assertEqual(r['unsupported_refused_at_discovery'],77)
        self.assertEqual(r['supported_remained_provisional'],20)

    def test_fixed_schedule_and_metrics_reproducible(self):
        a=benchmark(15,long_cases=3)
        b=benchmark(15,long_cases=3)
        for key in ('e10_frozen_schedule_sha256','supported_cases','unsupported_cases','by_variant','new_delayed_change_after_28'):
            self.assertEqual(a[key],b[key])


if __name__=='__main__':
    p=argparse.ArgumentParser()
    p.add_argument('--cases',type=int,default=110)
    p.add_argument('--long-cases',type=int,default=20)
    p.add_argument('--output',type=Path,default=Path('results/e11_monitor110.json'))
    args=p.parse_args()
    report=benchmark(args.cases,args.long_cases)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,indent=2))
    assert report['supported_initially_accepted'] <= report['supported_cases']
    assert report['supported_remained_provisional']+report['supported_quarantined'] <= report['supported_initially_accepted']
    assert report['unsupported_refused_at_discovery']+report['unsupported_accepted_provisionally']==report['unsupported_cases']
    print('E11_CONTINUOUS_MONITOR_PILOT_TESTED_NOT_COMMITTED')

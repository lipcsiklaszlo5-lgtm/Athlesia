"""E9 pilot: public action enumeration -> pixel control discovery -> real E6 revision.

Same-author deterministic grammar, externally generated fresh episodes. This is
NOT arbitrary affordance/goal discovery, independent validation, or ARC-AGI-3.
"""
from __future__ import annotations
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import time
import unittest

from evaluator import new_case,training_demonstrations
from athlesia.learner import Agent as CausalAgent, CannotIdentify
from athlesia.protocol import OBSERVE, Trace
from discovery import DiscoveredBridge, PublicAdapter, DiscoveryError
from pixel_world import HiddenSpec, World

BUDGET=12
SCHEDULE=0xE9A2026
LAWS=(((1,True),),((1,False),),((0,True),(1,True)),((0,False),(1,True)))

def setup(seed:int,law):
    case,_=new_case(seed)
    spec=HiddenSpec.create(case,law,seed)
    bridge=DiscoveredBridge(training_demonstrations(case))
    world=World(spec,-20)
    bridge.discover(world)
    assert bridge.exploration_actions==8
    assert world.real_actions==8
    assert len(bridge.switches)==2 and bridge.gate==spec.gate
    # Scorer validates these properties; neither is read by bridge code.
    return spec,bridge

def configured_port(bridge,spec,episode,assignment):
    port=PublicAdapter(bridge,World(spec,episode))
    obs=port.send_action(OBSERVE)
    for i,value in enumerate(assignment):
        if dict(obs.values)[f'switch{i}']!=value:
            switch=bridge.switches[i]
            obs=port.send_action(switch.on_token if value else switch.off_token)
    return port,obs

def establish_old(bridge,spec):
    from dataclasses import replace
    old_spec=replace(spec,law=((0,True),))
    demos=[]
    for episode,flags in enumerate(itertools.product((False,True),repeat=2),10):
        port=PublicAdapter(bridge,World(old_spec,episode,flags))
        before=port.send_action(OBSERVE)
        after=port.send_action(bridge.crossing_action)
        demos.append(Trace(before,bridge.crossing_action,after))
    agent=CausalAgent()
    agent.fit(tuple(demos))
    assert len(agent.version_space)==1
    return agent

def detect_drift(bridge,spec,agent):
    """Real observed contradiction selected without consulting the new law."""
    actions=0
    probes=0
    for index,bits in enumerate(itertools.product((False,True),repeat=2)):
        port,obs=configured_port(bridge,spec,100+index,bits)
        expected=dict(agent.predict_public(obs,bridge.crossing_action))
        after=port.send_action(bridge.crossing_action)
        actions+=port.actions
        probes+=port.probes
        if expected!=dict(after.values):
            result=agent.observe_mechanism_shift(Trace(obs,bridge.crossing_action,after))
            if result['status']!='quarantined':
                raise AssertionError('Unexpected first-anomaly response')
            return actions,probes,index+1
    raise AssertionError('Changed law indistinguishable from historical law')

def exhaustive_test(bridge,spec,agent):
    """Independent true transitions on all four public Boolean configurations.

    Never trust an agent-reported commit as success; verify every successor.
    These scoring actions are excluded from learning costs.
    """
    if agent.primary in agent.pending_actions or len(agent.version_space)!=1:
        return False
    for index,bits in enumerate(itertools.product((False,True),repeat=2)):
        port,obs=configured_port(bridge,spec,500+index,bits)
        expected=dict(agent.predict_public(obs,bridge.crossing_action))
        real=port.send_action(bridge.crossing_action)
        if expected!=dict(real.values):
            return False
    return True

def run_arm(bridge,spec,mode,seed,budget=BUDGET):
    agent=establish_old(bridge,spec)
    detected_actions,detected_probes,detected_episodes=detect_drift(bridge,spec,agent)
    if mode=='random':
        rng=random.Random(seed^0xA70AC)
    post_probe=0
    post_prep=0
    status='budget_exhausted'
    error=None
    for episode in range(200,200+budget):
        port=PublicAdapter(bridge,World(spec,episode))
        try:
            if mode=='active':
                result=agent.run_active_guard_probe(port,bridge.crossing_action)
            else:
                desired=tuple(bool(rng.getrandbits(1)) for _ in range(2))
                obs=port.send_action(OBSERVE)
                for i,value in enumerate(desired):
                    if dict(obs.values)[f'switch{i}']!=value:
                        switch=bridge.switches[i]
                        obs=port.send_action(switch.on_token if value else switch.off_token)
                before=port.send_action(OBSERVE)
                after=port.send_action(bridge.crossing_action)
                result=agent.observe_mechanism_shift(Trace(before,bridge.crossing_action,after))
        except (CannotIdentify,DiscoveryError,AssertionError) as exc:
            status='refused'
            error=f'{type(exc).__name__}: {str(exc)[:150]}'
            post_prep+=port.actions-port.probes
            post_probe+=port.probes
            break
        post_prep+=port.actions-port.probes
        post_probe+=port.probes
        if result['status']=='committed':
            status='committed'
            break
    correct=status=='committed' and exhaustive_test(bridge,spec,agent)
    if status=='committed' and not correct:
        status='incorrect_commit'
    return {'correct':correct,'status':status,'post_probe_actions':post_probe,
            'post_control_actions':post_prep,'post_real_actions':post_probe+post_prep,
            'drift_detection_actions':detected_actions,'drift_detection_probes':detected_probes,
            'drift_detection_episodes':detected_episodes,
            'real_actions_including_detection':detected_actions+post_probe+post_prep,
            'archived':bool(agent.mechanism_archive),'error':error}

def one_case(seed,index,budget=BUDGET):
    spec,bridge=setup(seed,LAWS[index%4])
    active=run_arm(bridge,spec,'active',seed,budget)
    random_arm=run_arm(bridge,spec,'random',seed,budget)
    return {'family':index%4,'calibration_actions_shared':bridge.exploration_actions,
            'active':active,'random':random_arm}

def benchmark(n=100,budget=BUDGET):
    if not 1<=n<=200 or not 1<=budget<=20:
        raise ValueError('Benchmark limits exceeded')
    rng=random.Random(SCHEDULE)
    seeds=[rng.randrange(1,2**50) for _ in range(n)]
    started=time.perf_counter_ns()
    pairs=[]
    for i,seed in enumerate(seeds):
        pairs.append(one_case(seed,i,budget))
        if (i+1)%10==0:
            print(f'E9_DISCOVERY {i+1}/{n}',flush=True)
    def summarize(mode):
        items=[p[mode] for p in pairs]
        return {'correct_models':sum(x['correct'] for x in items),
                'failures':sum(not x['correct'] for x in items),
                'post_probe_actions':sum(x['post_probe_actions'] for x in items),
                'post_control_actions':sum(x['post_control_actions'] for x in items),
                'post_real_actions':sum(x['post_real_actions'] for x in items),
                'drift_detection_actions':sum(x['drift_detection_actions'] for x in items),
                'real_actions_including_detection':sum(x['real_actions_including_detection'] for x in items),
                'status_counts':{status:sum(x['status']==status for x in items)
                                 for status in sorted(set(x['status'] for x in items))}}
    a,b=summarize('active'),summarize('random')
    return {'benchmark':'Athlesia E9 unlabeled pixel-affordance discovery E7/E6 pilot v0.1',
            'cases':n,'budget_per_arm':budget,
            'seed_schedule_sha256':hashlib.sha256(json.dumps(seeds,separators=(',',':')).encode()).hexdigest(),
            'active':a,'uniform_random':b,
            'discovery_actions_shared':sum(x['calibration_actions_shared'] for x in pairs),
            'external_movement_tutorial_actions_shared_per_case':5,
            'external_historical_action_traces_per_arm':4,
            'paired':{'both_correct':sum(x['active']['correct'] and x['random']['correct'] for x in pairs),
                      'active_only':sum(x['active']['correct'] and not x['random']['correct'] for x in pairs),
                      'random_only':sum(x['random']['correct'] and not x['active']['correct'] for x in pairs),
                      'neither':sum(not x['active']['correct'] and not x['random']['correct'] for x in pairs)},
            'law_families':[{'family':i,'cases':sum(p['family']==i for p in pairs),
                             'active_correct':sum(p['family']==i and p['active']['correct'] for p in pairs),
                             'random_correct':sum(p['family']==i and p['random']['correct'] for p in pairs)}
                            for i in range(4)],
            'sample_cases':pairs[:4],
            'raw_pixels_and_opaque_action_inventory':True,
            'public_control_pairs_prelabelled':False,
            'gate_coordinate_given_to_agent':False,
            'discovery_grammar_fixed_two_single_pixel_reversible_switches':True,
            'five_external_movement_tutorials':True,
            'external_episodes':True,
            'same_author_evaluator':True,
            'official_arc_agi_3':False,
            'wall_ms_total':round((time.perf_counter_ns()-started)/1e6,3)}

class E9Tests(unittest.TestCase):
    def test_opaque_action_effects_and_reversible_pairing(self):
        spec,bridge=setup(92001,LAWS[0])
        self.assertEqual({s.cell for s in bridge.switches},set(spec.pixels))
        self.assertEqual(bridge.gate,spec.gate)
        self.assertEqual(bridge.exploration_actions,8)
        self.assertEqual({s.off_token for s in bridge.switches}|{s.on_token for s in bridge.switches},
                         {x for pair in spec.control_pairs for x in pair})

    def test_false_true_visual_roles_can_reverse_physical_polarity(self):
        examples=0
        for seed in range(92010,92030):
            spec,bridge=setup(seed,LAWS[0])
            for s in bridge.switches:
                self.assertLess(s.false_colour,s.true_colour)
                examples+=int(s.false_colour in {x[1] for x in spec.colours})
        self.assertGreater(examples,0)

    def test_bridge_source_has_no_world_import(self):
        import inspect,discovery
        src=inspect.getsource(discovery)
        self.assertNotIn('from pixel_world import',src)
        self.assertNotIn('import pixel_world',src)

    def test_old_rule_and_new_guard_are_real_e6_models(self):
        spec,bridge=setup(92041,LAWS[2])
        agent=establish_old(bridge,spec)
        self.assertEqual(len(agent.version_space),1)
        count,probes,_=detect_drift(bridge,spec,agent)
        self.assertGreaterEqual(count,probes)
        self.assertIn(agent.primary,agent.pending_actions)
        outcome=run_arm(bridge,spec,'active',92041)
        self.assertTrue(outcome['correct'],outcome)
        self.assertTrue(outcome['archived'])

    def test_deterministic_outcomes(self):
        a=benchmark(4)
        b=benchmark(4)
        for key in ('seed_schedule_sha256','active','uniform_random','paired','law_families'):
            self.assertEqual(a[key],b[key])

if __name__=='__main__':
    p=argparse.ArgumentParser()
    p.add_argument('--cases',type=int,default=100)
    p.add_argument('--budget',type=int,default=12)
    p.add_argument('--output',type=Path,default=Path('results/e9_discovery100.json'))
    args=p.parse_args()
    report=benchmark(args.cases,args.budget)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,indent=2))
    assert report['active']['correct_models']+report['active']['failures']==args.cases
    assert report['uniform_random']['correct_models']+report['uniform_random']['failures']==args.cases
    print('E9_UNLABELED_AFFORDANCE_PILOT_TESTED_NOT_COMMITTED')

"""E8 integration pilot: real E7 pixel grounding + real E6 Agent guard revision.

Same-author fully observed instrumented grid. Not independent/ARC-AGI-3.
The solver must not read the fixture's hidden gate law.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import random
import time
import unittest
from agent import Demonstration
from evaluator import new_case,training_demonstrations
from athlesia.learner import Agent as CausalAgent, CannotIdentify, Literal
from athlesia.protocol import Trace, OBSERVE
from bridge import PixelBridge, PublicPixelAdapter, GroundingError
from pixel_gate_world import GateSpec, GateWorld

BUDGET=12
SCHEDULE=0xE82026
LAWS=( ((1,True),), ((1,False),), ((0,True),(1,True)), ((0,False),(1,True)) )


def make_bridge(seed:int, law):
    case,_=new_case(seed)
    spec=GateSpec.create(case,law)
    bridge=PixelBridge(training_demonstrations(case),spec.control_tokens,GateWorld.GATE)
    calibration=PublicPixelAdapter(bridge,GateWorld(spec,-20))
    bridge.fit_controls(calibration)
    assert len(bridge.switches)==2
    return spec,bridge


def trace_from_world(bridge,world):
    port=PublicPixelAdapter(bridge,world)
    before=port.send_action(OBSERVE)
    after=port.send_action(bridge.crossing_action)
    return Trace(before,bridge.crossing_action,after)


def establish_old(bridge,spec):
    old_spec=GateSpec.create(spec.case,((0,True),))
    demos=[]
    for episode,flags in enumerate(((False,False),(False,True),(True,False),(True,True)),10):
        demos.append(trace_from_world(bridge,GateWorld(old_spec,episode,flags)))
    agent=CausalAgent()
    agent.fit(tuple(demos))
    if len(agent.version_space)!=1:
        raise AssertionError('Old guard not identified from four pixel-grounded traces')
    old_role=agent.roles['switch0']
    if agent.rule.guards!=(Literal(old_role,True),):
        raise AssertionError('Unexpected historical guard identification')
    return agent


def first_contradiction(bridge,spec,agent):
    # Public pilot uses an externally supplied observed contradiction. The
    # tester may choose a distinguishing configuration, not the learner.
    possibilities=((True,False),(False,False),(True,True),(False,True))
    old=agent.rule
    for flags in possibilities:
        port=PublicPixelAdapter(bridge,GateWorld(spec,100,flags))
        before=port.send_action(OBSERVE)
        after=port.send_action(bridge.crossing_action)
        before_values=agent.normalize(before,agent.binding(before.scene))
        after_values=agent.normalize(after,agent.binding(after.scene))
        if old.predict(before_values)!=after_values:
            result=agent.observe_mechanism_shift(Trace(before,bridge.crossing_action,after))
            if result['status']!='quarantined':
                raise AssertionError('First contradictory trace must quarantine')
            return
    raise AssertionError('No contradiction between old and new bounded laws')


def run_arm(bridge,spec,policy:str,seed:int,budget:int=BUDGET):
    agent=establish_old(bridge,spec)
    first_contradiction(bridge,spec,agent)
    rng=random.Random(seed^0xEA17)
    probes=0
    preparations=0
    status='budget_exhausted'
    refusal=None
    for episode in range(101,101+budget):
        port=PublicPixelAdapter(bridge,GateWorld(spec,episode))
        try:
            if policy=='active':
                result=agent.run_active_guard_probe(port,bridge.crossing_action)
            else:
                obs=port.send_action(OBSERVE)
                desired=tuple(bool(rng.getrandbits(1)) for _ in range(2))
                for i,value in enumerate(desired):
                    if dict(obs.values)[f'switch{i}']!=value:
                        token=spec.control_tokens[i][1 if value else 0]
                        obs=port.send_action(token)
                before=port.send_action(OBSERVE)
                after=port.send_action(bridge.crossing_action)
                result=agent.observe_mechanism_shift(Trace(before,bridge.crossing_action,after))
        except (CannotIdentify,GroundingError) as exc:
            status='refused'
            refusal=type(exc).__name__+': '+str(exc)[:100]
            preparations+=port.actions-port.probe_actions
            probes+=port.probe_actions
            break
        probes+=port.probe_actions
        preparations+=port.actions-port.probe_actions
        if result['status']=='committed':
            status='committed'
            break
    true_rule=tuple(sorted(Literal(agent.roles[f'switch{i}'],v) for i,v in spec.law))
    correct=(status=='committed' and len(agent.rule.guards)==len(true_rule)
             and set(agent.rule.guards)==set(true_rule))
    if status=='committed' and not correct:
        status='wrong_commit'
    return {'success':correct,'status':status,'probes':probes,'calibration_actions':preparations,
            'real_actions':probes+preparations,'error':refusal,
            'archived':bool(agent.mechanism_archive)}


def one_case(seed:int,index:int,budget:int=BUDGET):
    law=LAWS[index%len(LAWS)]
    spec,bridge=make_bridge(seed,law)
    active=run_arm(bridge,spec,'active',seed,budget)
    random_baseline=run_arm(bridge,spec,'random',seed,budget)
    return {'index':index,'law_family':index%len(LAWS),'active':active,
            'random':random_baseline}


def benchmark(cases=100,budget=BUDGET):
    if not 1<=cases<=200 or not 1<=budget<=20:
        raise ValueError('Invalid benchmark bounds')
    rng=random.Random(SCHEDULE)
    seeds=[rng.randrange(1,2**50) for _ in range(cases)]
    start=time.perf_counter_ns()
    pairs=[]
    for i,seed in enumerate(seeds):
        pairs.append(one_case(seed,i,budget))
        if (i+1)%10==0:
            print(f'E8_PIXEL_CAUSAL {i+1}/{cases}',flush=True)
    def arm(which):
        items=[p[which] for p in pairs]
        return {'successes':sum(item['success'] for item in items),
                'failures':sum(not item['success'] for item in items),
                'probe_actions':sum(item['probes'] for item in items),
                'calibration_actions':sum(item['calibration_actions'] for item in items),
                'real_actions':sum(item['real_actions'] for item in items),
                'status_counts':{s:sum(item['status']==s for item in items)
                                 for s in sorted(set(item['status'] for item in items))}}
    return {'benchmark':'Athlesia E8 E7-pixels to E6-guard-learning integration pilot v0.1',
            'cases':cases,'budget_probes_per_arm':budget,
            'fixed_schedule_sha256':hashlib.sha256(json.dumps(seeds,separators=(',',':')).encode()).hexdigest(),
            'active':arm('active'),'uniform_random':arm('random'),
            'paired':{'both_success':sum(x['active']['success'] and x['random']['success'] for x in pairs),
                      'active_only':sum(x['active']['success'] and not x['random']['success'] for x in pairs),
                      'random_only':sum(x['random']['success'] and not x['active']['success'] for x in pairs),
                      'both_failed':sum(not x['active']['success'] and not x['random']['success'] for x in pairs)},
            'by_law':[{'name':str(LAWS[j]),'cases':sum(x['law_family']==j for x in pairs),
                       'active_successes':sum(x['law_family']==j and x['active']['success'] for x in pairs),
                       'random_successes':sum(x['law_family']==j and x['random']['success'] for x in pairs)}
                      for j in range(len(LAWS))],
            'sample_cases':pairs[:4],
            'simulator_source_imported_by_bridge':False,'pixel_controls_preinstrumented':True,
            'fully_observable':True,'externally_supplied_first_contradiction_and_resets':True,
            'same_author_test':True,'official_arc_agi_3':False,
            'wall_ms_total':round((time.perf_counter_ns()-start)/1e6,3)}

class E8Tests(unittest.TestCase):
    def test_visual_calibration_recovers_pixel_switches(self):
        spec,bridge=make_bridge(70201,LAWS[1]);w=GateWorld(spec,123,(False,True))
        values=dict(bridge.pixels_to_values(w.observe()))
        self.assertEqual(values,{'crossed':False,'switch0':False,'switch1':True})
    def test_e7_action_tokens_grounded(self):
        spec,bridge=make_bridge(70203,LAWS[0]);self.assertEqual(bridge.visual.action_delta[bridge.crossing_action],(0,1))
        self.assertEqual(dict(spec.case.action_map)[bridge.crossing_action],(0,1))
    def test_e6_old_model_is_identified(self):
        spec,bridge=make_bridge(70205,LAWS[0]);a=establish_old(bridge,spec)
        self.assertEqual(len(a.version_space),1)
    def test_true_changed_guards_can_be_reidentified(self):
        for i in range(4):
            spec,bridge=make_bridge(70300+i,LAWS[i])
            result=run_arm(bridge,spec,'active',i)
            self.assertTrue(result['success'],(i,result))
            self.assertTrue(result['archived'])
    def test_seed_schedule_deterministic(self):
        a=benchmark(4);b=benchmark(4)
        for k in ('fixed_schedule_sha256','active','uniform_random','paired','by_law'):
            self.assertEqual(a[k],b[k])
    def test_prior_e6_e7_source_exists(self):
        from pathlib import Path
        import hashlib
        root=Path(__file__).resolve().parents[1]
        for path in ('e6_online_revision_transfer/athlesia/learner.py',
                     'e7_raw_grid_transfer/agent.py'):
            self.assertTrue((root/path).is_file())
    def test_bridge_has_no_private_simulator_import(self):
        import inspect
        import bridge
        src=inspect.getsource(bridge)
        self.assertNotIn('from evaluator import',src)
        self.assertNotIn('from pixel_gate_world import',src)

if __name__=='__main__':
    ap=argparse.ArgumentParser()
    ap.add_argument('--cases',type=int,default=100)
    ap.add_argument('--budget',type=int,default=BUDGET)
    ap.add_argument('--output',type=Path,default=Path('results/e8_pixel_causal100.json'))
    args=ap.parse_args()
    report=benchmark(args.cases,args.budget)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))
    assert report['active']['successes']+report['active']['failures']==args.cases
    assert report['uniform_random']['successes']+report['uniform_random']['failures']==args.cases
    print('E8_PIXEL_CAUSAL_PILOT_TESTED_NOT_COMMITTED')

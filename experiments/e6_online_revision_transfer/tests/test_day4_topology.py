"""Day 4 pilot: actual larger-target graph, ambiguity and side-effect controls.

This is a narrow structural-extension test, not arbitrary novel-goal transfer.
"""
from dataclasses import replace
import unittest

from athlesia.environment import World, WorldConfig
from athlesia.graph import MatchStatus, induced_embedding
from athlesia.learner import Agent, CannotIdentify
from athlesia.protocol import Edge, Kind, Node, Observation, Relation, Scene, Trace, OBSERVE


def learned() -> Agent:
    traces = []
    for episode in range(3):
        w = World(WorldConfig(17,(0,),(True,True),episode))
        before=w.send_action(OBSERVE)
        traces.append(Trace(before,w.primary,w.send_action(w.primary)))
    agent=Agent()
    agent.fit(tuple(traces))
    agent.revise(World(WorldConfig(17,(0,),(False,False),9)))
    return agent


class ExpandedWorld:
    def __init__(self, world: World, unexpected: bool = False) -> None:
        self.world=world
        self.unexpected=unexpected
        self.extra_obj='day4_new_object'
        self.extra_fluent='day4_new_fluent'
        self.extra_goal='day4_new_goal'
        self.extra_reset='day4_new_reset_action'
        old=world.scene
        nodes=old.nodes+(Node(self.extra_obj,Kind.OBJECT),
                         Node(self.extra_fluent,Kind.FLUENT),
                         Node(self.extra_goal,Kind.GOAL),
                         Node(self.extra_reset,Kind.ACTION))
        edges=old.edges+(Edge(world.objects[0],Relation.LINK,self.extra_obj),
                         Edge(self.extra_fluent,Relation.AT,self.extra_obj),
                         Edge(self.extra_goal,Relation.AT,self.extra_obj),
                         Edge(self.extra_reset,Relation.CLEAR,self.extra_goal))
        layout=old.layout+((self.extra_obj,0,0),(self.extra_fluent,1,0),
                            (self.extra_goal,2,0),(self.extra_reset,3,0))
        self.scene=Scene(nodes,edges,old.width,old.height,layout)
        self.extra_changed=False

    def send_action(self, token: str) -> Observation:
        raw=self.world.send_action(token)
        if token == self.world.primary and self.unexpected:
            self.extra_changed=True
        values=dict(raw.values)
        values[self.extra_fluent]=self.extra_changed
        values[self.extra_goal]=False
        return replace(raw,scene=self.scene,values=tuple(sorted(values.items())))


class DuplicateWorld:
    def __init__(self, world: World) -> None:
        self.world=world
        old=world.scene
        self.scene=Scene(old.nodes+tuple(Node('dup_'+n.id,n.kind) for n in old.nodes),
            old.edges+tuple(Edge('dup_'+e.source,e.relation,'dup_'+e.target) for e in old.edges),
            old.width,old.height,
            old.layout+tuple(('dup_'+n,x,y) for n,x,y in old.layout))

    def send_action(self, token: str) -> Observation:
        raw=self.world.send_action(token)
        values=dict(raw.values)
        values.update({'dup_'+n:v for n,v in raw.values})
        return replace(raw,scene=self.scene,values=tuple(sorted(values.items())))


class Day4TopologyTests(unittest.TestCase):
    def test_genuine_larger_topology_unique_mapping_and_memory_constancy(self) -> None:
        a=learned()
        w=ExpandedWorld(World(WorldConfig(5555,(0,),(False,False),12)))
        match=induced_embedding(a.template,w.scene)
        self.assertEqual(match.status,MatchStatus.UNIQUE)
        self.assertGreater(len(w.scene.nodes),len(a.template.nodes))
        prior=a.memory_digest()
        result=a.transfer(w)
        self.assertTrue(result['first_task_attempt_success'])
        self.assertEqual(prior,a.memory_digest())

    def test_duplicate_subgraph_is_ambiguous_and_fails_closed(self) -> None:
        a=learned()
        w=DuplicateWorld(World(WorldConfig(5555,(0,),(False,False),12)))
        self.assertEqual(induced_embedding(a.template,w.scene).status,MatchStatus.AMBIGUOUS)
        prior=a.memory_digest()
        with self.assertRaisesRegex(CannotIdentify,'ambiguous'):
            a.transfer(w)
        self.assertEqual(prior,a.memory_digest())

    def test_new_fluent_side_effect_falsifies_transfer_without_memory_mutation(self) -> None:
        a=learned()
        w=ExpandedWorld(World(WorldConfig(5555,(0,),(False,False),12)),unexpected=True)
        prior=a.memory_digest()
        with self.assertRaisesRegex(CannotIdentify,'Same observable topology'):
            a.transfer(w)
        self.assertEqual(prior,a.memory_digest())

    def test_hard_search_limit_and_missing_structure_fail_closed(self) -> None:
        a=learned()
        w=ExpandedWorld(World(WorldConfig(5555,(0,),(False,False),12)))
        self.assertEqual(induced_embedding(a.template,w.scene,budget=1).status,
                         MatchStatus.BUDGET_EXCEEDED)
        missing=replace(w.scene,edges=tuple(e for e in w.scene.edges
                                         if e.relation!=Relation.ACTS_ON))
        self.assertEqual(induced_embedding(a.template,missing).status,MatchStatus.NO_MATCH)

    def test_existing_online_validation_refuses_changed_topology(self) -> None:
        a=learned()
        w=ExpandedWorld(World(WorldConfig(5555,(0,),(True,True),13)))
        before=w.send_action(OBSERVE)
        after=w.send_action(w.world.primary)
        with self.assertRaisesRegex(CannotIdentify,'no_match'):
            a.observe_transition(Trace(before,w.world.primary,after))


if __name__=='__main__':
    unittest.main()

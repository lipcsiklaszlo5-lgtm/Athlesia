from __future__ import annotations

import ast
from dataclasses import replace
from pathlib import Path
import unittest
from types import SimpleNamespace
from typing import cast

from athlesia.benchmark import RemoteWorld, independent_success

from athlesia.environment import World, WorldConfig
from athlesia.graph import MatchStatus, isomorphism
from athlesia.learner import Agent, CannotIdentify, Literal, Rule, MAX_WITNESSES
from athlesia.protocol import (ContractError, Edge, Kind, MAX_BYTES, Node,
                               Observation, Relation, Scene, Trace, OBSERVE, loads)


def trained(seed: int = 17, channels: int = 2, guards: tuple[int, ...] = (0,)) -> tuple[Agent, tuple[Trace, ...]]:
    records = []
    for i in range(3):
        w = World(WorldConfig(seed, guards, (True,) * channels, i))
        before = w.send_action(OBSERVE)
        records.append(Trace(before, w.primary, w.send_action(w.primary)))
    agent = Agent()
    agent.fit(tuple(records))
    return agent, tuple(records)


class ContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.world = World(WorldConfig(17))
        self.obs = self.world.send_action(OBSERVE)

    def test_roundtrip(self) -> None:
        self.assertEqual(Observation.from_wire(self.obs.to_wire()), self.obs)

    def test_reject_oracle_fields(self) -> None:
        wire = self.obs.to_wire() | {"hidden_guards": [0]}
        with self.assertRaises(ContractError):
            Observation.from_wire(wire)

    def test_reject_duplicate_json_keys(self) -> None:
        with self.assertRaises(ContractError):
            loads('{"token":"a","token":"b"}')

    def test_reject_nan_and_oversize(self) -> None:
        for text in ('{"a":NaN}', '"' + 'x' * MAX_BYTES + '"'):
            with self.assertRaises(ContractError):
                loads(text)

    def test_reject_missing_fluent(self) -> None:
        wire = self.obs.to_wire()
        wire["values"] = {}
        with self.assertRaises(ContractError):
            Observation.from_wire(wire)

    def test_unknown_is_not_false(self) -> None:
        with self.assertRaises(ContractError):
            self.obs.value("missing")
        with self.assertRaises(CannotIdentify):
            Literal("missing", False).holds({})

    def test_reject_stale_trace(self) -> None:
        with self.assertRaises(ContractError):
            Trace(self.obs, self.world.primary, self.obs).validate()

    def test_reject_cross_episode(self) -> None:
        after = self.world.send_action(self.world.primary)
        with self.assertRaises(ContractError):
            Trace(self.obs, self.world.primary, replace(after, episode="another")).validate()

    def test_unknown_action_has_no_effect(self) -> None:
        with self.assertRaises(ContractError):
            self.world.send_action("not_an_action")
        self.assertEqual(self.world.send_action(OBSERVE), self.obs)

    def test_hidden_law_absent_from_initial_observation(self) -> None:
        a = World(WorldConfig(61, (0,))).send_action(OBSERVE)
        b = World(WorldConfig(61, (1,))).send_action(OBSERVE)
        self.assertEqual(a.to_wire(), b.to_wire())

    def test_bool_is_not_sequence_integer(self) -> None:
        wire = self.obs.to_wire() | {"sequence": True}
        with self.assertRaises(ContractError):
            Observation.from_wire(wire)

    def test_evaluator_does_not_trust_claimed_success(self) -> None:
        fake = cast(RemoteWorld, SimpleNamespace(last_observation=self.obs, task_attempts=1))
        self.assertFalse(independent_success(fake))
        after = self.world.send_action(self.world.primary)
        fake.last_observation = after
        self.assertTrue(independent_success(fake))
        fake.task_attempts = 2
        self.assertFalse(independent_success(fake))


class GraphTests(unittest.TestCase):
    def test_random_relabelling_layout_and_order(self) -> None:
        a, b = World(WorldConfig(17)), World(WorldConfig(997))
        self.assertFalse({n.id for n in a.scene.nodes} & {n.id for n in b.scene.nodes})
        self.assertNotEqual((a.scene.width, a.scene.height), (b.scene.width, b.scene.height))
        result = isomorphism(a.scene, b.scene)
        mapping = result.unique()
        self.assertEqual(mapping[a.primary], b.primary)
        self.assertEqual(mapping[a.fluents[0]], b.fluents[0])

    def test_ambiguity_is_not_arbitrary_matching(self) -> None:
        s = Scene((Node("a", Kind.OBJECT), Node("b", Kind.OBJECT)), (), 2, 2, (("a", 0, 0), ("b", 1, 1)))
        self.assertEqual(isomorphism(s, s).status, MatchStatus.AMBIGUOUS)

    def test_budget_exhaustion_is_not_success(self) -> None:
        a, b = World(WorldConfig(17)), World(WorldConfig(997))
        self.assertEqual(isomorphism(a.scene, b.scene, 1).status, MatchStatus.BUDGET_EXCEEDED)

    def test_refinement_collision_does_not_prove_isomorphism(self) -> None:
        nodes = tuple(Node(f"n{i}", Kind.OBJECT) for i in range(6))
        def graph(pairs: list[tuple[int, int]]) -> Scene:
            edges = tuple(Edge(f"n{i}", Relation.LINK, f"n{j}")
                          for x, y in pairs for i, j in ((x, y), (y, x)))
            return Scene(nodes, edges, 6, 2, tuple((n.id, i, 0) for i, n in enumerate(nodes)))
        cycle = graph([(i, (i + 1) % 6) for i in range(6)])
        triangles = graph([(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)])
        self.assertEqual(isomorphism(cycle, triangles).status, MatchStatus.NO_MATCH)

    def test_wrong_topology_rejected(self) -> None:
        a = World(WorldConfig(17))
        b = replace(a.scene, edges=tuple(e for e in a.scene.edges if e.relation != Relation.LINK))
        self.assertEqual(isomorphism(a.scene, b).status, MatchStatus.NO_MATCH)


class LearningTests(unittest.TestCase):
    @staticmethod
    def trace(seed: int, flags: tuple[bool, ...], episode: int,
              action_index: int | None = None, set_true: bool = True) -> tuple[Trace, World]:
        world = World(WorldConfig(seed, (0,), flags, episode))
        before = world.send_action(OBSERVE)
        action = world.primary if action_index is None else (
            world.up[action_index] if set_true else world.down[action_index]
        )
        return Trace(before, action, world.send_action(action)), world

    def test_three_positives_do_not_identify_the_law(self) -> None:
        a, _ = trained()
        self.assertEqual(len(a.version_space), 4)
        self.assertEqual(a.rule.guards, ())

    def test_mixed_actions_are_induced_into_isolated_schema_spaces(self) -> None:
        def observed(flags: tuple[bool, ...], episode: int,
                     auxiliary: int | None) -> tuple[Trace, World]:
            world = World(WorldConfig(17, (0,), flags, episode, True))
            before = world.send_action(OBSERVE)
            action = world.primary if auxiliary is None else world.auxiliary[auxiliary]
            return Trace(before, action, world.send_action(action)), world

        primary = [observed((True, True, True), i, None)[0] for i in range(3)]
        aux_true, source = observed((False, False, False), 10, 0)
        aux_true_noop, _ = observed((True, True, True), 11, 0)
        aux_false, _ = observed((True, True, True), 12, 1)
        aux_false_noop, _ = observed((False, False, False), 13, 1)
        agent = Agent()
        agent.fit(tuple((*primary, aux_true, aux_true_noop,
                         aux_false, aux_false_noop)))

        primary_role = agent.roles[source.primary]
        up_role = agent.roles[source.auxiliary[0]]
        down_role = agent.roles[source.auxiliary[1]]
        self.assertEqual(set(agent.rules), {primary_role, up_role, down_role})
        self.assertEqual(len(agent.version_spaces[primary_role]), 7)
        self.assertEqual(agent.rules[up_role].effects,
                         (Literal(agent.roles[source.fluents[1]], True),))
        self.assertEqual(agent.rules[down_role].effects,
                         (Literal(agent.roles[source.fluents[2]], False),))
        for action in source.auxiliary:
            relations = {edge.relation for edge in source.scene.edges if edge.source == action}
            self.assertEqual(relations, {Relation.ACTS_ON})

        held_out = World(WorldConfig(17, (0,), (True, False, True), 14, True))
        before = held_out.send_action(OBSERVE)
        predicted = dict(agent.predict_public(before, held_out.auxiliary[0]))
        actual = dict(held_out.send_action(held_out.auxiliary[0]).values)
        self.assertEqual(predicted, actual)

        unrelated_rules = {role: agent.rules[role] for role in (up_role, down_role)}
        unrelated_spaces = {role: agent.version_spaces[role] for role in (up_role, down_role)}
        revision = agent.revise(World(WorldConfig(17, (0,), (False, False, False), 20, True)))
        self.assertTrue(revision["unrelated_models_unchanged"])
        self.assertEqual({role: agent.rules[role] for role in unrelated_rules}, unrelated_rules)
        self.assertEqual({role: agent.version_spaces[role] for role in unrelated_spaces},
                         unrelated_spaces)

    def test_negative_witness_filters_only_inconsistent_guard_models(self) -> None:
        positive, source = self.trace(31, (True, True), 0)
        negative, _ = self.trace(31, (False, True), 1)
        agent = Agent()
        agent.fit((positive, negative))
        first_feature = agent.roles[source.fluents[0]]
        self.assertEqual(len(agent.version_space), 2)
        self.assertEqual(agent.rule.guards, (Literal(first_feature, True),))
        self.assertTrue(all(agent.consistent(rule) for rule in agent.version_space))

    def test_nonuniform_effect_for_same_action_is_rejected(self) -> None:
        first, _ = self.trace(43, (True, True), 0)
        second, source = self.trace(43, (True, True), 1)
        values = dict(second.after.values)
        values[source.fluents[0]] = False
        corrupted = replace(second, after=replace(second.after,
                                                   values=tuple(sorted(values.items()))))
        agent = Agent()
        with self.assertRaisesRegex(CannotIdentify, "Nonuniform observed effect"):
            agent.fit((first, corrupted))

    def test_day1_exact_local_revision_and_history(self) -> None:
        a, traces = trained()
        before = a.rule
        trap = World(WorldConfig(17, (0,), (False, False), 3))
        result = a.revise(trap)
        self.assertEqual(result["literal_edits"], 1)
        self.assertEqual(result["experiments"], 1)
        self.assertEqual(result["interventions"], 1)
        self.assertEqual(result["task_attempts"], 2)
        self.assertEqual(before.effects, a.rule.effects)
        self.assertEqual(before.context, a.rule.context)
        for t in traces:
            b = a.binding(t.before.scene)
            self.assertEqual(a.rule.predict(a.normalize(t.before, b)), a.normalize(t.after, b))

    def test_latch_guard_is_learned_instead_of_hardcoded_power(self) -> None:
        a, _ = trained(23, guards=(1,))
        trap = World(WorldConfig(23, (1,), (False, False), 3))
        result = a.revise(trap)
        self.assertEqual(result["literal_edits"], 1)
        self.assertEqual(a.rule.guards[0].role, a.roles[trap.fluents[1]])

    def test_two_guard_hypothesis_not_discarded_by_occam(self) -> None:
        a, _ = trained(44, guards=(0, 1))
        result = a.revise(World(WorldConfig(44, (0, 1), (False, False), 3)))
        self.assertEqual(result["literal_edits"], 2)
        self.assertEqual(len(a.rule.guards), 2)

    def test_transfer_no_retraining_and_first_task_attempt(self) -> None:
        a, _ = trained()
        a.revise(World(WorldConfig(17, (0,), (False, False), 3)))
        digest = a.memory_digest()
        result = a.transfer(World(WorldConfig(5555, (0,), (False, False), 4)))
        self.assertTrue(result["first_task_attempt_success"])
        self.assertEqual(result["preparation_actions"], 1)
        self.assertEqual(result["adaptation_queries"], 0)
        self.assertEqual(digest, a.memory_digest())

    def test_same_topology_different_law_fails_without_fake_success(self) -> None:
        a, _ = trained()
        a.revise(World(WorldConfig(17, (0,), (False, False), 3)))
        digest = a.memory_digest()
        with self.assertRaisesRegex(CannotIdentify, "same mechanism"):
            a.transfer(World(WorldConfig(5555, (1,), (False, False), 4)))
        self.assertEqual(digest, a.memory_digest())

    def test_transfer_refuses_unidentified_model(self) -> None:
        a, _ = trained()
        with self.assertRaises(CannotIdentify):
            a.transfer(World(WorldConfig(5555)))

    def test_other_rule_unchanged(self) -> None:
        a, _ = trained()
        independent = Rule("?independent", (), (Literal("?q", True),), (Literal("?r", False),))
        a.rules[independent.action] = independent
        a.revise(World(WorldConfig(17, (0,), (False, False), 3)))
        self.assertIs(a.rules[independent.action], independent)

    def test_out_of_class_law_cannot_be_certified(self) -> None:
        a, _ = trained(72, 3, (0, 1, 2))
        before = a.rule
        with self.assertRaises(CannotIdentify):
            a.revise(World(WorldConfig(72, (0, 1, 2), (False,) * 3, 3)))
        self.assertEqual(a.rule, before)
        self.assertEqual(a.commits, 0)

    def test_no_silent_evidence_eviction(self) -> None:
        a = Agent()
        for i in range(MAX_WITNESSES):
            a.remember(((str(i), True),), ((str(i), False),))
        with self.assertRaises(CannotIdentify):
            a.remember((("overflow", True),), (("overflow", False),))
        self.assertEqual(len(a.witnesses), MAX_WITNESSES)

    def test_solver_source_has_no_environment_import(self) -> None:
        root = Path(__file__).parents[1] / "athlesia"
        for file in ("learner.py", "graph.py", "protocol.py"):
            tree = ast.parse((root / file).read_text())
            for node in ast.walk(tree):
                if isinstance(node, ast.ImportFrom):
                    self.assertNotIn("environment", node.module or "")


class OnlineTransitionTests(unittest.TestCase):
    def test_counterexample_deferred_then_action_local_commit(self) -> None:
        agent, _ = trained(17, 2, (0,))
        prior = agent.rule
        misleading = World(WorldConfig(17, (0,), (False, True), 80))
        before = misleading.send_action(OBSERVE)
        after = misleading.send_action(misleading.primary)
        first = agent.observe_transition(Trace(before, misleading.primary, after))
        self.assertEqual(first["status"], "deferred")
        self.assertTrue(first["contradiction"])
        self.assertEqual(len(agent.version_space), 2)
        self.assertEqual(agent.rule, prior)
        with self.assertRaisesRegex(CannotIdentify, "Unresolved online contradiction"):
            agent.predict_public(before, misleading.primary)
        discriminating = World(WorldConfig(17, (0,), (True, False), 81))
        before = discriminating.send_action(OBSERVE)
        after = discriminating.send_action(discriminating.primary)
        second = agent.observe_transition(Trace(before, discriminating.primary, after))
        self.assertEqual(second["status"], "committed")
        self.assertTrue(second["local_commit"])
        self.assertEqual(agent.commits, 1)
        self.assertEqual(len(agent.version_space), 1)
        self.assertEqual(agent.rule.guards,
                         (Literal(agent.roles[discriminating.fluents[0]], True),))
        self.assertFalse(agent.pending_actions)
        self.assertEqual(dict(agent.predict_public(before, discriminating.primary)),
                         dict(after.values))

    def test_auxiliary_revision_does_not_change_other_action_models(self) -> None:
        def trace(flags: tuple[bool, ...], episode: int, selected: int | None) -> Trace:
            world = World(WorldConfig(27, (0,), flags, episode, True))
            action = world.primary if selected is None else world.auxiliary[selected]
            before = world.send_action(OBSERVE)
            return Trace(before, action, world.send_action(action))
        demonstrations = tuple(
            [trace((True, True, True), i, None) for i in range(3)]
            + [trace((False, False, False), 10, 0),
               trace((True, True, True), 11, 0),
               trace((True, True, True), 12, 1),
               trace((False, False, False), 13, 1)])
        agent = Agent()
        agent.fit(demonstrations)
        target_action = agent.roles[demonstrations[3].action]
        other_actions = {a: (r, agent.version_spaces[a],
                             dict(agent.witness_store(a)))
                         for a, r in agent.rules.items() if a != target_action}
        # A synthetic but protocol-valid conditional actuator trace. This
        # tests the learner's declared rule class, not the private fixture law.
        staged = World(WorldConfig(27, (0,), (True, False, True), 85, True))
        before = staged.send_action(OBSERVE)
        unchanged = replace(before, sequence=before.sequence + 1)
        deferred = agent.observe_transition(
            Trace(before, staged.auxiliary[0], unchanged))
        self.assertEqual(deferred["status"], "deferred")
        separating = World(WorldConfig(27, (0,), (False, False, True), 86, True))
        before = separating.send_action(OBSERVE)
        after = separating.send_action(separating.auxiliary[0])
        confirmed = agent.observe_transition(
            Trace(before, separating.auxiliary[0], after))
        self.assertEqual(confirmed["status"], "committed")
        self.assertEqual(len(agent.version_spaces[target_action]), 1)
        for role, (rule, hypotheses, witnesses) in other_actions.items():
            self.assertEqual(agent.rules[role], rule)
            self.assertEqual(agent.version_spaces[role], hypotheses)
            self.assertEqual(agent.witness_store(role), witnesses)

    def test_out_of_class_transition_rejected_atomically(self) -> None:
        agent, _ = trained(19, 2, (0,))
        world = World(WorldConfig(19, (0,), (True, True), 83))
        before = world.send_action(OBSERVE)
        after = world.send_action(world.primary)
        forged = dict(after.values)
        forged[world.fluents[0]] = False  # target action does not change this fluent
        bad = replace(after, values=tuple(sorted(forged.items())))
        digest = agent.memory_digest()
        event_count = len(agent.events)
        with self.assertRaisesRegex(CannotIdentify, "atomic rejection"):
            agent.observe_transition(Trace(before, world.primary, bad))
        self.assertEqual(agent.memory_digest(), digest)
        self.assertEqual(len(agent.events), event_count)

    def test_stale_transition_rejected_atomically(self) -> None:
        agent, _ = trained(20, 2, (0,))
        world = World(WorldConfig(20, (0,), (True, True), 84))
        before = world.send_action(OBSERVE)
        after = world.send_action(world.primary)
        stale = replace(after, sequence=before.sequence)
        digest = agent.memory_digest()
        with self.assertRaises(ContractError):
            agent.observe_transition(Trace(before, world.primary, stale))
        self.assertEqual(agent.memory_digest(), digest)


if __name__ == "__main__":
    unittest.main()

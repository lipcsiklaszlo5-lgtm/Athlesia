#!/usr/bin/env python3
"""Deterministic acceptance tests for the Open Intelligence evidence harness."""

import unittest

from core_demo import (
    CoreAgent,
    ObservationActionEnvironment,
    ROLE_RULES,
    RoleSchema,
    Transition,
    _paired_bootstrap_interval,
    _reachable,
    _role_feature_values,
    _spatial_features,
    evaluate,
    generate_world,
    validate,
)


class OpenIntelligenceCoreTests(unittest.TestCase):
    def test_generation_is_deterministic(self) -> None:
        self.assertEqual(generate_world(20_042), generate_world(20_042))

    def test_action_semantics_and_tokens_vary(self) -> None:
        worlds = [generate_world(seed) for seed in range(20_000, 20_032)]
        self.assertGreater(len({world.action_directions for world in worlds}), 12)
        token_sets = [world.surface_tokens for world in worlds]
        for index, tokens in enumerate(token_sets):
            for other in token_sets[index + 1 :]:
                self.assertTrue(tokens.isdisjoint(other))

    def test_training_snapshot_is_not_mutated_by_an_evaluation_episode(self) -> None:
        schema = RoleSchema()
        schema.observe_collectible({"edge": 0})
        frozen = schema.clone()
        CoreAgent(frozen.clone()).run(
            ObservationActionEnvironment(generate_world(20_001))
        )
        self.assertEqual(schema, frozen)

    def test_interaction_contract_has_no_collection_or_hidden_done_oracle(self) -> None:
        self.assertEqual(
            set(Transition.__dataclass_fields__),
            {"observation", "reward", "terminated"},
        )
        environment = ObservationActionEnvironment(generate_world(1_111_111))
        self.assertFalse(hasattr(environment, "spec"))
        self.assertFalse(hasattr(environment, "done"))
        self.assertFalse(hasattr(environment, "collected"))
        result = CoreAgent().run(environment)
        self.assertLessEqual(result.steps, 96)
        self.assertEqual(result.motor_actions_learned, 4)

    def test_each_world_contains_twelve_remapped_distractors(self) -> None:
        first = generate_world(1_234_567)
        second = generate_world(2_345_678)
        self.assertEqual(len(first.decoys), 12)
        self.assertEqual(len(second.decoys), 12)
        self.assertEqual(len(first.surface_tokens), 16)
        self.assertTrue(first.surface_tokens.isdisjoint(second.surface_tokens))

    def test_all_hidden_rule_families_match_their_generated_roles(self) -> None:
        for rule_id, rule_name in enumerate(ROLE_RULES):
            world = generate_world(1_500_000 + rule_id, rule_id=rule_id)
            key_value, exit_value = _role_feature_values(rule_id)
            key_features = _spatial_features(
                world.key, world.start, world.width, world.height
            )
            exit_features = _spatial_features(
                world.exit, world.start, world.width, world.height
            )
            self.assertEqual(key_features[rule_name], key_value)
            self.assertEqual(exit_features[rule_name], exit_value)

    def test_generated_key_and_exit_are_reachable_and_not_corner_trapped(self) -> None:
        for rule_id in range(len(ROLE_RULES)):
            for offset in range(20):
                for reversed_roles in (False, True):
                    world = generate_world(
                        1_600_000 + rule_id * 100 + offset,
                        rule_id=rule_id,
                        reversed_roles=reversed_roles,
                    )
                    self.assertTrue(
                        _reachable(
                            world.start,
                            world.key,
                            world.width,
                            world.height,
                            world.walls,
                        )
                    )
                    self.assertTrue(
                        _reachable(
                            world.key,
                            world.exit,
                            world.width,
                            world.height,
                            world.walls,
                        )
                    )
                    for position in (world.key, world.exit):
                        is_corner = position[0] in (0, world.width - 1) and position[1] in (
                            0,
                            world.height - 1,
                        )
                        self.assertFalse(is_corner)

    def test_role_schema_copy_is_independent(self) -> None:
        schema = RoleSchema()
        schema.observe_collectible({"left_right": 1})
        clone = schema.clone()
        clone.observe_exit({"left_right": 0})
        self.assertEqual(schema.role_examples, {"collectible": 1})
        self.assertEqual(clone.role_examples, {"collectible": 1, "terminal": 1})

    def test_sixteen_interactive_examples_recover_each_hidden_rule(self) -> None:
        for rule_id, rule_name in enumerate(ROLE_RULES):
            learner = CoreAgent(RoleSchema(), {})
            for exposure in range(1, 17):
                seed = 1_700_000 + rule_id * 10_000 + exposure
                learner.run(
                    ObservationActionEnvironment(
                        generate_world(seed, rule_id=rule_id)
                    ),
                    budget=128,
                )
            self.assertEqual(learner.schema.selected_feature(), rule_name)

    def test_stratified_bootstrap_is_deterministic_and_checks_partition(self) -> None:
        values = [2.0, 2.0, 5.0, 5.0]
        first = _paired_bootstrap_interval(values, seed=73, stratum_sizes=[2, 2])
        second = _paired_bootstrap_interval(values, seed=73, stratum_sizes=[2, 2])
        self.assertEqual(first, second)
        self.assertEqual(first, (3.5, 3.5))
        with self.assertRaises(ValueError):
            _paired_bootstrap_interval(values, seed=73, stratum_sizes=[3, 2])

    def test_quick_benchmark_meets_predeclared_acceptance_thresholds(self) -> None:
        result = evaluate(
            train_count=8,
            heldout_per_family=12,
            ood_per_family=4,
            train_base=1_100_000,
            heldout_base=1_200_000,
            ood_base=1_300_000,
        )
        self.assertEqual(validate(result), [])
        self.assertTrue(
            result["leakage_guards"]["train_and_heldout_seeds_disjoint"]
        )
        self.assertTrue(
            result["leakage_guards"]["train_and_heldout_tokens_disjoint"]
        )


if __name__ == "__main__":
    unittest.main()

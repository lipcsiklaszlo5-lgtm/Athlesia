"""Bounded version-space AML and counterexample-guided local revision.

No simulator import; no neural model, numeric learned weight, or remote API.
Assumptions: complete Boolean observations, known intervention affordances,
stationary deterministic effect within an episode, <=2 conjunctive guard literals.
"""
from __future__ import annotations

import hashlib
import itertools
import json
from dataclasses import dataclass, replace
from typing import TypeAlias

from .graph import distances, isomorphism
from .protocol import ActionPort, ContractError, Json, Kind, Observation, Relation, Scene, Trace, OBSERVE

Role: TypeAlias = str
Values: TypeAlias = tuple[tuple[Role, bool], ...]
MAX_FEATURES = 8
MAX_GUARDS = 2
MAX_WITNESSES = 64
MAX_EXPERIMENTS = 8


class CannotIdentify(RuntimeError):
    """A result is unknown, out of scope, ambiguous, or resource-limited."""


@dataclass(frozen=True, slots=True, order=True)
class Literal:
    role: Role
    value: bool

    def holds(self, values: dict[Role, bool]) -> bool:
        if self.role not in values:
            raise CannotIdentify("Missing feature is unknown, not false")
        return values[self.role] == self.value


@dataclass(frozen=True, slots=True)
class Rule:
    action: Role
    context: tuple[tuple[Role, Relation, Role], ...]
    guards: tuple[Literal, ...]
    effects: tuple[Literal, ...]

    def predict(self, before: Values) -> Values:
        values = dict(before)
        if all(g.holds(values) for g in self.guards):
            for effect in self.effects:
                values[effect.role] = effect.value
        return tuple(sorted(values.items()))

    def guard_edit_distance(self, other: Rule) -> int:
        if (self.action, self.context, self.effects) != (other.action, other.context, other.effects):
            raise CannotIdentify("This revision class may not edit actions/context/effects")
        return len(set(self.guards) ^ set(other.guards))

    def to_wire(self) -> dict[str, Json]:
        return {"action_role": self.action,
                "context": [[a, rel.value, b] for a, rel, b in self.context],
                "guards": [{"role": g.role, "value": g.value} for g in self.guards],
                "effects": [{"role": e.role, "value": e.value} for e in self.effects]}


@dataclass(frozen=True, slots=True)
class Witness:
    before: Values
    after: Values


@dataclass(frozen=True, slots=True)
class Experiment:
    assignment: tuple[Literal, ...]
    worst_case_eliminated: int
    actions: int
    partitions: tuple[int, ...]


class Agent:
    def __init__(self) -> None:
        self.template: Scene | None = None
        self.roles: dict[str, Role] = {}
        self.features: tuple[Role, ...] = ()
        self.rules: dict[Role, Rule] = {}
        self.primary: Role | None = None
        self.version_space: tuple[Rule, ...] = ()
        self.witnesses: dict[Witness, int] = {}
        self.events: list[dict[str, Json]] = []
        self.commits = 0

    @property
    def rule(self) -> Rule:
        if self.primary is None:
            raise CannotIdentify("Untrained model")
        return self.rules[self.primary]

    def emit(self, name: str, **fields: Json) -> None:
        if len(self.events) >= 256:
            raise CannotIdentify("Audit-event budget exceeded")
        self.events.append({"event": name, **fields})

    def binding(self, scene: Scene) -> dict[Role, str]:
        if self.template is None:
            raise CannotIdentify("No source graph")
        match = isomorphism(self.template, scene)
        try:
            mapping = match.unique()
        except ValueError as ex:
            raise CannotIdentify(str(ex)) from ex
        return {self.roles[src]: dst for src, dst in mapping.items()}

    def normalize(self, obs: Observation, binding: dict[Role, str]) -> Values:
        raw = dict(obs.values)
        return tuple(sorted((role, raw[node]) for role, node in binding.items() if node in raw))

    def remember(self, before: Values, after: Values) -> None:
        w = Witness(before, after)
        if w not in self.witnesses and len(self.witnesses) >= MAX_WITNESSES:
            raise CannotIdentify("Evidence capacity reached; no silent forgetting")
        self.witnesses[w] = self.witnesses.get(w, 0) + 1

    def consistent(self, rule: Rule) -> bool:
        return all(rule.predict(w.before) == w.after for w in self.witnesses)

    def fit(self, traces: tuple[Trace, ...]) -> None:
        if self.template is not None or not 1 <= len(traces) <= 32:
            raise CannotIdentify("Fit requires a fresh agent and 1..32 demonstrations")
        first = traces[0]
        first.validate()
        self.template = first.before.scene
        # Alpha variables are internal names, not guessed token meanings.
        self.roles = {n.id: f"?v{i}" for i, n in enumerate(self.template.nodes)}
        self.primary = self.roles[first.action]
        distance = distances(self.template, first.action)
        features = sorted((n.id for n in self.template.nodes if n.kind == Kind.FLUENT),
                          key=lambda n: distance.get(n, 1_000_000))
        if not 1 <= len(features) <= MAX_FEATURES:
            raise CannotIdentify("Feature budget exceeded")
        # Equidistant features are allowed: ordering affects probe cost, not a
        # certified mapping. Graph ambiguity is checked separately.
        self.features = tuple(self.roles[n] for n in features)
        effects: set[Literal] | None = None
        first_values: Values | None = None
        for trace in traces:
            trace.validate()
            binding = self.binding(trace.before.scene)
            reverse = {v: k for k, v in binding.items()}
            if reverse[trace.action] != self.primary:
                raise CannotIdentify("Day-1 induction expects one action schema")
            before, after = self.normalize(trace.before, binding), self.normalize(trace.after, binding)
            delta = {Literal(role, val) for role, val in after if dict(before)[role] != val}
            effects = delta if effects is None else effects & delta
            self.remember(before, after)
            first_values = before if first_values is None else first_values
        if not effects:
            raise CannotIdentify("No consistent observed effect")
        if any(e.role in self.features for e in effects):
            raise CannotIdentify("The probe schema assumes primary action does not change guard channels")
        context = tuple((self.roles[e.source], e.relation, self.roles[e.target])
                        for e in self.template.edges if e.relation == Relation.LINK)
        naive = Rule(self.primary, context, (), tuple(sorted(effects)))
        if not self.consistent(naive):
            raise CannotIdentify("Nonuniform effect; expand model class explicitly")
        self.rules[self.primary] = naive
        # Both signs are part of the grammar; positive demonstrations remove
        # impossible ones. Search is finite and capped, not unconstrained ILP.
        if first_values is None:
            raise CannotIdentify("No examples")
        literals = tuple(Literal(f, dict(first_values)[f]) for f in self.features)
        hypotheses = tuple(replace(naive, guards=tuple(gs))
                           for k in range(MAX_GUARDS + 1)
                           for gs in itertools.combinations(literals, k))
        self.version_space = tuple(h for h in hypotheses if self.consistent(h))
        self.emit("NAIVE_RULE_INDUCED", demonstrations=len(traces),
                  remaining_hypotheses=len(self.version_space), exact_identification=False,
                  rule=naive.to_wire())

    def intervention(self, scene: Scene, binding: dict[Role, str], literal: Literal) -> str:
        relation = Relation.SET_TRUE if literal.value else Relation.SET_FALSE
        actions = [e.source for e in scene.edges if e.relation == relation and e.target == binding[literal.role]]
        if len(actions) != 1:
            raise CannotIdentify("Missing or ambiguous public intervention capability")
        return actions[0]

    def choose_experiment(self, values: Values) -> Experiment:
        before = dict(values)
        best: tuple[tuple[object, ...], Experiment] | None = None
        for assignment in itertools.product((False, True), repeat=len(self.features)):
            trial = before | dict(zip(self.features, assignment))
            outcomes: dict[Values, int] = {}
            for hypothesis in self.version_space:
                prediction = hypothesis.predict(tuple(sorted(trial.items())))
                outcomes[prediction] = outcomes.get(prediction, 0) + 1
            if len(outcomes) < 2:
                continue
            cost = 1 + sum(before[f] != v for f, v in zip(self.features, assignment))
            eliminated = len(self.version_space) - max(outcomes.values())
            # Minimax discrimination, then fewest real actions, then closest
            # structural feature first. No probabilities or learned weights.
            key: tuple[object, ...] = (-eliminated, cost, tuple(-int(v) for v in assignment))
            exp = Experiment(tuple(Literal(f, v) for f, v in zip(self.features, assignment)),
                             eliminated, cost, tuple(sorted(outcomes.values())))
            if best is None or key < best[0]:
                best = (key, exp)
        if best is None:
            raise CannotIdentify("Observationally indistinguishable hypotheses; refuse to guess")
        return best[1]

    def checked_action(self, port: ActionPort, before: Observation, action: str) -> Observation:
        after = port.send_action(action)
        Trace(before, action, after).validate()
        return after

    def set_literal(self, port: ActionPort, obs: Observation, binding: dict[Role, str], literal: Literal) -> Observation:
        if obs.value(binding[literal.role]) == literal.value:
            return obs
        action = self.intervention(obs.scene, binding, literal)
        after = self.checked_action(port, obs, action)
        expected = dict(obs.values) | {binding[literal.role]: literal.value}
        if dict(after.values) != expected:
            raise CannotIdentify("Advertised intervention contract violated")
        return after

    def rearm(self, port: ActionPort, obs: Observation, binding: dict[Role, str]) -> tuple[Observation, int]:
        count = 0
        for effect in self.rule.effects:
            if not obs.value(binding[effect.role]):
                continue
            clear = [e.source for e in obs.scene.edges
                     if e.relation == Relation.CLEAR and e.target == binding[effect.role]]
            if len(clear) != 1 or not effect.value:
                raise CannotIdentify("No calibrated way to re-arm this outcome")
            rearmed = self.checked_action(port, obs, clear[0])
            expected = dict(obs.values) | {binding[effect.role]: False}
            if dict(rearmed.values) != expected:
                raise CannotIdentify("Re-arm contract violated")
            obs = rearmed
            count += 1
        return obs, count

    def revise(self, port: ActionPort) -> dict[str, Json]:
        obs = port.send_action(OBSERVE)
        binding = self.binding(obs.scene)
        initial = self.normalize(obs, binding)
        prediction = self.rule.predict(initial)
        after = self.checked_action(port, obs, binding[self.rule.action])
        actual = self.normalize(after, binding)
        self.remember(initial, actual)
        if actual == prediction:
            raise CannotIdentify("No contradiction; this is not the acceptance scenario")
        self.emit("CONTRADICTION_FOUND", predicted=dict(prediction), observed=dict(actual))
        self.version_space = tuple(h for h in self.version_space if self.consistent(h))
        if not self.version_space:
            raise CannotIdentify("No consistent hypothesis in declared grammar")
        self.emit("REVISION_HYPOTHESES", count=len(self.version_space),
                  candidates=[h.to_wire() for h in self.version_space])
        experiments = 0
        task_attempts = 1
        interventions = 0
        obs = after
        while len(self.version_space) > 1:
            if experiments >= MAX_EXPERIMENTS:
                raise CannotIdentify("Experiment budget exceeded")
            # Successful interventions can leave the goal latched open. Re-arm
            # it explicitly and count the real action; never silently reset.
            obs, resets = self.rearm(port, obs, binding)
            interventions += resets
            exp = self.choose_experiment(self.normalize(obs, binding))
            self.emit("EXPERIMENT_SELECTED", assignment=[{"role": g.role, "value": g.value} for g in exp.assignment],
                      worst_case_eliminated=exp.worst_case_eliminated,
                      action_cost=exp.actions, partition_sizes=list(exp.partitions))
            for literal in exp.assignment:
                old_seq = obs.sequence
                obs = self.set_literal(port, obs, binding, literal)
                interventions += obs.sequence - old_seq
            before = self.normalize(obs, binding)
            obs = self.checked_action(port, obs, binding[self.rule.action])
            task_attempts += 1
            experiments += 1
            self.remember(before, self.normalize(obs, binding))
            self.version_space = tuple(h for h in self.version_space if self.consistent(h))
            self.emit("HYPOTHESES_FILTERED", remaining=len(self.version_space))
            if not self.version_space:
                raise CannotIdentify("Experiment falsified the declared hypothesis class")
        candidate = self.version_space[0]
        # Require a successful separating witness in the candidate's minimal
        # context before certification. This catches e.g. an omitted third
        # guard that negative evidence alone can disguise as a two-guard rule.
        first = dict(next(iter(self.witnesses)).before)
        selected = {g.role: g.value for g in candidate.guards}
        minimal = tuple(Literal(f, selected.get(f, not first[f])) for f in self.features)
        witnessed = any(all(g.holds(dict(w.before)) for g in minimal)
                        and w.before != w.after and candidate.predict(w.before) == w.after
                        for w in self.witnesses)
        if not witnessed:
            self.emit("CONFIRMATORY_EXPERIMENT", reason="positive_witness_in_minimal_guard_context")
            obs, resets = self.rearm(port, obs, binding)
            interventions += resets
            for literal in minimal:
                old_seq = obs.sequence
                obs = self.set_literal(port, obs, binding, literal)
                interventions += obs.sequence - old_seq
            before = self.normalize(obs, binding)
            obs = self.checked_action(port, obs, binding[self.rule.action])
            task_attempts += 1
            experiments += 1
            self.remember(before, self.normalize(obs, binding))
            if not self.consistent(candidate):
                self.version_space = ()
                raise CannotIdentify("Confirmatory intervention falsified the model class")
        old_rule = self.rule
        if not self.consistent(candidate):
            raise CannotIdentify("Historical counterexample; atomic commit refused")
        edits = old_rule.guard_edit_distance(candidate)
        # The sole mutation point. Other action schemas remain intact.
        self.rules = self.rules | {candidate.action: candidate}
        self.commits += 1
        self.emit("LOCAL_REVISION_COMMITTED", literal_edits=edits, experiments=experiments,
                  witnesses_preserved=sum(self.witnesses.values()), rule=candidate.to_wire())
        return {"experiments": experiments, "task_attempts": task_attempts,
                "interventions": interventions, "literal_edits": edits,
                "historical_consistency": self.consistent(candidate),
                "identified_within_declared_class": True}

    def transfer(self, port: ActionPort) -> dict[str, Json]:
        if len(self.version_space) != 1:
            raise CannotIdentify("Transfer requires an identified model")
        digest_before = self.memory_digest()
        obs = port.send_action(OBSERVE)
        binding = self.binding(obs.scene)
        self.emit("STRUCTURAL_MAPPING_VERIFIED", nodes=len(binding), exact_edges=True,
                  mapping=[[a, b] for a, b in sorted(binding.items())])
        prepared = 0
        for literal in self.rule.guards:
            old = obs.sequence
            obs = self.set_literal(port, obs, binding, literal)
            prepared += obs.sequence - old
        before = self.normalize(obs, binding)
        expected = self.rule.predict(before)
        after = self.checked_action(port, obs, binding[self.rule.action])
        actual = self.normalize(after, binding)
        if expected != actual or not all(e.holds(dict(actual)) for e in self.rule.effects):
            self.emit("TRANSFER_PREDICTION_FAILED")
            raise CannotIdentify("Same observable topology did not imply the same mechanism")
        unchanged = digest_before == self.memory_digest()
        if not unchanged:
            raise AssertionError("Transfer mutated learned memory")
        self.emit("TRANSFER_VERIFIED", first_task_attempt=True, preparation_actions=prepared,
                  adaptation_queries=0, model_unchanged=True)
        return {"first_task_attempt_success": True, "preparation_actions": prepared,
                "task_actions": 1, "adaptation_queries": 0, "model_unchanged": unchanged}

    def memory(self) -> dict[str, Json]:
        # Canonical learned state. Audit events/timestamps are excluded.
        return {"template": self.template.to_wire() if self.template else None,
                "roles": dict(self.roles), "features": list(self.features),
                "rules": {a: r.to_wire() for a, r in sorted(self.rules.items())},
                "hypotheses": [r.to_wire() for r in self.version_space],
                "witnesses": [{"before": dict(w.before), "after": dict(w.after), "count": count}
                              for w, count in sorted(self.witnesses.items(), key=lambda item: repr(item[0]))],
                "commits": self.commits}

    def memory_bytes(self) -> bytes:
        return json.dumps(self.memory(), sort_keys=True, separators=(",", ":")).encode()

    def memory_digest(self) -> str:
        return hashlib.sha256(self.memory_bytes()).hexdigest()

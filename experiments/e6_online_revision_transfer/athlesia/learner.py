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

from .graph import distances, isomorphism, induced_embedding
from .protocol import ActionPort, ContractError, Json, Kind, Observation, Relation, Scene, Trace, OBSERVE

Role: TypeAlias = str
Values: TypeAlias = tuple[tuple[Role, bool], ...]
MAX_FEATURES = 8
MAX_GUARDS = 2
MAX_WITNESSES = 64
MAX_EXPERIMENTS = 8
MAX_ACTION_SCHEMAS = 16


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
class MechanismCandidate:
    """Uncommitted alternative causal epoch, isolated from old witnesses."""
    hypotheses: tuple[Rule, ...]
    evidence: tuple[tuple[str, Witness], ...]


@dataclass(frozen=True, slots=True)
class EffectCandidate:
    """Candidate from a new single observed Boolean effect, not an oracle label."""
    effect: Literal
    evidence: tuple[tuple[str, Witness], ...]


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
        self.version_spaces: dict[Role, tuple[Rule, ...]] = {}
        self.witnesses_by_action: dict[Role, dict[Witness, int]] = {}
        self.events: list[dict[str, Json]] = []
        self.commits = 0
        self.pending_actions: set[Role] = set()
        self.mechanism_candidates: dict[Role, MechanismCandidate] = {}
        self.mechanism_archive: dict[Role, list[tuple[Rule, tuple[Rule, ...], dict[Witness, int]]]] = {}
        self.effect_candidates: dict[Role, EffectCandidate] = {}

    @property
    def rule(self) -> Rule:
        if self.primary is None:
            raise CannotIdentify("Untrained model")
        return self.rules[self.primary]

    def emit(self, name: str, **fields: Json) -> None:
        if len(self.events) >= 256:
            raise CannotIdentify("Audit-event budget exceeded")
        self.events.append({"event": name, **fields})

    def binding(self, scene: Scene, *, allow_extension: bool = False) -> dict[Role, str]:
        if self.template is None:
            raise CannotIdentify("No source graph")
        match = (induced_embedding(self.template, scene)
                 if allow_extension and len(scene.nodes) > len(self.template.nodes)
                 else isomorphism(self.template, scene))
        try:
            mapping = match.unique()
        except ValueError as ex:
            raise CannotIdentify(str(ex)) from ex
        return {self.roles[src]: dst for src, dst in mapping.items()}

    def normalize(self, obs: Observation, binding: dict[Role, str]) -> Values:
        raw = dict(obs.values)
        return tuple(sorted((role, raw[node]) for role, node in binding.items() if node in raw))

    def witness_store(self, action: Role | None = None) -> dict[Witness, int]:
        selected = self.primary if action is None else action
        if selected is None:
            return self.witnesses
        if selected == self.primary:
            self.witnesses_by_action[selected] = self.witnesses
            return self.witnesses
        return self.witnesses_by_action.setdefault(selected, {})

    def remember(self, before: Values, after: Values, action: Role | None = None) -> None:
        w = Witness(before, after)
        store = self.witness_store(action)
        distinct = len(self.witnesses) + sum(
            len(items) for role, items in self.witnesses_by_action.items()
            if role != self.primary
        )
        if w not in store and distinct >= MAX_WITNESSES:
            raise CannotIdentify("Evidence capacity reached; no silent forgetting")
        store[w] = store.get(w, 0) + 1

    def consistent(self, rule: Rule) -> bool:
        return all(rule.predict(w.before) == w.after
                   for w in self.witness_store(rule.action))

    def set_primary_version_space(self, hypotheses: tuple[Rule, ...]) -> None:
        if self.primary is None:
            raise CannotIdentify("Untrained model")
        self.version_space = hypotheses
        self.version_spaces[self.primary] = hypotheses

    def unrelated_model_digest(self) -> str:
        """Canonical digest of every learned action model except the primary."""
        payload = {
            "rules": {action: rule.to_wire() for action, rule in sorted(self.rules.items())
                      if action != self.primary},
            "hypotheses": {
                action: [rule.to_wire() for rule in hypotheses]
                for action, hypotheses in sorted(self.version_spaces.items())
                if action != self.primary
            },
            "witnesses": {
                action: [{"before": dict(w.before), "after": dict(w.after), "count": count}
                         for w, count in sorted(items.items(), key=lambda item: repr(item[0]))]
                for action, items in sorted(self.witnesses_by_action.items())
                if action != self.primary
            },
        }
        data = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
        return hashlib.sha256(data).hexdigest()

    @staticmethod
    def observed_delta(witness: Witness) -> tuple[Literal, ...]:
        before = dict(witness.before)
        return tuple(sorted(Literal(role, value) for role, value in witness.after
                            if before[role] != value))

    def induce_action_schema(
        self,
        action: Role,
        context: tuple[tuple[Role, Relation, Role], ...],
    ) -> tuple[Rule, tuple[Rule, ...]]:
        """Induce one bounded deterministic schema from action-local evidence.

        The effect signature and every guard candidate come from observed
        before/after traces.  No hidden rule, semantic action name, or evaluator
        label enters this method.
        """
        store = self.witness_store(action)
        changed = {self.observed_delta(w) for w in store if self.observed_delta(w)}
        if not changed:
            raise CannotIdentify("No observed effect for action schema")
        if len(changed) != 1:
            raise CannotIdentify("Nonuniform observed effect for action schema")
        effects = next(iter(changed))
        effect_roles = {effect.role for effect in effects}
        positive = tuple(w for w in store if self.observed_delta(w) == effects)
        literals: list[Literal] = []
        for feature in self.features:
            if feature in effect_roles:
                continue
            observed = {dict(w.before)[feature] for w in positive}
            if len(observed) == 1:
                literals.append(Literal(feature, observed.pop()))
        if len(literals) > MAX_FEATURES:
            raise CannotIdentify("Action-local feature budget exceeded")
        base = Rule(action, context, (), effects)
        hypotheses = tuple(
            replace(base, guards=tuple(guards))
            for size in range(min(MAX_GUARDS, len(literals)) + 1)
            for guards in itertools.combinations(literals, size)
            if self.consistent(replace(base, guards=tuple(guards)))
        )
        if not hypotheses:
            raise CannotIdentify("No consistent action schema in declared grammar")
        current = min(hypotheses, key=lambda rule: (len(rule.guards), rule.guards))
        return current, hypotheses

    def predict_public(self, observation: Observation, action_token: str) -> Values:
        """Predict a complete public successor state without mutating memory."""
        binding = self.binding(observation.scene)
        reverse = {node: role for role, node in binding.items()}
        if action_token not in reverse or reverse[action_token] not in self.rules:
            raise CannotIdentify("No induced schema for requested action")
        if reverse[action_token] in self.pending_actions:
            raise CannotIdentify("Unresolved online contradiction; refusing prediction")
        prediction = self.rules[reverse[action_token]].predict(
            self.normalize(observation, binding)
        )
        return tuple(sorted((binding[role], value) for role, value in prediction))

    def observe_effect_shift(self, trace: Trace) -> dict[str, Json]:
        """Quarantine and relearn ONE new Boolean effect of an unguarded action.

        Grammar: existing identified zero-guard action, exactly one changed
        observable on a positive trace; the new effect may target another
        grounded Boolean node or reverse polarity. Certification needs two
        distinct-episode positive transitions plus an idempotent no-op witness.
        No simulator internals, reset oracle, probabilities or action semantics.
        This is *observational*, not autonomously chosen experimentation.
        """
        trace.validate()
        if self.template is None:
            raise CannotIdentify("Fit required before effect shift")
        binding = self.binding(trace.before.scene)
        reverse = {node: role for role, node in binding.items()}
        action = reverse.get(trace.action)
        if action is None or action not in self.rules:
            raise CannotIdentify("Unknown learned action for effect shift")
        if action in self.mechanism_candidates:
            raise CannotIdentify("Guard epoch unresolved; effect shift blocked")
        previous = self.rules[action]
        if previous.guards or len(self.version_spaces.get(action, ())) != 1:
            raise CannotIdentify("Effect pilot requires identified unguarded action")
        before = self.normalize(trace.before, binding)
        after = self.normalize(trace.after, binding)
        witness = Witness(before, after)
        changed = self.observed_delta(witness)
        candidate = self.effect_candidates.get(action)
        if candidate is None:
            if action in self.pending_actions:
                raise CannotIdentify("Action already uncertain; effect shift blocked")
            if previous.predict(before) == after:
                return {"status": "unchanged", "local_commit": False}
            if len(changed) != 1:
                raise CannotIdentify("Effect signature not singly identifiable; atomic rejection")
            effect = changed[0]
            if effect == previous.effects[0] and len(previous.effects) == 1:
                raise CannotIdentify("Effect witness does not establish a new signature")
            if len(self.events) >= 256:
                raise CannotIdentify("Audit-event capacity reached; atomic rejection")
            self.effect_candidates[action] = EffectCandidate(
                effect, ((trace.before.episode, witness),))
            self.pending_actions.add(action)
            self.emit("EFFECT_SHIFT_QUARANTINED", action_role=action,
                      new_effect={"role": effect.role, "value": effect.value})
            return {"status": "quarantined", "local_commit": False,
                    "observations": 1}

        if len(candidate.evidence) >= MAX_WITNESSES:
            raise CannotIdentify("Effect evidence capacity reached; atomic rejection")
        if any(episode == trace.before.episode for episode, _ in candidate.evidence):
            raise CannotIdentify("Repeated effect episode; atomic rejection")
        proposed = replace(previous, guards=(), effects=(candidate.effect,))
        if proposed.predict(before) != after:
            raise CannotIdentify("Contradictory effect-epoch observation; atomic rejection")
        evidence = candidate.evidence + ((trace.before.episode, witness),)
        positives = sum(w.before != w.after for _, w in evidence)
        negatives = len(evidence) - positives
        certified = len(evidence) >= 3 and positives >= 2 and negatives >= 1
        if len(self.events) + (2 if certified else 1) > 256:
            raise CannotIdentify("Audit-event capacity reached; atomic rejection")
        if certified:
            history = self.mechanism_archive.get(action, [])
            if len(history) >= 4:
                raise CannotIdentify("Mechanism-epoch archive full; atomic rejection")
            fresh: dict[Witness, int] = {}
            for _, item in evidence:
                fresh[item] = fresh.get(item, 0) + 1
            distinct = len(self.witnesses) + sum(
                len(items) for role, items in self.witnesses_by_action.items()
                if role != self.primary)
            existing = len(self.witness_store(action))
            if distinct - existing + len(fresh) > MAX_WITNESSES:
                raise CannotIdentify("New effect evidence capacity exceeded; atomic rejection")
            snapshot = (previous, self.version_spaces[action],
                        dict(self.witness_store(action)))
            self.mechanism_archive[action] = [*history, snapshot]
            self.rules[action] = proposed
            self.version_spaces[action] = (proposed,)
            self.witnesses_by_action[action] = fresh
            if action == self.primary:
                self.version_space = (proposed,)
                self.witnesses = fresh
            self.effect_candidates.pop(action)
            self.pending_actions.discard(action)
            self.commits += 1
            self.emit("EFFECT_SHIFT_EVIDENCE", action_role=action,
                      observations=len(evidence), positives=positives,
                      idempotent_witnesses=negatives)
            self.emit("EFFECT_EPOCH_COMMITTED", action_role=action,
                      new_effect={"role": candidate.effect.role,
                                  "value": candidate.effect.value},
                      archived_epochs=len(history) + 1)
            return {"status": "committed", "local_commit": True,
                    "observations": len(evidence), "positive_witnesses": positives,
                    "idempotent_witnesses": negatives}
        self.effect_candidates[action] = EffectCandidate(candidate.effect, evidence)
        self.emit("EFFECT_SHIFT_EVIDENCE", action_role=action,
                  observations=len(evidence), positives=positives,
                  idempotent_witnesses=negatives)
        return {"status": "quarantined", "local_commit": False,
                "observations": len(evidence)}

    def choose_active_guard_probe(self, observation: Observation, action_token: str) -> dict[str, object]:
        """Read-only bounded minimax experiment design for a quarantined guard epoch.

        Enumerates at most 2**MAX_FEATURES Boolean assignments. All hypotheses
        see identical trial inputs; no source-world law or private seed is read.
        When only one hypothesis remains, deliberately gathers any missing
        positive/negative witness needed by the Day5a certification contract.
        """
        if self.template is None:
            raise CannotIdentify("Fit required for active guard probing")
        binding = self.binding(observation.scene)
        reverse = {node: role for role, node in binding.items()}
        action = reverse.get(action_token)
        candidate = self.mechanism_candidates.get(action) if action else None
        if candidate is None or action not in self.pending_actions:
            raise CannotIdentify("No quarantined guard epoch for requested action")
        if action in self.effect_candidates:
            raise CannotIdentify("Effect and guard epochs may not overlap")
        if any(ep == observation.episode for ep, _ in candidate.evidence):
            raise CannotIdentify("Active probe requires a fresh episode")
        if len(candidate.evidence) >= MAX_WITNESSES:
            raise CannotIdentify("Active probe evidence budget exhausted")
        hypotheses = candidate.hypotheses
        if not hypotheses:
            raise CannotIdentify("No surviving guard hypotheses")
        effect_roles = {e.role for e in self.rules[action].effects}
        toggles = tuple(f for f in self.features if f not in effect_roles)
        if len(toggles) > MAX_FEATURES:
            raise CannotIdentify("Active probe feature capacity exceeded")
        initial = dict(self.normalize(observation, binding))
        if any(f not in initial for f in toggles):
            raise CannotIdentify("Missing Boolean state in active probe")
        positives = sum(w.before != w.after for _, w in candidate.evidence)
        negatives = len(candidate.evidence) - positives
        desired_polarity = True if positives == 0 else (False if negatives == 0 else None)
        best: tuple[tuple[object, ...], dict[str, object]] | None = None
        for bits in itertools.product((False, True), repeat=len(toggles)):
            trial = initial | dict(zip(toggles, bits))
            trial_values = tuple(sorted(trial.items()))
            partitions: dict[Values, int] = {}
            for h in hypotheses:
                predicted = h.predict(trial_values)
                partitions[predicted] = partitions.get(predicted, 0) + 1
            worst = len(hypotheses) - max(partitions.values())
            # No distinguished outcome means no information with >1 models.
            if len(hypotheses) > 1 and worst == 0:
                continue
            changes = tuple(Literal(role, value) for role, value in zip(toggles, bits)
                            if initial[role] != value)
            if 1 + len(changes) > MAX_EXPERIMENTS:
                continue
            try:
                # Check EVERY advertised calibration token before any action.
                for literal in changes:
                    self.intervention(observation.scene, binding, literal)
            except CannotIdentify:
                continue
            if len(hypotheses) == 1:
                is_positive = next(iter(partitions)) != trial_values
                polarity_penalty = int(desired_polarity is not None and
                                       is_positive != desired_polarity)
                key: tuple[object, ...] = (polarity_penalty, len(changes), bits)
            else:
                key = (-worst, len(changes), bits)
            plan: dict[str, object] = {
                "action_role": action,
                "assignment": changes,
                "worst_case_eliminated": worst,
                "partition_sizes": tuple(sorted(partitions.values())),
                "preparation_actions": len(changes),
                "candidate_models": len(hypotheses),
            }
            if best is None or key < best[0]:
                best = (key, plan)
        if best is None:
            raise CannotIdentify("No safe identifiable active guard experiment")
        if len(hypotheses) == 1 and desired_polarity is not None and best[0][0] != 0:
            raise CannotIdentify("Required positive/negative witness not reachable")
        return best[1]

    def run_active_guard_probe(self, port: ActionPort, action_token: str) -> dict[str, Json]:
        """Perform a selected intervention and submit its real transition.

        One trial per externally supplied fresh episode. Calibration actions are
        charged, post-action states are verified, and model replacement is
        delegated solely to the existing Day5a evidence/epoch machinery.
        """
        obs = port.send_action(OBSERVE)
        plan = self.choose_active_guard_probe(obs, action_token)
        binding = self.binding(obs.scene)
        for literal in plan["assignment"]:
            assert isinstance(literal, Literal)
            obs = self.set_literal(port, obs, binding, literal)
        # No hypothetical successor may replace this actual observed trace.
        after = self.checked_action(port, obs, action_token)
        update = self.observe_mechanism_shift(Trace(obs, action_token, after))
        return {**update, "action": action_token,
                "preparation_actions": plan["preparation_actions"],
                "probe_actions": 1,
                "candidate_models_before": plan["candidate_models"],
                "worst_case_eliminated": plan["worst_case_eliminated"],
                "partition_sizes": list(plan["partition_sizes"]),
                "active_selection": True}

    def observe_mechanism_shift(self, trace: Trace) -> dict[str, Json]:
        """Reidentify an existing action's *guard* after a confirmed contradiction.

        Only the original Boolean effect signature and 0..MAX_GUARDS conjunctive
        guard literals are supported. Old witnesses stay in an archived epoch;
        they are NOT mixed into the replacement hypothesis space. A single
        anomaly quarantines this action; it cannot cause an immediate commit.
        Episode identifiers are public observational provenance, not oracle law.
        """
        trace.validate()
        if self.template is None:
            raise CannotIdentify("Fit required before mechanism shift")
        binding = self.binding(trace.before.scene)
        reverse = {token: role for role, token in binding.items()}
        action = reverse.get(trace.action)
        if action is None or action not in self.rules:
            raise CannotIdentify("Unknown action for mechanism shift")
        old_rule = self.rules[action]
        if action in self.effect_candidates:
            raise CannotIdentify("Effect epoch unresolved; guard shift blocked")
        before = self.normalize(trace.before, binding)
        after = self.normalize(trace.after, binding)
        # Do not silently learn a new effect signature in this restricted pilot.
        if any(dict(before)[role] != value for role, value in after
               if role not in {effect.role for effect in old_rule.effects}):
            raise CannotIdentify("Unmodelled changed fluent; atomic rejection")
        witness = Witness(before, after)
        candidate = self.mechanism_candidates.get(action)
        if candidate is None:
            if action in self.pending_actions or len(self.version_spaces.get(action, ())) != 1:
                raise CannotIdentify("Unidentified existing mechanism; cannot infer drift")
            if old_rule.predict(before) == after:
                return {"status": "unchanged", "local_commit": False,
                        "remaining_hypotheses": 1}
            # This is a contradiction in a previously identified model.
            # Construct a separate bounded class without using historical data.
            roles = tuple(role for role in self.features
                          if role not in {effect.role for effect in old_rule.effects})
            all_models: list[Rule] = []
            for size in range(min(MAX_GUARDS, len(roles)) + 1):
                for subset in itertools.combinations(roles, size):
                    for polarity in itertools.product((False, True), repeat=size):
                        guards = tuple(Literal(role, val) for role, val in zip(subset, polarity))
                        all_models.append(replace(old_rule, guards=guards))
            hypotheses = tuple(rule for rule in all_models
                               if rule.predict(before) == after)
            if not hypotheses:
                raise CannotIdentify("Shift outside fixed effect/guard class; atomic rejection")
            if len(self.events) >= 256:
                raise CannotIdentify("Audit-event capacity reached; atomic rejection")
            self.mechanism_candidates[action] = MechanismCandidate(
                hypotheses, ((trace.before.episode, witness),))
            self.pending_actions.add(action)
            self.emit("MECHANISM_SHIFT_QUARANTINED", action_role=action,
                      remaining_hypotheses=len(hypotheses))
            return {"status": "quarantined", "local_commit": False,
                    "remaining_hypotheses": len(hypotheses)}

        evidence = candidate.evidence + ((trace.before.episode, witness),)
        if len(evidence) > MAX_WITNESSES:
            raise CannotIdentify("Shift evidence limit reached; atomic rejection")
        if any(previous_episode == trace.before.episode for previous_episode, _ in candidate.evidence):
            raise CannotIdentify("Repeated episode in shift evidence; atomic rejection")
        hypotheses = tuple(rule for rule in candidate.hypotheses
                           if rule.predict(before) == after)
        if not hypotheses:
            raise CannotIdentify("Conflicting new-epoch evidence; atomic rejection")
        positives = sum(w.before != w.after for _, w in evidence)
        negatives = len(evidence) - positives
        certified = len(hypotheses) == 1 and len(evidence) >= 3 and positives >= 1 and negatives >= 1
        if len(self.events) + (2 if certified else 1) > 256:
            raise CannotIdentify("Audit-event capacity reached; atomic rejection")
        if certified:
            history = self.mechanism_archive.get(action, [])
            if len(history) >= 4:
                raise CannotIdentify("Mechanism-epoch archive full; atomic rejection")
            distinct = len(self.witnesses) + sum(len(store) for role, store in
                        self.witnesses_by_action.items() if role != self.primary)
            previous_count = len(self.witness_store(action))
            fresh_store: dict[Witness, int] = {}
            for _, item in evidence:
                fresh_store[item] = fresh_store.get(item, 0) + 1
            if distinct - previous_count + len(fresh_store) > MAX_WITNESSES:
                raise CannotIdentify("New epoch witness budget exceeded; atomic rejection")
            snapshot = (old_rule, self.version_spaces[action], dict(self.witness_store(action)))
            # Single semantic commit: only this action's current epoch is replaced.
            self.mechanism_archive[action] = [*history, snapshot]
            self.rules[action] = hypotheses[0]
            self.version_spaces[action] = hypotheses
            self.witnesses_by_action[action] = fresh_store
            if action == self.primary:
                self.witnesses = fresh_store
                self.version_space = hypotheses
            self.mechanism_candidates.pop(action)
            self.pending_actions.discard(action)
            self.commits += 1
            self.emit("MECHANISM_SHIFT_EVIDENCE", action_role=action,
                      observations=len(evidence), remaining_hypotheses=1)
            self.emit("MECHANISM_EPOCH_COMMITTED", action_role=action,
                      new_rule=hypotheses[0].to_wire(), previous_epochs=len(history) + 1)
            return {"status": "committed", "local_commit": True,
                    "remaining_hypotheses": 1, "observations": len(evidence),
                    "archived_epochs": len(history) + 1}

        self.mechanism_candidates[action] = MechanismCandidate(hypotheses, evidence)
        self.emit("MECHANISM_SHIFT_EVIDENCE", action_role=action,
                  observations=len(evidence), remaining_hypotheses=len(hypotheses))
        return {"status": "quarantined", "local_commit": False,
                "remaining_hypotheses": len(hypotheses), "observations": len(evidence)}

    def observe_transition(self, trace: Trace) -> dict[str, Json]:
        """Admit one externally observed transition into an action-local version space.

        The observed successor is never silently replaced by a prediction.
        Unexpected transitions outside the declared model class leave all learned
        state and audit events unchanged. An ambiguous contradiction makes only
        that action's public predictions unavailable until identification.
        """
        trace.validate()
        if self.template is None:
            raise CannotIdentify("Fit required before online validation")
        binding = self.binding(trace.before.scene)
        reverse = {token: role for role, token in binding.items()}
        action = reverse.get(trace.action)
        if action is None or action not in self.rules or action not in self.version_spaces:
            raise CannotIdentify("Action has no learned hypothesis space")
        if action in self.mechanism_candidates or action in self.effect_candidates:
            raise CannotIdentify("Mechanism epoch unresolved; ordinary learning blocked")
        before = self.normalize(trace.before, binding)
        after = self.normalize(trace.after, binding)
        hypotheses = tuple(rule for rule in self.version_spaces[action]
                           if rule.predict(before) == after)
        # Prepare everything that can fail before changing learned state.
        if not hypotheses:
            raise CannotIdentify("Transition outside declared hypothesis class; atomic rejection")
        witness = Witness(before, after)
        store = self.witness_store(action)
        distinct = len(self.witnesses) + sum(
            len(items) for role, items in self.witnesses_by_action.items()
            if role != self.primary)
        if witness not in store and distinct >= MAX_WITNESSES:
            raise CannotIdentify("Evidence capacity reached; atomic rejection")
        previous = self.rules[action]
        contradicted = previous.predict(before) != after
        selected = hypotheses[0] if len(hypotheses) == 1 else None
        commit = selected is not None and selected != previous
        edits = previous.guard_edit_distance(selected) if commit and selected is not None else 0
        if len(self.events) + (2 if commit else 1) > 256:
            raise CannotIdentify("Audit-event capacity reached; atomic rejection")
        # All admissibility checks passed: append evidence and update ONLY this action.
        self.remember(before, after, action)
        self.version_spaces[action] = hypotheses
        if action == self.primary:
            self.version_space = hypotheses
        if selected is not None:
            self.pending_actions.discard(action)
        elif contradicted:
            self.pending_actions.add(action)
        if commit and selected is not None:
            self.rules[action] = selected
            self.commits += 1
        status = "committed" if commit else (
            "deferred" if action in self.pending_actions else "validated")
        self.emit("ONLINE_TRANSITION_VALIDATED", action_role=action,
                  contradiction=contradicted, remaining_hypotheses=len(hypotheses),
                  status=status)
        if commit and selected is not None:
            self.emit("ONLINE_LOCAL_REVISION_COMMITTED", action_role=action,
                      literal_edits=edits, rule=selected.to_wire())
        return {"status": status, "action_role": action,
                "contradiction": contradicted,
                "remaining_hypotheses": len(hypotheses), "local_commit": commit}

    def fit(self, traces: tuple[Trace, ...]) -> None:
        if self.template is not None or not 1 <= len(traces) <= 32:
            raise CannotIdentify("Fit requires a fresh agent and 1..32 demonstrations")
        first = traces[0]
        first.validate()
        self.template = first.before.scene
        # Alpha variables are internal names, not guessed token meanings.
        self.roles = {n.id: f"?v{i}" for i, n in enumerate(self.template.nodes)}
        self.primary = self.roles[first.action]
        self.witnesses_by_action[self.primary] = self.witnesses
        distance = distances(self.template, first.action)
        features = sorted((n.id for n in self.template.nodes if n.kind == Kind.FLUENT),
                          key=lambda n: distance.get(n, 1_000_000))
        if not 1 <= len(features) <= MAX_FEATURES:
            raise CannotIdentify("Feature budget exceeded")
        # Equidistant features are allowed: ordering affects probe cost, not a
        # certified mapping. Graph ambiguity is checked separately.
        self.features = tuple(self.roles[n] for n in features)
        action_counts: dict[Role, int] = {}
        for trace in traces:
            trace.validate()
            binding = self.binding(trace.before.scene)
            reverse = {v: k for k, v in binding.items()}
            action = reverse[trace.action]
            before, after = self.normalize(trace.before, binding), self.normalize(trace.after, binding)
            self.remember(before, after, action)
            action_counts[action] = action_counts.get(action, 0) + 1
        if not 1 <= len(action_counts) <= MAX_ACTION_SCHEMAS:
            raise CannotIdentify("Action-schema budget exceeded")
        context = tuple((self.roles[e.source], e.relation, self.roles[e.target])
                        for e in self.template.edges if e.relation == Relation.LINK)
        for action in sorted(action_counts):
            rule, hypotheses = self.induce_action_schema(action, context)
            self.rules[action] = rule
            self.version_spaces[action] = hypotheses
            self.emit("ACTION_SCHEMA_INDUCED", action_role=action,
                      demonstrations=action_counts[action],
                      observed_effects=[{"role": effect.role, "value": effect.value}
                                        for effect in rule.effects],
                      remaining_hypotheses=len(hypotheses), rule=rule.to_wire())
        if self.primary not in self.rules:
            raise CannotIdentify("Primary action has no identifiable observed effect")
        self.set_primary_version_space(self.version_spaces[self.primary])
        self.emit("NAIVE_RULE_INDUCED", demonstrations=action_counts[self.primary],
                  observed_action_schemas=len(self.rules),
                  remaining_hypotheses=len(self.version_space), exact_identification=False,
                  rule=self.rule.to_wire())

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
        if self.primary in self.pending_actions:
            raise CannotIdentify("Mechanism epoch unresolved; revision blocked")
        unrelated_before = self.unrelated_model_digest()
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
        self.set_primary_version_space(
            tuple(h for h in self.version_space if self.consistent(h))
        )
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
            self.set_primary_version_space(
                tuple(h for h in self.version_space if self.consistent(h))
            )
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
                self.set_primary_version_space(())
                raise CannotIdentify("Confirmatory intervention falsified the model class")
        old_rule = self.rule
        if not self.consistent(candidate):
            raise CannotIdentify("Historical counterexample; atomic commit refused")
        edits = old_rule.guard_edit_distance(candidate)
        # The sole mutation point. Other action schemas remain intact.
        self.rules = self.rules | {candidate.action: candidate}
        self.set_primary_version_space((candidate,))
        unrelated_unchanged = unrelated_before == self.unrelated_model_digest()
        if not unrelated_unchanged:
            raise AssertionError("Local revision mutated an unrelated action schema")
        self.commits += 1
        self.emit("LOCAL_REVISION_COMMITTED", literal_edits=edits, experiments=experiments,
                  witnesses_preserved=sum(self.witnesses.values()),
                  unrelated_action_schemas=len(self.rules) - 1,
                  unrelated_models_unchanged=True, rule=candidate.to_wire())
        return {"experiments": experiments, "task_attempts": task_attempts,
                "interventions": interventions, "literal_edits": edits,
                "historical_consistency": self.consistent(candidate),
                "unrelated_models_unchanged": unrelated_unchanged,
                "identified_within_declared_class": True}

    def solve_public_goal(self, port: ActionPort, target_node: str, desired: bool) -> dict[str, Json]:
        """Execute a new *declared public goal* using existing learned action effects.

        This does not discover an unseen predicate, a novel action mechanism, or
        a goal intrinsically. It accepts an external Boolean target and acts
        only when surviving action hypotheses agree on its full successor.
        """
        if type(desired) is not bool or not isinstance(target_node, str):
            raise CannotIdentify("Goal must be an explicit Boolean observable token")
        if not self.rules or self.template is None:
            raise CannotIdentify("Fit required before goal execution")
        obs = port.send_action(OBSERVE)
        binding = self.binding(obs.scene, allow_extension=True)
        reverse = {node: role for role, node in binding.items()}
        if target_node not in dict(obs.values) or target_node not in reverse:
            raise CannotIdentify("Goal not grounded in learned observable roles")
        target_role = reverse[target_node]
        normalized = self.normalize(obs, binding)
        if dict(normalized)[target_role] == desired:
            return {"status": "already_satisfied", "target": target_node,
                    "desired": desired, "actions": 0, "model_unchanged": True,
                    "full_state_verified": True}
        candidates: list[tuple[Role, Values]] = []
        for action_role in sorted(self.rules):
            if action_role in self.pending_actions:
                continue
            space = self.version_spaces.get(action_role, ())
            if not space:
                continue
            predictions = {rule.predict(normalized) for rule in space}
            if len(predictions) != 1:
                continue  # uncertainty is not resolved by choosing one convenient model
            successor = next(iter(predictions))
            if dict(successor)[target_role] == desired:
                candidates.append((action_role, successor))
        if len(candidates) != 1:
            raise CannotIdentify("Goal has no unique hypothesis-agreed learned action")
        action_role, successor = candidates[0]
        if action_role not in binding:
            raise CannotIdentify("Learned action not mapped in current scene")
        action_token = binding[action_role]
        memory_before = self.memory_digest()
        actual = self.checked_action(port, obs, action_token)
        expected_full = dict(obs.values)
        expected_full.update({binding[role]: value for role, value in successor})
        if dict(actual.values) != expected_full or actual.value(target_node) != desired:
            raise CannotIdentify("Goal action falsified learned transition; no success reported")
        if self.memory_digest() != memory_before:
            raise AssertionError("Read-only goal execution mutated learned memory")
        return {"status": "achieved", "target": target_node, "desired": desired,
                "chosen_action": action_token, "actions": 1,
                "model_unchanged": True, "full_state_verified": True}

    def transfer(self, port: ActionPort) -> dict[str, Json]:
        if self.primary in self.pending_actions:
            raise CannotIdentify("Mechanism epoch unresolved; transfer blocked")
        if len(self.version_space) != 1:
            raise CannotIdentify("Transfer requires an identified model")
        digest_before = self.memory_digest()
        obs = port.send_action(OBSERVE)
        binding = self.binding(obs.scene, allow_extension=True)
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
        # Projection equality is insufficient on new topologies: any changed
        # unmapped fluent/goal is an unpredicted side-effect and must fail.
        full_prediction = dict(obs.values)
        full_prediction.update({binding[role]: value for role, value in expected})
        if (expected != actual or dict(after.values) != full_prediction
                or not all(e.holds(dict(actual)) for e in self.rule.effects)):
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
                "hypotheses_by_action": {
                    action: [rule.to_wire() for rule in hypotheses]
                    for action, hypotheses in sorted(self.version_spaces.items())
                },
                "witnesses_by_action": {
                    action: [{"before": dict(w.before), "after": dict(w.after), "count": count}
                             for w, count in sorted(items.items(), key=lambda item: repr(item[0]))]
                    for action, items in sorted(self.witnesses_by_action.items())
                },
                "commits": self.commits,
                "pending_actions": sorted(self.pending_actions),
                "mechanism_candidates": {
                    action: {"hypotheses": [r.to_wire() for r in item.hypotheses],
                             "evidence": [{"episode": ep, "before": dict(w.before),
                                           "after": dict(w.after)} for ep, w in item.evidence]}
                    for action, item in sorted(self.mechanism_candidates.items())},
                "mechanism_archive": {
                    action: [{"rule": old.to_wire(),
                              "hypotheses": [r.to_wire() for r in space],
                              "witnesses": [{"before": dict(w.before), "after": dict(w.after),
                                             "count": count} for w, count in
                                            sorted(store.items(), key=lambda kv: repr(kv[0]))]}
                             for old, space, store in snapshots]
                    for action, snapshots in sorted(self.mechanism_archive.items())},
                "effect_candidates": {
                    action: {"effect": {"role": item.effect.role,
                                        "value": item.effect.value},
                             "evidence": [{"episode": episode,
                                           "before": dict(w.before),
                                           "after": dict(w.after)}
                                          for episode, w in item.evidence]}
                    for action, item in sorted(self.effect_candidates.items())}}

    def memory_bytes(self) -> bytes:
        return json.dumps(self.memory(), sort_keys=True, separators=(",", ":")).encode()

    def memory_digest(self) -> str:
        return hashlib.sha256(self.memory_bytes()).hexdigest()

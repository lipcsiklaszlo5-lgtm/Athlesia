use athlesia_recursive::RecursiveUnit;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRule {
    premises: Vec<RecursiveUnit>,
    conclusions: Vec<RecursiveUnit>,
}

impl RecursiveWorldRule {
    pub fn new(premises: Vec<RecursiveUnit>, conclusions: Vec<RecursiveUnit>) -> Option<Self> {
        if premises.is_empty() || conclusions.is_empty() {
            return None;
        }

        let mut premises = premises;

        premises.sort();
        premises.dedup();

        let mut conclusions = conclusions;

        conclusions.sort();
        conclusions.dedup();

        Some(Self {
            premises,
            conclusions,
        })
    }

    pub fn premises(&self) -> &[RecursiveUnit] {
        &self.premises
    }

    pub fn conclusions(&self) -> &[RecursiveUnit] {
        &self.conclusions
    }

    pub fn premise_count(&self) -> usize {
        self.premises.len()
    }

    pub fn conclusion_count(&self) -> usize {
        self.conclusions.len()
    }

    pub fn contains_premise(&self, unit: &RecursiveUnit) -> bool {
        self.premises.binary_search(unit).is_ok()
    }

    pub fn contains_conclusion(&self, unit: &RecursiveUnit) -> bool {
        self.conclusions.binary_search(unit).is_ok()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldModel {
    rules: Vec<RecursiveWorldRule>,
}

impl RecursiveWorldModel {
    pub fn new(mut rules: Vec<RecursiveWorldRule>) -> Self {
        rules.sort();
        rules.dedup();

        Self { rules }
    }

    pub fn rules(&self) -> &[RecursiveWorldRule] {
        &self.rules
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn contains(&self, rule: &RecursiveWorldRule) -> bool {
        self.rules.binary_search(rule).is_ok()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldContradictionCandidate {
    left: RecursiveWorldRule,
    right: RecursiveWorldRule,
}

impl RecursiveWorldContradictionCandidate {
    pub fn new(left: RecursiveWorldRule, right: RecursiveWorldRule) -> Option<Self> {
        if left == right {
            return None;
        }

        if left.premises() != right.premises() {
            return None;
        }

        if left.conclusions() == right.conclusions() {
            return None;
        }

        let (left, right) = if left <= right {
            (left, right)
        } else {
            (right, left)
        };

        Some(Self { left, right })
    }

    pub fn left(&self) -> &RecursiveWorldRule {
        &self.left
    }

    pub fn right(&self) -> &RecursiveWorldRule {
        &self.right
    }

    pub fn premises(&self) -> &[RecursiveUnit] {
        self.left.premises()
    }

    pub fn shares_conclusion(&self) -> bool {
        self.left
            .conclusions()
            .iter()
            .any(|unit| self.right.contains_conclusion(unit))
    }

    pub fn is_disjoint(&self) -> bool {
        !self.shares_conclusion()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldContradictionSet {
    candidates: Vec<RecursiveWorldContradictionCandidate>,
}

impl RecursiveWorldContradictionSet {
    pub fn detect(model: &RecursiveWorldModel) -> Self {
        let rules = model.rules();

        let mut candidates = Vec::new();

        for left_index in 0..rules.len() {
            for right_index in (left_index + 1)..rules.len() {
                if let Some(candidate) = RecursiveWorldContradictionCandidate::new(
                    rules[left_index].clone(),
                    rules[right_index].clone(),
                ) {
                    candidates.push(candidate);
                }
            }
        }

        candidates.sort();
        candidates.dedup();

        Self { candidates }
    }

    pub fn candidates(&self) -> &[RecursiveWorldContradictionCandidate] {
        &self.candidates
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn disjoint_count(&self) -> usize {
        self.candidates
            .iter()
            .filter(|candidate| candidate.is_disjoint())
            .count()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldDependencyEdge {
    source: RecursiveWorldRule,
    target: RecursiveWorldRule,
}

impl RecursiveWorldDependencyEdge {
    pub fn new(source: RecursiveWorldRule, target: RecursiveWorldRule) -> Option<Self> {
        if source == target {
            return None;
        }

        let depends = source
            .conclusions()
            .iter()
            .any(|unit| target.contains_premise(unit));

        if !depends {
            return None;
        }

        Some(Self { source, target })
    }

    pub fn source(&self) -> &RecursiveWorldRule {
        &self.source
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn shared_units(&self) -> Vec<RecursiveUnit> {
        self.source
            .conclusions()
            .iter()
            .filter(|unit| self.target.contains_premise(unit))
            .cloned()
            .collect()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldDependencyGraph {
    edges: Vec<RecursiveWorldDependencyEdge>,
}

impl RecursiveWorldDependencyGraph {
    pub fn detect(model: &RecursiveWorldModel) -> Self {
        let rules = model.rules();

        let mut edges = Vec::new();

        for source_index in 0..rules.len() {
            for target_index in 0..rules.len() {
                if source_index == target_index {
                    continue;
                }

                if let Some(edge) = RecursiveWorldDependencyEdge::new(
                    rules[source_index].clone(),
                    rules[target_index].clone(),
                ) {
                    edges.push(edge);
                }
            }
        }

        edges.sort();
        edges.dedup();

        Self { edges }
    }

    pub fn edges(&self) -> &[RecursiveWorldDependencyEdge] {
        &self.edges
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn contains(&self, edge: &RecursiveWorldDependencyEdge) -> bool {
        self.edges.binary_search(edge).is_ok()
    }

    pub fn outgoing_count(&self, rule: &RecursiveWorldRule) -> usize {
        self.edges
            .iter()
            .filter(|edge| edge.source() == rule)
            .count()
    }

    pub fn incoming_count(&self, rule: &RecursiveWorldRule) -> usize {
        self.edges
            .iter()
            .filter(|edge| edge.target() == rule)
            .count()
    }

    pub fn direct_dependents(&self, rule: &RecursiveWorldRule) -> Vec<RecursiveWorldRule> {
        self.edges
            .iter()
            .filter(|edge| edge.source() == rule)
            .map(|edge| edge.target().clone())
            .collect()
    }
}

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldDependencyCone {
    root: RecursiveWorldRule,
    affected: Vec<RecursiveWorldRule>,
}

impl RecursiveWorldDependencyCone {
    pub fn compute(graph: &RecursiveWorldDependencyGraph, root: RecursiveWorldRule) -> Self {
        let mut visited = BTreeSet::new();

        let mut frontier = graph.direct_dependents(&root);

        frontier.sort();
        frontier.dedup();

        while let Some(rule) = frontier.pop() {
            if rule == root {
                continue;
            }

            if !visited.insert(rule.clone()) {
                continue;
            }

            for dependent in graph.direct_dependents(&rule) {
                if dependent != root && !visited.contains(&dependent) {
                    frontier.push(dependent);
                }
            }

            frontier.sort();
            frontier.dedup();
        }

        Self {
            root,
            affected: visited.into_iter().collect(),
        }
    }

    pub fn root(&self) -> &RecursiveWorldRule {
        &self.root
    }

    pub fn affected(&self) -> &[RecursiveWorldRule] {
        &self.affected
    }

    pub fn len(&self) -> usize {
        self.affected.len()
    }

    pub fn is_empty(&self) -> bool {
        self.affected.is_empty()
    }

    pub fn contains(&self, rule: &RecursiveWorldRule) -> bool {
        self.affected.binary_search(rule).is_ok()
    }

    pub fn includes_root(&self) -> bool {
        self.contains(&self.root)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldMinimalRevision {
    before: RecursiveWorldModel,
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
    affected_before: RecursiveWorldDependencyCone,
    after: RecursiveWorldModel,
}

impl RecursiveWorldMinimalRevision {
    pub fn apply(
        model: &RecursiveWorldModel,
        target: RecursiveWorldRule,
        replacement: RecursiveWorldRule,
    ) -> Option<Self> {
        if target == replacement {
            return None;
        }

        if !model.contains(&target) {
            return None;
        }

        if model.contains(&replacement) {
            return None;
        }

        let graph = RecursiveWorldDependencyGraph::detect(model);

        let affected_before = RecursiveWorldDependencyCone::compute(&graph, target.clone());

        let mut revised_rules = Vec::with_capacity(model.len());

        for rule in model.rules() {
            if rule == &target {
                revised_rules.push(replacement.clone());
            } else {
                revised_rules.push(rule.clone());
            }
        }

        let after = RecursiveWorldModel::new(revised_rules);

        if after.len() != model.len() {
            return None;
        }

        Some(Self {
            before: model.clone(),
            target,
            replacement,
            affected_before,
            after,
        })
    }

    pub fn before(&self) -> &RecursiveWorldModel {
        &self.before
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn replacement(&self) -> &RecursiveWorldRule {
        &self.replacement
    }

    pub fn affected_before(&self) -> &RecursiveWorldDependencyCone {
        &self.affected_before
    }

    pub fn after(&self) -> &RecursiveWorldModel {
        &self.after
    }

    pub fn changed_rule_count(&self) -> usize {
        1
    }

    pub fn unaffected_rule_count(&self) -> usize {
        self.before.len().saturating_sub(1)
    }

    pub fn preserves_rule_count(&self) -> bool {
        self.before.len() == self.after.len()
    }
}

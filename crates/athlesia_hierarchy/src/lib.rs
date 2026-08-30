use std::collections::BTreeSet;

use athlesia::StructuralConcept;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HierarchicalConcept {
    children: Vec<StructuralConcept>,
}

impl HierarchicalConcept {
    pub fn new(children: Vec<StructuralConcept>) -> Option<Self> {
        let unique: BTreeSet<StructuralConcept> = children.into_iter().collect();

        if unique.len() < 2 {
            return None;
        }

        Some(Self {
            children: unique.into_iter().collect(),
        })
    }

    pub fn children(&self) -> &[StructuralConcept] {
        &self.children
    }

    pub fn arity(&self) -> usize {
        self.children.len()
    }

    pub fn contains(&self, concept: &StructuralConcept) -> bool {
        self.children.binary_search(concept).is_ok()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HierarchicalMemory {
    concepts: BTreeSet<HierarchicalConcept>,
}

impl HierarchicalMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    pub fn insert(&mut self, concept: HierarchicalConcept) -> bool {
        self.concepts.insert(concept)
    }

    pub fn contains(&self, concept: &HierarchicalConcept) -> bool {
        self.concepts.contains(concept)
    }

    pub fn concepts(&self) -> impl Iterator<Item = &HierarchicalConcept> {
        self.concepts.iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyObservation {
    concepts: Vec<StructuralConcept>,
}

impl HierarchyObservation {
    pub fn new(concepts: Vec<StructuralConcept>) -> Self {
        let unique: BTreeSet<StructuralConcept> = concepts.into_iter().collect();

        Self {
            concepts: unique.into_iter().collect(),
        }
    }

    pub fn concepts(&self) -> &[StructuralConcept] {
        &self.concepts
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyCandidate {
    concept: HierarchicalConcept,
    support: usize,
}

impl HierarchyCandidate {
    pub fn concept(&self) -> &HierarchicalConcept {
        &self.concept
    }

    pub const fn support(&self) -> usize {
        self.support
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HierarchyDiscovery {
    minimum_support: usize,
}

impl HierarchyDiscovery {
    pub const fn new(minimum_support: usize) -> Self {
        Self { minimum_support }
    }

    pub const fn minimum_support(self) -> usize {
        self.minimum_support
    }

    pub fn discover(&self, observations: &[HierarchyObservation]) -> Vec<HierarchyCandidate> {
        use std::collections::BTreeMap;

        let mut support: BTreeMap<HierarchicalConcept, usize> = BTreeMap::new();

        for observation in observations {
            let concepts = observation.concepts();

            for left in 0..concepts.len() {
                for right in (left + 1)..concepts.len() {
                    let candidate = HierarchicalConcept::new(vec![
                        concepts[left].clone(),
                        concepts[right].clone(),
                    ]);

                    if let Some(candidate) = candidate {
                        *support.entry(candidate).or_insert(0) += 1;
                    }
                }
            }
        }

        support
            .into_iter()
            .filter_map(|(concept, count)| {
                if count >= self.minimum_support {
                    Some(HierarchyCandidate {
                        concept,
                        support: count,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn consolidate(
        &self,
        observations: &[HierarchyObservation],
        memory: &mut HierarchicalMemory,
    ) -> Vec<HierarchyCandidate> {
        let candidates = self.discover(observations);

        for candidate in &candidates {
            memory.insert(candidate.concept().clone());
        }

        candidates
    }
}

impl Default for HierarchyDiscovery {
    fn default() -> Self {
        Self::new(2)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyMatch {
    concept: HierarchicalConcept,
    matched_children: usize,
    observation_size: usize,
}

impl HierarchyMatch {
    pub fn concept(&self) -> &HierarchicalConcept {
        &self.concept
    }

    pub const fn matched_children(&self) -> usize {
        self.matched_children
    }

    pub const fn observation_size(&self) -> usize {
        self.observation_size
    }

    pub fn is_exact_context(&self) -> bool {
        self.matched_children == self.observation_size
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HierarchyRecognizer;

impl HierarchyRecognizer {
    pub const fn new() -> Self {
        Self
    }

    pub fn recognizes(
        &self,
        concept: &HierarchicalConcept,
        observation: &HierarchyObservation,
    ) -> bool {
        concept
            .children()
            .iter()
            .all(|child| observation.concepts().binary_search(child).is_ok())
    }

    pub fn recognize(
        &self,
        concept: &HierarchicalConcept,
        observation: &HierarchyObservation,
    ) -> Option<HierarchyMatch> {
        if !self.recognizes(concept, observation) {
            return None;
        }

        Some(HierarchyMatch {
            concept: concept.clone(),
            matched_children: concept.arity(),
            observation_size: observation.len(),
        })
    }

    pub fn recognize_memory(
        &self,
        memory: &HierarchicalMemory,
        observation: &HierarchyObservation,
    ) -> Vec<HierarchyMatch> {
        memory
            .concepts()
            .filter_map(|concept| self.recognize(concept, observation))
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyPrediction {
    hierarchy: HierarchicalConcept,
    observed_children: usize,
    missing_children: Vec<StructuralConcept>,
}

impl HierarchyPrediction {
    pub fn hierarchy(&self) -> &HierarchicalConcept {
        &self.hierarchy
    }

    pub const fn observed_children(&self) -> usize {
        self.observed_children
    }

    pub fn missing_children(&self) -> &[StructuralConcept] {
        &self.missing_children
    }

    pub fn missing_count(&self) -> usize {
        self.missing_children.len()
    }

    pub fn is_single_step_completion(&self) -> bool {
        self.missing_children.len() == 1
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HierarchyPredictor;

impl HierarchyPredictor {
    pub const fn new() -> Self {
        Self
    }

    pub fn predict(
        &self,
        hierarchy: &HierarchicalConcept,
        observation: &HierarchyObservation,
    ) -> Option<HierarchyPrediction> {
        let observed_children = hierarchy
            .children()
            .iter()
            .filter(|child| observation.concepts().binary_search(child).is_ok())
            .count();

        if observed_children == 0 || observed_children == hierarchy.arity() {
            return None;
        }

        let missing_children = hierarchy
            .children()
            .iter()
            .filter(|child| observation.concepts().binary_search(child).is_err())
            .cloned()
            .collect();

        Some(HierarchyPrediction {
            hierarchy: hierarchy.clone(),
            observed_children,
            missing_children,
        })
    }

    pub fn predict_memory(
        &self,
        memory: &HierarchicalMemory,
        observation: &HierarchyObservation,
    ) -> Vec<HierarchyPrediction> {
        let mut predictions: Vec<HierarchyPrediction> = memory
            .concepts()
            .filter_map(|hierarchy| self.predict(hierarchy, observation))
            .collect();

        predictions.sort_by(|left, right| {
            left.missing_count()
                .cmp(&right.missing_count())
                .then_with(|| right.observed_children().cmp(&left.observed_children()))
                .then_with(|| left.hierarchy().cmp(right.hierarchy()))
        });

        predictions
    }

    pub fn best_prediction(
        &self,
        memory: &HierarchicalMemory,
        observation: &HierarchyObservation,
    ) -> Option<HierarchyPrediction> {
        self.predict_memory(memory, observation).into_iter().next()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyCompletionCandidate {
    child: StructuralConcept,
    supporting_hierarchies: usize,
    single_step_support: usize,
}

impl HierarchyCompletionCandidate {
    pub fn child(&self) -> &StructuralConcept {
        &self.child
    }

    pub const fn supporting_hierarchies(&self) -> usize {
        self.supporting_hierarchies
    }

    pub const fn single_step_support(&self) -> usize {
        self.single_step_support
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HierarchyCompletionSelector;

impl HierarchyCompletionSelector {
    pub const fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        memory: &HierarchicalMemory,
        observation: &HierarchyObservation,
    ) -> Vec<HierarchyCompletionCandidate> {
        use std::collections::BTreeMap;

        let predictions = HierarchyPredictor::new().predict_memory(memory, observation);

        let mut support: BTreeMap<StructuralConcept, (usize, usize)> = BTreeMap::new();

        for prediction in predictions {
            let single_step = prediction.is_single_step_completion();

            for child in prediction.missing_children() {
                let entry = support.entry(child.clone()).or_insert((0, 0));

                entry.0 += 1;

                if single_step {
                    entry.1 += 1;
                }
            }
        }

        let mut candidates: Vec<HierarchyCompletionCandidate> = support
            .into_iter()
            .map(|(child, (supporting_hierarchies, single_step_support))| {
                HierarchyCompletionCandidate {
                    child,
                    supporting_hierarchies,
                    single_step_support,
                }
            })
            .collect();

        candidates.sort_by(|left, right| {
            right
                .supporting_hierarchies()
                .cmp(&left.supporting_hierarchies())
                .then_with(|| right.single_step_support().cmp(&left.single_step_support()))
                .then_with(|| left.child().cmp(right.child()))
        });

        candidates
    }

    pub fn select(
        &self,
        memory: &HierarchicalMemory,
        observation: &HierarchyObservation,
    ) -> Option<HierarchyCompletionCandidate> {
        self.generate(memory, observation).into_iter().next()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HierarchyCompletionOutcome {
    Confirmed,
    Violated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyCompletionEvaluation {
    target: StructuralConcept,
    outcome: HierarchyCompletionOutcome,
    supporting_hierarchies: usize,
    single_step_support: usize,
}

impl HierarchyCompletionEvaluation {
    pub fn target(&self) -> &StructuralConcept {
        &self.target
    }

    pub const fn outcome(&self) -> HierarchyCompletionOutcome {
        self.outcome
    }

    pub const fn supporting_hierarchies(&self) -> usize {
        self.supporting_hierarchies
    }

    pub const fn single_step_support(&self) -> usize {
        self.single_step_support
    }

    pub const fn is_confirmed(&self) -> bool {
        matches!(self.outcome, HierarchyCompletionOutcome::Confirmed)
    }

    pub const fn is_violated(&self) -> bool {
        matches!(self.outcome, HierarchyCompletionOutcome::Violated)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HierarchyCompletionEvaluator;

impl HierarchyCompletionEvaluator {
    pub const fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        candidate: &HierarchyCompletionCandidate,
        observation: &HierarchyObservation,
    ) -> HierarchyCompletionEvaluation {
        let outcome = if observation
            .concepts()
            .binary_search(candidate.child())
            .is_ok()
        {
            HierarchyCompletionOutcome::Confirmed
        } else {
            HierarchyCompletionOutcome::Violated
        };

        HierarchyCompletionEvaluation {
            target: candidate.child().clone(),
            outcome,
            supporting_hierarchies: candidate.supporting_hierarchies(),
            single_step_support: candidate.single_step_support(),
        }
    }

    pub fn select_and_evaluate(
        &self,
        memory: &HierarchicalMemory,
        prior_observation: &HierarchyObservation,
        next_observation: &HierarchyObservation,
    ) -> Option<(HierarchyCompletionCandidate, HierarchyCompletionEvaluation)> {
        let candidate = HierarchyCompletionSelector::new().select(memory, prior_observation)?;

        let evaluation = self.evaluate(&candidate, next_observation);

        Some((candidate, evaluation))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyActiveTransition {
    candidate: HierarchyCompletionCandidate,
    evaluation: HierarchyCompletionEvaluation,
    prior_observation: HierarchyObservation,
    next_observation: HierarchyObservation,
}

impl HierarchyActiveTransition {
    pub fn candidate(&self) -> &HierarchyCompletionCandidate {
        &self.candidate
    }

    pub fn evaluation(&self) -> &HierarchyCompletionEvaluation {
        &self.evaluation
    }

    pub fn prior_observation(&self) -> &HierarchyObservation {
        &self.prior_observation
    }

    pub fn next_observation(&self) -> &HierarchyObservation {
        &self.next_observation
    }

    pub const fn outcome(&self) -> HierarchyCompletionOutcome {
        self.evaluation.outcome()
    }

    pub const fn is_confirmed(&self) -> bool {
        self.evaluation.is_confirmed()
    }

    pub const fn is_violated(&self) -> bool {
        self.evaluation.is_violated()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HierarchyActiveCycle;

impl HierarchyActiveCycle {
    pub const fn new() -> Self {
        Self
    }

    pub fn step(
        &self,
        memory: &HierarchicalMemory,
        prior_observation: &HierarchyObservation,
        next_observation: &HierarchyObservation,
    ) -> Option<HierarchyActiveTransition> {
        let candidate = HierarchyCompletionSelector::new().select(memory, prior_observation)?;

        let evaluation = HierarchyCompletionEvaluator::new().evaluate(&candidate, next_observation);

        Some(HierarchyActiveTransition {
            candidate,
            evaluation,
            prior_observation: prior_observation.clone(),
            next_observation: next_observation.clone(),
        })
    }
}

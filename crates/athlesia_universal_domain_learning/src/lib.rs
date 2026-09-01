use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedLearningEpisode {
    facts: Vec<CognitiveStructure>,
    outcome: CognitiveStructure,
}

impl GroundedLearningEpisode {
    pub fn new(mut facts: Vec<CognitiveStructure>, outcome: CognitiveStructure) -> Option<Self> {
        if facts.is_empty() {
            return None;
        }

        facts.sort_by(PredicateDiscovery::compare_structure);

        facts.dedup();

        Some(Self { facts, outcome })
    }

    pub fn facts(&self) -> &[CognitiveStructure] {
        &self.facts
    }

    pub fn outcome(&self) -> &CognitiveStructure {
        &self.outcome
    }

    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    pub fn contains_fact(&self, target: &CognitiveStructure) -> bool {
        self.facts.iter().any(|fact| fact == target)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PredicateDiscoveryPolicy {
    minimum_support: u64,
    minimum_precision: CognitiveSignal,
    minimum_association_lift: CognitiveSignal,
    max_predicates: usize,
}

impl PredicateDiscoveryPolicy {
    pub fn new(
        minimum_support: u64,
        minimum_precision: CognitiveSignal,
        minimum_association_lift: CognitiveSignal,
        max_predicates: usize,
    ) -> Option<Self> {
        if minimum_support == 0
            || minimum_association_lift == CognitiveSignal::zero()
            || max_predicates == 0
        {
            return None;
        }

        Some(Self {
            minimum_support,
            minimum_precision,
            minimum_association_lift,
            max_predicates,
        })
    }

    pub fn minimum_support(self) -> u64 {
        self.minimum_support
    }

    pub fn minimum_precision(self) -> CognitiveSignal {
        self.minimum_precision
    }

    pub fn minimum_association_lift(self) -> CognitiveSignal {
        self.minimum_association_lift
    }

    pub fn max_predicates(self) -> usize {
        self.max_predicates
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedPredicateHypothesis {
    antecedent: CognitiveStructure,
    consequent: CognitiveStructure,
    support_count: u64,
    antecedent_count: u64,
    consequent_count: u64,
    episode_count: u64,
    precision: CognitiveSignal,
    baseline_rate: CognitiveSignal,
    association_lift: CognitiveSignal,
}

impl GroundedPredicateHypothesis {
    pub fn antecedent(&self) -> &CognitiveStructure {
        &self.antecedent
    }

    pub fn consequent(&self) -> &CognitiveStructure {
        &self.consequent
    }

    pub fn support_count(&self) -> u64 {
        self.support_count
    }

    pub fn antecedent_count(&self) -> u64 {
        self.antecedent_count
    }

    pub fn consequent_count(&self) -> u64 {
        self.consequent_count
    }

    pub fn episode_count(&self) -> u64 {
        self.episode_count
    }

    pub fn precision(&self) -> CognitiveSignal {
        self.precision
    }

    pub fn baseline_rate(&self) -> CognitiveSignal {
        self.baseline_rate
    }

    pub fn association_lift(&self) -> CognitiveSignal {
        self.association_lift
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateDiscoveryResult {
    episode_count: usize,
    discovered_before_policy: usize,
    selected: Vec<GroundedPredicateHypothesis>,
    truncated_by_frontier: bool,
}

impl PredicateDiscoveryResult {
    pub fn episode_count(&self) -> usize {
        self.episode_count
    }

    pub fn discovered_before_policy(&self) -> usize {
        self.discovered_before_policy
    }

    pub fn selected(&self) -> &[GroundedPredicateHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn truncated_by_frontier(&self) -> bool {
        self.truncated_by_frontier
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct PredicateDiscovery;

impl PredicateDiscovery {
    pub(crate) fn compare_structure(
        left: &CognitiveStructure,
        right: &CognitiveStructure,
    ) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (left, right) {
            (CognitiveStructure::Atom(left_value), CognitiveStructure::Atom(right_value)) => {
                left_value.cmp(right_value)
            }

            (CognitiveStructure::Atom(_), _) => Ordering::Less,

            (_, CognitiveStructure::Atom(_)) => Ordering::Greater,

            (
                CognitiveStructure::Ordered(left_values),
                CognitiveStructure::Ordered(right_values),
            )
            | (
                CognitiveStructure::Unordered(left_values),
                CognitiveStructure::Unordered(right_values),
            ) => {
                let mut left_iterator = left_values.iter();

                let mut right_iterator = right_values.iter();

                loop {
                    match (left_iterator.next(), right_iterator.next()) {
                        (Some(left_item), Some(right_item)) => {
                            let ordering = Self::compare_structure(left_item, right_item);

                            if ordering != Ordering::Equal {
                                return ordering;
                            }
                        }

                        (None, Some(_)) => {
                            return Ordering::Less;
                        }

                        (Some(_), None) => {
                            return Ordering::Greater;
                        }

                        (None, None) => {
                            return Ordering::Equal;
                        }
                    }
                }
            }

            (CognitiveStructure::Ordered(_), CognitiveStructure::Unordered(_)) => Ordering::Less,

            (CognitiveStructure::Unordered(_), CognitiveStructure::Ordered(_)) => Ordering::Greater,
        }
    }

    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16).expect("bounded empirical rate remains on signal scale")
    }

    fn association_lift(
        precision: CognitiveSignal,
        baseline_rate: CognitiveSignal,
    ) -> CognitiveSignal {
        CognitiveSignal::new(precision.value().saturating_sub(baseline_rate.value()))
            .expect("bounded positive association lift remains on signal scale")
    }

    fn ranking(
        left: &GroundedPredicateHypothesis,
        right: &GroundedPredicateHypothesis,
    ) -> std::cmp::Ordering {
        right
            .association_lift()
            .value()
            .cmp(&left.association_lift().value())
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| Self::compare_structure(left.antecedent(), right.antecedent()))
            .then_with(|| Self::compare_structure(left.consequent(), right.consequent()))
    }

    fn unique_facts(episodes: &[GroundedLearningEpisode]) -> Vec<CognitiveStructure> {
        let mut facts = episodes
            .iter()
            .flat_map(|episode| episode.facts().iter().cloned())
            .collect::<Vec<_>>();

        facts.sort_by(Self::compare_structure);

        facts.dedup();

        facts
    }

    fn unique_outcomes(episodes: &[GroundedLearningEpisode]) -> Vec<CognitiveStructure> {
        let mut outcomes = episodes
            .iter()
            .map(|episode| episode.outcome().clone())
            .collect::<Vec<_>>();

        outcomes.sort_by(Self::compare_structure);

        outcomes.dedup();

        outcomes
    }

    pub fn discover(
        episodes: &[GroundedLearningEpisode],
        policy: PredicateDiscoveryPolicy,
    ) -> PredicateDiscoveryResult {
        if episodes.is_empty() {
            return PredicateDiscoveryResult {
                episode_count: 0,
                discovered_before_policy: 0,
                selected: Vec::new(),
                truncated_by_frontier: false,
            };
        }

        let facts = Self::unique_facts(episodes);

        let outcomes = Self::unique_outcomes(episodes);

        let episode_count = episodes.len() as u64;

        let mut admitted = Vec::new();

        for antecedent in &facts {
            let antecedent_count = episodes
                .iter()
                .filter(|episode| episode.contains_fact(antecedent))
                .count() as u64;

            for consequent in &outcomes {
                let consequent_count = episodes
                    .iter()
                    .filter(|episode| episode.outcome() == consequent)
                    .count() as u64;

                let support_count = episodes
                    .iter()
                    .filter(|episode| {
                        episode.contains_fact(antecedent) && episode.outcome() == consequent
                    })
                    .count() as u64;

                if support_count == 0 {
                    continue;
                }

                let precision = Self::scaled_rate(support_count, antecedent_count);

                let baseline_rate = Self::scaled_rate(consequent_count, episode_count);

                let association_lift = Self::association_lift(precision, baseline_rate);

                if support_count < policy.minimum_support()
                    || precision.value() < policy.minimum_precision().value()
                    || association_lift.value() < policy.minimum_association_lift().value()
                {
                    continue;
                }

                admitted.push(GroundedPredicateHypothesis {
                    antecedent: antecedent.clone(),
                    consequent: consequent.clone(),
                    support_count,
                    antecedent_count,
                    consequent_count,
                    episode_count,
                    precision,
                    baseline_rate,
                    association_lift,
                });
            }
        }

        admitted.sort_by(Self::ranking);

        let discovered_before_policy = admitted.len();

        let truncated_by_frontier = discovered_before_policy > policy.max_predicates();

        admitted.truncate(policy.max_predicates());

        PredicateDiscoveryResult {
            episode_count: episodes.len(),
            discovered_before_policy,
            selected: admitted,
            truncated_by_frontier,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalPredicateDiscovery;

impl UniversalPredicateDiscovery {
    pub fn evaluate(
        episodes: &[GroundedLearningEpisode],
        policy: PredicateDiscoveryPolicy,
    ) -> PredicateDiscoveryResult {
        PredicateDiscovery::discover(episodes, policy)
    }
}

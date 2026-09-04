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

pub const MAX_RULE_PREMISES: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RulePremiseSet {
    premises: Vec<CognitiveStructure>,
}

impl RulePremiseSet {
    pub fn new(mut premises: Vec<CognitiveStructure>) -> Option<Self> {
        premises.sort_by(PredicateDiscovery::compare_structure);

        premises.dedup();

        if premises.len() < 2 {
            return None;
        }

        Some(Self { premises })
    }

    pub fn premises(&self) -> &[CognitiveStructure] {
        &self.premises
    }

    pub fn premise_count(&self) -> usize {
        self.premises.len()
    }

    pub fn is_satisfied_by(&self, episode: &GroundedLearningEpisode) -> bool {
        self.premises
            .iter()
            .all(|premise| episode.contains_fact(premise))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleEvidenceThresholds {
    minimum_support: u64,
    minimum_precision: CognitiveSignal,
    minimum_association_lift: CognitiveSignal,
    minimum_incremental_gain: CognitiveSignal,
}

impl RuleEvidenceThresholds {
    pub fn new(
        minimum_support: u64,
        minimum_precision: CognitiveSignal,
        minimum_association_lift: CognitiveSignal,
        minimum_incremental_gain: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_support == 0
            || minimum_association_lift == CognitiveSignal::zero()
            || minimum_incremental_gain == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_support,
            minimum_precision,
            minimum_association_lift,
            minimum_incremental_gain,
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

    pub fn minimum_incremental_gain(self) -> CognitiveSignal {
        self.minimum_incremental_gain
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleInductionPolicy {
    max_premises: usize,
    max_candidate_premise_sets: usize,
    max_rule_evaluations: usize,
    max_rules: usize,
    thresholds: RuleEvidenceThresholds,
}

impl RuleInductionPolicy {
    pub fn new(
        max_premises: usize,
        max_candidate_premise_sets: usize,
        max_rule_evaluations: usize,
        max_rules: usize,
        thresholds: RuleEvidenceThresholds,
    ) -> Option<Self> {
        if !(2..=MAX_RULE_PREMISES).contains(&max_premises)
            || max_candidate_premise_sets == 0
            || max_rule_evaluations == 0
            || max_rules == 0
        {
            return None;
        }

        Some(Self {
            max_premises,
            max_candidate_premise_sets,
            max_rule_evaluations,
            max_rules,
            thresholds,
        })
    }

    pub fn max_premises(self) -> usize {
        self.max_premises
    }

    pub fn max_candidate_premise_sets(self) -> usize {
        self.max_candidate_premise_sets
    }

    pub fn max_rule_evaluations(self) -> usize {
        self.max_rule_evaluations
    }

    pub fn max_rules(self) -> usize {
        self.max_rules
    }

    pub fn thresholds(self) -> RuleEvidenceThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedRuleHypothesis {
    premises: RulePremiseSet,
    consequent: CognitiveStructure,
    support_count: u64,
    premise_opportunity_count: u64,
    consequent_count: u64,
    episode_count: u64,
    counterexample_count: u64,
    precision: CognitiveSignal,
    baseline_rate: CognitiveSignal,
    association_lift: CognitiveSignal,
    best_proper_subset_precision: CognitiveSignal,
    incremental_precision_gain: CognitiveSignal,
}

impl GroundedRuleHypothesis {
    pub fn premises(&self) -> &RulePremiseSet {
        &self.premises
    }

    pub fn consequent(&self) -> &CognitiveStructure {
        &self.consequent
    }

    pub fn support_count(&self) -> u64 {
        self.support_count
    }

    pub fn premise_opportunity_count(&self) -> u64 {
        self.premise_opportunity_count
    }

    pub fn consequent_count(&self) -> u64 {
        self.consequent_count
    }

    pub fn episode_count(&self) -> u64 {
        self.episode_count
    }

    pub fn counterexample_count(&self) -> u64 {
        self.counterexample_count
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

    pub fn best_proper_subset_precision(&self) -> CognitiveSignal {
        self.best_proper_subset_precision
    }

    pub fn incremental_precision_gain(&self) -> CognitiveSignal {
        self.incremental_precision_gain
    }

    pub fn is_satisfied_by(&self, episode: &GroundedLearningEpisode) -> bool {
        self.premises.is_satisfied_by(episode)
    }

    pub fn is_counterexample(&self, episode: &GroundedLearningEpisode) -> bool {
        self.is_satisfied_by(episode) && episode.outcome() != self.consequent()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RuleVocabularyFact {
    structure: CognitiveStructure,
    seed_lift: u16,
    seed_precision: u16,
    seed_support: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleInductionResult {
    vocabulary_fact_count: usize,
    seeded_vocabulary_fact_count: usize,
    possible_premise_set_count: usize,
    generated_premise_set_count: usize,
    candidate_generation_truncated: bool,
    evaluated_rule_candidate_count: usize,
    rule_evaluation_truncated: bool,
    admitted_before_frontier: usize,
    selected: Vec<GroundedRuleHypothesis>,
}

impl RuleInductionResult {
    pub fn vocabulary_fact_count(&self) -> usize {
        self.vocabulary_fact_count
    }

    pub fn seeded_vocabulary_fact_count(&self) -> usize {
        self.seeded_vocabulary_fact_count
    }

    pub fn possible_premise_set_count(&self) -> usize {
        self.possible_premise_set_count
    }

    pub fn generated_premise_set_count(&self) -> usize {
        self.generated_premise_set_count
    }

    pub fn candidate_generation_truncated(&self) -> bool {
        self.candidate_generation_truncated
    }

    pub fn evaluated_rule_candidate_count(&self) -> usize {
        self.evaluated_rule_candidate_count
    }

    pub fn rule_evaluation_truncated(&self) -> bool {
        self.rule_evaluation_truncated
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedRuleHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct RuleInduction;

impl RuleInduction {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16).expect("bounded empirical rate remains on signal scale")
    }

    fn positive_difference(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        CognitiveSignal::new(left.value().saturating_sub(right.value()))
            .expect("bounded positive signal difference remains on scale")
    }

    fn unique_outcomes(episodes: &[GroundedLearningEpisode]) -> Vec<CognitiveStructure> {
        let mut outcomes = episodes
            .iter()
            .map(|episode| episode.outcome().clone())
            .collect::<Vec<_>>();

        outcomes.sort_by(PredicateDiscovery::compare_structure);

        outcomes.dedup();

        outcomes
    }

    fn seed_strength(
        fact: &CognitiveStructure,
        seeds: &[GroundedPredicateHypothesis],
    ) -> (u16, u16, u64) {
        seeds
            .iter()
            .filter(|seed| seed.antecedent() == fact)
            .map(|seed| {
                (
                    seed.association_lift().value(),
                    seed.precision().value(),
                    seed.support_count(),
                )
            })
            .max()
            .unwrap_or((0, 0, 0))
    }

    fn vocabulary(
        episodes: &[GroundedLearningEpisode],
        seeds: &[GroundedPredicateHypothesis],
    ) -> Vec<RuleVocabularyFact> {
        let mut facts = episodes
            .iter()
            .flat_map(|episode| episode.facts().iter().cloned())
            .collect::<Vec<_>>();

        facts.sort_by(PredicateDiscovery::compare_structure);

        facts.dedup();

        let mut vocabulary = facts
            .into_iter()
            .map(|structure| {
                let (seed_lift, seed_precision, seed_support) =
                    Self::seed_strength(&structure, seeds);

                RuleVocabularyFact {
                    structure,
                    seed_lift,
                    seed_precision,
                    seed_support,
                }
            })
            .collect::<Vec<_>>();

        vocabulary.sort_by(|left, right| {
            right
                .seed_lift
                .cmp(&left.seed_lift)
                .then_with(|| right.seed_precision.cmp(&left.seed_precision))
                .then_with(|| right.seed_support.cmp(&left.seed_support))
                .then_with(|| {
                    PredicateDiscovery::compare_structure(&left.structure, &right.structure)
                })
        });

        vocabulary
    }

    fn binomial_saturating(n: usize, k: usize) -> usize {
        if k > n {
            return 0;
        }

        let effective_k = k.min(n - k);

        let mut value = 1_u128;

        for index in 0..effective_k {
            value = value.saturating_mul((n - index) as u128) / (index + 1) as u128;

            if value > usize::MAX as u128 {
                return usize::MAX;
            }
        }

        value as usize
    }

    fn possible_premise_set_count(vocabulary_len: usize, max_premises: usize) -> usize {
        let upper = max_premises.min(vocabulary_len);

        (2..=upper).fold(0_usize, |total, size| {
            total.saturating_add(Self::binomial_saturating(vocabulary_len, size))
        })
    }

    fn generate_combinations(
        vocabulary: &[RuleVocabularyFact],
        target_size: usize,
        start: usize,
        current: &mut Vec<CognitiveStructure>,
        output: &mut Vec<RulePremiseSet>,
        limit: usize,
    ) {
        if output.len() >= limit {
            return;
        }

        if current.len() == target_size {
            output.push(
                RulePremiseSet::new(current.clone())
                    .expect("generated rule premise set has at least two unique facts"),
            );

            return;
        }

        let remaining_needed = target_size.saturating_sub(current.len());

        if vocabulary.len() < start.saturating_add(remaining_needed) {
            return;
        }

        let last_start = vocabulary.len() - remaining_needed;

        for index in start..=last_start {
            if output.len() >= limit {
                return;
            }

            current.push(vocabulary[index].structure.clone());

            Self::generate_combinations(vocabulary, target_size, index + 1, current, output, limit);

            current.pop();
        }
    }

    fn premise_sets(
        vocabulary: &[RuleVocabularyFact],
        policy: RuleInductionPolicy,
    ) -> (Vec<RulePremiseSet>, usize, bool) {
        let possible = Self::possible_premise_set_count(vocabulary.len(), policy.max_premises());

        let mut generated = Vec::new();

        let upper = policy.max_premises().min(vocabulary.len());

        for target_size in 2..=upper {
            if generated.len() >= policy.max_candidate_premise_sets() {
                break;
            }

            Self::generate_combinations(
                vocabulary,
                target_size,
                0,
                &mut Vec::new(),
                &mut generated,
                policy.max_candidate_premise_sets(),
            );
        }

        let truncated = possible > generated.len();

        (generated, possible, truncated)
    }

    fn subset_matches(
        episode: &GroundedLearningEpisode,
        premises: &[CognitiveStructure],
        mask: usize,
    ) -> bool {
        premises
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1_usize << index) != 0)
            .all(|(_, premise)| episode.contains_fact(premise))
    }

    fn best_proper_subset_precision(
        episodes: &[GroundedLearningEpisode],
        premises: &RulePremiseSet,
        consequent: &CognitiveStructure,
    ) -> CognitiveSignal {
        let premise_count = premises.premise_count();

        let full_mask = (1_usize << premise_count) - 1;

        let mut best = CognitiveSignal::zero();

        for mask in 1..full_mask {
            let mut opportunity = 0_u64;

            let mut support = 0_u64;

            for episode in episodes {
                if Self::subset_matches(episode, premises.premises(), mask) {
                    opportunity = opportunity.saturating_add(1);

                    if episode.outcome() == consequent {
                        support = support.saturating_add(1);
                    }
                }
            }

            if opportunity == 0 {
                continue;
            }

            let precision = Self::scaled_rate(support, opportunity);

            if precision > best {
                best = precision;
            }
        }

        best
    }

    fn evaluate_rule(
        episodes: &[GroundedLearningEpisode],
        premises: &RulePremiseSet,
        consequent: &CognitiveStructure,
        thresholds: RuleEvidenceThresholds,
    ) -> Option<GroundedRuleHypothesis> {
        let episode_count = episodes.len() as u64;

        let consequent_count = episodes
            .iter()
            .filter(|episode| episode.outcome() == consequent)
            .count() as u64;

        let premise_opportunity_count = episodes
            .iter()
            .filter(|episode| premises.is_satisfied_by(episode))
            .count() as u64;

        if premise_opportunity_count == 0 {
            return None;
        }

        let support_count = episodes
            .iter()
            .filter(|episode| premises.is_satisfied_by(episode) && episode.outcome() == consequent)
            .count() as u64;

        if support_count == 0 {
            return None;
        }

        let counterexample_count = premise_opportunity_count.saturating_sub(support_count);

        let precision = Self::scaled_rate(support_count, premise_opportunity_count);

        let baseline_rate = Self::scaled_rate(consequent_count, episode_count);

        let association_lift = Self::positive_difference(precision, baseline_rate);

        let best_proper_subset_precision =
            Self::best_proper_subset_precision(episodes, premises, consequent);

        let incremental_precision_gain =
            Self::positive_difference(precision, best_proper_subset_precision);

        if support_count < thresholds.minimum_support()
            || precision.value() < thresholds.minimum_precision().value()
            || association_lift.value() < thresholds.minimum_association_lift().value()
            || incremental_precision_gain.value() < thresholds.minimum_incremental_gain().value()
        {
            return None;
        }

        Some(GroundedRuleHypothesis {
            premises: premises.clone(),
            consequent: consequent.clone(),
            support_count,
            premise_opportunity_count,
            consequent_count,
            episode_count,
            counterexample_count,
            precision,
            baseline_rate,
            association_lift,
            best_proper_subset_precision,
            incremental_precision_gain,
        })
    }

    fn ranking(
        left: &GroundedRuleHypothesis,
        right: &GroundedRuleHypothesis,
    ) -> std::cmp::Ordering {
        right
            .incremental_precision_gain()
            .value()
            .cmp(&left.incremental_precision_gain().value())
            .then_with(|| {
                right
                    .association_lift()
                    .value()
                    .cmp(&left.association_lift().value())
            })
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| {
                left.premises()
                    .premise_count()
                    .cmp(&right.premises().premise_count())
            })
            .then_with(|| Self::compare_premise_sets(left.premises(), right.premises()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.consequent(), right.consequent())
            })
    }

    fn compare_premise_sets(left: &RulePremiseSet, right: &RulePremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_premise), Some(right_premise)) => {
                    let ordering =
                        PredicateDiscovery::compare_structure(left_premise, right_premise);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    pub fn induce(
        episodes: &[GroundedLearningEpisode],
        predicate_seeds: &[GroundedPredicateHypothesis],
        policy: RuleInductionPolicy,
    ) -> RuleInductionResult {
        if episodes.is_empty() {
            return RuleInductionResult {
                vocabulary_fact_count: 0,
                seeded_vocabulary_fact_count: 0,
                possible_premise_set_count: 0,
                generated_premise_set_count: 0,
                candidate_generation_truncated: false,
                evaluated_rule_candidate_count: 0,
                rule_evaluation_truncated: false,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let vocabulary = Self::vocabulary(episodes, predicate_seeds);

        let seeded_vocabulary_fact_count = vocabulary
            .iter()
            .filter(|fact| fact.seed_lift > 0 || fact.seed_precision > 0 || fact.seed_support > 0)
            .count();

        let (premise_sets, possible_premise_set_count, candidate_generation_truncated) =
            Self::premise_sets(&vocabulary, policy);

        let outcomes = Self::unique_outcomes(episodes);

        let possible_rule_evaluations = premise_sets.len().saturating_mul(outcomes.len());

        let mut evaluated_rule_candidate_count = 0_usize;

        let mut admitted = Vec::new();

        'premises: for premises in &premise_sets {
            for consequent in &outcomes {
                if evaluated_rule_candidate_count >= policy.max_rule_evaluations() {
                    break 'premises;
                }

                evaluated_rule_candidate_count = evaluated_rule_candidate_count.saturating_add(1);

                if let Some(rule) =
                    Self::evaluate_rule(episodes, premises, consequent, policy.thresholds())
                {
                    admitted.push(rule);
                }
            }
        }

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_rules());

        RuleInductionResult {
            vocabulary_fact_count: vocabulary.len(),
            seeded_vocabulary_fact_count,
            possible_premise_set_count,
            generated_premise_set_count: premise_sets.len(),
            candidate_generation_truncated,
            evaluated_rule_candidate_count,
            rule_evaluation_truncated: candidate_generation_truncated
                || possible_rule_evaluations > policy.max_rule_evaluations(),
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalRuleInduction;

impl UniversalRuleInduction {
    pub fn evaluate(
        episodes: &[GroundedLearningEpisode],
        predicate_seeds: &[GroundedPredicateHypothesis],
        policy: RuleInductionPolicy,
    ) -> RuleInductionResult {
        RuleInduction::induce(episodes, predicate_seeds, policy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedStateSnapshot {
    facts: Vec<CognitiveStructure>,
}

impl GroundedStateSnapshot {
    pub fn new(mut facts: Vec<CognitiveStructure>) -> Option<Self> {
        if facts.is_empty() {
            return None;
        }

        facts.sort_by(PredicateDiscovery::compare_structure);

        facts.dedup();

        Some(Self { facts })
    }

    pub fn facts(&self) -> &[CognitiveStructure] {
        &self.facts
    }

    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    pub fn contains_fact(&self, fact: &CognitiveStructure) -> bool {
        self.facts.iter().any(|candidate| candidate == fact)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedTransformationEpisode {
    before: GroundedStateSnapshot,
    after: GroundedStateSnapshot,
    transformation: CognitiveStructure,
}

impl GroundedTransformationEpisode {
    pub fn new(
        before: GroundedStateSnapshot,
        after: GroundedStateSnapshot,
        transformation: CognitiveStructure,
    ) -> Self {
        Self {
            before,
            after,
            transformation,
        }
    }

    pub fn before(&self) -> &GroundedStateSnapshot {
        &self.before
    }

    pub fn after(&self) -> &GroundedStateSnapshot {
        &self.after
    }

    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn preserves(&self, fact: &CognitiveStructure) -> bool {
        self.before.contains_fact(fact) && self.after.contains_fact(fact)
    }

    pub fn disrupts(&self, fact: &CognitiveStructure) -> bool {
        self.before.contains_fact(fact) && !self.after.contains_fact(fact)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InvariantDiscoveryPolicy {
    minimum_stable_support: u64,
    minimum_preservation_rate: CognitiveSignal,
    minimum_distinct_transformations: usize,
    max_candidate_facts: usize,
    max_invariants: usize,
}

impl InvariantDiscoveryPolicy {
    pub fn new(
        minimum_stable_support: u64,
        minimum_preservation_rate: CognitiveSignal,
        minimum_distinct_transformations: usize,
        max_candidate_facts: usize,
        max_invariants: usize,
    ) -> Option<Self> {
        if minimum_stable_support == 0
            || minimum_preservation_rate == CognitiveSignal::zero()
            || minimum_distinct_transformations == 0
            || max_candidate_facts == 0
            || max_invariants == 0
        {
            return None;
        }

        Some(Self {
            minimum_stable_support,
            minimum_preservation_rate,
            minimum_distinct_transformations,
            max_candidate_facts,
            max_invariants,
        })
    }

    pub fn minimum_stable_support(self) -> u64 {
        self.minimum_stable_support
    }

    pub fn minimum_preservation_rate(self) -> CognitiveSignal {
        self.minimum_preservation_rate
    }

    pub fn minimum_distinct_transformations(self) -> usize {
        self.minimum_distinct_transformations
    }

    pub fn max_candidate_facts(self) -> usize {
        self.max_candidate_facts
    }

    pub fn max_invariants(self) -> usize {
        self.max_invariants
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedInvariantHypothesis {
    fact: CognitiveStructure,
    stable_support_count: u64,
    opportunity_count: u64,
    disruption_count: u64,
    preservation_rate: CognitiveSignal,
    distinct_stable_transformations: usize,
    distinct_opportunity_transformations: usize,
    transformation_stability: CognitiveSignal,
}

impl GroundedInvariantHypothesis {
    pub fn fact(&self) -> &CognitiveStructure {
        &self.fact
    }

    pub fn stable_support_count(&self) -> u64 {
        self.stable_support_count
    }

    pub fn opportunity_count(&self) -> u64 {
        self.opportunity_count
    }

    pub fn disruption_count(&self) -> u64 {
        self.disruption_count
    }

    pub fn preservation_rate(&self) -> CognitiveSignal {
        self.preservation_rate
    }

    pub fn distinct_stable_transformations(&self) -> usize {
        self.distinct_stable_transformations
    }

    pub fn distinct_opportunity_transformations(&self) -> usize {
        self.distinct_opportunity_transformations
    }

    pub fn transformation_stability(&self) -> CognitiveSignal {
        self.transformation_stability
    }

    pub fn is_supported_by(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.preserves(&self.fact)
    }

    pub fn is_counterexample(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.disrupts(&self.fact)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InvariantVocabularyFact {
    structure: CognitiveStructure,
    seed_incremental_gain: u16,
    seed_lift: u16,
    seed_support: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvariantDiscoveryResult {
    episode_count: usize,
    vocabulary_fact_count: usize,
    seeded_vocabulary_fact_count: usize,
    evaluated_candidate_count: usize,
    candidate_generation_truncated: bool,
    admitted_before_frontier: usize,
    selected: Vec<GroundedInvariantHypothesis>,
}

impl InvariantDiscoveryResult {
    pub fn episode_count(&self) -> usize {
        self.episode_count
    }

    pub fn vocabulary_fact_count(&self) -> usize {
        self.vocabulary_fact_count
    }

    pub fn seeded_vocabulary_fact_count(&self) -> usize {
        self.seeded_vocabulary_fact_count
    }

    pub fn evaluated_candidate_count(&self) -> usize {
        self.evaluated_candidate_count
    }

    pub fn candidate_generation_truncated(&self) -> bool {
        self.candidate_generation_truncated
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedInvariantHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct InvariantDiscovery;

impl InvariantDiscovery {
    fn scaled_rate(numerator: usize, denominator: usize) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (numerator as u128 * 1000) / denominator as u128;

        CognitiveSignal::new(scaled as u16).expect("bounded invariant rate remains on signal scale")
    }

    fn rule_seed_strength(
        fact: &CognitiveStructure,
        rules: &[GroundedRuleHypothesis],
    ) -> (u16, u16, u64) {
        rules
            .iter()
            .filter(|rule| {
                rule.premises()
                    .premises()
                    .iter()
                    .any(|premise| premise == fact)
            })
            .map(|rule| {
                (
                    rule.incremental_precision_gain().value(),
                    rule.association_lift().value(),
                    rule.support_count(),
                )
            })
            .max()
            .unwrap_or((0, 0, 0))
    }

    fn vocabulary(
        episodes: &[GroundedTransformationEpisode],
        rule_seeds: &[GroundedRuleHypothesis],
    ) -> Vec<InvariantVocabularyFact> {
        let mut facts = episodes
            .iter()
            .flat_map(|episode| episode.before().facts().iter().cloned())
            .collect::<Vec<_>>();

        facts.sort_by(PredicateDiscovery::compare_structure);

        facts.dedup();

        let mut vocabulary = facts
            .into_iter()
            .map(|structure| {
                let (seed_incremental_gain, seed_lift, seed_support) =
                    Self::rule_seed_strength(&structure, rule_seeds);

                InvariantVocabularyFact {
                    structure,
                    seed_incremental_gain,
                    seed_lift,
                    seed_support,
                }
            })
            .collect::<Vec<_>>();

        vocabulary.sort_by(|left, right| {
            right
                .seed_incremental_gain
                .cmp(&left.seed_incremental_gain)
                .then_with(|| right.seed_lift.cmp(&left.seed_lift))
                .then_with(|| right.seed_support.cmp(&left.seed_support))
                .then_with(|| {
                    PredicateDiscovery::compare_structure(&left.structure, &right.structure)
                })
        });

        vocabulary
    }

    fn distinct_transformations(
        episodes: &[GroundedTransformationEpisode],
        fact: &CognitiveStructure,
        require_preserved: bool,
    ) -> Vec<CognitiveStructure> {
        let mut transformations = episodes
            .iter()
            .filter(|episode| {
                if require_preserved {
                    episode.preserves(fact)
                } else {
                    episode.before().contains_fact(fact)
                }
            })
            .map(|episode| episode.transformation().clone())
            .collect::<Vec<_>>();

        transformations.sort_by(PredicateDiscovery::compare_structure);

        transformations.dedup();

        transformations
    }

    fn evaluate_candidate(
        episodes: &[GroundedTransformationEpisode],
        fact: &CognitiveStructure,
        policy: InvariantDiscoveryPolicy,
    ) -> Option<GroundedInvariantHypothesis> {
        let opportunity_count = episodes
            .iter()
            .filter(|episode| episode.before().contains_fact(fact))
            .count();

        if opportunity_count == 0 {
            return None;
        }

        let stable_support_count = episodes
            .iter()
            .filter(|episode| episode.preserves(fact))
            .count();

        if stable_support_count == 0 {
            return None;
        }

        let disruption_count = opportunity_count.saturating_sub(stable_support_count);

        let preservation_rate = Self::scaled_rate(stable_support_count, opportunity_count);

        let stable_transformations = Self::distinct_transformations(episodes, fact, true);

        let opportunity_transformations = Self::distinct_transformations(episodes, fact, false);

        let transformation_stability = Self::scaled_rate(
            stable_transformations.len(),
            opportunity_transformations.len(),
        );

        if (stable_support_count as u64) < policy.minimum_stable_support()
            || preservation_rate.value() < policy.minimum_preservation_rate().value()
            || stable_transformations.len() < policy.minimum_distinct_transformations()
        {
            return None;
        }

        Some(GroundedInvariantHypothesis {
            fact: fact.clone(),
            stable_support_count: stable_support_count as u64,
            opportunity_count: opportunity_count as u64,
            disruption_count: disruption_count as u64,
            preservation_rate,
            distinct_stable_transformations: stable_transformations.len(),
            distinct_opportunity_transformations: opportunity_transformations.len(),
            transformation_stability,
        })
    }

    fn ranking(
        left: &GroundedInvariantHypothesis,
        right: &GroundedInvariantHypothesis,
    ) -> std::cmp::Ordering {
        right
            .preservation_rate()
            .value()
            .cmp(&left.preservation_rate().value())
            .then_with(|| {
                right
                    .transformation_stability()
                    .value()
                    .cmp(&left.transformation_stability().value())
            })
            .then_with(|| {
                right
                    .distinct_stable_transformations()
                    .cmp(&left.distinct_stable_transformations())
            })
            .then_with(|| {
                right
                    .stable_support_count()
                    .cmp(&left.stable_support_count())
            })
            .then_with(|| left.disruption_count().cmp(&right.disruption_count()))
            .then_with(|| PredicateDiscovery::compare_structure(left.fact(), right.fact()))
    }

    pub fn discover(
        episodes: &[GroundedTransformationEpisode],
        rule_seeds: &[GroundedRuleHypothesis],
        policy: InvariantDiscoveryPolicy,
    ) -> InvariantDiscoveryResult {
        if episodes.is_empty() {
            return InvariantDiscoveryResult {
                episode_count: 0,
                vocabulary_fact_count: 0,
                seeded_vocabulary_fact_count: 0,
                evaluated_candidate_count: 0,
                candidate_generation_truncated: false,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let vocabulary = Self::vocabulary(episodes, rule_seeds);

        let seeded_vocabulary_fact_count = vocabulary
            .iter()
            .filter(|fact| {
                fact.seed_incremental_gain > 0 || fact.seed_lift > 0 || fact.seed_support > 0
            })
            .count();

        let candidate_generation_truncated = vocabulary.len() > policy.max_candidate_facts();

        let evaluated_candidate_count = vocabulary.len().min(policy.max_candidate_facts());

        let mut admitted = vocabulary
            .iter()
            .take(policy.max_candidate_facts())
            .filter_map(|candidate| {
                Self::evaluate_candidate(episodes, &candidate.structure, policy)
            })
            .collect::<Vec<_>>();

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_invariants());

        InvariantDiscoveryResult {
            episode_count: episodes.len(),
            vocabulary_fact_count: vocabulary.len(),
            seeded_vocabulary_fact_count,
            evaluated_candidate_count,
            candidate_generation_truncated,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalInvariantDiscovery;

impl UniversalInvariantDiscovery {
    pub fn evaluate(
        episodes: &[GroundedTransformationEpisode],
        rule_seeds: &[GroundedRuleHypothesis],
        policy: InvariantDiscoveryPolicy,
    ) -> InvariantDiscoveryResult {
        InvariantDiscovery::discover(episodes, rule_seeds, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TransitionEffectKind {
    Added,
    Removed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TransitionSchemaPolicy {
    minimum_support: u64,
    minimum_precision: CognitiveSignal,
    minimum_association_lift: CognitiveSignal,
    max_candidate_effects: usize,
    max_schemas: usize,
}

impl TransitionSchemaPolicy {
    pub fn new(
        minimum_support: u64,
        minimum_precision: CognitiveSignal,
        minimum_association_lift: CognitiveSignal,
        max_candidate_effects: usize,
        max_schemas: usize,
    ) -> Option<Self> {
        if minimum_support == 0
            || minimum_association_lift == CognitiveSignal::zero()
            || max_candidate_effects == 0
            || max_schemas == 0
        {
            return None;
        }

        Some(Self {
            minimum_support,
            minimum_precision,
            minimum_association_lift,
            max_candidate_effects,
            max_schemas,
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

    pub fn max_candidate_effects(self) -> usize {
        self.max_candidate_effects
    }

    pub fn max_schemas(self) -> usize {
        self.max_schemas
    }
}

impl GroundedTransformationEpisode {
    pub fn effect_opportunity(
        &self,
        kind: TransitionEffectKind,
        fact: &CognitiveStructure,
    ) -> bool {
        match kind {
            TransitionEffectKind::Added => !self.before().contains_fact(fact),

            TransitionEffectKind::Removed => self.before().contains_fact(fact),
        }
    }

    pub fn effect_occurs(&self, kind: TransitionEffectKind, fact: &CognitiveStructure) -> bool {
        match kind {
            TransitionEffectKind::Added => {
                !self.before().contains_fact(fact) && self.after().contains_fact(fact)
            }

            TransitionEffectKind::Removed => {
                self.before().contains_fact(fact) && !self.after().contains_fact(fact)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedTransitionSchemaHypothesis {
    transformation: CognitiveStructure,
    effect_kind: TransitionEffectKind,
    fact: CognitiveStructure,
    support_count: u64,
    transformation_opportunity_count: u64,
    counterexample_count: u64,
    global_support_count: u64,
    global_opportunity_count: u64,
    precision: CognitiveSignal,
    baseline_rate: CognitiveSignal,
    association_lift: CognitiveSignal,
}

impl GroundedTransitionSchemaHypothesis {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn fact(&self) -> &CognitiveStructure {
        &self.fact
    }

    pub fn support_count(&self) -> u64 {
        self.support_count
    }

    pub fn transformation_opportunity_count(&self) -> u64 {
        self.transformation_opportunity_count
    }

    pub fn counterexample_count(&self) -> u64 {
        self.counterexample_count
    }

    pub fn global_support_count(&self) -> u64 {
        self.global_support_count
    }

    pub fn global_opportunity_count(&self) -> u64 {
        self.global_opportunity_count
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

    pub fn is_supported_by(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.transformation() == &self.transformation
            && episode.effect_occurs(self.effect_kind, &self.fact)
    }

    pub fn is_counterexample(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.transformation() == &self.transformation
            && episode.effect_opportunity(self.effect_kind, &self.fact)
            && !episode.effect_occurs(self.effect_kind, &self.fact)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TransitionEffectCandidate {
    transformation: CognitiveStructure,
    kind: TransitionEffectKind,
    fact: CognitiveStructure,
    observed_support: u64,
    invariance_penalty: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionSchemaInductionResult {
    transformation_count: usize,
    vocabulary_fact_count: usize,
    possible_effect_candidate_count: usize,
    evaluated_candidate_count: usize,
    candidate_generation_truncated: bool,
    invariant_seeded_fact_count: usize,
    admitted_before_frontier: usize,
    selected: Vec<GroundedTransitionSchemaHypothesis>,
}

impl TransitionSchemaInductionResult {
    pub fn transformation_count(&self) -> usize {
        self.transformation_count
    }

    pub fn vocabulary_fact_count(&self) -> usize {
        self.vocabulary_fact_count
    }

    pub fn possible_effect_candidate_count(&self) -> usize {
        self.possible_effect_candidate_count
    }

    pub fn evaluated_candidate_count(&self) -> usize {
        self.evaluated_candidate_count
    }

    pub fn candidate_generation_truncated(&self) -> bool {
        self.candidate_generation_truncated
    }

    pub fn invariant_seeded_fact_count(&self) -> usize {
        self.invariant_seeded_fact_count
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedTransitionSchemaHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct TransitionSchemaInduction;

impl TransitionSchemaInduction {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16)
            .expect("bounded transition-schema rate remains on signal scale")
    }

    fn positive_difference(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        CognitiveSignal::new(left.value().saturating_sub(right.value()))
            .expect("bounded transition-schema lift remains on signal scale")
    }

    fn transformations(episodes: &[GroundedTransformationEpisode]) -> Vec<CognitiveStructure> {
        let mut transformations = episodes
            .iter()
            .map(|episode| episode.transformation().clone())
            .collect::<Vec<_>>();

        transformations.sort_by(PredicateDiscovery::compare_structure);

        transformations.dedup();

        transformations
    }

    fn facts(episodes: &[GroundedTransformationEpisode]) -> Vec<CognitiveStructure> {
        let mut facts = episodes
            .iter()
            .flat_map(|episode| {
                episode
                    .before()
                    .facts()
                    .iter()
                    .chain(episode.after().facts().iter())
                    .cloned()
            })
            .collect::<Vec<_>>();

        facts.sort_by(PredicateDiscovery::compare_structure);

        facts.dedup();

        facts
    }

    fn invariance_penalty(
        fact: &CognitiveStructure,
        invariants: &[GroundedInvariantHypothesis],
    ) -> u16 {
        invariants
            .iter()
            .filter(|invariant| invariant.fact() == fact)
            .map(|invariant| invariant.preservation_rate().value())
            .max()
            .unwrap_or(0)
    }

    fn observed_support(
        episodes: &[GroundedTransformationEpisode],
        transformation: &CognitiveStructure,
        kind: TransitionEffectKind,
        fact: &CognitiveStructure,
    ) -> u64 {
        episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == transformation && episode.effect_occurs(kind, fact)
            })
            .count() as u64
    }

    fn candidates(
        episodes: &[GroundedTransformationEpisode],
        invariants: &[GroundedInvariantHypothesis],
    ) -> (Vec<TransitionEffectCandidate>, usize, usize) {
        let transformations = Self::transformations(episodes);

        let facts = Self::facts(episodes);

        let invariant_seeded_fact_count = facts
            .iter()
            .filter(|fact| Self::invariance_penalty(fact, invariants) > 0)
            .count();

        let possible = transformations
            .len()
            .saturating_mul(facts.len())
            .saturating_mul(2);

        let mut candidates = Vec::with_capacity(possible);

        for transformation in transformations {
            for kind in [TransitionEffectKind::Added, TransitionEffectKind::Removed] {
                for fact in &facts {
                    candidates.push(TransitionEffectCandidate {
                        observed_support: Self::observed_support(
                            episodes,
                            &transformation,
                            kind,
                            fact,
                        ),
                        invariance_penalty: Self::invariance_penalty(fact, invariants),
                        transformation: transformation.clone(),
                        kind,
                        fact: fact.clone(),
                    });
                }
            }
        }

        candidates.sort_by(|left, right| {
            right
                .observed_support
                .cmp(&left.observed_support)
                .then_with(|| left.invariance_penalty.cmp(&right.invariance_penalty))
                .then_with(|| {
                    PredicateDiscovery::compare_structure(
                        &left.transformation,
                        &right.transformation,
                    )
                })
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| PredicateDiscovery::compare_structure(&left.fact, &right.fact))
        });

        (candidates, facts.len(), invariant_seeded_fact_count)
    }

    fn evaluate_candidate(
        episodes: &[GroundedTransformationEpisode],
        candidate: &TransitionEffectCandidate,
        policy: TransitionSchemaPolicy,
    ) -> Option<GroundedTransitionSchemaHypothesis> {
        let transformation_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.transformation
                    && episode.effect_opportunity(candidate.kind, &candidate.fact)
            })
            .count() as u64;

        if transformation_opportunity_count == 0 {
            return None;
        }

        let support_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.transformation
                    && episode.effect_occurs(candidate.kind, &candidate.fact)
            })
            .count() as u64;

        if support_count == 0 {
            return None;
        }

        let global_opportunity_count = episodes
            .iter()
            .filter(|episode| episode.effect_opportunity(candidate.kind, &candidate.fact))
            .count() as u64;

        if global_opportunity_count == 0 {
            return None;
        }

        let global_support_count = episodes
            .iter()
            .filter(|episode| episode.effect_occurs(candidate.kind, &candidate.fact))
            .count() as u64;

        let counterexample_count = transformation_opportunity_count.saturating_sub(support_count);

        let precision = Self::scaled_rate(support_count, transformation_opportunity_count);

        let baseline_rate = Self::scaled_rate(global_support_count, global_opportunity_count);

        let association_lift = Self::positive_difference(precision, baseline_rate);

        if support_count < policy.minimum_support()
            || precision.value() < policy.minimum_precision().value()
            || association_lift.value() < policy.minimum_association_lift().value()
        {
            return None;
        }

        Some(GroundedTransitionSchemaHypothesis {
            transformation: candidate.transformation.clone(),
            effect_kind: candidate.kind,
            fact: candidate.fact.clone(),
            support_count,
            transformation_opportunity_count,
            counterexample_count,
            global_support_count,
            global_opportunity_count,
            precision,
            baseline_rate,
            association_lift,
        })
    }

    fn ranking(
        left: &GroundedTransitionSchemaHypothesis,
        right: &GroundedTransitionSchemaHypothesis,
    ) -> std::cmp::Ordering {
        right
            .association_lift()
            .value()
            .cmp(&left.association_lift().value())
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| {
                left.counterexample_count()
                    .cmp(&right.counterexample_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| PredicateDiscovery::compare_structure(left.fact(), right.fact()))
    }

    pub fn induce(
        episodes: &[GroundedTransformationEpisode],
        invariants: &[GroundedInvariantHypothesis],
        policy: TransitionSchemaPolicy,
    ) -> TransitionSchemaInductionResult {
        if episodes.is_empty() {
            return TransitionSchemaInductionResult {
                transformation_count: 0,
                vocabulary_fact_count: 0,
                possible_effect_candidate_count: 0,
                evaluated_candidate_count: 0,
                candidate_generation_truncated: false,
                invariant_seeded_fact_count: 0,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let transformation_count = Self::transformations(episodes).len();

        let (candidates, vocabulary_fact_count, invariant_seeded_fact_count) =
            Self::candidates(episodes, invariants);

        let possible_effect_candidate_count = candidates.len();

        let evaluated_candidate_count =
            possible_effect_candidate_count.min(policy.max_candidate_effects());

        let candidate_generation_truncated =
            possible_effect_candidate_count > policy.max_candidate_effects();

        let mut admitted = candidates
            .iter()
            .take(policy.max_candidate_effects())
            .filter_map(|candidate| Self::evaluate_candidate(episodes, candidate, policy))
            .collect::<Vec<_>>();

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_schemas());

        TransitionSchemaInductionResult {
            transformation_count,
            vocabulary_fact_count,
            possible_effect_candidate_count,
            evaluated_candidate_count,
            candidate_generation_truncated,
            invariant_seeded_fact_count,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalTransitionSchemaInduction;

impl UniversalTransitionSchemaInduction {
    pub fn evaluate(
        episodes: &[GroundedTransformationEpisode],
        invariants: &[GroundedInvariantHypothesis],
        policy: TransitionSchemaPolicy,
    ) -> TransitionSchemaInductionResult {
        TransitionSchemaInduction::induce(episodes, invariants, policy)
    }
}

pub const MAX_CONTEXT_PREMISES: usize = 6;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextPremiseSet {
    premises: Vec<CognitiveStructure>,
}

impl ContextPremiseSet {
    pub fn new(mut premises: Vec<CognitiveStructure>) -> Option<Self> {
        premises.sort_by(PredicateDiscovery::compare_structure);

        premises.dedup();

        if premises.is_empty() || premises.len() > MAX_CONTEXT_PREMISES {
            return None;
        }

        Some(Self { premises })
    }

    pub fn premises(&self) -> &[CognitiveStructure] {
        &self.premises
    }

    pub fn premise_count(&self) -> usize {
        self.premises.len()
    }

    pub fn is_satisfied_by(&self, state: &GroundedStateSnapshot) -> bool {
        self.premises
            .iter()
            .all(|premise| state.contains_fact(premise))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ContextualTransitionEvidenceThresholds {
    minimum_support: u64,
    minimum_precision: CognitiveSignal,
    minimum_association_lift: CognitiveSignal,
    minimum_incremental_gain: CognitiveSignal,
}

impl ContextualTransitionEvidenceThresholds {
    pub fn new(
        minimum_support: u64,
        minimum_precision: CognitiveSignal,
        minimum_association_lift: CognitiveSignal,
        minimum_incremental_gain: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_support == 0
            || minimum_association_lift == CognitiveSignal::zero()
            || minimum_incremental_gain == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_support,
            minimum_precision,
            minimum_association_lift,
            minimum_incremental_gain,
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

    pub fn minimum_incremental_gain(self) -> CognitiveSignal {
        self.minimum_incremental_gain
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ContextualTransitionRulePolicy {
    max_context_premises: usize,
    max_candidate_contexts: usize,
    max_rule_evaluations: usize,
    max_rules: usize,
    thresholds: ContextualTransitionEvidenceThresholds,
}

impl ContextualTransitionRulePolicy {
    pub fn new(
        max_context_premises: usize,
        max_candidate_contexts: usize,
        max_rule_evaluations: usize,
        max_rules: usize,
        thresholds: ContextualTransitionEvidenceThresholds,
    ) -> Option<Self> {
        if !(1..=MAX_CONTEXT_PREMISES).contains(&max_context_premises)
            || max_candidate_contexts == 0
            || max_rule_evaluations == 0
            || max_rules == 0
        {
            return None;
        }

        Some(Self {
            max_context_premises,
            max_candidate_contexts,
            max_rule_evaluations,
            max_rules,
            thresholds,
        })
    }

    pub fn max_context_premises(self) -> usize {
        self.max_context_premises
    }

    pub fn max_candidate_contexts(self) -> usize {
        self.max_candidate_contexts
    }

    pub fn max_rule_evaluations(self) -> usize {
        self.max_rule_evaluations
    }

    pub fn max_rules(self) -> usize {
        self.max_rules
    }

    pub fn thresholds(self) -> ContextualTransitionEvidenceThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedContextualTransitionRuleHypothesis {
    transformation: CognitiveStructure,
    context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    support_count: u64,
    context_opportunity_count: u64,
    transformation_support_count: u64,
    transformation_opportunity_count: u64,
    global_support_count: u64,
    global_opportunity_count: u64,
    counterexample_count: u64,
    precision: CognitiveSignal,
    transformation_precision: CognitiveSignal,
    baseline_rate: CognitiveSignal,
    association_lift: CognitiveSignal,
    incremental_precision_gain: CognitiveSignal,
}

impl GroundedContextualTransitionRuleHypothesis {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn context(&self) -> &ContextPremiseSet {
        &self.context
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn effect_fact(&self) -> &CognitiveStructure {
        &self.effect_fact
    }

    pub fn support_count(&self) -> u64 {
        self.support_count
    }

    pub fn context_opportunity_count(&self) -> u64 {
        self.context_opportunity_count
    }

    pub fn transformation_support_count(&self) -> u64 {
        self.transformation_support_count
    }

    pub fn transformation_opportunity_count(&self) -> u64 {
        self.transformation_opportunity_count
    }

    pub fn global_support_count(&self) -> u64 {
        self.global_support_count
    }

    pub fn global_opportunity_count(&self) -> u64 {
        self.global_opportunity_count
    }

    pub fn counterexample_count(&self) -> u64 {
        self.counterexample_count
    }

    pub fn precision(&self) -> CognitiveSignal {
        self.precision
    }

    pub fn transformation_precision(&self) -> CognitiveSignal {
        self.transformation_precision
    }

    pub fn baseline_rate(&self) -> CognitiveSignal {
        self.baseline_rate
    }

    pub fn association_lift(&self) -> CognitiveSignal {
        self.association_lift
    }

    pub fn incremental_precision_gain(&self) -> CognitiveSignal {
        self.incremental_precision_gain
    }

    pub fn is_supported_by(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.transformation() == &self.transformation
            && self.context.is_satisfied_by(episode.before())
            && episode.effect_occurs(self.effect_kind, &self.effect_fact)
    }

    pub fn is_counterexample(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.transformation() == &self.transformation
            && self.context.is_satisfied_by(episode.before())
            && episode.effect_opportunity(self.effect_kind, &self.effect_fact)
            && !episode.effect_occurs(self.effect_kind, &self.effect_fact)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ContextualEffectTarget {
    transformation: CognitiveStructure,
    kind: TransitionEffectKind,
    fact: CognitiveStructure,
    observed_support: u64,
    seed_lift: u16,
    seed_precision: u16,
    seed_support: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualTransitionRuleResult {
    vocabulary_fact_count: usize,
    effect_target_count: usize,
    schema_seeded_effect_target_count: usize,
    possible_context_count: usize,
    generated_context_count: usize,
    candidate_generation_truncated: bool,
    possible_rule_evaluation_count: usize,
    evaluated_rule_candidate_count: usize,
    rule_evaluation_truncated: bool,
    admitted_before_frontier: usize,
    selected: Vec<GroundedContextualTransitionRuleHypothesis>,
}

impl ContextualTransitionRuleResult {
    pub fn vocabulary_fact_count(&self) -> usize {
        self.vocabulary_fact_count
    }

    pub fn effect_target_count(&self) -> usize {
        self.effect_target_count
    }

    pub fn schema_seeded_effect_target_count(&self) -> usize {
        self.schema_seeded_effect_target_count
    }

    pub fn possible_context_count(&self) -> usize {
        self.possible_context_count
    }

    pub fn generated_context_count(&self) -> usize {
        self.generated_context_count
    }

    pub fn candidate_generation_truncated(&self) -> bool {
        self.candidate_generation_truncated
    }

    pub fn possible_rule_evaluation_count(&self) -> usize {
        self.possible_rule_evaluation_count
    }

    pub fn evaluated_rule_candidate_count(&self) -> usize {
        self.evaluated_rule_candidate_count
    }

    pub fn rule_evaluation_truncated(&self) -> bool {
        self.rule_evaluation_truncated
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedContextualTransitionRuleHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ContextualTransitionRuleInduction;

impl ContextualTransitionRuleInduction {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16)
            .expect("bounded contextual transition rate remains on signal scale")
    }

    fn positive_difference(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        CognitiveSignal::new(left.value().saturating_sub(right.value()))
            .expect("bounded contextual transition difference remains on signal scale")
    }

    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn compare_target_identity(
        left: &ContextualEffectTarget,
        right: &ContextualEffectTarget,
    ) -> std::cmp::Ordering {
        PredicateDiscovery::compare_structure(&left.transformation, &right.transformation)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| PredicateDiscovery::compare_structure(&left.fact, &right.fact))
    }

    fn schema_seed_strength(
        transformation: &CognitiveStructure,
        kind: TransitionEffectKind,
        fact: &CognitiveStructure,
        schemas: &[GroundedTransitionSchemaHypothesis],
    ) -> (u16, u16, u64) {
        schemas
            .iter()
            .filter(|schema| {
                schema.transformation() == transformation
                    && schema.effect_kind() == kind
                    && schema.fact() == fact
            })
            .map(|schema| {
                (
                    schema.association_lift().value(),
                    schema.precision().value(),
                    schema.support_count(),
                )
            })
            .max()
            .unwrap_or((0, 0, 0))
    }

    fn observed_support(
        episodes: &[GroundedTransformationEpisode],
        transformation: &CognitiveStructure,
        kind: TransitionEffectKind,
        fact: &CognitiveStructure,
    ) -> u64 {
        episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == transformation && episode.effect_occurs(kind, fact)
            })
            .count() as u64
    }

    fn effect_targets(
        episodes: &[GroundedTransformationEpisode],
        schemas: &[GroundedTransitionSchemaHypothesis],
    ) -> Vec<ContextualEffectTarget> {
        let mut targets = Vec::new();

        for episode in episodes {
            for fact in episode.after().facts() {
                if episode.effect_occurs(TransitionEffectKind::Added, fact) {
                    targets.push(ContextualEffectTarget {
                        transformation: episode.transformation().clone(),
                        kind: TransitionEffectKind::Added,
                        fact: fact.clone(),
                        observed_support: 0,
                        seed_lift: 0,
                        seed_precision: 0,
                        seed_support: 0,
                    });
                }
            }

            for fact in episode.before().facts() {
                if episode.effect_occurs(TransitionEffectKind::Removed, fact) {
                    targets.push(ContextualEffectTarget {
                        transformation: episode.transformation().clone(),
                        kind: TransitionEffectKind::Removed,
                        fact: fact.clone(),
                        observed_support: 0,
                        seed_lift: 0,
                        seed_precision: 0,
                        seed_support: 0,
                    });
                }
            }
        }

        targets.sort_by(Self::compare_target_identity);

        targets.dedup_by(|left, right| {
            Self::compare_target_identity(left, right) == std::cmp::Ordering::Equal
        });

        for target in &mut targets {
            target.observed_support =
                Self::observed_support(episodes, &target.transformation, target.kind, &target.fact);

            let (seed_lift, seed_precision, seed_support) = Self::schema_seed_strength(
                &target.transformation,
                target.kind,
                &target.fact,
                schemas,
            );

            target.seed_lift = seed_lift;

            target.seed_precision = seed_precision;

            target.seed_support = seed_support;
        }

        targets.sort_by(|left, right| {
            right
                .seed_lift
                .cmp(&left.seed_lift)
                .then_with(|| right.seed_precision.cmp(&left.seed_precision))
                .then_with(|| right.seed_support.cmp(&left.seed_support))
                .then_with(|| right.observed_support.cmp(&left.observed_support))
                .then_with(|| Self::compare_target_identity(left, right))
        });

        targets
    }

    fn context_vocabulary(episodes: &[GroundedTransformationEpisode]) -> Vec<CognitiveStructure> {
        let mut facts = episodes
            .iter()
            .flat_map(|episode| episode.before().facts().iter().cloned())
            .collect::<Vec<_>>();

        facts.sort_by(PredicateDiscovery::compare_structure);

        facts.dedup();

        facts
    }

    fn binomial_saturating(n: usize, k: usize) -> usize {
        if k > n {
            return 0;
        }

        let effective_k = k.min(n - k);

        let mut value = 1_u128;

        for index in 0..effective_k {
            value = value.saturating_mul((n - index) as u128) / (index + 1) as u128;

            if value > usize::MAX as u128 {
                return usize::MAX;
            }
        }

        value as usize
    }

    fn possible_context_count(vocabulary_len: usize, max_context_premises: usize) -> usize {
        let upper = max_context_premises.min(vocabulary_len);

        (1..=upper).fold(0_usize, |total, size| {
            total.saturating_add(Self::binomial_saturating(vocabulary_len, size))
        })
    }

    fn generate_context_combinations(
        vocabulary: &[CognitiveStructure],
        target_size: usize,
        start: usize,
        current: &mut Vec<CognitiveStructure>,
        output: &mut Vec<ContextPremiseSet>,
        limit: usize,
    ) {
        if output.len() >= limit {
            return;
        }

        if current.len() == target_size {
            output.push(
                ContextPremiseSet::new(current.clone())
                    .expect("generated contextual premise set is valid"),
            );

            return;
        }

        let remaining_needed = target_size.saturating_sub(current.len());

        if vocabulary.len() < start.saturating_add(remaining_needed) {
            return;
        }

        let last_start = vocabulary.len() - remaining_needed;

        for index in start..=last_start {
            if output.len() >= limit {
                return;
            }

            current.push(vocabulary[index].clone());

            Self::generate_context_combinations(
                vocabulary,
                target_size,
                index + 1,
                current,
                output,
                limit,
            );

            current.pop();
        }
    }

    fn contexts(
        vocabulary: &[CognitiveStructure],
        policy: ContextualTransitionRulePolicy,
    ) -> (Vec<ContextPremiseSet>, usize, bool) {
        let possible =
            Self::possible_context_count(vocabulary.len(), policy.max_context_premises());

        let upper = policy.max_context_premises().min(vocabulary.len());

        let mut generated = Vec::new();

        for target_size in 1..=upper {
            if generated.len() >= policy.max_candidate_contexts() {
                break;
            }

            Self::generate_context_combinations(
                vocabulary,
                target_size,
                0,
                &mut Vec::new(),
                &mut generated,
                policy.max_candidate_contexts(),
            );
        }

        let truncated = possible > generated.len();

        (generated, possible, truncated)
    }

    fn evaluate_rule(
        episodes: &[GroundedTransformationEpisode],
        target: &ContextualEffectTarget,
        context: &ContextPremiseSet,
        thresholds: ContextualTransitionEvidenceThresholds,
    ) -> Option<GroundedContextualTransitionRuleHypothesis> {
        let transformation_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &target.transformation
                    && episode.effect_opportunity(target.kind, &target.fact)
            })
            .count() as u64;

        if transformation_opportunity_count == 0 {
            return None;
        }

        let transformation_support_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &target.transformation
                    && episode.effect_occurs(target.kind, &target.fact)
            })
            .count() as u64;

        let context_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &target.transformation
                    && context.is_satisfied_by(episode.before())
                    && episode.effect_opportunity(target.kind, &target.fact)
            })
            .count() as u64;

        if context_opportunity_count == 0 {
            return None;
        }

        let support_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &target.transformation
                    && context.is_satisfied_by(episode.before())
                    && episode.effect_occurs(target.kind, &target.fact)
            })
            .count() as u64;

        if support_count == 0 {
            return None;
        }

        let global_opportunity_count = episodes
            .iter()
            .filter(|episode| episode.effect_opportunity(target.kind, &target.fact))
            .count() as u64;

        if global_opportunity_count == 0 {
            return None;
        }

        let global_support_count = episodes
            .iter()
            .filter(|episode| episode.effect_occurs(target.kind, &target.fact))
            .count() as u64;

        let counterexample_count = context_opportunity_count.saturating_sub(support_count);

        let precision = Self::scaled_rate(support_count, context_opportunity_count);

        let transformation_precision = Self::scaled_rate(
            transformation_support_count,
            transformation_opportunity_count,
        );

        let baseline_rate = Self::scaled_rate(global_support_count, global_opportunity_count);

        let association_lift = Self::positive_difference(precision, baseline_rate);

        let incremental_precision_gain =
            Self::positive_difference(precision, transformation_precision);

        if support_count < thresholds.minimum_support()
            || precision.value() < thresholds.minimum_precision().value()
            || association_lift.value() < thresholds.minimum_association_lift().value()
            || incremental_precision_gain.value() < thresholds.minimum_incremental_gain().value()
        {
            return None;
        }

        Some(GroundedContextualTransitionRuleHypothesis {
            transformation: target.transformation.clone(),
            context: context.clone(),
            effect_kind: target.kind,
            effect_fact: target.fact.clone(),
            support_count,
            context_opportunity_count,
            transformation_support_count,
            transformation_opportunity_count,
            global_support_count,
            global_opportunity_count,
            counterexample_count,
            precision,
            transformation_precision,
            baseline_rate,
            association_lift,
            incremental_precision_gain,
        })
    }

    fn ranking(
        left: &GroundedContextualTransitionRuleHypothesis,
        right: &GroundedContextualTransitionRuleHypothesis,
    ) -> std::cmp::Ordering {
        right
            .incremental_precision_gain()
            .value()
            .cmp(&left.incremental_precision_gain().value())
            .then_with(|| {
                right
                    .association_lift()
                    .value()
                    .cmp(&left.association_lift().value())
            })
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| {
                left.context()
                    .premise_count()
                    .cmp(&right.context().premise_count())
            })
            .then_with(|| {
                left.counterexample_count()
                    .cmp(&right.counterexample_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    pub fn induce(
        episodes: &[GroundedTransformationEpisode],
        schema_seeds: &[GroundedTransitionSchemaHypothesis],
        policy: ContextualTransitionRulePolicy,
    ) -> ContextualTransitionRuleResult {
        if episodes.is_empty() {
            return ContextualTransitionRuleResult {
                vocabulary_fact_count: 0,
                effect_target_count: 0,
                schema_seeded_effect_target_count: 0,
                possible_context_count: 0,
                generated_context_count: 0,
                candidate_generation_truncated: false,
                possible_rule_evaluation_count: 0,
                evaluated_rule_candidate_count: 0,
                rule_evaluation_truncated: false,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let vocabulary = Self::context_vocabulary(episodes);

        let targets = Self::effect_targets(episodes, schema_seeds);

        let schema_seeded_effect_target_count = targets
            .iter()
            .filter(|target| {
                target.seed_lift > 0 || target.seed_precision > 0 || target.seed_support > 0
            })
            .count();

        let (contexts, possible_context_count, candidate_generation_truncated) =
            Self::contexts(&vocabulary, policy);

        let possible_rule_evaluation_count = targets.len().saturating_mul(contexts.len());

        let mut evaluated_rule_candidate_count = 0_usize;

        let mut admitted = Vec::new();

        'targets: for target in &targets {
            for context in &contexts {
                if evaluated_rule_candidate_count >= policy.max_rule_evaluations() {
                    break 'targets;
                }

                evaluated_rule_candidate_count = evaluated_rule_candidate_count.saturating_add(1);

                if let Some(rule) =
                    Self::evaluate_rule(episodes, target, context, policy.thresholds())
                {
                    admitted.push(rule);
                }
            }
        }

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_rules());

        ContextualTransitionRuleResult {
            vocabulary_fact_count: vocabulary.len(),
            effect_target_count: targets.len(),
            schema_seeded_effect_target_count,
            possible_context_count,
            generated_context_count: contexts.len(),
            candidate_generation_truncated,
            possible_rule_evaluation_count,
            evaluated_rule_candidate_count,
            rule_evaluation_truncated: candidate_generation_truncated
                || possible_rule_evaluation_count > policy.max_rule_evaluations(),
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalContextualTransitionRuleInduction;

impl UniversalContextualTransitionRuleInduction {
    pub fn evaluate(
        episodes: &[GroundedTransformationEpisode],
        schema_seeds: &[GroundedTransitionSchemaHypothesis],
        policy: ContextualTransitionRulePolicy,
    ) -> ContextualTransitionRuleResult {
        ContextualTransitionRuleInduction::induce(episodes, schema_seeds, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CrossContextGeneralizationThresholds {
    minimum_seed_contexts: usize,
    minimum_support: u64,
    minimum_precision: CognitiveSignal,
    minimum_incremental_gain: CognitiveSignal,
}

impl CrossContextGeneralizationThresholds {
    pub fn new(
        minimum_seed_contexts: usize,
        minimum_support: u64,
        minimum_precision: CognitiveSignal,
        minimum_incremental_gain: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_seed_contexts < 2
            || minimum_support == 0
            || minimum_incremental_gain == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_seed_contexts,
            minimum_support,
            minimum_precision,
            minimum_incremental_gain,
        })
    }

    pub fn minimum_seed_contexts(self) -> usize {
        self.minimum_seed_contexts
    }

    pub fn minimum_support(self) -> u64 {
        self.minimum_support
    }

    pub fn minimum_precision(self) -> CognitiveSignal {
        self.minimum_precision
    }

    pub fn minimum_incremental_gain(self) -> CognitiveSignal {
        self.minimum_incremental_gain
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CrossContextGeneralizationPolicy {
    max_seed_rules: usize,
    max_generalized_premises: usize,
    max_candidate_generalizations: usize,
    max_generalizations: usize,
    thresholds: CrossContextGeneralizationThresholds,
}

impl CrossContextGeneralizationPolicy {
    pub fn new(
        max_seed_rules: usize,
        max_generalized_premises: usize,
        max_candidate_generalizations: usize,
        max_generalizations: usize,
        thresholds: CrossContextGeneralizationThresholds,
    ) -> Option<Self> {
        if max_seed_rules == 0
            || !(1..=MAX_CONTEXT_PREMISES).contains(&max_generalized_premises)
            || max_candidate_generalizations == 0
            || max_generalizations == 0
        {
            return None;
        }

        Some(Self {
            max_seed_rules,
            max_generalized_premises,
            max_candidate_generalizations,
            max_generalizations,
            thresholds,
        })
    }

    pub fn max_seed_rules(self) -> usize {
        self.max_seed_rules
    }

    pub fn max_generalized_premises(self) -> usize {
        self.max_generalized_premises
    }

    pub fn max_candidate_generalizations(self) -> usize {
        self.max_candidate_generalizations
    }

    pub fn max_generalizations(self) -> usize {
        self.max_generalizations
    }

    pub fn thresholds(self) -> CrossContextGeneralizationThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CrossContextTarget {
    transformation: CognitiveStructure,
    kind: TransitionEffectKind,
    fact: CognitiveStructure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CrossContextCandidate {
    target: CrossContextTarget,
    generalized_context: ContextPremiseSet,
    covered_seed_context_count: usize,
    minimum_premise_reduction: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedCrossContextGeneralizationHypothesis {
    transformation: CognitiveStructure,
    generalized_context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    covered_seed_context_count: usize,
    minimum_premise_reduction: usize,
    support_count: u64,
    context_opportunity_count: u64,
    transformation_support_count: u64,
    transformation_opportunity_count: u64,
    counterexample_count: u64,
    precision: CognitiveSignal,
    transformation_precision: CognitiveSignal,
    incremental_precision_gain: CognitiveSignal,
}

impl GroundedCrossContextGeneralizationHypothesis {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn generalized_context(&self) -> &ContextPremiseSet {
        &self.generalized_context
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn effect_fact(&self) -> &CognitiveStructure {
        &self.effect_fact
    }

    pub fn covered_seed_context_count(&self) -> usize {
        self.covered_seed_context_count
    }

    pub fn minimum_premise_reduction(&self) -> usize {
        self.minimum_premise_reduction
    }

    pub fn support_count(&self) -> u64 {
        self.support_count
    }

    pub fn context_opportunity_count(&self) -> u64 {
        self.context_opportunity_count
    }

    pub fn transformation_support_count(&self) -> u64 {
        self.transformation_support_count
    }

    pub fn transformation_opportunity_count(&self) -> u64 {
        self.transformation_opportunity_count
    }

    pub fn counterexample_count(&self) -> u64 {
        self.counterexample_count
    }

    pub fn precision(&self) -> CognitiveSignal {
        self.precision
    }

    pub fn transformation_precision(&self) -> CognitiveSignal {
        self.transformation_precision
    }

    pub fn incremental_precision_gain(&self) -> CognitiveSignal {
        self.incremental_precision_gain
    }

    pub fn is_supported_by(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.transformation() == &self.transformation
            && self.generalized_context.is_satisfied_by(episode.before())
            && episode.effect_occurs(self.effect_kind, &self.effect_fact)
    }

    pub fn is_counterexample(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.transformation() == &self.transformation
            && self.generalized_context.is_satisfied_by(episode.before())
            && episode.effect_opportunity(self.effect_kind, &self.effect_fact)
            && !episode.effect_occurs(self.effect_kind, &self.effect_fact)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossContextGeneralizationResult {
    input_seed_rule_count: usize,
    considered_seed_rule_count: usize,
    seed_rule_truncated: bool,
    possible_candidate_count: usize,
    evaluated_candidate_count: usize,
    candidate_generation_truncated: bool,
    admitted_before_frontier: usize,
    selected: Vec<GroundedCrossContextGeneralizationHypothesis>,
}

impl CrossContextGeneralizationResult {
    pub fn input_seed_rule_count(&self) -> usize {
        self.input_seed_rule_count
    }

    pub fn considered_seed_rule_count(&self) -> usize {
        self.considered_seed_rule_count
    }

    pub fn seed_rule_truncated(&self) -> bool {
        self.seed_rule_truncated
    }

    pub fn possible_candidate_count(&self) -> usize {
        self.possible_candidate_count
    }

    pub fn evaluated_candidate_count(&self) -> usize {
        self.evaluated_candidate_count
    }

    pub fn candidate_generation_truncated(&self) -> bool {
        self.candidate_generation_truncated
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedCrossContextGeneralizationHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CrossContextGeneralization;

impl CrossContextGeneralization {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16)
            .expect("bounded cross-context rate remains on signal scale")
    }

    fn positive_difference(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        CognitiveSignal::new(left.value().saturating_sub(right.value()))
            .expect("bounded cross-context gain remains on signal scale")
    }

    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn target_for_rule(rule: &GroundedContextualTransitionRuleHypothesis) -> CrossContextTarget {
        CrossContextTarget {
            transformation: rule.transformation().clone(),
            kind: rule.effect_kind(),
            fact: rule.effect_fact().clone(),
        }
    }

    fn compare_target(left: &CrossContextTarget, right: &CrossContextTarget) -> std::cmp::Ordering {
        PredicateDiscovery::compare_structure(&left.transformation, &right.transformation)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| PredicateDiscovery::compare_structure(&left.fact, &right.fact))
    }

    fn compare_seed_rule(
        left: &GroundedContextualTransitionRuleHypothesis,
        right: &GroundedContextualTransitionRuleHypothesis,
    ) -> std::cmp::Ordering {
        right
            .incremental_precision_gain()
            .value()
            .cmp(&left.incremental_precision_gain().value())
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| {
                let left_target = Self::target_for_rule(left);

                let right_target = Self::target_for_rule(right);

                Self::compare_target(&left_target, &right_target)
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    fn considered_seeds(
        seed_rules: &[GroundedContextualTransitionRuleHypothesis],
        policy: CrossContextGeneralizationPolicy,
    ) -> Vec<&GroundedContextualTransitionRuleHypothesis> {
        let mut seeds = seed_rules.iter().collect::<Vec<_>>();

        seeds.sort_by(|left, right| Self::compare_seed_rule(left, right));

        seeds.truncate(policy.max_seed_rules());

        seeds
    }

    fn generate_combinations(
        values: &[CognitiveStructure],
        target_size: usize,
        start: usize,
        current: &mut Vec<CognitiveStructure>,
        output: &mut Vec<ContextPremiseSet>,
    ) {
        if current.len() == target_size {
            output.push(
                ContextPremiseSet::new(current.clone())
                    .expect("generated generalized context remains valid"),
            );

            return;
        }

        let remaining = target_size.saturating_sub(current.len());

        if values.len() < start.saturating_add(remaining) {
            return;
        }

        let last_start = values.len() - remaining;

        for index in start..=last_start {
            current.push(values[index].clone());

            Self::generate_combinations(values, target_size, index + 1, current, output);

            current.pop();
        }
    }

    fn strictly_generalizes(candidate: &ContextPremiseSet, seed: &ContextPremiseSet) -> bool {
        candidate.premise_count() < seed.premise_count()
            && candidate
                .premises()
                .iter()
                .all(|premise| seed.premises().contains(premise))
    }

    fn compare_candidate_identity(
        left: &CrossContextCandidate,
        right: &CrossContextCandidate,
    ) -> std::cmp::Ordering {
        Self::compare_target(&left.target, &right.target).then_with(|| {
            Self::compare_context(&left.generalized_context, &right.generalized_context)
        })
    }

    fn candidates(
        considered: &[&GroundedContextualTransitionRuleHypothesis],
        policy: CrossContextGeneralizationPolicy,
    ) -> Vec<CrossContextCandidate> {
        let mut raw = Vec::new();

        for seed in considered {
            let premise_count = seed.context().premise_count();

            if premise_count < 2 {
                continue;
            }

            let upper = policy.max_generalized_premises().min(premise_count - 1);

            for size in 1..=upper {
                let mut subsets = Vec::new();

                Self::generate_combinations(
                    seed.context().premises(),
                    size,
                    0,
                    &mut Vec::new(),
                    &mut subsets,
                );

                for generalized_context in subsets {
                    raw.push(CrossContextCandidate {
                        target: Self::target_for_rule(seed),
                        generalized_context,
                        covered_seed_context_count: 0,
                        minimum_premise_reduction: 0,
                    });
                }
            }
        }

        raw.sort_by(Self::compare_candidate_identity);

        raw.dedup_by(|left, right| {
            Self::compare_candidate_identity(left, right) == std::cmp::Ordering::Equal
        });

        let mut qualified = Vec::new();

        for mut candidate in raw {
            let mut covered_contexts = considered
                .iter()
                .filter(|seed| {
                    Self::target_for_rule(seed) == candidate.target
                        && Self::strictly_generalizes(
                            &candidate.generalized_context,
                            seed.context(),
                        )
                })
                .map(|seed| seed.context().clone())
                .collect::<Vec<_>>();

            covered_contexts.sort_by(Self::compare_context);

            covered_contexts.dedup();

            if covered_contexts.len() < policy.thresholds().minimum_seed_contexts() {
                continue;
            }

            let minimum_premise_reduction = covered_contexts
                .iter()
                .map(|context| {
                    context.premise_count() - candidate.generalized_context.premise_count()
                })
                .min()
                .expect("qualified generalization has covered seed contexts");

            candidate.covered_seed_context_count = covered_contexts.len();

            candidate.minimum_premise_reduction = minimum_premise_reduction;

            qualified.push(candidate);
        }

        qualified.sort_by(|left, right| {
            right
                .covered_seed_context_count
                .cmp(&left.covered_seed_context_count)
                .then_with(|| {
                    right
                        .minimum_premise_reduction
                        .cmp(&left.minimum_premise_reduction)
                })
                .then_with(|| {
                    left.generalized_context
                        .premise_count()
                        .cmp(&right.generalized_context.premise_count())
                })
                .then_with(|| Self::compare_candidate_identity(left, right))
        });

        qualified
    }

    fn evaluate_candidate(
        episodes: &[GroundedTransformationEpisode],
        candidate: &CrossContextCandidate,
        thresholds: CrossContextGeneralizationThresholds,
    ) -> Option<GroundedCrossContextGeneralizationHypothesis> {
        let transformation_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.target.transformation
                    && episode.effect_opportunity(candidate.target.kind, &candidate.target.fact)
            })
            .count() as u64;

        if transformation_opportunity_count == 0 {
            return None;
        }

        let transformation_support_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.target.transformation
                    && episode.effect_occurs(candidate.target.kind, &candidate.target.fact)
            })
            .count() as u64;

        let context_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.target.transformation
                    && candidate
                        .generalized_context
                        .is_satisfied_by(episode.before())
                    && episode.effect_opportunity(candidate.target.kind, &candidate.target.fact)
            })
            .count() as u64;

        if context_opportunity_count == 0 {
            return None;
        }

        let support_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.target.transformation
                    && candidate
                        .generalized_context
                        .is_satisfied_by(episode.before())
                    && episode.effect_occurs(candidate.target.kind, &candidate.target.fact)
            })
            .count() as u64;

        if support_count == 0 {
            return None;
        }

        let counterexample_count = context_opportunity_count.saturating_sub(support_count);

        let precision = Self::scaled_rate(support_count, context_opportunity_count);

        let transformation_precision = Self::scaled_rate(
            transformation_support_count,
            transformation_opportunity_count,
        );

        let incremental_precision_gain =
            Self::positive_difference(precision, transformation_precision);

        if support_count < thresholds.minimum_support()
            || precision.value() < thresholds.minimum_precision().value()
            || incremental_precision_gain.value() < thresholds.minimum_incremental_gain().value()
        {
            return None;
        }

        Some(GroundedCrossContextGeneralizationHypothesis {
            transformation: candidate.target.transformation.clone(),
            generalized_context: candidate.generalized_context.clone(),
            effect_kind: candidate.target.kind,
            effect_fact: candidate.target.fact.clone(),
            covered_seed_context_count: candidate.covered_seed_context_count,
            minimum_premise_reduction: candidate.minimum_premise_reduction,
            support_count,
            context_opportunity_count,
            transformation_support_count,
            transformation_opportunity_count,
            counterexample_count,
            precision,
            transformation_precision,
            incremental_precision_gain,
        })
    }

    fn ranking(
        left: &GroundedCrossContextGeneralizationHypothesis,
        right: &GroundedCrossContextGeneralizationHypothesis,
    ) -> std::cmp::Ordering {
        right
            .incremental_precision_gain()
            .value()
            .cmp(&left.incremental_precision_gain().value())
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| {
                right
                    .covered_seed_context_count()
                    .cmp(&left.covered_seed_context_count())
            })
            .then_with(|| {
                right
                    .minimum_premise_reduction()
                    .cmp(&left.minimum_premise_reduction())
            })
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| {
                left.generalized_context()
                    .premise_count()
                    .cmp(&right.generalized_context().premise_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| {
                Self::compare_context(left.generalized_context(), right.generalized_context())
            })
    }

    pub fn generalize(
        episodes: &[GroundedTransformationEpisode],
        seed_rules: &[GroundedContextualTransitionRuleHypothesis],
        policy: CrossContextGeneralizationPolicy,
    ) -> CrossContextGeneralizationResult {
        if episodes.is_empty() || seed_rules.is_empty() {
            return CrossContextGeneralizationResult {
                input_seed_rule_count: seed_rules.len(),
                considered_seed_rule_count: 0,
                seed_rule_truncated: false,
                possible_candidate_count: 0,
                evaluated_candidate_count: 0,
                candidate_generation_truncated: false,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let considered = Self::considered_seeds(seed_rules, policy);

        let candidates = Self::candidates(&considered, policy);

        let possible_candidate_count = candidates.len();

        let evaluated_candidate_count =
            possible_candidate_count.min(policy.max_candidate_generalizations());

        let candidate_generation_truncated =
            possible_candidate_count > policy.max_candidate_generalizations();

        let mut admitted = candidates
            .iter()
            .take(policy.max_candidate_generalizations())
            .filter_map(|candidate| {
                Self::evaluate_candidate(episodes, candidate, policy.thresholds())
            })
            .collect::<Vec<_>>();

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_generalizations());

        CrossContextGeneralizationResult {
            input_seed_rule_count: seed_rules.len(),
            considered_seed_rule_count: considered.len(),
            seed_rule_truncated: seed_rules.len() > considered.len(),
            possible_candidate_count,
            evaluated_candidate_count,
            candidate_generation_truncated,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalCrossContextGeneralization;

impl UniversalCrossContextGeneralization {
    pub fn evaluate(
        episodes: &[GroundedTransformationEpisode],
        seed_rules: &[GroundedContextualTransitionRuleHypothesis],
        policy: CrossContextGeneralizationPolicy,
    ) -> CrossContextGeneralizationResult {
        CrossContextGeneralization::generalize(episodes, seed_rules, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExceptionRefinementThresholds {
    minimum_failure_support: u64,
    minimum_exception_failure_rate: CognitiveSignal,
    minimum_failure_lift: CognitiveSignal,
}

impl ExceptionRefinementThresholds {
    pub fn new(
        minimum_failure_support: u64,
        minimum_exception_failure_rate: CognitiveSignal,
        minimum_failure_lift: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_failure_support == 0
            || minimum_exception_failure_rate == CognitiveSignal::zero()
            || minimum_failure_lift == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_failure_support,
            minimum_exception_failure_rate,
            minimum_failure_lift,
        })
    }

    pub fn minimum_failure_support(self) -> u64 {
        self.minimum_failure_support
    }

    pub fn minimum_exception_failure_rate(self) -> CognitiveSignal {
        self.minimum_exception_failure_rate
    }

    pub fn minimum_failure_lift(self) -> CognitiveSignal {
        self.minimum_failure_lift
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExceptionRefinementPolicy {
    max_seed_generalizations: usize,
    max_exception_premises: usize,
    max_candidate_contexts: usize,
    max_evaluations: usize,
    max_refinements: usize,
    thresholds: ExceptionRefinementThresholds,
}

impl ExceptionRefinementPolicy {
    pub fn new(
        max_seed_generalizations: usize,
        max_exception_premises: usize,
        max_candidate_contexts: usize,
        max_evaluations: usize,
        max_refinements: usize,
        thresholds: ExceptionRefinementThresholds,
    ) -> Option<Self> {
        if max_seed_generalizations == 0
            || !(1..=MAX_CONTEXT_PREMISES).contains(&max_exception_premises)
            || max_candidate_contexts == 0
            || max_evaluations == 0
            || max_refinements == 0
        {
            return None;
        }

        Some(Self {
            max_seed_generalizations,
            max_exception_premises,
            max_candidate_contexts,
            max_evaluations,
            max_refinements,
            thresholds,
        })
    }

    pub fn max_seed_generalizations(self) -> usize {
        self.max_seed_generalizations
    }

    pub fn max_exception_premises(self) -> usize {
        self.max_exception_premises
    }

    pub fn max_candidate_contexts(self) -> usize {
        self.max_candidate_contexts
    }

    pub fn max_evaluations(self) -> usize {
        self.max_evaluations
    }

    pub fn max_refinements(self) -> usize {
        self.max_refinements
    }

    pub fn thresholds(self) -> ExceptionRefinementThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExceptionCandidate {
    transformation: CognitiveStructure,
    base_context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    exception_context: ContextPremiseSet,
    failure_seed_support: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedExceptionRefinementHypothesis {
    transformation: CognitiveStructure,
    base_context: ContextPremiseSet,
    exception_context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    base_opportunity_count: u64,
    base_failure_count: u64,
    exception_opportunity_count: u64,
    exception_failure_count: u64,
    exception_success_count: u64,
    base_failure_rate: CognitiveSignal,
    exception_failure_rate: CognitiveSignal,
    failure_lift: CognitiveSignal,
    failure_coverage: CognitiveSignal,
}

impl GroundedExceptionRefinementHypothesis {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn base_context(&self) -> &ContextPremiseSet {
        &self.base_context
    }

    pub fn exception_context(&self) -> &ContextPremiseSet {
        &self.exception_context
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn effect_fact(&self) -> &CognitiveStructure {
        &self.effect_fact
    }

    pub fn base_opportunity_count(&self) -> u64 {
        self.base_opportunity_count
    }

    pub fn base_failure_count(&self) -> u64 {
        self.base_failure_count
    }

    pub fn exception_opportunity_count(&self) -> u64 {
        self.exception_opportunity_count
    }

    pub fn exception_failure_count(&self) -> u64 {
        self.exception_failure_count
    }

    pub fn exception_success_count(&self) -> u64 {
        self.exception_success_count
    }

    pub fn base_failure_rate(&self) -> CognitiveSignal {
        self.base_failure_rate
    }

    pub fn exception_failure_rate(&self) -> CognitiveSignal {
        self.exception_failure_rate
    }

    pub fn failure_lift(&self) -> CognitiveSignal {
        self.failure_lift
    }

    pub fn failure_coverage(&self) -> CognitiveSignal {
        self.failure_coverage
    }

    pub fn is_triggered_by(&self, episode: &GroundedTransformationEpisode) -> bool {
        episode.transformation() == &self.transformation
            && self.base_context.is_satisfied_by(episode.before())
            && self.exception_context.is_satisfied_by(episode.before())
            && episode.effect_opportunity(self.effect_kind, &self.effect_fact)
    }

    pub fn explains_counterexample(&self, episode: &GroundedTransformationEpisode) -> bool {
        self.is_triggered_by(episode) && !episode.effect_occurs(self.effect_kind, &self.effect_fact)
    }

    pub fn leaks_on_support(&self, episode: &GroundedTransformationEpisode) -> bool {
        self.is_triggered_by(episode) && episode.effect_occurs(self.effect_kind, &self.effect_fact)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExceptionRefinementResult {
    input_seed_count: usize,
    considered_seed_count: usize,
    seed_truncated: bool,
    possible_candidate_context_count: usize,
    generated_candidate_context_count: usize,
    candidate_generation_truncated: bool,
    evaluated_candidate_count: usize,
    evaluation_truncated: bool,
    admitted_before_frontier: usize,
    selected: Vec<GroundedExceptionRefinementHypothesis>,
}

impl ExceptionRefinementResult {
    pub fn input_seed_count(&self) -> usize {
        self.input_seed_count
    }

    pub fn considered_seed_count(&self) -> usize {
        self.considered_seed_count
    }

    pub fn seed_truncated(&self) -> bool {
        self.seed_truncated
    }

    pub fn possible_candidate_context_count(&self) -> usize {
        self.possible_candidate_context_count
    }

    pub fn generated_candidate_context_count(&self) -> usize {
        self.generated_candidate_context_count
    }

    pub fn candidate_generation_truncated(&self) -> bool {
        self.candidate_generation_truncated
    }

    pub fn evaluated_candidate_count(&self) -> usize {
        self.evaluated_candidate_count
    }

    pub fn evaluation_truncated(&self) -> bool {
        self.evaluation_truncated
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedExceptionRefinementHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ExceptionRefinement;

impl ExceptionRefinement {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16)
            .expect("bounded exception-refinement rate remains on signal scale")
    }

    fn positive_difference(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        CognitiveSignal::new(left.value().saturating_sub(right.value()))
            .expect("bounded exception-refinement lift remains on signal scale")
    }

    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn compare_seed(
        left: &GroundedCrossContextGeneralizationHypothesis,
        right: &GroundedCrossContextGeneralizationHypothesis,
    ) -> std::cmp::Ordering {
        right
            .incremental_precision_gain()
            .value()
            .cmp(&left.incremental_precision_gain().value())
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| {
                Self::compare_context(left.generalized_context(), right.generalized_context())
            })
    }

    fn considered_seeds(
        seeds: &[GroundedCrossContextGeneralizationHypothesis],
        policy: ExceptionRefinementPolicy,
    ) -> Vec<&GroundedCrossContextGeneralizationHypothesis> {
        let mut considered = seeds.iter().collect::<Vec<_>>();

        considered.sort_by(|left, right| Self::compare_seed(left, right));

        considered.truncate(policy.max_seed_generalizations());

        considered
    }

    fn binomial_saturating(n: usize, k: usize) -> usize {
        if k > n {
            return 0;
        }

        let effective_k = k.min(n - k);

        let mut value = 1_u128;

        for index in 0..effective_k {
            value = value.saturating_mul((n - index) as u128) / (index + 1) as u128;

            if value > usize::MAX as u128 {
                return usize::MAX;
            }
        }

        value as usize
    }

    fn possible_context_count(vocabulary_len: usize, max_exception_premises: usize) -> usize {
        let upper = vocabulary_len.min(max_exception_premises);

        (1..=upper).fold(0_usize, |total, size| {
            total.saturating_add(Self::binomial_saturating(vocabulary_len, size))
        })
    }

    fn failure_vocabulary(
        episodes: &[GroundedTransformationEpisode],
        seed: &GroundedCrossContextGeneralizationHypothesis,
    ) -> Vec<CognitiveStructure> {
        let mut counts = Vec::<(CognitiveStructure, u64)>::new();

        for episode in episodes {
            if !seed.is_counterexample(episode) {
                continue;
            }

            for fact in episode.before().facts() {
                if seed.generalized_context().premises().contains(fact) {
                    continue;
                }

                if let Some(existing) = counts
                    .iter_mut()
                    .find(|(existing_fact, _)| existing_fact == fact)
                {
                    existing.1 = existing.1.saturating_add(1);
                } else {
                    counts.push((fact.clone(), 1));
                }
            }
        }

        counts.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| PredicateDiscovery::compare_structure(&left.0, &right.0))
        });

        counts.into_iter().map(|(fact, _)| fact).collect()
    }

    fn generate_combinations(
        values: &[CognitiveStructure],
        target_size: usize,
        start: usize,
        current: &mut Vec<CognitiveStructure>,
        output: &mut Vec<ContextPremiseSet>,
        limit: usize,
    ) {
        if output.len() >= limit {
            return;
        }

        if current.len() == target_size {
            output.push(
                ContextPremiseSet::new(current.clone())
                    .expect("generated exception context remains valid"),
            );

            return;
        }

        let remaining = target_size.saturating_sub(current.len());

        if values.len() < start.saturating_add(remaining) {
            return;
        }

        let last_start = values.len() - remaining;

        for index in start..=last_start {
            if output.len() >= limit {
                return;
            }

            current.push(values[index].clone());

            Self::generate_combinations(values, target_size, index + 1, current, output, limit);

            current.pop();
        }
    }

    fn failure_seed_support(
        episodes: &[GroundedTransformationEpisode],
        seed: &GroundedCrossContextGeneralizationHypothesis,
        context: &ContextPremiseSet,
    ) -> u64 {
        episodes
            .iter()
            .filter(|episode| {
                seed.is_counterexample(episode) && context.is_satisfied_by(episode.before())
            })
            .count() as u64
    }

    fn candidates(
        episodes: &[GroundedTransformationEpisode],
        considered: &[&GroundedCrossContextGeneralizationHypothesis],
        policy: ExceptionRefinementPolicy,
    ) -> (Vec<ExceptionCandidate>, usize) {
        let mut possible_total = 0_usize;

        let mut candidates = Vec::new();

        for seed in considered {
            let vocabulary = Self::failure_vocabulary(episodes, seed);

            possible_total = possible_total.saturating_add(Self::possible_context_count(
                vocabulary.len(),
                policy.max_exception_premises(),
            ));

            if candidates.len() >= policy.max_candidate_contexts() {
                continue;
            }

            let remaining_budget = policy
                .max_candidate_contexts()
                .saturating_sub(candidates.len());

            let upper = vocabulary.len().min(policy.max_exception_premises());

            let mut generated = Vec::new();

            for size in 1..=upper {
                if generated.len() >= remaining_budget {
                    break;
                }

                Self::generate_combinations(
                    &vocabulary,
                    size,
                    0,
                    &mut Vec::new(),
                    &mut generated,
                    remaining_budget,
                );
            }

            for exception_context in generated {
                candidates.push(ExceptionCandidate {
                    transformation: seed.transformation().clone(),
                    base_context: seed.generalized_context().clone(),
                    effect_kind: seed.effect_kind(),
                    effect_fact: seed.effect_fact().clone(),
                    failure_seed_support: Self::failure_seed_support(
                        episodes,
                        seed,
                        &exception_context,
                    ),
                    exception_context,
                });
            }
        }

        candidates.sort_by(|left, right| {
            right
                .failure_seed_support
                .cmp(&left.failure_seed_support)
                .then_with(|| {
                    PredicateDiscovery::compare_structure(
                        &left.transformation,
                        &right.transformation,
                    )
                })
                .then_with(|| left.effect_kind.cmp(&right.effect_kind))
                .then_with(|| {
                    PredicateDiscovery::compare_structure(&left.effect_fact, &right.effect_fact)
                })
                .then_with(|| Self::compare_context(&left.base_context, &right.base_context))
                .then_with(|| {
                    Self::compare_context(&left.exception_context, &right.exception_context)
                })
        });

        (candidates, possible_total)
    }

    fn evaluate_candidate(
        episodes: &[GroundedTransformationEpisode],
        candidate: &ExceptionCandidate,
        thresholds: ExceptionRefinementThresholds,
    ) -> Option<GroundedExceptionRefinementHypothesis> {
        let base_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.transformation
                    && candidate.base_context.is_satisfied_by(episode.before())
                    && episode.effect_opportunity(candidate.effect_kind, &candidate.effect_fact)
            })
            .count() as u64;

        if base_opportunity_count == 0 {
            return None;
        }

        let base_failure_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.transformation
                    && candidate.base_context.is_satisfied_by(episode.before())
                    && episode.effect_opportunity(candidate.effect_kind, &candidate.effect_fact)
                    && !episode.effect_occurs(candidate.effect_kind, &candidate.effect_fact)
            })
            .count() as u64;

        if base_failure_count == 0 {
            return None;
        }

        let exception_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.transformation
                    && candidate.base_context.is_satisfied_by(episode.before())
                    && candidate
                        .exception_context
                        .is_satisfied_by(episode.before())
                    && episode.effect_opportunity(candidate.effect_kind, &candidate.effect_fact)
            })
            .count() as u64;

        if exception_opportunity_count == 0 {
            return None;
        }

        let exception_failure_count = episodes
            .iter()
            .filter(|episode| {
                episode.transformation() == &candidate.transformation
                    && candidate.base_context.is_satisfied_by(episode.before())
                    && candidate
                        .exception_context
                        .is_satisfied_by(episode.before())
                    && episode.effect_opportunity(candidate.effect_kind, &candidate.effect_fact)
                    && !episode.effect_occurs(candidate.effect_kind, &candidate.effect_fact)
            })
            .count() as u64;

        if exception_failure_count == 0 {
            return None;
        }

        let exception_success_count =
            exception_opportunity_count.saturating_sub(exception_failure_count);

        let base_failure_rate = Self::scaled_rate(base_failure_count, base_opportunity_count);

        let exception_failure_rate =
            Self::scaled_rate(exception_failure_count, exception_opportunity_count);

        let failure_lift = Self::positive_difference(exception_failure_rate, base_failure_rate);

        let failure_coverage = Self::scaled_rate(exception_failure_count, base_failure_count);

        if exception_failure_count < thresholds.minimum_failure_support()
            || exception_failure_rate.value() < thresholds.minimum_exception_failure_rate().value()
            || failure_lift.value() < thresholds.minimum_failure_lift().value()
        {
            return None;
        }

        Some(GroundedExceptionRefinementHypothesis {
            transformation: candidate.transformation.clone(),
            base_context: candidate.base_context.clone(),
            exception_context: candidate.exception_context.clone(),
            effect_kind: candidate.effect_kind,
            effect_fact: candidate.effect_fact.clone(),
            base_opportunity_count,
            base_failure_count,
            exception_opportunity_count,
            exception_failure_count,
            exception_success_count,
            base_failure_rate,
            exception_failure_rate,
            failure_lift,
            failure_coverage,
        })
    }

    fn ranking(
        left: &GroundedExceptionRefinementHypothesis,
        right: &GroundedExceptionRefinementHypothesis,
    ) -> std::cmp::Ordering {
        right
            .failure_lift()
            .value()
            .cmp(&left.failure_lift().value())
            .then_with(|| {
                right
                    .exception_failure_rate()
                    .value()
                    .cmp(&left.exception_failure_rate().value())
            })
            .then_with(|| {
                right
                    .failure_coverage()
                    .value()
                    .cmp(&left.failure_coverage().value())
            })
            .then_with(|| {
                right
                    .exception_failure_count()
                    .cmp(&left.exception_failure_count())
            })
            .then_with(|| {
                left.exception_success_count()
                    .cmp(&right.exception_success_count())
            })
            .then_with(|| {
                left.exception_context()
                    .premise_count()
                    .cmp(&right.exception_context().premise_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.base_context(), right.base_context()))
            .then_with(|| {
                Self::compare_context(left.exception_context(), right.exception_context())
            })
    }

    pub fn refine(
        episodes: &[GroundedTransformationEpisode],
        seeds: &[GroundedCrossContextGeneralizationHypothesis],
        policy: ExceptionRefinementPolicy,
    ) -> ExceptionRefinementResult {
        if episodes.is_empty() || seeds.is_empty() {
            return ExceptionRefinementResult {
                input_seed_count: seeds.len(),
                considered_seed_count: 0,
                seed_truncated: false,
                possible_candidate_context_count: 0,
                generated_candidate_context_count: 0,
                candidate_generation_truncated: false,
                evaluated_candidate_count: 0,
                evaluation_truncated: false,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let considered = Self::considered_seeds(seeds, policy);

        let (candidates, possible_candidate_context_count) =
            Self::candidates(episodes, &considered, policy);

        let generated_candidate_context_count = candidates.len();

        let candidate_generation_truncated =
            possible_candidate_context_count > generated_candidate_context_count;

        let evaluated_candidate_count =
            generated_candidate_context_count.min(policy.max_evaluations());

        let evaluation_truncated = generated_candidate_context_count > evaluated_candidate_count;

        let mut admitted = candidates
            .iter()
            .take(policy.max_evaluations())
            .filter_map(|candidate| {
                Self::evaluate_candidate(episodes, candidate, policy.thresholds())
            })
            .collect::<Vec<_>>();

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_refinements());

        ExceptionRefinementResult {
            input_seed_count: seeds.len(),
            considered_seed_count: considered.len(),
            seed_truncated: seeds.len() > considered.len(),
            possible_candidate_context_count,
            generated_candidate_context_count,
            candidate_generation_truncated,
            evaluated_candidate_count,
            evaluation_truncated,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalExceptionRefinement;

impl UniversalExceptionRefinement {
    pub fn evaluate(
        episodes: &[GroundedTransformationEpisode],
        seeds: &[GroundedCrossContextGeneralizationHypothesis],
        policy: ExceptionRefinementPolicy,
    ) -> ExceptionRefinementResult {
        ExceptionRefinement::refine(episodes, seeds, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuleConfidenceCalibrationPolicy {
    minimum_effective_support: u64,
    full_confidence_support: u64,
    minimum_calibrated_confidence: CognitiveSignal,
    max_seed_rules: usize,
    max_exception_checks: usize,
    max_calibrated_rules: usize,
}

impl RuleConfidenceCalibrationPolicy {
    pub fn new(
        minimum_effective_support: u64,
        full_confidence_support: u64,
        minimum_calibrated_confidence: CognitiveSignal,
        max_seed_rules: usize,
        max_exception_checks: usize,
        max_calibrated_rules: usize,
    ) -> Option<Self> {
        if minimum_effective_support == 0
            || full_confidence_support == 0
            || full_confidence_support < minimum_effective_support
            || max_seed_rules == 0
            || max_exception_checks == 0
            || max_calibrated_rules == 0
        {
            return None;
        }

        Some(Self {
            minimum_effective_support,
            full_confidence_support,
            minimum_calibrated_confidence,
            max_seed_rules,
            max_exception_checks,
            max_calibrated_rules,
        })
    }

    pub fn minimum_effective_support(self) -> u64 {
        self.minimum_effective_support
    }

    pub fn full_confidence_support(self) -> u64 {
        self.full_confidence_support
    }

    pub fn minimum_calibrated_confidence(self) -> CognitiveSignal {
        self.minimum_calibrated_confidence
    }

    pub fn max_seed_rules(self) -> usize {
        self.max_seed_rules
    }

    pub fn max_exception_checks(self) -> usize {
        self.max_exception_checks
    }

    pub fn max_calibrated_rules(self) -> usize {
        self.max_calibrated_rules
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibratedRuleConfidence {
    transformation: CognitiveStructure,
    context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    total_opportunity_count: u64,
    total_success_count: u64,
    total_failure_count: u64,
    matching_exception_count: usize,
    checked_exception_count: usize,
    exception_check_truncated: bool,
    exception_triggered_opportunity_count: u64,
    exception_triggered_failure_count: u64,
    exception_triggered_success_count: u64,
    effective_opportunity_count: u64,
    effective_success_count: u64,
    effective_failure_count: u64,
    raw_precision: CognitiveSignal,
    effective_precision: CognitiveSignal,
    support_adequacy: CognitiveSignal,
    calibrated_confidence: CognitiveSignal,
    abstention_rate: CognitiveSignal,
}

impl CalibratedRuleConfidence {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn context(&self) -> &ContextPremiseSet {
        &self.context
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn effect_fact(&self) -> &CognitiveStructure {
        &self.effect_fact
    }

    pub fn total_opportunity_count(&self) -> u64 {
        self.total_opportunity_count
    }

    pub fn total_success_count(&self) -> u64 {
        self.total_success_count
    }

    pub fn total_failure_count(&self) -> u64 {
        self.total_failure_count
    }

    pub fn matching_exception_count(&self) -> usize {
        self.matching_exception_count
    }

    pub fn checked_exception_count(&self) -> usize {
        self.checked_exception_count
    }

    pub fn exception_check_truncated(&self) -> bool {
        self.exception_check_truncated
    }

    pub fn exception_triggered_opportunity_count(&self) -> u64 {
        self.exception_triggered_opportunity_count
    }

    pub fn exception_triggered_failure_count(&self) -> u64 {
        self.exception_triggered_failure_count
    }

    pub fn exception_triggered_success_count(&self) -> u64 {
        self.exception_triggered_success_count
    }

    pub fn effective_opportunity_count(&self) -> u64 {
        self.effective_opportunity_count
    }

    pub fn effective_success_count(&self) -> u64 {
        self.effective_success_count
    }

    pub fn effective_failure_count(&self) -> u64 {
        self.effective_failure_count
    }

    pub fn raw_precision(&self) -> CognitiveSignal {
        self.raw_precision
    }

    pub fn effective_precision(&self) -> CognitiveSignal {
        self.effective_precision
    }

    pub fn support_adequacy(&self) -> CognitiveSignal {
        self.support_adequacy
    }

    pub fn calibrated_confidence(&self) -> CognitiveSignal {
        self.calibrated_confidence
    }

    pub fn abstention_rate(&self) -> CognitiveSignal {
        self.abstention_rate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleConfidenceCalibrationResult {
    input_seed_count: usize,
    considered_seed_count: usize,
    seed_truncated: bool,
    total_matching_exception_count: usize,
    total_checked_exception_count: usize,
    exception_check_budget_exhausted: bool,
    rejected_insufficient_support: usize,
    rejected_below_confidence: usize,
    admitted_before_frontier: usize,
    selected: Vec<CalibratedRuleConfidence>,
}

impl RuleConfidenceCalibrationResult {
    pub fn input_seed_count(&self) -> usize {
        self.input_seed_count
    }

    pub fn considered_seed_count(&self) -> usize {
        self.considered_seed_count
    }

    pub fn seed_truncated(&self) -> bool {
        self.seed_truncated
    }

    pub fn total_matching_exception_count(&self) -> usize {
        self.total_matching_exception_count
    }

    pub fn total_checked_exception_count(&self) -> usize {
        self.total_checked_exception_count
    }

    pub fn exception_check_budget_exhausted(&self) -> bool {
        self.exception_check_budget_exhausted
    }

    pub fn rejected_insufficient_support(&self) -> usize {
        self.rejected_insufficient_support
    }

    pub fn rejected_below_confidence(&self) -> usize {
        self.rejected_below_confidence
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[CalibratedRuleConfidence] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct RuleConfidenceCalibration;

impl RuleConfidenceCalibration {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16)
            .expect("bounded rule confidence rate remains on signal scale")
    }

    fn scaled_product(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        let product = (u32::from(left.value()) * u32::from(right.value())) / 1000;

        CognitiveSignal::new(product as u16)
            .expect("bounded rule confidence product remains on signal scale")
    }

    fn support_adequacy(support: u64, full_confidence_support: u64) -> CognitiveSignal {
        if support >= full_confidence_support {
            return CognitiveSignal::new(1000).expect("full support confidence is on signal scale");
        }

        Self::scaled_rate(support, full_confidence_support)
    }

    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn compare_seed(
        left: &GroundedCrossContextGeneralizationHypothesis,
        right: &GroundedCrossContextGeneralizationHypothesis,
    ) -> std::cmp::Ordering {
        right
            .incremental_precision_gain()
            .value()
            .cmp(&left.incremental_precision_gain().value())
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| {
                Self::compare_context(left.generalized_context(), right.generalized_context())
            })
    }

    fn considered_seed_indices(
        seeds: &[GroundedCrossContextGeneralizationHypothesis],
        policy: RuleConfidenceCalibrationPolicy,
    ) -> Vec<usize> {
        let mut indices = (0..seeds.len()).collect::<Vec<_>>();

        indices.sort_by(|left, right| Self::compare_seed(&seeds[*left], &seeds[*right]));

        indices.truncate(policy.max_seed_rules());

        indices
    }

    fn exception_matches_seed(
        exception: &GroundedExceptionRefinementHypothesis,
        seed: &GroundedCrossContextGeneralizationHypothesis,
    ) -> bool {
        exception.transformation() == seed.transformation()
            && exception.base_context() == seed.generalized_context()
            && exception.effect_kind() == seed.effect_kind()
            && exception.effect_fact() == seed.effect_fact()
    }

    fn compare_exception(
        left: &GroundedExceptionRefinementHypothesis,
        right: &GroundedExceptionRefinementHypothesis,
    ) -> std::cmp::Ordering {
        right
            .failure_lift()
            .value()
            .cmp(&left.failure_lift().value())
            .then_with(|| {
                right
                    .exception_failure_rate()
                    .value()
                    .cmp(&left.exception_failure_rate().value())
            })
            .then_with(|| {
                right
                    .failure_coverage()
                    .value()
                    .cmp(&left.failure_coverage().value())
            })
            .then_with(|| {
                Self::compare_context(left.exception_context(), right.exception_context())
            })
    }

    fn matching_exception_indices(
        seed: &GroundedCrossContextGeneralizationHypothesis,
        exceptions: &[GroundedExceptionRefinementHypothesis],
    ) -> Vec<usize> {
        let mut indices = exceptions
            .iter()
            .enumerate()
            .filter_map(|(index, exception)| {
                if Self::exception_matches_seed(exception, seed) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        indices.sort_by(|left, right| {
            Self::compare_exception(&exceptions[*left], &exceptions[*right])
        });

        indices
    }

    fn is_rule_opportunity(
        seed: &GroundedCrossContextGeneralizationHypothesis,
        episode: &GroundedTransformationEpisode,
    ) -> bool {
        episode.transformation() == seed.transformation()
            && seed.generalized_context().is_satisfied_by(episode.before())
            && episode.effect_opportunity(seed.effect_kind(), seed.effect_fact())
    }

    fn any_checked_exception_triggered(
        episode: &GroundedTransformationEpisode,
        checked_indices: &[usize],
        exceptions: &[GroundedExceptionRefinementHypothesis],
    ) -> bool {
        checked_indices
            .iter()
            .any(|index| exceptions[*index].is_triggered_by(episode))
    }

    fn calibrate_seed(
        episodes: &[GroundedTransformationEpisode],
        seed: &GroundedCrossContextGeneralizationHypothesis,
        matching_exception_count: usize,
        checked_indices: &[usize],
        exceptions: &[GroundedExceptionRefinementHypothesis],
        policy: RuleConfidenceCalibrationPolicy,
    ) -> Option<CalibratedRuleConfidence> {
        let opportunities = episodes
            .iter()
            .filter(|episode| Self::is_rule_opportunity(seed, episode))
            .collect::<Vec<_>>();

        if opportunities.is_empty() {
            return None;
        }

        let total_opportunity_count = opportunities.len() as u64;

        let total_success_count = opportunities
            .iter()
            .filter(|episode| episode.effect_occurs(seed.effect_kind(), seed.effect_fact()))
            .count() as u64;

        let total_failure_count = total_opportunity_count.saturating_sub(total_success_count);

        let mut exception_triggered_opportunity_count = 0_u64;

        let mut exception_triggered_failure_count = 0_u64;

        let mut exception_triggered_success_count = 0_u64;

        for episode in &opportunities {
            if !Self::any_checked_exception_triggered(episode, checked_indices, exceptions) {
                continue;
            }

            exception_triggered_opportunity_count =
                exception_triggered_opportunity_count.saturating_add(1);

            if episode.effect_occurs(seed.effect_kind(), seed.effect_fact()) {
                exception_triggered_success_count =
                    exception_triggered_success_count.saturating_add(1);
            } else {
                exception_triggered_failure_count =
                    exception_triggered_failure_count.saturating_add(1);
            }
        }

        let effective_opportunity_count =
            total_opportunity_count.saturating_sub(exception_triggered_opportunity_count);

        if effective_opportunity_count == 0 {
            return None;
        }

        let effective_success_count =
            total_success_count.saturating_sub(exception_triggered_success_count);

        let effective_failure_count =
            effective_opportunity_count.saturating_sub(effective_success_count);

        let raw_precision = Self::scaled_rate(total_success_count, total_opportunity_count);

        let effective_precision =
            Self::scaled_rate(effective_success_count, effective_opportunity_count);

        let support_adequacy = Self::support_adequacy(
            effective_opportunity_count,
            policy.full_confidence_support(),
        );

        let calibrated_confidence = Self::scaled_product(effective_precision, support_adequacy);

        let abstention_rate = Self::scaled_rate(
            exception_triggered_opportunity_count,
            total_opportunity_count,
        );

        Some(CalibratedRuleConfidence {
            transformation: seed.transformation().clone(),
            context: seed.generalized_context().clone(),
            effect_kind: seed.effect_kind(),
            effect_fact: seed.effect_fact().clone(),
            total_opportunity_count,
            total_success_count,
            total_failure_count,
            matching_exception_count,
            checked_exception_count: checked_indices.len(),
            exception_check_truncated: matching_exception_count > checked_indices.len(),
            exception_triggered_opportunity_count,
            exception_triggered_failure_count,
            exception_triggered_success_count,
            effective_opportunity_count,
            effective_success_count,
            effective_failure_count,
            raw_precision,
            effective_precision,
            support_adequacy,
            calibrated_confidence,
            abstention_rate,
        })
    }

    fn ranking(
        left: &CalibratedRuleConfidence,
        right: &CalibratedRuleConfidence,
    ) -> std::cmp::Ordering {
        right
            .calibrated_confidence()
            .value()
            .cmp(&left.calibrated_confidence().value())
            .then_with(|| {
                right
                    .effective_precision()
                    .value()
                    .cmp(&left.effective_precision().value())
            })
            .then_with(|| {
                right
                    .support_adequacy()
                    .value()
                    .cmp(&left.support_adequacy().value())
            })
            .then_with(|| {
                right
                    .effective_opportunity_count()
                    .cmp(&left.effective_opportunity_count())
            })
            .then_with(|| {
                left.effective_failure_count()
                    .cmp(&right.effective_failure_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    pub fn calibrate(
        episodes: &[GroundedTransformationEpisode],
        seeds: &[GroundedCrossContextGeneralizationHypothesis],
        exceptions: &[GroundedExceptionRefinementHypothesis],
        policy: RuleConfidenceCalibrationPolicy,
    ) -> RuleConfidenceCalibrationResult {
        if episodes.is_empty() || seeds.is_empty() {
            return RuleConfidenceCalibrationResult {
                input_seed_count: seeds.len(),
                considered_seed_count: 0,
                seed_truncated: false,
                total_matching_exception_count: 0,
                total_checked_exception_count: 0,
                exception_check_budget_exhausted: false,
                rejected_insufficient_support: 0,
                rejected_below_confidence: 0,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let considered_indices = Self::considered_seed_indices(seeds, policy);

        let mut remaining_exception_checks = policy.max_exception_checks();

        let mut total_matching_exception_count = 0_usize;

        let mut total_checked_exception_count = 0_usize;

        let mut rejected_insufficient_support = 0_usize;

        let mut rejected_below_confidence = 0_usize;

        let mut admitted = Vec::new();

        for seed_index in &considered_indices {
            let seed = &seeds[*seed_index];

            let matching_indices = Self::matching_exception_indices(seed, exceptions);

            total_matching_exception_count =
                total_matching_exception_count.saturating_add(matching_indices.len());

            let checked_count = matching_indices.len().min(remaining_exception_checks);

            let checked_indices = &matching_indices[..checked_count];

            remaining_exception_checks = remaining_exception_checks.saturating_sub(checked_count);

            total_checked_exception_count =
                total_checked_exception_count.saturating_add(checked_count);

            let Some(calibrated) = Self::calibrate_seed(
                episodes,
                seed,
                matching_indices.len(),
                checked_indices,
                exceptions,
                policy,
            ) else {
                rejected_insufficient_support = rejected_insufficient_support.saturating_add(1);

                continue;
            };

            if calibrated.effective_opportunity_count() < policy.minimum_effective_support() {
                rejected_insufficient_support = rejected_insufficient_support.saturating_add(1);

                continue;
            }

            if calibrated.calibrated_confidence().value()
                < policy.minimum_calibrated_confidence().value()
            {
                rejected_below_confidence = rejected_below_confidence.saturating_add(1);

                continue;
            }

            admitted.push(calibrated);
        }

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_calibrated_rules());

        RuleConfidenceCalibrationResult {
            input_seed_count: seeds.len(),
            considered_seed_count: considered_indices.len(),
            seed_truncated: seeds.len() > considered_indices.len(),
            total_matching_exception_count,
            total_checked_exception_count,
            exception_check_budget_exhausted: total_matching_exception_count
                > total_checked_exception_count,
            rejected_insufficient_support,
            rejected_below_confidence,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalRuleConfidenceCalibration;

impl UniversalRuleConfidenceCalibration {
    pub fn evaluate(
        episodes: &[GroundedTransformationEpisode],
        seeds: &[GroundedCrossContextGeneralizationHypothesis],
        exceptions: &[GroundedExceptionRefinementHypothesis],
        policy: RuleConfidenceCalibrationPolicy,
    ) -> RuleConfidenceCalibrationResult {
        RuleConfidenceCalibration::calibrate(episodes, seeds, exceptions, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CausalContrastThresholds {
    minimum_matched_states: usize,
    minimum_target_opportunities: u64,
    minimum_contrast_opportunities: u64,
    minimum_contrast_lift: CognitiveSignal,
    minimum_contrast_confidence: CognitiveSignal,
}

impl CausalContrastThresholds {
    pub fn new(
        minimum_matched_states: usize,
        minimum_target_opportunities: u64,
        minimum_contrast_opportunities: u64,
        minimum_contrast_lift: CognitiveSignal,
        minimum_contrast_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_matched_states == 0
            || minimum_target_opportunities == 0
            || minimum_contrast_opportunities == 0
            || minimum_contrast_lift == CognitiveSignal::zero()
            || minimum_contrast_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_matched_states,
            minimum_target_opportunities,
            minimum_contrast_opportunities,
            minimum_contrast_lift,
            minimum_contrast_confidence,
        })
    }

    pub fn minimum_matched_states(self) -> usize {
        self.minimum_matched_states
    }

    pub fn minimum_target_opportunities(self) -> u64 {
        self.minimum_target_opportunities
    }

    pub fn minimum_contrast_opportunities(self) -> u64 {
        self.minimum_contrast_opportunities
    }

    pub fn minimum_contrast_lift(self) -> CognitiveSignal {
        self.minimum_contrast_lift
    }

    pub fn minimum_contrast_confidence(self) -> CognitiveSignal {
        self.minimum_contrast_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CausalContrastPolicy {
    max_seed_rules: usize,
    max_contrasts_per_seed: usize,
    max_evaluations: usize,
    max_hypotheses: usize,
    thresholds: CausalContrastThresholds,
}

impl CausalContrastPolicy {
    pub fn new(
        max_seed_rules: usize,
        max_contrasts_per_seed: usize,
        max_evaluations: usize,
        max_hypotheses: usize,
        thresholds: CausalContrastThresholds,
    ) -> Option<Self> {
        if max_seed_rules == 0
            || max_contrasts_per_seed == 0
            || max_evaluations == 0
            || max_hypotheses == 0
        {
            return None;
        }

        Some(Self {
            max_seed_rules,
            max_contrasts_per_seed,
            max_evaluations,
            max_hypotheses,
            thresholds,
        })
    }

    pub fn max_seed_rules(self) -> usize {
        self.max_seed_rules
    }

    pub fn max_contrasts_per_seed(self) -> usize {
        self.max_contrasts_per_seed
    }

    pub fn max_evaluations(self) -> usize {
        self.max_evaluations
    }

    pub fn max_hypotheses(self) -> usize {
        self.max_hypotheses
    }

    pub fn thresholds(self) -> CausalContrastThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CausalContrastCandidate {
    transformation: CognitiveStructure,
    contrast_transformation: CognitiveStructure,
    context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    seed_calibrated_confidence: CognitiveSignal,
    matched_state_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedCausalContrastHypothesis {
    transformation: CognitiveStructure,
    contrast_transformation: CognitiveStructure,
    context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    matched_state_count: usize,
    target_opportunity_count: u64,
    target_success_count: u64,
    target_failure_count: u64,
    contrast_opportunity_count: u64,
    contrast_success_count: u64,
    contrast_failure_count: u64,
    target_effect_rate: CognitiveSignal,
    contrast_effect_rate: CognitiveSignal,
    contrast_lift: CognitiveSignal,
    seed_calibrated_confidence: CognitiveSignal,
    contrast_confidence: CognitiveSignal,
}

impl GroundedCausalContrastHypothesis {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn contrast_transformation(&self) -> &CognitiveStructure {
        &self.contrast_transformation
    }

    pub fn context(&self) -> &ContextPremiseSet {
        &self.context
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn effect_fact(&self) -> &CognitiveStructure {
        &self.effect_fact
    }

    pub fn matched_state_count(&self) -> usize {
        self.matched_state_count
    }

    pub fn target_opportunity_count(&self) -> u64 {
        self.target_opportunity_count
    }

    pub fn target_success_count(&self) -> u64 {
        self.target_success_count
    }

    pub fn target_failure_count(&self) -> u64 {
        self.target_failure_count
    }

    pub fn contrast_opportunity_count(&self) -> u64 {
        self.contrast_opportunity_count
    }

    pub fn contrast_success_count(&self) -> u64 {
        self.contrast_success_count
    }

    pub fn contrast_failure_count(&self) -> u64 {
        self.contrast_failure_count
    }

    pub fn target_effect_rate(&self) -> CognitiveSignal {
        self.target_effect_rate
    }

    pub fn contrast_effect_rate(&self) -> CognitiveSignal {
        self.contrast_effect_rate
    }

    pub fn contrast_lift(&self) -> CognitiveSignal {
        self.contrast_lift
    }

    pub fn seed_calibrated_confidence(&self) -> CognitiveSignal {
        self.seed_calibrated_confidence
    }

    pub fn contrast_confidence(&self) -> CognitiveSignal {
        self.contrast_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalContrastInductionResult {
    input_seed_count: usize,
    considered_seed_count: usize,
    seed_truncated: bool,
    possible_contrast_count: usize,
    generated_contrast_count: usize,
    contrast_generation_truncated: bool,
    evaluated_candidate_count: usize,
    evaluation_truncated: bool,
    admitted_before_frontier: usize,
    selected: Vec<GroundedCausalContrastHypothesis>,
}

impl CausalContrastInductionResult {
    pub fn input_seed_count(&self) -> usize {
        self.input_seed_count
    }

    pub fn considered_seed_count(&self) -> usize {
        self.considered_seed_count
    }

    pub fn seed_truncated(&self) -> bool {
        self.seed_truncated
    }

    pub fn possible_contrast_count(&self) -> usize {
        self.possible_contrast_count
    }

    pub fn generated_contrast_count(&self) -> usize {
        self.generated_contrast_count
    }

    pub fn contrast_generation_truncated(&self) -> bool {
        self.contrast_generation_truncated
    }

    pub fn evaluated_candidate_count(&self) -> usize {
        self.evaluated_candidate_count
    }

    pub fn evaluation_truncated(&self) -> bool {
        self.evaluation_truncated
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedCausalContrastHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CausalContrastInduction;

impl CausalContrastInduction {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16)
            .expect("bounded causal contrast rate remains on signal scale")
    }

    fn positive_difference(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        CognitiveSignal::new(left.value().saturating_sub(right.value()))
            .expect("bounded causal contrast lift remains on signal scale")
    }

    fn scaled_product(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        let product = (u32::from(left.value()) * u32::from(right.value())) / 1000;

        CognitiveSignal::new(product as u16)
            .expect("bounded causal contrast confidence remains on signal scale")
    }

    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn compare_seed(
        left: &CalibratedRuleConfidence,
        right: &CalibratedRuleConfidence,
    ) -> std::cmp::Ordering {
        right
            .calibrated_confidence()
            .value()
            .cmp(&left.calibrated_confidence().value())
            .then_with(|| {
                right
                    .effective_precision()
                    .value()
                    .cmp(&left.effective_precision().value())
            })
            .then_with(|| {
                right
                    .support_adequacy()
                    .value()
                    .cmp(&left.support_adequacy().value())
            })
            .then_with(|| {
                right
                    .effective_opportunity_count()
                    .cmp(&left.effective_opportunity_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    fn considered_seeds(
        seeds: &[CalibratedRuleConfidence],
        policy: CausalContrastPolicy,
    ) -> Vec<&CalibratedRuleConfidence> {
        let mut considered = seeds.iter().collect::<Vec<_>>();

        considered.sort_by(|left, right| Self::compare_seed(left, right));

        considered.truncate(policy.max_seed_rules());

        considered
    }

    fn target_episode_matches(
        seed: &CalibratedRuleConfidence,
        episode: &GroundedTransformationEpisode,
    ) -> bool {
        episode.transformation() == seed.transformation()
            && seed.context().is_satisfied_by(episode.before())
            && episode.effect_opportunity(seed.effect_kind(), seed.effect_fact())
    }

    fn contrast_episode_matches(
        seed: &CalibratedRuleConfidence,
        contrast: &CognitiveStructure,
        episode: &GroundedTransformationEpisode,
    ) -> bool {
        episode.transformation() == contrast
            && seed.context().is_satisfied_by(episode.before())
            && episode.effect_opportunity(seed.effect_kind(), seed.effect_fact())
    }

    fn state_has_target(
        episodes: &[GroundedTransformationEpisode],
        seed: &CalibratedRuleConfidence,
        state: &GroundedStateSnapshot,
    ) -> bool {
        episodes
            .iter()
            .any(|episode| episode.before() == state && Self::target_episode_matches(seed, episode))
    }

    fn state_has_contrast(
        episodes: &[GroundedTransformationEpisode],
        seed: &CalibratedRuleConfidence,
        contrast: &CognitiveStructure,
        state: &GroundedStateSnapshot,
    ) -> bool {
        episodes.iter().any(|episode| {
            episode.before() == state && Self::contrast_episode_matches(seed, contrast, episode)
        })
    }

    fn matched_states(
        episodes: &[GroundedTransformationEpisode],
        seed: &CalibratedRuleConfidence,
        contrast: &CognitiveStructure,
    ) -> Vec<GroundedStateSnapshot> {
        let mut states = Vec::new();

        for episode in episodes {
            if !Self::target_episode_matches(seed, episode) {
                continue;
            }

            if !Self::state_has_contrast(episodes, seed, contrast, episode.before()) {
                continue;
            }

            if !states.contains(episode.before()) {
                states.push(episode.before().clone());
            }
        }

        states
    }

    fn possible_contrasts(
        episodes: &[GroundedTransformationEpisode],
        seed: &CalibratedRuleConfidence,
    ) -> Vec<(CognitiveStructure, usize)> {
        let mut transformations = Vec::<CognitiveStructure>::new();

        for episode in episodes {
            if episode.transformation() == seed.transformation() {
                continue;
            }

            if !seed.context().is_satisfied_by(episode.before()) {
                continue;
            }

            if !episode.effect_opportunity(seed.effect_kind(), seed.effect_fact()) {
                continue;
            }

            if !Self::state_has_target(episodes, seed, episode.before()) {
                continue;
            }

            if !transformations.contains(episode.transformation()) {
                transformations.push(episode.transformation().clone());
            }
        }

        let mut contrasts = transformations
            .into_iter()
            .map(|transformation| {
                let matched_state_count =
                    Self::matched_states(episodes, seed, &transformation).len();

                (transformation, matched_state_count)
            })
            .collect::<Vec<_>>();

        contrasts.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| PredicateDiscovery::compare_structure(&left.0, &right.0))
        });

        contrasts
    }

    fn candidates(
        episodes: &[GroundedTransformationEpisode],
        considered: &[&CalibratedRuleConfidence],
        policy: CausalContrastPolicy,
    ) -> (Vec<CausalContrastCandidate>, usize) {
        let mut candidates = Vec::new();

        let mut possible_contrast_count = 0_usize;

        for seed in considered {
            let contrasts = Self::possible_contrasts(episodes, seed);

            possible_contrast_count = possible_contrast_count.saturating_add(contrasts.len());

            for (contrast_transformation, matched_state_count) in
                contrasts.into_iter().take(policy.max_contrasts_per_seed())
            {
                candidates.push(CausalContrastCandidate {
                    transformation: seed.transformation().clone(),
                    contrast_transformation,
                    context: seed.context().clone(),
                    effect_kind: seed.effect_kind(),
                    effect_fact: seed.effect_fact().clone(),
                    seed_calibrated_confidence: seed.calibrated_confidence(),
                    matched_state_count,
                });
            }
        }

        (candidates, possible_contrast_count)
    }

    fn candidate_episode_matches(
        candidate: &CausalContrastCandidate,
        transformation: &CognitiveStructure,
        episode: &GroundedTransformationEpisode,
    ) -> bool {
        episode.transformation() == transformation
            && candidate.context.is_satisfied_by(episode.before())
            && episode.effect_opportunity(candidate.effect_kind, &candidate.effect_fact)
    }

    fn candidate_matched_states(
        episodes: &[GroundedTransformationEpisode],
        candidate: &CausalContrastCandidate,
    ) -> Vec<GroundedStateSnapshot> {
        let mut states = Vec::new();

        for episode in episodes {
            if !Self::candidate_episode_matches(candidate, &candidate.transformation, episode) {
                continue;
            }

            let has_contrast = episodes.iter().any(|other| {
                other.before() == episode.before()
                    && Self::candidate_episode_matches(
                        candidate,
                        &candidate.contrast_transformation,
                        other,
                    )
            });

            if has_contrast && !states.contains(episode.before()) {
                states.push(episode.before().clone());
            }
        }

        states
    }

    fn episode_in_matched_states(
        episode: &GroundedTransformationEpisode,
        matched_states: &[GroundedStateSnapshot],
    ) -> bool {
        matched_states.contains(episode.before())
    }

    fn evaluate_candidate(
        episodes: &[GroundedTransformationEpisode],
        candidate: &CausalContrastCandidate,
        thresholds: CausalContrastThresholds,
    ) -> Option<GroundedCausalContrastHypothesis> {
        let matched_states = Self::candidate_matched_states(episodes, candidate);

        let matched_state_count = matched_states.len();

        if matched_state_count < thresholds.minimum_matched_states() {
            return None;
        }

        let target_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                Self::candidate_episode_matches(candidate, &candidate.transformation, episode)
                    && Self::episode_in_matched_states(episode, &matched_states)
            })
            .count() as u64;

        let contrast_opportunity_count = episodes
            .iter()
            .filter(|episode| {
                Self::candidate_episode_matches(
                    candidate,
                    &candidate.contrast_transformation,
                    episode,
                ) && Self::episode_in_matched_states(episode, &matched_states)
            })
            .count() as u64;

        if target_opportunity_count < thresholds.minimum_target_opportunities()
            || contrast_opportunity_count < thresholds.minimum_contrast_opportunities()
        {
            return None;
        }

        let target_success_count = episodes
            .iter()
            .filter(|episode| {
                Self::candidate_episode_matches(candidate, &candidate.transformation, episode)
                    && Self::episode_in_matched_states(episode, &matched_states)
                    && episode.effect_occurs(candidate.effect_kind, &candidate.effect_fact)
            })
            .count() as u64;

        let contrast_success_count = episodes
            .iter()
            .filter(|episode| {
                Self::candidate_episode_matches(
                    candidate,
                    &candidate.contrast_transformation,
                    episode,
                ) && Self::episode_in_matched_states(episode, &matched_states)
                    && episode.effect_occurs(candidate.effect_kind, &candidate.effect_fact)
            })
            .count() as u64;

        let target_failure_count = target_opportunity_count.saturating_sub(target_success_count);

        let contrast_failure_count =
            contrast_opportunity_count.saturating_sub(contrast_success_count);

        let target_effect_rate = Self::scaled_rate(target_success_count, target_opportunity_count);

        let contrast_effect_rate =
            Self::scaled_rate(contrast_success_count, contrast_opportunity_count);

        let contrast_lift = Self::positive_difference(target_effect_rate, contrast_effect_rate);

        let contrast_confidence =
            Self::scaled_product(contrast_lift, candidate.seed_calibrated_confidence);

        if contrast_lift.value() < thresholds.minimum_contrast_lift().value()
            || contrast_confidence.value() < thresholds.minimum_contrast_confidence().value()
        {
            return None;
        }

        Some(GroundedCausalContrastHypothesis {
            transformation: candidate.transformation.clone(),
            contrast_transformation: candidate.contrast_transformation.clone(),
            context: candidate.context.clone(),
            effect_kind: candidate.effect_kind,
            effect_fact: candidate.effect_fact.clone(),
            matched_state_count,
            target_opportunity_count,
            target_success_count,
            target_failure_count,
            contrast_opportunity_count,
            contrast_success_count,
            contrast_failure_count,
            target_effect_rate,
            contrast_effect_rate,
            contrast_lift,
            seed_calibrated_confidence: candidate.seed_calibrated_confidence,
            contrast_confidence,
        })
    }

    fn ranking(
        left: &GroundedCausalContrastHypothesis,
        right: &GroundedCausalContrastHypothesis,
    ) -> std::cmp::Ordering {
        right
            .contrast_confidence()
            .value()
            .cmp(&left.contrast_confidence().value())
            .then_with(|| {
                right
                    .contrast_lift()
                    .value()
                    .cmp(&left.contrast_lift().value())
            })
            .then_with(|| right.matched_state_count().cmp(&left.matched_state_count()))
            .then_with(|| {
                right
                    .target_opportunity_count()
                    .cmp(&left.target_opportunity_count())
            })
            .then_with(|| {
                left.target_failure_count()
                    .cmp(&right.target_failure_count())
            })
            .then_with(|| {
                left.contrast_success_count()
                    .cmp(&right.contrast_success_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.contrast_transformation(),
                    right.contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    pub fn induce(
        episodes: &[GroundedTransformationEpisode],
        seeds: &[CalibratedRuleConfidence],
        policy: CausalContrastPolicy,
    ) -> CausalContrastInductionResult {
        if episodes.is_empty() || seeds.is_empty() {
            return CausalContrastInductionResult {
                input_seed_count: seeds.len(),
                considered_seed_count: 0,
                seed_truncated: false,
                possible_contrast_count: 0,
                generated_contrast_count: 0,
                contrast_generation_truncated: false,
                evaluated_candidate_count: 0,
                evaluation_truncated: false,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let considered = Self::considered_seeds(seeds, policy);

        let (candidates, possible_contrast_count) = Self::candidates(episodes, &considered, policy);

        let generated_contrast_count = candidates.len();

        let contrast_generation_truncated = possible_contrast_count > generated_contrast_count;

        let evaluated_candidate_count = generated_contrast_count.min(policy.max_evaluations());

        let evaluation_truncated = generated_contrast_count > evaluated_candidate_count;

        let mut admitted = candidates
            .iter()
            .take(policy.max_evaluations())
            .filter_map(|candidate| {
                Self::evaluate_candidate(episodes, candidate, policy.thresholds())
            })
            .collect::<Vec<_>>();

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_hypotheses());

        CausalContrastInductionResult {
            input_seed_count: seeds.len(),
            considered_seed_count: considered.len(),
            seed_truncated: seeds.len() > considered.len(),
            possible_contrast_count,
            generated_contrast_count,
            contrast_generation_truncated,
            evaluated_candidate_count,
            evaluation_truncated,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalCausalContrastInduction;

impl UniversalCausalContrastInduction {
    pub fn evaluate(
        episodes: &[GroundedTransformationEpisode],
        seeds: &[CalibratedRuleConfidence],
        policy: CausalContrastPolicy,
    ) -> CausalContrastInductionResult {
        CausalContrastInduction::induce(episodes, seeds, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InterventionEvidenceKind {
    PassiveObservation,
    ControlledAssignment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterventionalTransformationEpisode {
    episode: GroundedTransformationEpisode,
    evidence_kind: InterventionEvidenceKind,
}

impl InterventionalTransformationEpisode {
    pub fn new(
        episode: GroundedTransformationEpisode,
        evidence_kind: InterventionEvidenceKind,
    ) -> Self {
        Self {
            episode,
            evidence_kind,
        }
    }

    pub fn controlled(episode: GroundedTransformationEpisode) -> Self {
        Self::new(episode, InterventionEvidenceKind::ControlledAssignment)
    }

    pub fn observed(episode: GroundedTransformationEpisode) -> Self {
        Self::new(episode, InterventionEvidenceKind::PassiveObservation)
    }

    pub fn episode(&self) -> &GroundedTransformationEpisode {
        &self.episode
    }

    pub fn evidence_kind(&self) -> InterventionEvidenceKind {
        self.evidence_kind
    }

    pub fn is_controlled(&self) -> bool {
        self.evidence_kind == InterventionEvidenceKind::ControlledAssignment
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InterventionalCausalThresholds {
    minimum_matched_intervention_states: usize,
    minimum_target_interventions: u64,
    minimum_contrast_interventions: u64,
    minimum_interventional_lift: CognitiveSignal,
    minimum_validated_confidence: CognitiveSignal,
}

impl InterventionalCausalThresholds {
    pub fn new(
        minimum_matched_intervention_states: usize,
        minimum_target_interventions: u64,
        minimum_contrast_interventions: u64,
        minimum_interventional_lift: CognitiveSignal,
        minimum_validated_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_matched_intervention_states == 0
            || minimum_target_interventions == 0
            || minimum_contrast_interventions == 0
            || minimum_interventional_lift == CognitiveSignal::zero()
            || minimum_validated_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_matched_intervention_states,
            minimum_target_interventions,
            minimum_contrast_interventions,
            minimum_interventional_lift,
            minimum_validated_confidence,
        })
    }

    pub fn minimum_matched_intervention_states(self) -> usize {
        self.minimum_matched_intervention_states
    }

    pub fn minimum_target_interventions(self) -> u64 {
        self.minimum_target_interventions
    }

    pub fn minimum_contrast_interventions(self) -> u64 {
        self.minimum_contrast_interventions
    }

    pub fn minimum_interventional_lift(self) -> CognitiveSignal {
        self.minimum_interventional_lift
    }

    pub fn minimum_validated_confidence(self) -> CognitiveSignal {
        self.minimum_validated_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InterventionalCausalValidationPolicy {
    max_seed_contrasts: usize,
    max_evaluations: usize,
    max_validated_hypotheses: usize,
    full_confidence_interventions: u64,
    thresholds: InterventionalCausalThresholds,
}

impl InterventionalCausalValidationPolicy {
    pub fn new(
        max_seed_contrasts: usize,
        max_evaluations: usize,
        max_validated_hypotheses: usize,
        full_confidence_interventions: u64,
        thresholds: InterventionalCausalThresholds,
    ) -> Option<Self> {
        if max_seed_contrasts == 0
            || max_evaluations == 0
            || max_validated_hypotheses == 0
            || full_confidence_interventions == 0
        {
            return None;
        }

        Some(Self {
            max_seed_contrasts,
            max_evaluations,
            max_validated_hypotheses,
            full_confidence_interventions,
            thresholds,
        })
    }

    pub fn max_seed_contrasts(self) -> usize {
        self.max_seed_contrasts
    }

    pub fn max_evaluations(self) -> usize {
        self.max_evaluations
    }

    pub fn max_validated_hypotheses(self) -> usize {
        self.max_validated_hypotheses
    }

    pub fn full_confidence_interventions(self) -> u64 {
        self.full_confidence_interventions
    }

    pub fn thresholds(self) -> InterventionalCausalThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedInterventionalCausalHypothesis {
    transformation: CognitiveStructure,
    contrast_transformation: CognitiveStructure,
    context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    matched_intervention_state_count: usize,
    target_intervention_opportunity_count: u64,
    target_intervention_success_count: u64,
    target_intervention_failure_count: u64,
    contrast_intervention_opportunity_count: u64,
    contrast_intervention_success_count: u64,
    contrast_intervention_failure_count: u64,
    target_intervention_rate: CognitiveSignal,
    contrast_intervention_rate: CognitiveSignal,
    interventional_lift: CognitiveSignal,
    balanced_intervention_support: u64,
    intervention_support_adequacy: CognitiveSignal,
    source_contrast_confidence: CognitiveSignal,
    validated_causal_confidence: CognitiveSignal,
    passive_corroborating_count: u64,
    passive_counterevidence_count: u64,
}

impl GroundedInterventionalCausalHypothesis {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn contrast_transformation(&self) -> &CognitiveStructure {
        &self.contrast_transformation
    }

    pub fn context(&self) -> &ContextPremiseSet {
        &self.context
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn effect_fact(&self) -> &CognitiveStructure {
        &self.effect_fact
    }

    pub fn matched_intervention_state_count(&self) -> usize {
        self.matched_intervention_state_count
    }

    pub fn target_intervention_opportunity_count(&self) -> u64 {
        self.target_intervention_opportunity_count
    }

    pub fn target_intervention_success_count(&self) -> u64 {
        self.target_intervention_success_count
    }

    pub fn target_intervention_failure_count(&self) -> u64 {
        self.target_intervention_failure_count
    }

    pub fn contrast_intervention_opportunity_count(&self) -> u64 {
        self.contrast_intervention_opportunity_count
    }

    pub fn contrast_intervention_success_count(&self) -> u64 {
        self.contrast_intervention_success_count
    }

    pub fn contrast_intervention_failure_count(&self) -> u64 {
        self.contrast_intervention_failure_count
    }

    pub fn target_intervention_rate(&self) -> CognitiveSignal {
        self.target_intervention_rate
    }

    pub fn contrast_intervention_rate(&self) -> CognitiveSignal {
        self.contrast_intervention_rate
    }

    pub fn interventional_lift(&self) -> CognitiveSignal {
        self.interventional_lift
    }

    pub fn balanced_intervention_support(&self) -> u64 {
        self.balanced_intervention_support
    }

    pub fn intervention_support_adequacy(&self) -> CognitiveSignal {
        self.intervention_support_adequacy
    }

    pub fn source_contrast_confidence(&self) -> CognitiveSignal {
        self.source_contrast_confidence
    }

    pub fn validated_causal_confidence(&self) -> CognitiveSignal {
        self.validated_causal_confidence
    }

    pub fn passive_corroborating_count(&self) -> u64 {
        self.passive_corroborating_count
    }

    pub fn passive_counterevidence_count(&self) -> u64 {
        self.passive_counterevidence_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterventionalCausalValidationResult {
    input_seed_count: usize,
    considered_seed_count: usize,
    seed_truncated: bool,
    evaluated_seed_count: usize,
    evaluation_truncated: bool,
    rejected_without_matched_interventions: usize,
    rejected_below_interventional_threshold: usize,
    admitted_before_frontier: usize,
    selected: Vec<GroundedInterventionalCausalHypothesis>,
}

impl InterventionalCausalValidationResult {
    pub fn input_seed_count(&self) -> usize {
        self.input_seed_count
    }

    pub fn considered_seed_count(&self) -> usize {
        self.considered_seed_count
    }

    pub fn seed_truncated(&self) -> bool {
        self.seed_truncated
    }

    pub fn evaluated_seed_count(&self) -> usize {
        self.evaluated_seed_count
    }

    pub fn evaluation_truncated(&self) -> bool {
        self.evaluation_truncated
    }

    pub fn rejected_without_matched_interventions(&self) -> usize {
        self.rejected_without_matched_interventions
    }

    pub fn rejected_below_interventional_threshold(&self) -> usize {
        self.rejected_below_interventional_threshold
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedInterventionalCausalHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct InterventionalCausalValidation;

impl InterventionalCausalValidation {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16)
            .expect("bounded interventional rate remains on signal scale")
    }

    fn positive_difference(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        CognitiveSignal::new(left.value().saturating_sub(right.value()))
            .expect("bounded interventional lift remains on signal scale")
    }

    fn scaled_product(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        let scaled = (u32::from(left.value()) * u32::from(right.value())) / 1000;

        CognitiveSignal::new(scaled as u16)
            .expect("bounded interventional confidence remains on signal scale")
    }

    fn support_adequacy(support: u64, full_support: u64) -> CognitiveSignal {
        if support >= full_support {
            return CognitiveSignal::new(1000)
                .expect("full intervention support remains on signal scale");
        }

        Self::scaled_rate(support, full_support)
    }

    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn compare_seed(
        left: &GroundedCausalContrastHypothesis,
        right: &GroundedCausalContrastHypothesis,
    ) -> std::cmp::Ordering {
        right
            .contrast_confidence()
            .value()
            .cmp(&left.contrast_confidence().value())
            .then_with(|| {
                right
                    .contrast_lift()
                    .value()
                    .cmp(&left.contrast_lift().value())
            })
            .then_with(|| right.matched_state_count().cmp(&left.matched_state_count()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.contrast_transformation(),
                    right.contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    fn considered_seeds(
        seeds: &[GroundedCausalContrastHypothesis],
        policy: InterventionalCausalValidationPolicy,
    ) -> Vec<&GroundedCausalContrastHypothesis> {
        let mut considered = seeds.iter().collect::<Vec<_>>();

        considered.sort_by(|left, right| Self::compare_seed(left, right));

        considered.truncate(policy.max_seed_contrasts());

        considered
    }

    fn controlled_episode_matches(
        seed: &GroundedCausalContrastHypothesis,
        transformation: &CognitiveStructure,
        evidence: &InterventionalTransformationEpisode,
    ) -> bool {
        let episode = evidence.episode();

        evidence.is_controlled()
            && episode.transformation() == transformation
            && seed.context().is_satisfied_by(episode.before())
            && episode.effect_opportunity(seed.effect_kind(), seed.effect_fact())
    }

    fn matched_intervention_states(
        evidence: &[InterventionalTransformationEpisode],
        seed: &GroundedCausalContrastHypothesis,
    ) -> Vec<GroundedStateSnapshot> {
        let mut states = Vec::new();

        for target_evidence in evidence {
            if !Self::controlled_episode_matches(seed, seed.transformation(), target_evidence) {
                continue;
            }

            let target_state = target_evidence.episode().before();

            let contrast_exists = evidence.iter().any(|contrast_evidence| {
                contrast_evidence.episode().before() == target_state
                    && Self::controlled_episode_matches(
                        seed,
                        seed.contrast_transformation(),
                        contrast_evidence,
                    )
            });

            if contrast_exists && !states.contains(target_state) {
                states.push(target_state.clone());
            }
        }

        states
    }

    fn in_matched_states(
        evidence: &InterventionalTransformationEpisode,
        states: &[GroundedStateSnapshot],
    ) -> bool {
        states.contains(evidence.episode().before())
    }

    fn passive_counts(
        evidence: &[InterventionalTransformationEpisode],
        seed: &GroundedCausalContrastHypothesis,
    ) -> (u64, u64) {
        let mut corroborating = 0_u64;

        let mut counterevidence = 0_u64;

        for item in evidence {
            if item.is_controlled() {
                continue;
            }

            let episode = item.episode();

            if !seed.context().is_satisfied_by(episode.before())
                || !episode.effect_opportunity(seed.effect_kind(), seed.effect_fact())
            {
                continue;
            }

            if episode.transformation() == seed.transformation() {
                if episode.effect_occurs(seed.effect_kind(), seed.effect_fact()) {
                    corroborating = corroborating.saturating_add(1);
                } else {
                    counterevidence = counterevidence.saturating_add(1);
                }
            } else if episode.transformation() == seed.contrast_transformation() {
                if episode.effect_occurs(seed.effect_kind(), seed.effect_fact()) {
                    counterevidence = counterevidence.saturating_add(1);
                } else {
                    corroborating = corroborating.saturating_add(1);
                }
            }
        }

        (corroborating, counterevidence)
    }

    fn evaluate_seed(
        evidence: &[InterventionalTransformationEpisode],
        seed: &GroundedCausalContrastHypothesis,
        policy: InterventionalCausalValidationPolicy,
    ) -> Option<GroundedInterventionalCausalHypothesis> {
        let matched_states = Self::matched_intervention_states(evidence, seed);

        if matched_states.len() < policy.thresholds().minimum_matched_intervention_states() {
            return None;
        }

        let target_intervention_opportunity_count = evidence
            .iter()
            .filter(|item| {
                Self::controlled_episode_matches(seed, seed.transformation(), item)
                    && Self::in_matched_states(item, &matched_states)
            })
            .count() as u64;

        let contrast_intervention_opportunity_count = evidence
            .iter()
            .filter(|item| {
                Self::controlled_episode_matches(seed, seed.contrast_transformation(), item)
                    && Self::in_matched_states(item, &matched_states)
            })
            .count() as u64;

        if target_intervention_opportunity_count
            < policy.thresholds().minimum_target_interventions()
            || contrast_intervention_opportunity_count
                < policy.thresholds().minimum_contrast_interventions()
        {
            return None;
        }

        let target_intervention_success_count = evidence
            .iter()
            .filter(|item| {
                Self::controlled_episode_matches(seed, seed.transformation(), item)
                    && Self::in_matched_states(item, &matched_states)
                    && item
                        .episode()
                        .effect_occurs(seed.effect_kind(), seed.effect_fact())
            })
            .count() as u64;

        let contrast_intervention_success_count = evidence
            .iter()
            .filter(|item| {
                Self::controlled_episode_matches(seed, seed.contrast_transformation(), item)
                    && Self::in_matched_states(item, &matched_states)
                    && item
                        .episode()
                        .effect_occurs(seed.effect_kind(), seed.effect_fact())
            })
            .count() as u64;

        let target_intervention_failure_count =
            target_intervention_opportunity_count.saturating_sub(target_intervention_success_count);

        let contrast_intervention_failure_count = contrast_intervention_opportunity_count
            .saturating_sub(contrast_intervention_success_count);

        let target_intervention_rate = Self::scaled_rate(
            target_intervention_success_count,
            target_intervention_opportunity_count,
        );

        let contrast_intervention_rate = Self::scaled_rate(
            contrast_intervention_success_count,
            contrast_intervention_opportunity_count,
        );

        let interventional_lift =
            Self::positive_difference(target_intervention_rate, contrast_intervention_rate);

        if interventional_lift.value() < policy.thresholds().minimum_interventional_lift().value() {
            return None;
        }

        let balanced_intervention_support =
            target_intervention_opportunity_count.min(contrast_intervention_opportunity_count);

        let intervention_support_adequacy = Self::support_adequacy(
            balanced_intervention_support,
            policy.full_confidence_interventions(),
        );

        let lift_with_support =
            Self::scaled_product(interventional_lift, intervention_support_adequacy);

        let validated_causal_confidence =
            Self::scaled_product(lift_with_support, seed.contrast_confidence());

        if validated_causal_confidence.value()
            < policy.thresholds().minimum_validated_confidence().value()
        {
            return None;
        }

        let (passive_corroborating_count, passive_counterevidence_count) =
            Self::passive_counts(evidence, seed);

        Some(GroundedInterventionalCausalHypothesis {
            transformation: seed.transformation().clone(),
            contrast_transformation: seed.contrast_transformation().clone(),
            context: seed.context().clone(),
            effect_kind: seed.effect_kind(),
            effect_fact: seed.effect_fact().clone(),
            matched_intervention_state_count: matched_states.len(),
            target_intervention_opportunity_count,
            target_intervention_success_count,
            target_intervention_failure_count,
            contrast_intervention_opportunity_count,
            contrast_intervention_success_count,
            contrast_intervention_failure_count,
            target_intervention_rate,
            contrast_intervention_rate,
            interventional_lift,
            balanced_intervention_support,
            intervention_support_adequacy,
            source_contrast_confidence: seed.contrast_confidence(),
            validated_causal_confidence,
            passive_corroborating_count,
            passive_counterevidence_count,
        })
    }

    fn ranking(
        left: &GroundedInterventionalCausalHypothesis,
        right: &GroundedInterventionalCausalHypothesis,
    ) -> std::cmp::Ordering {
        right
            .validated_causal_confidence()
            .value()
            .cmp(&left.validated_causal_confidence().value())
            .then_with(|| {
                right
                    .interventional_lift()
                    .value()
                    .cmp(&left.interventional_lift().value())
            })
            .then_with(|| {
                right
                    .intervention_support_adequacy()
                    .value()
                    .cmp(&left.intervention_support_adequacy().value())
            })
            .then_with(|| {
                right
                    .matched_intervention_state_count()
                    .cmp(&left.matched_intervention_state_count())
            })
            .then_with(|| {
                left.target_intervention_failure_count()
                    .cmp(&right.target_intervention_failure_count())
            })
            .then_with(|| {
                left.contrast_intervention_success_count()
                    .cmp(&right.contrast_intervention_success_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.contrast_transformation(),
                    right.contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    pub fn validate(
        evidence: &[InterventionalTransformationEpisode],
        seeds: &[GroundedCausalContrastHypothesis],
        policy: InterventionalCausalValidationPolicy,
    ) -> InterventionalCausalValidationResult {
        if evidence.is_empty() || seeds.is_empty() {
            return InterventionalCausalValidationResult {
                input_seed_count: seeds.len(),
                considered_seed_count: 0,
                seed_truncated: false,
                evaluated_seed_count: 0,
                evaluation_truncated: false,
                rejected_without_matched_interventions: 0,
                rejected_below_interventional_threshold: 0,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let considered = Self::considered_seeds(seeds, policy);

        let evaluated_seed_count = considered.len().min(policy.max_evaluations());

        let mut rejected_without_matched_interventions = 0_usize;

        let mut rejected_below_interventional_threshold = 0_usize;

        let mut admitted = Vec::new();

        for seed in considered.iter().take(policy.max_evaluations()) {
            let matched_states = Self::matched_intervention_states(evidence, seed);

            if matched_states.len() < policy.thresholds().minimum_matched_intervention_states() {
                rejected_without_matched_interventions =
                    rejected_without_matched_interventions.saturating_add(1);

                continue;
            }

            if let Some(hypothesis) = Self::evaluate_seed(evidence, seed, policy) {
                admitted.push(hypothesis);
            } else {
                rejected_below_interventional_threshold =
                    rejected_below_interventional_threshold.saturating_add(1);
            }
        }

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_validated_hypotheses());

        InterventionalCausalValidationResult {
            input_seed_count: seeds.len(),
            considered_seed_count: considered.len(),
            seed_truncated: seeds.len() > considered.len(),
            evaluated_seed_count,
            evaluation_truncated: considered.len() > evaluated_seed_count,
            rejected_without_matched_interventions,
            rejected_below_interventional_threshold,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalInterventionalCausalValidation;

impl UniversalInterventionalCausalValidation {
    pub fn evaluate(
        evidence: &[InterventionalTransformationEpisode],
        seeds: &[GroundedCausalContrastHypothesis],
        policy: InterventionalCausalValidationPolicy,
    ) -> InterventionalCausalValidationResult {
        InterventionalCausalValidation::validate(evidence, seeds, policy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedTransferCorrespondence {
    source: CognitiveStructure,
    target: CognitiveStructure,
}

impl GroundedTransferCorrespondence {
    pub fn new(source: CognitiveStructure, target: CognitiveStructure) -> Self {
        Self { source, target }
    }

    pub fn source(&self) -> &CognitiveStructure {
        &self.source
    }

    pub fn target(&self) -> &CognitiveStructure {
        &self.target
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossDomainTransferMap {
    source_domain: CognitiveStructure,
    target_domain: CognitiveStructure,
    correspondences: Vec<GroundedTransferCorrespondence>,
}

impl CrossDomainTransferMap {
    pub fn new(
        source_domain: CognitiveStructure,
        target_domain: CognitiveStructure,
        mut correspondences: Vec<GroundedTransferCorrespondence>,
    ) -> Option<Self> {
        if source_domain == target_domain || correspondences.is_empty() {
            return None;
        }

        correspondences.sort_by(|left, right| {
            PredicateDiscovery::compare_structure(left.source(), right.source())
                .then_with(|| PredicateDiscovery::compare_structure(left.target(), right.target()))
        });

        correspondences.dedup();

        let duplicate_source = correspondences
            .windows(2)
            .any(|window| window[0].source() == window[1].source());

        if duplicate_source {
            return None;
        }

        let mut by_target = correspondences.iter().collect::<Vec<_>>();

        by_target.sort_by(|left, right| {
            PredicateDiscovery::compare_structure(left.target(), right.target())
                .then_with(|| PredicateDiscovery::compare_structure(left.source(), right.source()))
        });

        let duplicate_target = by_target
            .windows(2)
            .any(|window| window[0].target() == window[1].target());

        if duplicate_target {
            return None;
        }

        Some(Self {
            source_domain,
            target_domain,
            correspondences,
        })
    }

    pub fn source_domain(&self) -> &CognitiveStructure {
        &self.source_domain
    }

    pub fn target_domain(&self) -> &CognitiveStructure {
        &self.target_domain
    }

    pub fn correspondences(&self) -> &[GroundedTransferCorrespondence] {
        &self.correspondences
    }

    pub fn correspondence_count(&self) -> usize {
        self.correspondences.len()
    }

    pub fn translate(&self, source: &CognitiveStructure) -> Option<&CognitiveStructure> {
        self.correspondences
            .iter()
            .find(|correspondence| correspondence.source() == source)
            .map(GroundedTransferCorrespondence::target)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CrossDomainTransferThresholds {
    minimum_matched_target_states: usize,
    minimum_target_interventions: u64,
    minimum_contrast_interventions: u64,
    minimum_target_lift: CognitiveSignal,
    minimum_transfer_confidence: CognitiveSignal,
}

impl CrossDomainTransferThresholds {
    pub fn new(
        minimum_matched_target_states: usize,
        minimum_target_interventions: u64,
        minimum_contrast_interventions: u64,
        minimum_target_lift: CognitiveSignal,
        minimum_transfer_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_matched_target_states == 0
            || minimum_target_interventions == 0
            || minimum_contrast_interventions == 0
            || minimum_target_lift == CognitiveSignal::zero()
            || minimum_transfer_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_matched_target_states,
            minimum_target_interventions,
            minimum_contrast_interventions,
            minimum_target_lift,
            minimum_transfer_confidence,
        })
    }

    pub fn minimum_matched_target_states(self) -> usize {
        self.minimum_matched_target_states
    }

    pub fn minimum_target_interventions(self) -> u64 {
        self.minimum_target_interventions
    }

    pub fn minimum_contrast_interventions(self) -> u64 {
        self.minimum_contrast_interventions
    }

    pub fn minimum_target_lift(self) -> CognitiveSignal {
        self.minimum_target_lift
    }

    pub fn minimum_transfer_confidence(self) -> CognitiveSignal {
        self.minimum_transfer_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CrossDomainTransferPolicy {
    max_source_hypotheses: usize,
    max_evaluations: usize,
    max_transferred_hypotheses: usize,
    full_confidence_target_interventions: u64,
    thresholds: CrossDomainTransferThresholds,
}

impl CrossDomainTransferPolicy {
    pub fn new(
        max_source_hypotheses: usize,
        max_evaluations: usize,
        max_transferred_hypotheses: usize,
        full_confidence_target_interventions: u64,
        thresholds: CrossDomainTransferThresholds,
    ) -> Option<Self> {
        if max_source_hypotheses == 0
            || max_evaluations == 0
            || max_transferred_hypotheses == 0
            || full_confidence_target_interventions == 0
        {
            return None;
        }

        Some(Self {
            max_source_hypotheses,
            max_evaluations,
            max_transferred_hypotheses,
            full_confidence_target_interventions,
            thresholds,
        })
    }

    pub fn max_source_hypotheses(self) -> usize {
        self.max_source_hypotheses
    }

    pub fn max_evaluations(self) -> usize {
        self.max_evaluations
    }

    pub fn max_transferred_hypotheses(self) -> usize {
        self.max_transferred_hypotheses
    }

    pub fn full_confidence_target_interventions(self) -> u64 {
        self.full_confidence_target_interventions
    }

    pub fn thresholds(self) -> CrossDomainTransferThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TranslatedCausalSeed {
    source_transformation: CognitiveStructure,
    source_contrast_transformation: CognitiveStructure,
    target_transformation: CognitiveStructure,
    target_contrast_transformation: CognitiveStructure,
    target_context: ContextPremiseSet,
    source_effect_fact: CognitiveStructure,
    target_effect_fact: CognitiveStructure,
    effect_kind: TransitionEffectKind,
    source_validated_confidence: CognitiveSignal,
    source_interventional_lift: CognitiveSignal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedCrossDomainTransferHypothesis {
    source_domain: CognitiveStructure,
    target_domain: CognitiveStructure,
    source_transformation: CognitiveStructure,
    source_contrast_transformation: CognitiveStructure,
    target_transformation: CognitiveStructure,
    target_contrast_transformation: CognitiveStructure,
    target_context: ContextPremiseSet,
    source_effect_fact: CognitiveStructure,
    target_effect_fact: CognitiveStructure,
    effect_kind: TransitionEffectKind,
    matched_target_state_count: usize,
    target_intervention_opportunity_count: u64,
    target_intervention_success_count: u64,
    target_intervention_failure_count: u64,
    contrast_intervention_opportunity_count: u64,
    contrast_intervention_success_count: u64,
    contrast_intervention_failure_count: u64,
    target_effect_rate: CognitiveSignal,
    target_contrast_effect_rate: CognitiveSignal,
    target_interventional_lift: CognitiveSignal,
    balanced_target_support: u64,
    target_support_adequacy: CognitiveSignal,
    source_validated_confidence: CognitiveSignal,
    source_interventional_lift: CognitiveSignal,
    target_evidence_confidence: CognitiveSignal,
    transfer_confidence: CognitiveSignal,
    passive_corroborating_count: u64,
    passive_counterevidence_count: u64,
}

impl GroundedCrossDomainTransferHypothesis {
    pub fn source_domain(&self) -> &CognitiveStructure {
        &self.source_domain
    }

    pub fn target_domain(&self) -> &CognitiveStructure {
        &self.target_domain
    }

    pub fn source_transformation(&self) -> &CognitiveStructure {
        &self.source_transformation
    }

    pub fn source_contrast_transformation(&self) -> &CognitiveStructure {
        &self.source_contrast_transformation
    }

    pub fn target_transformation(&self) -> &CognitiveStructure {
        &self.target_transformation
    }

    pub fn target_contrast_transformation(&self) -> &CognitiveStructure {
        &self.target_contrast_transformation
    }

    pub fn target_context(&self) -> &ContextPremiseSet {
        &self.target_context
    }

    pub fn source_effect_fact(&self) -> &CognitiveStructure {
        &self.source_effect_fact
    }

    pub fn target_effect_fact(&self) -> &CognitiveStructure {
        &self.target_effect_fact
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn matched_target_state_count(&self) -> usize {
        self.matched_target_state_count
    }

    pub fn target_intervention_opportunity_count(&self) -> u64 {
        self.target_intervention_opportunity_count
    }

    pub fn target_intervention_success_count(&self) -> u64 {
        self.target_intervention_success_count
    }

    pub fn target_intervention_failure_count(&self) -> u64 {
        self.target_intervention_failure_count
    }

    pub fn contrast_intervention_opportunity_count(&self) -> u64 {
        self.contrast_intervention_opportunity_count
    }

    pub fn contrast_intervention_success_count(&self) -> u64 {
        self.contrast_intervention_success_count
    }

    pub fn contrast_intervention_failure_count(&self) -> u64 {
        self.contrast_intervention_failure_count
    }

    pub fn target_effect_rate(&self) -> CognitiveSignal {
        self.target_effect_rate
    }

    pub fn target_contrast_effect_rate(&self) -> CognitiveSignal {
        self.target_contrast_effect_rate
    }

    pub fn target_interventional_lift(&self) -> CognitiveSignal {
        self.target_interventional_lift
    }

    pub fn balanced_target_support(&self) -> u64 {
        self.balanced_target_support
    }

    pub fn target_support_adequacy(&self) -> CognitiveSignal {
        self.target_support_adequacy
    }

    pub fn source_validated_confidence(&self) -> CognitiveSignal {
        self.source_validated_confidence
    }

    pub fn source_interventional_lift(&self) -> CognitiveSignal {
        self.source_interventional_lift
    }

    pub fn target_evidence_confidence(&self) -> CognitiveSignal {
        self.target_evidence_confidence
    }

    pub fn transfer_confidence(&self) -> CognitiveSignal {
        self.transfer_confidence
    }

    pub fn passive_corroborating_count(&self) -> u64 {
        self.passive_corroborating_count
    }

    pub fn passive_counterevidence_count(&self) -> u64 {
        self.passive_counterevidence_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossDomainTransferResult {
    input_source_hypothesis_count: usize,
    considered_source_hypothesis_count: usize,
    source_frontier_truncated: bool,
    rejected_incomplete_mapping: usize,
    evaluated_candidate_count: usize,
    evaluation_truncated: bool,
    rejected_without_matched_target_interventions: usize,
    rejected_below_transfer_threshold: usize,
    admitted_before_frontier: usize,
    selected: Vec<GroundedCrossDomainTransferHypothesis>,
}

impl CrossDomainTransferResult {
    pub fn input_source_hypothesis_count(&self) -> usize {
        self.input_source_hypothesis_count
    }

    pub fn considered_source_hypothesis_count(&self) -> usize {
        self.considered_source_hypothesis_count
    }

    pub fn source_frontier_truncated(&self) -> bool {
        self.source_frontier_truncated
    }

    pub fn rejected_incomplete_mapping(&self) -> usize {
        self.rejected_incomplete_mapping
    }

    pub fn evaluated_candidate_count(&self) -> usize {
        self.evaluated_candidate_count
    }

    pub fn evaluation_truncated(&self) -> bool {
        self.evaluation_truncated
    }

    pub fn rejected_without_matched_target_interventions(&self) -> usize {
        self.rejected_without_matched_target_interventions
    }

    pub fn rejected_below_transfer_threshold(&self) -> usize {
        self.rejected_below_transfer_threshold
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedCrossDomainTransferHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CrossDomainTransfer;

impl CrossDomainTransfer {
    fn scaled_rate(numerator: u64, denominator: u64) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = (u128::from(numerator) * 1000) / u128::from(denominator);

        CognitiveSignal::new(scaled as u16).expect("bounded transfer rate remains on signal scale")
    }

    fn positive_difference(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        CognitiveSignal::new(left.value().saturating_sub(right.value()))
            .expect("bounded transfer lift remains on signal scale")
    }

    fn scaled_product(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        let scaled = (u32::from(left.value()) * u32::from(right.value())) / 1000;

        CognitiveSignal::new(scaled as u16)
            .expect("bounded transfer confidence remains on signal scale")
    }

    fn support_adequacy(support: u64, full_support: u64) -> CognitiveSignal {
        if support >= full_support {
            return CognitiveSignal::new(1000)
                .expect("full transfer support remains on signal scale");
        }

        Self::scaled_rate(support, full_support)
    }

    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn compare_source(
        left: &GroundedInterventionalCausalHypothesis,
        right: &GroundedInterventionalCausalHypothesis,
    ) -> std::cmp::Ordering {
        right
            .validated_causal_confidence()
            .value()
            .cmp(&left.validated_causal_confidence().value())
            .then_with(|| {
                right
                    .interventional_lift()
                    .value()
                    .cmp(&left.interventional_lift().value())
            })
            .then_with(|| {
                right
                    .matched_intervention_state_count()
                    .cmp(&left.matched_intervention_state_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.contrast_transformation(),
                    right.contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    fn considered_sources(
        source_hypotheses: &[GroundedInterventionalCausalHypothesis],
        policy: CrossDomainTransferPolicy,
    ) -> Vec<&GroundedInterventionalCausalHypothesis> {
        let mut considered = source_hypotheses.iter().collect::<Vec<_>>();

        considered.sort_by(|left, right| Self::compare_source(left, right));

        considered.truncate(policy.max_source_hypotheses());

        considered
    }

    fn translate_context(
        context: &ContextPremiseSet,
        transfer_map: &CrossDomainTransferMap,
    ) -> Option<ContextPremiseSet> {
        let translated = context
            .premises()
            .iter()
            .map(|premise| transfer_map.translate(premise).cloned())
            .collect::<Option<Vec<_>>>()?;

        ContextPremiseSet::new(translated)
    }

    fn translate_seed(
        source: &GroundedInterventionalCausalHypothesis,
        transfer_map: &CrossDomainTransferMap,
    ) -> Option<TranslatedCausalSeed> {
        let target_transformation = transfer_map.translate(source.transformation())?.clone();

        let target_contrast_transformation = transfer_map
            .translate(source.contrast_transformation())?
            .clone();

        let target_effect_fact = transfer_map.translate(source.effect_fact())?.clone();

        let target_context = Self::translate_context(source.context(), transfer_map)?;

        Some(TranslatedCausalSeed {
            source_transformation: source.transformation().clone(),
            source_contrast_transformation: source.contrast_transformation().clone(),
            target_transformation,
            target_contrast_transformation,
            target_context,
            source_effect_fact: source.effect_fact().clone(),
            target_effect_fact,
            effect_kind: source.effect_kind(),
            source_validated_confidence: source.validated_causal_confidence(),
            source_interventional_lift: source.interventional_lift(),
        })
    }

    fn controlled_matches(
        candidate: &TranslatedCausalSeed,
        transformation: &CognitiveStructure,
        evidence: &InterventionalTransformationEpisode,
    ) -> bool {
        let episode = evidence.episode();

        evidence.is_controlled()
            && episode.transformation() == transformation
            && candidate.target_context.is_satisfied_by(episode.before())
            && episode.effect_opportunity(candidate.effect_kind, &candidate.target_effect_fact)
    }

    fn matched_target_states(
        target_evidence: &[InterventionalTransformationEpisode],
        candidate: &TranslatedCausalSeed,
    ) -> Vec<GroundedStateSnapshot> {
        let mut states = Vec::new();

        for target_item in target_evidence {
            if !Self::controlled_matches(candidate, &candidate.target_transformation, target_item) {
                continue;
            }

            let state = target_item.episode().before();

            let has_contrast = target_evidence.iter().any(|contrast_item| {
                contrast_item.episode().before() == state
                    && Self::controlled_matches(
                        candidate,
                        &candidate.target_contrast_transformation,
                        contrast_item,
                    )
            });

            if has_contrast && !states.contains(state) {
                states.push(state.clone());
            }
        }

        states
    }

    fn in_matched_states(
        evidence: &InterventionalTransformationEpisode,
        states: &[GroundedStateSnapshot],
    ) -> bool {
        states.contains(evidence.episode().before())
    }

    fn passive_counts(
        target_evidence: &[InterventionalTransformationEpisode],
        candidate: &TranslatedCausalSeed,
    ) -> (u64, u64) {
        let mut corroborating = 0_u64;

        let mut counterevidence = 0_u64;

        for item in target_evidence {
            if item.is_controlled() {
                continue;
            }

            let episode = item.episode();

            if !candidate.target_context.is_satisfied_by(episode.before())
                || !episode.effect_opportunity(candidate.effect_kind, &candidate.target_effect_fact)
            {
                continue;
            }

            if episode.transformation() == &candidate.target_transformation {
                if episode.effect_occurs(candidate.effect_kind, &candidate.target_effect_fact) {
                    corroborating = corroborating.saturating_add(1);
                } else {
                    counterevidence = counterevidence.saturating_add(1);
                }
            } else if episode.transformation() == &candidate.target_contrast_transformation {
                if episode.effect_occurs(candidate.effect_kind, &candidate.target_effect_fact) {
                    counterevidence = counterevidence.saturating_add(1);
                } else {
                    corroborating = corroborating.saturating_add(1);
                }
            }
        }

        (corroborating, counterevidence)
    }

    fn evaluate_candidate(
        target_evidence: &[InterventionalTransformationEpisode],
        candidate: &TranslatedCausalSeed,
        transfer_map: &CrossDomainTransferMap,
        policy: CrossDomainTransferPolicy,
    ) -> Option<GroundedCrossDomainTransferHypothesis> {
        let matched_states = Self::matched_target_states(target_evidence, candidate);

        if matched_states.len() < policy.thresholds().minimum_matched_target_states() {
            return None;
        }

        let target_intervention_opportunity_count = target_evidence
            .iter()
            .filter(|item| {
                Self::controlled_matches(candidate, &candidate.target_transformation, item)
                    && Self::in_matched_states(item, &matched_states)
            })
            .count() as u64;

        let contrast_intervention_opportunity_count = target_evidence
            .iter()
            .filter(|item| {
                Self::controlled_matches(candidate, &candidate.target_contrast_transformation, item)
                    && Self::in_matched_states(item, &matched_states)
            })
            .count() as u64;

        if target_intervention_opportunity_count
            < policy.thresholds().minimum_target_interventions()
            || contrast_intervention_opportunity_count
                < policy.thresholds().minimum_contrast_interventions()
        {
            return None;
        }

        let target_intervention_success_count = target_evidence
            .iter()
            .filter(|item| {
                Self::controlled_matches(candidate, &candidate.target_transformation, item)
                    && Self::in_matched_states(item, &matched_states)
                    && item
                        .episode()
                        .effect_occurs(candidate.effect_kind, &candidate.target_effect_fact)
            })
            .count() as u64;

        let contrast_intervention_success_count = target_evidence
            .iter()
            .filter(|item| {
                Self::controlled_matches(candidate, &candidate.target_contrast_transformation, item)
                    && Self::in_matched_states(item, &matched_states)
                    && item
                        .episode()
                        .effect_occurs(candidate.effect_kind, &candidate.target_effect_fact)
            })
            .count() as u64;

        let target_intervention_failure_count =
            target_intervention_opportunity_count.saturating_sub(target_intervention_success_count);

        let contrast_intervention_failure_count = contrast_intervention_opportunity_count
            .saturating_sub(contrast_intervention_success_count);

        let target_effect_rate = Self::scaled_rate(
            target_intervention_success_count,
            target_intervention_opportunity_count,
        );

        let target_contrast_effect_rate = Self::scaled_rate(
            contrast_intervention_success_count,
            contrast_intervention_opportunity_count,
        );

        let target_interventional_lift =
            Self::positive_difference(target_effect_rate, target_contrast_effect_rate);

        if target_interventional_lift.value() < policy.thresholds().minimum_target_lift().value() {
            return None;
        }

        let balanced_target_support =
            target_intervention_opportunity_count.min(contrast_intervention_opportunity_count);

        let target_support_adequacy = Self::support_adequacy(
            balanced_target_support,
            policy.full_confidence_target_interventions(),
        );

        let target_evidence_confidence =
            Self::scaled_product(target_interventional_lift, target_support_adequacy);

        let transfer_confidence = Self::scaled_product(
            candidate.source_validated_confidence,
            target_evidence_confidence,
        );

        if transfer_confidence.value() < policy.thresholds().minimum_transfer_confidence().value() {
            return None;
        }

        let (passive_corroborating_count, passive_counterevidence_count) =
            Self::passive_counts(target_evidence, candidate);

        Some(GroundedCrossDomainTransferHypothesis {
            source_domain: transfer_map.source_domain().clone(),
            target_domain: transfer_map.target_domain().clone(),
            source_transformation: candidate.source_transformation.clone(),
            source_contrast_transformation: candidate.source_contrast_transformation.clone(),
            target_transformation: candidate.target_transformation.clone(),
            target_contrast_transformation: candidate.target_contrast_transformation.clone(),
            target_context: candidate.target_context.clone(),
            source_effect_fact: candidate.source_effect_fact.clone(),
            target_effect_fact: candidate.target_effect_fact.clone(),
            effect_kind: candidate.effect_kind,
            matched_target_state_count: matched_states.len(),
            target_intervention_opportunity_count,
            target_intervention_success_count,
            target_intervention_failure_count,
            contrast_intervention_opportunity_count,
            contrast_intervention_success_count,
            contrast_intervention_failure_count,
            target_effect_rate,
            target_contrast_effect_rate,
            target_interventional_lift,
            balanced_target_support,
            target_support_adequacy,
            source_validated_confidence: candidate.source_validated_confidence,
            source_interventional_lift: candidate.source_interventional_lift,
            target_evidence_confidence,
            transfer_confidence,
            passive_corroborating_count,
            passive_counterevidence_count,
        })
    }

    fn ranking(
        left: &GroundedCrossDomainTransferHypothesis,
        right: &GroundedCrossDomainTransferHypothesis,
    ) -> std::cmp::Ordering {
        right
            .transfer_confidence()
            .value()
            .cmp(&left.transfer_confidence().value())
            .then_with(|| {
                right
                    .target_interventional_lift()
                    .value()
                    .cmp(&left.target_interventional_lift().value())
            })
            .then_with(|| {
                right
                    .target_support_adequacy()
                    .value()
                    .cmp(&left.target_support_adequacy().value())
            })
            .then_with(|| {
                right
                    .source_validated_confidence()
                    .value()
                    .cmp(&left.source_validated_confidence().value())
            })
            .then_with(|| {
                right
                    .matched_target_state_count()
                    .cmp(&left.matched_target_state_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_transformation(),
                    right.target_transformation(),
                )
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_contrast_transformation(),
                    right.target_contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_effect_fact(),
                    right.target_effect_fact(),
                )
            })
            .then_with(|| Self::compare_context(left.target_context(), right.target_context()))
    }

    pub fn transfer(
        target_evidence: &[InterventionalTransformationEpisode],
        source_hypotheses: &[GroundedInterventionalCausalHypothesis],
        transfer_map: &CrossDomainTransferMap,
        policy: CrossDomainTransferPolicy,
    ) -> CrossDomainTransferResult {
        if target_evidence.is_empty() || source_hypotheses.is_empty() {
            return CrossDomainTransferResult {
                input_source_hypothesis_count: source_hypotheses.len(),
                considered_source_hypothesis_count: 0,
                source_frontier_truncated: false,
                rejected_incomplete_mapping: 0,
                evaluated_candidate_count: 0,
                evaluation_truncated: false,
                rejected_without_matched_target_interventions: 0,
                rejected_below_transfer_threshold: 0,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let considered = Self::considered_sources(source_hypotheses, policy);

        let mut rejected_incomplete_mapping = 0_usize;

        let mut translated = Vec::new();

        for source in &considered {
            if let Some(candidate) = Self::translate_seed(source, transfer_map) {
                translated.push(candidate);
            } else {
                rejected_incomplete_mapping = rejected_incomplete_mapping.saturating_add(1);
            }
        }

        let evaluated_candidate_count = translated.len().min(policy.max_evaluations());

        let mut rejected_without_matched_target_interventions = 0_usize;

        let mut rejected_below_transfer_threshold = 0_usize;

        let mut admitted = Vec::new();

        for candidate in translated.iter().take(policy.max_evaluations()) {
            let matched_states = Self::matched_target_states(target_evidence, candidate);

            if matched_states.len() < policy.thresholds().minimum_matched_target_states() {
                rejected_without_matched_target_interventions =
                    rejected_without_matched_target_interventions.saturating_add(1);

                continue;
            }

            if let Some(hypothesis) =
                Self::evaluate_candidate(target_evidence, candidate, transfer_map, policy)
            {
                admitted.push(hypothesis);
            } else {
                rejected_below_transfer_threshold =
                    rejected_below_transfer_threshold.saturating_add(1);
            }
        }

        admitted.sort_by(Self::ranking);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_transferred_hypotheses());

        CrossDomainTransferResult {
            input_source_hypothesis_count: source_hypotheses.len(),
            considered_source_hypothesis_count: considered.len(),
            source_frontier_truncated: source_hypotheses.len() > considered.len(),
            rejected_incomplete_mapping,
            evaluated_candidate_count,
            evaluation_truncated: translated.len() > evaluated_candidate_count,
            rejected_without_matched_target_interventions,
            rejected_below_transfer_threshold,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalCrossDomainTransfer;

impl UniversalCrossDomainTransfer {
    pub fn evaluate(
        target_evidence: &[InterventionalTransformationEpisode],
        source_hypotheses: &[GroundedInterventionalCausalHypothesis],
        transfer_map: &CrossDomainTransferMap,
        policy: CrossDomainTransferPolicy,
    ) -> CrossDomainTransferResult {
        CrossDomainTransfer::transfer(target_evidence, source_hypotheses, transfer_map, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomainModelCompressionPolicy {
    max_input_hypotheses: usize,
    max_model_groups: usize,
    max_output_models: usize,
}

impl DomainModelCompressionPolicy {
    pub fn new(
        max_input_hypotheses: usize,
        max_model_groups: usize,
        max_output_models: usize,
    ) -> Option<Self> {
        if max_input_hypotheses == 0 || max_model_groups == 0 || max_output_models == 0 {
            return None;
        }

        Some(Self {
            max_input_hypotheses,
            max_model_groups,
            max_output_models,
        })
    }

    pub fn max_input_hypotheses(self) -> usize {
        self.max_input_hypotheses
    }

    pub fn max_model_groups(self) -> usize {
        self.max_model_groups
    }

    pub fn max_output_models(self) -> usize {
        self.max_output_models
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompressedTransferProvenance {
    source_domain: CognitiveStructure,
    source_transformation: CognitiveStructure,
    source_contrast_transformation: CognitiveStructure,
    source_effect_fact: CognitiveStructure,
    source_validated_confidence: CognitiveSignal,
    source_interventional_lift: CognitiveSignal,
    transfer_confidence: CognitiveSignal,
}

impl CompressedTransferProvenance {
    pub fn source_domain(&self) -> &CognitiveStructure {
        &self.source_domain
    }

    pub fn source_transformation(&self) -> &CognitiveStructure {
        &self.source_transformation
    }

    pub fn source_contrast_transformation(&self) -> &CognitiveStructure {
        &self.source_contrast_transformation
    }

    pub fn source_effect_fact(&self) -> &CognitiveStructure {
        &self.source_effect_fact
    }

    pub fn source_validated_confidence(&self) -> CognitiveSignal {
        self.source_validated_confidence
    }

    pub fn source_interventional_lift(&self) -> CognitiveSignal {
        self.source_interventional_lift
    }

    pub fn transfer_confidence(&self) -> CognitiveSignal {
        self.transfer_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompressedDomainModel {
    target_domain: CognitiveStructure,
    target_transformation: CognitiveStructure,
    target_contrast_transformation: CognitiveStructure,
    target_context: ContextPremiseSet,
    target_effect_fact: CognitiveStructure,
    effect_kind: TransitionEffectKind,
    matched_target_state_count: usize,
    target_intervention_opportunity_count: u64,
    target_intervention_success_count: u64,
    target_intervention_failure_count: u64,
    contrast_intervention_opportunity_count: u64,
    contrast_intervention_success_count: u64,
    contrast_intervention_failure_count: u64,
    target_effect_rate: CognitiveSignal,
    target_contrast_effect_rate: CognitiveSignal,
    target_interventional_lift: CognitiveSignal,
    balanced_target_support: u64,
    target_support_adequacy: CognitiveSignal,
    target_evidence_confidence: CognitiveSignal,
    passive_corroborating_count: u64,
    passive_counterevidence_count: u64,
    member_count: usize,
    provenances: Vec<CompressedTransferProvenance>,
    strongest_transfer_confidence: CognitiveSignal,
    weakest_transfer_confidence: CognitiveSignal,
}

impl CompressedDomainModel {
    pub fn target_domain(&self) -> &CognitiveStructure {
        &self.target_domain
    }

    pub fn target_transformation(&self) -> &CognitiveStructure {
        &self.target_transformation
    }

    pub fn target_contrast_transformation(&self) -> &CognitiveStructure {
        &self.target_contrast_transformation
    }

    pub fn target_context(&self) -> &ContextPremiseSet {
        &self.target_context
    }

    pub fn target_effect_fact(&self) -> &CognitiveStructure {
        &self.target_effect_fact
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn matched_target_state_count(&self) -> usize {
        self.matched_target_state_count
    }

    pub fn target_intervention_opportunity_count(&self) -> u64 {
        self.target_intervention_opportunity_count
    }

    pub fn target_intervention_success_count(&self) -> u64 {
        self.target_intervention_success_count
    }

    pub fn target_intervention_failure_count(&self) -> u64 {
        self.target_intervention_failure_count
    }

    pub fn contrast_intervention_opportunity_count(&self) -> u64 {
        self.contrast_intervention_opportunity_count
    }

    pub fn contrast_intervention_success_count(&self) -> u64 {
        self.contrast_intervention_success_count
    }

    pub fn contrast_intervention_failure_count(&self) -> u64 {
        self.contrast_intervention_failure_count
    }

    pub fn target_effect_rate(&self) -> CognitiveSignal {
        self.target_effect_rate
    }

    pub fn target_contrast_effect_rate(&self) -> CognitiveSignal {
        self.target_contrast_effect_rate
    }

    pub fn target_interventional_lift(&self) -> CognitiveSignal {
        self.target_interventional_lift
    }

    pub fn balanced_target_support(&self) -> u64 {
        self.balanced_target_support
    }

    pub fn target_support_adequacy(&self) -> CognitiveSignal {
        self.target_support_adequacy
    }

    pub fn target_evidence_confidence(&self) -> CognitiveSignal {
        self.target_evidence_confidence
    }

    pub fn passive_corroborating_count(&self) -> u64 {
        self.passive_corroborating_count
    }

    pub fn passive_counterevidence_count(&self) -> u64 {
        self.passive_counterevidence_count
    }

    pub fn member_count(&self) -> usize {
        self.member_count
    }

    pub fn provenances(&self) -> &[CompressedTransferProvenance] {
        &self.provenances
    }

    pub fn provenance_count(&self) -> usize {
        self.provenances.len()
    }

    pub fn strongest_transfer_confidence(&self) -> CognitiveSignal {
        self.strongest_transfer_confidence
    }

    pub fn weakest_transfer_confidence(&self) -> CognitiveSignal {
        self.weakest_transfer_confidence
    }

    pub fn structurally_removed_member_count(&self) -> usize {
        self.member_count.saturating_sub(1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DomainModelCompressionGroup {
    representative: GroundedCrossDomainTransferHypothesis,
    members: Vec<GroundedCrossDomainTransferHypothesis>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainModelCompressionResult {
    input_hypothesis_count: usize,
    considered_hypothesis_count: usize,
    input_frontier_truncated: bool,
    possible_model_group_count: usize,
    generated_model_group_count: usize,
    group_generation_truncated: bool,
    grouped_member_count: usize,
    structurally_removed_member_count: usize,
    compression_gain: CognitiveSignal,
    admitted_before_frontier: usize,
    selected: Vec<CompressedDomainModel>,
}

impl DomainModelCompressionResult {
    pub fn input_hypothesis_count(&self) -> usize {
        self.input_hypothesis_count
    }

    pub fn considered_hypothesis_count(&self) -> usize {
        self.considered_hypothesis_count
    }

    pub fn input_frontier_truncated(&self) -> bool {
        self.input_frontier_truncated
    }

    pub fn possible_model_group_count(&self) -> usize {
        self.possible_model_group_count
    }

    pub fn generated_model_group_count(&self) -> usize {
        self.generated_model_group_count
    }

    pub fn group_generation_truncated(&self) -> bool {
        self.group_generation_truncated
    }

    pub fn grouped_member_count(&self) -> usize {
        self.grouped_member_count
    }

    pub fn structurally_removed_member_count(&self) -> usize {
        self.structurally_removed_member_count
    }

    pub fn compression_gain(&self) -> CognitiveSignal {
        self.compression_gain
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[CompressedDomainModel] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DomainModelCompression;

impl DomainModelCompression {
    fn scaled_rate(numerator: usize, denominator: usize) -> CognitiveSignal {
        debug_assert!(denominator > 0);

        let scaled = ((numerator as u128) * 1000) / denominator as u128;

        CognitiveSignal::new(scaled as u16)
            .expect("bounded domain-model compression gain remains on signal scale")
    }

    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn same_target_model(
        left: &GroundedCrossDomainTransferHypothesis,
        right: &GroundedCrossDomainTransferHypothesis,
    ) -> bool {
        left.target_domain() == right.target_domain()
            && left.target_transformation() == right.target_transformation()
            && left.target_contrast_transformation() == right.target_contrast_transformation()
            && left.target_context() == right.target_context()
            && left.target_effect_fact() == right.target_effect_fact()
            && left.effect_kind() == right.effect_kind()
            && left.matched_target_state_count() == right.matched_target_state_count()
            && left.target_intervention_opportunity_count()
                == right.target_intervention_opportunity_count()
            && left.target_intervention_success_count() == right.target_intervention_success_count()
            && left.target_intervention_failure_count() == right.target_intervention_failure_count()
            && left.contrast_intervention_opportunity_count()
                == right.contrast_intervention_opportunity_count()
            && left.contrast_intervention_success_count()
                == right.contrast_intervention_success_count()
            && left.contrast_intervention_failure_count()
                == right.contrast_intervention_failure_count()
            && left.target_effect_rate() == right.target_effect_rate()
            && left.target_contrast_effect_rate() == right.target_contrast_effect_rate()
            && left.target_interventional_lift() == right.target_interventional_lift()
            && left.balanced_target_support() == right.balanced_target_support()
            && left.target_support_adequacy() == right.target_support_adequacy()
            && left.target_evidence_confidence() == right.target_evidence_confidence()
            && left.passive_corroborating_count() == right.passive_corroborating_count()
            && left.passive_counterevidence_count() == right.passive_counterevidence_count()
    }

    fn compare_target_identity(
        left: &GroundedCrossDomainTransferHypothesis,
        right: &GroundedCrossDomainTransferHypothesis,
    ) -> std::cmp::Ordering {
        PredicateDiscovery::compare_structure(left.target_domain(), right.target_domain())
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_transformation(),
                    right.target_transformation(),
                )
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_contrast_transformation(),
                    right.target_contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_effect_fact(),
                    right.target_effect_fact(),
                )
            })
            .then_with(|| Self::compare_context(left.target_context(), right.target_context()))
    }

    fn compare_hypothesis(
        left: &GroundedCrossDomainTransferHypothesis,
        right: &GroundedCrossDomainTransferHypothesis,
    ) -> std::cmp::Ordering {
        right
            .transfer_confidence()
            .value()
            .cmp(&left.transfer_confidence().value())
            .then_with(|| {
                right
                    .target_evidence_confidence()
                    .value()
                    .cmp(&left.target_evidence_confidence().value())
            })
            .then_with(|| {
                right
                    .target_interventional_lift()
                    .value()
                    .cmp(&left.target_interventional_lift().value())
            })
            .then_with(|| Self::compare_target_identity(left, right))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.source_domain(), right.source_domain())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.source_transformation(),
                    right.source_transformation(),
                )
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.source_contrast_transformation(),
                    right.source_contrast_transformation(),
                )
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.source_effect_fact(),
                    right.source_effect_fact(),
                )
            })
    }

    fn considered(
        hypotheses: &[GroundedCrossDomainTransferHypothesis],
        policy: DomainModelCompressionPolicy,
    ) -> Vec<&GroundedCrossDomainTransferHypothesis> {
        let mut considered = hypotheses.iter().collect::<Vec<_>>();

        considered.sort_by(|left, right| Self::compare_hypothesis(left, right));

        considered.truncate(policy.max_input_hypotheses());

        considered
    }

    fn possible_group_count(considered: &[&GroundedCrossDomainTransferHypothesis]) -> usize {
        let mut representatives = Vec::<&GroundedCrossDomainTransferHypothesis>::new();

        for hypothesis in considered {
            if representatives
                .iter()
                .any(|representative| Self::same_target_model(representative, hypothesis))
            {
                continue;
            }

            representatives.push(hypothesis);
        }

        representatives.len()
    }

    fn build_groups(
        considered: &[&GroundedCrossDomainTransferHypothesis],
        policy: DomainModelCompressionPolicy,
    ) -> Vec<DomainModelCompressionGroup> {
        let mut groups = Vec::<DomainModelCompressionGroup>::new();

        for hypothesis in considered {
            if let Some(group) = groups
                .iter_mut()
                .find(|group| Self::same_target_model(&group.representative, hypothesis))
            {
                group.members.push((*hypothesis).clone());

                continue;
            }

            if groups.len() >= policy.max_model_groups() {
                continue;
            }

            groups.push(DomainModelCompressionGroup {
                representative: (*hypothesis).clone(),
                members: vec![(*hypothesis).clone()],
            });
        }

        groups
    }

    fn provenance_from(
        hypothesis: &GroundedCrossDomainTransferHypothesis,
    ) -> CompressedTransferProvenance {
        CompressedTransferProvenance {
            source_domain: hypothesis.source_domain().clone(),
            source_transformation: hypothesis.source_transformation().clone(),
            source_contrast_transformation: hypothesis.source_contrast_transformation().clone(),
            source_effect_fact: hypothesis.source_effect_fact().clone(),
            source_validated_confidence: hypothesis.source_validated_confidence(),
            source_interventional_lift: hypothesis.source_interventional_lift(),
            transfer_confidence: hypothesis.transfer_confidence(),
        }
    }

    fn compare_provenance(
        left: &CompressedTransferProvenance,
        right: &CompressedTransferProvenance,
    ) -> std::cmp::Ordering {
        PredicateDiscovery::compare_structure(left.source_domain(), right.source_domain())
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.source_transformation(),
                    right.source_transformation(),
                )
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.source_contrast_transformation(),
                    right.source_contrast_transformation(),
                )
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.source_effect_fact(),
                    right.source_effect_fact(),
                )
            })
            .then_with(|| {
                right
                    .source_validated_confidence()
                    .value()
                    .cmp(&left.source_validated_confidence().value())
            })
            .then_with(|| {
                right
                    .source_interventional_lift()
                    .value()
                    .cmp(&left.source_interventional_lift().value())
            })
            .then_with(|| {
                right
                    .transfer_confidence()
                    .value()
                    .cmp(&left.transfer_confidence().value())
            })
    }

    fn compress_group(group: DomainModelCompressionGroup) -> CompressedDomainModel {
        let representative = &group.representative;

        let mut provenances = group
            .members
            .iter()
            .map(Self::provenance_from)
            .collect::<Vec<_>>();

        provenances.sort_by(Self::compare_provenance);

        provenances.dedup();

        let strongest_transfer_confidence = group
            .members
            .iter()
            .map(GroundedCrossDomainTransferHypothesis::transfer_confidence)
            .max_by_key(|signal| signal.value())
            .expect("compression group always has at least one member");

        let weakest_transfer_confidence = group
            .members
            .iter()
            .map(GroundedCrossDomainTransferHypothesis::transfer_confidence)
            .min_by_key(|signal| signal.value())
            .expect("compression group always has at least one member");

        CompressedDomainModel {
            target_domain: representative.target_domain().clone(),
            target_transformation: representative.target_transformation().clone(),
            target_contrast_transformation: representative.target_contrast_transformation().clone(),
            target_context: representative.target_context().clone(),
            target_effect_fact: representative.target_effect_fact().clone(),
            effect_kind: representative.effect_kind(),
            matched_target_state_count: representative.matched_target_state_count(),
            target_intervention_opportunity_count: representative
                .target_intervention_opportunity_count(),
            target_intervention_success_count: representative.target_intervention_success_count(),
            target_intervention_failure_count: representative.target_intervention_failure_count(),
            contrast_intervention_opportunity_count: representative
                .contrast_intervention_opportunity_count(),
            contrast_intervention_success_count: representative
                .contrast_intervention_success_count(),
            contrast_intervention_failure_count: representative
                .contrast_intervention_failure_count(),
            target_effect_rate: representative.target_effect_rate(),
            target_contrast_effect_rate: representative.target_contrast_effect_rate(),
            target_interventional_lift: representative.target_interventional_lift(),
            balanced_target_support: representative.balanced_target_support(),
            target_support_adequacy: representative.target_support_adequacy(),
            target_evidence_confidence: representative.target_evidence_confidence(),
            passive_corroborating_count: representative.passive_corroborating_count(),
            passive_counterevidence_count: representative.passive_counterevidence_count(),
            member_count: group.members.len(),
            provenances,
            strongest_transfer_confidence,
            weakest_transfer_confidence,
        }
    }

    fn compare_compressed(
        left: &CompressedDomainModel,
        right: &CompressedDomainModel,
    ) -> std::cmp::Ordering {
        right
            .member_count()
            .cmp(&left.member_count())
            .then_with(|| {
                right
                    .strongest_transfer_confidence()
                    .value()
                    .cmp(&left.strongest_transfer_confidence().value())
            })
            .then_with(|| {
                right
                    .target_evidence_confidence()
                    .value()
                    .cmp(&left.target_evidence_confidence().value())
            })
            .then_with(|| {
                right
                    .target_interventional_lift()
                    .value()
                    .cmp(&left.target_interventional_lift().value())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.target_domain(), right.target_domain())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_transformation(),
                    right.target_transformation(),
                )
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_contrast_transformation(),
                    right.target_contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_effect_fact(),
                    right.target_effect_fact(),
                )
            })
            .then_with(|| Self::compare_context(left.target_context(), right.target_context()))
    }

    pub fn compress(
        hypotheses: &[GroundedCrossDomainTransferHypothesis],
        policy: DomainModelCompressionPolicy,
    ) -> DomainModelCompressionResult {
        if hypotheses.is_empty() {
            return DomainModelCompressionResult {
                input_hypothesis_count: 0,
                considered_hypothesis_count: 0,
                input_frontier_truncated: false,
                possible_model_group_count: 0,
                generated_model_group_count: 0,
                group_generation_truncated: false,
                grouped_member_count: 0,
                structurally_removed_member_count: 0,
                compression_gain: CognitiveSignal::zero(),
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let considered = Self::considered(hypotheses, policy);

        let possible_model_group_count = Self::possible_group_count(&considered);

        let groups = Self::build_groups(&considered, policy);

        let generated_model_group_count = groups.len();

        let grouped_member_count = groups
            .iter()
            .map(|group| group.members.len())
            .sum::<usize>();

        let structurally_removed_member_count =
            grouped_member_count.saturating_sub(generated_model_group_count);

        let compression_gain = if grouped_member_count == 0 {
            CognitiveSignal::zero()
        } else {
            Self::scaled_rate(structurally_removed_member_count, grouped_member_count)
        };

        let mut compressed = groups
            .into_iter()
            .map(Self::compress_group)
            .collect::<Vec<_>>();

        compressed.sort_by(Self::compare_compressed);

        let admitted_before_frontier = compressed.len();

        compressed.truncate(policy.max_output_models());

        DomainModelCompressionResult {
            input_hypothesis_count: hypotheses.len(),
            considered_hypothesis_count: considered.len(),
            input_frontier_truncated: hypotheses.len() > considered.len(),
            possible_model_group_count,
            generated_model_group_count,
            group_generation_truncated: possible_model_group_count > generated_model_group_count,
            grouped_member_count,
            structurally_removed_member_count,
            compression_gain,
            admitted_before_frontier,
            selected: compressed,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalDomainModelCompression;

impl UniversalDomainModelCompression {
    pub fn evaluate(
        hypotheses: &[GroundedCrossDomainTransferHypothesis],
        policy: DomainModelCompressionPolicy,
    ) -> DomainModelCompressionResult {
        DomainModelCompression::compress(hypotheses, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IntegratedDomainRelationAuthority {
    LocalInterventional,
    TransferredCompressed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IntegratedDomainModelPolicy {
    max_local_hypotheses: usize,
    max_transferred_models: usize,
    max_relations: usize,
}

impl IntegratedDomainModelPolicy {
    pub fn new(
        max_local_hypotheses: usize,
        max_transferred_models: usize,
        max_relations: usize,
    ) -> Option<Self> {
        if max_local_hypotheses == 0 || max_transferred_models == 0 || max_relations == 0 {
            return None;
        }

        Some(Self {
            max_local_hypotheses,
            max_transferred_models,
            max_relations,
        })
    }

    pub fn max_local_hypotheses(self) -> usize {
        self.max_local_hypotheses
    }

    pub fn max_transferred_models(self) -> usize {
        self.max_transferred_models
    }

    pub fn max_relations(self) -> usize {
        self.max_relations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedDomainRelation {
    domain: CognitiveStructure,
    transformation: CognitiveStructure,
    contrast_transformation: CognitiveStructure,
    context: ContextPremiseSet,
    effect_kind: TransitionEffectKind,
    effect_fact: CognitiveStructure,
    authority: IntegratedDomainRelationAuthority,
    confidence_ceiling: CognitiveSignal,
    confidence_floor: CognitiveSignal,
    interventional_lift: CognitiveSignal,
    support_adequacy: CognitiveSignal,
    matched_state_count: usize,
    target_opportunity_count: u64,
    target_success_count: u64,
    target_failure_count: u64,
    contrast_opportunity_count: u64,
    contrast_success_count: u64,
    contrast_failure_count: u64,
    passive_corroborating_count: u64,
    passive_counterevidence_count: u64,
    provenance_count: usize,
    source_member_count: usize,
}

impl IntegratedDomainRelation {
    pub fn domain(&self) -> &CognitiveStructure {
        &self.domain
    }

    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn contrast_transformation(&self) -> &CognitiveStructure {
        &self.contrast_transformation
    }

    pub fn context(&self) -> &ContextPremiseSet {
        &self.context
    }

    pub fn effect_kind(&self) -> TransitionEffectKind {
        self.effect_kind
    }

    pub fn effect_fact(&self) -> &CognitiveStructure {
        &self.effect_fact
    }

    pub fn authority(&self) -> IntegratedDomainRelationAuthority {
        self.authority
    }

    pub fn confidence_ceiling(&self) -> CognitiveSignal {
        self.confidence_ceiling
    }

    pub fn confidence_floor(&self) -> CognitiveSignal {
        self.confidence_floor
    }

    pub fn interventional_lift(&self) -> CognitiveSignal {
        self.interventional_lift
    }

    pub fn support_adequacy(&self) -> CognitiveSignal {
        self.support_adequacy
    }

    pub fn matched_state_count(&self) -> usize {
        self.matched_state_count
    }

    pub fn target_opportunity_count(&self) -> u64 {
        self.target_opportunity_count
    }

    pub fn target_success_count(&self) -> u64 {
        self.target_success_count
    }

    pub fn target_failure_count(&self) -> u64 {
        self.target_failure_count
    }

    pub fn contrast_opportunity_count(&self) -> u64 {
        self.contrast_opportunity_count
    }

    pub fn contrast_success_count(&self) -> u64 {
        self.contrast_success_count
    }

    pub fn contrast_failure_count(&self) -> u64 {
        self.contrast_failure_count
    }

    pub fn passive_corroborating_count(&self) -> u64 {
        self.passive_corroborating_count
    }

    pub fn passive_counterevidence_count(&self) -> u64 {
        self.passive_counterevidence_count
    }

    pub fn provenance_count(&self) -> usize {
        self.provenance_count
    }

    pub fn source_member_count(&self) -> usize {
        self.source_member_count
    }

    pub fn same_semantic_key(&self, other: &Self) -> bool {
        self.domain == other.domain
            && self.transformation == other.transformation
            && self.contrast_transformation == other.contrast_transformation
            && self.context == other.context
            && self.effect_kind == other.effect_kind
            && self.effect_fact == other.effect_fact
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedDomainModelResult {
    domain: CognitiveStructure,
    input_local_hypothesis_count: usize,
    considered_local_hypothesis_count: usize,
    local_frontier_truncated: bool,
    input_transferred_model_count: usize,
    matching_transferred_model_count: usize,
    considered_transferred_model_count: usize,
    transferred_frontier_truncated: bool,
    rejected_target_domain_mismatch: usize,
    admitted_before_frontier: usize,
    relations: Vec<IntegratedDomainRelation>,
}

impl IntegratedDomainModelResult {
    pub fn domain(&self) -> &CognitiveStructure {
        &self.domain
    }

    pub fn input_local_hypothesis_count(&self) -> usize {
        self.input_local_hypothesis_count
    }

    pub fn considered_local_hypothesis_count(&self) -> usize {
        self.considered_local_hypothesis_count
    }

    pub fn local_frontier_truncated(&self) -> bool {
        self.local_frontier_truncated
    }

    pub fn input_transferred_model_count(&self) -> usize {
        self.input_transferred_model_count
    }

    pub fn matching_transferred_model_count(&self) -> usize {
        self.matching_transferred_model_count
    }

    pub fn considered_transferred_model_count(&self) -> usize {
        self.considered_transferred_model_count
    }

    pub fn transferred_frontier_truncated(&self) -> bool {
        self.transferred_frontier_truncated
    }

    pub fn rejected_target_domain_mismatch(&self) -> usize {
        self.rejected_target_domain_mismatch
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn relations(&self) -> &[IntegratedDomainRelation] {
        &self.relations
    }

    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    pub fn best_exact(
        &self,
        transformation: &CognitiveStructure,
        contrast_transformation: &CognitiveStructure,
        context: &ContextPremiseSet,
        effect_kind: TransitionEffectKind,
        effect_fact: &CognitiveStructure,
    ) -> Option<&IntegratedDomainRelation> {
        self.relations.iter().find(|relation| {
            relation.transformation() == transformation
                && relation.contrast_transformation() == contrast_transformation
                && relation.context() == context
                && relation.effect_kind() == effect_kind
                && relation.effect_fact() == effect_fact
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct IntegratedDomainModel;

impl IntegratedDomainModel {
    fn compare_context(left: &ContextPremiseSet, right: &ContextPremiseSet) -> std::cmp::Ordering {
        let mut left_iterator = left.premises().iter();

        let mut right_iterator = right.premises().iter();

        loop {
            match (left_iterator.next(), right_iterator.next()) {
                (Some(left_value), Some(right_value)) => {
                    let ordering = PredicateDiscovery::compare_structure(left_value, right_value);

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                (None, Some(_)) => {
                    return std::cmp::Ordering::Less;
                }

                (Some(_), None) => {
                    return std::cmp::Ordering::Greater;
                }

                (None, None) => {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
    }

    fn compare_local(
        left: &GroundedInterventionalCausalHypothesis,
        right: &GroundedInterventionalCausalHypothesis,
    ) -> std::cmp::Ordering {
        right
            .validated_causal_confidence()
            .value()
            .cmp(&left.validated_causal_confidence().value())
            .then_with(|| {
                right
                    .interventional_lift()
                    .value()
                    .cmp(&left.interventional_lift().value())
            })
            .then_with(|| {
                right
                    .intervention_support_adequacy()
                    .value()
                    .cmp(&left.intervention_support_adequacy().value())
            })
            .then_with(|| {
                right
                    .matched_intervention_state_count()
                    .cmp(&left.matched_intervention_state_count())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.contrast_transformation(),
                    right.contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
    }

    fn compare_transferred(
        left: &CompressedDomainModel,
        right: &CompressedDomainModel,
    ) -> std::cmp::Ordering {
        right
            .strongest_transfer_confidence()
            .value()
            .cmp(&left.strongest_transfer_confidence().value())
            .then_with(|| {
                right
                    .target_evidence_confidence()
                    .value()
                    .cmp(&left.target_evidence_confidence().value())
            })
            .then_with(|| {
                right
                    .target_interventional_lift()
                    .value()
                    .cmp(&left.target_interventional_lift().value())
            })
            .then_with(|| {
                right
                    .target_support_adequacy()
                    .value()
                    .cmp(&left.target_support_adequacy().value())
            })
            .then_with(|| right.member_count().cmp(&left.member_count()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_transformation(),
                    right.target_transformation(),
                )
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_contrast_transformation(),
                    right.target_contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.target_effect_fact(),
                    right.target_effect_fact(),
                )
            })
            .then_with(|| Self::compare_context(left.target_context(), right.target_context()))
    }

    fn considered_local(
        local: &[GroundedInterventionalCausalHypothesis],
        policy: IntegratedDomainModelPolicy,
    ) -> Vec<&GroundedInterventionalCausalHypothesis> {
        let mut considered = local.iter().collect::<Vec<_>>();

        considered.sort_by(|left, right| Self::compare_local(left, right));

        considered.truncate(policy.max_local_hypotheses());

        considered
    }

    fn considered_transferred<'a>(
        domain: &CognitiveStructure,
        transferred: &'a [CompressedDomainModel],
        policy: IntegratedDomainModelPolicy,
    ) -> Vec<&'a CompressedDomainModel> {
        let mut considered = transferred
            .iter()
            .filter(|model| model.target_domain() == domain)
            .collect::<Vec<_>>();

        considered.sort_by(|left, right| Self::compare_transferred(left, right));

        considered.truncate(policy.max_transferred_models());

        considered
    }

    fn relation_from_local(
        domain: &CognitiveStructure,
        hypothesis: &GroundedInterventionalCausalHypothesis,
    ) -> IntegratedDomainRelation {
        let confidence = hypothesis.validated_causal_confidence();

        IntegratedDomainRelation {
            domain: domain.clone(),
            transformation: hypothesis.transformation().clone(),
            contrast_transformation: hypothesis.contrast_transformation().clone(),
            context: hypothesis.context().clone(),
            effect_kind: hypothesis.effect_kind(),
            effect_fact: hypothesis.effect_fact().clone(),
            authority: IntegratedDomainRelationAuthority::LocalInterventional,
            confidence_ceiling: confidence,
            confidence_floor: confidence,
            interventional_lift: hypothesis.interventional_lift(),
            support_adequacy: hypothesis.intervention_support_adequacy(),
            matched_state_count: hypothesis.matched_intervention_state_count(),
            target_opportunity_count: hypothesis.target_intervention_opportunity_count(),
            target_success_count: hypothesis.target_intervention_success_count(),
            target_failure_count: hypothesis.target_intervention_failure_count(),
            contrast_opportunity_count: hypothesis.contrast_intervention_opportunity_count(),
            contrast_success_count: hypothesis.contrast_intervention_success_count(),
            contrast_failure_count: hypothesis.contrast_intervention_failure_count(),
            passive_corroborating_count: hypothesis.passive_corroborating_count(),
            passive_counterevidence_count: hypothesis.passive_counterevidence_count(),
            provenance_count: 0,
            source_member_count: 1,
        }
    }

    fn relation_from_transferred(model: &CompressedDomainModel) -> IntegratedDomainRelation {
        IntegratedDomainRelation {
            domain: model.target_domain().clone(),
            transformation: model.target_transformation().clone(),
            contrast_transformation: model.target_contrast_transformation().clone(),
            context: model.target_context().clone(),
            effect_kind: model.effect_kind(),
            effect_fact: model.target_effect_fact().clone(),
            authority: IntegratedDomainRelationAuthority::TransferredCompressed,
            confidence_ceiling: model.strongest_transfer_confidence(),
            confidence_floor: model.weakest_transfer_confidence(),
            interventional_lift: model.target_interventional_lift(),
            support_adequacy: model.target_support_adequacy(),
            matched_state_count: model.matched_target_state_count(),
            target_opportunity_count: model.target_intervention_opportunity_count(),
            target_success_count: model.target_intervention_success_count(),
            target_failure_count: model.target_intervention_failure_count(),
            contrast_opportunity_count: model.contrast_intervention_opportunity_count(),
            contrast_success_count: model.contrast_intervention_success_count(),
            contrast_failure_count: model.contrast_intervention_failure_count(),
            passive_corroborating_count: model.passive_corroborating_count(),
            passive_counterevidence_count: model.passive_counterevidence_count(),
            provenance_count: model.provenance_count(),
            source_member_count: model.member_count(),
        }
    }

    fn compare_relation(
        left: &IntegratedDomainRelation,
        right: &IntegratedDomainRelation,
    ) -> std::cmp::Ordering {
        left.authority()
            .cmp(&right.authority())
            .then_with(|| {
                right
                    .confidence_ceiling()
                    .value()
                    .cmp(&left.confidence_ceiling().value())
            })
            .then_with(|| {
                right
                    .confidence_floor()
                    .value()
                    .cmp(&left.confidence_floor().value())
            })
            .then_with(|| {
                right
                    .interventional_lift()
                    .value()
                    .cmp(&left.interventional_lift().value())
            })
            .then_with(|| {
                right
                    .support_adequacy()
                    .value()
                    .cmp(&left.support_adequacy().value())
            })
            .then_with(|| right.matched_state_count().cmp(&left.matched_state_count()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            })
            .then_with(|| {
                PredicateDiscovery::compare_structure(
                    left.contrast_transformation(),
                    right.contrast_transformation(),
                )
            })
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| {
                PredicateDiscovery::compare_structure(left.effect_fact(), right.effect_fact())
            })
            .then_with(|| Self::compare_context(left.context(), right.context()))
            .then_with(|| {
                right
                    .target_opportunity_count()
                    .cmp(&left.target_opportunity_count())
            })
            .then_with(|| {
                left.target_failure_count()
                    .cmp(&right.target_failure_count())
            })
            .then_with(|| {
                left.contrast_success_count()
                    .cmp(&right.contrast_success_count())
            })
    }

    pub fn build(
        domain: &CognitiveStructure,
        local: &[GroundedInterventionalCausalHypothesis],
        transferred: &[CompressedDomainModel],
        policy: IntegratedDomainModelPolicy,
    ) -> IntegratedDomainModelResult {
        let considered_local = Self::considered_local(local, policy);

        let matching_transferred_model_count = transferred
            .iter()
            .filter(|model| model.target_domain() == domain)
            .count();

        let rejected_target_domain_mismatch = transferred
            .len()
            .saturating_sub(matching_transferred_model_count);

        let considered_transferred = Self::considered_transferred(domain, transferred, policy);

        let mut relations = considered_local
            .iter()
            .map(|hypothesis| Self::relation_from_local(domain, hypothesis))
            .chain(
                considered_transferred
                    .iter()
                    .map(|model| Self::relation_from_transferred(model)),
            )
            .collect::<Vec<_>>();

        relations.sort_by(Self::compare_relation);

        let admitted_before_frontier = relations.len();

        relations.truncate(policy.max_relations());

        IntegratedDomainModelResult {
            domain: domain.clone(),
            input_local_hypothesis_count: local.len(),
            considered_local_hypothesis_count: considered_local.len(),
            local_frontier_truncated: local.len() > considered_local.len(),
            input_transferred_model_count: transferred.len(),
            matching_transferred_model_count,
            considered_transferred_model_count: considered_transferred.len(),
            transferred_frontier_truncated: matching_transferred_model_count
                > considered_transferred.len(),
            rejected_target_domain_mismatch,
            admitted_before_frontier,
            relations,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalIntegratedDomainModel;

impl UniversalIntegratedDomainModel {
    pub fn evaluate(
        domain: &CognitiveStructure,
        local: &[GroundedInterventionalCausalHypothesis],
        transferred: &[CompressedDomainModel],
        policy: IntegratedDomainModelPolicy,
    ) -> IntegratedDomainModelResult {
        IntegratedDomainModel::build(domain, local, transferred, policy)
    }
}
// ============================================================================
// K0-A — EXECUTABLE GENERATIVE WORLD MODEL FOUNDATION
// ============================================================================
//
// This is the first executable generative world-model primitive.
//
// It executes only semantics already admitted by grounded transition-schema
// learning:
//
//     grounded state + exact transformation + learned schemas
//         -> partial structural delta
//
// It intentionally does not construct a complete successor state.
//
// Facts not explicitly predicted as added or removed remain epistemically
// unknown. GroundedInvariantHypothesis is intentionally not executed as
// persistence authority here because its current semantics are empirical
// cross-transformation stability rather than transformation-specific certainty.
//
// Exact CognitiveStructure identity is authoritative throughout.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GroundedExecutableWorldModelPolicy {
    max_schemas: usize,
}

impl GroundedExecutableWorldModelPolicy {
    pub fn new(max_schemas: usize) -> Option<Self> {
        if max_schemas == 0 {
            return None;
        }

        Some(Self { max_schemas })
    }

    pub fn max_schemas(self) -> usize {
        self.max_schemas
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedExecutableWorldModel {
    schemas: Vec<GroundedTransitionSchemaHypothesis>,
    admitted_before_frontier: usize,
}

impl GroundedExecutableWorldModel {
    fn schema_identity_cmp(
        left: &GroundedTransitionSchemaHypothesis,
        right: &GroundedTransitionSchemaHypothesis,
    ) -> std::cmp::Ordering {
        PredicateDiscovery::compare_structure(left.transformation(), right.transformation())
            .then_with(|| left.effect_kind().cmp(&right.effect_kind()))
            .then_with(|| PredicateDiscovery::compare_structure(left.fact(), right.fact()))
    }

    fn schema_strength_cmp(
        left: &GroundedTransitionSchemaHypothesis,
        right: &GroundedTransitionSchemaHypothesis,
    ) -> std::cmp::Ordering {
        right
            .association_lift()
            .value()
            .cmp(&left.association_lift().value())
            .then_with(|| right.precision().value().cmp(&left.precision().value()))
            .then_with(|| right.support_count().cmp(&left.support_count()))
            .then_with(|| {
                left.counterexample_count()
                    .cmp(&right.counterexample_count())
            })
            .then_with(|| Self::schema_identity_cmp(left, right))
    }

    fn same_schema_identity(
        left: &GroundedTransitionSchemaHypothesis,
        right: &GroundedTransitionSchemaHypothesis,
    ) -> bool {
        left.transformation() == right.transformation()
            && left.effect_kind() == right.effect_kind()
            && left.fact() == right.fact()
    }

    pub fn build(
        schemas: &[GroundedTransitionSchemaHypothesis],
        policy: GroundedExecutableWorldModelPolicy,
    ) -> Self {
        let mut canonical = schemas.to_vec();

        canonical.sort_by(Self::schema_strength_cmp);

        canonical.dedup_by(|left, right| Self::same_schema_identity(left, right));

        let admitted_before_frontier = canonical.len();

        canonical.truncate(policy.max_schemas());

        canonical.sort_by(Self::schema_identity_cmp);

        Self {
            schemas: canonical,
            admitted_before_frontier,
        }
    }

    pub fn schemas(&self) -> &[GroundedTransitionSchemaHypothesis] {
        &self.schemas
    }

    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn frontier_truncated(&self) -> bool {
        self.admitted_before_frontier > self.schemas.len()
    }

    pub fn predict(
        &self,
        state: &GroundedStateSnapshot,
        transformation: &CognitiveStructure,
    ) -> GroundedStructuralPrediction {
        GroundedStructuralPredictionEngine::predict(state, transformation, self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GroundedStructuralPredictionStatus {
    Predicted,
    NoApplicableEffect,
    EffectConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedStructuralPrediction {
    transformation: CognitiveStructure,
    status: GroundedStructuralPredictionStatus,
    additions: Vec<CognitiveStructure>,
    removals: Vec<CognitiveStructure>,
    applicable_schema_count: usize,
}

impl GroundedStructuralPrediction {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn status(&self) -> GroundedStructuralPredictionStatus {
        self.status
    }

    pub fn additions(&self) -> &[CognitiveStructure] {
        &self.additions
    }

    pub fn removals(&self) -> &[CognitiveStructure] {
        &self.removals
    }

    pub fn applicable_schema_count(&self) -> usize {
        self.applicable_schema_count
    }

    pub fn predicted(&self) -> bool {
        self.status == GroundedStructuralPredictionStatus::Predicted
    }

    pub fn abstained(&self) -> bool {
        self.status != GroundedStructuralPredictionStatus::Predicted
    }

    pub fn predicts_addition(&self, fact: &CognitiveStructure) -> bool {
        self.additions
            .binary_search_by(|candidate| PredicateDiscovery::compare_structure(candidate, fact))
            .is_ok()
    }

    pub fn predicts_removal(&self, fact: &CognitiveStructure) -> bool {
        self.removals
            .binary_search_by(|candidate| PredicateDiscovery::compare_structure(candidate, fact))
            .is_ok()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GroundedStructuralPredictionEngine;

impl GroundedStructuralPredictionEngine {
    pub const fn new() -> Self {
        Self
    }

    fn schema_is_applicable(
        schema: &GroundedTransitionSchemaHypothesis,
        state: &GroundedStateSnapshot,
        transformation: &CognitiveStructure,
    ) -> bool {
        if schema.transformation() != transformation {
            return false;
        }

        match schema.effect_kind() {
            TransitionEffectKind::Added => !state.contains_fact(schema.fact()),
            TransitionEffectKind::Removed => state.contains_fact(schema.fact()),
        }
    }

    pub fn predict(
        state: &GroundedStateSnapshot,
        transformation: &CognitiveStructure,
        model: &GroundedExecutableWorldModel,
    ) -> GroundedStructuralPrediction {
        let applicable = model
            .schemas()
            .iter()
            .filter(|schema| Self::schema_is_applicable(schema, state, transformation))
            .collect::<Vec<_>>();

        let applicable_schema_count = applicable.len();

        if applicable.is_empty() {
            return GroundedStructuralPrediction {
                transformation: transformation.clone(),
                status: GroundedStructuralPredictionStatus::NoApplicableEffect,
                additions: Vec::new(),
                removals: Vec::new(),
                applicable_schema_count,
            };
        }

        let mut additions = applicable
            .iter()
            .filter(|schema| schema.effect_kind() == TransitionEffectKind::Added)
            .map(|schema| schema.fact().clone())
            .collect::<Vec<_>>();

        let mut removals = applicable
            .iter()
            .filter(|schema| schema.effect_kind() == TransitionEffectKind::Removed)
            .map(|schema| schema.fact().clone())
            .collect::<Vec<_>>();

        additions.sort_by(PredicateDiscovery::compare_structure);
        additions.dedup();

        removals.sort_by(PredicateDiscovery::compare_structure);
        removals.dedup();

        let conflict = additions.iter().any(|fact| {
            removals
                .binary_search_by(|candidate| {
                    PredicateDiscovery::compare_structure(candidate, fact)
                })
                .is_ok()
        });

        if conflict {
            return GroundedStructuralPrediction {
                transformation: transformation.clone(),
                status: GroundedStructuralPredictionStatus::EffectConflict,
                additions: Vec::new(),
                removals: Vec::new(),
                applicable_schema_count,
            };
        }

        GroundedStructuralPrediction {
            transformation: transformation.clone(),
            status: GroundedStructuralPredictionStatus::Predicted,
            additions,
            removals,
            applicable_schema_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalGroundedExecutableWorldModel;

impl UniversalGroundedExecutableWorldModel {
    pub fn build(
        schemas: &[GroundedTransitionSchemaHypothesis],
        policy: GroundedExecutableWorldModelPolicy,
    ) -> GroundedExecutableWorldModel {
        GroundedExecutableWorldModel::build(schemas, policy)
    }

    pub fn predict(
        state: &GroundedStateSnapshot,
        transformation: &CognitiveStructure,
        model: &GroundedExecutableWorldModel,
    ) -> GroundedStructuralPrediction {
        GroundedStructuralPredictionEngine::predict(state, transformation, model)
    }
}
// ============================================================================
// K0-B — COMPETING EXECUTABLE MODEL FRONTIER
// ============================================================================
//
// A frontier retains multiple distinct executable world models without
// collapsing uncertainty into one preferred explanation.
//
// Every model is executed against the same exact grounded state and exact
// transformation. Disagreement is measured from the resulting partial
// structural predictions.
//
// Unknown is preserved as a real epistemic state. A model that predicts an
// effect and a model that remains silent about that effect therefore disagree.
//
// This layer does not select actions and does not claim information gain.
// It exposes the exact structural disagreement required for those later steps.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GroundedExecutableModelFrontierPolicy {
    max_models: usize,
}

impl GroundedExecutableModelFrontierPolicy {
    pub fn new(max_models: usize) -> Option<Self> {
        if max_models == 0 {
            return None;
        }

        Some(Self { max_models })
    }

    pub fn max_models(self) -> usize {
        self.max_models
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedExecutableModelFrontier {
    models: Vec<GroundedExecutableWorldModel>,
    admitted_before_frontier: usize,
}

impl GroundedExecutableModelFrontier {
    fn compare_models(
        left: &GroundedExecutableWorldModel,
        right: &GroundedExecutableWorldModel,
    ) -> std::cmp::Ordering {
        left.schema_count()
            .cmp(&right.schema_count())
            .then_with(|| {
                for (left_schema, right_schema) in left.schemas().iter().zip(right.schemas().iter())
                {
                    let ordering = GroundedExecutableWorldModel::schema_identity_cmp(
                        left_schema,
                        right_schema,
                    );

                    if ordering != std::cmp::Ordering::Equal {
                        return ordering;
                    }
                }

                std::cmp::Ordering::Equal
            })
    }

    fn same_predictive_model(
        left: &GroundedExecutableWorldModel,
        right: &GroundedExecutableWorldModel,
    ) -> bool {
        left.schema_count() == right.schema_count()
            && left.schemas().iter().zip(right.schemas().iter()).all(
                |(left_schema, right_schema)| {
                    GroundedExecutableWorldModel::same_schema_identity(left_schema, right_schema)
                },
            )
    }

    pub fn build(
        models: &[GroundedExecutableWorldModel],
        policy: GroundedExecutableModelFrontierPolicy,
    ) -> Self {
        let mut canonical = models.to_vec();

        canonical.sort_by(Self::compare_models);

        canonical.dedup_by(|left, right| Self::same_predictive_model(left, right));

        let admitted_before_frontier = canonical.len();

        canonical.truncate(policy.max_models());

        Self {
            models: canonical,
            admitted_before_frontier,
        }
    }

    pub fn models(&self) -> &[GroundedExecutableWorldModel] {
        &self.models
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn frontier_truncated(&self) -> bool {
        self.admitted_before_frontier > self.models.len()
    }

    pub fn evaluate(
        &self,
        state: &GroundedStateSnapshot,
        transformation: &CognitiveStructure,
    ) -> GroundedExecutableModelDisagreement {
        GroundedExecutableModelDisagreementEngine::evaluate(state, transformation, self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedStructuralDisagreementFact {
    fact: CognitiveStructure,
    added_model_count: usize,
    removed_model_count: usize,
    unknown_model_count: usize,
    conflict_model_count: usize,
}

impl GroundedStructuralDisagreementFact {
    pub fn fact(&self) -> &CognitiveStructure {
        &self.fact
    }

    pub fn added_model_count(&self) -> usize {
        self.added_model_count
    }

    pub fn removed_model_count(&self) -> usize {
        self.removed_model_count
    }

    pub fn unknown_model_count(&self) -> usize {
        self.unknown_model_count
    }

    pub fn conflict_model_count(&self) -> usize {
        self.conflict_model_count
    }

    pub fn participating_model_count(&self) -> usize {
        self.added_model_count
            .saturating_add(self.removed_model_count)
            .saturating_add(self.unknown_model_count)
            .saturating_add(self.conflict_model_count)
    }

    pub fn disposition_count(&self) -> usize {
        [
            self.added_model_count,
            self.removed_model_count,
            self.unknown_model_count,
            self.conflict_model_count,
        ]
        .into_iter()
        .filter(|count| *count > 0)
        .count()
    }

    pub fn is_disputed(&self) -> bool {
        self.disposition_count() > 1
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedExecutableModelDisagreement {
    transformation: CognitiveStructure,
    model_predictions: Vec<GroundedStructuralPrediction>,
    disputed_facts: Vec<GroundedStructuralDisagreementFact>,
    predicted_model_count: usize,
    no_applicable_effect_model_count: usize,
    conflict_model_count: usize,
}

impl GroundedExecutableModelDisagreement {
    pub fn transformation(&self) -> &CognitiveStructure {
        &self.transformation
    }

    pub fn model_predictions(&self) -> &[GroundedStructuralPrediction] {
        &self.model_predictions
    }

    pub fn model_count(&self) -> usize {
        self.model_predictions.len()
    }

    pub fn disputed_facts(&self) -> &[GroundedStructuralDisagreementFact] {
        &self.disputed_facts
    }

    pub fn disputed_fact_count(&self) -> usize {
        self.disputed_facts.len()
    }

    pub fn predicted_model_count(&self) -> usize {
        self.predicted_model_count
    }

    pub fn no_applicable_effect_model_count(&self) -> usize {
        self.no_applicable_effect_model_count
    }

    pub fn conflict_model_count(&self) -> usize {
        self.conflict_model_count
    }

    pub fn has_disagreement(&self) -> bool {
        !self.disputed_facts.is_empty()
    }

    pub fn disagreement_for(
        &self,
        fact: &CognitiveStructure,
    ) -> Option<&GroundedStructuralDisagreementFact> {
        self.disputed_facts
            .binary_search_by(|candidate| {
                PredicateDiscovery::compare_structure(candidate.fact(), fact)
            })
            .ok()
            .map(|index| &self.disputed_facts[index])
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GroundedExecutableModelDisagreementEngine;

impl GroundedExecutableModelDisagreementEngine {
    pub const fn new() -> Self {
        Self
    }

    fn prediction_vocabulary(
        predictions: &[GroundedStructuralPrediction],
    ) -> Vec<CognitiveStructure> {
        let mut facts = predictions
            .iter()
            .flat_map(|prediction| {
                prediction
                    .additions()
                    .iter()
                    .chain(prediction.removals().iter())
            })
            .cloned()
            .collect::<Vec<_>>();

        facts.sort_by(PredicateDiscovery::compare_structure);
        facts.dedup();

        facts
    }

    fn summarize_fact(
        fact: &CognitiveStructure,
        predictions: &[GroundedStructuralPrediction],
    ) -> GroundedStructuralDisagreementFact {
        let mut added_model_count = 0_usize;
        let mut removed_model_count = 0_usize;
        let mut unknown_model_count = 0_usize;
        let mut conflict_model_count = 0_usize;

        for prediction in predictions {
            if prediction.status() == GroundedStructuralPredictionStatus::EffectConflict {
                conflict_model_count = conflict_model_count.saturating_add(1);
                continue;
            }

            if prediction.predicts_addition(fact) {
                added_model_count = added_model_count.saturating_add(1);
            } else if prediction.predicts_removal(fact) {
                removed_model_count = removed_model_count.saturating_add(1);
            } else {
                unknown_model_count = unknown_model_count.saturating_add(1);
            }
        }

        GroundedStructuralDisagreementFact {
            fact: fact.clone(),
            added_model_count,
            removed_model_count,
            unknown_model_count,
            conflict_model_count,
        }
    }

    pub fn evaluate(
        state: &GroundedStateSnapshot,
        transformation: &CognitiveStructure,
        frontier: &GroundedExecutableModelFrontier,
    ) -> GroundedExecutableModelDisagreement {
        let model_predictions = frontier
            .models()
            .iter()
            .map(|model| model.predict(state, transformation))
            .collect::<Vec<_>>();

        let predicted_model_count = model_predictions
            .iter()
            .filter(|prediction| {
                prediction.status() == GroundedStructuralPredictionStatus::Predicted
            })
            .count();

        let no_applicable_effect_model_count = model_predictions
            .iter()
            .filter(|prediction| {
                prediction.status() == GroundedStructuralPredictionStatus::NoApplicableEffect
            })
            .count();

        let conflict_model_count = model_predictions
            .iter()
            .filter(|prediction| {
                prediction.status() == GroundedStructuralPredictionStatus::EffectConflict
            })
            .count();

        let vocabulary = Self::prediction_vocabulary(&model_predictions);

        let mut disputed_facts = vocabulary
            .iter()
            .map(|fact| Self::summarize_fact(fact, &model_predictions))
            .filter(GroundedStructuralDisagreementFact::is_disputed)
            .collect::<Vec<_>>();

        disputed_facts.sort_by(|left, right| {
            PredicateDiscovery::compare_structure(left.fact(), right.fact())
        });

        GroundedExecutableModelDisagreement {
            transformation: transformation.clone(),
            model_predictions,
            disputed_facts,
            predicted_model_count,
            no_applicable_effect_model_count,
            conflict_model_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalGroundedExecutableModelFrontier;

impl UniversalGroundedExecutableModelFrontier {
    pub fn build(
        models: &[GroundedExecutableWorldModel],
        policy: GroundedExecutableModelFrontierPolicy,
    ) -> GroundedExecutableModelFrontier {
        GroundedExecutableModelFrontier::build(models, policy)
    }

    pub fn evaluate(
        state: &GroundedStateSnapshot,
        transformation: &CognitiveStructure,
        frontier: &GroundedExecutableModelFrontier,
    ) -> GroundedExecutableModelDisagreement {
        GroundedExecutableModelDisagreementEngine::evaluate(state, transformation, frontier)
    }
}

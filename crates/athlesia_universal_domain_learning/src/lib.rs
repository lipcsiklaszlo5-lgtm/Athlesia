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

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

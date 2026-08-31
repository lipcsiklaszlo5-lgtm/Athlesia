use std::collections::{BTreeMap, BTreeSet};

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInducedClassSet,
    RecursiveWorldRevisionAbstractionInductionContext,
    RecursiveWorldRevisionAbstractionInductionSide,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationThreshold {
    min_context_support: usize,
}

impl RecursiveWorldRevisionAbstractionGeneralizationThreshold {
    pub fn new(min_context_support: usize) -> Option<Self> {
        if min_context_support < 2 {
            return None;
        }

        Some(Self {
            min_context_support,
        })
    }

    pub const fn min_context_support(&self) -> usize {
        self.min_context_support
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionGeneralizedPairSupport {
    side: RecursiveWorldRevisionAbstractionInductionSide,
    first: RecursiveUnit,
    second: RecursiveUnit,
    contexts: Vec<RecursiveWorldRevisionAbstractionInductionContext>,
}

impl RecursiveWorldRevisionAbstractionGeneralizedPairSupport {
    fn new(
        side: RecursiveWorldRevisionAbstractionInductionSide,
        first: RecursiveUnit,
        second: RecursiveUnit,
        contexts: Vec<RecursiveWorldRevisionAbstractionInductionContext>,
    ) -> Self {
        let (first, second) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };

        let mut contexts = contexts;

        contexts.sort();
        contexts.dedup();

        Self {
            side,
            first,
            second,
            contexts,
        }
    }

    pub const fn side(&self) -> RecursiveWorldRevisionAbstractionInductionSide {
        self.side
    }

    pub fn first(&self) -> &RecursiveUnit {
        &self.first
    }

    pub fn second(&self) -> &RecursiveUnit {
        &self.second
    }

    pub fn contexts(&self) -> &[RecursiveWorldRevisionAbstractionInductionContext] {
        &self.contexts
    }

    pub fn support_count(&self) -> usize {
        self.contexts.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionGeneralizedClass {
    side: RecursiveWorldRevisionAbstractionInductionSide,
    abstraction_class: RecursiveWorldRevisionAbstractionClass,
    threshold: RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    pair_supports: Vec<RecursiveWorldRevisionAbstractionGeneralizedPairSupport>,
    supporting_contexts: Vec<RecursiveWorldRevisionAbstractionInductionContext>,
}

impl RecursiveWorldRevisionAbstractionGeneralizedClass {
    pub const fn side(&self) -> RecursiveWorldRevisionAbstractionInductionSide {
        self.side
    }

    pub fn abstraction_class(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.abstraction_class
    }

    pub const fn threshold(&self) -> RecursiveWorldRevisionAbstractionGeneralizationThreshold {
        self.threshold
    }

    pub fn pair_supports(&self) -> &[RecursiveWorldRevisionAbstractionGeneralizedPairSupport] {
        &self.pair_supports
    }

    pub fn supporting_contexts(&self) -> &[RecursiveWorldRevisionAbstractionInductionContext] {
        &self.supporting_contexts
    }

    pub fn minimum_pair_support(&self) -> usize {
        self.pair_supports
            .iter()
            .map(RecursiveWorldRevisionAbstractionGeneralizedPairSupport::support_count)
            .min()
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    source: RecursiveWorldRevisionAbstractionInducedClassSet,
    threshold: RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    classes: Vec<RecursiveWorldRevisionAbstractionGeneralizedClass>,
    pair_supports: Vec<RecursiveWorldRevisionAbstractionGeneralizedPairSupport>,
}

impl RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    pub fn generalize(
        source: RecursiveWorldRevisionAbstractionInducedClassSet,
        threshold: RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    ) -> Option<Self> {
        type PairKey = (
            RecursiveWorldRevisionAbstractionInductionSide,
            RecursiveUnit,
            RecursiveUnit,
        );

        let mut support_map =
            BTreeMap::<PairKey, BTreeSet<RecursiveWorldRevisionAbstractionInductionContext>>::new();

        for induced in source.classes() {
            let side = induced.context().side();

            let members = induced.abstraction_class().members();

            for first_index in 0..members.len() {
                for second_index in (first_index + 1)..members.len() {
                    let first = members[first_index].clone();

                    let second = members[second_index].clone();

                    let key = if first <= second {
                        (side, first, second)
                    } else {
                        (side, second, first)
                    };

                    support_map
                        .entry(key)
                        .or_default()
                        .insert(induced.context().clone());
                }
            }
        }

        let pair_supports = support_map
            .iter()
            .map(|((side, first, second), contexts)| {
                RecursiveWorldRevisionAbstractionGeneralizedPairSupport::new(
                    *side,
                    first.clone(),
                    second.clone(),
                    contexts.iter().cloned().collect(),
                )
            })
            .collect::<Vec<_>>();

        let mut qualifying_adjacency = BTreeMap::<
            (
                RecursiveWorldRevisionAbstractionInductionSide,
                RecursiveUnit,
            ),
            BTreeSet<RecursiveUnit>,
        >::new();

        for support in &pair_supports {
            if support.support_count() < threshold.min_context_support() {
                continue;
            }

            qualifying_adjacency
                .entry((support.side(), support.first().clone()))
                .or_default()
                .insert(support.second().clone());

            qualifying_adjacency
                .entry((support.side(), support.second().clone()))
                .or_default()
                .insert(support.first().clone());
        }

        let mut visited = BTreeSet::<(
            RecursiveWorldRevisionAbstractionInductionSide,
            RecursiveUnit,
        )>::new();

        let mut generalized = Vec::<RecursiveWorldRevisionAbstractionGeneralizedClass>::new();

        for (side, seed) in qualifying_adjacency.keys() {
            let seed_key = (*side, seed.clone());

            if visited.contains(&seed_key) {
                continue;
            }

            let mut frontier = vec![seed.clone()];

            let mut component = BTreeSet::<RecursiveUnit>::new();

            while let Some(current) = frontier.pop() {
                let current_key = (*side, current.clone());

                if !visited.insert(current_key) {
                    continue;
                }

                component.insert(current.clone());

                if let Some(neighbors) = qualifying_adjacency.get(&(*side, current)) {
                    for neighbor in neighbors {
                        let neighbor_key = (*side, neighbor.clone());

                        if !visited.contains(&neighbor_key) {
                            frontier.push(neighbor.clone());
                        }
                    }
                }
            }

            if component.len() < 2 {
                continue;
            }

            let members = component.iter().cloned().collect::<Vec<_>>();

            let mut component_pair_supports =
                Vec::<RecursiveWorldRevisionAbstractionGeneralizedPairSupport>::new();

            let mut complete = true;

            for first_index in 0..members.len() {
                for second_index in (first_index + 1)..members.len() {
                    let first = &members[first_index];

                    let second = &members[second_index];

                    let support = pair_supports.iter().find(|support| {
                        support.side() == *side
                            && support.first() == first
                            && support.second() == second
                    });

                    match support {
                        Some(support)
                            if support.support_count() >= threshold.min_context_support() =>
                        {
                            component_pair_supports.push(support.clone());
                        }

                        _ => {
                            complete = false;

                            break;
                        }
                    }
                }

                if !complete {
                    break;
                }
            }

            if !complete {
                continue;
            }

            let Some(abstraction_class) = RecursiveWorldRevisionAbstractionClass::new(members)
            else {
                continue;
            };

            let mut supporting_contexts =
                BTreeSet::<RecursiveWorldRevisionAbstractionInductionContext>::new();

            for support in &component_pair_supports {
                supporting_contexts.extend(support.contexts().iter().cloned());
            }

            component_pair_supports.sort();
            component_pair_supports.dedup();

            generalized.push(RecursiveWorldRevisionAbstractionGeneralizedClass {
                side: *side,
                abstraction_class,
                threshold,
                pair_supports: component_pair_supports,
                supporting_contexts: supporting_contexts.into_iter().collect(),
            });
        }

        generalized.sort();
        generalized.dedup();

        if generalized.is_empty() {
            return None;
        }

        Some(Self {
            source,
            threshold,
            classes: generalized,
            pair_supports,
        })
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionInducedClassSet {
        &self.source
    }

    pub const fn threshold(&self) -> RecursiveWorldRevisionAbstractionGeneralizationThreshold {
        self.threshold
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionGeneralizedClass] {
        &self.classes
    }

    pub fn pair_supports(&self) -> &[RecursiveWorldRevisionAbstractionGeneralizedPairSupport] {
        &self.pair_supports
    }

    pub fn len(&self) -> usize {
        self.classes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    pub fn premise_classes(&self) -> Vec<&RecursiveWorldRevisionAbstractionGeneralizedClass> {
        self.classes
            .iter()
            .filter(|class| class.side() == RecursiveWorldRevisionAbstractionInductionSide::Premise)
            .collect()
    }

    pub fn conclusion_classes(&self) -> Vec<&RecursiveWorldRevisionAbstractionGeneralizedClass> {
        self.classes
            .iter()
            .filter(|class| {
                class.side() == RecursiveWorldRevisionAbstractionInductionSide::Conclusion
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizer;

impl RecursiveWorldRevisionAbstractionGeneralizer {
    pub fn generalize(
        source: RecursiveWorldRevisionAbstractionInducedClassSet,
        threshold: RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    ) -> Option<RecursiveWorldRevisionAbstractionGeneralizedClassSet> {
        RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(source, threshold)
    }
}

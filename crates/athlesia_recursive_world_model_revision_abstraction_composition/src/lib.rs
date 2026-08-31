use std::collections::{BTreeMap, BTreeSet};

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionThreshold {
    min_observation_support: usize,
}

impl RecursiveWorldRevisionAbstractionCompositionThreshold {
    pub fn new(min_observation_support: usize) -> Option<Self> {
        if min_observation_support < 2 {
            return None;
        }

        Some(Self {
            min_observation_support,
        })
    }

    pub fn min_observation_support(&self) -> usize {
        self.min_observation_support
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionWitness {
    from: RecursiveWorldRevisionAbstractionClass,
    to: RecursiveWorldRevisionAbstractionClass,
    observation: RecursiveWorldRevisionDiscoveryObservation,
}

impl RecursiveWorldRevisionAbstractionCompositionWitness {
    pub fn new(
        from: RecursiveWorldRevisionAbstractionClass,
        to: RecursiveWorldRevisionAbstractionClass,
        observation: RecursiveWorldRevisionDiscoveryObservation,
    ) -> Option<Self> {
        if from == to {
            return None;
        }

        let premise_covered = observation
            .premises()
            .iter()
            .any(|unit| from.contains(unit));

        if !premise_covered {
            return None;
        }

        let conclusion_covered = observation
            .conclusions()
            .iter()
            .any(|unit| to.contains(unit));

        if !conclusion_covered {
            return None;
        }

        Some(Self {
            from,
            to,
            observation,
        })
    }

    pub fn from(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.from
    }

    pub fn to(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.to
    }

    pub fn observation(&self) -> &RecursiveWorldRevisionDiscoveryObservation {
        &self.observation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionWitnessSet {
    witnesses: Vec<RecursiveWorldRevisionAbstractionCompositionWitness>,
}

impl RecursiveWorldRevisionAbstractionCompositionWitnessSet {
    pub fn new(
        mut witnesses: Vec<RecursiveWorldRevisionAbstractionCompositionWitness>,
    ) -> Option<Self> {
        witnesses.sort();
        witnesses.dedup();

        if witnesses.is_empty() {
            return None;
        }

        Some(Self { witnesses })
    }

    pub fn witnesses(&self) -> &[RecursiveWorldRevisionAbstractionCompositionWitness] {
        &self.witnesses
    }

    pub fn len(&self) -> usize {
        self.witnesses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.witnesses.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionEdge {
    from: RecursiveWorldRevisionAbstractionClass,
    to: RecursiveWorldRevisionAbstractionClass,
    threshold: RecursiveWorldRevisionAbstractionCompositionThreshold,
    supporting_observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
}

impl RecursiveWorldRevisionAbstractionCompositionEdge {
    pub fn from(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.from
    }

    pub fn to(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.to
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionAbstractionCompositionThreshold {
        self.threshold
    }

    pub fn supporting_observations(&self) -> &[RecursiveWorldRevisionDiscoveryObservation] {
        &self.supporting_observations
    }

    pub fn support_count(&self) -> usize {
        self.supporting_observations.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionComposition {
    source: RecursiveWorldRevisionAbstractionCompositionWitnessSet,
    threshold: RecursiveWorldRevisionAbstractionCompositionThreshold,
    edges: Vec<RecursiveWorldRevisionAbstractionCompositionEdge>,
}

impl RecursiveWorldRevisionAbstractionComposition {
    pub fn compose(
        source: RecursiveWorldRevisionAbstractionCompositionWitnessSet,
        threshold: RecursiveWorldRevisionAbstractionCompositionThreshold,
    ) -> Option<Self> {
        let mut grouped: BTreeMap<
            (
                RecursiveWorldRevisionAbstractionClass,
                RecursiveWorldRevisionAbstractionClass,
            ),
            BTreeSet<RecursiveWorldRevisionDiscoveryObservation>,
        > = BTreeMap::new();

        for witness in source.witnesses() {
            grouped
                .entry((witness.from().clone(), witness.to().clone()))
                .or_default()
                .insert(witness.observation().clone());
        }

        let mut edges = Vec::new();

        for ((from, to), observations) in grouped {
            if observations.len() < threshold.min_observation_support() {
                continue;
            }

            edges.push(RecursiveWorldRevisionAbstractionCompositionEdge {
                from,
                to,
                threshold,
                supporting_observations: observations.into_iter().collect(),
            });
        }

        edges.sort();
        edges.dedup();

        if edges.is_empty() {
            return None;
        }

        Some(Self {
            source,
            threshold,
            edges,
        })
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionCompositionWitnessSet {
        &self.source
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionAbstractionCompositionThreshold {
        self.threshold
    }

    pub fn edges(&self) -> &[RecursiveWorldRevisionAbstractionCompositionEdge] {
        &self.edges
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn edge(
        &self,
        from: &RecursiveWorldRevisionAbstractionClass,
        to: &RecursiveWorldRevisionAbstractionClass,
    ) -> Option<&RecursiveWorldRevisionAbstractionCompositionEdge> {
        self.edges
            .iter()
            .find(|edge| edge.from() == from && edge.to() == to)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionComposer;

impl RecursiveWorldRevisionAbstractionComposer {
    pub fn compose(
        source: RecursiveWorldRevisionAbstractionCompositionWitnessSet,
        threshold: RecursiveWorldRevisionAbstractionCompositionThreshold,
    ) -> Option<RecursiveWorldRevisionAbstractionComposition> {
        RecursiveWorldRevisionAbstractionComposition::compose(source, threshold)
    }
}

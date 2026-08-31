use std::collections::{BTreeMap, BTreeSet};

use athlesia_recursive::RecursiveUnit;
use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;
use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionClass {
    members: Vec<RecursiveUnit>,
}

impl RecursiveWorldRevisionAbstractionClass {
    pub fn new(mut members: Vec<RecursiveUnit>) -> Option<Self> {
        members.sort();
        members.dedup();

        if members.len() < 2 {
            return None;
        }

        Some(Self { members })
    }

    pub fn members(&self) -> &[RecursiveUnit] {
        &self.members
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn contains(&self, unit: &RecursiveUnit) -> bool {
        self.members.binary_search(unit).is_ok()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionVocabulary {
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    unit_to_class: BTreeMap<RecursiveUnit, RecursiveWorldRevisionAbstractionClass>,
}

impl RecursiveWorldRevisionAbstractionVocabulary {
    pub fn new(mut classes: Vec<RecursiveWorldRevisionAbstractionClass>) -> Option<Self> {
        classes.sort();
        classes.dedup();

        if classes.is_empty() {
            return None;
        }

        let mut unit_to_class =
            BTreeMap::<RecursiveUnit, RecursiveWorldRevisionAbstractionClass>::new();

        for class in &classes {
            for unit in class.members() {
                if unit_to_class.insert(unit.clone(), class.clone()).is_some() {
                    return None;
                }
            }
        }

        Some(Self {
            classes,
            unit_to_class,
        })
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        &self.classes
    }

    pub fn len(&self) -> usize {
        self.classes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    pub fn class_for(
        &self,
        unit: &RecursiveUnit,
    ) -> Option<&RecursiveWorldRevisionAbstractionClass> {
        self.unit_to_class.get(unit)
    }

    pub fn covers(&self, unit: &RecursiveUnit) -> bool {
        self.class_for(unit).is_some()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractObservation {
    premise_classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    conclusion_classes: Vec<RecursiveWorldRevisionAbstractionClass>,
}

impl RecursiveWorldRevisionAbstractObservation {
    pub fn new(
        mut premise_classes: Vec<RecursiveWorldRevisionAbstractionClass>,
        mut conclusion_classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    ) -> Option<Self> {
        premise_classes.sort();
        premise_classes.dedup();

        conclusion_classes.sort();
        conclusion_classes.dedup();

        if premise_classes.is_empty() || conclusion_classes.is_empty() {
            return None;
        }

        Some(Self {
            premise_classes,
            conclusion_classes,
        })
    }

    pub fn premise_classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        &self.premise_classes
    }

    pub fn conclusion_classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        &self.conclusion_classes
    }

    pub fn premise_class_count(&self) -> usize {
        self.premise_classes.len()
    }

    pub fn conclusion_class_count(&self) -> usize {
        self.conclusion_classes.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionProjection {
    vocabulary: RecursiveWorldRevisionAbstractionVocabulary,
    source_observations: RecursiveWorldRevisionInductionObservationSet,
    projected: Vec<(
        RecursiveWorldRevisionDiscoveryObservation,
        RecursiveWorldRevisionAbstractObservation,
    )>,
}

impl RecursiveWorldRevisionAbstractionProjection {
    pub fn project(
        vocabulary: RecursiveWorldRevisionAbstractionVocabulary,
        source_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Option<Self> {
        let mut projected = Vec::new();

        for observation in source_observations.observations() {
            let premise_classes = observation
                .premises()
                .iter()
                .filter_map(|unit| vocabulary.class_for(unit).cloned())
                .collect::<Vec<_>>();

            let conclusion_classes = observation
                .conclusions()
                .iter()
                .filter_map(|unit| vocabulary.class_for(unit).cloned())
                .collect::<Vec<_>>();

            let abstract_observation = RecursiveWorldRevisionAbstractObservation::new(
                premise_classes,
                conclusion_classes,
            )?;

            projected.push((observation.clone(), abstract_observation));
        }

        projected.sort_by(|left, right| left.0.cmp(&right.0));

        Some(Self {
            vocabulary,
            source_observations,
            projected,
        })
    }

    pub fn vocabulary(&self) -> &RecursiveWorldRevisionAbstractionVocabulary {
        &self.vocabulary
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.source_observations
    }

    pub fn projected(
        &self,
    ) -> &[(
        RecursiveWorldRevisionDiscoveryObservation,
        RecursiveWorldRevisionAbstractObservation,
    )] {
        &self.projected
    }

    pub fn len(&self) -> usize {
        self.projected.len()
    }

    pub fn is_empty(&self) -> bool {
        self.projected.is_empty()
    }

    pub fn abstract_observation_for(
        &self,
        source: &RecursiveWorldRevisionDiscoveryObservation,
    ) -> Option<&RecursiveWorldRevisionAbstractObservation> {
        self.projected
            .iter()
            .find_map(|(observation, abstract_observation)| {
                if observation == source {
                    Some(abstract_observation)
                } else {
                    None
                }
            })
    }

    pub fn represented_classes(&self) -> BTreeSet<RecursiveWorldRevisionAbstractionClass> {
        self.projected
            .iter()
            .flat_map(|(_, abstract_observation)| {
                abstract_observation
                    .premise_classes()
                    .iter()
                    .chain(abstract_observation.conclusion_classes().iter())
                    .cloned()
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionProjector;

impl RecursiveWorldRevisionAbstractionProjector {
    pub fn project(
        vocabulary: RecursiveWorldRevisionAbstractionVocabulary,
        source_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Option<RecursiveWorldRevisionAbstractionProjection> {
        RecursiveWorldRevisionAbstractionProjection::project(vocabulary, source_observations)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionConsensus {
    projection: RecursiveWorldRevisionAbstractionProjection,
    premise_classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    conclusion_classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    premise_support: BTreeMap<RecursiveWorldRevisionAbstractionClass, usize>,
    conclusion_support: BTreeMap<RecursiveWorldRevisionAbstractionClass, usize>,
}

impl RecursiveWorldRevisionAbstractionConsensus {
    pub fn derive(projection: RecursiveWorldRevisionAbstractionProjection) -> Option<Self> {
        let observation_count = projection.len();

        let mut premise_support = BTreeMap::<RecursiveWorldRevisionAbstractionClass, usize>::new();

        let mut conclusion_support =
            BTreeMap::<RecursiveWorldRevisionAbstractionClass, usize>::new();

        for (_, abstract_observation) in projection.projected() {
            for class in abstract_observation.premise_classes() {
                let count = premise_support.entry(class.clone()).or_insert(0);

                *count = count.saturating_add(1);
            }

            for class in abstract_observation.conclusion_classes() {
                let count = conclusion_support.entry(class.clone()).or_insert(0);

                *count = count.saturating_add(1);
            }
        }

        let premise_classes = premise_support
            .iter()
            .filter_map(|(class, support)| {
                if *support == observation_count {
                    Some(class.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let conclusion_classes = conclusion_support
            .iter()
            .filter_map(|(class, support)| {
                if *support == observation_count {
                    Some(class.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if premise_classes.is_empty() || conclusion_classes.is_empty() {
            return None;
        }

        Some(Self {
            projection,
            premise_classes,
            conclusion_classes,
            premise_support,
            conclusion_support,
        })
    }

    pub fn projection(&self) -> &RecursiveWorldRevisionAbstractionProjection {
        &self.projection
    }

    pub fn premise_classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        &self.premise_classes
    }

    pub fn conclusion_classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        &self.conclusion_classes
    }

    pub fn premise_support(&self, class: &RecursiveWorldRevisionAbstractionClass) -> usize {
        self.premise_support.get(class).copied().unwrap_or(0)
    }

    pub fn conclusion_support(&self, class: &RecursiveWorldRevisionAbstractionClass) -> usize {
        self.conclusion_support.get(class).copied().unwrap_or(0)
    }

    pub fn observation_count(&self) -> usize {
        self.projection.len()
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.projection.source_observations()
    }

    pub fn vocabulary(&self) -> &RecursiveWorldRevisionAbstractionVocabulary {
        self.projection.vocabulary()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionConsensusBuilder;

impl RecursiveWorldRevisionAbstractionConsensusBuilder {
    pub fn derive(
        projection: RecursiveWorldRevisionAbstractionProjection,
    ) -> Option<RecursiveWorldRevisionAbstractionConsensus> {
        RecursiveWorldRevisionAbstractionConsensus::derive(projection)
    }
}

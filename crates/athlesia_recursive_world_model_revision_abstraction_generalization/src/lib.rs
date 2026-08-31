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

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionVocabulary;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationResolvedClass {
    abstraction_class: RecursiveWorldRevisionAbstractionClass,
    sources: Vec<RecursiveWorldRevisionAbstractionGeneralizedClass>,
}

impl RecursiveWorldRevisionAbstractionGeneralizationResolvedClass {
    fn new(
        abstraction_class: RecursiveWorldRevisionAbstractionClass,
        mut sources: Vec<RecursiveWorldRevisionAbstractionGeneralizedClass>,
    ) -> Self {
        sources.sort();
        sources.dedup();

        Self {
            abstraction_class,
            sources,
        }
    }

    pub fn abstraction_class(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.abstraction_class
    }

    pub fn sources(&self) -> &[RecursiveWorldRevisionAbstractionGeneralizedClass] {
        &self.sources
    }

    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    pub fn supporting_contexts(&self) -> Vec<&RecursiveWorldRevisionAbstractionInductionContext> {
        let mut contexts = BTreeSet::<&RecursiveWorldRevisionAbstractionInductionContext>::new();

        for source in &self.sources {
            contexts.extend(source.supporting_contexts().iter());
        }

        contexts.into_iter().collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationConflict {
    first: RecursiveWorldRevisionAbstractionGeneralizationResolvedClass,
    second: RecursiveWorldRevisionAbstractionGeneralizationResolvedClass,
    overlap: Vec<RecursiveUnit>,
}

impl RecursiveWorldRevisionAbstractionGeneralizationConflict {
    fn new(
        first: RecursiveWorldRevisionAbstractionGeneralizationResolvedClass,
        second: RecursiveWorldRevisionAbstractionGeneralizationResolvedClass,
        mut overlap: Vec<RecursiveUnit>,
    ) -> Self {
        overlap.sort();
        overlap.dedup();

        let (first, second) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };

        Self {
            first,
            second,
            overlap,
        }
    }

    pub fn first(&self) -> &RecursiveWorldRevisionAbstractionGeneralizationResolvedClass {
        &self.first
    }

    pub fn second(&self) -> &RecursiveWorldRevisionAbstractionGeneralizationResolvedClass {
        &self.second
    }

    pub fn overlap(&self) -> &[RecursiveUnit] {
        &self.overlap
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationResolution {
    source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
    resolved_classes: Vec<RecursiveWorldRevisionAbstractionGeneralizationResolvedClass>,
    conflicted_classes: Vec<RecursiveWorldRevisionAbstractionGeneralizationResolvedClass>,
    conflicts: Vec<RecursiveWorldRevisionAbstractionGeneralizationConflict>,
    vocabulary: Option<RecursiveWorldRevisionAbstractionVocabulary>,
}

impl RecursiveWorldRevisionAbstractionGeneralizationResolution {
    pub fn resolve(source: RecursiveWorldRevisionAbstractionGeneralizedClassSet) -> Self {
        let mut grouped = BTreeMap::<
            RecursiveWorldRevisionAbstractionClass,
            Vec<RecursiveWorldRevisionAbstractionGeneralizedClass>,
        >::new();

        for generalized in source.classes() {
            grouped
                .entry(generalized.abstraction_class().clone())
                .or_default()
                .push(generalized.clone());
        }

        let unique_classes = grouped
            .into_iter()
            .map(|(abstraction_class, sources)| {
                RecursiveWorldRevisionAbstractionGeneralizationResolvedClass::new(
                    abstraction_class,
                    sources,
                )
            })
            .collect::<Vec<_>>();

        let mut conflicted_identities = BTreeSet::<RecursiveWorldRevisionAbstractionClass>::new();

        let mut conflicts = Vec::<RecursiveWorldRevisionAbstractionGeneralizationConflict>::new();

        for first_index in 0..unique_classes.len() {
            for second_index in (first_index + 1)..unique_classes.len() {
                let first = &unique_classes[first_index];

                let second = &unique_classes[second_index];

                let first_members = first
                    .abstraction_class()
                    .members()
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>();

                let second_members = second
                    .abstraction_class()
                    .members()
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>();

                let overlap = first_members
                    .intersection(&second_members)
                    .cloned()
                    .collect::<Vec<_>>();

                if overlap.is_empty() {
                    continue;
                }

                conflicted_identities.insert(first.abstraction_class().clone());

                conflicted_identities.insert(second.abstraction_class().clone());

                conflicts.push(
                    RecursiveWorldRevisionAbstractionGeneralizationConflict::new(
                        first.clone(),
                        second.clone(),
                        overlap,
                    ),
                );
            }
        }

        conflicts.sort();
        conflicts.dedup();

        let mut resolved_classes =
            Vec::<RecursiveWorldRevisionAbstractionGeneralizationResolvedClass>::new();

        let mut conflicted_classes =
            Vec::<RecursiveWorldRevisionAbstractionGeneralizationResolvedClass>::new();

        for class in unique_classes {
            if conflicted_identities.contains(class.abstraction_class()) {
                conflicted_classes.push(class);
            } else {
                resolved_classes.push(class);
            }
        }

        resolved_classes.sort();
        conflicted_classes.sort();

        let vocabulary = if resolved_classes.is_empty() {
            None
        } else {
            RecursiveWorldRevisionAbstractionVocabulary::new(
                resolved_classes
                    .iter()
                    .map(|resolved| resolved.abstraction_class().clone())
                    .collect(),
            )
        };

        Self {
            source,
            resolved_classes,
            conflicted_classes,
            conflicts,
            vocabulary,
        }
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionGeneralizedClassSet {
        &self.source
    }

    pub fn resolved_classes(
        &self,
    ) -> &[RecursiveWorldRevisionAbstractionGeneralizationResolvedClass] {
        &self.resolved_classes
    }

    pub fn conflicted_classes(
        &self,
    ) -> &[RecursiveWorldRevisionAbstractionGeneralizationResolvedClass] {
        &self.conflicted_classes
    }

    pub fn conflicts(&self) -> &[RecursiveWorldRevisionAbstractionGeneralizationConflict] {
        &self.conflicts
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.vocabulary.as_ref()
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    pub fn resolved_count(&self) -> usize {
        self.resolved_classes.len()
    }

    pub fn conflicted_count(&self) -> usize {
        self.conflicted_classes.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationResolver;

impl RecursiveWorldRevisionAbstractionGeneralizationResolver {
    pub fn resolve(
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
    ) -> RecursiveWorldRevisionAbstractionGeneralizationResolution {
        RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(source)
    }
}

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionProjection;

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus {
    VocabularyUnavailable,
    ProjectionUnavailable,
    Projected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge {
    resolution: RecursiveWorldRevisionAbstractionGeneralizationResolution,
    application_observations: RecursiveWorldRevisionInductionObservationSet,
    projection: Option<RecursiveWorldRevisionAbstractionProjection>,
    status: RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus,
}

impl RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge {
    pub fn project(
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
        application_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(source);

        let Some(vocabulary) = resolution.vocabulary().cloned() else {
            return Self {
                resolution,
                application_observations,
                projection: None,
                status:
                    RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus::
                        VocabularyUnavailable,
            };
        };

        let projection = RecursiveWorldRevisionAbstractionProjection::project(
            vocabulary,
            application_observations.clone(),
        );

        let status = if projection.is_some() {
            RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus::Projected
        } else {
            RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus::ProjectionUnavailable
        };

        Self {
            resolution,
            application_observations,
            projection,
            status,
        }
    }

    pub fn resolution(&self) -> &RecursiveWorldRevisionAbstractionGeneralizationResolution {
        &self.resolution
    }

    pub fn generalized_source(&self) -> &RecursiveWorldRevisionAbstractionGeneralizedClassSet {
        self.resolution.source()
    }

    pub fn application_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.application_observations
    }

    pub fn projection(&self) -> Option<&RecursiveWorldRevisionAbstractionProjection> {
        self.projection.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus {
        self.status
    }

    pub fn is_projected(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus::Projected
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.resolution.vocabulary()
    }

    pub fn conflicts(&self) -> &[RecursiveWorldRevisionAbstractionGeneralizationConflict] {
        self.resolution.conflicts()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationProjector;

impl RecursiveWorldRevisionAbstractionGeneralizationProjector {
    pub fn project(
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
        application_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge {
        RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
            source,
            application_observations,
        )
    }
}

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionConsensus;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus {
    ProjectionUnavailable,
    ConsensusUnavailable,
    ConsensusDerived,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge {
    projection_bridge: RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge,
    consensus: Option<RecursiveWorldRevisionAbstractionConsensus>,
    status: RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus,
}

impl RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge {
    pub fn derive(
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
        application_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        let projection_bridge =
            RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
                source,
                application_observations,
            );

        let Some(projection) = projection_bridge.projection().cloned() else {
            return Self {
                projection_bridge,
                consensus: None,
                status:
                    RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus::
                        ProjectionUnavailable,
            };
        };

        let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection);

        let status = if consensus.is_some() {
            RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus::ConsensusDerived
        } else {
            RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus::ConsensusUnavailable
        };

        Self {
            projection_bridge,
            consensus,
            status,
        }
    }

    pub fn projection_bridge(
        &self,
    ) -> &RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge {
        &self.projection_bridge
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.consensus.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus {
        self.status
    }

    pub fn is_consensus_derived(&self) -> bool {
        self.status
            == RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus::ConsensusDerived
    }

    pub fn generalized_source(&self) -> &RecursiveWorldRevisionAbstractionGeneralizedClassSet {
        self.projection_bridge.generalized_source()
    }

    pub fn application_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.projection_bridge.application_observations()
    }

    pub fn resolution(&self) -> &RecursiveWorldRevisionAbstractionGeneralizationResolution {
        self.projection_bridge.resolution()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.projection_bridge.vocabulary()
    }

    pub fn projection(&self) -> Option<&RecursiveWorldRevisionAbstractionProjection> {
        self.projection_bridge.projection()
    }

    pub fn conflicts(&self) -> &[RecursiveWorldRevisionAbstractionGeneralizationConflict] {
        self.projection_bridge.conflicts()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationConsensusBuilder;

impl RecursiveWorldRevisionAbstractionGeneralizationConsensusBuilder {
    pub fn derive(
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
        application_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge {
        RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
            source,
            application_observations,
        )
    }
}

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractionRealization,
    RecursiveWorldRevisionAbstractionRealizationStatus,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus {
    ConsensusUnavailable,
    Ambiguous,
    Deterministic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge {
    consensus_bridge: RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge,
    realization: Option<RecursiveWorldRevisionAbstractionRealization>,
    status: RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus,
}

impl RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge {
    pub fn realize(
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
        application_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        let consensus_bridge =
            RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
                source,
                application_observations,
            );

        let Some(consensus) = consensus_bridge.consensus().cloned() else {
            return Self {
                consensus_bridge,
                realization: None,
                status:
                    RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::
                        ConsensusUnavailable,
            };
        };

        let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus);

        let status = match realization.status() {
            RecursiveWorldRevisionAbstractionRealizationStatus::Ambiguous => {
                RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::Ambiguous
            }

            RecursiveWorldRevisionAbstractionRealizationStatus::Deterministic => {
                RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::Deterministic
            }
        };

        Self {
            consensus_bridge,
            realization: Some(realization),
            status,
        }
    }

    pub fn consensus_bridge(
        &self,
    ) -> &RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge {
        &self.consensus_bridge
    }

    pub fn realization(&self) -> Option<&RecursiveWorldRevisionAbstractionRealization> {
        self.realization.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus {
        self.status
    }

    pub fn is_ambiguous(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::Ambiguous
    }

    pub fn is_deterministic(&self) -> bool {
        self.status
            == RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::Deterministic
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.realization
            .as_ref()
            .and_then(RecursiveWorldRevisionAbstractionRealization::realized_observation)
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.consensus_bridge.consensus()
    }

    pub fn generalized_source(&self) -> &RecursiveWorldRevisionAbstractionGeneralizedClassSet {
        self.consensus_bridge.generalized_source()
    }

    pub fn application_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.consensus_bridge.application_observations()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.consensus_bridge.vocabulary()
    }

    pub fn premise_witnesses(
        &self,
        class: &RecursiveWorldRevisionAbstractionClass,
    ) -> &[RecursiveUnit] {
        self.realization
            .as_ref()
            .map(|realization| realization.premise_witnesses(class))
            .unwrap_or(&[])
    }

    pub fn conclusion_witnesses(
        &self,
        class: &RecursiveWorldRevisionAbstractionClass,
    ) -> &[RecursiveUnit] {
        self.realization
            .as_ref()
            .map(|realization| realization.conclusion_witnesses(class))
            .unwrap_or(&[])
    }

    pub fn conflicts(&self) -> &[RecursiveWorldRevisionAbstractionGeneralizationConflict] {
        self.consensus_bridge.conflicts()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationRealizer;

impl RecursiveWorldRevisionAbstractionGeneralizationRealizer {
    pub fn realize(
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
        application_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge {
        RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
            source,
            application_observations,
        )
    }
}

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryHypothesis;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus {
    RealizationUnavailable,
    DiscoveryUnavailable,
    Discovered,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge {
    target: RecursiveWorldRule,
    realization: RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge,
    hypothesis: Option<RecursiveWorldRevisionDiscoveryHypothesis>,
    status: RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus,
}

impl RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge {
    pub fn discover(
        target: RecursiveWorldRule,
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
        application_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        let realization = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
            source,
            application_observations,
        );

        let Some(realized_observation) = realization.realized_observation().cloned() else {
            return Self {
                target,
                realization,
                hypothesis: None,
                status:
                    RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus::
                        RealizationUnavailable,
            };
        };

        let hypothesis = RecursiveWorldRevisionDiscoveryHypothesis::discover(
            target.clone(),
            realized_observation,
        );

        let status = if hypothesis.is_some() {
            RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus::Discovered
        } else {
            RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus::DiscoveryUnavailable
        };

        Self {
            target,
            realization,
            hypothesis,
            status,
        }
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn realization(&self) -> &RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge {
        &self.realization
    }

    pub fn hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.hypothesis.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus {
        self.status
    }

    pub fn is_discovered(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus::Discovered
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.realization.realized_observation()
    }

    pub fn replacement(&self) -> Option<&RecursiveWorldRule> {
        self.hypothesis
            .as_ref()
            .map(RecursiveWorldRevisionDiscoveryHypothesis::replacement)
    }

    pub fn generalized_source(&self) -> &RecursiveWorldRevisionAbstractionGeneralizedClassSet {
        self.realization.generalized_source()
    }

    pub fn application_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.realization.application_observations()
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.realization.consensus()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.realization.vocabulary()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBuilder;

impl RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBuilder {
    pub fn discover(
        target: RecursiveWorldRule,
        source: RecursiveWorldRevisionAbstractionGeneralizedClassSet,
        application_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge {
        RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
            target,
            source,
            application_observations,
        )
    }
}

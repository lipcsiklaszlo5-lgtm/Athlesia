use std::collections::{BTreeMap, BTreeSet};

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionCompositionPathSelection,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold {
    min_context_support: usize,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold {
    pub fn new(min_context_support: usize) -> Option<Self> {
        if min_context_support < 2 {
            return None;
        }

        Some(Self {
            min_context_support,
        })
    }

    pub fn min_context_support(&self) -> usize {
        self.min_context_support
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif {
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif {
    pub fn new(classes: Vec<RecursiveWorldRevisionAbstractionClass>) -> Option<Self> {
        if classes.len() != 3 {
            return None;
        }

        let distinct: BTreeSet<RecursiveWorldRevisionAbstractionClass> =
            classes.iter().cloned().collect();

        if distinct.len() != 3 {
            return None;
        }

        Some(Self { classes })
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        &self.classes
    }

    pub fn start(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.classes[0]
    }

    pub fn middle(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.classes[1]
    }

    pub fn end(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.classes[2]
    }

    pub fn edge_count(&self) -> usize {
        2
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationSource {
    contexts: Vec<RecursiveWorldRevisionAbstractionCompositionPathSelectionSet>,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationSource {
    pub fn new(
        mut contexts: Vec<RecursiveWorldRevisionAbstractionCompositionPathSelectionSet>,
    ) -> Option<Self> {
        contexts.sort_by(|left, right| left.selections().cmp(right.selections()));

        contexts.dedup();

        if contexts.len() < 2 {
            return None;
        }

        Some(Self { contexts })
    }

    pub fn contexts(&self) -> &[RecursiveWorldRevisionAbstractionCompositionPathSelectionSet] {
        &self.contexts
    }

    pub fn len(&self) -> usize {
        self.contexts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif {
    motif: RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
    threshold: RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
    supporting_contexts: Vec<RecursiveWorldRevisionAbstractionCompositionPathSelectionSet>,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif {
    pub fn motif(&self) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif {
        &self.motif
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold {
        self.threshold
    }

    pub fn supporting_contexts(
        &self,
    ) -> &[RecursiveWorldRevisionAbstractionCompositionPathSelectionSet] {
        &self.supporting_contexts
    }

    pub fn support_count(&self) -> usize {
        self.supporting_contexts.len()
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        self.motif.classes()
    }
}

impl Ord for RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.motif
            .cmp(&other.motif)
            .then_with(|| self.threshold.cmp(&other.threshold))
            .then_with(|| self.support_count().cmp(&other.support_count()))
    }
}

impl PartialOrd for RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet {
    source: RecursiveWorldRevisionAbstractionCompositionGeneralizationSource,
    threshold: RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
    motifs: Vec<RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif>,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet {
    pub fn generalize(
        source: RecursiveWorldRevisionAbstractionCompositionGeneralizationSource,
        threshold: RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
    ) -> Option<Self> {
        let mut supports: BTreeMap<
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
            Vec<RecursiveWorldRevisionAbstractionCompositionPathSelectionSet>,
        > = BTreeMap::new();

        for context in source.contexts() {
            let mut context_motifs: BTreeSet<
                RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
            > = BTreeSet::new();

            for selection in context.selections() {
                Self::collect_selection_motifs(selection, &mut context_motifs);
            }

            for motif in context_motifs {
                supports.entry(motif).or_default().push(context.clone());
            }
        }

        let mut motifs = Vec::new();

        for (motif, supporting_contexts) in supports {
            if supporting_contexts.len() < threshold.min_context_support() {
                continue;
            }

            motifs.push(
                RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif {
                    motif,
                    threshold,
                    supporting_contexts,
                },
            );
        }

        motifs.sort();

        if motifs.is_empty() {
            return None;
        }

        Some(Self {
            source,
            threshold,
            motifs,
        })
    }

    fn collect_selection_motifs(
        selection: &RecursiveWorldRevisionAbstractionCompositionPathSelection,
        motifs: &mut BTreeSet<RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif>,
    ) {
        for classes in selection.path().classes().windows(3) {
            if let Some(motif) =
                RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(
                    classes.to_vec(),
                )
            {
                motifs.insert(motif);
            }
        }
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationSource {
        &self.source
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold {
        self.threshold
    }

    pub fn motifs(&self) -> &[RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif] {
        &self.motifs
    }

    pub fn len(&self) -> usize {
        self.motifs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.motifs.is_empty()
    }

    pub fn motif(
        &self,
        classes: &[RecursiveWorldRevisionAbstractionClass],
    ) -> Option<&RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif> {
        self.motifs.iter().find(|motif| motif.classes() == classes)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizer;

impl RecursiveWorldRevisionAbstractionCompositionGeneralizer {
    pub fn generalize(
        source: RecursiveWorldRevisionAbstractionCompositionGeneralizationSource,
        threshold: RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
    ) -> Option<RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet> {
        RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
            source, threshold,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict {
    first: RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
    second: RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
    shared_positions: Vec<usize>,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict {
    pub fn between(
        first: RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
        second: RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
    ) -> Option<Self> {
        if first == second {
            return None;
        }

        let shared_positions: Vec<usize> = first
            .classes()
            .iter()
            .zip(second.classes().iter())
            .enumerate()
            .filter_map(
                |(index, (left, right))| {
                    if left == right {
                        Some(index)
                    } else {
                        None
                    }
                },
            )
            .collect();

        if shared_positions.len() != 2 {
            return None;
        }

        let (first, second) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };

        Some(Self {
            first,
            second,
            shared_positions,
        })
    }

    pub fn first(&self) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif {
        &self.first
    }

    pub fn second(&self) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif {
        &self.second
    }

    pub fn shared_positions(&self) -> &[usize] {
        &self.shared_positions
    }

    pub fn shares_start_middle(&self) -> bool {
        self.shared_positions == [0, 1]
    }

    pub fn shares_middle_end(&self) -> bool {
        self.shared_positions == [1, 2]
    }

    pub fn shares_start_end(&self) -> bool {
        self.shared_positions == [0, 2]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution {
    source: RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet,
    resolved_motifs: Vec<RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif>,
    conflicted_motifs: Vec<RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif>,
    conflicts: Vec<RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict>,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution {
    pub fn resolve(
        source: RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet,
    ) -> Self {
        let motifs = source.motifs();

        let mut conflicts: BTreeSet<
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict,
        > = BTreeSet::new();

        let mut conflicted_identities: BTreeSet<
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
        > = BTreeSet::new();

        for first_index in 0..motifs.len() {
            for second_index in (first_index + 1)..motifs.len() {
                let first = motifs[first_index].motif().clone();

                let second = motifs[second_index].motif().clone();

                if let Some(conflict) =
                    RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict::between(
                        first.clone(),
                        second.clone(),
                    )
                {
                    conflicted_identities.insert(first);

                    conflicted_identities.insert(second);

                    conflicts.insert(conflict);
                }
            }
        }

        let mut resolved_motifs = Vec::new();

        let mut conflicted_motifs = Vec::new();

        for motif in motifs {
            if conflicted_identities.contains(motif.motif()) {
                conflicted_motifs.push(motif.clone());
            } else {
                resolved_motifs.push(motif.clone());
            }
        }

        resolved_motifs.sort();
        resolved_motifs.dedup();

        conflicted_motifs.sort();
        conflicted_motifs.dedup();

        Self {
            source,
            resolved_motifs,
            conflicted_motifs,
            conflicts: conflicts.into_iter().collect(),
        }
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet {
        &self.source
    }

    pub fn resolved_motifs(
        &self,
    ) -> &[RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif] {
        &self.resolved_motifs
    }

    pub fn conflicted_motifs(
        &self,
    ) -> &[RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif] {
        &self.conflicted_motifs
    }

    pub fn conflicts(
        &self,
    ) -> &[RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict] {
        &self.conflicts
    }

    pub fn resolved_len(&self) -> usize {
        self.resolved_motifs.len()
    }

    pub fn conflicted_len(&self) -> usize {
        self.conflicted_motifs.len()
    }

    pub fn conflict_len(&self) -> usize {
        self.conflicts.len()
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    pub fn resolved_motif(
        &self,
        classes: &[RecursiveWorldRevisionAbstractionClass],
    ) -> Option<&RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif> {
        self.resolved_motifs
            .iter()
            .find(|motif| motif.classes() == classes)
    }

    pub fn conflicted_motif(
        &self,
        classes: &[RecursiveWorldRevisionAbstractionClass],
    ) -> Option<&RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif> {
        self.conflicted_motifs
            .iter()
            .find(|motif| motif.classes() == classes)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationResolver;

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationResolver {
    pub fn resolve(
        source: RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution {
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(source)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus {
    NoResolvedMotifs,
    NoApplicationMatch,
    Projected,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
    motif:
        RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif,
    matching_selections:
        Vec<
            athlesia_recursive_world_model_revision_abstraction_composition::
                RecursiveWorldRevisionAbstractionCompositionPathSelection,
        >,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
    pub fn motif(&self) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif {
        &self.motif
    }

    pub fn matching_selections(
        &self,
    ) -> &[
        athlesia_recursive_world_model_revision_abstraction_composition::
            RecursiveWorldRevisionAbstractionCompositionPathSelection
    ]{
        &self.matching_selections
    }

    pub fn match_count(&self) -> usize {
        self.matching_selections.len()
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        self.motif.classes()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection {
    resolution:
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
    application:
        athlesia_recursive_world_model_revision_abstraction_composition::
            RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    projected_motifs:
        Vec<
            RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
        >,
    status:
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection {
    pub fn project(
        resolution: RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
        application:
            athlesia_recursive_world_model_revision_abstraction_composition::
                RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    ) -> Self {
        if resolution.resolved_motifs().is_empty() {
            return Self {
                resolution,
                application,
                projected_motifs:
                    Vec::new(),
                status:
                    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus::
                        NoResolvedMotifs,
            };
        }

        let mut projected_motifs = Vec::new();

        for motif in resolution.resolved_motifs() {
            let mut matching_selections:
                BTreeSet<
                    athlesia_recursive_world_model_revision_abstraction_composition::
                        RecursiveWorldRevisionAbstractionCompositionPathSelection,
                > =
                BTreeSet::new();

            for selection in application.selections() {
                let matched = selection
                    .path()
                    .classes()
                    .windows(3)
                    .any(|window| window == motif.classes());

                if matched {
                    matching_selections.insert(selection.clone());
                }
            }

            if matching_selections.is_empty() {
                continue;
            }

            projected_motifs.push(
                RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
                    motif: motif.clone(),
                    matching_selections: matching_selections.into_iter().collect(),
                },
            );
        }

        projected_motifs.sort();
        projected_motifs.dedup();

        let status = if projected_motifs.is_empty() {
            RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus::
                    NoApplicationMatch
        } else {
            RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus::Projected
        };

        Self {
            resolution,
            application,
            projected_motifs,
            status,
        }
    }

    pub fn resolution(
        &self,
    ) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution {
        &self.resolution
    }

    pub fn application(
        &self,
    ) -> &athlesia_recursive_world_model_revision_abstraction_composition::
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet{
        &self.application
    }

    pub fn projected_motifs(
        &self,
    ) -> &[RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif] {
        &self.projected_motifs
    }

    pub fn status(
        &self,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectionStatus {
        self.status
    }

    pub fn len(&self) -> usize {
        self.projected_motifs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.projected_motifs.is_empty()
    }

    pub fn projected_motif(
        &self,
        classes: &[RecursiveWorldRevisionAbstractionClass],
    ) -> Option<&RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif> {
        self.projected_motifs
            .iter()
            .find(|motif| motif.classes() == classes)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationProjector;

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationProjector {
    pub fn project(
        resolution: RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
        application:
            athlesia_recursive_world_model_revision_abstraction_composition::
                RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection {
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
            resolution,
            application,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus {
    Unavailable,
    Ambiguous,
    Deterministic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization {
    projected_motif:
        RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
    application_observations:
        Vec<
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryObservation,
        >,
    premise_witnesses:
        Vec<
            athlesia_recursive::RecursiveUnit,
        >,
    conclusion_witnesses:
        Vec<
            athlesia_recursive::RecursiveUnit,
        >,
    realized_observation:
        Option<
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryObservation,
        >,
    status:
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization {
    pub fn realize(
        projected_motif: RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
        mut application_observations:
            Vec<
                athlesia_recursive_world_model_revision_discovery::
                    RecursiveWorldRevisionDiscoveryObservation,
            >,
    ) -> Self {
        application_observations.sort();
        application_observations.dedup();

        let start_class = projected_motif
            .classes()
            .first()
            .expect("generalized composition motif must have start class");

        let end_class = projected_motif
            .classes()
            .last()
            .expect("generalized composition motif must have end class");

        let mut premise_witnesses: BTreeSet<athlesia_recursive::RecursiveUnit> = BTreeSet::new();

        let mut conclusion_witnesses: BTreeSet<athlesia_recursive::RecursiveUnit> = BTreeSet::new();

        for observation in &application_observations {
            for unit in observation.premises() {
                if start_class.contains(unit) {
                    premise_witnesses.insert(unit.clone());
                }
            }

            for unit in observation.conclusions() {
                if end_class.contains(unit) {
                    conclusion_witnesses.insert(unit.clone());
                }
            }
        }

        let premise_witnesses: Vec<athlesia_recursive::RecursiveUnit> =
            premise_witnesses.into_iter().collect();

        let conclusion_witnesses: Vec<athlesia_recursive::RecursiveUnit> =
            conclusion_witnesses.into_iter().collect();

        let (realized_observation, status) = if premise_witnesses.is_empty()
            || conclusion_witnesses.is_empty()
        {
            (
                    None,
                    RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::
                        Unavailable,
                )
        } else if premise_witnesses.len() == 1 && conclusion_witnesses.len() == 1 {
            let realized =
                    athlesia_recursive_world_model_revision_discovery::
                        RecursiveWorldRevisionDiscoveryObservation::new(
                            vec![
                                premise_witnesses[
                                    0
                                ]
                                .clone(),
                            ],
                            vec![
                                conclusion_witnesses[
                                    0
                                ]
                                .clone(),
                            ],
                        );

            match realized {
                    Some(
                        observation,
                    ) =>
                    {
                        (
                            Some(
                                observation,
                            ),
                            RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::
                                Deterministic,
                        )
                    }

                    None =>
                    {
                        (
                            None,
                            RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::
                                Unavailable,
                        )
                    }
                }
        } else {
            (
                    None,
                    RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::
                        Ambiguous,
                )
        };

        Self {
            projected_motif,
            application_observations,
            premise_witnesses,
            conclusion_witnesses,
            realized_observation,
            status,
        }
    }

    pub fn projected_motif(
        &self,
    ) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
        &self.projected_motif
    }

    pub fn motif(&self) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif {
        self.projected_motif.motif()
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        self.projected_motif.classes()
    }

    pub fn application_observations(
        &self,
    ) -> &[
        athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryObservation
    ]{
        &self.application_observations
    }

    pub fn premise_witnesses(&self) -> &[athlesia_recursive::RecursiveUnit] {
        &self.premise_witnesses
    }

    pub fn conclusion_witnesses(&self) -> &[athlesia_recursive::RecursiveUnit] {
        &self.conclusion_witnesses
    }

    pub fn realized_observation(
        &self,
    ) -> Option<
        &athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryObservation,
    >{
        self.realized_observation.as_ref()
    }

    pub fn status(
        &self,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus {
        self.status
    }

    pub fn is_deterministic(&self) -> bool {
        self.status
            == RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::
                Deterministic
    }

    pub fn is_ambiguous(&self) -> bool {
        self.status
            == RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::
                Ambiguous
    }

    pub fn start_class(&self) -> &RecursiveWorldRevisionAbstractionClass {
        self.classes()
            .first()
            .expect("generalized composition motif must have start class")
    }

    pub fn middle_class(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.classes()[1]
    }

    pub fn end_class(&self) -> &RecursiveWorldRevisionAbstractionClass {
        self.classes()
            .last()
            .expect("generalized composition motif must have end class")
    }

    pub fn support_count(&self) -> usize {
        self.motif().support_count()
    }

    pub fn matching_selections(
        &self,
    ) -> &[
        athlesia_recursive_world_model_revision_abstraction_composition::
            RecursiveWorldRevisionAbstractionCompositionPathSelection
    ]{
        self.projected_motif.matching_selections()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizer;

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizer {
    pub fn realize(
        projected_motif: RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
        application_observations:
            Vec<
                athlesia_recursive_world_model_revision_discovery::
                    RecursiveWorldRevisionDiscoveryObservation,
            >,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization {
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected_motif,
            application_observations,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus {
    RealizationUnavailable,
    DiscoveryUnavailable,
    Discovered,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge {
    target:
        athlesia_recursive_world_model::RecursiveWorldRule,
    realization:
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization,
    hypothesis:
        Option<
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryHypothesis,
        >,
    status:
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge {
    pub fn discover(
        target: athlesia_recursive_world_model::RecursiveWorldRule,
        projected_motif: RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
        application_observations:
            Vec<
                athlesia_recursive_world_model_revision_discovery::
                    RecursiveWorldRevisionDiscoveryObservation,
            >,
    ) -> Self {
        let realization =
            RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
                projected_motif,
                application_observations,
            );

        let Some(realized_observation) = realization.realized_observation().cloned() else {
            return Self {
                target,
                realization,
                hypothesis: None,
                status:
                    RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus::
                        RealizationUnavailable,
            };
        };

        let hypothesis =
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryHypothesis::
                    discover(
                        target.clone(),
                        realized_observation,
                    );

        let status = if hypothesis.is_some() {
            RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus::Discovered
        } else {
            RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus::
                    DiscoveryUnavailable
        };

        Self {
            target,
            realization,
            hypothesis,
            status,
        }
    }

    pub fn target(&self) -> &athlesia_recursive_world_model::RecursiveWorldRule {
        &self.target
    }

    pub fn realization(
        &self,
    ) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization {
        &self.realization
    }

    pub fn hypothesis(
        &self,
    ) -> Option<
        &athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryHypothesis,
    >{
        self.hypothesis.as_ref()
    }

    pub fn status(
        &self,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus {
        self.status
    }

    pub fn is_discovered(&self) -> bool {
        self.status
            == RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus::Discovered
    }

    pub fn realized_observation(
        &self,
    ) -> Option<
        &athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryObservation,
    >{
        self.realization.realized_observation()
    }

    pub fn replacement(&self) -> Option<&athlesia_recursive_world_model::RecursiveWorldRule> {
        self.hypothesis
            .as_ref()
            .map(|hypothesis| hypothesis.replacement())
    }

    pub fn projected_motif(
        &self,
    ) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
        self.realization.projected_motif()
    }

    pub fn motif(&self) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizedMotif {
        self.realization.motif()
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        self.realization.classes()
    }

    pub fn support_count(&self) -> usize {
        self.realization.support_count()
    }

    pub fn matching_selections(
        &self,
    ) -> &[
        athlesia_recursive_world_model_revision_abstraction_composition::
            RecursiveWorldRevisionAbstractionCompositionPathSelection
    ]{
        self.realization.matching_selections()
    }

    pub fn application_observations(
        &self,
    ) -> &[
        athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryObservation
    ]{
        self.realization.application_observations()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBuilder;

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBuilder {
    pub fn discover(
        target: athlesia_recursive_world_model::RecursiveWorldRule,
        projected_motif: RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
        application_observations:
            Vec<
                athlesia_recursive_world_model_revision_discovery::
                    RecursiveWorldRevisionDiscoveryObservation,
            >,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge {
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            target,
            projected_motif,
            application_observations,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus {
    DiscoveryUnavailable,
    Rejected,
    Accepted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation {
    model:
        athlesia_recursive_world_model::RecursiveWorldModel,
    discovery:
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge,
    validation:
        Option<
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryValidation,
        >,
    status:
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus,
}

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation {
    pub fn validate(
        model: athlesia_recursive_world_model::RecursiveWorldModel,
        target: athlesia_recursive_world_model::RecursiveWorldRule,
        projected_motif: RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
        application_observations:
            Vec<
                athlesia_recursive_world_model_revision_discovery::
                    RecursiveWorldRevisionDiscoveryObservation,
            >,
    ) -> Self {
        let discovery =
            RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
                target,
                projected_motif,
                application_observations,
            );

        let Some(hypothesis) = discovery.hypothesis().cloned() else {
            return Self {
                model,
                discovery,
                validation: None,
                status:
                    RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::
                        DiscoveryUnavailable,
            };
        };

        let hypothesis_set =
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryHypothesisSet::new(
                    vec![
                        hypothesis,
                    ],
                );

        let validation =
            athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryValidator::validate(
                    &model,
                    hypothesis_set,
                );

        let status = if validation.accepted_count() == 1 {
            RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::Accepted
        } else {
            RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::Rejected
        };

        Self {
            model,
            discovery,
            validation: Some(validation),
            status,
        }
    }

    pub fn model(&self) -> &athlesia_recursive_world_model::RecursiveWorldModel {
        &self.model
    }

    pub fn discovery(
        &self,
    ) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge {
        &self.discovery
    }

    pub fn validation_result(
        &self,
    ) -> Option<
        &athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryValidation,
    >{
        self.validation.as_ref()
    }

    pub fn status(
        &self,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus {
        self.status
    }

    pub fn is_accepted(&self) -> bool {
        self.status
            == RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::Accepted
    }

    pub fn is_rejected(&self) -> bool {
        self.status
            == RecursiveWorldRevisionAbstractionCompositionGeneralizationValidationStatus::Rejected
    }

    pub fn accepted_hypothesis(
        &self,
    ) -> Option<
        &athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryHypothesis,
    >{
        if self.is_accepted() {
            self.discovery.hypothesis()
        } else {
            None
        }
    }

    pub fn rejected_hypothesis(
        &self,
    ) -> Option<
        &athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryHypothesis,
    >{
        if self.is_rejected() {
            self.discovery.hypothesis()
        } else {
            None
        }
    }

    pub fn target(&self) -> &athlesia_recursive_world_model::RecursiveWorldRule {
        self.discovery.target()
    }

    pub fn hypothesis(
        &self,
    ) -> Option<
        &athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryHypothesis,
    >{
        self.discovery.hypothesis()
    }

    pub fn replacement(&self) -> Option<&athlesia_recursive_world_model::RecursiveWorldRule> {
        self.discovery.replacement()
    }

    pub fn realized_observation(
        &self,
    ) -> Option<
        &athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryObservation,
    >{
        self.discovery.realized_observation()
    }

    pub fn projected_motif(
        &self,
    ) -> &RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
        self.discovery.projected_motif()
    }

    pub fn support_count(&self) -> usize {
        self.discovery.support_count()
    }

    pub fn matching_selections(
        &self,
    ) -> &[
        athlesia_recursive_world_model_revision_abstraction_composition::
            RecursiveWorldRevisionAbstractionCompositionPathSelection
    ]{
        self.discovery.matching_selections()
    }

    pub fn application_observations(
        &self,
    ) -> &[
        athlesia_recursive_world_model_revision_discovery::
            RecursiveWorldRevisionDiscoveryObservation
    ]{
        self.discovery.application_observations()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionGeneralizationValidator;

impl RecursiveWorldRevisionAbstractionCompositionGeneralizationValidator {
    pub fn validate(
        model: athlesia_recursive_world_model::RecursiveWorldModel,
        target: athlesia_recursive_world_model::RecursiveWorldRule,
        projected_motif: RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
        application_observations:
            Vec<
                athlesia_recursive_world_model_revision_discovery::
                    RecursiveWorldRevisionDiscoveryObservation,
            >,
    ) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation {
        RecursiveWorldRevisionAbstractionCompositionGeneralizationValidation::validate(
            model,
            target,
            projected_motif,
            application_observations,
        )
    }
}

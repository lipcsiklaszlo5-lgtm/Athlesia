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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionPath {
    edges: Vec<RecursiveWorldRevisionAbstractionCompositionEdge>,
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
}

impl RecursiveWorldRevisionAbstractionCompositionPath {
    pub fn new(edges: Vec<RecursiveWorldRevisionAbstractionCompositionEdge>) -> Option<Self> {
        if edges.len() < 2 {
            return None;
        }

        for pair in edges.windows(2) {
            if pair[0].to() != pair[1].from() {
                return None;
            }
        }

        let mut classes = Vec::with_capacity(edges.len() + 1);

        classes.push(edges[0].from().clone());

        for edge in &edges {
            classes.push(edge.to().clone());
        }

        let distinct: BTreeSet<RecursiveWorldRevisionAbstractionClass> =
            classes.iter().cloned().collect();

        if distinct.len() != classes.len() {
            return None;
        }

        Some(Self { edges, classes })
    }

    pub fn edges(&self) -> &[RecursiveWorldRevisionAbstractionCompositionEdge] {
        &self.edges
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        &self.classes
    }

    pub fn start(&self) -> &RecursiveWorldRevisionAbstractionClass {
        self.classes
            .first()
            .expect("composition path always has start class")
    }

    pub fn end(&self) -> &RecursiveWorldRevisionAbstractionClass {
        self.classes
            .last()
            .expect("composition path always has end class")
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn class_count(&self) -> usize {
        self.classes.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathSet {
    source: RecursiveWorldRevisionAbstractionComposition,
    paths: Vec<RecursiveWorldRevisionAbstractionCompositionPath>,
}

impl RecursiveWorldRevisionAbstractionCompositionPathSet {
    pub fn induce(source: RecursiveWorldRevisionAbstractionComposition) -> Option<Self> {
        let mut paths: BTreeSet<RecursiveWorldRevisionAbstractionCompositionPath> = BTreeSet::new();

        for edge in source.edges() {
            let mut current_edges = vec![edge.clone()];

            let mut visited: BTreeSet<RecursiveWorldRevisionAbstractionClass> = BTreeSet::new();

            visited.insert(edge.from().clone());

            visited.insert(edge.to().clone());

            Self::extend_paths(
                &source,
                edge.to(),
                &mut current_edges,
                &mut visited,
                &mut paths,
            );
        }

        if paths.is_empty() {
            return None;
        }

        Some(Self {
            source,
            paths: paths.into_iter().collect(),
        })
    }

    fn extend_paths(
        source: &RecursiveWorldRevisionAbstractionComposition,
        current: &RecursiveWorldRevisionAbstractionClass,
        current_edges: &mut Vec<RecursiveWorldRevisionAbstractionCompositionEdge>,
        visited: &mut BTreeSet<RecursiveWorldRevisionAbstractionClass>,
        paths: &mut BTreeSet<RecursiveWorldRevisionAbstractionCompositionPath>,
    ) {
        for next_edge in source.edges() {
            if next_edge.from() != current {
                continue;
            }

            if visited.contains(next_edge.to()) {
                continue;
            }

            current_edges.push(next_edge.clone());

            visited.insert(next_edge.to().clone());

            if current_edges.len() >= 2 {
                if let Some(path) =
                    RecursiveWorldRevisionAbstractionCompositionPath::new(current_edges.clone())
                {
                    paths.insert(path);
                }
            }

            Self::extend_paths(source, next_edge.to(), current_edges, visited, paths);

            visited.remove(next_edge.to());

            current_edges.pop();
        }
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionComposition {
        &self.source
    }

    pub fn paths(&self) -> &[RecursiveWorldRevisionAbstractionCompositionPath] {
        &self.paths
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn paths_from_to(
        &self,
        from: &RecursiveWorldRevisionAbstractionClass,
        to: &RecursiveWorldRevisionAbstractionClass,
    ) -> Vec<&RecursiveWorldRevisionAbstractionCompositionPath> {
        self.paths
            .iter()
            .filter(|path| path.start() == from && path.end() == to)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathInducer;

impl RecursiveWorldRevisionAbstractionCompositionPathInducer {
    pub fn induce(
        source: RecursiveWorldRevisionAbstractionComposition,
    ) -> Option<RecursiveWorldRevisionAbstractionCompositionPathSet> {
        RecursiveWorldRevisionAbstractionCompositionPathSet::induce(source)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathEdgeSupport {
    edge: RecursiveWorldRevisionAbstractionCompositionEdge,
    support_count: usize,
}

impl RecursiveWorldRevisionAbstractionCompositionPathEdgeSupport {
    pub fn from_edge(edge: RecursiveWorldRevisionAbstractionCompositionEdge) -> Self {
        let support_count = edge.support_count();

        Self {
            edge,
            support_count,
        }
    }

    pub fn edge(&self) -> &RecursiveWorldRevisionAbstractionCompositionEdge {
        &self.edge
    }

    pub fn support_count(&self) -> usize {
        self.support_count
    }

    pub fn supporting_observations(&self) -> &[RecursiveWorldRevisionDiscoveryObservation] {
        self.edge.supporting_observations()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathSupport {
    path: RecursiveWorldRevisionAbstractionCompositionPath,
    edge_supports: Vec<RecursiveWorldRevisionAbstractionCompositionPathEdgeSupport>,
    minimum_support: usize,
}

impl RecursiveWorldRevisionAbstractionCompositionPathSupport {
    pub fn derive(path: RecursiveWorldRevisionAbstractionCompositionPath) -> Self {
        let edge_supports: Vec<RecursiveWorldRevisionAbstractionCompositionPathEdgeSupport> = path
            .edges()
            .iter()
            .cloned()
            .map(RecursiveWorldRevisionAbstractionCompositionPathEdgeSupport::from_edge)
            .collect();

        let minimum_support = edge_supports
            .iter()
            .map(RecursiveWorldRevisionAbstractionCompositionPathEdgeSupport::support_count)
            .min()
            .expect("composition path contains at least two edges");

        Self {
            path,
            edge_supports,
            minimum_support,
        }
    }

    pub fn path(&self) -> &RecursiveWorldRevisionAbstractionCompositionPath {
        &self.path
    }

    pub fn edge_supports(&self) -> &[RecursiveWorldRevisionAbstractionCompositionPathEdgeSupport] {
        &self.edge_supports
    }

    pub fn minimum_support(&self) -> usize {
        self.minimum_support
    }

    pub fn edge_count(&self) -> usize {
        self.edge_supports.len()
    }

    pub fn support_for_edge(
        &self,
        edge: &RecursiveWorldRevisionAbstractionCompositionEdge,
    ) -> Option<&RecursiveWorldRevisionAbstractionCompositionPathEdgeSupport> {
        self.edge_supports
            .iter()
            .find(|support| support.edge() == edge)
    }

    pub fn start(&self) -> &RecursiveWorldRevisionAbstractionClass {
        self.path.start()
    }

    pub fn end(&self) -> &RecursiveWorldRevisionAbstractionClass {
        self.path.end()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathSupportSet {
    source: RecursiveWorldRevisionAbstractionCompositionPathSet,
    supports: Vec<RecursiveWorldRevisionAbstractionCompositionPathSupport>,
}

impl RecursiveWorldRevisionAbstractionCompositionPathSupportSet {
    pub fn derive(source: RecursiveWorldRevisionAbstractionCompositionPathSet) -> Self {
        let mut supports: Vec<RecursiveWorldRevisionAbstractionCompositionPathSupport> = source
            .paths()
            .iter()
            .cloned()
            .map(RecursiveWorldRevisionAbstractionCompositionPathSupport::derive)
            .collect();

        supports.sort();
        supports.dedup();

        Self { source, supports }
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionCompositionPathSet {
        &self.source
    }

    pub fn supports(&self) -> &[RecursiveWorldRevisionAbstractionCompositionPathSupport] {
        &self.supports
    }

    pub fn len(&self) -> usize {
        self.supports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.supports.is_empty()
    }

    pub fn support_for_path(
        &self,
        path: &RecursiveWorldRevisionAbstractionCompositionPath,
    ) -> Option<&RecursiveWorldRevisionAbstractionCompositionPathSupport> {
        self.supports.iter().find(|support| support.path() == path)
    }

    pub fn supports_from_to(
        &self,
        from: &RecursiveWorldRevisionAbstractionClass,
        to: &RecursiveWorldRevisionAbstractionClass,
    ) -> Vec<&RecursiveWorldRevisionAbstractionCompositionPathSupport> {
        self.supports
            .iter()
            .filter(|support| support.start() == from && support.end() == to)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathSupportDeriver;

impl RecursiveWorldRevisionAbstractionCompositionPathSupportDeriver {
    pub fn derive(
        source: RecursiveWorldRevisionAbstractionCompositionPathSet,
    ) -> RecursiveWorldRevisionAbstractionCompositionPathSupportSet {
        RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(source)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathSelection {
    from: RecursiveWorldRevisionAbstractionClass,
    to: RecursiveWorldRevisionAbstractionClass,
    selected: RecursiveWorldRevisionAbstractionCompositionPathSupport,
}

impl RecursiveWorldRevisionAbstractionCompositionPathSelection {
    pub fn from(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.from
    }

    pub fn to(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.to
    }

    pub fn selected(&self) -> &RecursiveWorldRevisionAbstractionCompositionPathSupport {
        &self.selected
    }

    pub fn path(&self) -> &RecursiveWorldRevisionAbstractionCompositionPath {
        self.selected.path()
    }

    pub fn minimum_support(&self) -> usize {
        self.selected.minimum_support()
    }

    pub fn edge_count(&self) -> usize {
        self.selected.edge_count()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    source: RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    selections: Vec<RecursiveWorldRevisionAbstractionCompositionPathSelection>,
}

impl RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    pub fn select(source: RecursiveWorldRevisionAbstractionCompositionPathSupportSet) -> Self {
        let mut grouped: BTreeMap<
            (
                RecursiveWorldRevisionAbstractionClass,
                RecursiveWorldRevisionAbstractionClass,
            ),
            Vec<RecursiveWorldRevisionAbstractionCompositionPathSupport>,
        > = BTreeMap::new();

        for support in source.supports() {
            grouped
                .entry((support.start().clone(), support.end().clone()))
                .or_default()
                .push(support.clone());
        }

        let mut selections = Vec::new();

        for ((from, to), mut candidates) in grouped {
            candidates.sort_by(|left, right| {
                right
                    .minimum_support()
                    .cmp(&left.minimum_support())
                    .then_with(|| left.edge_count().cmp(&right.edge_count()))
                    .then_with(|| left.path().cmp(right.path()))
            });

            let selected = candidates
                .into_iter()
                .next()
                .expect("grouped composition endpoint pair has candidate");

            selections.push(RecursiveWorldRevisionAbstractionCompositionPathSelection {
                from,
                to,
                selected,
            });
        }

        selections.sort();
        selections.dedup();

        Self { source, selections }
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionCompositionPathSupportSet {
        &self.source
    }

    pub fn selections(&self) -> &[RecursiveWorldRevisionAbstractionCompositionPathSelection] {
        &self.selections
    }

    pub fn len(&self) -> usize {
        self.selections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }

    pub fn selection_for(
        &self,
        from: &RecursiveWorldRevisionAbstractionClass,
        to: &RecursiveWorldRevisionAbstractionClass,
    ) -> Option<&RecursiveWorldRevisionAbstractionCompositionPathSelection> {
        self.selections
            .iter()
            .find(|selection| selection.from() == from && selection.to() == to)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathSelector;

impl RecursiveWorldRevisionAbstractionCompositionPathSelector {
    pub fn select(
        source: RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    ) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(source)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus {
    Unavailable,
    Ambiguous,
    Deterministic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathRealization {
    selection: RecursiveWorldRevisionAbstractionCompositionPathSelection,
    application_observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    premise_witnesses: Vec<athlesia_recursive::RecursiveUnit>,
    conclusion_witnesses: Vec<athlesia_recursive::RecursiveUnit>,
    realized_observation: Option<RecursiveWorldRevisionDiscoveryObservation>,
    status: RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus,
}

impl RecursiveWorldRevisionAbstractionCompositionPathRealization {
    pub fn realize(
        selection: RecursiveWorldRevisionAbstractionCompositionPathSelection,
        mut application_observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    ) -> Self {
        application_observations.sort();
        application_observations.dedup();

        let mut premise_witnesses: BTreeSet<athlesia_recursive::RecursiveUnit> = BTreeSet::new();

        let mut conclusion_witnesses: BTreeSet<athlesia_recursive::RecursiveUnit> = BTreeSet::new();

        for observation in &application_observations {
            for unit in observation.premises() {
                if selection.from().contains(unit) {
                    premise_witnesses.insert(unit.clone());
                }
            }

            for unit in observation.conclusions() {
                if selection.to().contains(unit) {
                    conclusion_witnesses.insert(unit.clone());
                }
            }
        }

        let premise_witnesses: Vec<athlesia_recursive::RecursiveUnit> =
            premise_witnesses.into_iter().collect();

        let conclusion_witnesses: Vec<athlesia_recursive::RecursiveUnit> =
            conclusion_witnesses.into_iter().collect();

        let (realized_observation, status) =
            match (premise_witnesses.len(), conclusion_witnesses.len()) {
                (1, 1) => {
                    let realized = RecursiveWorldRevisionDiscoveryObservation::new(
                        vec![premise_witnesses[0].clone()],
                        vec![conclusion_witnesses[0].clone()],
                    );

                    match realized {
                        Some(
                            observation,
                        ) => {
                            (
                                Some(
                                    observation,
                                ),
                                RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::
                                    Deterministic,
                            )
                        }

                        None => {
                            (
                                None,
                                RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::
                                    Unavailable,
                            )
                        }
                    }
                }

                (0, _) | (_, 0) => (
                    None,
                    RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Unavailable,
                ),

                _ => (
                    None,
                    RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Ambiguous,
                ),
            };

        Self {
            selection,
            application_observations,
            premise_witnesses,
            conclusion_witnesses,
            realized_observation,
            status,
        }
    }

    pub fn selection(&self) -> &RecursiveWorldRevisionAbstractionCompositionPathSelection {
        &self.selection
    }

    pub fn application_observations(&self) -> &[RecursiveWorldRevisionDiscoveryObservation] {
        &self.application_observations
    }

    pub fn premise_witnesses(&self) -> &[athlesia_recursive::RecursiveUnit] {
        &self.premise_witnesses
    }

    pub fn conclusion_witnesses(&self) -> &[athlesia_recursive::RecursiveUnit] {
        &self.conclusion_witnesses
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.realized_observation.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus {
        self.status
    }

    pub fn is_deterministic(&self) -> bool {
        self.status
            == RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Deterministic
    }

    pub fn is_ambiguous(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Ambiguous
    }

    pub fn from(&self) -> &RecursiveWorldRevisionAbstractionClass {
        self.selection.from()
    }

    pub fn to(&self) -> &RecursiveWorldRevisionAbstractionClass {
        self.selection.to()
    }

    pub fn path(&self) -> &RecursiveWorldRevisionAbstractionCompositionPath {
        self.selection.path()
    }

    pub fn minimum_support(&self) -> usize {
        self.selection.minimum_support()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionPathRealizer;

impl RecursiveWorldRevisionAbstractionCompositionPathRealizer {
    pub fn realize(
        selection: RecursiveWorldRevisionAbstractionCompositionPathSelection,
        application_observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    ) -> RecursiveWorldRevisionAbstractionCompositionPathRealization {
        RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
            selection,
            application_observations,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus {
    RealizationUnavailable,
    DiscoveryUnavailable,
    Discovered,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge {
    target:
        athlesia_recursive_world_model::RecursiveWorldRule,
    realization:
        RecursiveWorldRevisionAbstractionCompositionPathRealization,
    hypothesis:
        Option<
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryHypothesis,
        >,
    status:
        RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus,
}

impl RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge {
    pub fn discover(
        target: athlesia_recursive_world_model::RecursiveWorldRule,
        selection: RecursiveWorldRevisionAbstractionCompositionPathSelection,
        application_observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    ) -> Self {
        let realization = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
            selection,
            application_observations,
        );

        let realized_observation = realization.realized_observation().cloned();

        let Some(realized_observation) = realized_observation else {
            return Self {
                target,
                realization,
                hypothesis: None,
                status:
                    RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus::
                        RealizationUnavailable,
            };
        };

        let hypothesis =
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryHypothesis::discover(
                    target.clone(),
                    realized_observation,
                );

        let status = if hypothesis.is_some() {
            RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus::Discovered
        } else {
            RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus::DiscoveryUnavailable
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

    pub fn realization(&self) -> &RecursiveWorldRevisionAbstractionCompositionPathRealization {
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

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus {
        self.status
    }

    pub fn is_discovered(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus::Discovered
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.realization.realized_observation()
    }

    pub fn replacement(&self) -> Option<&athlesia_recursive_world_model::RecursiveWorldRule> {
        self.hypothesis
            .as_ref()
            .map(|hypothesis| hypothesis.replacement())
    }

    pub fn selection(&self) -> &RecursiveWorldRevisionAbstractionCompositionPathSelection {
        self.realization.selection()
    }

    pub fn path(&self) -> &RecursiveWorldRevisionAbstractionCompositionPath {
        self.realization.path()
    }

    pub fn minimum_support(&self) -> usize {
        self.realization.minimum_support()
    }

    pub fn application_observations(&self) -> &[RecursiveWorldRevisionDiscoveryObservation] {
        self.realization.application_observations()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionDiscoveryBuilder;

impl RecursiveWorldRevisionAbstractionCompositionDiscoveryBuilder {
    pub fn discover(
        target: athlesia_recursive_world_model::RecursiveWorldRule,
        selection: RecursiveWorldRevisionAbstractionCompositionPathSelection,
        application_observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    ) -> RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge {
        RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
            target,
            selection,
            application_observations,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionCompositionValidationStatus {
    DiscoveryUnavailable,
    Rejected,
    Accepted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionValidation {
    model:
        athlesia_recursive_world_model::RecursiveWorldModel,
    discovery:
        RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge,
    validation:
        Option<
            athlesia_recursive_world_model_revision_discovery::
                RecursiveWorldRevisionDiscoveryValidation,
        >,
    status:
        RecursiveWorldRevisionAbstractionCompositionValidationStatus,
}

impl RecursiveWorldRevisionAbstractionCompositionValidation {
    pub fn validate(
        model: athlesia_recursive_world_model::RecursiveWorldModel,
        target: athlesia_recursive_world_model::RecursiveWorldRule,
        selection: RecursiveWorldRevisionAbstractionCompositionPathSelection,
        application_observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    ) -> Self {
        let discovery = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
            target,
            selection,
            application_observations,
        );

        let Some(hypothesis) = discovery.hypothesis().cloned() else {
            return Self {
                model,
                discovery,
                validation: None,
                status:
                    RecursiveWorldRevisionAbstractionCompositionValidationStatus::
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

        let accepted = validation.accepted_count() == 1;

        let status = if accepted {
            RecursiveWorldRevisionAbstractionCompositionValidationStatus::Accepted
        } else {
            RecursiveWorldRevisionAbstractionCompositionValidationStatus::Rejected
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

    pub fn discovery(&self) -> &RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge {
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

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionCompositionValidationStatus {
        self.status
    }

    pub fn is_accepted(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionCompositionValidationStatus::Accepted
    }

    pub fn is_rejected(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionCompositionValidationStatus::Rejected
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

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.discovery.realized_observation()
    }

    pub fn selection(&self) -> &RecursiveWorldRevisionAbstractionCompositionPathSelection {
        self.discovery.selection()
    }

    pub fn path(&self) -> &RecursiveWorldRevisionAbstractionCompositionPath {
        self.discovery.path()
    }

    pub fn minimum_support(&self) -> usize {
        self.discovery.minimum_support()
    }

    pub fn application_observations(&self) -> &[RecursiveWorldRevisionDiscoveryObservation] {
        self.discovery.application_observations()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionCompositionValidator;

impl RecursiveWorldRevisionAbstractionCompositionValidator {
    pub fn validate(
        model: athlesia_recursive_world_model::RecursiveWorldModel,
        target: athlesia_recursive_world_model::RecursiveWorldRule,
        selection: RecursiveWorldRevisionAbstractionCompositionPathSelection,
        application_observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    ) -> RecursiveWorldRevisionAbstractionCompositionValidation {
        RecursiveWorldRevisionAbstractionCompositionValidation::validate(
            model,
            target,
            selection,
            application_observations,
        )
    }
}

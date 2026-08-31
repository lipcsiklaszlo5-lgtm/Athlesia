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

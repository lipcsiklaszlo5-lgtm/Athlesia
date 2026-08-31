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

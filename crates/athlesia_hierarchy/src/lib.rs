use std::collections::BTreeSet;

use athlesia::StructuralConcept;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HierarchicalConcept {
    children: Vec<StructuralConcept>,
}

impl HierarchicalConcept {
    pub fn new(children: Vec<StructuralConcept>) -> Option<Self> {
        let unique: BTreeSet<StructuralConcept> = children.into_iter().collect();

        if unique.len() < 2 {
            return None;
        }

        Some(Self {
            children: unique.into_iter().collect(),
        })
    }

    pub fn children(&self) -> &[StructuralConcept] {
        &self.children
    }

    pub fn arity(&self) -> usize {
        self.children.len()
    }

    pub fn contains(&self, concept: &StructuralConcept) -> bool {
        self.children.binary_search(concept).is_ok()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HierarchicalMemory {
    concepts: BTreeSet<HierarchicalConcept>,
}

impl HierarchicalMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    pub fn insert(&mut self, concept: HierarchicalConcept) -> bool {
        self.concepts.insert(concept)
    }

    pub fn contains(&self, concept: &HierarchicalConcept) -> bool {
        self.concepts.contains(concept)
    }

    pub fn concepts(&self) -> impl Iterator<Item = &HierarchicalConcept> {
        self.concepts.iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyObservation {
    concepts: Vec<StructuralConcept>,
}

impl HierarchyObservation {
    pub fn new(concepts: Vec<StructuralConcept>) -> Self {
        let unique: BTreeSet<StructuralConcept> = concepts.into_iter().collect();

        Self {
            concepts: unique.into_iter().collect(),
        }
    }

    pub fn concepts(&self) -> &[StructuralConcept] {
        &self.concepts
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyCandidate {
    concept: HierarchicalConcept,
    support: usize,
}

impl HierarchyCandidate {
    pub fn concept(&self) -> &HierarchicalConcept {
        &self.concept
    }

    pub const fn support(&self) -> usize {
        self.support
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HierarchyDiscovery {
    minimum_support: usize,
}

impl HierarchyDiscovery {
    pub const fn new(minimum_support: usize) -> Self {
        Self { minimum_support }
    }

    pub const fn minimum_support(self) -> usize {
        self.minimum_support
    }

    pub fn discover(&self, observations: &[HierarchyObservation]) -> Vec<HierarchyCandidate> {
        use std::collections::BTreeMap;

        let mut support: BTreeMap<HierarchicalConcept, usize> = BTreeMap::new();

        for observation in observations {
            let concepts = observation.concepts();

            for left in 0..concepts.len() {
                for right in (left + 1)..concepts.len() {
                    let candidate = HierarchicalConcept::new(vec![
                        concepts[left].clone(),
                        concepts[right].clone(),
                    ]);

                    if let Some(candidate) = candidate {
                        *support.entry(candidate).or_insert(0) += 1;
                    }
                }
            }
        }

        support
            .into_iter()
            .filter_map(|(concept, count)| {
                if count >= self.minimum_support {
                    Some(HierarchyCandidate {
                        concept,
                        support: count,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn consolidate(
        &self,
        observations: &[HierarchyObservation],
        memory: &mut HierarchicalMemory,
    ) -> Vec<HierarchyCandidate> {
        let candidates = self.discover(observations);

        for candidate in &candidates {
            memory.insert(candidate.concept().clone());
        }

        candidates
    }
}

impl Default for HierarchyDiscovery {
    fn default() -> Self {
        Self::new(2)
    }
}

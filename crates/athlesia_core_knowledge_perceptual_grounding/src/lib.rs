use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualElementHandle(u64);

impl PerceptualElementHandle {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualElement {
    handle: PerceptualElementHandle,
    signature: CognitiveStructure,
}

impl PerceptualElement {
    pub fn new(handle: PerceptualElementHandle, signature: CognitiveStructure) -> Self {
        Self { handle, signature }
    }

    pub fn handle(&self) -> PerceptualElementHandle {
        self.handle
    }

    pub fn signature(&self) -> &CognitiveStructure {
        &self.signature
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualFrame {
    observation_index: u64,
    elements: Vec<PerceptualElement>,
}

impl PerceptualFrame {
    pub fn new(observation_index: u64, mut elements: Vec<PerceptualElement>) -> Option<Self> {
        if elements.is_empty() {
            return None;
        }

        elements.sort_by_key(PerceptualElement::handle);

        if elements
            .windows(2)
            .any(|pair| pair[0].handle() == pair[1].handle())
        {
            return None;
        }

        Some(Self {
            observation_index,
            elements,
        })
    }

    pub fn observation_index(&self) -> u64 {
        self.observation_index
    }

    pub fn elements(&self) -> &[PerceptualElement] {
        &self.elements
    }

    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    pub fn contains_handle(&self, handle: PerceptualElementHandle) -> bool {
        self.elements
            .binary_search_by_key(&handle, PerceptualElement::handle)
            .is_ok()
    }

    pub fn element(&self, handle: PerceptualElementHandle) -> Option<&PerceptualElement> {
        self.elements
            .binary_search_by_key(&handle, PerceptualElement::handle)
            .ok()
            .map(|index| &self.elements[index])
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObjecthoodEvidence {
    cohesion: CognitiveSignal,
    persistence: CognitiveSignal,
    common_change: CognitiveSignal,
    boundary: CognitiveSignal,
    containment: CognitiveSignal,
    topology: CognitiveSignal,
}

impl ObjecthoodEvidence {
    pub fn new(
        cohesion: CognitiveSignal,
        persistence: CognitiveSignal,
        common_change: CognitiveSignal,
        boundary: CognitiveSignal,
        containment: CognitiveSignal,
        topology: CognitiveSignal,
    ) -> Self {
        Self {
            cohesion,
            persistence,
            common_change,
            boundary,
            containment,
            topology,
        }
    }

    pub fn cohesion(self) -> CognitiveSignal {
        self.cohesion
    }

    pub fn persistence(self) -> CognitiveSignal {
        self.persistence
    }

    pub fn common_change(self) -> CognitiveSignal {
        self.common_change
    }

    pub fn boundary(self) -> CognitiveSignal {
        self.boundary
    }

    pub fn containment(self) -> CognitiveSignal {
        self.containment
    }

    pub fn topology(self) -> CognitiveSignal {
        self.topology
    }

    pub fn has_support(self) -> bool {
        [
            self.cohesion,
            self.persistence,
            self.common_change,
            self.boundary,
            self.containment,
            self.topology,
        ]
        .into_iter()
        .any(|signal| signal > CognitiveSignal::zero())
    }

    pub fn peak_support(self) -> CognitiveSignal {
        [
            self.cohesion,
            self.persistence,
            self.common_change,
            self.boundary,
            self.containment,
            self.topology,
        ]
        .into_iter()
        .max()
        .expect("objecthood evidence has a fixed nonempty axis set")
    }

    fn canonical_key(self) -> [u16; 6] {
        [
            self.cohesion.value(),
            self.persistence.value(),
            self.common_change.value(),
            self.boundary.value(),
            self.containment.value(),
            self.topology.value(),
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectHypothesis {
    members: Vec<PerceptualElementHandle>,
    evidence: ObjecthoodEvidence,
}

impl ObjectHypothesis {
    pub fn new(
        mut members: Vec<PerceptualElementHandle>,
        evidence: ObjecthoodEvidence,
    ) -> Option<Self> {
        if members.is_empty() || !evidence.has_support() {
            return None;
        }

        members.sort_unstable();
        members.dedup();

        Some(Self { members, evidence })
    }

    pub fn members(&self) -> &[PerceptualElementHandle] {
        &self.members
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn evidence(&self) -> ObjecthoodEvidence {
        self.evidence
    }

    pub fn contains(&self, handle: PerceptualElementHandle) -> bool {
        self.members.binary_search(&handle).is_ok()
    }

    pub fn is_grounded_in(&self, frame: &PerceptualFrame) -> bool {
        self.members
            .iter()
            .copied()
            .all(|handle| frame.contains_handle(handle))
    }

    fn membership_key(&self) -> Vec<u64> {
        self.members.iter().map(|handle| handle.value()).collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneInterpretation {
    hypotheses: Vec<ObjectHypothesis>,
    explanatory_support: CognitiveSignal,
}

impl SceneInterpretation {
    pub fn new(
        mut hypotheses: Vec<ObjectHypothesis>,
        explanatory_support: CognitiveSignal,
    ) -> Option<Self> {
        if hypotheses.is_empty() || explanatory_support == CognitiveSignal::zero() {
            return None;
        }

        hypotheses.sort_by(|left, right| left.members.cmp(&right.members));

        if hypotheses
            .windows(2)
            .any(|pair| pair[0].members() == pair[1].members())
        {
            return None;
        }

        Some(Self {
            hypotheses,
            explanatory_support,
        })
    }

    pub fn hypotheses(&self) -> &[ObjectHypothesis] {
        &self.hypotheses
    }

    pub fn hypothesis_count(&self) -> usize {
        self.hypotheses.len()
    }

    pub fn explanatory_support(&self) -> CognitiveSignal {
        self.explanatory_support
    }

    pub fn is_grounded_in(&self, frame: &PerceptualFrame) -> bool {
        self.hypotheses
            .iter()
            .all(|hypothesis| hypothesis.is_grounded_in(frame))
    }

    pub fn contains_overlapping_hypotheses(&self) -> bool {
        for left_index in 0..self.hypotheses.len() {
            for right_index in left_index.saturating_add(1)..self.hypotheses.len() {
                if self.hypotheses[left_index]
                    .members()
                    .iter()
                    .any(|member| self.hypotheses[right_index].contains(*member))
                {
                    return true;
                }
            }
        }

        false
    }

    fn grouping_key(&self) -> Vec<Vec<u64>> {
        self.hypotheses
            .iter()
            .map(ObjectHypothesis::membership_key)
            .collect()
    }

    fn evidence_key(&self) -> Vec<[u16; 6]> {
        self.hypotheses
            .iter()
            .map(|hypothesis| hypothesis.evidence().canonical_key())
            .collect()
    }

    fn same_grouping(&self, other: &Self) -> bool {
        self.hypotheses.len() == other.hypotheses.len()
            && self
                .hypotheses
                .iter()
                .zip(other.hypotheses.iter())
                .all(|(left, right)| left.members() == right.members())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroundingPolicy {
    max_object_hypotheses_per_scene: usize,
    max_scene_interpretations: usize,
}

impl PerceptualGroundingPolicy {
    pub fn new(
        max_object_hypotheses_per_scene: usize,
        max_scene_interpretations: usize,
    ) -> Option<Self> {
        if max_object_hypotheses_per_scene == 0 || max_scene_interpretations == 0 {
            return None;
        }

        Some(Self {
            max_object_hypotheses_per_scene,
            max_scene_interpretations,
        })
    }

    pub fn max_object_hypotheses_per_scene(self) -> usize {
        self.max_object_hypotheses_per_scene
    }

    pub fn max_scene_interpretations(self) -> usize {
        self.max_scene_interpretations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneCompetitionResult {
    input_scene_count: usize,
    valid_scene_count: usize,
    rejected_scene_count: usize,
    duplicate_grouping_count: usize,
    dropped_by_scene_bound_count: usize,
    selected: Vec<SceneInterpretation>,
}

impl SceneCompetitionResult {
    pub fn input_scene_count(&self) -> usize {
        self.input_scene_count
    }

    pub fn valid_scene_count(&self) -> usize {
        self.valid_scene_count
    }

    pub fn rejected_scene_count(&self) -> usize {
        self.rejected_scene_count
    }

    pub fn duplicate_grouping_count(&self) -> usize {
        self.duplicate_grouping_count
    }

    pub fn dropped_by_scene_bound_count(&self) -> usize {
        self.dropped_by_scene_bound_count
    }

    pub fn selected(&self) -> &[SceneInterpretation] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CompetingSceneInterpretations;

impl CompetingSceneInterpretations {
    fn ranking(left: &SceneInterpretation, right: &SceneInterpretation) -> std::cmp::Ordering {
        right
            .explanatory_support()
            .cmp(&left.explanatory_support())
            .then_with(|| left.hypothesis_count().cmp(&right.hypothesis_count()))
            .then_with(|| left.grouping_key().cmp(&right.grouping_key()))
            .then_with(|| left.evidence_key().cmp(&right.evidence_key()))
    }

    pub fn select(
        frame: &PerceptualFrame,
        candidates: &[SceneInterpretation],
        policy: PerceptualGroundingPolicy,
    ) -> SceneCompetitionResult {
        let input_scene_count = candidates.len();

        let mut valid_scene_count = 0_usize;

        let mut rejected_scene_count = 0_usize;

        let mut duplicate_grouping_count = 0_usize;

        let mut dropped_by_scene_bound_count = 0_usize;

        let mut frontier: Vec<SceneInterpretation> =
            Vec::with_capacity(policy.max_scene_interpretations());

        for candidate in candidates {
            if candidate.hypothesis_count() > policy.max_object_hypotheses_per_scene()
                || !candidate.is_grounded_in(frame)
            {
                rejected_scene_count = rejected_scene_count.saturating_add(1);

                continue;
            }

            valid_scene_count = valid_scene_count.saturating_add(1);

            if let Some(duplicate_index) = frontier
                .iter()
                .position(|existing| existing.same_grouping(candidate))
            {
                duplicate_grouping_count = duplicate_grouping_count.saturating_add(1);

                if Self::ranking(candidate, &frontier[duplicate_index]) == std::cmp::Ordering::Less
                {
                    frontier[duplicate_index] = candidate.clone();

                    frontier.sort_by(Self::ranking);
                }

                continue;
            }

            frontier.push(candidate.clone());

            frontier.sort_by(Self::ranking);

            if frontier.len() > policy.max_scene_interpretations() {
                frontier.pop();

                dropped_by_scene_bound_count = dropped_by_scene_bound_count.saturating_add(1);
            }
        }

        SceneCompetitionResult {
            input_scene_count,
            valid_scene_count,
            rejected_scene_count,
            duplicate_grouping_count,
            dropped_by_scene_bound_count,
            selected: frontier,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreKnowledgePerceptualGrounding;

impl CoreKnowledgePerceptualGrounding {
    pub fn evaluate(
        frame: &PerceptualFrame,
        candidates: &[SceneInterpretation],
        policy: PerceptualGroundingPolicy,
    ) -> SceneCompetitionResult {
        CompetingSceneInterpretations::select(frame, candidates, policy)
    }
}

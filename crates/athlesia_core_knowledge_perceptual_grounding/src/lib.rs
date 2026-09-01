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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectObservation {
    observation_index: u64,
    members: Vec<PerceptualElementHandle>,
}

impl ObjectObservation {
    pub fn from_hypothesis(frame: &PerceptualFrame, hypothesis: &ObjectHypothesis) -> Option<Self> {
        if !hypothesis.is_grounded_in(frame) {
            return None;
        }

        Some(Self {
            observation_index: frame.observation_index(),
            members: hypothesis.members().to_vec(),
        })
    }

    pub fn observation_index(&self) -> u64 {
        self.observation_index
    }

    pub fn members(&self) -> &[PerceptualElementHandle] {
        &self.members
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn contains(&self, handle: PerceptualElementHandle) -> bool {
        self.members.binary_search(&handle).is_ok()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PersistenceEvidence {
    structural_continuity: CognitiveSignal,
    relational_continuity: CognitiveSignal,
    change_continuity: CognitiveSignal,
    boundary_continuity: CognitiveSignal,
    containment_continuity: CognitiveSignal,
    causal_continuity: CognitiveSignal,
}

impl PersistenceEvidence {
    pub fn new(
        structural_continuity: CognitiveSignal,
        relational_continuity: CognitiveSignal,
        change_continuity: CognitiveSignal,
        boundary_continuity: CognitiveSignal,
        containment_continuity: CognitiveSignal,
        causal_continuity: CognitiveSignal,
    ) -> Self {
        Self {
            structural_continuity,
            relational_continuity,
            change_continuity,
            boundary_continuity,
            containment_continuity,
            causal_continuity,
        }
    }

    pub fn structural_continuity(self) -> CognitiveSignal {
        self.structural_continuity
    }

    pub fn relational_continuity(self) -> CognitiveSignal {
        self.relational_continuity
    }

    pub fn change_continuity(self) -> CognitiveSignal {
        self.change_continuity
    }

    pub fn boundary_continuity(self) -> CognitiveSignal {
        self.boundary_continuity
    }

    pub fn containment_continuity(self) -> CognitiveSignal {
        self.containment_continuity
    }

    pub fn causal_continuity(self) -> CognitiveSignal {
        self.causal_continuity
    }

    pub fn has_support(self) -> bool {
        self.axes()
            .into_iter()
            .any(|value| value > CognitiveSignal::zero())
    }

    pub fn peak_support(self) -> CognitiveSignal {
        self.axes()
            .into_iter()
            .max()
            .expect("persistence evidence has fixed nonempty axes")
    }

    pub fn continuity_score(self) -> CognitiveSignal {
        let axes = self.axes();

        let peak = u32::from(self.peak_support().value());

        let total = axes
            .into_iter()
            .map(|value| u32::from(value.value()))
            .sum::<u32>();

        let composite = (peak.saturating_mul(2).saturating_add(total)) / 8;

        CognitiveSignal::new(composite as u16)
            .expect("bounded persistence composite remains on signal scale")
    }

    fn axes(self) -> [CognitiveSignal; 6] {
        [
            self.structural_continuity,
            self.relational_continuity,
            self.change_continuity,
            self.boundary_continuity,
            self.containment_continuity,
            self.causal_continuity,
        ]
    }

    fn canonical_key(self) -> [u16; 6] {
        [
            self.structural_continuity.value(),
            self.relational_continuity.value(),
            self.change_continuity.value(),
            self.boundary_continuity.value(),
            self.containment_continuity.value(),
            self.causal_continuity.value(),
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceLinkHypothesis {
    previous: ObjectObservation,
    current: ObjectObservation,
    evidence: PersistenceEvidence,
}

impl PersistenceLinkHypothesis {
    pub fn new(
        previous: ObjectObservation,
        current: ObjectObservation,
        evidence: PersistenceEvidence,
    ) -> Option<Self> {
        if previous.observation_index() >= current.observation_index() || !evidence.has_support() {
            return None;
        }

        Some(Self {
            previous,
            current,
            evidence,
        })
    }

    pub fn previous(&self) -> &ObjectObservation {
        &self.previous
    }

    pub fn current(&self) -> &ObjectObservation {
        &self.current
    }

    pub fn evidence(&self) -> PersistenceEvidence {
        self.evidence
    }

    pub fn continuity_score(&self) -> CognitiveSignal {
        self.evidence.continuity_score()
    }

    pub fn temporal_gap(&self) -> u64 {
        self.current
            .observation_index()
            .saturating_sub(self.previous.observation_index())
    }

    fn same_transition(&self, other: &Self) -> bool {
        self.previous == other.previous && self.current == other.current
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PersistenceTrackingPolicy {
    max_predecessors_per_current: usize,
    max_successors_per_previous: usize,
    max_total_links: usize,
}

impl PersistenceTrackingPolicy {
    pub fn new(
        max_predecessors_per_current: usize,
        max_successors_per_previous: usize,
        max_total_links: usize,
    ) -> Option<Self> {
        if max_predecessors_per_current == 0
            || max_successors_per_previous == 0
            || max_total_links == 0
        {
            return None;
        }

        Some(Self {
            max_predecessors_per_current,
            max_successors_per_previous,
            max_total_links,
        })
    }

    pub fn max_predecessors_per_current(self) -> usize {
        self.max_predecessors_per_current
    }

    pub fn max_successors_per_previous(self) -> usize {
        self.max_successors_per_previous
    }

    pub fn max_total_links(self) -> usize {
        self.max_total_links
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceTrackingResult {
    input_link_count: usize,
    canonical_link_count: usize,
    duplicate_transition_count: usize,
    dropped_by_predecessor_bound_count: usize,
    dropped_by_successor_bound_count: usize,
    dropped_by_global_bound_count: usize,
    selected: Vec<PersistenceLinkHypothesis>,
}

impl PersistenceTrackingResult {
    pub fn input_link_count(&self) -> usize {
        self.input_link_count
    }

    pub fn canonical_link_count(&self) -> usize {
        self.canonical_link_count
    }

    pub fn duplicate_transition_count(&self) -> usize {
        self.duplicate_transition_count
    }

    pub fn dropped_by_predecessor_bound_count(&self) -> usize {
        self.dropped_by_predecessor_bound_count
    }

    pub fn dropped_by_successor_bound_count(&self) -> usize {
        self.dropped_by_successor_bound_count
    }

    pub fn dropped_by_global_bound_count(&self) -> usize {
        self.dropped_by_global_bound_count
    }

    pub fn selected(&self) -> &[PersistenceLinkHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PersistenceTracking;

impl PersistenceTracking {
    fn ranking(
        left: &PersistenceLinkHypothesis,
        right: &PersistenceLinkHypothesis,
    ) -> std::cmp::Ordering {
        right
            .continuity_score()
            .cmp(&left.continuity_score())
            .then_with(|| {
                right
                    .evidence()
                    .peak_support()
                    .cmp(&left.evidence().peak_support())
            })
            .then_with(|| left.temporal_gap().cmp(&right.temporal_gap()))
            .then_with(|| left.previous().cmp(right.previous()))
            .then_with(|| left.current().cmp(right.current()))
            .then_with(|| {
                left.evidence()
                    .canonical_key()
                    .cmp(&right.evidence().canonical_key())
            })
    }

    pub fn select(
        candidates: &[PersistenceLinkHypothesis],
        policy: PersistenceTrackingPolicy,
    ) -> PersistenceTrackingResult {
        let input_link_count = candidates.len();

        let mut duplicate_transition_count = 0_usize;

        let mut canonical: Vec<PersistenceLinkHypothesis> = Vec::new();

        for candidate in candidates {
            if let Some(position) = canonical
                .iter()
                .position(|existing| existing.same_transition(candidate))
            {
                duplicate_transition_count = duplicate_transition_count.saturating_add(1);

                if Self::ranking(candidate, &canonical[position]) == std::cmp::Ordering::Less {
                    canonical[position] = candidate.clone();
                }
            } else {
                canonical.push(candidate.clone());
            }
        }

        canonical.sort_by(Self::ranking);

        let canonical_link_count = canonical.len();

        let mut predecessor_counts: std::collections::BTreeMap<ObjectObservation, usize> =
            std::collections::BTreeMap::new();

        let mut successor_counts: std::collections::BTreeMap<ObjectObservation, usize> =
            std::collections::BTreeMap::new();

        let mut selected = Vec::with_capacity(policy.max_total_links().min(canonical_link_count));

        let mut dropped_by_predecessor_bound_count = 0_usize;

        let mut dropped_by_successor_bound_count = 0_usize;

        let mut dropped_by_global_bound_count = 0_usize;

        for (index, candidate) in canonical.into_iter().enumerate() {
            if selected.len() >= policy.max_total_links() {
                dropped_by_global_bound_count = dropped_by_global_bound_count
                    .saturating_add(canonical_link_count.saturating_sub(index));

                break;
            }

            let current_count = predecessor_counts
                .get(candidate.current())
                .copied()
                .unwrap_or(0);

            if current_count >= policy.max_predecessors_per_current() {
                dropped_by_predecessor_bound_count =
                    dropped_by_predecessor_bound_count.saturating_add(1);

                continue;
            }

            let previous_count = successor_counts
                .get(candidate.previous())
                .copied()
                .unwrap_or(0);

            if previous_count >= policy.max_successors_per_previous() {
                dropped_by_successor_bound_count =
                    dropped_by_successor_bound_count.saturating_add(1);

                continue;
            }

            predecessor_counts
                .entry(candidate.current().clone())
                .and_modify(|count| {
                    *count = count.saturating_add(1);
                })
                .or_insert(1);

            successor_counts
                .entry(candidate.previous().clone())
                .and_modify(|count| {
                    *count = count.saturating_add(1);
                })
                .or_insert(1);

            selected.push(candidate);
        }

        PersistenceTrackingResult {
            input_link_count,
            canonical_link_count,
            duplicate_transition_count,
            dropped_by_predecessor_bound_count,
            dropped_by_successor_bound_count,
            dropped_by_global_bound_count,
            selected,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreKnowledgePersistenceTracking;

impl CoreKnowledgePersistenceTracking {
    pub fn evaluate(
        candidates: &[PersistenceLinkHypothesis],
        policy: PersistenceTrackingPolicy,
    ) -> PersistenceTrackingResult {
        PersistenceTracking::select(candidates, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TopologicalRelationKind {
    Adjacent,
    Contact,
    Contains,
    Overlap,
    Separate,
}

impl TopologicalRelationKind {
    pub fn is_symmetric(self) -> bool {
        matches!(
            self,
            Self::Adjacent | Self::Contact | Self::Overlap | Self::Separate
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct TopologicalPairKey {
    first: ObjectObservation,
    second: ObjectObservation,
}

impl TopologicalPairKey {
    fn new(left: &ObjectObservation, right: &ObjectObservation) -> Self {
        if left <= right {
            Self {
                first: left.clone(),
                second: right.clone(),
            }
        } else {
            Self {
                first: right.clone(),
                second: left.clone(),
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologicalRelationHypothesis {
    subject: ObjectObservation,
    relation: TopologicalRelationKind,
    object: ObjectObservation,
    support: CognitiveSignal,
}

impl TopologicalRelationHypothesis {
    pub fn new(
        mut subject: ObjectObservation,
        relation: TopologicalRelationKind,
        mut object: ObjectObservation,
        support: CognitiveSignal,
    ) -> Option<Self> {
        if subject.observation_index() != object.observation_index()
            || subject == object
            || support == CognitiveSignal::zero()
        {
            return None;
        }

        if relation.is_symmetric() && object < subject {
            std::mem::swap(&mut subject, &mut object);
        }

        Some(Self {
            subject,
            relation,
            object,
            support,
        })
    }

    pub fn observation_index(&self) -> u64 {
        self.subject.observation_index()
    }

    pub fn subject(&self) -> &ObjectObservation {
        &self.subject
    }

    pub fn relation(&self) -> TopologicalRelationKind {
        self.relation
    }

    pub fn object(&self) -> &ObjectObservation {
        &self.object
    }

    pub fn support(&self) -> CognitiveSignal {
        self.support
    }

    pub fn is_directional(&self) -> bool {
        !self.relation.is_symmetric()
    }

    fn pair_key(&self) -> TopologicalPairKey {
        TopologicalPairKey::new(&self.subject, &self.object)
    }

    fn same_relation_identity(&self, other: &Self) -> bool {
        self.subject == other.subject
            && self.relation == other.relation
            && self.object == other.object
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TopologicalRelationPolicy {
    max_relations_per_pair: usize,
    max_total_relations: usize,
}

impl TopologicalRelationPolicy {
    pub fn new(max_relations_per_pair: usize, max_total_relations: usize) -> Option<Self> {
        if max_relations_per_pair == 0 || max_total_relations == 0 {
            return None;
        }

        Some(Self {
            max_relations_per_pair,
            max_total_relations,
        })
    }

    pub fn max_relations_per_pair(self) -> usize {
        self.max_relations_per_pair
    }

    pub fn max_total_relations(self) -> usize {
        self.max_total_relations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologicalRelationCompetitionResult {
    input_relation_count: usize,
    canonical_relation_count: usize,
    duplicate_relation_count: usize,
    dropped_by_pair_bound_count: usize,
    dropped_by_global_bound_count: usize,
    selected: Vec<TopologicalRelationHypothesis>,
}

impl TopologicalRelationCompetitionResult {
    pub fn input_relation_count(&self) -> usize {
        self.input_relation_count
    }

    pub fn canonical_relation_count(&self) -> usize {
        self.canonical_relation_count
    }

    pub fn duplicate_relation_count(&self) -> usize {
        self.duplicate_relation_count
    }

    pub fn dropped_by_pair_bound_count(&self) -> usize {
        self.dropped_by_pair_bound_count
    }

    pub fn dropped_by_global_bound_count(&self) -> usize {
        self.dropped_by_global_bound_count
    }

    pub fn selected(&self) -> &[TopologicalRelationHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TopologicalRelationCompetition;

impl TopologicalRelationCompetition {
    fn ranking(
        left: &TopologicalRelationHypothesis,
        right: &TopologicalRelationHypothesis,
    ) -> std::cmp::Ordering {
        right
            .support()
            .cmp(&left.support())
            .then_with(|| left.relation().cmp(&right.relation()))
            .then_with(|| left.subject().cmp(right.subject()))
            .then_with(|| left.object().cmp(right.object()))
    }

    pub fn select(
        candidates: &[TopologicalRelationHypothesis],
        policy: TopologicalRelationPolicy,
    ) -> TopologicalRelationCompetitionResult {
        let input_relation_count = candidates.len();

        let mut canonical: Vec<TopologicalRelationHypothesis> = Vec::new();

        let mut duplicate_relation_count = 0_usize;

        for candidate in candidates {
            if let Some(position) = canonical
                .iter()
                .position(|existing| existing.same_relation_identity(candidate))
            {
                duplicate_relation_count = duplicate_relation_count.saturating_add(1);

                if Self::ranking(candidate, &canonical[position]) == std::cmp::Ordering::Less {
                    canonical[position] = candidate.clone();
                }
            } else {
                canonical.push(candidate.clone());
            }
        }

        canonical.sort_by(Self::ranking);

        let canonical_relation_count = canonical.len();

        let mut pair_counts: std::collections::BTreeMap<TopologicalPairKey, usize> =
            std::collections::BTreeMap::new();

        let mut selected =
            Vec::with_capacity(policy.max_total_relations().min(canonical_relation_count));

        let mut dropped_by_pair_bound_count = 0_usize;

        let mut dropped_by_global_bound_count = 0_usize;

        for (index, candidate) in canonical.into_iter().enumerate() {
            if selected.len() >= policy.max_total_relations() {
                dropped_by_global_bound_count = dropped_by_global_bound_count
                    .saturating_add(canonical_relation_count.saturating_sub(index));

                break;
            }

            let pair_key = candidate.pair_key();

            let current_count = pair_counts.get(&pair_key).copied().unwrap_or(0);

            if current_count >= policy.max_relations_per_pair() {
                dropped_by_pair_bound_count = dropped_by_pair_bound_count.saturating_add(1);

                continue;
            }

            pair_counts
                .entry(pair_key)
                .and_modify(|count| {
                    *count = count.saturating_add(1);
                })
                .or_insert(1);

            selected.push(candidate);
        }

        TopologicalRelationCompetitionResult {
            input_relation_count,
            canonical_relation_count,
            duplicate_relation_count,
            dropped_by_pair_bound_count,
            dropped_by_global_bound_count,
            selected,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreKnowledgeTopologicalRelations;

impl CoreKnowledgeTopologicalRelations {
    pub fn evaluate(
        candidates: &[TopologicalRelationHypothesis],
        policy: TopologicalRelationPolicy,
    ) -> TopologicalRelationCompetitionResult {
        TopologicalRelationCompetition::select(candidates, policy)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectTransitionObservation {
    previous: ObjectObservation,
    current: ObjectObservation,
}

impl ObjectTransitionObservation {
    pub fn from_persistence_link(link: &PersistenceLinkHypothesis) -> Self {
        Self {
            previous: link.previous().clone(),
            current: link.current().clone(),
        }
    }

    pub fn previous(&self) -> &ObjectObservation {
        &self.previous
    }

    pub fn current(&self) -> &ObjectObservation {
        &self.current
    }

    pub fn start_index(&self) -> u64 {
        self.previous.observation_index()
    }

    pub fn end_index(&self) -> u64 {
        self.current.observation_index()
    }

    pub fn temporal_gap(&self) -> u64 {
        self.end_index().saturating_sub(self.start_index())
    }

    pub fn shares_time_window(&self, other: &Self) -> bool {
        self.start_index() == other.start_index() && self.end_index() == other.end_index()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PerceptualChangeKind {
    StateTransition,
    Motion,
    RelativeChange,
    CommonChange,
}

impl PerceptualChangeKind {
    pub fn requires_reference(self) -> bool {
        matches!(self, Self::RelativeChange | Self::CommonChange)
    }

    pub fn is_symmetric_comparison(self) -> bool {
        matches!(self, Self::CommonChange)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChangeEvidence {
    state_difference: CognitiveSignal,
    relational_difference: CognitiveSignal,
    structural_difference: CognitiveSignal,
    temporal_consistency: CognitiveSignal,
    commonality: CognitiveSignal,
    causal_support: CognitiveSignal,
}

impl ChangeEvidence {
    pub fn new(
        state_difference: CognitiveSignal,
        relational_difference: CognitiveSignal,
        structural_difference: CognitiveSignal,
        temporal_consistency: CognitiveSignal,
        commonality: CognitiveSignal,
        causal_support: CognitiveSignal,
    ) -> Self {
        Self {
            state_difference,
            relational_difference,
            structural_difference,
            temporal_consistency,
            commonality,
            causal_support,
        }
    }

    pub fn state_difference(self) -> CognitiveSignal {
        self.state_difference
    }

    pub fn relational_difference(self) -> CognitiveSignal {
        self.relational_difference
    }

    pub fn structural_difference(self) -> CognitiveSignal {
        self.structural_difference
    }

    pub fn temporal_consistency(self) -> CognitiveSignal {
        self.temporal_consistency
    }

    pub fn commonality(self) -> CognitiveSignal {
        self.commonality
    }

    pub fn causal_support(self) -> CognitiveSignal {
        self.causal_support
    }

    pub fn has_support(self) -> bool {
        self.axes()
            .into_iter()
            .any(|value| value > CognitiveSignal::zero())
    }

    pub fn peak_support(self) -> CognitiveSignal {
        self.axes()
            .into_iter()
            .max()
            .expect("change evidence has fixed nonempty axes")
    }

    pub fn change_score(self) -> CognitiveSignal {
        let axes = self.axes();

        let peak = u32::from(self.peak_support().value());

        let total = axes
            .into_iter()
            .map(|value| u32::from(value.value()))
            .sum::<u32>();

        let composite = peak.saturating_mul(2).saturating_add(total) / 8;

        CognitiveSignal::new(composite as u16)
            .expect("bounded change composite remains on signal scale")
    }

    fn axes(self) -> [CognitiveSignal; 6] {
        [
            self.state_difference,
            self.relational_difference,
            self.structural_difference,
            self.temporal_consistency,
            self.commonality,
            self.causal_support,
        ]
    }

    fn canonical_key(self) -> [u16; 6] {
        [
            self.state_difference.value(),
            self.relational_difference.value(),
            self.structural_difference.value(),
            self.temporal_consistency.value(),
            self.commonality.value(),
            self.causal_support.value(),
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualChangeHypothesis {
    transition: ObjectTransitionObservation,
    kind: PerceptualChangeKind,
    reference: Option<ObjectTransitionObservation>,
    descriptor: CognitiveStructure,
    evidence: ChangeEvidence,
}

impl PerceptualChangeHypothesis {
    pub fn new(
        mut transition: ObjectTransitionObservation,
        kind: PerceptualChangeKind,
        mut reference: Option<ObjectTransitionObservation>,
        descriptor: CognitiveStructure,
        evidence: ChangeEvidence,
    ) -> Option<Self> {
        if !evidence.has_support() || kind.requires_reference() != reference.is_some() {
            return None;
        }

        if let Some(reference_transition) = reference.as_ref()
            && (transition == *reference_transition
                || !transition.shares_time_window(reference_transition))
        {
            return None;
        }

        if kind.is_symmetric_comparison()
            && let Some(reference_transition) = reference.as_mut()
            && *reference_transition < transition
        {
            std::mem::swap(&mut transition, reference_transition);
        }

        Some(Self {
            transition,
            kind,
            reference,
            descriptor,
            evidence,
        })
    }

    pub fn transition(&self) -> &ObjectTransitionObservation {
        &self.transition
    }

    pub fn kind(&self) -> PerceptualChangeKind {
        self.kind
    }

    pub fn reference(&self) -> Option<&ObjectTransitionObservation> {
        self.reference.as_ref()
    }

    pub fn descriptor(&self) -> &CognitiveStructure {
        &self.descriptor
    }

    pub fn evidence(&self) -> ChangeEvidence {
        self.evidence
    }

    pub fn change_score(&self) -> CognitiveSignal {
        self.evidence.change_score()
    }

    pub fn is_comparative(&self) -> bool {
        self.reference.is_some()
    }

    fn same_change_identity(&self, other: &Self) -> bool {
        self.transition == other.transition
            && self.kind == other.kind
            && self.reference == other.reference
            && self.descriptor == other.descriptor
    }

    fn involved_transitions(&self) -> Vec<ObjectTransitionObservation> {
        let mut transitions = vec![self.transition.clone()];

        if let Some(reference) = &self.reference {
            transitions.push(reference.clone());

            transitions.sort();
            transitions.dedup();
        }

        transitions
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualChangePolicy {
    max_hypotheses_per_transition: usize,
    max_total_hypotheses: usize,
}

impl PerceptualChangePolicy {
    pub fn new(max_hypotheses_per_transition: usize, max_total_hypotheses: usize) -> Option<Self> {
        if max_hypotheses_per_transition == 0 || max_total_hypotheses == 0 {
            return None;
        }

        Some(Self {
            max_hypotheses_per_transition,
            max_total_hypotheses,
        })
    }

    pub fn max_hypotheses_per_transition(self) -> usize {
        self.max_hypotheses_per_transition
    }

    pub fn max_total_hypotheses(self) -> usize {
        self.max_total_hypotheses
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualChangeCompetitionResult {
    input_hypothesis_count: usize,
    canonical_hypothesis_count: usize,
    duplicate_hypothesis_count: usize,
    dropped_by_transition_bound_count: usize,
    dropped_by_global_bound_count: usize,
    selected: Vec<PerceptualChangeHypothesis>,
}

impl PerceptualChangeCompetitionResult {
    pub fn input_hypothesis_count(&self) -> usize {
        self.input_hypothesis_count
    }

    pub fn canonical_hypothesis_count(&self) -> usize {
        self.canonical_hypothesis_count
    }

    pub fn duplicate_hypothesis_count(&self) -> usize {
        self.duplicate_hypothesis_count
    }

    pub fn dropped_by_transition_bound_count(&self) -> usize {
        self.dropped_by_transition_bound_count
    }

    pub fn dropped_by_global_bound_count(&self) -> usize {
        self.dropped_by_global_bound_count
    }

    pub fn selected(&self) -> &[PerceptualChangeHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualChangeCompetition;

impl PerceptualChangeCompetition {
    fn compare_structure(
        left: &CognitiveStructure,
        right: &CognitiveStructure,
    ) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (left, right) {
            (CognitiveStructure::Atom(left_value), CognitiveStructure::Atom(right_value)) => {
                left_value.cmp(right_value)
            }

            (CognitiveStructure::Atom(_), _) => Ordering::Less,

            (_, CognitiveStructure::Atom(_)) => Ordering::Greater,

            (
                CognitiveStructure::Ordered(left_values),
                CognitiveStructure::Ordered(right_values),
            )
            | (
                CognitiveStructure::Unordered(left_values),
                CognitiveStructure::Unordered(right_values),
            ) => {
                let mut left_iterator = left_values.iter();

                let mut right_iterator = right_values.iter();

                loop {
                    match (left_iterator.next(), right_iterator.next()) {
                        (Some(left_item), Some(right_item)) => {
                            let ordering = Self::compare_structure(left_item, right_item);

                            if ordering != Ordering::Equal {
                                return ordering;
                            }
                        }

                        (None, Some(_)) => {
                            return Ordering::Less;
                        }

                        (Some(_), None) => {
                            return Ordering::Greater;
                        }

                        (None, None) => {
                            return Ordering::Equal;
                        }
                    }
                }
            }

            (CognitiveStructure::Ordered(_), CognitiveStructure::Unordered(_)) => Ordering::Less,

            (CognitiveStructure::Unordered(_), CognitiveStructure::Ordered(_)) => Ordering::Greater,
        }
    }

    fn ranking(
        left: &PerceptualChangeHypothesis,
        right: &PerceptualChangeHypothesis,
    ) -> std::cmp::Ordering {
        right
            .change_score()
            .cmp(&left.change_score())
            .then_with(|| {
                right
                    .evidence()
                    .peak_support()
                    .cmp(&left.evidence().peak_support())
            })
            .then_with(|| {
                left.transition()
                    .temporal_gap()
                    .cmp(&right.transition().temporal_gap())
            })
            .then_with(|| left.kind().cmp(&right.kind()))
            .then_with(|| left.transition().cmp(right.transition()))
            .then_with(|| left.reference().cmp(&right.reference()))
            .then_with(|| Self::compare_structure(left.descriptor(), right.descriptor()))
            .then_with(|| {
                left.evidence()
                    .canonical_key()
                    .cmp(&right.evidence().canonical_key())
            })
    }

    pub fn select(
        candidates: &[PerceptualChangeHypothesis],
        policy: PerceptualChangePolicy,
    ) -> PerceptualChangeCompetitionResult {
        let input_hypothesis_count = candidates.len();

        let mut canonical: Vec<PerceptualChangeHypothesis> = Vec::new();

        let mut duplicate_hypothesis_count = 0_usize;

        for candidate in candidates {
            if let Some(position) = canonical
                .iter()
                .position(|existing| existing.same_change_identity(candidate))
            {
                duplicate_hypothesis_count = duplicate_hypothesis_count.saturating_add(1);

                if Self::ranking(candidate, &canonical[position]) == std::cmp::Ordering::Less {
                    canonical[position] = candidate.clone();
                }
            } else {
                canonical.push(candidate.clone());
            }
        }

        canonical.sort_by(Self::ranking);

        let canonical_hypothesis_count = canonical.len();

        let mut transition_counts: std::collections::BTreeMap<ObjectTransitionObservation, usize> =
            std::collections::BTreeMap::new();

        let mut selected = Vec::with_capacity(
            policy
                .max_total_hypotheses()
                .min(canonical_hypothesis_count),
        );

        let mut dropped_by_transition_bound_count = 0_usize;

        let mut dropped_by_global_bound_count = 0_usize;

        for (index, candidate) in canonical.into_iter().enumerate() {
            if selected.len() >= policy.max_total_hypotheses() {
                dropped_by_global_bound_count = dropped_by_global_bound_count
                    .saturating_add(canonical_hypothesis_count.saturating_sub(index));

                break;
            }

            let involved = candidate.involved_transitions();

            let bound_reached = involved.iter().any(|transition| {
                transition_counts.get(transition).copied().unwrap_or(0)
                    >= policy.max_hypotheses_per_transition()
            });

            if bound_reached {
                dropped_by_transition_bound_count =
                    dropped_by_transition_bound_count.saturating_add(1);

                continue;
            }

            for transition in involved {
                transition_counts
                    .entry(transition)
                    .and_modify(|count| {
                        *count = count.saturating_add(1);
                    })
                    .or_insert(1);
            }

            selected.push(candidate);
        }

        PerceptualChangeCompetitionResult {
            input_hypothesis_count,
            canonical_hypothesis_count,
            duplicate_hypothesis_count,
            dropped_by_transition_bound_count,
            dropped_by_global_bound_count,
            selected,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreKnowledgeMotionChange;

impl CoreKnowledgeMotionChange {
    pub fn evaluate(
        candidates: &[PerceptualChangeHypothesis],
        policy: PerceptualChangePolicy,
    ) -> PerceptualChangeCompetitionResult {
        PerceptualChangeCompetition::select(candidates, policy)
    }
}

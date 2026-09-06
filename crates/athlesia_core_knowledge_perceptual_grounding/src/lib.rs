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

// -----------------------------------------------------------------------------
// Evidence-neutral perceptual object proposal frontier
// -----------------------------------------------------------------------------
//
// A proposal is not an ObjectHypothesis.
//
// Perceptual proposal generation answers only:
//
//     "which grounded perceptual elements may be worth considering together?"
//
// It does not claim cohesion, persistence, common change, boundary,
// containment, topology, or objecthood.
//
// Promotion into ObjectHypothesis remains downstream and requires explicit
// ObjecthoodEvidence. This separation prevents raw candidate generation from
// fabricating semantic evidence merely to enter the M46 competition.

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualObjectProposal {
    members: Vec<PerceptualElementHandle>,
}

impl PerceptualObjectProposal {
    pub fn new(mut members: Vec<PerceptualElementHandle>) -> Option<Self> {
        if members.is_empty() {
            return None;
        }

        members.sort_unstable();
        members.dedup();

        Some(Self { members })
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

    pub fn is_grounded_in(&self, frame: &PerceptualFrame) -> bool {
        self.members
            .iter()
            .copied()
            .all(|handle| frame.contains_handle(handle))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AtomicPerceptualProposalPolicy {
    max_proposals: usize,
}

impl AtomicPerceptualProposalPolicy {
    pub fn new(max_proposals: usize) -> Option<Self> {
        if max_proposals == 0 {
            return None;
        }

        Some(Self { max_proposals })
    }

    pub fn max_proposals(self) -> usize {
        self.max_proposals
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomicPerceptualProposalResult {
    input_element_count: usize,
    excluded_element_count: usize,
    eligible_element_count: usize,
    dropped_by_bound_count: usize,
    proposals: Vec<PerceptualObjectProposal>,
}

impl AtomicPerceptualProposalResult {
    pub fn input_element_count(&self) -> usize {
        self.input_element_count
    }

    pub fn excluded_element_count(&self) -> usize {
        self.excluded_element_count
    }

    pub fn eligible_element_count(&self) -> usize {
        self.eligible_element_count
    }

    pub fn dropped_by_bound_count(&self) -> usize {
        self.dropped_by_bound_count
    }

    pub fn proposals(&self) -> &[PerceptualObjectProposal] {
        &self.proposals
    }

    pub fn proposal_count(&self) -> usize {
        self.proposals.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AtomicPerceptualProposalGeneration;

impl AtomicPerceptualProposalGeneration {
    pub fn generate(
        frame: &PerceptualFrame,
        excluded_handles: &[PerceptualElementHandle],
        policy: AtomicPerceptualProposalPolicy,
    ) -> AtomicPerceptualProposalResult {
        let mut excluded = excluded_handles.to_vec();
        excluded.sort_unstable();
        excluded.dedup();

        let input_element_count = frame.element_count();

        let mut excluded_element_count = 0_usize;
        let mut eligible_handles = Vec::new();

        for element in frame.elements() {
            if excluded.binary_search(&element.handle()).is_ok() {
                excluded_element_count = excluded_element_count.saturating_add(1);
                continue;
            }

            eligible_handles.push(element.handle());
        }

        let eligible_element_count = eligible_handles.len();

        let proposals = eligible_handles
            .iter()
            .copied()
            .take(policy.max_proposals())
            .map(|handle| {
                PerceptualObjectProposal::new(vec![handle])
                    .expect("atomic proposal has exactly one grounded member")
            })
            .collect::<Vec<_>>();

        let dropped_by_bound_count = eligible_element_count.saturating_sub(proposals.len());

        AtomicPerceptualProposalResult {
            input_element_count,
            excluded_element_count,
            eligible_element_count,
            dropped_by_bound_count,
            proposals,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalAtomicPerceptualProposalGeneration;

impl UniversalAtomicPerceptualProposalGeneration {
    pub fn evaluate(
        frame: &PerceptualFrame,
        excluded_handles: &[PerceptualElementHandle],
        policy: AtomicPerceptualProposalPolicy,
    ) -> AtomicPerceptualProposalResult {
        AtomicPerceptualProposalGeneration::generate(frame, excluded_handles, policy)
    }
}

// -----------------------------------------------------------------------------
// Exact cross-frame observational evidence for perceptual proposals
// -----------------------------------------------------------------------------
//
// This layer records only what two grounded PerceptualFrames establish
// directly about a proposal's member identities.
//
// It intentionally does NOT:
// - infer objecthood;
// - fabricate ObjecthoodEvidence;
// - assign confidence;
// - infer motion from coordinate handles;
// - infer causality;
// - merge members into larger groups.
//
// Promotion into ObjectHypothesis remains a separate evidentiary decision.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PerceptualProposalObservationStatus {
    Stable,
    Changed,
    Appeared,
    Disappeared,
    Mixed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualProposalObservationEvidence {
    proposal: PerceptualObjectProposal,
    status: PerceptualProposalObservationStatus,
    previous_present_count: usize,
    current_present_count: usize,
    stable_member_count: usize,
    changed_member_count: usize,
    appeared_member_count: usize,
    disappeared_member_count: usize,
}

impl PerceptualProposalObservationEvidence {
    pub fn proposal(&self) -> &PerceptualObjectProposal {
        &self.proposal
    }

    pub fn status(&self) -> PerceptualProposalObservationStatus {
        self.status
    }

    pub fn previous_present_count(&self) -> usize {
        self.previous_present_count
    }

    pub fn current_present_count(&self) -> usize {
        self.current_present_count
    }

    pub fn stable_member_count(&self) -> usize {
        self.stable_member_count
    }

    pub fn changed_member_count(&self) -> usize {
        self.changed_member_count
    }

    pub fn appeared_member_count(&self) -> usize {
        self.appeared_member_count
    }

    pub fn disappeared_member_count(&self) -> usize {
        self.disappeared_member_count
    }

    pub fn member_count(&self) -> usize {
        self.proposal.member_count()
    }

    pub fn has_direct_temporal_evidence(&self) -> bool {
        self.previous_present_count > 0 || self.current_present_count > 0
    }

    pub fn is_exactly_stable(&self) -> bool {
        self.status == PerceptualProposalObservationStatus::Stable
    }

    pub fn is_exactly_changed(&self) -> bool {
        self.status == PerceptualProposalObservationStatus::Changed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualProposalObservationResult {
    evidence: Vec<PerceptualProposalObservationEvidence>,
}

impl PerceptualProposalObservationResult {
    pub fn evidence(&self) -> &[PerceptualProposalObservationEvidence] {
        &self.evidence
    }

    pub fn evidence_count(&self) -> usize {
        self.evidence.len()
    }

    pub fn stable_count(&self) -> usize {
        self.evidence
            .iter()
            .filter(|item| item.status() == PerceptualProposalObservationStatus::Stable)
            .count()
    }

    pub fn changed_count(&self) -> usize {
        self.evidence
            .iter()
            .filter(|item| item.status() == PerceptualProposalObservationStatus::Changed)
            .count()
    }

    pub fn appeared_count(&self) -> usize {
        self.evidence
            .iter()
            .filter(|item| item.status() == PerceptualProposalObservationStatus::Appeared)
            .count()
    }

    pub fn disappeared_count(&self) -> usize {
        self.evidence
            .iter()
            .filter(|item| item.status() == PerceptualProposalObservationStatus::Disappeared)
            .count()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualProposalObservation;

impl PerceptualProposalObservation {
    fn classify(
        previous_present_count: usize,
        current_present_count: usize,
        stable_member_count: usize,
        changed_member_count: usize,
        appeared_member_count: usize,
        disappeared_member_count: usize,
        member_count: usize,
    ) -> PerceptualProposalObservationStatus {
        if stable_member_count == member_count {
            return PerceptualProposalObservationStatus::Stable;
        }

        if changed_member_count == member_count {
            return PerceptualProposalObservationStatus::Changed;
        }

        if appeared_member_count == member_count {
            return PerceptualProposalObservationStatus::Appeared;
        }

        if disappeared_member_count == member_count {
            return PerceptualProposalObservationStatus::Disappeared;
        }

        let classified = stable_member_count
            .saturating_add(changed_member_count)
            .saturating_add(appeared_member_count)
            .saturating_add(disappeared_member_count);

        assert_eq!(
            classified, member_count,
            "every proposal member must receive one exact cross-frame classification"
        );

        assert!(
            previous_present_count <= member_count && current_present_count <= member_count,
            "presence counts are bounded by proposal membership"
        );

        PerceptualProposalObservationStatus::Mixed
    }

    pub fn observe(
        previous_frame: &PerceptualFrame,
        current_frame: &PerceptualFrame,
        proposals: &[PerceptualObjectProposal],
    ) -> PerceptualProposalObservationResult {
        let mut evidence = Vec::with_capacity(proposals.len());

        for proposal in proposals {
            let mut previous_present_count = 0_usize;
            let mut current_present_count = 0_usize;
            let mut stable_member_count = 0_usize;
            let mut changed_member_count = 0_usize;
            let mut appeared_member_count = 0_usize;
            let mut disappeared_member_count = 0_usize;

            for handle in proposal.members() {
                let previous = previous_frame.element(*handle);
                let current = current_frame.element(*handle);

                match (previous, current) {
                    (Some(previous), Some(current)) => {
                        previous_present_count = previous_present_count.saturating_add(1);

                        current_present_count = current_present_count.saturating_add(1);

                        if previous.signature() == current.signature() {
                            stable_member_count = stable_member_count.saturating_add(1);
                        } else {
                            changed_member_count = changed_member_count.saturating_add(1);
                        }
                    }
                    (None, Some(_)) => {
                        current_present_count = current_present_count.saturating_add(1);

                        appeared_member_count = appeared_member_count.saturating_add(1);
                    }
                    (Some(_), None) => {
                        previous_present_count = previous_present_count.saturating_add(1);

                        disappeared_member_count = disappeared_member_count.saturating_add(1);
                    }
                    (None, None) => {
                        /*
                         * Proposals are allowed to originate from either side
                         * of a transition, but a member absent from both frames
                         * carries no grounded temporal evidence and therefore
                         * must not silently receive a semantic classification.
                         */
                    }
                }
            }

            let classified = stable_member_count
                .saturating_add(changed_member_count)
                .saturating_add(appeared_member_count)
                .saturating_add(disappeared_member_count);

            if classified != proposal.member_count() {
                continue;
            }

            let status = Self::classify(
                previous_present_count,
                current_present_count,
                stable_member_count,
                changed_member_count,
                appeared_member_count,
                disappeared_member_count,
                proposal.member_count(),
            );

            evidence.push(PerceptualProposalObservationEvidence {
                proposal: proposal.clone(),
                status,
                previous_present_count,
                current_present_count,
                stable_member_count,
                changed_member_count,
                appeared_member_count,
                disappeared_member_count,
            });
        }

        PerceptualProposalObservationResult { evidence }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalPerceptualProposalObservation;

impl UniversalPerceptualProposalObservation {
    pub fn evaluate(
        previous_frame: &PerceptualFrame,
        current_frame: &PerceptualFrame,
        proposals: &[PerceptualObjectProposal],
    ) -> PerceptualProposalObservationResult {
        PerceptualProposalObservation::observe(previous_frame, current_frame, proposals)
    }
}

// -----------------------------------------------------------------------------
// Retained temporal evidence for perceptual proposals
// -----------------------------------------------------------------------------
//
// This state accumulates exact cross-frame observational evidence without
// promoting a proposal into ObjectHypothesis.
//
// Continued presence means only that the proposal's exact member handles were
// grounded on both sides of successive transitions. A changed signature does
// not destroy that temporal identity observation. Appearance, disappearance,
// or mixed membership interrupts the consecutive support chain but does not
// erase prior evidence.
//
// Objecthood remains a later, stronger semantic judgement.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PerceptualProposalTemporalSupportStatus {
    Unknown,
    InsufficientHistory,
    Supported,
    BoundaryInterrupted,
    MixedEvidence,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualProposalTemporalEvidencePolicy {
    minimum_consecutive_cross_frame_presence: usize,
}

impl PerceptualProposalTemporalEvidencePolicy {
    pub fn new(minimum_consecutive_cross_frame_presence: usize) -> Option<Self> {
        if minimum_consecutive_cross_frame_presence == 0 {
            return None;
        }

        Some(Self {
            minimum_consecutive_cross_frame_presence,
        })
    }

    pub fn minimum_consecutive_cross_frame_presence(self) -> usize {
        self.minimum_consecutive_cross_frame_presence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualProposalTemporalEvidenceRecord {
    proposal: PerceptualObjectProposal,
    observation_count: usize,
    stable_count: usize,
    changed_count: usize,
    appeared_count: usize,
    disappeared_count: usize,
    mixed_count: usize,
    consecutive_cross_frame_presence: usize,
    max_consecutive_cross_frame_presence: usize,
    last_status: PerceptualProposalObservationStatus,
}

impl PerceptualProposalTemporalEvidenceRecord {
    fn new(
        proposal: PerceptualObjectProposal,
        status: PerceptualProposalObservationStatus,
    ) -> Self {
        let mut record = Self {
            proposal,
            observation_count: 0,
            stable_count: 0,
            changed_count: 0,
            appeared_count: 0,
            disappeared_count: 0,
            mixed_count: 0,
            consecutive_cross_frame_presence: 0,
            max_consecutive_cross_frame_presence: 0,
            last_status: status,
        };

        record.observe(status);

        record
    }

    fn observe(&mut self, status: PerceptualProposalObservationStatus) {
        self.observation_count = self.observation_count.saturating_add(1);

        match status {
            PerceptualProposalObservationStatus::Stable => {
                self.stable_count = self.stable_count.saturating_add(1);

                self.consecutive_cross_frame_presence =
                    self.consecutive_cross_frame_presence.saturating_add(1);
            }
            PerceptualProposalObservationStatus::Changed => {
                self.changed_count = self.changed_count.saturating_add(1);

                self.consecutive_cross_frame_presence =
                    self.consecutive_cross_frame_presence.saturating_add(1);
            }
            PerceptualProposalObservationStatus::Appeared => {
                self.appeared_count = self.appeared_count.saturating_add(1);
                self.consecutive_cross_frame_presence = 0;
            }
            PerceptualProposalObservationStatus::Disappeared => {
                self.disappeared_count = self.disappeared_count.saturating_add(1);
                self.consecutive_cross_frame_presence = 0;
            }
            PerceptualProposalObservationStatus::Mixed => {
                self.mixed_count = self.mixed_count.saturating_add(1);
                self.consecutive_cross_frame_presence = 0;
            }
        }

        self.max_consecutive_cross_frame_presence = self
            .max_consecutive_cross_frame_presence
            .max(self.consecutive_cross_frame_presence);

        self.last_status = status;
    }

    pub fn proposal(&self) -> &PerceptualObjectProposal {
        &self.proposal
    }

    pub fn observation_count(&self) -> usize {
        self.observation_count
    }

    pub fn stable_count(&self) -> usize {
        self.stable_count
    }

    pub fn changed_count(&self) -> usize {
        self.changed_count
    }

    pub fn appeared_count(&self) -> usize {
        self.appeared_count
    }

    pub fn disappeared_count(&self) -> usize {
        self.disappeared_count
    }

    pub fn mixed_count(&self) -> usize {
        self.mixed_count
    }

    pub fn cross_frame_presence_count(&self) -> usize {
        self.stable_count.saturating_add(self.changed_count)
    }

    pub fn consecutive_cross_frame_presence(&self) -> usize {
        self.consecutive_cross_frame_presence
    }

    pub fn max_consecutive_cross_frame_presence(&self) -> usize {
        self.max_consecutive_cross_frame_presence
    }

    pub fn last_status(&self) -> PerceptualProposalObservationStatus {
        self.last_status
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PerceptualProposalTemporalEvidenceState {
    records: Vec<PerceptualProposalTemporalEvidenceRecord>,
}

impl PerceptualProposalTemporalEvidenceState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn records(&self) -> &[PerceptualProposalTemporalEvidenceRecord] {
        &self.records
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn record(
        &self,
        proposal: &PerceptualObjectProposal,
    ) -> Option<&PerceptualProposalTemporalEvidenceRecord> {
        self.records
            .binary_search_by(|record| record.proposal().cmp(proposal))
            .ok()
            .map(|index| &self.records[index])
    }

    pub fn observe(&mut self, result: &PerceptualProposalObservationResult) {
        for evidence in result.evidence() {
            match self
                .records
                .binary_search_by(|record| record.proposal().cmp(evidence.proposal()))
            {
                Ok(index) => {
                    self.records[index].observe(evidence.status());
                }
                Err(index) => {
                    self.records.insert(
                        index,
                        PerceptualProposalTemporalEvidenceRecord::new(
                            evidence.proposal().clone(),
                            evidence.status(),
                        ),
                    );
                }
            }
        }
    }

    pub fn support_status(
        &self,
        proposal: &PerceptualObjectProposal,
        policy: PerceptualProposalTemporalEvidencePolicy,
    ) -> PerceptualProposalTemporalSupportStatus {
        let Some(record) = self.record(proposal) else {
            return PerceptualProposalTemporalSupportStatus::Unknown;
        };

        match record.last_status() {
            PerceptualProposalObservationStatus::Appeared
            | PerceptualProposalObservationStatus::Disappeared => {
                PerceptualProposalTemporalSupportStatus::BoundaryInterrupted
            }
            PerceptualProposalObservationStatus::Mixed => {
                PerceptualProposalTemporalSupportStatus::MixedEvidence
            }
            PerceptualProposalObservationStatus::Stable
            | PerceptualProposalObservationStatus::Changed => {
                if record.consecutive_cross_frame_presence()
                    >= policy.minimum_consecutive_cross_frame_presence()
                {
                    PerceptualProposalTemporalSupportStatus::Supported
                } else {
                    PerceptualProposalTemporalSupportStatus::InsufficientHistory
                }
            }
        }
    }

    pub fn supported_records(
        &self,
        policy: PerceptualProposalTemporalEvidencePolicy,
    ) -> Vec<&PerceptualProposalTemporalEvidenceRecord> {
        self.records
            .iter()
            .filter(|record| {
                self.support_status(record.proposal(), policy)
                    == PerceptualProposalTemporalSupportStatus::Supported
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalPerceptualProposalTemporalEvidence;

impl UniversalPerceptualProposalTemporalEvidence {
    pub fn observe(
        state: &mut PerceptualProposalTemporalEvidenceState,
        result: &PerceptualProposalObservationResult,
    ) {
        state.observe(result);
    }

    pub fn support_status(
        state: &PerceptualProposalTemporalEvidenceState,
        proposal: &PerceptualObjectProposal,
        policy: PerceptualProposalTemporalEvidencePolicy,
    ) -> PerceptualProposalTemporalSupportStatus {
        state.support_status(proposal, policy)
    }
}

#[cfg(test)]
mod perceptual_proposal_temporal_evidence_tests {
    use super::*;

    fn atom(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn frame(observation_index: u64, elements: &[(u64, u64)]) -> PerceptualFrame {
        PerceptualFrame::new(
            observation_index,
            elements
                .iter()
                .map(|(handle, signature)| {
                    PerceptualElement::new(PerceptualElementHandle::new(*handle), atom(*signature))
                })
                .collect(),
        )
        .expect("test frame is valid")
    }

    fn proposal(handle: u64) -> PerceptualObjectProposal {
        PerceptualObjectProposal::new(vec![PerceptualElementHandle::new(handle)])
            .expect("test proposal is valid")
    }

    fn observe(
        previous: &PerceptualFrame,
        current: &PerceptualFrame,
        proposal: &PerceptualObjectProposal,
    ) -> PerceptualProposalObservationResult {
        PerceptualProposalObservation::observe(previous, current, &[proposal.clone()])
    }

    #[test]
    fn repeated_cross_frame_presence_becomes_temporally_supported() {
        let p = proposal(1);
        let policy = PerceptualProposalTemporalEvidencePolicy::new(2).unwrap();

        let f1 = frame(1, &[(1, 10)]);
        let f2 = frame(2, &[(1, 10)]);
        let f3 = frame(3, &[(1, 20)]);

        let mut state = PerceptualProposalTemporalEvidenceState::new();

        state.observe(&observe(&f1, &f2, &p));

        assert_eq!(
            state.support_status(&p, policy),
            PerceptualProposalTemporalSupportStatus::InsufficientHistory
        );

        state.observe(&observe(&f2, &f3, &p));

        assert_eq!(
            state.support_status(&p, policy),
            PerceptualProposalTemporalSupportStatus::Supported
        );

        let record = state.record(&p).expect("record must exist");

        assert_eq!(record.observation_count(), 2);
        assert_eq!(record.stable_count(), 1);
        assert_eq!(record.changed_count(), 1);
        assert_eq!(record.cross_frame_presence_count(), 2);
        assert_eq!(record.consecutive_cross_frame_presence(), 2);
        assert_eq!(record.max_consecutive_cross_frame_presence(), 2);

        assert_eq!(state.supported_records(policy).len(), 1);
    }

    #[test]
    fn boundary_interrupts_current_support_without_destroying_history_and_support_can_recover() {
        let p = proposal(1);
        let policy = PerceptualProposalTemporalEvidencePolicy::new(2).unwrap();

        let f1 = frame(1, &[(1, 10), (2, 90)]);
        let f2 = frame(2, &[(1, 10), (2, 90)]);
        let f3 = frame(3, &[(1, 20), (2, 90)]);
        let f4 = frame(4, &[(2, 90)]);
        let f5 = frame(5, &[(1, 30), (2, 90)]);
        let f6 = frame(6, &[(1, 30), (2, 90)]);
        let f7 = frame(7, &[(1, 40), (2, 90)]);

        let mut state = PerceptualProposalTemporalEvidenceState::new();

        state.observe(&observe(&f1, &f2, &p));
        state.observe(&observe(&f2, &f3, &p));

        assert_eq!(
            state.support_status(&p, policy),
            PerceptualProposalTemporalSupportStatus::Supported
        );

        state.observe(&observe(&f3, &f4, &p));

        assert_eq!(
            state.support_status(&p, policy),
            PerceptualProposalTemporalSupportStatus::BoundaryInterrupted
        );

        let interrupted = state.record(&p).unwrap();

        assert_eq!(interrupted.max_consecutive_cross_frame_presence(), 2);
        assert_eq!(interrupted.consecutive_cross_frame_presence(), 0);
        assert_eq!(interrupted.disappeared_count(), 1);

        state.observe(&observe(&f4, &f5, &p));

        assert_eq!(
            state.support_status(&p, policy),
            PerceptualProposalTemporalSupportStatus::BoundaryInterrupted
        );

        state.observe(&observe(&f5, &f6, &p));

        assert_eq!(
            state.support_status(&p, policy),
            PerceptualProposalTemporalSupportStatus::InsufficientHistory
        );

        state.observe(&observe(&f6, &f7, &p));

        assert_eq!(
            state.support_status(&p, policy),
            PerceptualProposalTemporalSupportStatus::Supported
        );

        let recovered = state.record(&p).unwrap();

        assert_eq!(recovered.max_consecutive_cross_frame_presence(), 2);
        assert_eq!(recovered.consecutive_cross_frame_presence(), 2);
        assert_eq!(recovered.appeared_count(), 1);
        assert_eq!(recovered.disappeared_count(), 1);
    }

    #[test]
    fn unseen_proposal_remains_unknown() {
        let state = PerceptualProposalTemporalEvidenceState::new();

        let policy = PerceptualProposalTemporalEvidencePolicy::new(2).unwrap();

        assert_eq!(
            state.support_status(&proposal(999), policy),
            PerceptualProposalTemporalSupportStatus::Unknown
        );
    }
}

// -----------------------------------------------------------------------------
// Competing structural grouping proposal frontier
// -----------------------------------------------------------------------------
//
// A grouping candidate is not an object hypothesis.
//
// This layer combines two independently grounded facts:
//
//   1. atomic perceptual identities have retained temporal support;
//   2. the caller supplies an explicit structural relation between identities.
//
// The output is a bounded competing grouping frontier. It makes no claim about
// cohesion, objecthood, containment, common change, causality, or semantic
// identity. Promotion into ObjectHypothesis remains downstream and requires
// explicit ObjecthoodEvidence.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroupingRelation {
    left: PerceptualElementHandle,
    right: PerceptualElementHandle,
}

impl PerceptualGroupingRelation {
    pub fn new(left: PerceptualElementHandle, right: PerceptualElementHandle) -> Option<Self> {
        if left == right {
            return None;
        }

        let (left, right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };

        Some(Self { left, right })
    }

    pub fn left(self) -> PerceptualElementHandle {
        self.left
    }

    pub fn right(self) -> PerceptualElementHandle {
        self.right
    }

    pub fn is_grounded_in(self, frame: &PerceptualFrame) -> bool {
        frame.contains_handle(self.left) && frame.contains_handle(self.right)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PerceptualGroupingCandidateKind {
    PairwiseRelation,
    ConnectedComponent,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroupingCandidate {
    members: Vec<PerceptualElementHandle>,
    kind: PerceptualGroupingCandidateKind,
}

impl PerceptualGroupingCandidate {
    pub fn new(
        mut members: Vec<PerceptualElementHandle>,
        kind: PerceptualGroupingCandidateKind,
    ) -> Option<Self> {
        members.sort_unstable();
        members.dedup();

        if members.len() < 2 {
            return None;
        }

        Some(Self { members, kind })
    }

    pub fn members(&self) -> &[PerceptualElementHandle] {
        &self.members
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn kind(&self) -> PerceptualGroupingCandidateKind {
        self.kind
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
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroupingGenerationPolicy {
    max_relations: usize,
    max_candidates: usize,
}

impl PerceptualGroupingGenerationPolicy {
    pub fn new(max_relations: usize, max_candidates: usize) -> Option<Self> {
        if max_relations == 0 || max_candidates == 0 {
            return None;
        }

        Some(Self {
            max_relations,
            max_candidates,
        })
    }

    pub fn max_relations(self) -> usize {
        self.max_relations
    }

    pub fn max_candidates(self) -> usize {
        self.max_candidates
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualGroupingGenerationResult {
    input_relation_count: usize,
    considered_relation_count: usize,
    relation_frontier_truncated: bool,
    rejected_ungrounded_relation_count: usize,
    rejected_temporal_support_count: usize,
    admitted_relation_count: usize,
    pairwise_candidate_count: usize,
    component_candidate_count: usize,
    candidate_count_before_frontier: usize,
    candidate_frontier_truncated: bool,
    candidates: Vec<PerceptualGroupingCandidate>,
}

impl PerceptualGroupingGenerationResult {
    pub fn input_relation_count(&self) -> usize {
        self.input_relation_count
    }

    pub fn considered_relation_count(&self) -> usize {
        self.considered_relation_count
    }

    pub fn relation_frontier_truncated(&self) -> bool {
        self.relation_frontier_truncated
    }

    pub fn rejected_ungrounded_relation_count(&self) -> usize {
        self.rejected_ungrounded_relation_count
    }

    pub fn rejected_temporal_support_count(&self) -> usize {
        self.rejected_temporal_support_count
    }

    pub fn admitted_relation_count(&self) -> usize {
        self.admitted_relation_count
    }

    pub fn pairwise_candidate_count(&self) -> usize {
        self.pairwise_candidate_count
    }

    pub fn component_candidate_count(&self) -> usize {
        self.component_candidate_count
    }

    pub fn candidate_count_before_frontier(&self) -> usize {
        self.candidate_count_before_frontier
    }

    pub fn candidate_frontier_truncated(&self) -> bool {
        self.candidate_frontier_truncated
    }

    pub fn candidates(&self) -> &[PerceptualGroupingCandidate] {
        &self.candidates
    }

    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroupingFrontierGeneration;

impl PerceptualGroupingFrontierGeneration {
    pub fn generate(
        frame: &PerceptualFrame,
        temporal_state: &PerceptualProposalTemporalEvidenceState,
        temporal_policy: PerceptualProposalTemporalEvidencePolicy,
        relations: &[PerceptualGroupingRelation],
        policy: PerceptualGroupingGenerationPolicy,
    ) -> PerceptualGroupingGenerationResult {
        let input_relation_count = relations.len();

        let mut canonical_relations = relations.to_vec();
        canonical_relations.sort_unstable();
        canonical_relations.dedup();

        let unique_relation_count = canonical_relations.len();

        canonical_relations.truncate(policy.max_relations());

        let considered_relation_count = canonical_relations.len();
        let relation_frontier_truncated = unique_relation_count > considered_relation_count;

        let mut rejected_ungrounded_relation_count = 0_usize;
        let mut rejected_temporal_support_count = 0_usize;
        let mut admitted_relations = Vec::new();

        for relation in canonical_relations {
            if !relation.is_grounded_in(frame) {
                rejected_ungrounded_relation_count =
                    rejected_ungrounded_relation_count.saturating_add(1);
                continue;
            }

            let left_proposal = PerceptualObjectProposal::new(vec![relation.left()])
                .expect("single relation endpoint forms a valid atomic proposal");

            let right_proposal = PerceptualObjectProposal::new(vec![relation.right()])
                .expect("single relation endpoint forms a valid atomic proposal");

            let left_supported = temporal_state.support_status(&left_proposal, temporal_policy)
                == PerceptualProposalTemporalSupportStatus::Supported;

            let right_supported = temporal_state.support_status(&right_proposal, temporal_policy)
                == PerceptualProposalTemporalSupportStatus::Supported;

            if !left_supported || !right_supported {
                rejected_temporal_support_count = rejected_temporal_support_count.saturating_add(1);
                continue;
            }

            admitted_relations.push(relation);
        }

        let admitted_relation_count = admitted_relations.len();

        let mut candidates = admitted_relations
            .iter()
            .copied()
            .map(|relation| {
                PerceptualGroupingCandidate::new(
                    vec![relation.left(), relation.right()],
                    PerceptualGroupingCandidateKind::PairwiseRelation,
                )
                .expect("admitted pairwise relation has two distinct members")
            })
            .collect::<Vec<_>>();

        let pairwise_candidate_count = candidates.len();

        /*
         * Connected components are an alternative structural explanation
         * over the same admitted relation graph.
         *
         * They compete with pairwise groupings rather than replacing them.
         * A component is emitted only when it contains at least three members;
         * a two-member component would duplicate its pairwise candidate.
         */
        let mut remaining = admitted_relations
            .iter()
            .flat_map(|relation| [relation.left(), relation.right()])
            .collect::<Vec<_>>();

        remaining.sort_unstable();
        remaining.dedup();

        let mut component_candidate_count = 0_usize;

        while let Some(seed) = remaining.first().copied() {
            remaining.remove(0);

            let mut component = vec![seed];
            let mut frontier = vec![seed];

            while let Some(current) = frontier.pop() {
                for relation in &admitted_relations {
                    let neighbor = if relation.left() == current {
                        Some(relation.right())
                    } else if relation.right() == current {
                        Some(relation.left())
                    } else {
                        None
                    };

                    let Some(neighbor) = neighbor else {
                        continue;
                    };

                    if let Ok(index) = remaining.binary_search(&neighbor) {
                        remaining.remove(index);
                        component.push(neighbor);
                        frontier.push(neighbor);
                    }
                }
            }

            component.sort_unstable();
            component.dedup();

            if component.len() >= 3 {
                candidates.push(
                    PerceptualGroupingCandidate::new(
                        component,
                        PerceptualGroupingCandidateKind::ConnectedComponent,
                    )
                    .expect("component candidate has at least three members"),
                );

                component_candidate_count = component_candidate_count.saturating_add(1);
            }
        }

        candidates.sort();
        candidates.dedup();

        let candidate_count_before_frontier = candidates.len();

        candidates.truncate(policy.max_candidates());

        let candidate_frontier_truncated = candidate_count_before_frontier > candidates.len();

        PerceptualGroupingGenerationResult {
            input_relation_count,
            considered_relation_count,
            relation_frontier_truncated,
            rejected_ungrounded_relation_count,
            rejected_temporal_support_count,
            admitted_relation_count,
            pairwise_candidate_count,
            component_candidate_count,
            candidate_count_before_frontier,
            candidate_frontier_truncated,
            candidates,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalPerceptualGroupingFrontierGeneration;

impl UniversalPerceptualGroupingFrontierGeneration {
    pub fn evaluate(
        frame: &PerceptualFrame,
        temporal_state: &PerceptualProposalTemporalEvidenceState,
        temporal_policy: PerceptualProposalTemporalEvidencePolicy,
        relations: &[PerceptualGroupingRelation],
        policy: PerceptualGroupingGenerationPolicy,
    ) -> PerceptualGroupingGenerationResult {
        PerceptualGroupingFrontierGeneration::generate(
            frame,
            temporal_state,
            temporal_policy,
            relations,
            policy,
        )
    }
}

// -----------------------------------------------------------------------------
// Exact grouping behavior evidence
// -----------------------------------------------------------------------------
//
// A grouping candidate is still only a structural proposal.
//
// This layer asks a narrower empirical question:
//
//     did the candidate's grounded members behave coherently
//     across this exact observed transition?
//
// Uniform change is evidence of common change.
// Uniform stability is recorded separately.
// Mixed member behavior is explicit disagreement.
// Appearance/disappearance is an observed boundary interruption.
//
// None of these outcomes promotes the grouping into ObjectHypothesis and none
// fabricates ObjecthoodEvidence.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PerceptualGroupingBehaviorStatus {
    UniformStable,
    UniformChanged,
    BoundaryInterrupted,
    Mixed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualGroupingBehaviorEvidence {
    candidate: PerceptualGroupingCandidate,
    status: PerceptualGroupingBehaviorStatus,
    stable_member_count: usize,
    changed_member_count: usize,
    boundary_member_count: usize,
    mixed_member_count: usize,
}

impl PerceptualGroupingBehaviorEvidence {
    pub fn candidate(&self) -> &PerceptualGroupingCandidate {
        &self.candidate
    }

    pub fn status(&self) -> PerceptualGroupingBehaviorStatus {
        self.status
    }

    pub fn stable_member_count(&self) -> usize {
        self.stable_member_count
    }

    pub fn changed_member_count(&self) -> usize {
        self.changed_member_count
    }

    pub fn boundary_member_count(&self) -> usize {
        self.boundary_member_count
    }

    pub fn mixed_member_count(&self) -> usize {
        self.mixed_member_count
    }

    pub fn member_count(&self) -> usize {
        self.candidate.member_count()
    }

    pub fn is_uniform_change(&self) -> bool {
        self.status == PerceptualGroupingBehaviorStatus::UniformChanged
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualGroupingBehaviorObservationResult {
    input_candidate_count: usize,
    evaluated_candidate_count: usize,
    skipped_incomplete_atomic_evidence_count: usize,
    uniform_stable_count: usize,
    uniform_changed_count: usize,
    boundary_interrupted_count: usize,
    mixed_count: usize,
    evidence: Vec<PerceptualGroupingBehaviorEvidence>,
}

impl PerceptualGroupingBehaviorObservationResult {
    pub fn input_candidate_count(&self) -> usize {
        self.input_candidate_count
    }

    pub fn evaluated_candidate_count(&self) -> usize {
        self.evaluated_candidate_count
    }

    pub fn skipped_incomplete_atomic_evidence_count(&self) -> usize {
        self.skipped_incomplete_atomic_evidence_count
    }

    pub fn uniform_stable_count(&self) -> usize {
        self.uniform_stable_count
    }

    pub fn uniform_changed_count(&self) -> usize {
        self.uniform_changed_count
    }

    pub fn boundary_interrupted_count(&self) -> usize {
        self.boundary_interrupted_count
    }

    pub fn mixed_count(&self) -> usize {
        self.mixed_count
    }

    pub fn evidence(&self) -> &[PerceptualGroupingBehaviorEvidence] {
        &self.evidence
    }

    pub fn evidence_count(&self) -> usize {
        self.evidence.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroupingBehaviorObservation;

impl PerceptualGroupingBehaviorObservation {
    pub fn observe(
        candidates: &[PerceptualGroupingCandidate],
        atomic_observation: &PerceptualProposalObservationResult,
    ) -> PerceptualGroupingBehaviorObservationResult {
        let input_candidate_count = candidates.len();

        let mut evidence = Vec::with_capacity(candidates.len());
        let mut skipped_incomplete_atomic_evidence_count = 0_usize;

        for candidate in candidates {
            let mut stable_member_count = 0_usize;
            let mut changed_member_count = 0_usize;
            let mut boundary_member_count = 0_usize;
            let mut mixed_member_count = 0_usize;
            let mut complete = true;

            for handle in candidate.members() {
                let atomic_proposal = PerceptualObjectProposal::new(vec![*handle])
                    .expect("one exact member always forms a valid atomic proposal");

                let Some(member_evidence) = atomic_observation
                    .evidence()
                    .iter()
                    .find(|item| item.proposal() == &atomic_proposal)
                else {
                    complete = false;
                    break;
                };

                match member_evidence.status() {
                    PerceptualProposalObservationStatus::Stable => {
                        stable_member_count = stable_member_count.saturating_add(1);
                    }
                    PerceptualProposalObservationStatus::Changed => {
                        changed_member_count = changed_member_count.saturating_add(1);
                    }
                    PerceptualProposalObservationStatus::Appeared
                    | PerceptualProposalObservationStatus::Disappeared => {
                        boundary_member_count = boundary_member_count.saturating_add(1);
                    }
                    PerceptualProposalObservationStatus::Mixed => {
                        mixed_member_count = mixed_member_count.saturating_add(1);
                    }
                }
            }

            if !complete {
                skipped_incomplete_atomic_evidence_count =
                    skipped_incomplete_atomic_evidence_count.saturating_add(1);
                continue;
            }

            let member_count = candidate.member_count();

            let status = if changed_member_count == member_count {
                PerceptualGroupingBehaviorStatus::UniformChanged
            } else if stable_member_count == member_count {
                PerceptualGroupingBehaviorStatus::UniformStable
            } else if boundary_member_count > 0 {
                PerceptualGroupingBehaviorStatus::BoundaryInterrupted
            } else {
                PerceptualGroupingBehaviorStatus::Mixed
            };

            evidence.push(PerceptualGroupingBehaviorEvidence {
                candidate: candidate.clone(),
                status,
                stable_member_count,
                changed_member_count,
                boundary_member_count,
                mixed_member_count,
            });
        }

        let uniform_stable_count = evidence
            .iter()
            .filter(|item| item.status() == PerceptualGroupingBehaviorStatus::UniformStable)
            .count();

        let uniform_changed_count = evidence
            .iter()
            .filter(|item| item.status() == PerceptualGroupingBehaviorStatus::UniformChanged)
            .count();

        let boundary_interrupted_count = evidence
            .iter()
            .filter(|item| item.status() == PerceptualGroupingBehaviorStatus::BoundaryInterrupted)
            .count();

        let mixed_count = evidence
            .iter()
            .filter(|item| item.status() == PerceptualGroupingBehaviorStatus::Mixed)
            .count();

        PerceptualGroupingBehaviorObservationResult {
            input_candidate_count,
            evaluated_candidate_count: evidence.len(),
            skipped_incomplete_atomic_evidence_count,
            uniform_stable_count,
            uniform_changed_count,
            boundary_interrupted_count,
            mixed_count,
            evidence,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalPerceptualGroupingBehaviorObservation;

impl UniversalPerceptualGroupingBehaviorObservation {
    pub fn evaluate(
        candidates: &[PerceptualGroupingCandidate],
        atomic_observation: &PerceptualProposalObservationResult,
    ) -> PerceptualGroupingBehaviorObservationResult {
        PerceptualGroupingBehaviorObservation::observe(candidates, atomic_observation)
    }
}

#[cfg(test)]
mod perceptual_grouping_behavior_observation_tests {
    use super::*;

    fn atom(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn frame(observation_index: u64, elements: &[(u64, u64)]) -> PerceptualFrame {
        PerceptualFrame::new(
            observation_index,
            elements
                .iter()
                .map(|(handle, signature)| {
                    PerceptualElement::new(PerceptualElementHandle::new(*handle), atom(*signature))
                })
                .collect(),
        )
        .expect("test frame is valid")
    }

    fn atomic(handle: u64) -> PerceptualObjectProposal {
        PerceptualObjectProposal::new(vec![PerceptualElementHandle::new(handle)])
            .expect("atomic proposal is valid")
    }

    fn grouping(handles: &[u64]) -> PerceptualGroupingCandidate {
        PerceptualGroupingCandidate::new(
            handles
                .iter()
                .copied()
                .map(PerceptualElementHandle::new)
                .collect(),
            PerceptualGroupingCandidateKind::PairwiseRelation,
        )
        .expect("grouping candidate is valid")
    }

    #[test]
    fn grouping_behavior_distinguishes_uniform_change_from_mixed_behavior() {
        let previous = frame(1, &[(1, 10), (2, 20), (3, 30)]);

        let current = frame(2, &[(1, 11), (2, 21), (3, 30)]);

        let atomic_observation = PerceptualProposalObservation::observe(
            &previous,
            &current,
            &[atomic(1), atomic(2), atomic(3)],
        );

        let uniformly_changed = grouping(&[1, 2]);
        let mixed = grouping(&[1, 3]);

        let result = PerceptualGroupingBehaviorObservation::observe(
            &[uniformly_changed.clone(), mixed.clone()],
            &atomic_observation,
        );

        assert_eq!(result.input_candidate_count(), 2);
        assert_eq!(result.evaluated_candidate_count(), 2);
        assert_eq!(result.skipped_incomplete_atomic_evidence_count(), 0);

        assert_eq!(result.uniform_changed_count(), 1);
        assert_eq!(result.uniform_stable_count(), 0);
        assert_eq!(result.boundary_interrupted_count(), 0);
        assert_eq!(result.mixed_count(), 1);

        let changed_evidence = result
            .evidence()
            .iter()
            .find(|item| item.candidate() == &uniformly_changed)
            .expect("uniformly changed candidate must be evaluated");

        assert_eq!(
            changed_evidence.status(),
            PerceptualGroupingBehaviorStatus::UniformChanged
        );
        assert_eq!(changed_evidence.changed_member_count(), 2);
        assert_eq!(changed_evidence.stable_member_count(), 0);
        assert!(changed_evidence.is_uniform_change());

        let mixed_evidence = result
            .evidence()
            .iter()
            .find(|item| item.candidate() == &mixed)
            .expect("mixed candidate must be evaluated");

        assert_eq!(
            mixed_evidence.status(),
            PerceptualGroupingBehaviorStatus::Mixed
        );
        assert_eq!(mixed_evidence.changed_member_count(), 1);
        assert_eq!(mixed_evidence.stable_member_count(), 1);
    }

    #[test]
    fn grouping_behavior_records_uniform_stability_without_claiming_common_change() {
        let previous = frame(10, &[(1, 10), (2, 20)]);

        let current = frame(11, &[(1, 10), (2, 20)]);

        let atomic_observation =
            PerceptualProposalObservation::observe(&previous, &current, &[atomic(1), atomic(2)]);

        let candidate = grouping(&[1, 2]);

        let result = PerceptualGroupingBehaviorObservation::observe(
            &[candidate.clone()],
            &atomic_observation,
        );

        assert_eq!(result.uniform_stable_count(), 1);
        assert_eq!(result.uniform_changed_count(), 0);

        let item = &result.evidence()[0];

        assert_eq!(item.candidate(), &candidate);
        assert_eq!(
            item.status(),
            PerceptualGroupingBehaviorStatus::UniformStable
        );
        assert_eq!(item.stable_member_count(), 2);
        assert!(!item.is_uniform_change());
    }

    #[test]
    fn grouping_behavior_skips_candidate_when_atomic_evidence_is_incomplete() {
        let previous = frame(20, &[(1, 10), (2, 20)]);

        let current = frame(21, &[(1, 11), (2, 21)]);

        /*
         * Only member 1 is supplied to the atomic evidence frontier.
         * Group [1,2] therefore cannot receive fabricated group evidence.
         */
        let atomic_observation =
            PerceptualProposalObservation::observe(&previous, &current, &[atomic(1)]);

        let result = PerceptualGroupingBehaviorObservation::observe(
            &[grouping(&[1, 2])],
            &atomic_observation,
        );

        assert_eq!(result.input_candidate_count(), 1);
        assert_eq!(result.evaluated_candidate_count(), 0);
        assert_eq!(result.evidence_count(), 0);
        assert_eq!(result.skipped_incomplete_atomic_evidence_count(), 1);
    }
}

// -----------------------------------------------------------------------------
// Retained grouping behavior evidence
// -----------------------------------------------------------------------------
//
// This state remembers repeated empirical behavior of structural grouping
// candidates.
//
// UniformChanged is positive evidence for common change.
// Mixed is contradictory evidence.
// UniformStable is neutral with respect to common change.
// BoundaryInterrupted prevents current support while the grouping is not
// jointly grounded.
//
// Support requires:
// - a minimum number of observed common-change events; and
// - a minimum advantage of common-change evidence over mixed evidence.
//
// Therefore an isolated contradiction can weaken a candidate without
// permanently destroying stronger accumulated evidence, while ties and weak
// majorities remain epistemically unresolved.
//
// This layer still does NOT construct ObjecthoodEvidence or ObjectHypothesis.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PerceptualGroupingBehaviorSupportStatus {
    Unknown,
    InsufficientCommonChangeEvidence,
    Supported,
    Conflicted,
    BoundaryInterrupted,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroupingBehaviorRetentionPolicy {
    minimum_common_change_observations: usize,
    minimum_common_change_advantage_over_mixed: usize,
}

impl PerceptualGroupingBehaviorRetentionPolicy {
    pub fn new(
        minimum_common_change_observations: usize,
        minimum_common_change_advantage_over_mixed: usize,
    ) -> Option<Self> {
        if minimum_common_change_observations == 0
            || minimum_common_change_advantage_over_mixed == 0
        {
            return None;
        }

        Some(Self {
            minimum_common_change_observations,
            minimum_common_change_advantage_over_mixed,
        })
    }

    pub fn minimum_common_change_observations(self) -> usize {
        self.minimum_common_change_observations
    }

    pub fn minimum_common_change_advantage_over_mixed(self) -> usize {
        self.minimum_common_change_advantage_over_mixed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualGroupingBehaviorEvidenceRecord {
    candidate: PerceptualGroupingCandidate,
    observation_count: usize,
    uniform_stable_count: usize,
    uniform_changed_count: usize,
    boundary_interrupted_count: usize,
    mixed_count: usize,
    last_status: PerceptualGroupingBehaviorStatus,
}

impl PerceptualGroupingBehaviorEvidenceRecord {
    fn new(
        candidate: PerceptualGroupingCandidate,
        status: PerceptualGroupingBehaviorStatus,
    ) -> Self {
        let mut record = Self {
            candidate,
            observation_count: 0,
            uniform_stable_count: 0,
            uniform_changed_count: 0,
            boundary_interrupted_count: 0,
            mixed_count: 0,
            last_status: status,
        };

        record.observe(status);

        record
    }

    fn observe(&mut self, status: PerceptualGroupingBehaviorStatus) {
        self.observation_count = self.observation_count.saturating_add(1);

        match status {
            PerceptualGroupingBehaviorStatus::UniformStable => {
                self.uniform_stable_count = self.uniform_stable_count.saturating_add(1);
            }
            PerceptualGroupingBehaviorStatus::UniformChanged => {
                self.uniform_changed_count = self.uniform_changed_count.saturating_add(1);
            }
            PerceptualGroupingBehaviorStatus::BoundaryInterrupted => {
                self.boundary_interrupted_count = self.boundary_interrupted_count.saturating_add(1);
            }
            PerceptualGroupingBehaviorStatus::Mixed => {
                self.mixed_count = self.mixed_count.saturating_add(1);
            }
        }

        self.last_status = status;
    }

    pub fn candidate(&self) -> &PerceptualGroupingCandidate {
        &self.candidate
    }

    pub fn observation_count(&self) -> usize {
        self.observation_count
    }

    pub fn uniform_stable_count(&self) -> usize {
        self.uniform_stable_count
    }

    pub fn uniform_changed_count(&self) -> usize {
        self.uniform_changed_count
    }

    pub fn boundary_interrupted_count(&self) -> usize {
        self.boundary_interrupted_count
    }

    pub fn mixed_count(&self) -> usize {
        self.mixed_count
    }

    pub fn last_status(&self) -> PerceptualGroupingBehaviorStatus {
        self.last_status
    }

    pub fn common_change_advantage_over_mixed(&self) -> usize {
        self.uniform_changed_count.saturating_sub(self.mixed_count)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PerceptualGroupingBehaviorEvidenceState {
    records: Vec<PerceptualGroupingBehaviorEvidenceRecord>,
}

impl PerceptualGroupingBehaviorEvidenceState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn records(&self) -> &[PerceptualGroupingBehaviorEvidenceRecord] {
        &self.records
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn record(
        &self,
        candidate: &PerceptualGroupingCandidate,
    ) -> Option<&PerceptualGroupingBehaviorEvidenceRecord> {
        self.records
            .binary_search_by(|record| record.candidate().cmp(candidate))
            .ok()
            .map(|index| &self.records[index])
    }

    pub fn observe(&mut self, result: &PerceptualGroupingBehaviorObservationResult) {
        for evidence in result.evidence() {
            match self
                .records
                .binary_search_by(|record| record.candidate().cmp(evidence.candidate()))
            {
                Ok(index) => {
                    self.records[index].observe(evidence.status());
                }
                Err(index) => {
                    self.records.insert(
                        index,
                        PerceptualGroupingBehaviorEvidenceRecord::new(
                            evidence.candidate().clone(),
                            evidence.status(),
                        ),
                    );
                }
            }
        }
    }

    pub fn support_status(
        &self,
        candidate: &PerceptualGroupingCandidate,
        policy: PerceptualGroupingBehaviorRetentionPolicy,
    ) -> PerceptualGroupingBehaviorSupportStatus {
        let Some(record) = self.record(candidate) else {
            return PerceptualGroupingBehaviorSupportStatus::Unknown;
        };

        if record.last_status() == PerceptualGroupingBehaviorStatus::BoundaryInterrupted {
            return PerceptualGroupingBehaviorSupportStatus::BoundaryInterrupted;
        }

        if record.uniform_changed_count() < policy.minimum_common_change_observations() {
            return PerceptualGroupingBehaviorSupportStatus::InsufficientCommonChangeEvidence;
        }

        let required_common_change = record
            .mixed_count()
            .saturating_add(policy.minimum_common_change_advantage_over_mixed());

        if record.uniform_changed_count() >= required_common_change {
            PerceptualGroupingBehaviorSupportStatus::Supported
        } else {
            PerceptualGroupingBehaviorSupportStatus::Conflicted
        }
    }

    pub fn supported_records(
        &self,
        policy: PerceptualGroupingBehaviorRetentionPolicy,
    ) -> Vec<&PerceptualGroupingBehaviorEvidenceRecord> {
        self.records
            .iter()
            .filter(|record| {
                self.support_status(record.candidate(), policy)
                    == PerceptualGroupingBehaviorSupportStatus::Supported
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalPerceptualGroupingBehaviorEvidence;

impl UniversalPerceptualGroupingBehaviorEvidence {
    pub fn observe(
        state: &mut PerceptualGroupingBehaviorEvidenceState,
        result: &PerceptualGroupingBehaviorObservationResult,
    ) {
        state.observe(result);
    }

    pub fn support_status(
        state: &PerceptualGroupingBehaviorEvidenceState,
        candidate: &PerceptualGroupingCandidate,
        policy: PerceptualGroupingBehaviorRetentionPolicy,
    ) -> PerceptualGroupingBehaviorSupportStatus {
        state.support_status(candidate, policy)
    }
}

#[cfg(test)]
mod retained_perceptual_grouping_behavior_evidence_tests {
    use super::*;

    fn atom(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn frame(observation_index: u64, elements: &[(u64, u64)]) -> PerceptualFrame {
        PerceptualFrame::new(
            observation_index,
            elements
                .iter()
                .map(|(handle, signature)| {
                    PerceptualElement::new(PerceptualElementHandle::new(*handle), atom(*signature))
                })
                .collect(),
        )
        .expect("test frame is valid")
    }

    fn atomic(handle: u64) -> PerceptualObjectProposal {
        PerceptualObjectProposal::new(vec![PerceptualElementHandle::new(handle)])
            .expect("atomic proposal is valid")
    }

    fn candidate() -> PerceptualGroupingCandidate {
        PerceptualGroupingCandidate::new(
            vec![
                PerceptualElementHandle::new(1),
                PerceptualElementHandle::new(2),
            ],
            PerceptualGroupingCandidateKind::PairwiseRelation,
        )
        .expect("test grouping is valid")
    }

    fn behavior(
        previous: &PerceptualFrame,
        current: &PerceptualFrame,
        candidate: &PerceptualGroupingCandidate,
    ) -> PerceptualGroupingBehaviorObservationResult {
        let atomic_observation =
            PerceptualProposalObservation::observe(previous, current, &[atomic(1), atomic(2)]);

        PerceptualGroupingBehaviorObservation::observe(&[candidate.clone()], &atomic_observation)
    }

    #[test]
    fn repeated_common_change_becomes_retained_support() {
        let grouping = candidate();

        let policy = PerceptualGroupingBehaviorRetentionPolicy::new(2, 1).unwrap();

        let f1 = frame(1, &[(1, 10), (2, 20)]);
        let f2 = frame(2, &[(1, 11), (2, 21)]);
        let f3 = frame(3, &[(1, 12), (2, 22)]);

        let mut state = PerceptualGroupingBehaviorEvidenceState::new();

        state.observe(&behavior(&f1, &f2, &grouping));

        assert_eq!(
            state.support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::InsufficientCommonChangeEvidence
        );

        state.observe(&behavior(&f2, &f3, &grouping));

        assert_eq!(
            state.support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::Supported
        );

        let record = state
            .record(&grouping)
            .expect("grouping behavior record must exist");

        assert_eq!(record.observation_count(), 2);
        assert_eq!(record.uniform_changed_count(), 2);
        assert_eq!(record.mixed_count(), 0);
        assert_eq!(record.common_change_advantage_over_mixed(), 2);

        assert_eq!(state.supported_records(policy).len(), 1);
    }

    #[test]
    fn mixed_evidence_can_conflict_and_later_common_change_can_recover_support() {
        let grouping = candidate();

        /*
         * Require a two-observation advantage over mixed evidence.
         *
         * 2 changed / 0 mixed -> supported
         * 2 changed / 1 mixed -> conflicted
         * 3 changed / 1 mixed -> supported again
         */
        let policy = PerceptualGroupingBehaviorRetentionPolicy::new(2, 2).unwrap();

        let f1 = frame(1, &[(1, 10), (2, 20)]);
        let f2 = frame(2, &[(1, 11), (2, 21)]);
        let f3 = frame(3, &[(1, 12), (2, 22)]);

        let mixed_current = frame(4, &[(1, 13), (2, 22)]);

        let recovered_current = frame(5, &[(1, 14), (2, 23)]);

        let mut state = PerceptualGroupingBehaviorEvidenceState::new();

        state.observe(&behavior(&f1, &f2, &grouping));
        state.observe(&behavior(&f2, &f3, &grouping));

        assert_eq!(
            state.support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::Supported
        );

        state.observe(&behavior(&f3, &mixed_current, &grouping));

        assert_eq!(
            state.support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::Conflicted
        );

        let conflicted = state.record(&grouping).unwrap();

        assert_eq!(conflicted.uniform_changed_count(), 2);
        assert_eq!(conflicted.mixed_count(), 1);
        assert_eq!(conflicted.common_change_advantage_over_mixed(), 1);

        state.observe(&behavior(&mixed_current, &recovered_current, &grouping));

        assert_eq!(
            state.support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::Supported,
            "later coherent evidence must be able to recover from isolated mixed evidence"
        );

        let recovered = state.record(&grouping).unwrap();

        assert_eq!(recovered.uniform_changed_count(), 3);
        assert_eq!(recovered.mixed_count(), 1);
        assert_eq!(recovered.common_change_advantage_over_mixed(), 2);
    }

    #[test]
    fn boundary_interruption_blocks_current_grouping_support_without_erasing_history() {
        let grouping = candidate();

        let policy = PerceptualGroupingBehaviorRetentionPolicy::new(2, 1).unwrap();

        let f1 = frame(1, &[(1, 10), (2, 20)]);
        let f2 = frame(2, &[(1, 11), (2, 21)]);
        let f3 = frame(3, &[(1, 12), (2, 22)]);
        let boundary = frame(4, &[(1, 13)]);

        let mut state = PerceptualGroupingBehaviorEvidenceState::new();

        state.observe(&behavior(&f1, &f2, &grouping));
        state.observe(&behavior(&f2, &f3, &grouping));

        assert_eq!(
            state.support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::Supported
        );

        state.observe(&behavior(&f3, &boundary, &grouping));

        assert_eq!(
            state.support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::BoundaryInterrupted
        );

        let record = state.record(&grouping).unwrap();

        assert_eq!(record.uniform_changed_count(), 2);
        assert_eq!(record.boundary_interrupted_count(), 1);
        assert_eq!(
            record.common_change_advantage_over_mixed(),
            2,
            "historical common-change evidence must survive the boundary interruption"
        );
    }

    #[test]
    fn unseen_grouping_remains_unknown() {
        let state = PerceptualGroupingBehaviorEvidenceState::new();

        let policy = PerceptualGroupingBehaviorRetentionPolicy::new(2, 1).unwrap();

        assert_eq!(
            state.support_status(&candidate(), policy),
            PerceptualGroupingBehaviorSupportStatus::Unknown
        );
    }
}

// -----------------------------------------------------------------------------
// Multi-axis objecthood eligibility
// -----------------------------------------------------------------------------
//
// This gate deliberately stops before semantic object promotion.
//
// A structural grouping is merely ELIGIBLE for future ObjectHypothesis
// construction when multiple independent evidence families agree:
//
// - retained temporal persistence;
// - retained common-change behavior;
// - current perceptual appearance cohesion;
// - an explicit contrast boundary.
//
// No CognitiveSignal is synthesized here. Therefore no unsupported confidence
// value is smuggled into ObjecthoodEvidence.
//
// P4F-B may later translate independently grounded evidence into calibrated
// objecthood axes, but only after this eligibility behavior is functionally
// validated.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualObjectPromotionEvidence {
    temporal_persistence: bool,
    common_change: bool,
    appearance_cohesion: bool,
    contrast_boundary: bool,
}

impl PerceptualObjectPromotionEvidence {
    pub fn new(
        temporal_persistence: bool,
        common_change: bool,
        appearance_cohesion: bool,
        contrast_boundary: bool,
    ) -> Self {
        Self {
            temporal_persistence,
            common_change,
            appearance_cohesion,
            contrast_boundary,
        }
    }

    pub fn temporal_persistence(self) -> bool {
        self.temporal_persistence
    }

    pub fn common_change(self) -> bool {
        self.common_change
    }

    pub fn appearance_cohesion(self) -> bool {
        self.appearance_cohesion
    }

    pub fn contrast_boundary(self) -> bool {
        self.contrast_boundary
    }

    pub fn all_required_axes_supported(self) -> bool {
        self.temporal_persistence
            && self.common_change
            && self.appearance_cohesion
            && self.contrast_boundary
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualObjectPromotionCandidate {
    grouping: PerceptualGroupingCandidate,
    evidence: PerceptualObjectPromotionEvidence,
}

impl PerceptualObjectPromotionCandidate {
    pub fn grouping(&self) -> &PerceptualGroupingCandidate {
        &self.grouping
    }

    pub fn evidence(&self) -> PerceptualObjectPromotionEvidence {
        self.evidence
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualObjectPromotionGate;

impl PerceptualObjectPromotionGate {
    pub fn evaluate(
        grouping: PerceptualGroupingCandidate,
        evidence: PerceptualObjectPromotionEvidence,
    ) -> Option<PerceptualObjectPromotionCandidate> {
        if !evidence.all_required_axes_supported() {
            return None;
        }

        Some(PerceptualObjectPromotionCandidate { grouping, evidence })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalPerceptualObjectPromotionGate;

impl UniversalPerceptualObjectPromotionGate {
    pub fn evaluate(
        grouping: PerceptualGroupingCandidate,
        evidence: PerceptualObjectPromotionEvidence,
    ) -> Option<PerceptualObjectPromotionCandidate> {
        PerceptualObjectPromotionGate::evaluate(grouping, evidence)
    }
}

// -----------------------------------------------------------------------------
// Retained grouping appearance evidence
// -----------------------------------------------------------------------------
//
// P4F-A established a sufficient multi-axis eligibility route.
//
// P4F-B now gives the visual axes retained empirical history rather than
// converting one boolean observation into arbitrary maximal confidence.
//
// Each observation is one explicit opportunity:
// - appearance_cohesion_supported: candidate members currently satisfy the
//   adapter's exact cohesion criterion;
// - contrast_boundary_supported: candidate currently satisfies the adapter's
//   exact exterior-contrast criterion.
//
// A visual axis is historically supported only when:
// - at least the minimum number of observations exists; and
// - support strictly exceeds contradiction.
//
// This is deliberately symmetric and does not encode ARC colors or game rules.

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroupingAppearanceObservationEvidence {
    candidate: PerceptualGroupingCandidate,
    appearance_cohesion_supported: bool,
    contrast_boundary_supported: bool,
}

impl PerceptualGroupingAppearanceObservationEvidence {
    pub fn new(
        candidate: PerceptualGroupingCandidate,
        appearance_cohesion_supported: bool,
        contrast_boundary_supported: bool,
    ) -> Self {
        Self {
            candidate,
            appearance_cohesion_supported,
            contrast_boundary_supported,
        }
    }

    pub fn candidate(&self) -> &PerceptualGroupingCandidate {
        &self.candidate
    }

    pub fn appearance_cohesion_supported(&self) -> bool {
        self.appearance_cohesion_supported
    }

    pub fn contrast_boundary_supported(&self) -> bool {
        self.contrast_boundary_supported
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PerceptualGroupingAppearanceObservationResult {
    evidence: Vec<PerceptualGroupingAppearanceObservationEvidence>,
}

impl PerceptualGroupingAppearanceObservationResult {
    pub fn new(mut evidence: Vec<PerceptualGroupingAppearanceObservationEvidence>) -> Self {
        evidence.sort_by(|left, right| left.candidate().cmp(right.candidate()));

        evidence.dedup_by(|left, right| left.candidate() == right.candidate());

        Self { evidence }
    }

    pub fn evidence(&self) -> &[PerceptualGroupingAppearanceObservationEvidence] {
        &self.evidence
    }

    pub fn evidence_count(&self) -> usize {
        self.evidence.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PerceptualGroupingAppearanceSupportStatus {
    Unknown,
    InsufficientHistory,
    Supported,
    Conflicted,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerceptualGroupingAppearanceRetentionPolicy {
    minimum_observations: usize,
}

impl PerceptualGroupingAppearanceRetentionPolicy {
    pub fn new(minimum_observations: usize) -> Option<Self> {
        if minimum_observations == 0 {
            return None;
        }

        Some(Self {
            minimum_observations,
        })
    }

    pub fn minimum_observations(self) -> usize {
        self.minimum_observations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualGroupingAppearanceEvidenceRecord {
    candidate: PerceptualGroupingCandidate,
    observation_count: usize,
    appearance_cohesion_support_count: usize,
    contrast_boundary_support_count: usize,
}

impl PerceptualGroupingAppearanceEvidenceRecord {
    fn new(evidence: &PerceptualGroupingAppearanceObservationEvidence) -> Self {
        let mut record = Self {
            candidate: evidence.candidate().clone(),
            observation_count: 0,
            appearance_cohesion_support_count: 0,
            contrast_boundary_support_count: 0,
        };

        record.observe(evidence);

        record
    }

    fn observe(&mut self, evidence: &PerceptualGroupingAppearanceObservationEvidence) {
        self.observation_count = self.observation_count.saturating_add(1);

        if evidence.appearance_cohesion_supported() {
            self.appearance_cohesion_support_count =
                self.appearance_cohesion_support_count.saturating_add(1);
        }

        if evidence.contrast_boundary_supported() {
            self.contrast_boundary_support_count =
                self.contrast_boundary_support_count.saturating_add(1);
        }
    }

    pub fn candidate(&self) -> &PerceptualGroupingCandidate {
        &self.candidate
    }

    pub fn observation_count(&self) -> usize {
        self.observation_count
    }

    pub fn appearance_cohesion_support_count(&self) -> usize {
        self.appearance_cohesion_support_count
    }

    pub fn contrast_boundary_support_count(&self) -> usize {
        self.contrast_boundary_support_count
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PerceptualGroupingAppearanceEvidenceState {
    records: Vec<PerceptualGroupingAppearanceEvidenceRecord>,
}

impl PerceptualGroupingAppearanceEvidenceState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn records(&self) -> &[PerceptualGroupingAppearanceEvidenceRecord] {
        &self.records
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn record(
        &self,
        candidate: &PerceptualGroupingCandidate,
    ) -> Option<&PerceptualGroupingAppearanceEvidenceRecord> {
        self.records
            .binary_search_by(|record| record.candidate().cmp(candidate))
            .ok()
            .map(|index| &self.records[index])
    }

    pub fn observe(&mut self, result: &PerceptualGroupingAppearanceObservationResult) {
        for evidence in result.evidence() {
            match self
                .records
                .binary_search_by(|record| record.candidate().cmp(evidence.candidate()))
            {
                Ok(index) => {
                    self.records[index].observe(evidence);
                }
                Err(index) => {
                    self.records.insert(
                        index,
                        PerceptualGroupingAppearanceEvidenceRecord::new(evidence),
                    );
                }
            }
        }
    }

    pub fn support_status(
        &self,
        candidate: &PerceptualGroupingCandidate,
        policy: PerceptualGroupingAppearanceRetentionPolicy,
    ) -> PerceptualGroupingAppearanceSupportStatus {
        let Some(record) = self.record(candidate) else {
            return PerceptualGroupingAppearanceSupportStatus::Unknown;
        };

        if record.observation_count() < policy.minimum_observations() {
            return PerceptualGroupingAppearanceSupportStatus::InsufficientHistory;
        }

        let cohesion_contradiction_count = record
            .observation_count()
            .saturating_sub(record.appearance_cohesion_support_count());

        let boundary_contradiction_count = record
            .observation_count()
            .saturating_sub(record.contrast_boundary_support_count());

        let cohesion_supported =
            record.appearance_cohesion_support_count() > cohesion_contradiction_count;

        let boundary_supported =
            record.contrast_boundary_support_count() > boundary_contradiction_count;

        if cohesion_supported && boundary_supported {
            PerceptualGroupingAppearanceSupportStatus::Supported
        } else {
            PerceptualGroupingAppearanceSupportStatus::Conflicted
        }
    }
}

// -----------------------------------------------------------------------------
// Exact empirical support -> CognitiveSignal calibration
// -----------------------------------------------------------------------------
//
// The signal is not a hand-authored confidence.
//
// It is exactly:
//
//      floor(1000 * support_count / opportunity_count)
//
// Invalid count relations abstain.
//
// A zero-support empirical history yields CognitiveSignal::zero().

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EmpiricalObjecthoodSignalCalibration;

impl EmpiricalObjecthoodSignalCalibration {
    pub fn from_counts(support_count: usize, opportunity_count: usize) -> Option<CognitiveSignal> {
        if opportunity_count == 0 || support_count > opportunity_count {
            return None;
        }

        if support_count == 0 {
            return Some(CognitiveSignal::zero());
        }

        let scaled = (support_count as u128).saturating_mul(1000) / (opportunity_count as u128);

        let value = u16::try_from(scaled).ok()?;

        CognitiveSignal::new(value)
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActionSource {
    SelfGenerated,
    ObservedExternal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionObservation {
    event_index: u64,
    source: ActionSource,
    descriptor: CognitiveStructure,
}

impl ActionObservation {
    pub fn new(event_index: u64, source: ActionSource, descriptor: CognitiveStructure) -> Self {
        Self {
            event_index,
            source,
            descriptor,
        }
    }

    pub fn event_index(&self) -> u64 {
        self.event_index
    }

    pub fn source(&self) -> ActionSource {
        self.source
    }

    pub fn descriptor(&self) -> &CognitiveStructure {
        &self.descriptor
    }

    pub fn occurs_within(&self, transition: &ObjectTransitionObservation) -> bool {
        self.event_index >= transition.start_index() && self.event_index < transition.end_index()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ActionConsequenceEvidence {
    temporal_alignment: CognitiveSignal,
    action_change_association: CognitiveSignal,
    repeatability: CognitiveSignal,
    counterfactual_contrast: CognitiveSignal,
    outcome_specificity: CognitiveSignal,
    causal_lift: CognitiveSignal,
}

impl ActionConsequenceEvidence {
    pub fn new(
        temporal_alignment: CognitiveSignal,
        action_change_association: CognitiveSignal,
        repeatability: CognitiveSignal,
        counterfactual_contrast: CognitiveSignal,
        outcome_specificity: CognitiveSignal,
        causal_lift: CognitiveSignal,
    ) -> Self {
        Self {
            temporal_alignment,
            action_change_association,
            repeatability,
            counterfactual_contrast,
            outcome_specificity,
            causal_lift,
        }
    }

    pub fn temporal_alignment(self) -> CognitiveSignal {
        self.temporal_alignment
    }

    pub fn action_change_association(self) -> CognitiveSignal {
        self.action_change_association
    }

    pub fn repeatability(self) -> CognitiveSignal {
        self.repeatability
    }

    pub fn counterfactual_contrast(self) -> CognitiveSignal {
        self.counterfactual_contrast
    }

    pub fn outcome_specificity(self) -> CognitiveSignal {
        self.outcome_specificity
    }

    pub fn causal_lift(self) -> CognitiveSignal {
        self.causal_lift
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
            .expect("action consequence evidence has fixed nonempty axes")
    }

    pub fn consequence_score(self) -> CognitiveSignal {
        let axes = self.axes();

        let peak = u32::from(self.peak_support().value());

        let total = axes
            .into_iter()
            .map(|value| u32::from(value.value()))
            .sum::<u32>();

        let composite = peak.saturating_mul(2).saturating_add(total) / 8;

        CognitiveSignal::new(composite as u16)
            .expect("bounded consequence composite remains on signal scale")
    }

    fn axes(self) -> [CognitiveSignal; 6] {
        [
            self.temporal_alignment,
            self.action_change_association,
            self.repeatability,
            self.counterfactual_contrast,
            self.outcome_specificity,
            self.causal_lift,
        ]
    }

    fn canonical_key(self) -> [u16; 6] {
        [
            self.temporal_alignment.value(),
            self.action_change_association.value(),
            self.repeatability.value(),
            self.counterfactual_contrast.value(),
            self.outcome_specificity.value(),
            self.causal_lift.value(),
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionConsequenceHypothesis {
    action: ActionObservation,
    change: PerceptualChangeHypothesis,
    consequence_descriptor: CognitiveStructure,
    evidence: ActionConsequenceEvidence,
}

impl ActionConsequenceHypothesis {
    pub fn new(
        action: ActionObservation,
        change: PerceptualChangeHypothesis,
        consequence_descriptor: CognitiveStructure,
        evidence: ActionConsequenceEvidence,
    ) -> Option<Self> {
        if !action.occurs_within(change.transition()) || !evidence.has_support() {
            return None;
        }

        Some(Self {
            action,
            change,
            consequence_descriptor,
            evidence,
        })
    }

    pub fn action(&self) -> &ActionObservation {
        &self.action
    }

    pub fn change(&self) -> &PerceptualChangeHypothesis {
        &self.change
    }

    pub fn consequence_descriptor(&self) -> &CognitiveStructure {
        &self.consequence_descriptor
    }

    pub fn evidence(&self) -> ActionConsequenceEvidence {
        self.evidence
    }

    pub fn consequence_score(&self) -> CognitiveSignal {
        self.evidence.consequence_score()
    }

    fn same_change_identity(
        left: &PerceptualChangeHypothesis,
        right: &PerceptualChangeHypothesis,
    ) -> bool {
        left.transition() == right.transition()
            && left.kind() == right.kind()
            && left.reference() == right.reference()
            && left.descriptor() == right.descriptor()
    }

    fn same_consequence_identity(&self, other: &Self) -> bool {
        self.action == other.action
            && Self::same_change_identity(&self.change, &other.change)
            && self.consequence_descriptor == other.consequence_descriptor
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionConsequencePolicy {
    max_consequences_per_action: usize,
    max_actions_per_change: usize,
    max_total_consequences: usize,
}

impl ActionConsequencePolicy {
    pub fn new(
        max_consequences_per_action: usize,
        max_actions_per_change: usize,
        max_total_consequences: usize,
    ) -> Option<Self> {
        if max_consequences_per_action == 0
            || max_actions_per_change == 0
            || max_total_consequences == 0
        {
            return None;
        }

        Some(Self {
            max_consequences_per_action,
            max_actions_per_change,
            max_total_consequences,
        })
    }

    pub fn max_consequences_per_action(self) -> usize {
        self.max_consequences_per_action
    }

    pub fn max_actions_per_change(self) -> usize {
        self.max_actions_per_change
    }

    pub fn max_total_consequences(self) -> usize {
        self.max_total_consequences
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionConsequenceCompetitionResult {
    input_hypothesis_count: usize,
    canonical_hypothesis_count: usize,
    duplicate_hypothesis_count: usize,
    dropped_by_action_bound_count: usize,
    dropped_by_change_bound_count: usize,
    dropped_by_global_bound_count: usize,
    selected: Vec<ActionConsequenceHypothesis>,
}

impl ActionConsequenceCompetitionResult {
    pub fn input_hypothesis_count(&self) -> usize {
        self.input_hypothesis_count
    }

    pub fn canonical_hypothesis_count(&self) -> usize {
        self.canonical_hypothesis_count
    }

    pub fn duplicate_hypothesis_count(&self) -> usize {
        self.duplicate_hypothesis_count
    }

    pub fn dropped_by_action_bound_count(&self) -> usize {
        self.dropped_by_action_bound_count
    }

    pub fn dropped_by_change_bound_count(&self) -> usize {
        self.dropped_by_change_bound_count
    }

    pub fn dropped_by_global_bound_count(&self) -> usize {
        self.dropped_by_global_bound_count
    }

    pub fn selected(&self) -> &[ActionConsequenceHypothesis] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionConsequenceCompetition;

impl ActionConsequenceCompetition {
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

    fn compare_action(left: &ActionObservation, right: &ActionObservation) -> std::cmp::Ordering {
        left.event_index()
            .cmp(&right.event_index())
            .then_with(|| left.source().cmp(&right.source()))
            .then_with(|| Self::compare_structure(left.descriptor(), right.descriptor()))
    }

    fn compare_change(
        left: &PerceptualChangeHypothesis,
        right: &PerceptualChangeHypothesis,
    ) -> std::cmp::Ordering {
        left.transition()
            .cmp(right.transition())
            .then_with(|| left.kind().cmp(&right.kind()))
            .then_with(|| left.reference().cmp(&right.reference()))
            .then_with(|| Self::compare_structure(left.descriptor(), right.descriptor()))
    }

    fn ranking(
        left: &ActionConsequenceHypothesis,
        right: &ActionConsequenceHypothesis,
    ) -> std::cmp::Ordering {
        right
            .consequence_score()
            .cmp(&left.consequence_score())
            .then_with(|| {
                right
                    .evidence()
                    .peak_support()
                    .cmp(&left.evidence().peak_support())
            })
            .then_with(|| Self::compare_action(left.action(), right.action()))
            .then_with(|| Self::compare_change(left.change(), right.change()))
            .then_with(|| {
                Self::compare_structure(
                    left.consequence_descriptor(),
                    right.consequence_descriptor(),
                )
            })
            .then_with(|| {
                left.evidence()
                    .canonical_key()
                    .cmp(&right.evidence().canonical_key())
            })
    }

    fn action_selected_count(
        selected: &[ActionConsequenceHypothesis],
        action: &ActionObservation,
    ) -> usize {
        selected
            .iter()
            .filter(|candidate| candidate.action() == action)
            .count()
    }

    fn change_selected_count(
        selected: &[ActionConsequenceHypothesis],
        change: &PerceptualChangeHypothesis,
    ) -> usize {
        selected
            .iter()
            .filter(|candidate| {
                ActionConsequenceHypothesis::same_change_identity(candidate.change(), change)
            })
            .count()
    }

    pub fn select(
        candidates: &[ActionConsequenceHypothesis],
        policy: ActionConsequencePolicy,
    ) -> ActionConsequenceCompetitionResult {
        let input_hypothesis_count = candidates.len();

        let mut canonical: Vec<ActionConsequenceHypothesis> = Vec::new();

        let mut duplicate_hypothesis_count = 0_usize;

        for candidate in candidates {
            if let Some(position) = canonical
                .iter()
                .position(|existing| existing.same_consequence_identity(candidate))
            {
                duplicate_hypothesis_count = duplicate_hypothesis_count.saturating_add(1);

                let replace =
                    Self::ranking(candidate, &canonical[position]) == std::cmp::Ordering::Less;

                if replace {
                    canonical[position] = candidate.clone();
                }
            } else {
                canonical.push(candidate.clone());
            }
        }

        canonical.sort_by(Self::ranking);

        let canonical_hypothesis_count = canonical.len();

        let mut selected = Vec::with_capacity(
            policy
                .max_total_consequences()
                .min(canonical_hypothesis_count),
        );

        let mut dropped_by_action_bound_count = 0_usize;

        let mut dropped_by_change_bound_count = 0_usize;

        let mut dropped_by_global_bound_count = 0_usize;

        for (index, candidate) in canonical.into_iter().enumerate() {
            if selected.len() >= policy.max_total_consequences() {
                dropped_by_global_bound_count = dropped_by_global_bound_count
                    .saturating_add(canonical_hypothesis_count.saturating_sub(index));

                break;
            }

            if Self::action_selected_count(&selected, candidate.action())
                >= policy.max_consequences_per_action()
            {
                dropped_by_action_bound_count = dropped_by_action_bound_count.saturating_add(1);

                continue;
            }

            if Self::change_selected_count(&selected, candidate.change())
                >= policy.max_actions_per_change()
            {
                dropped_by_change_bound_count = dropped_by_change_bound_count.saturating_add(1);

                continue;
            }

            selected.push(candidate);
        }

        ActionConsequenceCompetitionResult {
            input_hypothesis_count,
            canonical_hypothesis_count,
            duplicate_hypothesis_count,
            dropped_by_action_bound_count,
            dropped_by_change_bound_count,
            dropped_by_global_bound_count,
            selected,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreKnowledgeActionConsequences;

impl CoreKnowledgeActionConsequences {
    pub fn evaluate(
        candidates: &[ActionConsequenceHypothesis],
        policy: ActionConsequencePolicy,
    ) -> ActionConsequenceCompetitionResult {
        ActionConsequenceCompetition::select(candidates, policy)
    }
}

impl PerceptualChangeHypothesis {
    pub fn same_identity_as(&self, other: &Self) -> bool {
        self.transition == other.transition
            && self.kind == other.kind
            && self.reference == other.reference
            && self.descriptor == other.descriptor
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedPerceptualWorldCandidates {
    previous_scene_candidates: Vec<SceneInterpretation>,
    current_scene_candidates: Vec<SceneInterpretation>,
    persistence_candidates: Vec<PersistenceLinkHypothesis>,
    topology_candidates: Vec<TopologicalRelationHypothesis>,
    change_candidates: Vec<PerceptualChangeHypothesis>,
    action_consequence_candidates: Vec<ActionConsequenceHypothesis>,
}

impl IntegratedPerceptualWorldCandidates {
    pub fn new(
        previous_scene_candidates: Vec<SceneInterpretation>,
        current_scene_candidates: Vec<SceneInterpretation>,
        persistence_candidates: Vec<PersistenceLinkHypothesis>,
        topology_candidates: Vec<TopologicalRelationHypothesis>,
        change_candidates: Vec<PerceptualChangeHypothesis>,
        action_consequence_candidates: Vec<ActionConsequenceHypothesis>,
    ) -> Self {
        Self {
            previous_scene_candidates,
            current_scene_candidates,
            persistence_candidates,
            topology_candidates,
            change_candidates,
            action_consequence_candidates,
        }
    }

    pub fn previous_scene_candidates(&self) -> &[SceneInterpretation] {
        &self.previous_scene_candidates
    }

    pub fn current_scene_candidates(&self) -> &[SceneInterpretation] {
        &self.current_scene_candidates
    }

    pub fn persistence_candidates(&self) -> &[PersistenceLinkHypothesis] {
        &self.persistence_candidates
    }

    pub fn topology_candidates(&self) -> &[TopologicalRelationHypothesis] {
        &self.topology_candidates
    }

    pub fn change_candidates(&self) -> &[PerceptualChangeHypothesis] {
        &self.change_candidates
    }

    pub fn action_consequence_candidates(&self) -> &[ActionConsequenceHypothesis] {
        &self.action_consequence_candidates
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedPerceptualWorldInput {
    previous_frame: PerceptualFrame,
    current_frame: PerceptualFrame,
    candidates: IntegratedPerceptualWorldCandidates,
}

impl IntegratedPerceptualWorldInput {
    pub fn new(
        previous_frame: PerceptualFrame,
        current_frame: PerceptualFrame,
        candidates: IntegratedPerceptualWorldCandidates,
    ) -> Option<Self> {
        if previous_frame.observation_index() >= current_frame.observation_index() {
            return None;
        }

        Some(Self {
            previous_frame,
            current_frame,
            candidates,
        })
    }

    pub fn previous_frame(&self) -> &PerceptualFrame {
        &self.previous_frame
    }

    pub fn current_frame(&self) -> &PerceptualFrame {
        &self.current_frame
    }

    pub fn candidates(&self) -> &IntegratedPerceptualWorldCandidates {
        &self.candidates
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IntegratedPerceptualWorldContext {
    scene_policy: PerceptualGroundingPolicy,
    persistence_policy: PersistenceTrackingPolicy,
    topology_policy: TopologicalRelationPolicy,
    change_policy: PerceptualChangePolicy,
    action_consequence_policy: ActionConsequencePolicy,
}

impl IntegratedPerceptualWorldContext {
    pub fn new(
        scene_policy: PerceptualGroundingPolicy,
        persistence_policy: PersistenceTrackingPolicy,
        topology_policy: TopologicalRelationPolicy,
        change_policy: PerceptualChangePolicy,
        action_consequence_policy: ActionConsequencePolicy,
    ) -> Self {
        Self {
            scene_policy,
            persistence_policy,
            topology_policy,
            change_policy,
            action_consequence_policy,
        }
    }

    pub fn scene_policy(self) -> PerceptualGroundingPolicy {
        self.scene_policy
    }

    pub fn persistence_policy(self) -> PersistenceTrackingPolicy {
        self.persistence_policy
    }

    pub fn topology_policy(self) -> TopologicalRelationPolicy {
        self.topology_policy
    }

    pub fn change_policy(self) -> PerceptualChangePolicy {
        self.change_policy
    }

    pub fn action_consequence_policy(self) -> ActionConsequencePolicy {
        self.action_consequence_policy
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedPerceptualWorldResult {
    previous_scene: SceneCompetitionResult,
    current_scene: SceneCompetitionResult,
    persistence: PersistenceTrackingResult,
    topology: TopologicalRelationCompetitionResult,
    changes: PerceptualChangeCompetitionResult,
    action_consequences: ActionConsequenceCompetitionResult,
    rejected_persistence_dependency_count: usize,
    rejected_topology_dependency_count: usize,
    rejected_change_dependency_count: usize,
    rejected_action_consequence_dependency_count: usize,
}

impl IntegratedPerceptualWorldResult {
    pub fn previous_scene(&self) -> &SceneCompetitionResult {
        &self.previous_scene
    }

    pub fn current_scene(&self) -> &SceneCompetitionResult {
        &self.current_scene
    }

    pub fn persistence(&self) -> &PersistenceTrackingResult {
        &self.persistence
    }

    pub fn topology(&self) -> &TopologicalRelationCompetitionResult {
        &self.topology
    }

    pub fn changes(&self) -> &PerceptualChangeCompetitionResult {
        &self.changes
    }

    pub fn action_consequences(&self) -> &ActionConsequenceCompetitionResult {
        &self.action_consequences
    }

    pub fn rejected_persistence_dependency_count(&self) -> usize {
        self.rejected_persistence_dependency_count
    }

    pub fn rejected_topology_dependency_count(&self) -> usize {
        self.rejected_topology_dependency_count
    }

    pub fn rejected_change_dependency_count(&self) -> usize {
        self.rejected_change_dependency_count
    }

    pub fn rejected_action_consequence_dependency_count(&self) -> usize {
        self.rejected_action_consequence_dependency_count
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IntegratedPerceptualWorld;

impl IntegratedPerceptualWorld {
    fn scene_observations(
        frame: &PerceptualFrame,
        result: &SceneCompetitionResult,
    ) -> Vec<ObjectObservation> {
        let mut observations = result
            .selected()
            .iter()
            .flat_map(|scene| scene.hypotheses().iter())
            .map(|hypothesis| {
                ObjectObservation::from_hypothesis(frame, hypothesis)
                    .expect("selected scene hypotheses are grounded in their frame")
            })
            .collect::<Vec<_>>();

        observations.sort();
        observations.dedup();

        observations
    }

    fn contains_observation(
        observations: &[ObjectObservation],
        target: &ObjectObservation,
    ) -> bool {
        observations.binary_search(target).is_ok()
    }

    fn selected_transitions(
        persistence: &PersistenceTrackingResult,
    ) -> Vec<ObjectTransitionObservation> {
        let mut transitions = persistence
            .selected()
            .iter()
            .map(ObjectTransitionObservation::from_persistence_link)
            .collect::<Vec<_>>();

        transitions.sort();
        transitions.dedup();

        transitions
    }

    fn contains_transition(
        transitions: &[ObjectTransitionObservation],
        target: &ObjectTransitionObservation,
    ) -> bool {
        transitions.binary_search(target).is_ok()
    }

    pub fn evaluate(
        input: &IntegratedPerceptualWorldInput,
        context: IntegratedPerceptualWorldContext,
    ) -> IntegratedPerceptualWorldResult {
        let candidates = input.candidates();

        let previous_scene = CompetingSceneInterpretations::select(
            input.previous_frame(),
            candidates.previous_scene_candidates(),
            context.scene_policy(),
        );

        let current_scene = CompetingSceneInterpretations::select(
            input.current_frame(),
            candidates.current_scene_candidates(),
            context.scene_policy(),
        );

        let previous_objects = Self::scene_observations(input.previous_frame(), &previous_scene);

        let current_objects = Self::scene_observations(input.current_frame(), &current_scene);

        let eligible_persistence = candidates
            .persistence_candidates()
            .iter()
            .filter(|candidate| {
                Self::contains_observation(&previous_objects, candidate.previous())
                    && Self::contains_observation(&current_objects, candidate.current())
            })
            .cloned()
            .collect::<Vec<_>>();

        let rejected_persistence_dependency_count = candidates
            .persistence_candidates()
            .len()
            .saturating_sub(eligible_persistence.len());

        let persistence =
            PersistenceTracking::select(&eligible_persistence, context.persistence_policy());

        let mut all_scene_objects = previous_objects.clone();

        all_scene_objects.extend(current_objects.iter().cloned());

        all_scene_objects.sort();
        all_scene_objects.dedup();

        let eligible_topology = candidates
            .topology_candidates()
            .iter()
            .filter(|candidate| {
                Self::contains_observation(&all_scene_objects, candidate.subject())
                    && Self::contains_observation(&all_scene_objects, candidate.object())
            })
            .cloned()
            .collect::<Vec<_>>();

        let rejected_topology_dependency_count = candidates
            .topology_candidates()
            .len()
            .saturating_sub(eligible_topology.len());

        let topology =
            TopologicalRelationCompetition::select(&eligible_topology, context.topology_policy());

        let selected_transitions = Self::selected_transitions(&persistence);

        let eligible_changes = candidates
            .change_candidates()
            .iter()
            .filter(|candidate| {
                if !Self::contains_transition(&selected_transitions, candidate.transition()) {
                    return false;
                }

                if let Some(reference) = candidate.reference() {
                    return Self::contains_transition(&selected_transitions, reference);
                }

                true
            })
            .cloned()
            .collect::<Vec<_>>();

        let rejected_change_dependency_count = candidates
            .change_candidates()
            .len()
            .saturating_sub(eligible_changes.len());

        let changes =
            PerceptualChangeCompetition::select(&eligible_changes, context.change_policy());

        let eligible_action_consequences = candidates
            .action_consequence_candidates()
            .iter()
            .filter(|candidate| {
                changes
                    .selected()
                    .iter()
                    .any(|selected_change| selected_change.same_identity_as(candidate.change()))
            })
            .cloned()
            .collect::<Vec<_>>();

        let rejected_action_consequence_dependency_count = candidates
            .action_consequence_candidates()
            .len()
            .saturating_sub(eligible_action_consequences.len());

        let action_consequences = ActionConsequenceCompetition::select(
            &eligible_action_consequences,
            context.action_consequence_policy(),
        );

        IntegratedPerceptualWorldResult {
            previous_scene,
            current_scene,
            persistence,
            topology,
            changes,
            action_consequences,
            rejected_persistence_dependency_count,
            rejected_topology_dependency_count,
            rejected_change_dependency_count,
            rejected_action_consequence_dependency_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreKnowledgePerceptualWorld;

impl CoreKnowledgePerceptualWorld {
    pub fn evaluate(
        input: &IntegratedPerceptualWorldInput,
        context: IntegratedPerceptualWorldContext,
    ) -> IntegratedPerceptualWorldResult {
        IntegratedPerceptualWorld::evaluate(input, context)
    }
}

// ============================================================================
// E5D — GROUNDED PERCEPTUAL STATE FACT PROJECTION
// ============================================================================
//
// This projection is intentionally conservative.
//
// State facts are derived only from:
//   1. exact CognitiveStructure signatures already present in PerceptualElement,
//   2. object membership already admitted by the selected grounded scene.
//
// No observation index, element handle, enum discriminant, synthetic tag atom,
// action descriptor, predicted outcome, or observed outcome is fabricated into
// a state fact.
//
// Object and scene composition use CognitiveStructure::Unordered directly.
// This preserves structural grouping without introducing a new symbolic
// namespace or benchmark-specific vocabulary.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GroundedPerceptualStateProjectionStatus {
    Projected,
    MissingPreviousScene,
    AmbiguousPreviousScene,
    MissingCurrentScene,
    AmbiguousCurrentScene,
    UngroundedPreviousScene,
    UngroundedCurrentScene,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedPerceptualStateProjection {
    previous_facts: Vec<CognitiveStructure>,
    current_facts: Vec<CognitiveStructure>,
}

impl GroundedPerceptualStateProjection {
    pub fn previous_facts(&self) -> &[CognitiveStructure] {
        &self.previous_facts
    }

    pub fn current_facts(&self) -> &[CognitiveStructure] {
        &self.current_facts
    }

    pub fn previous_fact_count(&self) -> usize {
        self.previous_facts.len()
    }

    pub fn current_fact_count(&self) -> usize {
        self.current_facts.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedPerceptualStateProjectionResult {
    status: GroundedPerceptualStateProjectionStatus,
    projection: Option<GroundedPerceptualStateProjection>,
}

impl GroundedPerceptualStateProjectionResult {
    fn rejected(status: GroundedPerceptualStateProjectionStatus) -> Self {
        Self {
            status,
            projection: None,
        }
    }

    pub fn status(&self) -> GroundedPerceptualStateProjectionStatus {
        self.status
    }

    pub fn projection(&self) -> Option<&GroundedPerceptualStateProjection> {
        self.projection.as_ref()
    }

    pub fn projected(&self) -> bool {
        self.status == GroundedPerceptualStateProjectionStatus::Projected
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GroundedPerceptualStateProjector;

impl GroundedPerceptualStateProjector {
    pub fn scene_facts(
        frame: &PerceptualFrame,
        scene: &SceneInterpretation,
    ) -> Option<Vec<CognitiveStructure>> {
        if !scene.is_grounded_in(frame) {
            return None;
        }

        let mut facts = Vec::new();
        let mut object_facts = Vec::with_capacity(scene.hypothesis_count());

        for hypothesis in scene.hypotheses() {
            let mut member_signatures = Vec::with_capacity(hypothesis.member_count());

            for handle in hypothesis.members() {
                let element = frame.element(*handle)?;
                let signature = element.signature().clone();

                facts.push(signature.clone());
                member_signatures.push(signature);
            }

            let object_fact = CognitiveStructure::unordered(member_signatures)?;

            facts.push(object_fact.clone());
            object_facts.push(object_fact);
        }

        let scene_fact = CognitiveStructure::unordered(object_facts)?;

        facts.push(scene_fact);
        facts.sort();
        facts.dedup();

        if facts.is_empty() {
            return None;
        }

        Some(facts)
    }

    pub fn project(
        input: &IntegratedPerceptualWorldInput,
        context: IntegratedPerceptualWorldContext,
    ) -> GroundedPerceptualStateProjectionResult {
        let world = CoreKnowledgePerceptualWorld::evaluate(input, context);

        let previous_scene = match world.previous_scene().selected() {
            [] => {
                return GroundedPerceptualStateProjectionResult::rejected(
                    GroundedPerceptualStateProjectionStatus::MissingPreviousScene,
                );
            }
            [scene] => scene,
            _ => {
                return GroundedPerceptualStateProjectionResult::rejected(
                    GroundedPerceptualStateProjectionStatus::AmbiguousPreviousScene,
                );
            }
        };

        let current_scene = match world.current_scene().selected() {
            [] => {
                return GroundedPerceptualStateProjectionResult::rejected(
                    GroundedPerceptualStateProjectionStatus::MissingCurrentScene,
                );
            }
            [scene] => scene,
            _ => {
                return GroundedPerceptualStateProjectionResult::rejected(
                    GroundedPerceptualStateProjectionStatus::AmbiguousCurrentScene,
                );
            }
        };

        let Some(previous_facts) = Self::scene_facts(input.previous_frame(), previous_scene) else {
            return GroundedPerceptualStateProjectionResult::rejected(
                GroundedPerceptualStateProjectionStatus::UngroundedPreviousScene,
            );
        };

        let Some(current_facts) = Self::scene_facts(input.current_frame(), current_scene) else {
            return GroundedPerceptualStateProjectionResult::rejected(
                GroundedPerceptualStateProjectionStatus::UngroundedCurrentScene,
            );
        };

        GroundedPerceptualStateProjectionResult {
            status: GroundedPerceptualStateProjectionStatus::Projected,
            projection: Some(GroundedPerceptualStateProjection {
                previous_facts,
                current_facts,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalGroundedPerceptualStateProjection;

impl UniversalGroundedPerceptualStateProjection {
    pub fn evaluate(
        input: &IntegratedPerceptualWorldInput,
        context: IntegratedPerceptualWorldContext,
    ) -> GroundedPerceptualStateProjectionResult {
        GroundedPerceptualStateProjector::project(input, context)
    }
}

#[cfg(test)]
mod grounded_perceptual_state_projection_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        CognitiveSignal::new(value).expect("test signal must be positive and bounded")
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn frame(observation_index: u64, elements: &[(u64, u64)]) -> PerceptualFrame {
        PerceptualFrame::new(
            observation_index,
            elements
                .iter()
                .map(|(handle, signature)| {
                    PerceptualElement::new(PerceptualElementHandle::new(*handle), a(*signature))
                })
                .collect(),
        )
        .expect("test frame is valid")
    }

    fn evidence() -> ObjecthoodEvidence {
        ObjecthoodEvidence::new(s(900), s(900), s(900), s(900), s(900), s(900))
    }

    fn scene(groups: &[Vec<u64>]) -> SceneInterpretation {
        let hypotheses = groups
            .iter()
            .map(|members| {
                ObjectHypothesis::new(
                    members
                        .iter()
                        .copied()
                        .map(PerceptualElementHandle::new)
                        .collect(),
                    evidence(),
                )
                .expect("test object hypothesis is grounded")
            })
            .collect();

        SceneInterpretation::new(hypotheses, s(900)).expect("test scene is valid")
    }

    fn context() -> IntegratedPerceptualWorldContext {
        IntegratedPerceptualWorldContext::new(
            PerceptualGroundingPolicy::new(8, 8).expect("scene policy is valid"),
            PersistenceTrackingPolicy::new(8, 8, 16).expect("persistence policy is valid"),
            TopologicalRelationPolicy::new(8, 16).expect("topology policy is valid"),
            PerceptualChangePolicy::new(8, 16).expect("change policy is valid"),
            ActionConsequencePolicy::new(8, 8, 16).expect("action consequence policy is valid"),
        )
    }

    fn input(
        previous_elements: &[(u64, u64)],
        current_elements: &[(u64, u64)],
        previous_groups: &[Vec<u64>],
        current_groups: &[Vec<u64>],
    ) -> IntegratedPerceptualWorldInput {
        IntegratedPerceptualWorldInput::new(
            frame(1, previous_elements),
            frame(3, current_elements),
            IntegratedPerceptualWorldCandidates::new(
                vec![scene(previous_groups)],
                vec![scene(current_groups)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        )
        .expect("test perceptual input is valid")
    }

    fn canonical(mut facts: Vec<CognitiveStructure>) -> Vec<CognitiveStructure> {
        facts.sort();
        facts.dedup();
        facts
    }

    #[test]
    fn exact_selected_scene_signatures_become_grounded_multilevel_facts() {
        let input = input(
            &[(1001, 10), (1002, 20)],
            &[(1001, 10), (1002, 30)],
            &[vec![1001, 1002]],
            &[vec![1001, 1002]],
        );

        let result = UniversalGroundedPerceptualStateProjection::evaluate(&input, context());

        assert_eq!(
            result.status(),
            GroundedPerceptualStateProjectionStatus::Projected
        );

        let projection = result
            .projection()
            .expect("projected result contains projection");

        let previous_object = CognitiveStructure::unordered(vec![a(10), a(20)])
            .expect("object structure is nonempty");

        let previous_scene = CognitiveStructure::unordered(vec![previous_object.clone()])
            .expect("scene structure is nonempty");

        let current_object = CognitiveStructure::unordered(vec![a(10), a(30)])
            .expect("object structure is nonempty");

        let current_scene = CognitiveStructure::unordered(vec![current_object.clone()])
            .expect("scene structure is nonempty");

        assert_eq!(
            projection.previous_facts(),
            canonical(vec![a(10), a(20), previous_object, previous_scene,]).as_slice()
        );

        assert_eq!(
            projection.current_facts(),
            canonical(vec![a(10), a(30), current_object, current_scene,]).as_slice()
        );
    }

    #[test]
    fn opaque_handles_and_observation_indices_are_not_fabricated_as_facts() {
        let input = input(
            &[(1001, 10), (1002, 20)],
            &[(1001, 10), (1002, 30)],
            &[vec![1001, 1002]],
            &[vec![1001, 1002]],
        );

        let result = UniversalGroundedPerceptualStateProjection::evaluate(&input, context());

        let projection = result.projection().expect("projection must succeed");

        for forbidden in [a(1), a(3), a(1001), a(1002)] {
            assert!(!projection.previous_facts().contains(&forbidden));
            assert!(!projection.current_facts().contains(&forbidden));
        }
    }

    #[test]
    fn scene_composition_preserves_repeated_object_structure_without_synthetic_tags() {
        let input = input(
            &[(1001, 10), (1002, 10)],
            &[(1001, 10), (1002, 10)],
            &[vec![1001], vec![1002]],
            &[vec![1001], vec![1002]],
        );

        let result = UniversalGroundedPerceptualStateProjection::evaluate(&input, context());

        let projection = result.projection().expect("projection must succeed");

        let object =
            CognitiveStructure::unordered(vec![a(10)]).expect("object structure is nonempty");

        let repeated_scene = CognitiveStructure::unordered(vec![object.clone(), object])
            .expect("scene structure is nonempty");

        assert!(projection.previous_facts().contains(&repeated_scene));

        assert!(projection.current_facts().contains(&repeated_scene));
    }

    #[test]
    fn missing_selected_scene_causes_epistemic_abstention() {
        let input = IntegratedPerceptualWorldInput::new(
            frame(1, &[(1001, 10)]),
            frame(3, &[(1001, 20)]),
            IntegratedPerceptualWorldCandidates::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        )
        .expect("test perceptual input is valid");

        let result = UniversalGroundedPerceptualStateProjection::evaluate(&input, context());

        assert_eq!(
            result.status(),
            GroundedPerceptualStateProjectionStatus::MissingPreviousScene
        );

        assert!(result.projection().is_none());
    }

    #[test]
    fn single_scene_facts_project_exact_grounded_current_scene() {
        let current = frame(
            41,
            &[(1001, 10), (1002, 20)],
        );
        let current_scene = scene(&[vec![1001, 1002]]);

        let facts =
            GroundedPerceptualStateProjector::scene_facts(
                &current,
                &current_scene,
            )
            .expect("grounded current scene must project exact perceptual facts");

        assert!(facts.contains(&a(10)));
        assert!(facts.contains(&a(20)));

        let object_fact =
            CognitiveStructure::unordered(vec![a(10), a(20)])
                .expect("grounded object fact is valid");

        assert!(
            facts.contains(&object_fact),
            "single-scene projection must preserve canonical object facts",
        );
    }

    #[test]
    fn single_scene_facts_reject_scene_not_grounded_in_current_frame() {
        let current = frame(
            51,
            &[(1001, 10)],
        );
        let stale_scene = scene(&[vec![1001, 1002]]);

        assert!(
            GroundedPerceptualStateProjector::scene_facts(
                &current,
                &stale_scene,
            )
            .is_none(),
            "historical or stale scene must not become current grounded state",
        );
    }

    #[test]
    fn projection_is_deterministic_and_non_mutating() {
        let input = input(
            &[(1001, 10), (1002, 20)],
            &[(1001, 10), (1002, 30)],
            &[vec![1001, 1002]],
            &[vec![1001, 1002]],
        );

        let before = input.clone();

        let first = GroundedPerceptualStateProjector::project(&input, context());

        let second = UniversalGroundedPerceptualStateProjection::evaluate(&input, context());

        assert_eq!(first, second);
        assert_eq!(input, before);
    }
}

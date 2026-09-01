pub const COGNITIVE_SIGNAL_SCALE: u16 = 1000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveSignal(u16);

impl CognitiveSignal {
    pub fn new(value: u16) -> Option<Self> {
        if value > COGNITIVE_SIGNAL_SCALE {
            return None;
        }

        Some(Self(value))
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn maximum() -> Self {
        Self(COGNITIVE_SIGNAL_SCALE)
    }

    pub fn value(self) -> u16 {
        self.0
    }

    pub fn learning_progress(previous_uncertainty: Self, current_uncertainty: Self) -> Self {
        if previous_uncertainty > current_uncertainty {
            Self(previous_uncertainty.0.saturating_sub(current_uncertainty.0))
        } else {
            Self::zero()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneSignalProfile {
    surprise: CognitiveSignal,
    uncertainty: CognitiveSignal,
    novelty: CognitiveSignal,
    learning_progress: CognitiveSignal,
    information_gain: CognitiveSignal,
}

impl MindstoneSignalProfile {
    pub fn new(
        surprise: CognitiveSignal,
        uncertainty: CognitiveSignal,
        novelty: CognitiveSignal,
        learning_progress: CognitiveSignal,
        information_gain: CognitiveSignal,
    ) -> Self {
        Self {
            surprise,
            uncertainty,
            novelty,
            learning_progress,
            information_gain,
        }
    }

    pub fn surprise(self) -> CognitiveSignal {
        self.surprise
    }

    pub fn uncertainty(self) -> CognitiveSignal {
        self.uncertainty
    }

    pub fn novelty(self) -> CognitiveSignal {
        self.novelty
    }

    pub fn learning_progress_signal(self) -> CognitiveSignal {
        self.learning_progress
    }

    pub fn information_gain(self) -> CognitiveSignal {
        self.information_gain
    }

    pub fn salience(self) -> CognitiveSalience {
        let values = [
            self.surprise,
            self.uncertainty,
            self.novelty,
            self.learning_progress,
            self.information_gain,
        ];

        let total = values
            .iter()
            .map(|signal| u32::from(signal.value()))
            .sum::<u32>();

        let peak = values
            .iter()
            .map(|signal| u32::from(signal.value()))
            .max()
            .unwrap_or(0);

        let composite = (peak.saturating_mul(2).saturating_add(total)) / 7;

        CognitiveSalience(composite as u16)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveSalience(u16);

impl CognitiveSalience {
    pub fn value(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveBudget {
    units: u32,
}

impl CognitiveBudget {
    pub fn new(units: u32) -> Option<Self> {
        if units == 0 {
            return None;
        }

        Some(Self { units })
    }

    pub fn units(self) -> u32 {
        self.units
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CognitiveAdmissionClass {
    Ignore,
    CheapUpdate,
    Deliberate,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SparseCognitionPolicy {
    cheap_threshold: CognitiveSignal,
    deliberate_threshold: CognitiveSignal,
    cheap_compute_units: u32,
    deliberate_compute_units: u32,
}

impl SparseCognitionPolicy {
    pub fn new(
        cheap_threshold: CognitiveSignal,
        deliberate_threshold: CognitiveSignal,
        cheap_compute_units: u32,
        deliberate_compute_units: u32,
    ) -> Option<Self> {
        if cheap_threshold >= deliberate_threshold {
            return None;
        }

        if cheap_compute_units == 0 {
            return None;
        }

        if deliberate_compute_units < cheap_compute_units {
            return None;
        }

        Some(Self {
            cheap_threshold,
            deliberate_threshold,
            cheap_compute_units,
            deliberate_compute_units,
        })
    }

    pub fn cheap_threshold(self) -> CognitiveSignal {
        self.cheap_threshold
    }

    pub fn deliberate_threshold(self) -> CognitiveSignal {
        self.deliberate_threshold
    }

    pub fn cheap_compute_units(self) -> u32 {
        self.cheap_compute_units
    }

    pub fn deliberate_compute_units(self) -> u32 {
        self.deliberate_compute_units
    }

    pub fn classify(self, profile: MindstoneSignalProfile) -> CognitiveAdmissionClass {
        let salience = profile.salience().value();

        if salience < self.cheap_threshold.value() {
            CognitiveAdmissionClass::Ignore
        } else if salience < self.deliberate_threshold.value() {
            CognitiveAdmissionClass::CheapUpdate
        } else {
            CognitiveAdmissionClass::Deliberate
        }
    }

    pub fn admit(
        self,
        profile: MindstoneSignalProfile,
        budget: CognitiveBudget,
    ) -> CognitiveAdmissionDecision {
        let salience = profile.salience();

        let class = self.classify(profile);

        let requested_units = match class {
            CognitiveAdmissionClass::Ignore => 0,
            CognitiveAdmissionClass::CheapUpdate => self.cheap_compute_units,
            CognitiveAdmissionClass::Deliberate => self.deliberate_compute_units,
        };

        let granted_units = requested_units.min(budget.units());

        CognitiveAdmissionDecision {
            class,
            salience,
            requested_units,
            granted_units,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveAdmissionDecision {
    class: CognitiveAdmissionClass,
    salience: CognitiveSalience,
    requested_units: u32,
    granted_units: u32,
}

impl CognitiveAdmissionDecision {
    pub fn class(self) -> CognitiveAdmissionClass {
        self.class
    }

    pub fn salience(self) -> CognitiveSalience {
        self.salience
    }

    pub fn requested_units(self) -> u32 {
        self.requested_units
    }

    pub fn granted_units(self) -> u32 {
        self.granted_units
    }

    pub fn is_admitted(self) -> bool {
        self.class != CognitiveAdmissionClass::Ignore
    }

    pub fn is_deliberative(self) -> bool {
        self.class == CognitiveAdmissionClass::Deliberate
    }

    pub fn is_budget_limited(self) -> bool {
        self.granted_units < self.requested_units
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneSparseCognition;

impl MindstoneSparseCognition {
    pub fn evaluate(
        profile: MindstoneSignalProfile,
        policy: SparseCognitionPolicy,
        budget: CognitiveBudget,
    ) -> CognitiveAdmissionDecision {
        policy.admit(profile, budget)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveFingerprint(u64);

impl CognitiveFingerprint {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoveltyMemory {
    capacity: usize,
    fingerprints: std::collections::VecDeque<CognitiveFingerprint>,
}

impl NoveltyMemory {
    pub fn new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            return None;
        }

        Some(Self {
            capacity,
            fingerprints: std::collections::VecDeque::new(),
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.fingerprints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    pub fn contains(&self, fingerprint: CognitiveFingerprint) -> bool {
        self.fingerprints.contains(&fingerprint)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NoveltyStatus {
    Known,
    Novel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoveltyGateResult {
    memory_before: NoveltyMemory,
    memory_after: NoveltyMemory,
    fingerprint: CognitiveFingerprint,
    evicted: Option<CognitiveFingerprint>,
    novelty_signal: CognitiveSignal,
    status: NoveltyStatus,
}

impl NoveltyGateResult {
    pub fn memory_before(&self) -> &NoveltyMemory {
        &self.memory_before
    }

    pub fn memory_after(&self) -> &NoveltyMemory {
        &self.memory_after
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn evicted(&self) -> Option<CognitiveFingerprint> {
        self.evicted
    }

    pub fn novelty_signal(&self) -> CognitiveSignal {
        self.novelty_signal
    }

    pub fn status(&self) -> NoveltyStatus {
        self.status
    }

    pub fn is_novel(&self) -> bool {
        self.status == NoveltyStatus::Novel
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NoveltyGate;

impl NoveltyGate {
    pub fn observe(memory: NoveltyMemory, fingerprint: CognitiveFingerprint) -> NoveltyGateResult {
        let memory_before = memory.clone();

        if memory.contains(fingerprint) {
            return NoveltyGateResult {
                memory_before,
                memory_after: memory,
                fingerprint,
                evicted: None,
                novelty_signal: CognitiveSignal::zero(),
                status: NoveltyStatus::Known,
            };
        }

        let mut memory_after = memory;

        let evicted = if memory_after.is_full() {
            memory_after.fingerprints.pop_front()
        } else {
            None
        };

        memory_after.fingerprints.push_back(fingerprint);

        NoveltyGateResult {
            memory_before,
            memory_after,
            fingerprint,
            evicted,
            novelty_signal: CognitiveSignal::maximum(),
            status: NoveltyStatus::Novel,
        }
    }
}

impl MindstoneSignalProfile {
    pub fn with_novelty(self, novelty: CognitiveSignal) -> Self {
        Self { novelty, ..self }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MindstoneNoveltyAdmissionResult {
    novelty: NoveltyGateResult,
    profile: MindstoneSignalProfile,
    decision: CognitiveAdmissionDecision,
}

impl MindstoneNoveltyAdmissionResult {
    pub fn novelty(&self) -> &NoveltyGateResult {
        &self.novelty
    }

    pub fn profile(&self) -> MindstoneSignalProfile {
        self.profile
    }

    pub fn decision(&self) -> CognitiveAdmissionDecision {
        self.decision
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneNoveltyGate;

impl MindstoneNoveltyGate {
    pub fn evaluate(
        memory: NoveltyMemory,
        fingerprint: CognitiveFingerprint,
        profile: MindstoneSignalProfile,
        policy: SparseCognitionPolicy,
        budget: CognitiveBudget,
    ) -> MindstoneNoveltyAdmissionResult {
        let novelty = NoveltyGate::observe(memory, fingerprint);

        let profile = profile.with_novelty(novelty.novelty_signal());

        let decision = policy.admit(profile, budget);

        MindstoneNoveltyAdmissionResult {
            novelty,
            profile,
            decision,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CognitiveStructure {
    Atom(u64),
    Ordered(Vec<CognitiveStructure>),
    Unordered(Vec<CognitiveStructure>),
}

impl CognitiveStructure {
    pub fn atom(value: u64) -> Self {
        Self::Atom(value)
    }

    pub fn ordered(children: Vec<CognitiveStructure>) -> Option<Self> {
        if children.is_empty() {
            return None;
        }

        Some(Self::Ordered(children))
    }

    pub fn unordered(mut children: Vec<CognitiveStructure>) -> Option<Self> {
        if children.is_empty() {
            return None;
        }

        children.sort();

        Some(Self::Unordered(children))
    }

    pub fn is_atom(&self) -> bool {
        matches!(self, Self::Atom(_))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StructuralHasher;

impl StructuralHasher {
    const OFFSET: u64 = 14_695_981_039_346_656_037;

    const PRIME: u64 = 1_099_511_628_211;

    const ATOM_TAG: u64 = 0xA1;

    const ORDERED_TAG: u64 = 0xB2;

    const UNORDERED_TAG: u64 = 0xC3;

    fn mix_u64(mut state: u64, value: u64) -> u64 {
        for byte in value.to_le_bytes() {
            state ^= u64::from(byte);

            state = state.wrapping_mul(Self::PRIME);
        }

        state
    }

    fn hash_into(structure: &CognitiveStructure, state: u64) -> u64 {
        match structure {
            CognitiveStructure::Atom(value) => {
                let state = Self::mix_u64(state, Self::ATOM_TAG);

                Self::mix_u64(state, *value)
            }

            CognitiveStructure::Ordered(children) => {
                let mut state = Self::mix_u64(state, Self::ORDERED_TAG);

                state = Self::mix_u64(state, children.len() as u64);

                for child in children {
                    state = Self::hash_into(child, state);
                }

                state
            }

            CognitiveStructure::Unordered(children) => {
                let mut state = Self::mix_u64(state, Self::UNORDERED_TAG);

                state = Self::mix_u64(state, children.len() as u64);

                for child in children {
                    state = Self::hash_into(child, state);
                }

                state
            }
        }
    }

    pub fn fingerprint(structure: &CognitiveStructure) -> CognitiveFingerprint {
        CognitiveFingerprint::new(Self::hash_into(structure, Self::OFFSET))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MindstoneStructuralNoveltyAdmissionResult {
    structure: CognitiveStructure,
    fingerprint: CognitiveFingerprint,
    admission: MindstoneNoveltyAdmissionResult,
}

impl MindstoneStructuralNoveltyAdmissionResult {
    pub fn structure(&self) -> &CognitiveStructure {
        &self.structure
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn admission(&self) -> &MindstoneNoveltyAdmissionResult {
        &self.admission
    }

    pub fn decision(&self) -> CognitiveAdmissionDecision {
        self.admission.decision()
    }

    pub fn is_novel(&self) -> bool {
        self.admission.novelty().is_novel()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneStructuralNoveltyGate;

impl MindstoneStructuralNoveltyGate {
    pub fn evaluate(
        memory: NoveltyMemory,
        structure: CognitiveStructure,
        profile: MindstoneSignalProfile,
        policy: SparseCognitionPolicy,
        budget: CognitiveBudget,
    ) -> MindstoneStructuralNoveltyAdmissionResult {
        let fingerprint = StructuralHasher::fingerprint(&structure);

        let admission =
            MindstoneNoveltyGate::evaluate(memory, fingerprint, profile, policy, budget);

        MindstoneStructuralNoveltyAdmissionResult {
            structure,
            fingerprint,
            admission,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingAggregate {
    fingerprint: CognitiveFingerprint,
    observation_count: u64,
    first_seen: u64,
    last_seen: u64,
    total_salience: u128,
    peak_salience: CognitiveSalience,
}

impl StreamingAggregate {
    fn new(
        fingerprint: CognitiveFingerprint,
        event_index: u64,
        salience: CognitiveSalience,
    ) -> Self {
        Self {
            fingerprint,
            observation_count: 1,
            first_seen: event_index,
            last_seen: event_index,
            total_salience: u128::from(salience.value()),
            peak_salience: salience,
        }
    }

    fn observe(&mut self, event_index: u64, salience: CognitiveSalience) {
        self.observation_count = self.observation_count.saturating_add(1);

        self.last_seen = event_index;

        self.total_salience = self
            .total_salience
            .saturating_add(u128::from(salience.value()));

        if salience > self.peak_salience {
            self.peak_salience = salience;
        }
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn observation_count(&self) -> u64 {
        self.observation_count
    }

    pub fn first_seen(&self) -> u64 {
        self.first_seen
    }

    pub fn last_seen(&self) -> u64 {
        self.last_seen
    }

    pub fn total_salience(&self) -> u128 {
        self.total_salience
    }

    pub fn mean_salience(&self) -> u16 {
        (self.total_salience / u128::from(self.observation_count)) as u16
    }

    pub fn peak_salience(&self) -> CognitiveSalience {
        self.peak_salience
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingAggregationState {
    capacity: usize,
    last_event_index: Option<u64>,
    aggregates: std::collections::BTreeMap<CognitiveFingerprint, StreamingAggregate>,
}

impl StreamingAggregationState {
    pub fn new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            return None;
        }

        Some(Self {
            capacity,
            last_event_index: None,
            aggregates: std::collections::BTreeMap::new(),
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.aggregates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.aggregates.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    pub fn last_event_index(&self) -> Option<u64> {
        self.last_event_index
    }

    pub fn contains(&self, fingerprint: CognitiveFingerprint) -> bool {
        self.aggregates.contains_key(&fingerprint)
    }

    pub fn aggregate(&self, fingerprint: CognitiveFingerprint) -> Option<&StreamingAggregate> {
        self.aggregates.get(&fingerprint)
    }

    pub fn total_retained_observations(&self) -> u64 {
        self.aggregates.values().fold(0_u64, |total, aggregate| {
            total.saturating_add(aggregate.observation_count())
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StreamingAggregationStatus {
    RejectedOutOfOrder,
    Inserted,
    Updated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingAggregationResult {
    state_before: StreamingAggregationState,
    state_after: StreamingAggregationState,
    fingerprint: CognitiveFingerprint,
    event_index: u64,
    salience: CognitiveSalience,
    evicted: Option<CognitiveFingerprint>,
    aggregate: Option<StreamingAggregate>,
    status: StreamingAggregationStatus,
}

impl StreamingAggregationResult {
    pub fn state_before(&self) -> &StreamingAggregationState {
        &self.state_before
    }

    pub fn state_after(&self) -> &StreamingAggregationState {
        &self.state_after
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn event_index(&self) -> u64 {
        self.event_index
    }

    pub fn salience(&self) -> CognitiveSalience {
        self.salience
    }

    pub fn evicted(&self) -> Option<CognitiveFingerprint> {
        self.evicted
    }

    pub fn aggregate(&self) -> Option<&StreamingAggregate> {
        self.aggregate.as_ref()
    }

    pub fn status(&self) -> StreamingAggregationStatus {
        self.status
    }

    pub fn accepted(&self) -> bool {
        self.status != StreamingAggregationStatus::RejectedOutOfOrder
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StreamingAggregator;

impl StreamingAggregator {
    pub fn observe(
        state: StreamingAggregationState,
        event_index: u64,
        fingerprint: CognitiveFingerprint,
        salience: CognitiveSalience,
    ) -> StreamingAggregationResult {
        let state_before = state.clone();

        if let Some(previous_index) = state.last_event_index() {
            if event_index <= previous_index {
                return StreamingAggregationResult {
                    state_before,
                    state_after: state,
                    fingerprint,
                    event_index,
                    salience,
                    evicted: None,
                    aggregate: None,
                    status: StreamingAggregationStatus::RejectedOutOfOrder,
                };
            }
        }

        let mut state_after = state;

        let mut evicted = None;

        let status = if let Some(aggregate) = state_after.aggregates.get_mut(&fingerprint) {
            aggregate.observe(event_index, salience);

            StreamingAggregationStatus::Updated
        } else {
            if state_after.is_full() {
                let victim = state_after
                    .aggregates
                    .iter()
                    .min_by_key(|(victim_fingerprint, aggregate)| {
                        (aggregate.last_seen(), **victim_fingerprint)
                    })
                    .map(|(victim_fingerprint, _)| *victim_fingerprint);

                if let Some(victim) = victim {
                    state_after.aggregates.remove(&victim);

                    evicted = Some(victim);
                }
            }

            state_after.aggregates.insert(
                fingerprint,
                StreamingAggregate::new(fingerprint, event_index, salience),
            );

            StreamingAggregationStatus::Inserted
        };

        state_after.last_event_index = Some(event_index);

        let aggregate = state_after.aggregates.get(&fingerprint).cloned();

        StreamingAggregationResult {
            state_before,
            state_after,
            fingerprint,
            event_index,
            salience,
            evicted,
            aggregate,
            status,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MindstoneStreamingAggregationResult {
    structure: CognitiveStructure,
    fingerprint: CognitiveFingerprint,
    profile: MindstoneSignalProfile,
    aggregation: StreamingAggregationResult,
}

impl MindstoneStreamingAggregationResult {
    pub fn structure(&self) -> &CognitiveStructure {
        &self.structure
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn profile(&self) -> MindstoneSignalProfile {
        self.profile
    }

    pub fn aggregation(&self) -> &StreamingAggregationResult {
        &self.aggregation
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneStreamingAggregator;

impl MindstoneStreamingAggregator {
    pub fn observe(
        state: StreamingAggregationState,
        event_index: u64,
        structure: CognitiveStructure,
        profile: MindstoneSignalProfile,
    ) -> MindstoneStreamingAggregationResult {
        let fingerprint = StructuralHasher::fingerprint(&structure);

        let aggregation =
            StreamingAggregator::observe(state, event_index, fingerprint, profile.salience());

        MindstoneStreamingAggregationResult {
            structure,
            fingerprint,
            profile,
            aggregation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CognitiveCandidate {
    fingerprint: CognitiveFingerprint,
    salience: CognitiveSalience,
    support: u64,
    estimated_cost: u32,
}

impl CognitiveCandidate {
    pub fn new(
        fingerprint: CognitiveFingerprint,
        salience: CognitiveSalience,
        support: u64,
        estimated_cost: u32,
    ) -> Option<Self> {
        if support == 0 {
            return None;
        }

        if estimated_cost == 0 {
            return None;
        }

        Some(Self {
            fingerprint,
            salience,
            support,
            estimated_cost,
        })
    }

    pub fn from_streaming_aggregate(
        aggregate: &StreamingAggregate,
        estimated_cost: u32,
    ) -> Option<Self> {
        Self::new(
            aggregate.fingerprint(),
            aggregate.peak_salience(),
            aggregate.observation_count(),
            estimated_cost,
        )
    }

    pub fn fingerprint(self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn salience(self) -> CognitiveSalience {
        self.salience
    }

    pub fn support(self) -> u64 {
        self.support
    }

    pub fn estimated_cost(self) -> u32 {
        self.estimated_cost
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundedCandidateSearchPolicy {
    max_candidates: usize,
}

impl BoundedCandidateSearchPolicy {
    pub fn new(max_candidates: usize) -> Option<Self> {
        if max_candidates == 0 {
            return None;
        }

        Some(Self { max_candidates })
    }

    pub fn max_candidates(self) -> usize {
        self.max_candidates
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedCandidateSearchResult {
    input_candidate_count: usize,
    unique_candidate_count: usize,
    selected: Vec<CognitiveCandidate>,
    total_selected_cost: u32,
    truncated_by_candidate_limit: bool,
    truncated_by_compute_budget: bool,
}

impl BoundedCandidateSearchResult {
    pub fn input_candidate_count(&self) -> usize {
        self.input_candidate_count
    }

    pub fn unique_candidate_count(&self) -> usize {
        self.unique_candidate_count
    }

    pub fn selected(&self) -> &[CognitiveCandidate] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn total_selected_cost(&self) -> u32 {
        self.total_selected_cost
    }

    pub fn truncated_by_candidate_limit(&self) -> bool {
        self.truncated_by_candidate_limit
    }

    pub fn truncated_by_compute_budget(&self) -> bool {
        self.truncated_by_compute_budget
    }

    pub fn was_truncated(&self) -> bool {
        self.truncated_by_candidate_limit || self.truncated_by_compute_budget
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundedCandidateSearch;

impl BoundedCandidateSearch {
    fn ranking(left: &CognitiveCandidate, right: &CognitiveCandidate) -> std::cmp::Ordering {
        right
            .salience()
            .cmp(&left.salience())
            .then_with(|| right.support().cmp(&left.support()))
            .then_with(|| left.estimated_cost().cmp(&right.estimated_cost()))
            .then_with(|| left.fingerprint().cmp(&right.fingerprint()))
    }

    fn canonicalize(candidates: Vec<CognitiveCandidate>) -> Vec<CognitiveCandidate> {
        let mut unique =
            std::collections::BTreeMap::<CognitiveFingerprint, CognitiveCandidate>::new();

        for candidate in candidates {
            match unique.get(&candidate.fingerprint()).copied() {
                None => {
                    unique.insert(candidate.fingerprint(), candidate);
                }

                Some(existing) => {
                    if Self::ranking(&candidate, &existing) == std::cmp::Ordering::Less {
                        unique.insert(candidate.fingerprint(), candidate);
                    }
                }
            }
        }

        let mut ranked = unique.into_values().collect::<Vec<_>>();

        ranked.sort_by(Self::ranking);

        ranked
    }

    pub fn select(
        candidates: Vec<CognitiveCandidate>,
        policy: BoundedCandidateSearchPolicy,
        budget: CognitiveBudget,
    ) -> BoundedCandidateSearchResult {
        let input_candidate_count = candidates.len();

        let ranked = Self::canonicalize(candidates);

        let unique_candidate_count = ranked.len();

        let mut selected = Vec::with_capacity(policy.max_candidates().min(unique_candidate_count));

        let mut total_selected_cost = 0_u32;

        let mut truncated_by_candidate_limit = false;

        let mut truncated_by_compute_budget = false;

        for (index, candidate) in ranked.into_iter().enumerate() {
            if selected.len() >= policy.max_candidates() {
                truncated_by_candidate_limit = index < unique_candidate_count;

                break;
            }

            let Some(next_total) = total_selected_cost.checked_add(candidate.estimated_cost())
            else {
                truncated_by_compute_budget = true;

                break;
            };

            if next_total > budget.units() {
                truncated_by_compute_budget = true;

                break;
            }

            total_selected_cost = next_total;

            selected.push(candidate);
        }

        BoundedCandidateSearchResult {
            input_candidate_count,
            unique_candidate_count,
            selected,
            total_selected_cost,
            truncated_by_candidate_limit,
            truncated_by_compute_budget,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneBoundedCandidateSearch;

impl MindstoneBoundedCandidateSearch {
    pub fn evaluate(
        candidates: Vec<CognitiveCandidate>,
        policy: BoundedCandidateSearchPolicy,
        budget: CognitiveBudget,
    ) -> BoundedCandidateSearchResult {
        BoundedCandidateSearch::select(candidates, policy, budget)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CognitiveMemoryTier {
    Cold,
    Consolidated,
    Active,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HierarchicalMemoryAdmissionClass {
    Discard,
    Cold,
    Consolidated,
    Active,
}

impl HierarchicalMemoryAdmissionClass {
    pub fn tier(self) -> Option<CognitiveMemoryTier> {
        match self {
            Self::Discard => None,
            Self::Cold => Some(CognitiveMemoryTier::Cold),
            Self::Consolidated => Some(CognitiveMemoryTier::Consolidated),
            Self::Active => Some(CognitiveMemoryTier::Active),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HierarchicalMemoryPolicy {
    consolidated_salience_threshold: CognitiveSignal,
    active_salience_threshold: CognitiveSignal,
    cold_support_threshold: u64,
    consolidated_support_threshold: u64,
    active_capacity: usize,
    consolidated_capacity: usize,
    cold_capacity: usize,
}

impl HierarchicalMemoryPolicy {
    pub fn new(
        consolidated_salience_threshold: CognitiveSignal,
        active_salience_threshold: CognitiveSignal,
        cold_support_threshold: u64,
        consolidated_support_threshold: u64,
        active_capacity: usize,
        consolidated_capacity: usize,
        cold_capacity: usize,
    ) -> Option<Self> {
        if consolidated_salience_threshold >= active_salience_threshold {
            return None;
        }

        if cold_support_threshold == 0 || consolidated_support_threshold == 0 {
            return None;
        }

        if cold_support_threshold > consolidated_support_threshold {
            return None;
        }

        if active_capacity == 0 || consolidated_capacity == 0 || cold_capacity == 0 {
            return None;
        }

        Some(Self {
            consolidated_salience_threshold,
            active_salience_threshold,
            cold_support_threshold,
            consolidated_support_threshold,
            active_capacity,
            consolidated_capacity,
            cold_capacity,
        })
    }

    pub fn consolidated_salience_threshold(self) -> CognitiveSignal {
        self.consolidated_salience_threshold
    }

    pub fn active_salience_threshold(self) -> CognitiveSignal {
        self.active_salience_threshold
    }

    pub fn cold_support_threshold(self) -> u64 {
        self.cold_support_threshold
    }

    pub fn consolidated_support_threshold(self) -> u64 {
        self.consolidated_support_threshold
    }

    pub fn active_capacity(self) -> usize {
        self.active_capacity
    }

    pub fn consolidated_capacity(self) -> usize {
        self.consolidated_capacity
    }

    pub fn cold_capacity(self) -> usize {
        self.cold_capacity
    }

    pub fn total_capacity(self) -> usize {
        self.active_capacity
            .saturating_add(self.consolidated_capacity)
            .saturating_add(self.cold_capacity)
    }

    pub fn classify(self, candidate: CognitiveCandidate) -> HierarchicalMemoryAdmissionClass {
        if candidate.salience().value() >= self.active_salience_threshold.value() {
            HierarchicalMemoryAdmissionClass::Active
        } else if candidate.salience().value() >= self.consolidated_salience_threshold.value()
            && candidate.support() >= self.consolidated_support_threshold
        {
            HierarchicalMemoryAdmissionClass::Consolidated
        } else if candidate.support() >= self.cold_support_threshold {
            HierarchicalMemoryAdmissionClass::Cold
        } else {
            HierarchicalMemoryAdmissionClass::Discard
        }
    }

    fn capacity_for(self, tier: CognitiveMemoryTier) -> usize {
        match tier {
            CognitiveMemoryTier::Active => self.active_capacity,
            CognitiveMemoryTier::Consolidated => self.consolidated_capacity,
            CognitiveMemoryTier::Cold => self.cold_capacity,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CognitiveMemoryRecord {
    candidate: CognitiveCandidate,
    tier: CognitiveMemoryTier,
    first_admitted_at: u64,
    last_admitted_at: u64,
    admission_count: u64,
}

impl CognitiveMemoryRecord {
    fn new(candidate: CognitiveCandidate, tier: CognitiveMemoryTier, event_index: u64) -> Self {
        Self {
            candidate,
            tier,
            first_admitted_at: event_index,
            last_admitted_at: event_index,
            admission_count: 1,
        }
    }

    fn updated(
        previous: CognitiveMemoryRecord,
        candidate: CognitiveCandidate,
        tier: CognitiveMemoryTier,
        event_index: u64,
    ) -> Self {
        Self {
            candidate,
            tier,
            first_admitted_at: previous.first_admitted_at,
            last_admitted_at: event_index,
            admission_count: previous.admission_count.saturating_add(1),
        }
    }

    pub fn candidate(&self) -> CognitiveCandidate {
        self.candidate
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.candidate.fingerprint()
    }

    pub fn tier(&self) -> CognitiveMemoryTier {
        self.tier
    }

    pub fn first_admitted_at(&self) -> u64 {
        self.first_admitted_at
    }

    pub fn last_admitted_at(&self) -> u64 {
        self.last_admitted_at
    }

    pub fn admission_count(&self) -> u64 {
        self.admission_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchicalMemoryState {
    last_event_index: Option<u64>,
    active: std::collections::BTreeMap<CognitiveFingerprint, CognitiveMemoryRecord>,
    consolidated: std::collections::BTreeMap<CognitiveFingerprint, CognitiveMemoryRecord>,
    cold: std::collections::BTreeMap<CognitiveFingerprint, CognitiveMemoryRecord>,
}

impl HierarchicalMemoryState {
    pub fn empty() -> Self {
        Self {
            last_event_index: None,
            active: std::collections::BTreeMap::new(),
            consolidated: std::collections::BTreeMap::new(),
            cold: std::collections::BTreeMap::new(),
        }
    }

    pub fn last_event_index(&self) -> Option<u64> {
        self.last_event_index
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    pub fn consolidated_len(&self) -> usize {
        self.consolidated.len()
    }

    pub fn cold_len(&self) -> usize {
        self.cold.len()
    }

    pub fn total_len(&self) -> usize {
        self.active_len()
            .saturating_add(self.consolidated_len())
            .saturating_add(self.cold_len())
    }

    pub fn contains(&self, fingerprint: CognitiveFingerprint) -> bool {
        self.active.contains_key(&fingerprint)
            || self.consolidated.contains_key(&fingerprint)
            || self.cold.contains_key(&fingerprint)
    }

    pub fn tier_of(&self, fingerprint: CognitiveFingerprint) -> Option<CognitiveMemoryTier> {
        if self.active.contains_key(&fingerprint) {
            Some(CognitiveMemoryTier::Active)
        } else if self.consolidated.contains_key(&fingerprint) {
            Some(CognitiveMemoryTier::Consolidated)
        } else if self.cold.contains_key(&fingerprint) {
            Some(CognitiveMemoryTier::Cold)
        } else {
            None
        }
    }

    pub fn record(&self, fingerprint: CognitiveFingerprint) -> Option<&CognitiveMemoryRecord> {
        self.active
            .get(&fingerprint)
            .or_else(|| self.consolidated.get(&fingerprint))
            .or_else(|| self.cold.get(&fingerprint))
    }

    fn map(
        &self,
        tier: CognitiveMemoryTier,
    ) -> &std::collections::BTreeMap<CognitiveFingerprint, CognitiveMemoryRecord> {
        match tier {
            CognitiveMemoryTier::Active => &self.active,
            CognitiveMemoryTier::Consolidated => &self.consolidated,
            CognitiveMemoryTier::Cold => &self.cold,
        }
    }

    fn map_mut(
        &mut self,
        tier: CognitiveMemoryTier,
    ) -> &mut std::collections::BTreeMap<CognitiveFingerprint, CognitiveMemoryRecord> {
        match tier {
            CognitiveMemoryTier::Active => &mut self.active,
            CognitiveMemoryTier::Consolidated => &mut self.consolidated,
            CognitiveMemoryTier::Cold => &mut self.cold,
        }
    }

    fn remove(
        &mut self,
        fingerprint: CognitiveFingerprint,
    ) -> Option<(CognitiveMemoryTier, CognitiveMemoryRecord)> {
        if let Some(record) = self.active.remove(&fingerprint) {
            return Some((CognitiveMemoryTier::Active, record));
        }

        if let Some(record) = self.consolidated.remove(&fingerprint) {
            return Some((CognitiveMemoryTier::Consolidated, record));
        }

        self.cold
            .remove(&fingerprint)
            .map(|record| (CognitiveMemoryTier::Cold, record))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveMemoryEviction {
    fingerprint: CognitiveFingerprint,
    tier: CognitiveMemoryTier,
}

impl CognitiveMemoryEviction {
    fn new(fingerprint: CognitiveFingerprint, tier: CognitiveMemoryTier) -> Self {
        Self { fingerprint, tier }
    }

    pub fn fingerprint(self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn tier(self) -> CognitiveMemoryTier {
        self.tier
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HierarchicalMemoryAdmissionStatus {
    RejectedOutOfOrder,
    Discarded,
    Cold,
    Consolidated,
    Active,
}

impl HierarchicalMemoryAdmissionStatus {
    pub fn tier(self) -> Option<CognitiveMemoryTier> {
        match self {
            Self::RejectedOutOfOrder | Self::Discarded => None,
            Self::Cold => Some(CognitiveMemoryTier::Cold),
            Self::Consolidated => Some(CognitiveMemoryTier::Consolidated),
            Self::Active => Some(CognitiveMemoryTier::Active),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchicalMemoryAdmissionResult {
    state_before: HierarchicalMemoryState,
    state_after: HierarchicalMemoryState,
    event_index: u64,
    candidate: CognitiveCandidate,
    class: HierarchicalMemoryAdmissionClass,
    previous_tier: Option<CognitiveMemoryTier>,
    eviction: Option<CognitiveMemoryEviction>,
    record: Option<CognitiveMemoryRecord>,
    status: HierarchicalMemoryAdmissionStatus,
}

impl HierarchicalMemoryAdmissionResult {
    pub fn state_before(&self) -> &HierarchicalMemoryState {
        &self.state_before
    }

    pub fn state_after(&self) -> &HierarchicalMemoryState {
        &self.state_after
    }

    pub fn event_index(&self) -> u64 {
        self.event_index
    }

    pub fn candidate(&self) -> CognitiveCandidate {
        self.candidate
    }

    pub fn class(&self) -> HierarchicalMemoryAdmissionClass {
        self.class
    }

    pub fn previous_tier(&self) -> Option<CognitiveMemoryTier> {
        self.previous_tier
    }

    pub fn eviction(&self) -> Option<CognitiveMemoryEviction> {
        self.eviction
    }

    pub fn record(&self) -> Option<&CognitiveMemoryRecord> {
        self.record.as_ref()
    }

    pub fn status(&self) -> HierarchicalMemoryAdmissionStatus {
        self.status
    }

    pub fn accepted(&self) -> bool {
        self.status != HierarchicalMemoryAdmissionStatus::RejectedOutOfOrder
    }

    pub fn retained(&self) -> bool {
        self.status.tier().is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HierarchicalMemoryAdmission;

impl HierarchicalMemoryAdmission {
    fn status_for(class: HierarchicalMemoryAdmissionClass) -> HierarchicalMemoryAdmissionStatus {
        match class {
            HierarchicalMemoryAdmissionClass::Discard => {
                HierarchicalMemoryAdmissionStatus::Discarded
            }
            HierarchicalMemoryAdmissionClass::Cold => HierarchicalMemoryAdmissionStatus::Cold,
            HierarchicalMemoryAdmissionClass::Consolidated => {
                HierarchicalMemoryAdmissionStatus::Consolidated
            }
            HierarchicalMemoryAdmissionClass::Active => HierarchicalMemoryAdmissionStatus::Active,
        }
    }

    fn evict_oldest(
        state: &mut HierarchicalMemoryState,
        tier: CognitiveMemoryTier,
    ) -> Option<CognitiveMemoryEviction> {
        let victim = state
            .map(tier)
            .iter()
            .min_by_key(|(fingerprint, record)| (record.last_admitted_at(), **fingerprint))
            .map(|(fingerprint, _)| *fingerprint);

        victim.map(|fingerprint| {
            state.map_mut(tier).remove(&fingerprint);

            CognitiveMemoryEviction::new(fingerprint, tier)
        })
    }

    pub fn admit(
        state: HierarchicalMemoryState,
        event_index: u64,
        candidate: CognitiveCandidate,
        policy: HierarchicalMemoryPolicy,
    ) -> HierarchicalMemoryAdmissionResult {
        let state_before = state.clone();

        let class = policy.classify(candidate);

        if let Some(previous_index) = state.last_event_index() {
            if event_index <= previous_index {
                return HierarchicalMemoryAdmissionResult {
                    state_before,
                    state_after: state,
                    event_index,
                    candidate,
                    class,
                    previous_tier: None,
                    eviction: None,
                    record: None,
                    status: HierarchicalMemoryAdmissionStatus::RejectedOutOfOrder,
                };
            }
        }

        let mut state_after = state;

        let previous = state_after.remove(candidate.fingerprint());

        let previous_tier = previous.as_ref().map(|(tier, _)| *tier);

        state_after.last_event_index = Some(event_index);

        let Some(target_tier) = class.tier() else {
            return HierarchicalMemoryAdmissionResult {
                state_before,
                state_after,
                event_index,
                candidate,
                class,
                previous_tier,
                eviction: None,
                record: None,
                status: HierarchicalMemoryAdmissionStatus::Discarded,
            };
        };

        let eviction = if state_after.map(target_tier).len() >= policy.capacity_for(target_tier) {
            Self::evict_oldest(&mut state_after, target_tier)
        } else {
            None
        };

        let record = match previous {
            Some((_, previous_record)) => {
                CognitiveMemoryRecord::updated(previous_record, candidate, target_tier, event_index)
            }

            None => CognitiveMemoryRecord::new(candidate, target_tier, event_index),
        };

        state_after
            .map_mut(target_tier)
            .insert(candidate.fingerprint(), record.clone());

        HierarchicalMemoryAdmissionResult {
            state_before,
            state_after,
            event_index,
            candidate,
            class,
            previous_tier,
            eviction,
            record: Some(record),
            status: Self::status_for(class),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneHierarchicalMemoryAdmission;

impl MindstoneHierarchicalMemoryAdmission {
    pub fn evaluate(
        state: HierarchicalMemoryState,
        event_index: u64,
        candidate: CognitiveCandidate,
        policy: HierarchicalMemoryPolicy,
    ) -> HierarchicalMemoryAdmissionResult {
        HierarchicalMemoryAdmission::admit(state, event_index, candidate, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveForgettingPolicy {
    active_cool_after: u64,
    consolidated_cool_after: u64,
    cold_forget_after: u64,
    protected_admission_count: u64,
    protected_salience_threshold: CognitiveSignal,
}

impl CognitiveForgettingPolicy {
    pub fn new(
        active_cool_after: u64,
        consolidated_cool_after: u64,
        cold_forget_after: u64,
        protected_admission_count: u64,
        protected_salience_threshold: CognitiveSignal,
    ) -> Option<Self> {
        if active_cool_after == 0
            || consolidated_cool_after == 0
            || cold_forget_after == 0
            || protected_admission_count == 0
        {
            return None;
        }

        if active_cool_after >= consolidated_cool_after
            || consolidated_cool_after >= cold_forget_after
        {
            return None;
        }

        Some(Self {
            active_cool_after,
            consolidated_cool_after,
            cold_forget_after,
            protected_admission_count,
            protected_salience_threshold,
        })
    }

    pub fn active_cool_after(self) -> u64 {
        self.active_cool_after
    }

    pub fn consolidated_cool_after(self) -> u64 {
        self.consolidated_cool_after
    }

    pub fn cold_forget_after(self) -> u64 {
        self.cold_forget_after
    }

    pub fn protected_admission_count(self) -> u64 {
        self.protected_admission_count
    }

    pub fn protected_salience_threshold(self) -> CognitiveSignal {
        self.protected_salience_threshold
    }

    pub fn protects(self, record: &CognitiveMemoryRecord) -> bool {
        record.admission_count() >= self.protected_admission_count
            || record.candidate().salience().value() >= self.protected_salience_threshold.value()
    }
}

impl CognitiveMemoryRecord {
    fn with_tier(mut self, tier: CognitiveMemoryTier) -> Self {
        self.tier = tier;

        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CognitiveMemoryMaintenanceAction {
    Cooled {
        fingerprint: CognitiveFingerprint,
        from: CognitiveMemoryTier,
        to: CognitiveMemoryTier,
    },
    Forgotten {
        fingerprint: CognitiveFingerprint,
    },
    CapacityEvicted {
        fingerprint: CognitiveFingerprint,
        tier: CognitiveMemoryTier,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CognitiveMemoryMaintenanceStatus {
    RejectedOutOfOrder,
    Maintained,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CognitiveMemoryMaintenanceResult {
    state_before: HierarchicalMemoryState,
    state_after: HierarchicalMemoryState,
    current_index: u64,
    actions: Vec<CognitiveMemoryMaintenanceAction>,
    status: CognitiveMemoryMaintenanceStatus,
}

impl CognitiveMemoryMaintenanceResult {
    pub fn state_before(&self) -> &HierarchicalMemoryState {
        &self.state_before
    }

    pub fn state_after(&self) -> &HierarchicalMemoryState {
        &self.state_after
    }

    pub fn current_index(&self) -> u64 {
        self.current_index
    }

    pub fn actions(&self) -> &[CognitiveMemoryMaintenanceAction] {
        &self.actions
    }

    pub fn status(&self) -> CognitiveMemoryMaintenanceStatus {
        self.status
    }

    pub fn accepted(&self) -> bool {
        self.status == CognitiveMemoryMaintenanceStatus::Maintained
    }

    pub fn changed(&self) -> bool {
        !self.actions.is_empty()
    }

    pub fn cooled_count(&self) -> usize {
        self.actions
            .iter()
            .filter(|action| matches!(action, CognitiveMemoryMaintenanceAction::Cooled { .. }))
            .count()
    }

    pub fn forgotten_count(&self) -> usize {
        self.actions
            .iter()
            .filter(|action| matches!(action, CognitiveMemoryMaintenanceAction::Forgotten { .. }))
            .count()
    }

    pub fn capacity_eviction_count(&self) -> usize {
        self.actions
            .iter()
            .filter(|action| {
                matches!(
                    action,
                    CognitiveMemoryMaintenanceAction::CapacityEvicted { .. }
                )
            })
            .count()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveMemoryMaintenance;

impl CognitiveMemoryMaintenance {
    fn age(current_index: u64, record: &CognitiveMemoryRecord) -> u64 {
        current_index.saturating_sub(record.last_admitted_at())
    }

    fn cool_record(
        state: &mut HierarchicalMemoryState,
        fingerprint: CognitiveFingerprint,
        from: CognitiveMemoryTier,
        to: CognitiveMemoryTier,
        admission_policy: HierarchicalMemoryPolicy,
        actions: &mut Vec<CognitiveMemoryMaintenanceAction>,
    ) {
        let Some(record) = state.map_mut(from).remove(&fingerprint) else {
            return;
        };

        if state.map(to).len() >= admission_policy.capacity_for(to) {
            if let Some(eviction) = HierarchicalMemoryAdmission::evict_oldest(state, to) {
                actions.push(CognitiveMemoryMaintenanceAction::CapacityEvicted {
                    fingerprint: eviction.fingerprint(),
                    tier: eviction.tier(),
                });
            }
        }

        state.map_mut(to).insert(fingerprint, record.with_tier(to));

        actions.push(CognitiveMemoryMaintenanceAction::Cooled {
            fingerprint,
            from,
            to,
        });
    }

    pub fn maintain(
        state: HierarchicalMemoryState,
        current_index: u64,
        admission_policy: HierarchicalMemoryPolicy,
        forgetting_policy: CognitiveForgettingPolicy,
    ) -> CognitiveMemoryMaintenanceResult {
        let state_before = state.clone();

        if let Some(previous_index) = state.last_event_index() {
            if current_index <= previous_index {
                return CognitiveMemoryMaintenanceResult {
                    state_before,
                    state_after: state,
                    current_index,
                    actions: Vec::new(),
                    status: CognitiveMemoryMaintenanceStatus::RejectedOutOfOrder,
                };
            }
        }

        let active_snapshot = state.active.values().cloned().collect::<Vec<_>>();

        let consolidated_snapshot = state.consolidated.values().cloned().collect::<Vec<_>>();

        let cold_snapshot = state.cold.values().cloned().collect::<Vec<_>>();

        let mut state_after = state;

        state_after.last_event_index = Some(current_index);

        let mut actions = Vec::new();

        for record in active_snapshot {
            if forgetting_policy.protects(&record) {
                continue;
            }

            if Self::age(current_index, &record) >= forgetting_policy.active_cool_after() {
                Self::cool_record(
                    &mut state_after,
                    record.fingerprint(),
                    CognitiveMemoryTier::Active,
                    CognitiveMemoryTier::Consolidated,
                    admission_policy,
                    &mut actions,
                );
            }
        }

        for record in consolidated_snapshot {
            if forgetting_policy.protects(&record) {
                continue;
            }

            if Self::age(current_index, &record) >= forgetting_policy.consolidated_cool_after() {
                Self::cool_record(
                    &mut state_after,
                    record.fingerprint(),
                    CognitiveMemoryTier::Consolidated,
                    CognitiveMemoryTier::Cold,
                    admission_policy,
                    &mut actions,
                );
            }
        }

        for record in cold_snapshot {
            if forgetting_policy.protects(&record) {
                continue;
            }

            if Self::age(current_index, &record) >= forgetting_policy.cold_forget_after()
                && state_after.cold.remove(&record.fingerprint()).is_some()
            {
                actions.push(CognitiveMemoryMaintenanceAction::Forgotten {
                    fingerprint: record.fingerprint(),
                });
            }
        }

        CognitiveMemoryMaintenanceResult {
            state_before,
            state_after,
            current_index,
            actions,
            status: CognitiveMemoryMaintenanceStatus::Maintained,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneForgettingColdStorage;

impl MindstoneForgettingColdStorage {
    pub fn evaluate(
        state: HierarchicalMemoryState,
        current_index: u64,
        admission_policy: HierarchicalMemoryPolicy,
        forgetting_policy: CognitiveForgettingPolicy,
    ) -> CognitiveMemoryMaintenanceResult {
        CognitiveMemoryMaintenance::maintain(
            state,
            current_index,
            admission_policy,
            forgetting_policy,
        )
    }
}

impl CognitiveSignal {
    pub fn compression_gain(original_units: u64, compressed_units: u64) -> Option<Self> {
        if original_units == 0 {
            return None;
        }

        if compressed_units >= original_units {
            return Some(Self::zero());
        }

        let saved = original_units.saturating_sub(compressed_units);

        let scaled = u128::from(saved).saturating_mul(u128::from(COGNITIVE_SIGNAL_SCALE))
            / u128::from(original_units);

        Some(Self(scaled as u16))
    }

    pub fn controllability(
        successful_interventions: u64,
        intervention_attempts: u64,
    ) -> Option<Self> {
        if intervention_attempts == 0 || successful_interventions > intervention_attempts {
            return None;
        }

        let scaled = u128::from(successful_interventions)
            .saturating_mul(u128::from(COGNITIVE_SIGNAL_SCALE))
            / u128::from(intervention_attempts);

        Some(Self(scaled as u16))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneExtendedSignalProfile {
    base: MindstoneSignalProfile,
    compression_gain: CognitiveSignal,
    controllability: CognitiveSignal,
}

impl MindstoneExtendedSignalProfile {
    pub fn new(
        base: MindstoneSignalProfile,
        compression_gain: CognitiveSignal,
        controllability: CognitiveSignal,
    ) -> Self {
        Self {
            base,
            compression_gain,
            controllability,
        }
    }

    pub fn base(self) -> MindstoneSignalProfile {
        self.base
    }

    pub fn compression_gain(self) -> CognitiveSignal {
        self.compression_gain
    }

    pub fn controllability(self) -> CognitiveSignal {
        self.controllability
    }

    pub fn meta_salience(self) -> CognitiveSalience {
        let compression = u32::from(self.compression_gain.value());

        let controllability = u32::from(self.controllability.value());

        let peak = compression.max(controllability);

        let total = compression.saturating_add(controllability);

        CognitiveSalience((peak.saturating_mul(2).saturating_add(total) / 4) as u16)
    }

    pub fn salience(self) -> CognitiveSalience {
        let base = u32::from(self.base.salience().value());

        let meta = u32::from(self.meta_salience().value());

        let remaining = u32::from(COGNITIVE_SIGNAL_SCALE).saturating_sub(base);

        let augmentation = remaining.saturating_mul(meta) / u32::from(COGNITIVE_SIGNAL_SCALE);

        CognitiveSalience(base.saturating_add(augmentation) as u16)
    }
}

impl SparseCognitionPolicy {
    pub fn classify_extended(
        self,
        profile: MindstoneExtendedSignalProfile,
    ) -> CognitiveAdmissionClass {
        let salience = profile.salience().value();

        if salience < self.cheap_threshold().value() {
            CognitiveAdmissionClass::Ignore
        } else if salience < self.deliberate_threshold().value() {
            CognitiveAdmissionClass::CheapUpdate
        } else {
            CognitiveAdmissionClass::Deliberate
        }
    }

    pub fn admit_extended(
        self,
        profile: MindstoneExtendedSignalProfile,
        budget: CognitiveBudget,
    ) -> CognitiveAdmissionDecision {
        let salience = profile.salience();

        let class = self.classify_extended(profile);

        let requested_units = match class {
            CognitiveAdmissionClass::Ignore => 0,
            CognitiveAdmissionClass::CheapUpdate => self.cheap_compute_units(),
            CognitiveAdmissionClass::Deliberate => self.deliberate_compute_units(),
        };

        let granted_units = requested_units.min(budget.units());

        CognitiveAdmissionDecision {
            class,
            salience,
            requested_units,
            granted_units,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MindstoneCompressionControllabilityResult {
    base_profile: MindstoneSignalProfile,
    compression_gain: CognitiveSignal,
    controllability: CognitiveSignal,
    extended_profile: MindstoneExtendedSignalProfile,
    decision: CognitiveAdmissionDecision,
}

impl MindstoneCompressionControllabilityResult {
    pub fn base_profile(&self) -> MindstoneSignalProfile {
        self.base_profile
    }

    pub fn compression_gain(&self) -> CognitiveSignal {
        self.compression_gain
    }

    pub fn controllability(&self) -> CognitiveSignal {
        self.controllability
    }

    pub fn extended_profile(&self) -> MindstoneExtendedSignalProfile {
        self.extended_profile
    }

    pub fn decision(&self) -> CognitiveAdmissionDecision {
        self.decision
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneCompressionControllability;

impl MindstoneCompressionControllability {
    pub fn evaluate(
        base_profile: MindstoneSignalProfile,
        original_units: u64,
        compressed_units: u64,
        successful_interventions: u64,
        intervention_attempts: u64,
        policy: SparseCognitionPolicy,
        budget: CognitiveBudget,
    ) -> Option<MindstoneCompressionControllabilityResult> {
        let compression_gain = CognitiveSignal::compression_gain(original_units, compressed_units)?;

        let controllability =
            CognitiveSignal::controllability(successful_interventions, intervention_attempts)?;

        let extended_profile =
            MindstoneExtendedSignalProfile::new(base_profile, compression_gain, controllability);

        let decision = policy.admit_extended(extended_profile, budget);

        Some(MindstoneCompressionControllabilityResult {
            base_profile,
            compression_gain,
            controllability,
            extended_profile,
            decision,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EpistemicSelfClass {
    Uncertain,
    Learning,
    Stable,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EpistemicSelfPolicy {
    stable_uncertainty_max: CognitiveSignal,
    learning_progress_min: CognitiveSignal,
    compression_gain_min: CognitiveSignal,
    controllability_min: CognitiveSignal,
    minimum_observations: u64,
}

impl EpistemicSelfPolicy {
    pub fn new(
        stable_uncertainty_max: CognitiveSignal,
        learning_progress_min: CognitiveSignal,
        compression_gain_min: CognitiveSignal,
        controllability_min: CognitiveSignal,
        minimum_observations: u64,
    ) -> Option<Self> {
        if learning_progress_min == CognitiveSignal::zero()
            || compression_gain_min == CognitiveSignal::zero()
            || controllability_min == CognitiveSignal::zero()
            || minimum_observations == 0
        {
            return None;
        }

        Some(Self {
            stable_uncertainty_max,
            learning_progress_min,
            compression_gain_min,
            controllability_min,
            minimum_observations,
        })
    }

    pub fn stable_uncertainty_max(self) -> CognitiveSignal {
        self.stable_uncertainty_max
    }

    pub fn learning_progress_min(self) -> CognitiveSignal {
        self.learning_progress_min
    }

    pub fn compression_gain_min(self) -> CognitiveSignal {
        self.compression_gain_min
    }

    pub fn controllability_min(self) -> CognitiveSignal {
        self.controllability_min
    }

    pub fn minimum_observations(self) -> u64 {
        self.minimum_observations
    }

    pub fn assess(self, record: &EpistemicSelfRecord) -> EpistemicSelfAssessment {
        let sufficiently_observed = record.observation_count() >= self.minimum_observations;

        let class = if sufficiently_observed && record.uncertainty() <= self.stable_uncertainty_max
        {
            EpistemicSelfClass::Stable
        } else if record.learning_progress() >= self.learning_progress_min {
            EpistemicSelfClass::Learning
        } else {
            EpistemicSelfClass::Uncertain
        };

        let compressible =
            sufficiently_observed && record.compression_gain() >= self.compression_gain_min;

        let controllable =
            sufficiently_observed && record.controllability() >= self.controllability_min;

        EpistemicSelfAssessment {
            class,
            compressible,
            controllable,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EpistemicSelfAssessment {
    class: EpistemicSelfClass,
    compressible: bool,
    controllable: bool,
}

impl EpistemicSelfAssessment {
    pub fn class(self) -> EpistemicSelfClass {
        self.class
    }

    pub fn is_uncertain(self) -> bool {
        self.class == EpistemicSelfClass::Uncertain
    }

    pub fn is_learning(self) -> bool {
        self.class == EpistemicSelfClass::Learning
    }

    pub fn is_stable(self) -> bool {
        self.class == EpistemicSelfClass::Stable
    }

    pub fn is_compressible(self) -> bool {
        self.compressible
    }

    pub fn is_controllable(self) -> bool {
        self.controllable
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicSelfRecord {
    fingerprint: CognitiveFingerprint,
    observation_count: u64,
    first_updated_at: u64,
    last_updated_at: u64,
    uncertainty: CognitiveSignal,
    learning_progress: CognitiveSignal,
    compression_gain: CognitiveSignal,
    controllability: CognitiveSignal,
}

impl EpistemicSelfRecord {
    fn new(
        fingerprint: CognitiveFingerprint,
        update_index: u64,
        profile: MindstoneExtendedSignalProfile,
    ) -> Self {
        let base = profile.base();

        Self {
            fingerprint,
            observation_count: 1,
            first_updated_at: update_index,
            last_updated_at: update_index,
            uncertainty: base.uncertainty(),
            learning_progress: base.learning_progress_signal(),
            compression_gain: profile.compression_gain(),
            controllability: profile.controllability(),
        }
    }

    fn updated(
        previous: EpistemicSelfRecord,
        update_index: u64,
        profile: MindstoneExtendedSignalProfile,
    ) -> Self {
        let base = profile.base();

        Self {
            fingerprint: previous.fingerprint,
            observation_count: previous.observation_count.saturating_add(1),
            first_updated_at: previous.first_updated_at,
            last_updated_at: update_index,
            uncertainty: base.uncertainty(),
            learning_progress: base.learning_progress_signal(),
            compression_gain: profile.compression_gain(),
            controllability: profile.controllability(),
        }
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn observation_count(&self) -> u64 {
        self.observation_count
    }

    pub fn first_updated_at(&self) -> u64 {
        self.first_updated_at
    }

    pub fn last_updated_at(&self) -> u64 {
        self.last_updated_at
    }

    pub fn uncertainty(&self) -> CognitiveSignal {
        self.uncertainty
    }

    pub fn learning_progress(&self) -> CognitiveSignal {
        self.learning_progress
    }

    pub fn compression_gain(&self) -> CognitiveSignal {
        self.compression_gain
    }

    pub fn controllability(&self) -> CognitiveSignal {
        self.controllability
    }

    pub fn assessment(&self, policy: EpistemicSelfPolicy) -> EpistemicSelfAssessment {
        policy.assess(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicSelfState {
    capacity: usize,
    last_update_index: Option<u64>,
    records: std::collections::BTreeMap<CognitiveFingerprint, EpistemicSelfRecord>,
}

impl EpistemicSelfState {
    pub fn new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            return None;
        }

        Some(Self {
            capacity,
            last_update_index: None,
            records: std::collections::BTreeMap::new(),
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    pub fn last_update_index(&self) -> Option<u64> {
        self.last_update_index
    }

    pub fn contains(&self, fingerprint: CognitiveFingerprint) -> bool {
        self.records.contains_key(&fingerprint)
    }

    pub fn record(&self, fingerprint: CognitiveFingerprint) -> Option<&EpistemicSelfRecord> {
        self.records.get(&fingerprint)
    }

    fn evict_oldest(&mut self) -> Option<CognitiveFingerprint> {
        let victim = self
            .records
            .iter()
            .min_by_key(|(fingerprint, record)| (record.last_updated_at(), **fingerprint))
            .map(|(fingerprint, _)| *fingerprint);

        if let Some(fingerprint) = victim {
            self.records.remove(&fingerprint);
        }

        victim
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EpistemicSelfUpdateStatus {
    RejectedOutOfOrder,
    Updated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicSelfUpdateResult {
    state_before: EpistemicSelfState,
    state_after: EpistemicSelfState,
    update_index: u64,
    fingerprint: CognitiveFingerprint,
    evicted: Option<CognitiveFingerprint>,
    record: Option<EpistemicSelfRecord>,
    assessment: Option<EpistemicSelfAssessment>,
    status: EpistemicSelfUpdateStatus,
}

impl EpistemicSelfUpdateResult {
    pub fn state_before(&self) -> &EpistemicSelfState {
        &self.state_before
    }

    pub fn state_after(&self) -> &EpistemicSelfState {
        &self.state_after
    }

    pub fn update_index(&self) -> u64 {
        self.update_index
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn evicted(&self) -> Option<CognitiveFingerprint> {
        self.evicted
    }

    pub fn record(&self) -> Option<&EpistemicSelfRecord> {
        self.record.as_ref()
    }

    pub fn assessment(&self) -> Option<EpistemicSelfAssessment> {
        self.assessment
    }

    pub fn status(&self) -> EpistemicSelfUpdateStatus {
        self.status
    }

    pub fn accepted(&self) -> bool {
        self.status == EpistemicSelfUpdateStatus::Updated
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EpistemicSelfModel;

impl EpistemicSelfModel {
    pub fn observe(
        state: EpistemicSelfState,
        update_index: u64,
        fingerprint: CognitiveFingerprint,
        profile: MindstoneExtendedSignalProfile,
        policy: EpistemicSelfPolicy,
    ) -> EpistemicSelfUpdateResult {
        let state_before = state.clone();

        if let Some(previous_index) = state.last_update_index() {
            if update_index <= previous_index {
                return EpistemicSelfUpdateResult {
                    state_before,
                    state_after: state,
                    update_index,
                    fingerprint,
                    evicted: None,
                    record: None,
                    assessment: None,
                    status: EpistemicSelfUpdateStatus::RejectedOutOfOrder,
                };
            }
        }

        let mut state_after = state;

        let previous = state_after.records.remove(&fingerprint);

        let evicted = if previous.is_none() && state_after.is_full() {
            state_after.evict_oldest()
        } else {
            None
        };

        let record = match previous {
            Some(previous_record) => {
                EpistemicSelfRecord::updated(previous_record, update_index, profile)
            }

            None => EpistemicSelfRecord::new(fingerprint, update_index, profile),
        };

        let assessment = record.assessment(policy);

        state_after.records.insert(fingerprint, record.clone());

        state_after.last_update_index = Some(update_index);

        EpistemicSelfUpdateResult {
            state_before,
            state_after,
            update_index,
            fingerprint,
            evicted,
            record: Some(record),
            assessment: Some(assessment),
            status: EpistemicSelfUpdateStatus::Updated,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneEpistemicSelfModel;

impl MindstoneEpistemicSelfModel {
    pub fn observe_structure(
        state: EpistemicSelfState,
        update_index: u64,
        structure: &CognitiveStructure,
        profile: MindstoneExtendedSignalProfile,
        policy: EpistemicSelfPolicy,
    ) -> EpistemicSelfUpdateResult {
        let fingerprint = StructuralHasher::fingerprint(structure);

        EpistemicSelfModel::observe(state, update_index, fingerprint, profile, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelfGeneratedGoalKind {
    ResolveUncertainty,
    ContinueLearning,
    TestControl,
    CompressRepresentation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SelfGeneratedGoal {
    fingerprint: CognitiveFingerprint,
    kind: SelfGeneratedGoalKind,
    priority: CognitiveSignal,
    estimated_cost: u32,
}

impl SelfGeneratedGoal {
    pub fn new(
        fingerprint: CognitiveFingerprint,
        kind: SelfGeneratedGoalKind,
        priority: CognitiveSignal,
        estimated_cost: u32,
    ) -> Option<Self> {
        if priority == CognitiveSignal::zero() || estimated_cost == 0 {
            return None;
        }

        Some(Self {
            fingerprint,
            kind,
            priority,
            estimated_cost,
        })
    }

    pub fn fingerprint(self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn kind(self) -> SelfGeneratedGoalKind {
        self.kind
    }

    pub fn priority(self) -> CognitiveSignal {
        self.priority
    }

    pub fn estimated_cost(self) -> u32 {
        self.estimated_cost
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelfGeneratedGoalPolicy {
    max_goals: usize,
    resolve_uncertainty_cost: u32,
    continue_learning_cost: u32,
    test_control_cost: u32,
    compress_representation_cost: u32,
}

impl SelfGeneratedGoalPolicy {
    pub fn new(
        max_goals: usize,
        resolve_uncertainty_cost: u32,
        continue_learning_cost: u32,
        test_control_cost: u32,
        compress_representation_cost: u32,
    ) -> Option<Self> {
        if max_goals == 0
            || resolve_uncertainty_cost == 0
            || continue_learning_cost == 0
            || test_control_cost == 0
            || compress_representation_cost == 0
        {
            return None;
        }

        Some(Self {
            max_goals,
            resolve_uncertainty_cost,
            continue_learning_cost,
            test_control_cost,
            compress_representation_cost,
        })
    }

    pub fn max_goals(self) -> usize {
        self.max_goals
    }

    pub fn resolve_uncertainty_cost(self) -> u32 {
        self.resolve_uncertainty_cost
    }

    pub fn continue_learning_cost(self) -> u32 {
        self.continue_learning_cost
    }

    pub fn test_control_cost(self) -> u32 {
        self.test_control_cost
    }

    pub fn compress_representation_cost(self) -> u32 {
        self.compress_representation_cost
    }

    fn cost_for(self, kind: SelfGeneratedGoalKind) -> u32 {
        match kind {
            SelfGeneratedGoalKind::ResolveUncertainty => self.resolve_uncertainty_cost,
            SelfGeneratedGoalKind::ContinueLearning => self.continue_learning_cost,
            SelfGeneratedGoalKind::TestControl => self.test_control_cost,
            SelfGeneratedGoalKind::CompressRepresentation => self.compress_representation_cost,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfGeneratedGoalResult {
    source_record_count: usize,
    candidate_goal_count: usize,
    selected: Vec<SelfGeneratedGoal>,
    total_selected_cost: u32,
    truncated_by_goal_limit: bool,
    truncated_by_compute_budget: bool,
}

impl SelfGeneratedGoalResult {
    pub fn source_record_count(&self) -> usize {
        self.source_record_count
    }

    pub fn candidate_goal_count(&self) -> usize {
        self.candidate_goal_count
    }

    pub fn selected(&self) -> &[SelfGeneratedGoal] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn total_selected_cost(&self) -> u32 {
        self.total_selected_cost
    }

    pub fn truncated_by_goal_limit(&self) -> bool {
        self.truncated_by_goal_limit
    }

    pub fn truncated_by_compute_budget(&self) -> bool {
        self.truncated_by_compute_budget
    }

    pub fn was_truncated(&self) -> bool {
        self.truncated_by_goal_limit || self.truncated_by_compute_budget
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelfGeneratedGoalEngine;

impl SelfGeneratedGoalEngine {
    fn primary_goal(
        record: &EpistemicSelfRecord,
        assessment: EpistemicSelfAssessment,
        policy: SelfGeneratedGoalPolicy,
    ) -> Option<SelfGeneratedGoal> {
        let (kind, priority) = if assessment.is_stable() && assessment.is_compressible() {
            (
                SelfGeneratedGoalKind::CompressRepresentation,
                record.compression_gain(),
            )
        } else if !assessment.is_stable() && assessment.is_controllable() {
            (SelfGeneratedGoalKind::TestControl, record.controllability())
        } else if assessment.is_learning() {
            (
                SelfGeneratedGoalKind::ContinueLearning,
                record.learning_progress(),
            )
        } else if assessment.is_uncertain() {
            (
                SelfGeneratedGoalKind::ResolveUncertainty,
                record.uncertainty(),
            )
        } else {
            return None;
        };

        SelfGeneratedGoal::new(record.fingerprint(), kind, priority, policy.cost_for(kind))
    }

    fn ranking(left: &SelfGeneratedGoal, right: &SelfGeneratedGoal) -> std::cmp::Ordering {
        right
            .priority()
            .cmp(&left.priority())
            .then_with(|| left.estimated_cost().cmp(&right.estimated_cost()))
            .then_with(|| left.kind().cmp(&right.kind()))
            .then_with(|| left.fingerprint().cmp(&right.fingerprint()))
    }

    pub fn generate(
        state: &EpistemicSelfState,
        epistemic_policy: EpistemicSelfPolicy,
        goal_policy: SelfGeneratedGoalPolicy,
        budget: CognitiveBudget,
    ) -> SelfGeneratedGoalResult {
        let source_record_count = state.len();

        let mut candidates = state
            .records
            .values()
            .filter_map(|record| {
                let assessment = record.assessment(epistemic_policy);

                Self::primary_goal(record, assessment, goal_policy)
            })
            .collect::<Vec<_>>();

        candidates.sort_by(Self::ranking);

        let candidate_goal_count = candidates.len();

        let mut selected = Vec::with_capacity(goal_policy.max_goals().min(candidate_goal_count));

        let mut total_selected_cost = 0_u32;

        let mut truncated_by_goal_limit = false;

        let mut truncated_by_compute_budget = false;

        for (index, goal) in candidates.into_iter().enumerate() {
            if selected.len() >= goal_policy.max_goals() {
                truncated_by_goal_limit = index < candidate_goal_count;

                break;
            }

            let Some(next_total) = total_selected_cost.checked_add(goal.estimated_cost()) else {
                truncated_by_compute_budget = true;

                break;
            };

            if next_total > budget.units() {
                truncated_by_compute_budget = true;

                break;
            }

            total_selected_cost = next_total;

            selected.push(goal);
        }

        SelfGeneratedGoalResult {
            source_record_count,
            candidate_goal_count,
            selected,
            total_selected_cost,
            truncated_by_goal_limit,
            truncated_by_compute_budget,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneSelfGeneratedGoals;

impl MindstoneSelfGeneratedGoals {
    pub fn evaluate(
        state: &EpistemicSelfState,
        epistemic_policy: EpistemicSelfPolicy,
        goal_policy: SelfGeneratedGoalPolicy,
        budget: CognitiveBudget,
    ) -> SelfGeneratedGoalResult {
        SelfGeneratedGoalEngine::generate(state, epistemic_policy, goal_policy, budget)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EpistemicOutcomePrediction {
    weight: u32,
    resulting_uncertainty: CognitiveSignal,
}

impl EpistemicOutcomePrediction {
    pub fn new(weight: u32, resulting_uncertainty: CognitiveSignal) -> Option<Self> {
        if weight == 0 {
            return None;
        }

        Some(Self {
            weight,
            resulting_uncertainty,
        })
    }

    pub fn weight(self) -> u32 {
        self.weight
    }

    pub fn resulting_uncertainty(self) -> CognitiveSignal {
        self.resulting_uncertainty
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedInformationGainEstimate {
    current_uncertainty: CognitiveSignal,
    expected_uncertainty: CognitiveSignal,
    information_gain: CognitiveSignal,
    total_weight: u128,
    outcome_count: usize,
}

impl ExpectedInformationGainEstimate {
    pub fn current_uncertainty(&self) -> CognitiveSignal {
        self.current_uncertainty
    }

    pub fn expected_uncertainty(&self) -> CognitiveSignal {
        self.expected_uncertainty
    }

    pub fn information_gain(&self) -> CognitiveSignal {
        self.information_gain
    }

    pub fn total_weight(&self) -> u128 {
        self.total_weight
    }

    pub fn outcome_count(&self) -> usize {
        self.outcome_count
    }

    pub fn predicts_learning(&self) -> bool {
        self.information_gain > CognitiveSignal::zero()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExpectedInformationGain;

impl ExpectedInformationGain {
    pub fn estimate(
        current_uncertainty: CognitiveSignal,
        outcomes: &[EpistemicOutcomePrediction],
    ) -> Option<ExpectedInformationGainEstimate> {
        if outcomes.is_empty() {
            return None;
        }

        let total_weight = outcomes.iter().fold(0_u128, |total, outcome| {
            total.saturating_add(u128::from(outcome.weight()))
        });

        if total_weight == 0 {
            return None;
        }

        let weighted_uncertainty = outcomes.iter().fold(0_u128, |total, outcome| {
            let contribution = u128::from(outcome.weight())
                .saturating_mul(u128::from(outcome.resulting_uncertainty().value()));

            total.saturating_add(contribution)
        });

        let expected_value =
            weighted_uncertainty.saturating_add(total_weight.saturating_sub(1)) / total_weight;

        let expected_uncertainty = CognitiveSignal::new(expected_value as u16)?;

        let information_gain = if current_uncertainty > expected_uncertainty {
            CognitiveSignal::new(
                current_uncertainty
                    .value()
                    .saturating_sub(expected_uncertainty.value()),
            )?
        } else {
            CognitiveSignal::zero()
        };

        Some(ExpectedInformationGainEstimate {
            current_uncertainty,
            expected_uncertainty,
            information_gain,
            total_weight,
            outcome_count: outcomes.len(),
        })
    }
}

impl MindstoneSignalProfile {
    pub fn with_information_gain(mut self, information_gain: CognitiveSignal) -> Self {
        self.information_gain = information_gain;
        self
    }

    pub fn expected_information_gain_signal(self) -> CognitiveSignal {
        self.information_gain
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MindstoneExpectedInformationGainResult {
    estimate: ExpectedInformationGainEstimate,
    profile: MindstoneSignalProfile,
    decision: CognitiveAdmissionDecision,
}

impl MindstoneExpectedInformationGainResult {
    pub fn estimate(&self) -> &ExpectedInformationGainEstimate {
        &self.estimate
    }

    pub fn profile(&self) -> MindstoneSignalProfile {
        self.profile
    }

    pub fn decision(&self) -> CognitiveAdmissionDecision {
        self.decision
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneExpectedInformationGain;

impl MindstoneExpectedInformationGain {
    pub fn evaluate(
        base_profile: MindstoneSignalProfile,
        outcomes: &[EpistemicOutcomePrediction],
        policy: SparseCognitionPolicy,
        budget: CognitiveBudget,
    ) -> Option<MindstoneExpectedInformationGainResult> {
        let estimate = ExpectedInformationGain::estimate(base_profile.uncertainty(), outcomes)?;

        let profile = base_profile.with_information_gain(estimate.information_gain());

        let decision = policy.admit(profile, budget);

        Some(MindstoneExpectedInformationGainResult {
            estimate,
            profile,
            decision,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalInformationPrediction {
    fingerprint: CognitiveFingerprint,
    outcomes: Vec<EpistemicOutcomePrediction>,
}

impl GoalInformationPrediction {
    pub fn new(
        fingerprint: CognitiveFingerprint,
        outcomes: Vec<EpistemicOutcomePrediction>,
    ) -> Option<Self> {
        if outcomes.is_empty() {
            return None;
        }

        Some(Self {
            fingerprint,
            outcomes,
        })
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn outcomes(&self) -> &[EpistemicOutcomePrediction] {
        &self.outcomes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalInformationPredictionSet {
    predictions: std::collections::BTreeMap<CognitiveFingerprint, Vec<EpistemicOutcomePrediction>>,
}

impl GoalInformationPredictionSet {
    pub fn new(predictions: Vec<GoalInformationPrediction>) -> Option<Self> {
        let mut canonical = std::collections::BTreeMap::new();

        for prediction in predictions {
            if canonical
                .insert(prediction.fingerprint(), prediction.outcomes)
                .is_some()
            {
                return None;
            }
        }

        Some(Self {
            predictions: canonical,
        })
    }

    pub fn empty() -> Self {
        Self {
            predictions: std::collections::BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.predictions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.predictions.is_empty()
    }

    pub fn contains(&self, fingerprint: CognitiveFingerprint) -> bool {
        self.predictions.contains_key(&fingerprint)
    }

    pub fn outcomes_for(
        &self,
        fingerprint: CognitiveFingerprint,
    ) -> Option<&[EpistemicOutcomePrediction]> {
        self.predictions.get(&fingerprint).map(Vec::as_slice)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InformationRankedGoal {
    goal: SelfGeneratedGoal,
    information_gain: CognitiveSignal,
    expected_uncertainty: Option<CognitiveSignal>,
    has_prediction: bool,
}

impl InformationRankedGoal {
    pub fn goal(self) -> SelfGeneratedGoal {
        self.goal
    }

    pub fn fingerprint(self) -> CognitiveFingerprint {
        self.goal.fingerprint()
    }

    pub fn kind(self) -> SelfGeneratedGoalKind {
        self.goal.kind()
    }

    pub fn base_priority(self) -> CognitiveSignal {
        self.goal.priority()
    }

    pub fn estimated_cost(self) -> u32 {
        self.goal.estimated_cost()
    }

    pub fn information_gain(self) -> CognitiveSignal {
        self.information_gain
    }

    pub fn expected_uncertainty(self) -> Option<CognitiveSignal> {
        self.expected_uncertainty
    }

    pub fn has_prediction(self) -> bool {
        self.has_prediction
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InformationGainGoalRankingResult {
    source_record_count: usize,
    candidate_goal_count: usize,
    prediction_count: usize,
    matched_prediction_count: usize,
    selected: Vec<InformationRankedGoal>,
    total_selected_cost: u32,
    truncated_by_goal_limit: bool,
    truncated_by_compute_budget: bool,
}

impl InformationGainGoalRankingResult {
    pub fn source_record_count(&self) -> usize {
        self.source_record_count
    }

    pub fn candidate_goal_count(&self) -> usize {
        self.candidate_goal_count
    }

    pub fn prediction_count(&self) -> usize {
        self.prediction_count
    }

    pub fn matched_prediction_count(&self) -> usize {
        self.matched_prediction_count
    }

    pub fn selected(&self) -> &[InformationRankedGoal] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn total_selected_cost(&self) -> u32 {
        self.total_selected_cost
    }

    pub fn truncated_by_goal_limit(&self) -> bool {
        self.truncated_by_goal_limit
    }

    pub fn truncated_by_compute_budget(&self) -> bool {
        self.truncated_by_compute_budget
    }

    pub fn was_truncated(&self) -> bool {
        self.truncated_by_goal_limit || self.truncated_by_compute_budget
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InformationGainGoalRanking;

impl InformationGainGoalRanking {
    fn enrich_goal(
        goal: SelfGeneratedGoal,
        record: &EpistemicSelfRecord,
        predictions: &GoalInformationPredictionSet,
    ) -> InformationRankedGoal {
        if let Some(outcomes) = predictions.outcomes_for(goal.fingerprint()) {
            let estimate = ExpectedInformationGain::estimate(record.uncertainty(), outcomes)
                .expect("validated nonempty prediction outcomes");

            InformationRankedGoal {
                goal,
                information_gain: estimate.information_gain(),
                expected_uncertainty: Some(estimate.expected_uncertainty()),
                has_prediction: true,
            }
        } else {
            InformationRankedGoal {
                goal,
                information_gain: CognitiveSignal::zero(),
                expected_uncertainty: None,
                has_prediction: false,
            }
        }
    }

    fn ranking(left: &InformationRankedGoal, right: &InformationRankedGoal) -> std::cmp::Ordering {
        right
            .information_gain()
            .cmp(&left.information_gain())
            .then_with(|| right.base_priority().cmp(&left.base_priority()))
            .then_with(|| left.estimated_cost().cmp(&right.estimated_cost()))
            .then_with(|| left.kind().cmp(&right.kind()))
            .then_with(|| left.fingerprint().cmp(&right.fingerprint()))
    }

    pub fn rank(
        state: &EpistemicSelfState,
        epistemic_policy: EpistemicSelfPolicy,
        goal_policy: SelfGeneratedGoalPolicy,
        predictions: &GoalInformationPredictionSet,
        budget: CognitiveBudget,
    ) -> InformationGainGoalRankingResult {
        let source_record_count = state.len();

        let mut candidates = state
            .records
            .values()
            .filter_map(|record| {
                let assessment = record.assessment(epistemic_policy);

                let goal = SelfGeneratedGoalEngine::primary_goal(record, assessment, goal_policy)?;

                Some(Self::enrich_goal(goal, record, predictions))
            })
            .collect::<Vec<_>>();

        candidates.sort_by(Self::ranking);

        let candidate_goal_count = candidates.len();

        let matched_prediction_count = candidates
            .iter()
            .filter(|candidate| candidate.has_prediction())
            .count();

        let mut selected = Vec::with_capacity(goal_policy.max_goals().min(candidate_goal_count));

        let mut total_selected_cost = 0_u32;

        let mut truncated_by_goal_limit = false;

        let mut truncated_by_compute_budget = false;

        for (index, candidate) in candidates.into_iter().enumerate() {
            if selected.len() >= goal_policy.max_goals() {
                truncated_by_goal_limit = index < candidate_goal_count;

                break;
            }

            let Some(next_total) = total_selected_cost.checked_add(candidate.estimated_cost())
            else {
                truncated_by_compute_budget = true;

                break;
            };

            if next_total > budget.units() {
                truncated_by_compute_budget = true;

                break;
            }

            total_selected_cost = next_total;

            selected.push(candidate);
        }

        InformationGainGoalRankingResult {
            source_record_count,
            candidate_goal_count,
            prediction_count: predictions.len(),
            matched_prediction_count,
            selected,
            total_selected_cost,
            truncated_by_goal_limit,
            truncated_by_compute_budget,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneInformationGainGoalRanking;

impl MindstoneInformationGainGoalRanking {
    pub fn evaluate(
        state: &EpistemicSelfState,
        epistemic_policy: EpistemicSelfPolicy,
        goal_policy: SelfGeneratedGoalPolicy,
        predictions: &GoalInformationPredictionSet,
        budget: CognitiveBudget,
    ) -> InformationGainGoalRankingResult {
        InformationGainGoalRanking::rank(state, epistemic_policy, goal_policy, predictions, budget)
    }
}

#[derive(Clone, Debug)]
pub struct CollisionSafeStructuralIdentity {
    fingerprint: CognitiveFingerprint,
    structure: CognitiveStructure,
}

impl CollisionSafeStructuralIdentity {
    pub fn from_structure(structure: CognitiveStructure) -> Self {
        let fingerprint = StructuralHasher::fingerprint(&structure);

        Self {
            fingerprint,
            structure,
        }
    }

    pub fn with_fingerprint_hint(
        fingerprint: CognitiveFingerprint,
        structure: CognitiveStructure,
    ) -> Self {
        Self {
            fingerprint,
            structure,
        }
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.fingerprint
    }

    pub fn structure(&self) -> &CognitiveStructure {
        &self.structure
    }

    pub fn semantically_matches(&self, other: &Self) -> bool {
        self.structure == other.structure
    }
}

impl PartialEq for CollisionSafeStructuralIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.semantically_matches(other)
    }
}

impl Eq for CollisionSafeStructuralIdentity {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeStructuralRecord {
    identity: CollisionSafeStructuralIdentity,
    observation_count: u64,
    first_seen: u64,
    last_seen: u64,
    total_salience: u128,
    peak_salience: CognitiveSalience,
}

impl CollisionSafeStructuralRecord {
    fn new(
        identity: CollisionSafeStructuralIdentity,
        event_index: u64,
        salience: CognitiveSalience,
    ) -> Self {
        Self {
            identity,
            observation_count: 1,
            first_seen: event_index,
            last_seen: event_index,
            total_salience: u128::from(salience.value()),
            peak_salience: salience,
        }
    }

    fn observe(&mut self, event_index: u64, salience: CognitiveSalience) {
        self.observation_count = self.observation_count.saturating_add(1);

        self.last_seen = event_index;

        self.total_salience = self
            .total_salience
            .saturating_add(u128::from(salience.value()));

        if salience > self.peak_salience {
            self.peak_salience = salience;
        }
    }

    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.identity.fingerprint()
    }

    pub fn structure(&self) -> &CognitiveStructure {
        self.identity.structure()
    }

    pub fn observation_count(&self) -> u64 {
        self.observation_count
    }

    pub fn first_seen(&self) -> u64 {
        self.first_seen
    }

    pub fn last_seen(&self) -> u64 {
        self.last_seen
    }

    pub fn total_salience(&self) -> u128 {
        self.total_salience
    }

    pub fn mean_salience_value(&self) -> u16 {
        if self.observation_count == 0 {
            return 0;
        }

        let mean = self.total_salience / u128::from(self.observation_count);

        mean as u16
    }

    pub fn peak_salience(&self) -> CognitiveSalience {
        self.peak_salience
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeStructuralState {
    capacity: usize,
    last_event_index: Option<u64>,
    buckets: std::collections::BTreeMap<CognitiveFingerprint, Vec<CollisionSafeStructuralRecord>>,
}

impl CollisionSafeStructuralState {
    pub fn new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            return None;
        }

        Some(Self {
            capacity,
            last_event_index: None,
            buckets: std::collections::BTreeMap::new(),
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.buckets.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    pub fn last_event_index(&self) -> Option<u64> {
        self.last_event_index
    }

    pub fn bucket_len(&self, fingerprint: CognitiveFingerprint) -> usize {
        self.buckets.get(&fingerprint).map_or(0, Vec::len)
    }

    pub fn records_in_bucket(
        &self,
        fingerprint: CognitiveFingerprint,
    ) -> Option<&[CollisionSafeStructuralRecord]> {
        self.buckets.get(&fingerprint).map(Vec::as_slice)
    }

    pub fn record_for_identity(
        &self,
        identity: &CollisionSafeStructuralIdentity,
    ) -> Option<&CollisionSafeStructuralRecord> {
        self.buckets
            .get(&identity.fingerprint())
            .and_then(|bucket| {
                bucket
                    .iter()
                    .find(|record| record.identity().semantically_matches(identity))
            })
    }

    pub fn record_for_structure(
        &self,
        structure: &CognitiveStructure,
    ) -> Option<&CollisionSafeStructuralRecord> {
        let fingerprint = StructuralHasher::fingerprint(structure);

        self.buckets
            .get(&fingerprint)
            .and_then(|bucket| bucket.iter().find(|record| record.structure() == structure))
    }

    pub fn contains_structure(&self, structure: &CognitiveStructure) -> bool {
        self.record_for_structure(structure).is_some()
    }

    pub fn total_retained_observations(&self) -> u64 {
        self.buckets
            .values()
            .flat_map(|bucket| bucket.iter())
            .fold(0_u64, |total, record| {
                total.saturating_add(record.observation_count())
            })
    }

    fn find_exact_position(&self, identity: &CollisionSafeStructuralIdentity) -> Option<usize> {
        self.buckets
            .get(&identity.fingerprint())
            .and_then(|bucket| {
                bucket
                    .iter()
                    .position(|record| record.identity().semantically_matches(identity))
            })
    }

    fn evict_oldest(&mut self) -> Option<CollisionSafeStructuralIdentity> {
        let mut victim: Option<(u64, CognitiveFingerprint, usize)> = None;

        for (fingerprint, bucket) in &self.buckets {
            for (index, record) in bucket.iter().enumerate() {
                let candidate = (record.last_seen(), *fingerprint, index);

                if victim.as_ref().is_none_or(|current| candidate < *current) {
                    victim = Some(candidate);
                }
            }
        }

        let (_, fingerprint, index) = victim?;

        let (removed, bucket_empty) = {
            let bucket = self
                .buckets
                .get_mut(&fingerprint)
                .expect("selected collision-safe bucket exists");

            let removed = bucket.remove(index);

            (removed, bucket.is_empty())
        };

        if bucket_empty {
            self.buckets.remove(&fingerprint);
        }

        Some(removed.identity)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CollisionSafeStructuralObservationStatus {
    RejectedOutOfOrder,
    NewStructure,
    ExistingStructure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeStructuralObservationResult {
    state_before: CollisionSafeStructuralState,
    state_after: CollisionSafeStructuralState,
    event_index: u64,
    identity: CollisionSafeStructuralIdentity,
    novelty: CognitiveSignal,
    evicted: Option<CollisionSafeStructuralIdentity>,
    record: Option<CollisionSafeStructuralRecord>,
    status: CollisionSafeStructuralObservationStatus,
}

impl CollisionSafeStructuralObservationResult {
    pub fn state_before(&self) -> &CollisionSafeStructuralState {
        &self.state_before
    }

    pub fn state_after(&self) -> &CollisionSafeStructuralState {
        &self.state_after
    }

    pub fn event_index(&self) -> u64 {
        self.event_index
    }

    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn novelty(&self) -> CognitiveSignal {
        self.novelty
    }

    pub fn evicted(&self) -> Option<&CollisionSafeStructuralIdentity> {
        self.evicted.as_ref()
    }

    pub fn record(&self) -> Option<&CollisionSafeStructuralRecord> {
        self.record.as_ref()
    }

    pub fn status(&self) -> CollisionSafeStructuralObservationStatus {
        self.status
    }

    pub fn accepted(&self) -> bool {
        self.status != CollisionSafeStructuralObservationStatus::RejectedOutOfOrder
    }

    pub fn is_novel(&self) -> bool {
        self.status == CollisionSafeStructuralObservationStatus::NewStructure
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CollisionSafeStructuralCognition;

impl CollisionSafeStructuralCognition {
    fn observe_identity(
        state: CollisionSafeStructuralState,
        event_index: u64,
        identity: CollisionSafeStructuralIdentity,
        salience: CognitiveSalience,
    ) -> CollisionSafeStructuralObservationResult {
        let state_before = state.clone();

        if let Some(previous_index) = state.last_event_index() {
            if event_index <= previous_index {
                return CollisionSafeStructuralObservationResult {
                    state_before,
                    state_after: state,
                    event_index,
                    identity,
                    novelty: CognitiveSignal::zero(),
                    evicted: None,
                    record: None,
                    status: CollisionSafeStructuralObservationStatus::RejectedOutOfOrder,
                };
            }
        }

        let mut state_after = state;

        let fingerprint = identity.fingerprint();

        let existing_position = state_after.find_exact_position(&identity);

        let (status, novelty, evicted, record) = if let Some(position) = existing_position {
            let bucket = state_after
                .buckets
                .get_mut(&fingerprint)
                .expect("existing collision-safe bucket exists");

            let existing = bucket
                .get_mut(position)
                .expect("existing collision-safe record exists");

            existing.observe(event_index, salience);

            (
                CollisionSafeStructuralObservationStatus::ExistingStructure,
                CognitiveSignal::zero(),
                None,
                existing.clone(),
            )
        } else {
            let evicted = if state_after.is_full() {
                state_after.evict_oldest()
            } else {
                None
            };

            let record =
                CollisionSafeStructuralRecord::new(identity.clone(), event_index, salience);

            state_after
                .buckets
                .entry(fingerprint)
                .or_default()
                .push(record.clone());

            (
                CollisionSafeStructuralObservationStatus::NewStructure,
                CognitiveSignal::maximum(),
                evicted,
                record,
            )
        };

        state_after.last_event_index = Some(event_index);

        CollisionSafeStructuralObservationResult {
            state_before,
            state_after,
            event_index,
            identity,
            novelty,
            evicted,
            record: Some(record),
            status,
        }
    }

    pub fn observe(
        state: CollisionSafeStructuralState,
        event_index: u64,
        structure: CognitiveStructure,
        salience: CognitiveSalience,
    ) -> CollisionSafeStructuralObservationResult {
        let identity = CollisionSafeStructuralIdentity::from_structure(structure);

        Self::observe_identity(state, event_index, identity, salience)
    }

    pub fn observe_with_fingerprint_hint(
        state: CollisionSafeStructuralState,
        event_index: u64,
        fingerprint: CognitiveFingerprint,
        structure: CognitiveStructure,
        salience: CognitiveSalience,
    ) -> CollisionSafeStructuralObservationResult {
        let identity =
            CollisionSafeStructuralIdentity::with_fingerprint_hint(fingerprint, structure);

        Self::observe_identity(state, event_index, identity, salience)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneCollisionSafeStructuralCognition;

impl MindstoneCollisionSafeStructuralCognition {
    pub fn observe(
        state: CollisionSafeStructuralState,
        event_index: u64,
        structure: CognitiveStructure,
        salience: CognitiveSalience,
    ) -> CollisionSafeStructuralObservationResult {
        CollisionSafeStructuralCognition::observe(state, event_index, structure, salience)
    }
}

impl MindstoneSignalProfile {
    pub fn with_integrated_novelty(mut self, novelty: CognitiveSignal) -> Self {
        self.novelty = novelty;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeEpistemicRecord {
    identity: CollisionSafeStructuralIdentity,
    observation_count: u64,
    first_updated_at: u64,
    last_updated_at: u64,
    uncertainty: CognitiveSignal,
    learning_progress: CognitiveSignal,
    compression_gain: CognitiveSignal,
    controllability: CognitiveSignal,
}

impl CollisionSafeEpistemicRecord {
    fn new(
        identity: CollisionSafeStructuralIdentity,
        update_index: u64,
        profile: MindstoneExtendedSignalProfile,
    ) -> Self {
        let base = profile.base();

        Self {
            identity,
            observation_count: 1,
            first_updated_at: update_index,
            last_updated_at: update_index,
            uncertainty: base.uncertainty(),
            learning_progress: base.learning_progress_signal(),
            compression_gain: profile.compression_gain(),
            controllability: profile.controllability(),
        }
    }

    fn updated(
        previous: CollisionSafeEpistemicRecord,
        update_index: u64,
        profile: MindstoneExtendedSignalProfile,
    ) -> Self {
        let base = profile.base();

        Self {
            identity: previous.identity,
            observation_count: previous.observation_count.saturating_add(1),
            first_updated_at: previous.first_updated_at,
            last_updated_at: update_index,
            uncertainty: base.uncertainty(),
            learning_progress: base.learning_progress_signal(),
            compression_gain: profile.compression_gain(),
            controllability: profile.controllability(),
        }
    }

    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.identity.fingerprint()
    }

    pub fn structure(&self) -> &CognitiveStructure {
        self.identity.structure()
    }

    pub fn observation_count(&self) -> u64 {
        self.observation_count
    }

    pub fn first_updated_at(&self) -> u64 {
        self.first_updated_at
    }

    pub fn last_updated_at(&self) -> u64 {
        self.last_updated_at
    }

    pub fn uncertainty(&self) -> CognitiveSignal {
        self.uncertainty
    }

    pub fn learning_progress(&self) -> CognitiveSignal {
        self.learning_progress
    }

    pub fn compression_gain(&self) -> CognitiveSignal {
        self.compression_gain
    }

    pub fn controllability(&self) -> CognitiveSignal {
        self.controllability
    }

    pub fn assessment(&self, policy: EpistemicSelfPolicy) -> EpistemicSelfAssessment {
        let sufficiently_observed = self.observation_count() >= policy.minimum_observations();

        let class =
            if sufficiently_observed && self.uncertainty() <= policy.stable_uncertainty_max() {
                EpistemicSelfClass::Stable
            } else if self.learning_progress() >= policy.learning_progress_min() {
                EpistemicSelfClass::Learning
            } else {
                EpistemicSelfClass::Uncertain
            };

        EpistemicSelfAssessment {
            class,
            compressible: sufficiently_observed
                && self.compression_gain() >= policy.compression_gain_min(),
            controllable: sufficiently_observed
                && self.controllability() >= policy.controllability_min(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeEpistemicState {
    capacity: usize,
    last_update_index: Option<u64>,
    buckets: std::collections::BTreeMap<CognitiveFingerprint, Vec<CollisionSafeEpistemicRecord>>,
}

impl CollisionSafeEpistemicState {
    pub fn new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            return None;
        }

        Some(Self {
            capacity,
            last_update_index: None,
            buckets: std::collections::BTreeMap::new(),
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.buckets.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    pub fn last_update_index(&self) -> Option<u64> {
        self.last_update_index
    }

    pub fn bucket_len(&self, fingerprint: CognitiveFingerprint) -> usize {
        self.buckets.get(&fingerprint).map_or(0, Vec::len)
    }

    pub fn record_for_identity(
        &self,
        identity: &CollisionSafeStructuralIdentity,
    ) -> Option<&CollisionSafeEpistemicRecord> {
        self.buckets
            .get(&identity.fingerprint())
            .and_then(|bucket| {
                bucket
                    .iter()
                    .find(|record| record.identity().semantically_matches(identity))
            })
    }

    pub fn records_in_bucket(
        &self,
        fingerprint: CognitiveFingerprint,
    ) -> Option<&[CollisionSafeEpistemicRecord]> {
        self.buckets.get(&fingerprint).map(Vec::as_slice)
    }

    fn exact_position(&self, identity: &CollisionSafeStructuralIdentity) -> Option<usize> {
        self.buckets
            .get(&identity.fingerprint())
            .and_then(|bucket| {
                bucket
                    .iter()
                    .position(|record| record.identity().semantically_matches(identity))
            })
    }

    fn evict_oldest(&mut self) -> Option<CollisionSafeStructuralIdentity> {
        let mut victim: Option<(u64, CognitiveFingerprint, usize)> = None;

        for (fingerprint, bucket) in &self.buckets {
            for (index, record) in bucket.iter().enumerate() {
                let candidate = (record.last_updated_at(), *fingerprint, index);

                if victim.as_ref().is_none_or(|current| candidate < *current) {
                    victim = Some(candidate);
                }
            }
        }

        let (_, fingerprint, index) = victim?;

        let (removed, bucket_empty) = {
            let bucket = self
                .buckets
                .get_mut(&fingerprint)
                .expect("selected epistemic bucket exists");

            let removed = bucket.remove(index);

            (removed, bucket.is_empty())
        };

        if bucket_empty {
            self.buckets.remove(&fingerprint);
        }

        Some(removed.identity)
    }

    fn all_records(&self) -> impl Iterator<Item = &CollisionSafeEpistemicRecord> {
        self.buckets.values().flat_map(|bucket| bucket.iter())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CollisionSafeEpistemicUpdateStatus {
    RejectedOutOfOrder,
    Updated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeEpistemicUpdateResult {
    state_before: CollisionSafeEpistemicState,
    state_after: CollisionSafeEpistemicState,
    identity: CollisionSafeStructuralIdentity,
    record: Option<CollisionSafeEpistemicRecord>,
    evicted: Option<CollisionSafeStructuralIdentity>,
    status: CollisionSafeEpistemicUpdateStatus,
}

impl CollisionSafeEpistemicUpdateResult {
    pub fn state_before(&self) -> &CollisionSafeEpistemicState {
        &self.state_before
    }

    pub fn state_after(&self) -> &CollisionSafeEpistemicState {
        &self.state_after
    }

    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn record(&self) -> Option<&CollisionSafeEpistemicRecord> {
        self.record.as_ref()
    }

    pub fn evicted(&self) -> Option<&CollisionSafeStructuralIdentity> {
        self.evicted.as_ref()
    }

    pub fn status(&self) -> CollisionSafeEpistemicUpdateStatus {
        self.status
    }

    pub fn accepted(&self) -> bool {
        self.status == CollisionSafeEpistemicUpdateStatus::Updated
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CollisionSafeEpistemicSelfModel;

impl CollisionSafeEpistemicSelfModel {
    pub fn observe(
        state: CollisionSafeEpistemicState,
        update_index: u64,
        identity: CollisionSafeStructuralIdentity,
        profile: MindstoneExtendedSignalProfile,
    ) -> CollisionSafeEpistemicUpdateResult {
        let state_before = state.clone();

        if let Some(previous_index) = state.last_update_index() {
            if update_index <= previous_index {
                return CollisionSafeEpistemicUpdateResult {
                    state_before,
                    state_after: state,
                    identity,
                    record: None,
                    evicted: None,
                    status: CollisionSafeEpistemicUpdateStatus::RejectedOutOfOrder,
                };
            }
        }

        let mut state_after = state;

        let fingerprint = identity.fingerprint();

        let existing_position = state_after.exact_position(&identity);

        let (record, evicted) = if let Some(position) = existing_position {
            let bucket = state_after
                .buckets
                .get_mut(&fingerprint)
                .expect("existing epistemic bucket exists");

            let previous = bucket.remove(position);

            let updated = CollisionSafeEpistemicRecord::updated(previous, update_index, profile);

            bucket.push(updated.clone());

            (updated, None)
        } else {
            let evicted = if state_after.is_full() {
                state_after.evict_oldest()
            } else {
                None
            };

            let record = CollisionSafeEpistemicRecord::new(identity.clone(), update_index, profile);

            state_after
                .buckets
                .entry(fingerprint)
                .or_default()
                .push(record.clone());

            (record, evicted)
        };

        state_after.last_update_index = Some(update_index);

        CollisionSafeEpistemicUpdateResult {
            state_before,
            state_after,
            identity,
            record: Some(record),
            evicted,
            status: CollisionSafeEpistemicUpdateStatus::Updated,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeGoalInformationPrediction {
    identity: CollisionSafeStructuralIdentity,
    outcomes: Vec<EpistemicOutcomePrediction>,
}

impl CollisionSafeGoalInformationPrediction {
    pub fn new(
        identity: CollisionSafeStructuralIdentity,
        outcomes: Vec<EpistemicOutcomePrediction>,
    ) -> Option<Self> {
        if outcomes.is_empty() {
            return None;
        }

        Some(Self { identity, outcomes })
    }

    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn outcomes(&self) -> &[EpistemicOutcomePrediction] {
        &self.outcomes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeGoalInformationPredictionSet {
    buckets: std::collections::BTreeMap<
        CognitiveFingerprint,
        Vec<CollisionSafeGoalInformationPrediction>,
    >,
}

impl CollisionSafeGoalInformationPredictionSet {
    pub fn new(predictions: Vec<CollisionSafeGoalInformationPrediction>) -> Option<Self> {
        let mut buckets: std::collections::BTreeMap<
            CognitiveFingerprint,
            Vec<CollisionSafeGoalInformationPrediction>,
        > = std::collections::BTreeMap::new();

        for prediction in predictions {
            let fingerprint = prediction.identity().fingerprint();

            let bucket = buckets.entry(fingerprint).or_default();

            if bucket.iter().any(|existing| {
                existing
                    .identity()
                    .semantically_matches(prediction.identity())
            }) {
                return None;
            }

            bucket.push(prediction);
        }

        Some(Self { buckets })
    }

    pub fn empty() -> Self {
        Self {
            buckets: std::collections::BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.buckets.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn outcomes_for(
        &self,
        identity: &CollisionSafeStructuralIdentity,
    ) -> Option<&[EpistemicOutcomePrediction]> {
        self.buckets
            .get(&identity.fingerprint())
            .and_then(|bucket| {
                bucket
                    .iter()
                    .find(|prediction| prediction.identity().semantically_matches(identity))
            })
            .map(CollisionSafeGoalInformationPrediction::outcomes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeRankedGoal {
    identity: CollisionSafeStructuralIdentity,
    kind: SelfGeneratedGoalKind,
    base_priority: CognitiveSignal,
    information_gain: CognitiveSignal,
    expected_uncertainty: Option<CognitiveSignal>,
    estimated_cost: u32,
}

impl CollisionSafeRankedGoal {
    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.identity.fingerprint()
    }

    pub fn structure(&self) -> &CognitiveStructure {
        self.identity.structure()
    }

    pub fn kind(&self) -> SelfGeneratedGoalKind {
        self.kind
    }

    pub fn base_priority(&self) -> CognitiveSignal {
        self.base_priority
    }

    pub fn information_gain(&self) -> CognitiveSignal {
        self.information_gain
    }

    pub fn expected_uncertainty(&self) -> Option<CognitiveSignal> {
        self.expected_uncertainty
    }

    pub fn estimated_cost(&self) -> u32 {
        self.estimated_cost
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionSafeGoalFrontierResult {
    candidate_count: usize,
    selected: Vec<CollisionSafeRankedGoal>,
    total_selected_cost: u32,
    truncated_by_goal_limit: bool,
    truncated_by_compute_budget: bool,
}

impl CollisionSafeGoalFrontierResult {
    fn empty() -> Self {
        Self {
            candidate_count: 0,
            selected: Vec::new(),
            total_selected_cost: 0,
            truncated_by_goal_limit: false,
            truncated_by_compute_budget: false,
        }
    }

    pub fn candidate_count(&self) -> usize {
        self.candidate_count
    }

    pub fn selected(&self) -> &[CollisionSafeRankedGoal] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn total_selected_cost(&self) -> u32 {
        self.total_selected_cost
    }

    pub fn truncated_by_goal_limit(&self) -> bool {
        self.truncated_by_goal_limit
    }

    pub fn truncated_by_compute_budget(&self) -> bool {
        self.truncated_by_compute_budget
    }

    pub fn was_truncated(&self) -> bool {
        self.truncated_by_goal_limit || self.truncated_by_compute_budget
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CollisionSafeInformationGoalEngine;

impl CollisionSafeInformationGoalEngine {
    fn primary_goal(
        record: &CollisionSafeEpistemicRecord,
        assessment: EpistemicSelfAssessment,
        policy: SelfGeneratedGoalPolicy,
    ) -> Option<(SelfGeneratedGoalKind, CognitiveSignal, u32)> {
        let (kind, priority) = if assessment.is_stable() && assessment.is_compressible() {
            (
                SelfGeneratedGoalKind::CompressRepresentation,
                record.compression_gain(),
            )
        } else if !assessment.is_stable() && assessment.is_controllable() {
            (SelfGeneratedGoalKind::TestControl, record.controllability())
        } else if assessment.is_learning() {
            (
                SelfGeneratedGoalKind::ContinueLearning,
                record.learning_progress(),
            )
        } else if assessment.is_uncertain() {
            (
                SelfGeneratedGoalKind::ResolveUncertainty,
                record.uncertainty(),
            )
        } else {
            return None;
        };

        Some((kind, priority, policy.cost_for(kind)))
    }

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
        left: &CollisionSafeRankedGoal,
        right: &CollisionSafeRankedGoal,
    ) -> std::cmp::Ordering {
        right
            .information_gain()
            .cmp(&left.information_gain())
            .then_with(|| right.base_priority().cmp(&left.base_priority()))
            .then_with(|| left.estimated_cost().cmp(&right.estimated_cost()))
            .then_with(|| left.kind().cmp(&right.kind()))
            .then_with(|| left.fingerprint().cmp(&right.fingerprint()))
            .then_with(|| Self::compare_structure(left.structure(), right.structure()))
    }

    pub fn rank(
        state: &CollisionSafeEpistemicState,
        epistemic_policy: EpistemicSelfPolicy,
        goal_policy: SelfGeneratedGoalPolicy,
        predictions: &CollisionSafeGoalInformationPredictionSet,
        available_compute_units: u32,
    ) -> CollisionSafeGoalFrontierResult {
        let mut candidates = state
            .all_records()
            .filter_map(|record| {
                let assessment = record.assessment(epistemic_policy);

                let (kind, base_priority, estimated_cost) =
                    Self::primary_goal(record, assessment, goal_policy)?;

                let (information_gain, expected_uncertainty) =
                    if let Some(outcomes) = predictions.outcomes_for(record.identity()) {
                        let estimate =
                            ExpectedInformationGain::estimate(record.uncertainty(), outcomes)
                                .expect("validated exact prediction has outcomes");

                        (
                            estimate.information_gain(),
                            Some(estimate.expected_uncertainty()),
                        )
                    } else {
                        (CognitiveSignal::zero(), None)
                    };

                Some(CollisionSafeRankedGoal {
                    identity: record.identity().clone(),
                    kind,
                    base_priority,
                    information_gain,
                    expected_uncertainty,
                    estimated_cost,
                })
            })
            .collect::<Vec<_>>();

        candidates.sort_by(Self::ranking);

        let candidate_count = candidates.len();

        let mut selected = Vec::with_capacity(goal_policy.max_goals().min(candidate_count));

        let mut total_selected_cost = 0_u32;

        let mut truncated_by_goal_limit = false;

        let mut truncated_by_compute_budget = false;

        for (index, candidate) in candidates.into_iter().enumerate() {
            if selected.len() >= goal_policy.max_goals() {
                truncated_by_goal_limit = index < candidate_count;

                break;
            }

            let Some(next_total) = total_selected_cost.checked_add(candidate.estimated_cost())
            else {
                truncated_by_compute_budget = true;

                break;
            };

            if next_total > available_compute_units {
                truncated_by_compute_budget = true;

                break;
            }

            total_selected_cost = next_total;

            selected.push(candidate);
        }

        CollisionSafeGoalFrontierResult {
            candidate_count,
            selected,
            total_selected_cost,
            truncated_by_goal_limit,
            truncated_by_compute_budget,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedSparseCycleState {
    structural: CollisionSafeStructuralState,
    epistemic: CollisionSafeEpistemicState,
}

impl IntegratedSparseCycleState {
    pub fn new(structural_capacity: usize, epistemic_capacity: usize) -> Option<Self> {
        Some(Self {
            structural: CollisionSafeStructuralState::new(structural_capacity)?,
            epistemic: CollisionSafeEpistemicState::new(epistemic_capacity)?,
        })
    }

    pub fn structural(&self) -> &CollisionSafeStructuralState {
        &self.structural
    }

    pub fn epistemic(&self) -> &CollisionSafeEpistemicState {
        &self.epistemic
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedSparseCycleContext {
    predictions: CollisionSafeGoalInformationPredictionSet,
    sparse_policy: SparseCognitionPolicy,
    epistemic_policy: EpistemicSelfPolicy,
    goal_policy: SelfGeneratedGoalPolicy,
    budget: CognitiveBudget,
}

impl IntegratedSparseCycleContext {
    pub fn new(
        predictions: CollisionSafeGoalInformationPredictionSet,
        sparse_policy: SparseCognitionPolicy,
        epistemic_policy: EpistemicSelfPolicy,
        goal_policy: SelfGeneratedGoalPolicy,
        budget: CognitiveBudget,
    ) -> Self {
        Self {
            predictions,
            sparse_policy,
            epistemic_policy,
            goal_policy,
            budget,
        }
    }

    pub fn predictions(&self) -> &CollisionSafeGoalInformationPredictionSet {
        &self.predictions
    }

    pub fn sparse_policy(&self) -> SparseCognitionPolicy {
        self.sparse_policy
    }

    pub fn epistemic_policy(&self) -> EpistemicSelfPolicy {
        self.epistemic_policy
    }

    pub fn goal_policy(&self) -> SelfGeneratedGoalPolicy {
        self.goal_policy
    }

    pub fn budget(&self) -> CognitiveBudget {
        self.budget
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IntegratedSparseCycleStatus {
    RejectedOutOfOrder,
    Ignored,
    Processed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedSparseCycleResult {
    state_before: IntegratedSparseCycleState,
    state_after: IntegratedSparseCycleState,
    event_index: u64,
    identity: CollisionSafeStructuralIdentity,
    novelty: CognitiveSignal,
    current_information_gain: CognitiveSignal,
    effective_profile: Option<MindstoneExtendedSignalProfile>,
    admission: Option<CognitiveAdmissionDecision>,
    epistemic_record: Option<CollisionSafeEpistemicRecord>,
    goal_compute_units: u32,
    goals: CollisionSafeGoalFrontierResult,
    status: IntegratedSparseCycleStatus,
}

impl IntegratedSparseCycleResult {
    pub fn state_before(&self) -> &IntegratedSparseCycleState {
        &self.state_before
    }

    pub fn state_after(&self) -> &IntegratedSparseCycleState {
        &self.state_after
    }

    pub fn event_index(&self) -> u64 {
        self.event_index
    }

    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn novelty(&self) -> CognitiveSignal {
        self.novelty
    }

    pub fn current_information_gain(&self) -> CognitiveSignal {
        self.current_information_gain
    }

    pub fn effective_profile(&self) -> Option<MindstoneExtendedSignalProfile> {
        self.effective_profile
    }

    pub fn admission(&self) -> Option<CognitiveAdmissionDecision> {
        self.admission
    }

    pub fn epistemic_record(&self) -> Option<&CollisionSafeEpistemicRecord> {
        self.epistemic_record.as_ref()
    }

    pub fn goal_compute_units(&self) -> u32 {
        self.goal_compute_units
    }

    pub fn goals(&self) -> &CollisionSafeGoalFrontierResult {
        &self.goals
    }

    pub fn status(&self) -> IntegratedSparseCycleStatus {
        self.status
    }

    pub fn accepted(&self) -> bool {
        self.status != IntegratedSparseCycleStatus::RejectedOutOfOrder
    }

    pub fn processed(&self) -> bool {
        self.status == IntegratedSparseCycleStatus::Processed
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IntegratedSparseCycle;

impl IntegratedSparseCycle {
    fn evaluate_identity(
        state: IntegratedSparseCycleState,
        event_index: u64,
        identity: CollisionSafeStructuralIdentity,
        profile: MindstoneExtendedSignalProfile,
        context: &IntegratedSparseCycleContext,
    ) -> IntegratedSparseCycleResult {
        let state_before = state.clone();

        let structural_result = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
            state.structural.clone(),
            event_index,
            identity.fingerprint(),
            identity.structure().clone(),
            profile.salience(),
        );

        if !structural_result.accepted() {
            return IntegratedSparseCycleResult {
                state_before,
                state_after: state,
                event_index,
                identity,
                novelty: CognitiveSignal::zero(),
                current_information_gain: CognitiveSignal::zero(),
                effective_profile: None,
                admission: None,
                epistemic_record: None,
                goal_compute_units: 0,
                goals: CollisionSafeGoalFrontierResult::empty(),
                status: IntegratedSparseCycleStatus::RejectedOutOfOrder,
            };
        }

        let exact_identity = structural_result.identity().clone();

        let current_information_gain = context
            .predictions()
            .outcomes_for(&exact_identity)
            .and_then(|outcomes| {
                ExpectedInformationGain::estimate(profile.base().uncertainty(), outcomes)
            })
            .map_or(CognitiveSignal::zero(), |estimate| {
                estimate.information_gain()
            });

        let effective_base = profile
            .base()
            .with_integrated_novelty(structural_result.novelty())
            .with_information_gain(current_information_gain);

        let effective_profile = MindstoneExtendedSignalProfile::new(
            effective_base,
            profile.compression_gain(),
            profile.controllability(),
        );

        let admission = context
            .sparse_policy()
            .admit_extended(effective_profile, context.budget());

        let mut state_after = IntegratedSparseCycleState {
            structural: structural_result.state_after().clone(),
            epistemic: state.epistemic,
        };

        if admission.class() == CognitiveAdmissionClass::Ignore {
            return IntegratedSparseCycleResult {
                state_before,
                state_after,
                event_index,
                identity: exact_identity,
                novelty: structural_result.novelty(),
                current_information_gain,
                effective_profile: Some(effective_profile),
                admission: Some(admission),
                epistemic_record: None,
                goal_compute_units: context.budget().units(),
                goals: CollisionSafeGoalFrontierResult::empty(),
                status: IntegratedSparseCycleStatus::Ignored,
            };
        }

        let epistemic_result = CollisionSafeEpistemicSelfModel::observe(
            state_after.epistemic.clone(),
            event_index,
            exact_identity.clone(),
            effective_profile,
        );

        let epistemic_record = epistemic_result.record().cloned();

        state_after.epistemic = epistemic_result.state_after().clone();

        let goal_compute_units = context
            .budget()
            .units()
            .saturating_sub(admission.granted_units());

        let goals = CollisionSafeInformationGoalEngine::rank(
            &state_after.epistemic,
            context.epistemic_policy(),
            context.goal_policy(),
            context.predictions(),
            goal_compute_units,
        );

        IntegratedSparseCycleResult {
            state_before,
            state_after,
            event_index,
            identity: exact_identity,
            novelty: structural_result.novelty(),
            current_information_gain,
            effective_profile: Some(effective_profile),
            admission: Some(admission),
            epistemic_record,
            goal_compute_units,
            goals,
            status: IntegratedSparseCycleStatus::Processed,
        }
    }

    pub fn evaluate(
        state: IntegratedSparseCycleState,
        event_index: u64,
        structure: CognitiveStructure,
        profile: MindstoneExtendedSignalProfile,
        context: &IntegratedSparseCycleContext,
    ) -> IntegratedSparseCycleResult {
        let identity = CollisionSafeStructuralIdentity::from_structure(structure);

        Self::evaluate_identity(state, event_index, identity, profile, context)
    }

    pub fn evaluate_with_fingerprint_hint(
        state: IntegratedSparseCycleState,
        event_index: u64,
        fingerprint: CognitiveFingerprint,
        structure: CognitiveStructure,
        profile: MindstoneExtendedSignalProfile,
        context: &IntegratedSparseCycleContext,
    ) -> IntegratedSparseCycleResult {
        let identity =
            CollisionSafeStructuralIdentity::with_fingerprint_hint(fingerprint, structure);

        Self::evaluate_identity(state, event_index, identity, profile, context)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneIntegratedSparseCycle;

impl MindstoneIntegratedSparseCycle {
    pub fn evaluate(
        state: IntegratedSparseCycleState,
        event_index: u64,
        structure: CognitiveStructure,
        profile: MindstoneExtendedSignalProfile,
        context: &IntegratedSparseCycleContext,
    ) -> IntegratedSparseCycleResult {
        IntegratedSparseCycle::evaluate(state, event_index, structure, profile, context)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedHypothesisSearchNode {
    identity: CollisionSafeStructuralIdentity,
    score: CognitiveSignal,
    estimated_cost: u32,
    depth: u16,
    path_length: usize,
}

impl BoundedHypothesisSearchNode {
    pub fn new(
        identity: CollisionSafeStructuralIdentity,
        score: CognitiveSignal,
        estimated_cost: u32,
        depth: u16,
        path_length: usize,
    ) -> Option<Self> {
        if score == CognitiveSignal::zero()
            || estimated_cost == 0
            || path_length == 0
            || usize::from(depth) >= path_length
        {
            return None;
        }

        Some(Self {
            identity,
            score,
            estimated_cost,
            depth,
            path_length,
        })
    }

    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn fingerprint(&self) -> CognitiveFingerprint {
        self.identity.fingerprint()
    }

    pub fn structure(&self) -> &CognitiveStructure {
        self.identity.structure()
    }

    pub fn score(&self) -> CognitiveSignal {
        self.score
    }

    pub fn estimated_cost(&self) -> u32 {
        self.estimated_cost
    }

    pub fn depth(&self) -> u16 {
        self.depth
    }

    pub fn path_length(&self) -> usize {
        self.path_length
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundedHypothesisSearchPolicy {
    max_hypotheses: usize,
    max_path_length: usize,
    max_depth: u16,
}

impl BoundedHypothesisSearchPolicy {
    pub fn new(max_hypotheses: usize, max_path_length: usize, max_depth: u16) -> Option<Self> {
        if max_hypotheses == 0 || max_path_length == 0 {
            return None;
        }

        Some(Self {
            max_hypotheses,
            max_path_length,
            max_depth,
        })
    }

    pub fn max_hypotheses(self) -> usize {
        self.max_hypotheses
    }

    pub fn max_path_length(self) -> usize {
        self.max_path_length
    }

    pub fn max_depth(self) -> u16 {
        self.max_depth
    }

    pub fn admits(self, node: &BoundedHypothesisSearchNode) -> bool {
        node.depth() <= self.max_depth && node.path_length() <= self.max_path_length
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedHypothesisSearchResult {
    input_count: usize,
    eligible_count: usize,
    retained_frontier_count: usize,
    rejected_by_depth_count: usize,
    rejected_by_path_count: usize,
    dropped_by_frontier_count: usize,
    selected: Vec<BoundedHypothesisSearchNode>,
    total_selected_cost: u32,
    truncated_by_compute_budget: bool,
}

impl BoundedHypothesisSearchResult {
    pub fn input_count(&self) -> usize {
        self.input_count
    }

    pub fn eligible_count(&self) -> usize {
        self.eligible_count
    }

    pub fn retained_frontier_count(&self) -> usize {
        self.retained_frontier_count
    }

    pub fn rejected_by_depth_count(&self) -> usize {
        self.rejected_by_depth_count
    }

    pub fn rejected_by_path_count(&self) -> usize {
        self.rejected_by_path_count
    }

    pub fn dropped_by_frontier_count(&self) -> usize {
        self.dropped_by_frontier_count
    }

    pub fn selected(&self) -> &[BoundedHypothesisSearchNode] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn total_selected_cost(&self) -> u32 {
        self.total_selected_cost
    }

    pub fn truncated_by_compute_budget(&self) -> bool {
        self.truncated_by_compute_budget
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundedHypothesisPathDepthSearch;

impl BoundedHypothesisPathDepthSearch {
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
        left: &BoundedHypothesisSearchNode,
        right: &BoundedHypothesisSearchNode,
    ) -> std::cmp::Ordering {
        right
            .score()
            .cmp(&left.score())
            .then_with(|| left.depth().cmp(&right.depth()))
            .then_with(|| left.path_length().cmp(&right.path_length()))
            .then_with(|| left.estimated_cost().cmp(&right.estimated_cost()))
            .then_with(|| left.fingerprint().cmp(&right.fingerprint()))
            .then_with(|| Self::compare_structure(left.structure(), right.structure()))
    }

    fn exact_identity_position(
        frontier: &[BoundedHypothesisSearchNode],
        candidate: &BoundedHypothesisSearchNode,
    ) -> Option<usize> {
        frontier.iter().position(|existing| {
            existing
                .identity()
                .semantically_matches(candidate.identity())
        })
    }

    pub fn search(
        candidates: &[BoundedHypothesisSearchNode],
        policy: BoundedHypothesisSearchPolicy,
        budget: CognitiveBudget,
    ) -> BoundedHypothesisSearchResult {
        let input_count = candidates.len();

        let mut eligible_count = 0_usize;

        let mut rejected_by_depth_count = 0_usize;

        let mut rejected_by_path_count = 0_usize;

        let mut dropped_by_frontier_count = 0_usize;

        let mut frontier: Vec<BoundedHypothesisSearchNode> =
            Vec::with_capacity(policy.max_hypotheses());

        for candidate in candidates {
            if candidate.depth() > policy.max_depth() {
                rejected_by_depth_count = rejected_by_depth_count.saturating_add(1);

                continue;
            }

            if candidate.path_length() > policy.max_path_length() {
                rejected_by_path_count = rejected_by_path_count.saturating_add(1);

                continue;
            }

            eligible_count = eligible_count.saturating_add(1);

            if let Some(position) = Self::exact_identity_position(&frontier, candidate) {
                if Self::ranking(candidate, &frontier[position]) == std::cmp::Ordering::Less {
                    frontier[position] = candidate.clone();
                }
            } else {
                frontier.push(candidate.clone());
            }

            frontier.sort_by(Self::ranking);

            if frontier.len() > policy.max_hypotheses() {
                frontier.pop();

                dropped_by_frontier_count = dropped_by_frontier_count.saturating_add(1);
            }
        }

        let retained_frontier_count = frontier.len();

        let mut selected = Vec::with_capacity(retained_frontier_count);

        let mut total_selected_cost = 0_u32;

        let mut truncated_by_compute_budget = false;

        for candidate in frontier {
            let Some(next_total) = total_selected_cost.checked_add(candidate.estimated_cost())
            else {
                truncated_by_compute_budget = true;

                break;
            };

            if next_total > budget.units() {
                truncated_by_compute_budget = true;

                break;
            }

            total_selected_cost = next_total;

            selected.push(candidate);
        }

        BoundedHypothesisSearchResult {
            input_count,
            eligible_count,
            retained_frontier_count,
            rejected_by_depth_count,
            rejected_by_path_count,
            dropped_by_frontier_count,
            selected,
            total_selected_cost,
            truncated_by_compute_budget,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneBoundedHypothesisPathDepthSearch;

impl MindstoneBoundedHypothesisPathDepthSearch {
    pub fn evaluate(
        candidates: &[BoundedHypothesisSearchNode],
        policy: BoundedHypothesisSearchPolicy,
        budget: CognitiveBudget,
    ) -> BoundedHypothesisSearchResult {
        BoundedHypothesisPathDepthSearch::search(candidates, policy, budget)
    }
}

impl MindstoneSignalProfile {
    pub fn adaptive_surprise_signal(self) -> CognitiveSignal {
        self.surprise
    }

    pub fn adaptive_information_gain_signal(self) -> CognitiveSignal {
        self.information_gain
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AdaptiveComputeSignals {
    surprise: CognitiveSignal,
    learning_progress: CognitiveSignal,
    information_gain: CognitiveSignal,
    controllability: CognitiveSignal,
}

impl AdaptiveComputeSignals {
    pub fn new(
        surprise: CognitiveSignal,
        learning_progress: CognitiveSignal,
        information_gain: CognitiveSignal,
        controllability: CognitiveSignal,
    ) -> Self {
        Self {
            surprise,
            learning_progress,
            information_gain,
            controllability,
        }
    }

    pub fn from_profile(profile: MindstoneExtendedSignalProfile) -> Self {
        let base = profile.base();

        Self {
            surprise: base.adaptive_surprise_signal(),
            learning_progress: base.learning_progress_signal(),
            information_gain: base.adaptive_information_gain_signal(),
            controllability: profile.controllability(),
        }
    }

    pub fn surprise(self) -> CognitiveSignal {
        self.surprise
    }

    pub fn learning_progress(self) -> CognitiveSignal {
        self.learning_progress
    }

    pub fn information_gain(self) -> CognitiveSignal {
        self.information_gain
    }

    pub fn controllability(self) -> CognitiveSignal {
        self.controllability
    }

    pub fn goal_pressure(self) -> CognitiveSignal {
        self.information_gain.max(self.controllability)
    }

    pub fn hypothesis_pressure(self) -> CognitiveSignal {
        self.surprise.max(self.learning_progress)
    }

    pub fn overall_pressure(self) -> CognitiveSignal {
        self.goal_pressure().max(self.hypothesis_pressure())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AdaptiveComputeAllocation {
    hard_cap_units: u32,
    reserved_units: u32,
    available_units: u32,
    activated_units: u32,
    goal_units: u32,
    hypothesis_units: u32,
    unused_units: u32,
    goal_pressure: CognitiveSignal,
    hypothesis_pressure: CognitiveSignal,
    overall_pressure: CognitiveSignal,
}

impl AdaptiveComputeAllocation {
    pub fn hard_cap_units(self) -> u32 {
        self.hard_cap_units
    }

    pub fn reserved_units(self) -> u32 {
        self.reserved_units
    }

    pub fn available_units(self) -> u32 {
        self.available_units
    }

    pub fn activated_units(self) -> u32 {
        self.activated_units
    }

    pub fn goal_units(self) -> u32 {
        self.goal_units
    }

    pub fn hypothesis_units(self) -> u32 {
        self.hypothesis_units
    }

    pub fn unused_units(self) -> u32 {
        self.unused_units
    }

    pub fn goal_pressure(self) -> CognitiveSignal {
        self.goal_pressure
    }

    pub fn hypothesis_pressure(self) -> CognitiveSignal {
        self.hypothesis_pressure
    }

    pub fn overall_pressure(self) -> CognitiveSignal {
        self.overall_pressure
    }

    pub fn total_accounted_units(self) -> u32 {
        self.reserved_units
            .saturating_add(self.goal_units)
            .saturating_add(self.hypothesis_units)
            .saturating_add(self.unused_units)
    }

    pub fn is_idle(self) -> bool {
        self.activated_units == 0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdaptiveComputeAllocator;

impl AdaptiveComputeAllocator {
    fn activated_units(available_units: u32, overall_pressure: CognitiveSignal) -> u32 {
        if available_units == 0 || overall_pressure == CognitiveSignal::zero() {
            return 0;
        }

        let numerator =
            u128::from(available_units).saturating_mul(u128::from(overall_pressure.value()));

        let activated = numerator.div_ceil(1000_u128);

        activated.min(u128::from(available_units)) as u32
    }

    fn split_active_units(
        activated_units: u32,
        goal_pressure: CognitiveSignal,
        hypothesis_pressure: CognitiveSignal,
    ) -> (u32, u32) {
        if activated_units == 0 {
            return (0, 0);
        }

        let goal = u128::from(goal_pressure.value());

        let hypothesis = u128::from(hypothesis_pressure.value());

        let total_pressure = goal.saturating_add(hypothesis);

        if total_pressure == 0 {
            return (0, 0);
        }

        if hypothesis == 0 {
            return (activated_units, 0);
        }

        if goal == 0 {
            return (0, activated_units);
        }

        let goal_numerator = u128::from(activated_units).saturating_mul(goal);

        let rounded_goal_units = goal_numerator.saturating_add(total_pressure / 2) / total_pressure;

        let goal_units = rounded_goal_units.min(u128::from(activated_units)) as u32;

        let hypothesis_units = activated_units.saturating_sub(goal_units);

        (goal_units, hypothesis_units)
    }

    pub fn allocate(
        signals: AdaptiveComputeSignals,
        hard_budget: CognitiveBudget,
        reserved_units: u32,
    ) -> Option<AdaptiveComputeAllocation> {
        let hard_cap_units = hard_budget.units();

        if reserved_units > hard_cap_units {
            return None;
        }

        let available_units = hard_cap_units.saturating_sub(reserved_units);

        let goal_pressure = signals.goal_pressure();

        let hypothesis_pressure = signals.hypothesis_pressure();

        let overall_pressure = signals.overall_pressure();

        let activated_units = Self::activated_units(available_units, overall_pressure);

        let (goal_units, hypothesis_units) =
            Self::split_active_units(activated_units, goal_pressure, hypothesis_pressure);

        let unused_units =
            available_units.saturating_sub(goal_units.saturating_add(hypothesis_units));

        Some(AdaptiveComputeAllocation {
            hard_cap_units,
            reserved_units,
            available_units,
            activated_units,
            goal_units,
            hypothesis_units,
            unused_units,
            goal_pressure,
            hypothesis_pressure,
            overall_pressure,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneAdaptiveComputeAllocation;

impl MindstoneAdaptiveComputeAllocation {
    pub fn evaluate(
        profile: MindstoneExtendedSignalProfile,
        hard_budget: CognitiveBudget,
        reserved_units: u32,
    ) -> Option<AdaptiveComputeAllocation> {
        AdaptiveComputeAllocator::allocate(
            AdaptiveComputeSignals::from_profile(profile),
            hard_budget,
            reserved_units,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CausalControllabilityEvidence {
    intervention_successes: u64,
    intervention_attempts: u64,
    passive_successes: u64,
    passive_attempts: u64,
}

impl CausalControllabilityEvidence {
    pub fn new(
        intervention_successes: u64,
        intervention_attempts: u64,
        passive_successes: u64,
        passive_attempts: u64,
    ) -> Option<Self> {
        if intervention_attempts == 0
            || passive_attempts == 0
            || intervention_successes > intervention_attempts
            || passive_successes > passive_attempts
        {
            return None;
        }

        Some(Self {
            intervention_successes,
            intervention_attempts,
            passive_successes,
            passive_attempts,
        })
    }

    pub fn intervention_successes(self) -> u64 {
        self.intervention_successes
    }

    pub fn intervention_attempts(self) -> u64 {
        self.intervention_attempts
    }

    pub fn passive_successes(self) -> u64 {
        self.passive_successes
    }

    pub fn passive_attempts(self) -> u64 {
        self.passive_attempts
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CausalControllabilityEstimate {
    intervention_rate: CognitiveSignal,
    passive_rate: CognitiveSignal,
    causal_lift: CognitiveSignal,
}

impl CausalControllabilityEstimate {
    pub fn intervention_rate(self) -> CognitiveSignal {
        self.intervention_rate
    }

    pub fn passive_rate(self) -> CognitiveSignal {
        self.passive_rate
    }

    pub fn causal_lift(self) -> CognitiveSignal {
        self.causal_lift
    }

    pub fn has_positive_causal_lift(self) -> bool {
        self.causal_lift > CognitiveSignal::zero()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CausalControllabilityEstimator;

impl CausalControllabilityEstimator {
    fn empirical_rate(successes: u64, attempts: u64) -> CognitiveSignal {
        let scaled = u128::from(successes).saturating_mul(1000_u128) / u128::from(attempts);

        CognitiveSignal::new(scaled as u16).expect("validated empirical success rate is bounded")
    }

    pub fn estimate(evidence: CausalControllabilityEvidence) -> CausalControllabilityEstimate {
        let intervention_rate = Self::empirical_rate(
            evidence.intervention_successes(),
            evidence.intervention_attempts(),
        );

        let passive_rate =
            Self::empirical_rate(evidence.passive_successes(), evidence.passive_attempts());

        let causal_lift = if intervention_rate > passive_rate {
            CognitiveSignal::new(
                intervention_rate
                    .value()
                    .saturating_sub(passive_rate.value()),
            )
            .expect("causal lift remains on cognitive signal scale")
        } else {
            CognitiveSignal::zero()
        };

        CausalControllabilityEstimate {
            intervention_rate,
            passive_rate,
            causal_lift,
        }
    }
}

impl CognitiveSignal {
    pub fn causal_controllability(
        intervention_successes: u64,
        intervention_attempts: u64,
        passive_successes: u64,
        passive_attempts: u64,
    ) -> Option<Self> {
        let evidence = CausalControllabilityEvidence::new(
            intervention_successes,
            intervention_attempts,
            passive_successes,
            passive_attempts,
        )?;

        Some(CausalControllabilityEstimator::estimate(evidence).causal_lift())
    }
}

impl MindstoneExtendedSignalProfile {
    pub fn with_causal_controllability(mut self, estimate: CausalControllabilityEstimate) -> Self {
        self.controllability = estimate.causal_lift();

        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MindstoneCausalControllabilityResult {
    evidence: CausalControllabilityEvidence,
    estimate: CausalControllabilityEstimate,
    profile: MindstoneExtendedSignalProfile,
}

impl MindstoneCausalControllabilityResult {
    pub fn evidence(self) -> CausalControllabilityEvidence {
        self.evidence
    }

    pub fn estimate(self) -> CausalControllabilityEstimate {
        self.estimate
    }

    pub fn profile(self) -> MindstoneExtendedSignalProfile {
        self.profile
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneCausalControllability;

impl MindstoneCausalControllability {
    pub fn evaluate(
        profile: MindstoneExtendedSignalProfile,
        evidence: CausalControllabilityEvidence,
    ) -> MindstoneCausalControllabilityResult {
        let estimate = CausalControllabilityEstimator::estimate(evidence);

        let profile = profile.with_causal_controllability(estimate);

        MindstoneCausalControllabilityResult {
            evidence,
            estimate,
            profile,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseMindstoneFinalInput {
    identity: CollisionSafeStructuralIdentity,
    profile: MindstoneExtendedSignalProfile,
    causal_evidence: Option<CausalControllabilityEvidence>,
    hypotheses: Vec<BoundedHypothesisSearchNode>,
}

impl SparseMindstoneFinalInput {
    pub fn from_structure(
        structure: CognitiveStructure,
        profile: MindstoneExtendedSignalProfile,
        causal_evidence: Option<CausalControllabilityEvidence>,
        hypotheses: Vec<BoundedHypothesisSearchNode>,
    ) -> Self {
        Self {
            identity: CollisionSafeStructuralIdentity::from_structure(structure),
            profile,
            causal_evidence,
            hypotheses,
        }
    }

    pub fn with_fingerprint_hint(
        fingerprint: CognitiveFingerprint,
        structure: CognitiveStructure,
        profile: MindstoneExtendedSignalProfile,
        causal_evidence: Option<CausalControllabilityEvidence>,
        hypotheses: Vec<BoundedHypothesisSearchNode>,
    ) -> Self {
        Self {
            identity: CollisionSafeStructuralIdentity::with_fingerprint_hint(
                fingerprint,
                structure,
            ),
            profile,
            causal_evidence,
            hypotheses,
        }
    }

    pub fn identity(&self) -> &CollisionSafeStructuralIdentity {
        &self.identity
    }

    pub fn profile(&self) -> MindstoneExtendedSignalProfile {
        self.profile
    }

    pub fn causal_evidence(&self) -> Option<CausalControllabilityEvidence> {
        self.causal_evidence
    }

    pub fn hypotheses(&self) -> &[BoundedHypothesisSearchNode] {
        &self.hypotheses
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseMindstoneFinalContext {
    cycle_context: IntegratedSparseCycleContext,
    hypothesis_policy: BoundedHypothesisSearchPolicy,
}

impl SparseMindstoneFinalContext {
    pub fn new(
        cycle_context: IntegratedSparseCycleContext,
        hypothesis_policy: BoundedHypothesisSearchPolicy,
    ) -> Self {
        Self {
            cycle_context,
            hypothesis_policy,
        }
    }

    pub fn cycle_context(&self) -> &IntegratedSparseCycleContext {
        &self.cycle_context
    }

    pub fn hypothesis_policy(&self) -> BoundedHypothesisSearchPolicy {
        self.hypothesis_policy
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseMindstoneFinalResult {
    corrected_profile: MindstoneExtendedSignalProfile,
    causal_estimate: Option<CausalControllabilityEstimate>,
    cycle: IntegratedSparseCycleResult,
    allocation: Option<AdaptiveComputeAllocation>,
    final_goals: CollisionSafeGoalFrontierResult,
    hypothesis_search: Option<BoundedHypothesisSearchResult>,
}

impl SparseMindstoneFinalResult {
    pub fn corrected_profile(&self) -> MindstoneExtendedSignalProfile {
        self.corrected_profile
    }

    pub fn causal_estimate(&self) -> Option<CausalControllabilityEstimate> {
        self.causal_estimate
    }

    pub fn cycle(&self) -> &IntegratedSparseCycleResult {
        &self.cycle
    }

    pub fn allocation(&self) -> Option<AdaptiveComputeAllocation> {
        self.allocation
    }

    pub fn final_goals(&self) -> &CollisionSafeGoalFrontierResult {
        &self.final_goals
    }

    pub fn hypothesis_search(&self) -> Option<&BoundedHypothesisSearchResult> {
        self.hypothesis_search.as_ref()
    }

    pub fn processed(&self) -> bool {
        self.cycle.processed()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SparseMindstoneFinalOrchestrator;

impl SparseMindstoneFinalOrchestrator {
    fn causal_correct(
        profile: MindstoneExtendedSignalProfile,
        evidence: Option<CausalControllabilityEvidence>,
    ) -> (
        MindstoneExtendedSignalProfile,
        Option<CausalControllabilityEstimate>,
    ) {
        if let Some(evidence) = evidence {
            let estimate = CausalControllabilityEstimator::estimate(evidence);

            (
                profile.with_causal_controllability(estimate),
                Some(estimate),
            )
        } else {
            (
                MindstoneExtendedSignalProfile::new(
                    profile.base(),
                    profile.compression_gain(),
                    CognitiveSignal::zero(),
                ),
                None,
            )
        }
    }

    pub fn evaluate(
        state: IntegratedSparseCycleState,
        event_index: u64,
        input: SparseMindstoneFinalInput,
        context: &SparseMindstoneFinalContext,
    ) -> SparseMindstoneFinalResult {
        let (corrected_profile, causal_estimate) =
            Self::causal_correct(input.profile(), input.causal_evidence());

        let cycle = IntegratedSparseCycle::evaluate_with_fingerprint_hint(
            state,
            event_index,
            input.identity().fingerprint(),
            input.identity().structure().clone(),
            corrected_profile,
            context.cycle_context(),
        );

        if !cycle.processed() {
            return SparseMindstoneFinalResult {
                corrected_profile,
                causal_estimate,
                cycle,
                allocation: None,
                final_goals: CollisionSafeGoalFrontierResult::empty(),
                hypothesis_search: None,
            };
        }

        let effective_profile = cycle
            .effective_profile()
            .expect("processed integrated sparse cycle has effective profile");

        let reserved_units = cycle
            .admission()
            .expect("processed integrated sparse cycle has admission decision")
            .granted_units();

        let allocation = MindstoneAdaptiveComputeAllocation::evaluate(
            effective_profile,
            context.cycle_context().budget(),
            reserved_units,
        )
        .expect("admission reservation cannot exceed hard cycle budget");

        let final_goals = CollisionSafeInformationGoalEngine::rank(
            cycle.state_after().epistemic(),
            context.cycle_context().epistemic_policy(),
            context.cycle_context().goal_policy(),
            context.cycle_context().predictions(),
            allocation.goal_units(),
        );

        let hypothesis_search =
            if allocation.hypothesis_units() == 0 || input.hypotheses().is_empty() {
                None
            } else {
                let hypothesis_budget = CognitiveBudget::new(allocation.hypothesis_units())
                    .expect("positive allocated hypothesis budget is valid");

                Some(BoundedHypothesisPathDepthSearch::search(
                    input.hypotheses(),
                    context.hypothesis_policy(),
                    hypothesis_budget,
                ))
            };

        SparseMindstoneFinalResult {
            corrected_profile,
            causal_estimate,
            cycle,
            allocation: Some(allocation),
            final_goals,
            hypothesis_search,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneSparseMindstoneFinal;

impl MindstoneSparseMindstoneFinal {
    pub fn evaluate(
        state: IntegratedSparseCycleState,
        event_index: u64,
        input: SparseMindstoneFinalInput,
        context: &SparseMindstoneFinalContext,
    ) -> SparseMindstoneFinalResult {
        SparseMindstoneFinalOrchestrator::evaluate(state, event_index, input, context)
    }
}

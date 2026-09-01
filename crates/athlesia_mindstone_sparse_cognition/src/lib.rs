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

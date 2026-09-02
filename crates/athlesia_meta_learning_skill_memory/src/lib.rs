use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedSkillStep {
    required_state: CognitiveStructure,
    action: CognitiveStructure,
    observed_outcome: CognitiveStructure,
    evidence_confidence: CognitiveSignal,
}

impl GroundedSkillStep {
    pub fn new(
        required_state: CognitiveStructure,
        action: CognitiveStructure,
        observed_outcome: CognitiveStructure,
        evidence_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if evidence_confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            required_state,
            action,
            observed_outcome,
            evidence_confidence,
        })
    }

    pub fn required_state(&self) -> &CognitiveStructure {
        &self.required_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn observed_outcome(&self) -> &CognitiveStructure {
        &self.observed_outcome
    }

    pub fn evidence_confidence(&self) -> CognitiveSignal {
        self.evidence_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedSkillEpisode {
    initial_state: CognitiveStructure,
    goal_identity: CognitiveStructure,
    steps: Vec<GroundedSkillStep>,
    success_confidence: CognitiveSignal,
}

impl GroundedSkillEpisode {
    pub fn new(
        initial_state: CognitiveStructure,
        goal_identity: CognitiveStructure,
        steps: Vec<GroundedSkillStep>,
        success_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if steps.is_empty() || success_confidence == CognitiveSignal::zero() {
            return None;
        }

        if steps[0].required_state() != &initial_state {
            return None;
        }

        for pair in steps.windows(2) {
            if pair[1].required_state() != pair[0].observed_outcome() {
                return None;
            }
        }

        Some(Self {
            initial_state,
            goal_identity,
            steps,
            success_confidence,
        })
    }

    pub fn initial_state(&self) -> &CognitiveStructure {
        &self.initial_state
    }

    pub fn goal_identity(&self) -> &CognitiveStructure {
        &self.goal_identity
    }

    pub fn steps(&self) -> &[GroundedSkillStep] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn success_confidence(&self) -> CognitiveSignal {
        self.success_confidence
    }

    pub fn step_confidence_floor(&self) -> CognitiveSignal {
        self.steps
            .iter()
            .map(GroundedSkillStep::evidence_confidence)
            .min_by_key(|signal| signal.value())
            .expect("validated skill episode always contains at least one step")
    }

    pub fn exact_trace(&self) -> ExactSkillTrace {
        ExactSkillTrace {
            initial_state: self.initial_state.clone(),
            goal_identity: self.goal_identity.clone(),
            steps: self
                .steps
                .iter()
                .map(|step| ExactSkillTraceStep {
                    required_state: step.required_state().clone(),
                    action: step.action().clone(),
                    observed_outcome: step.observed_outcome().clone(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactSkillTraceStep {
    required_state: CognitiveStructure,
    action: CognitiveStructure,
    observed_outcome: CognitiveStructure,
}

impl ExactSkillTraceStep {
    pub fn required_state(&self) -> &CognitiveStructure {
        &self.required_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn observed_outcome(&self) -> &CognitiveStructure {
        &self.observed_outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactSkillTrace {
    initial_state: CognitiveStructure,
    goal_identity: CognitiveStructure,
    steps: Vec<ExactSkillTraceStep>,
}

impl ExactSkillTrace {
    pub fn initial_state(&self) -> &CognitiveStructure {
        &self.initial_state
    }

    pub fn goal_identity(&self) -> &CognitiveStructure {
        &self.goal_identity
    }

    pub fn steps(&self) -> &[ExactSkillTraceStep] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillMemoryPolicy {
    max_input_episodes: usize,
    max_episode_steps: usize,
    max_episode_evaluations: usize,
    max_memory_entries: usize,
    minimum_success_confidence: CognitiveSignal,
    minimum_step_confidence: CognitiveSignal,
}

impl SkillMemoryPolicy {
    pub fn new(
        max_input_episodes: usize,
        max_episode_steps: usize,
        max_episode_evaluations: usize,
        max_memory_entries: usize,
        minimum_success_confidence: CognitiveSignal,
        minimum_step_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if max_input_episodes == 0
            || max_episode_steps == 0
            || max_episode_evaluations == 0
            || max_memory_entries == 0
            || minimum_success_confidence == CognitiveSignal::zero()
            || minimum_step_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            max_input_episodes,
            max_episode_steps,
            max_episode_evaluations,
            max_memory_entries,
            minimum_success_confidence,
            minimum_step_confidence,
        })
    }

    pub fn max_input_episodes(self) -> usize {
        self.max_input_episodes
    }

    pub fn max_episode_steps(self) -> usize {
        self.max_episode_steps
    }

    pub fn max_episode_evaluations(self) -> usize {
        self.max_episode_evaluations
    }

    pub fn max_memory_entries(self) -> usize {
        self.max_memory_entries
    }

    pub fn minimum_success_confidence(self) -> CognitiveSignal {
        self.minimum_success_confidence
    }

    pub fn minimum_step_confidence(self) -> CognitiveSignal {
        self.minimum_step_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillMemoryEntry {
    trace: ExactSkillTrace,
    support_count: usize,
    success_confidence_floor: CognitiveSignal,
    step_confidence_floor: CognitiveSignal,
}

impl SkillMemoryEntry {
    fn from_episode(episode: &GroundedSkillEpisode) -> Self {
        Self {
            trace: episode.exact_trace(),
            support_count: 1,
            success_confidence_floor: episode.success_confidence(),
            step_confidence_floor: episode.step_confidence_floor(),
        }
    }

    fn observe(&mut self, episode: &GroundedSkillEpisode) {
        self.support_count = self.support_count.saturating_add(1);

        if episode.success_confidence().value() < self.success_confidence_floor.value() {
            self.success_confidence_floor = episode.success_confidence();
        }

        let episode_step_floor = episode.step_confidence_floor();

        if episode_step_floor.value() < self.step_confidence_floor.value() {
            self.step_confidence_floor = episode_step_floor;
        }
    }

    pub fn trace(&self) -> &ExactSkillTrace {
        &self.trace
    }

    pub fn support_count(&self) -> usize {
        self.support_count
    }

    pub fn success_confidence_floor(&self) -> CognitiveSignal {
        self.success_confidence_floor
    }

    pub fn step_confidence_floor(&self) -> CognitiveSignal {
        self.step_confidence_floor
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillMemoryFoundationResult {
    input_episode_count: usize,
    considered_episode_count: usize,
    episode_frontier_truncated: bool,
    episode_evaluation_count: usize,
    episode_evaluation_truncated: bool,
    rejected_step_bound_count: usize,
    rejected_threshold_count: usize,
    admitted_episode_count: usize,
    entries_before_memory_frontier: usize,
    memory_frontier_truncated: bool,
    entries: Vec<SkillMemoryEntry>,
}

impl SkillMemoryFoundationResult {
    pub fn input_episode_count(&self) -> usize {
        self.input_episode_count
    }

    pub fn considered_episode_count(&self) -> usize {
        self.considered_episode_count
    }

    pub fn episode_frontier_truncated(&self) -> bool {
        self.episode_frontier_truncated
    }

    pub fn episode_evaluation_count(&self) -> usize {
        self.episode_evaluation_count
    }

    pub fn episode_evaluation_truncated(&self) -> bool {
        self.episode_evaluation_truncated
    }

    pub fn rejected_step_bound_count(&self) -> usize {
        self.rejected_step_bound_count
    }

    pub fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }

    pub fn admitted_episode_count(&self) -> usize {
        self.admitted_episode_count
    }

    pub fn entries_before_memory_frontier(&self) -> usize {
        self.entries_before_memory_frontier
    }

    pub fn memory_frontier_truncated(&self) -> bool {
        self.memory_frontier_truncated
    }

    pub fn entries(&self) -> &[SkillMemoryEntry] {
        &self.entries
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn abstained(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SkillMemoryFoundation;

impl SkillMemoryFoundation {
    fn exact_tiebreak(left: &ExactSkillTrace, right: &ExactSkillTrace) -> std::cmp::Ordering {
        format!("{left:?}").cmp(&format!("{right:?}"))
    }

    fn compare_episode(
        left: &GroundedSkillEpisode,
        right: &GroundedSkillEpisode,
    ) -> std::cmp::Ordering {
        right
            .success_confidence()
            .value()
            .cmp(&left.success_confidence().value())
            .then_with(|| {
                right
                    .step_confidence_floor()
                    .value()
                    .cmp(&left.step_confidence_floor().value())
            })
            .then_with(|| Self::exact_tiebreak(&left.exact_trace(), &right.exact_trace()))
    }

    fn compare_entry(left: &SkillMemoryEntry, right: &SkillMemoryEntry) -> std::cmp::Ordering {
        right
            .support_count()
            .cmp(&left.support_count())
            .then_with(|| {
                right
                    .success_confidence_floor()
                    .value()
                    .cmp(&left.success_confidence_floor().value())
            })
            .then_with(|| {
                right
                    .step_confidence_floor()
                    .value()
                    .cmp(&left.step_confidence_floor().value())
            })
            .then_with(|| Self::exact_tiebreak(left.trace(), right.trace()))
    }

    pub fn build(
        episodes: &[GroundedSkillEpisode],
        policy: SkillMemoryPolicy,
    ) -> SkillMemoryFoundationResult {
        let input_episode_count = episodes.len();

        let mut ranked = episodes.to_vec();

        ranked.sort_by(Self::compare_episode);

        ranked.truncate(policy.max_input_episodes());

        let considered_episode_count = ranked.len();

        let episode_frontier_truncated = input_episode_count > considered_episode_count;

        let mut episode_evaluation_count = 0_usize;

        let mut episode_evaluation_truncated = false;

        let mut rejected_step_bound_count = 0_usize;

        let mut rejected_threshold_count = 0_usize;

        let mut admitted_episode_count = 0_usize;

        let mut entries: Vec<SkillMemoryEntry> = Vec::new();

        for episode in ranked {
            if episode_evaluation_count >= policy.max_episode_evaluations() {
                episode_evaluation_truncated = true;

                break;
            }

            episode_evaluation_count = episode_evaluation_count.saturating_add(1);

            if episode.step_count() > policy.max_episode_steps() {
                rejected_step_bound_count = rejected_step_bound_count.saturating_add(1);

                continue;
            }

            if episode.success_confidence().value() < policy.minimum_success_confidence().value()
                || episode.step_confidence_floor().value()
                    < policy.minimum_step_confidence().value()
            {
                rejected_threshold_count = rejected_threshold_count.saturating_add(1);

                continue;
            }

            admitted_episode_count = admitted_episode_count.saturating_add(1);

            let trace = episode.exact_trace();

            if let Some(existing) = entries.iter_mut().find(|entry| entry.trace() == &trace) {
                existing.observe(&episode);
            } else {
                entries.push(SkillMemoryEntry::from_episode(&episode));
            }
        }

        entries.sort_by(Self::compare_entry);

        let entries_before_memory_frontier = entries.len();

        entries.truncate(policy.max_memory_entries());

        SkillMemoryFoundationResult {
            input_episode_count,
            considered_episode_count,
            episode_frontier_truncated,
            episode_evaluation_count,
            episode_evaluation_truncated,
            rejected_step_bound_count,
            rejected_threshold_count,
            admitted_episode_count,
            entries_before_memory_frontier,
            memory_frontier_truncated: entries_before_memory_frontier > entries.len(),
            entries,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalSkillMemoryFoundation;

impl UniversalSkillMemoryFoundation {
    pub fn evaluate(
        episodes: &[GroundedSkillEpisode],
        policy: SkillMemoryPolicy,
    ) -> SkillMemoryFoundationResult {
        SkillMemoryFoundation::build(episodes, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepeatedSkillCandidatePolicy {
    max_input_entries: usize,
    max_entry_evaluations: usize,
    max_candidate_steps: usize,
    max_candidates: usize,
    minimum_support_count: usize,
    minimum_success_confidence: CognitiveSignal,
    minimum_step_confidence: CognitiveSignal,
}

impl RepeatedSkillCandidatePolicy {
    pub fn new(
        max_input_entries: usize,
        max_entry_evaluations: usize,
        max_candidate_steps: usize,
        max_candidates: usize,
        minimum_support_count: usize,
        minimum_success_confidence: CognitiveSignal,
        minimum_step_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if max_input_entries == 0
            || max_entry_evaluations == 0
            || max_candidate_steps == 0
            || max_candidates == 0
            || minimum_support_count < 2
            || minimum_success_confidence == CognitiveSignal::zero()
            || minimum_step_confidence == CognitiveSignal::zero()
        {
            return None;
        }
        Some(Self {
            max_input_entries,
            max_entry_evaluations,
            max_candidate_steps,
            max_candidates,
            minimum_support_count,
            minimum_success_confidence,
            minimum_step_confidence,
        })
    }

    pub fn max_input_entries(self) -> usize {
        self.max_input_entries
    }
    pub fn max_entry_evaluations(self) -> usize {
        self.max_entry_evaluations
    }
    pub fn max_candidate_steps(self) -> usize {
        self.max_candidate_steps
    }
    pub fn max_candidates(self) -> usize {
        self.max_candidates
    }
    pub fn minimum_support_count(self) -> usize {
        self.minimum_support_count
    }
    pub fn minimum_success_confidence(self) -> CognitiveSignal {
        self.minimum_success_confidence
    }
    pub fn minimum_step_confidence(self) -> CognitiveSignal {
        self.minimum_step_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepeatedSkillCandidate {
    trace: ExactSkillTrace,
    support_count: usize,
    success_confidence_floor: CognitiveSignal,
    step_confidence_floor: CognitiveSignal,
}

impl RepeatedSkillCandidate {
    fn from_entry(entry: &SkillMemoryEntry) -> Self {
        Self {
            trace: entry.trace().clone(),
            support_count: entry.support_count(),
            success_confidence_floor: entry.success_confidence_floor(),
            step_confidence_floor: entry.step_confidence_floor(),
        }
    }

    pub fn trace(&self) -> &ExactSkillTrace {
        &self.trace
    }
    pub fn support_count(&self) -> usize {
        self.support_count
    }
    pub fn success_confidence_floor(&self) -> CognitiveSignal {
        self.success_confidence_floor
    }
    pub fn step_confidence_floor(&self) -> CognitiveSignal {
        self.step_confidence_floor
    }
    pub fn conservative_evidence_floor(&self) -> CognitiveSignal {
        if self.success_confidence_floor.value() <= self.step_confidence_floor.value() {
            self.success_confidence_floor
        } else {
            self.step_confidence_floor
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepeatedSkillCandidateDiscoveryResult {
    input_entry_count: usize,
    unique_entry_count: usize,
    considered_entry_count: usize,
    entry_frontier_truncated: bool,
    entry_evaluation_count: usize,
    entry_evaluation_truncated: bool,
    rejected_support_count: usize,
    rejected_step_bound_count: usize,
    rejected_threshold_count: usize,
    candidates_before_frontier: usize,
    candidate_frontier_truncated: bool,
    candidates: Vec<RepeatedSkillCandidate>,
}

impl RepeatedSkillCandidateDiscoveryResult {
    pub fn input_entry_count(&self) -> usize {
        self.input_entry_count
    }
    pub fn unique_entry_count(&self) -> usize {
        self.unique_entry_count
    }
    pub fn considered_entry_count(&self) -> usize {
        self.considered_entry_count
    }
    pub fn entry_frontier_truncated(&self) -> bool {
        self.entry_frontier_truncated
    }
    pub fn entry_evaluation_count(&self) -> usize {
        self.entry_evaluation_count
    }
    pub fn entry_evaluation_truncated(&self) -> bool {
        self.entry_evaluation_truncated
    }
    pub fn rejected_support_count(&self) -> usize {
        self.rejected_support_count
    }
    pub fn rejected_step_bound_count(&self) -> usize {
        self.rejected_step_bound_count
    }
    pub fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }
    pub fn candidates_before_frontier(&self) -> usize {
        self.candidates_before_frontier
    }
    pub fn candidate_frontier_truncated(&self) -> bool {
        self.candidate_frontier_truncated
    }
    pub fn candidates(&self) -> &[RepeatedSkillCandidate] {
        &self.candidates
    }
    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }
    pub fn abstained(&self) -> bool {
        self.candidates.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RepeatedSkillCandidateDiscovery;

impl RepeatedSkillCandidateDiscovery {
    fn key(trace: &ExactSkillTrace) -> String {
        format!("{trace:?}")
    }

    fn entry_order(a: &SkillMemoryEntry, b: &SkillMemoryEntry) -> std::cmp::Ordering {
        b.support_count()
            .cmp(&a.support_count())
            .then_with(|| {
                b.success_confidence_floor()
                    .value()
                    .cmp(&a.success_confidence_floor().value())
            })
            .then_with(|| {
                b.step_confidence_floor()
                    .value()
                    .cmp(&a.step_confidence_floor().value())
            })
            .then_with(|| Self::key(a.trace()).cmp(&Self::key(b.trace())))
    }

    fn candidate_order(
        a: &RepeatedSkillCandidate,
        b: &RepeatedSkillCandidate,
    ) -> std::cmp::Ordering {
        b.support_count()
            .cmp(&a.support_count())
            .then_with(|| {
                b.conservative_evidence_floor()
                    .value()
                    .cmp(&a.conservative_evidence_floor().value())
            })
            .then_with(|| Self::key(a.trace()).cmp(&Self::key(b.trace())))
    }

    pub fn discover(
        entries: &[SkillMemoryEntry],
        policy: RepeatedSkillCandidatePolicy,
    ) -> RepeatedSkillCandidateDiscoveryResult {
        let input_entry_count = entries.len();
        let mut ranked = entries.to_vec();

        ranked.sort_by(Self::entry_order);
        ranked.dedup_by(|a, b| a.trace() == b.trace());

        let unique_entry_count = ranked.len();
        ranked.truncate(policy.max_input_entries());
        let considered_entry_count = ranked.len();

        let mut evaluation_count = 0;
        let mut evaluation_truncated = false;
        let mut rejected_support = 0;
        let mut rejected_steps = 0;
        let mut rejected_threshold = 0;
        let mut candidates = Vec::new();

        for entry in ranked {
            if evaluation_count >= policy.max_entry_evaluations() {
                evaluation_truncated = true;
                break;
            }
            evaluation_count += 1;

            if entry.support_count() < policy.minimum_support_count() {
                rejected_support += 1;
                continue;
            }

            if entry.trace().step_count() > policy.max_candidate_steps() {
                rejected_steps += 1;
                continue;
            }

            if entry.success_confidence_floor().value()
                < policy.minimum_success_confidence().value()
                || entry.step_confidence_floor().value() < policy.minimum_step_confidence().value()
            {
                rejected_threshold += 1;
                continue;
            }

            candidates.push(RepeatedSkillCandidate::from_entry(&entry));
        }

        candidates.sort_by(Self::candidate_order);
        let before = candidates.len();
        candidates.truncate(policy.max_candidates());

        RepeatedSkillCandidateDiscoveryResult {
            input_entry_count,
            unique_entry_count,
            considered_entry_count,
            entry_frontier_truncated: unique_entry_count > considered_entry_count,
            entry_evaluation_count: evaluation_count,
            entry_evaluation_truncated: evaluation_truncated,
            rejected_support_count: rejected_support,
            rejected_step_bound_count: rejected_steps,
            rejected_threshold_count: rejected_threshold,
            candidates_before_frontier: before,
            candidate_frontier_truncated: before > candidates.len(),
            candidates,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalRepeatedSkillCandidateDiscovery;

impl UniversalRepeatedSkillCandidateDiscovery {
    pub fn evaluate(
        entries: &[SkillMemoryEntry],
        policy: RepeatedSkillCandidatePolicy,
    ) -> RepeatedSkillCandidateDiscoveryResult {
        RepeatedSkillCandidateDiscovery::discover(entries, policy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructuralSkillTerm {
    Invariant(CognitiveStructure),
    Variable(usize),
}

impl StructuralSkillTerm {
    pub fn invariant(&self) -> Option<&CognitiveStructure> {
        match self {
            Self::Invariant(value) => Some(value),
            Self::Variable(_) => None,
        }
    }

    pub fn variable_id(&self) -> Option<usize> {
        match self {
            Self::Invariant(_) => None,
            Self::Variable(id) => Some(*id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralSkillStep {
    required_state: StructuralSkillTerm,
    action: StructuralSkillTerm,
    observed_outcome: StructuralSkillTerm,
}

impl StructuralSkillStep {
    pub fn required_state(&self) -> &StructuralSkillTerm {
        &self.required_state
    }

    pub fn action(&self) -> &StructuralSkillTerm {
        &self.action
    }

    pub fn observed_outcome(&self) -> &StructuralSkillTerm {
        &self.observed_outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralSkillAbstraction {
    initial_state: StructuralSkillTerm,
    goal_identity: StructuralSkillTerm,
    steps: Vec<StructuralSkillStep>,
    variable_count: usize,
}

impl StructuralSkillAbstraction {
    pub fn initial_state(&self) -> &StructuralSkillTerm {
        &self.initial_state
    }

    pub fn goal_identity(&self) -> &StructuralSkillTerm {
        &self.goal_identity
    }

    pub fn steps(&self) -> &[StructuralSkillStep] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn variable_count(&self) -> usize {
        self.variable_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructuralSkillAbstractionPolicy {
    max_input_candidates: usize,
    max_pair_evaluations: usize,
    max_candidate_steps: usize,
    max_abstractions: usize,
    minimum_candidate_support: usize,
    minimum_success_confidence: CognitiveSignal,
    minimum_step_confidence: CognitiveSignal,
}

impl StructuralSkillAbstractionPolicy {
    pub fn new(
        max_input_candidates: usize,
        max_pair_evaluations: usize,
        max_candidate_steps: usize,
        max_abstractions: usize,
        minimum_candidate_support: usize,
        minimum_success_confidence: CognitiveSignal,
        minimum_step_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if max_input_candidates < 2
            || max_pair_evaluations == 0
            || max_candidate_steps == 0
            || max_abstractions == 0
            || minimum_candidate_support < 2
            || minimum_success_confidence == CognitiveSignal::zero()
            || minimum_step_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            max_input_candidates,
            max_pair_evaluations,
            max_candidate_steps,
            max_abstractions,
            minimum_candidate_support,
            minimum_success_confidence,
            minimum_step_confidence,
        })
    }

    pub fn max_input_candidates(self) -> usize {
        self.max_input_candidates
    }
    pub fn max_pair_evaluations(self) -> usize {
        self.max_pair_evaluations
    }
    pub fn max_candidate_steps(self) -> usize {
        self.max_candidate_steps
    }
    pub fn max_abstractions(self) -> usize {
        self.max_abstractions
    }
    pub fn minimum_candidate_support(self) -> usize {
        self.minimum_candidate_support
    }
    pub fn minimum_success_confidence(self) -> CognitiveSignal {
        self.minimum_success_confidence
    }
    pub fn minimum_step_confidence(self) -> CognitiveSignal {
        self.minimum_step_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralSkillAbstractionEvidence {
    abstraction: StructuralSkillAbstraction,
    source_pair_count: usize,
    source_support_sum: usize,
    success_confidence_floor: CognitiveSignal,
    step_confidence_floor: CognitiveSignal,
}

impl StructuralSkillAbstractionEvidence {
    fn new(
        abstraction: StructuralSkillAbstraction,
        left: &RepeatedSkillCandidate,
        right: &RepeatedSkillCandidate,
    ) -> Self {
        Self {
            abstraction,
            source_pair_count: 1,
            source_support_sum: left.support_count().saturating_add(right.support_count()),
            success_confidence_floor: Self::signal_floor(
                left.success_confidence_floor(),
                right.success_confidence_floor(),
            ),
            step_confidence_floor: Self::signal_floor(
                left.step_confidence_floor(),
                right.step_confidence_floor(),
            ),
        }
    }

    fn signal_floor(a: CognitiveSignal, b: CognitiveSignal) -> CognitiveSignal {
        if a.value() <= b.value() {
            a
        } else {
            b
        }
    }

    fn observe(&mut self, left: &RepeatedSkillCandidate, right: &RepeatedSkillCandidate) {
        self.source_pair_count = self.source_pair_count.saturating_add(1);
        self.source_support_sum = self
            .source_support_sum
            .saturating_add(left.support_count().saturating_add(right.support_count()));
        self.success_confidence_floor = Self::signal_floor(
            self.success_confidence_floor,
            Self::signal_floor(
                left.success_confidence_floor(),
                right.success_confidence_floor(),
            ),
        );
        self.step_confidence_floor = Self::signal_floor(
            self.step_confidence_floor,
            Self::signal_floor(left.step_confidence_floor(), right.step_confidence_floor()),
        );
    }

    pub fn abstraction(&self) -> &StructuralSkillAbstraction {
        &self.abstraction
    }

    pub fn source_pair_count(&self) -> usize {
        self.source_pair_count
    }

    pub fn source_support_sum(&self) -> usize {
        self.source_support_sum
    }

    pub fn success_confidence_floor(&self) -> CognitiveSignal {
        self.success_confidence_floor
    }

    pub fn step_confidence_floor(&self) -> CognitiveSignal {
        self.step_confidence_floor
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralSkillAbstractionResult {
    input_candidate_count: usize,
    unique_candidate_count: usize,
    considered_candidate_count: usize,
    candidate_frontier_truncated: bool,
    rejected_support_count: usize,
    rejected_step_bound_count: usize,
    rejected_threshold_count: usize,
    pair_evaluation_count: usize,
    pair_evaluation_truncated: bool,
    rejected_step_mismatch_count: usize,
    abstractions_before_frontier: usize,
    abstraction_frontier_truncated: bool,
    abstractions: Vec<StructuralSkillAbstractionEvidence>,
}

impl StructuralSkillAbstractionResult {
    pub fn input_candidate_count(&self) -> usize {
        self.input_candidate_count
    }
    pub fn unique_candidate_count(&self) -> usize {
        self.unique_candidate_count
    }
    pub fn considered_candidate_count(&self) -> usize {
        self.considered_candidate_count
    }
    pub fn candidate_frontier_truncated(&self) -> bool {
        self.candidate_frontier_truncated
    }
    pub fn rejected_support_count(&self) -> usize {
        self.rejected_support_count
    }
    pub fn rejected_step_bound_count(&self) -> usize {
        self.rejected_step_bound_count
    }
    pub fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }
    pub fn pair_evaluation_count(&self) -> usize {
        self.pair_evaluation_count
    }
    pub fn pair_evaluation_truncated(&self) -> bool {
        self.pair_evaluation_truncated
    }
    pub fn rejected_step_mismatch_count(&self) -> usize {
        self.rejected_step_mismatch_count
    }
    pub fn abstractions_before_frontier(&self) -> usize {
        self.abstractions_before_frontier
    }
    pub fn abstraction_frontier_truncated(&self) -> bool {
        self.abstraction_frontier_truncated
    }
    pub fn abstractions(&self) -> &[StructuralSkillAbstractionEvidence] {
        &self.abstractions
    }
    pub fn abstraction_count(&self) -> usize {
        self.abstractions.len()
    }
    pub fn abstained(&self) -> bool {
        self.abstractions.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StructuralSkillAbstractionInduction;

impl StructuralSkillAbstractionInduction {
    fn trace_key(trace: &ExactSkillTrace) -> String {
        format!("{trace:?}")
    }

    fn candidate_order(
        left: &RepeatedSkillCandidate,
        right: &RepeatedSkillCandidate,
    ) -> std::cmp::Ordering {
        right
            .support_count()
            .cmp(&left.support_count())
            .then_with(|| {
                right
                    .conservative_evidence_floor()
                    .value()
                    .cmp(&left.conservative_evidence_floor().value())
            })
            .then_with(|| Self::trace_key(left.trace()).cmp(&Self::trace_key(right.trace())))
    }

    fn variable_for(
        left: &CognitiveStructure,
        right: &CognitiveStructure,
        variables: &mut Vec<(CognitiveStructure, CognitiveStructure)>,
    ) -> StructuralSkillTerm {
        if left == right {
            return StructuralSkillTerm::Invariant(left.clone());
        }

        if let Some(index) = variables
            .iter()
            .position(|(a, b)| (a == left && b == right) || (a == right && b == left))
        {
            return StructuralSkillTerm::Variable(index);
        }

        let index = variables.len();
        variables.push((left.clone(), right.clone()));
        StructuralSkillTerm::Variable(index)
    }

    fn abstract_pair(
        left: &RepeatedSkillCandidate,
        right: &RepeatedSkillCandidate,
    ) -> StructuralSkillAbstraction {
        let mut variables = Vec::new();

        let initial_state = Self::variable_for(
            left.trace().initial_state(),
            right.trace().initial_state(),
            &mut variables,
        );

        let goal_identity = Self::variable_for(
            left.trace().goal_identity(),
            right.trace().goal_identity(),
            &mut variables,
        );

        let steps = left
            .trace()
            .steps()
            .iter()
            .zip(right.trace().steps())
            .map(|(a, b)| StructuralSkillStep {
                required_state: Self::variable_for(
                    a.required_state(),
                    b.required_state(),
                    &mut variables,
                ),
                action: Self::variable_for(a.action(), b.action(), &mut variables),
                observed_outcome: Self::variable_for(
                    a.observed_outcome(),
                    b.observed_outcome(),
                    &mut variables,
                ),
            })
            .collect();

        StructuralSkillAbstraction {
            initial_state,
            goal_identity,
            steps,
            variable_count: variables.len(),
        }
    }

    fn evidence_order(
        left: &StructuralSkillAbstractionEvidence,
        right: &StructuralSkillAbstractionEvidence,
    ) -> std::cmp::Ordering {
        right
            .source_pair_count()
            .cmp(&left.source_pair_count())
            .then_with(|| right.source_support_sum().cmp(&left.source_support_sum()))
            .then_with(|| {
                right
                    .success_confidence_floor()
                    .value()
                    .cmp(&left.success_confidence_floor().value())
            })
            .then_with(|| {
                left.abstraction()
                    .variable_count()
                    .cmp(&right.abstraction().variable_count())
            })
            .then_with(|| {
                format!("{:?}", left.abstraction()).cmp(&format!("{:?}", right.abstraction()))
            })
    }

    pub fn induce(
        candidates: &[RepeatedSkillCandidate],
        policy: StructuralSkillAbstractionPolicy,
    ) -> StructuralSkillAbstractionResult {
        let input_candidate_count = candidates.len();
        let mut ranked = candidates.to_vec();

        ranked.sort_by(Self::candidate_order);
        ranked.dedup_by(|a, b| a.trace() == b.trace());

        let unique_candidate_count = ranked.len();
        ranked.truncate(policy.max_input_candidates());
        let considered_candidate_count = ranked.len();

        let mut rejected_support_count = 0;
        let mut rejected_step_bound_count = 0;
        let mut rejected_threshold_count = 0;

        let eligible: Vec<_> = ranked
            .into_iter()
            .filter(|candidate| {
                if candidate.support_count() < policy.minimum_candidate_support() {
                    rejected_support_count += 1;
                    return false;
                }

                if candidate.trace().step_count() > policy.max_candidate_steps() {
                    rejected_step_bound_count += 1;
                    return false;
                }

                if candidate.success_confidence_floor().value()
                    < policy.minimum_success_confidence().value()
                    || candidate.step_confidence_floor().value()
                        < policy.minimum_step_confidence().value()
                {
                    rejected_threshold_count += 1;
                    return false;
                }

                true
            })
            .collect();

        let total_possible_pairs = eligible
            .len()
            .saturating_mul(eligible.len().saturating_sub(1))
            / 2;

        let mut pair_evaluation_count = 0;
        let mut rejected_step_mismatch_count = 0;
        let mut abstractions: Vec<StructuralSkillAbstractionEvidence> = Vec::new();

        'outer: for left_index in 0..eligible.len() {
            for right_index in (left_index + 1)..eligible.len() {
                if pair_evaluation_count >= policy.max_pair_evaluations() {
                    break 'outer;
                }

                pair_evaluation_count += 1;

                let left = &eligible[left_index];
                let right = &eligible[right_index];

                if left.trace().step_count() != right.trace().step_count() {
                    rejected_step_mismatch_count += 1;
                    continue;
                }

                let abstraction = Self::abstract_pair(left, right);

                if let Some(existing) = abstractions
                    .iter_mut()
                    .find(|item| item.abstraction() == &abstraction)
                {
                    existing.observe(left, right);
                } else {
                    abstractions.push(StructuralSkillAbstractionEvidence::new(
                        abstraction,
                        left,
                        right,
                    ));
                }
            }
        }

        abstractions.sort_by(Self::evidence_order);
        let abstractions_before_frontier = abstractions.len();
        abstractions.truncate(policy.max_abstractions());

        StructuralSkillAbstractionResult {
            input_candidate_count,
            unique_candidate_count,
            considered_candidate_count,
            candidate_frontier_truncated: unique_candidate_count > considered_candidate_count,
            rejected_support_count,
            rejected_step_bound_count,
            rejected_threshold_count,
            pair_evaluation_count,
            pair_evaluation_truncated: total_possible_pairs > pair_evaluation_count,
            rejected_step_mismatch_count,
            abstractions_before_frontier,
            abstraction_frontier_truncated: abstractions_before_frontier > abstractions.len(),
            abstractions,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalStructuralSkillAbstractionInduction;

impl UniversalStructuralSkillAbstractionInduction {
    pub fn evaluate(
        candidates: &[RepeatedSkillCandidate],
        policy: StructuralSkillAbstractionPolicy,
    ) -> StructuralSkillAbstractionResult {
        StructuralSkillAbstractionInduction::induce(candidates, policy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneralizedSkillTerm {
    Invariant(CognitiveStructure),
    StructuralVariable(usize),
    ContextVariable(usize),
}

impl GeneralizedSkillTerm {
    pub fn invariant(&self) -> Option<&CognitiveStructure> {
        match self {
            Self::Invariant(value) => Some(value),
            _ => None,
        }
    }

    pub fn structural_variable_id(&self) -> Option<usize> {
        match self {
            Self::StructuralVariable(id) => Some(*id),
            _ => None,
        }
    }

    pub fn context_variable_id(&self) -> Option<usize> {
        match self {
            Self::ContextVariable(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossContextSkillStep {
    required_state: GeneralizedSkillTerm,
    action: GeneralizedSkillTerm,
    observed_outcome: GeneralizedSkillTerm,
}

impl CrossContextSkillStep {
    pub fn required_state(&self) -> &GeneralizedSkillTerm {
        &self.required_state
    }

    pub fn action(&self) -> &GeneralizedSkillTerm {
        &self.action
    }

    pub fn observed_outcome(&self) -> &GeneralizedSkillTerm {
        &self.observed_outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossContextSkillSchema {
    initial_state: GeneralizedSkillTerm,
    goal_identity: GeneralizedSkillTerm,
    steps: Vec<CrossContextSkillStep>,
    structural_variable_count: usize,
    context_variable_count: usize,
}

impl CrossContextSkillSchema {
    pub fn initial_state(&self) -> &GeneralizedSkillTerm {
        &self.initial_state
    }

    pub fn goal_identity(&self) -> &GeneralizedSkillTerm {
        &self.goal_identity
    }

    pub fn steps(&self) -> &[CrossContextSkillStep] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn structural_variable_count(&self) -> usize {
        self.structural_variable_count
    }

    pub fn context_variable_count(&self) -> usize {
        self.context_variable_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrossContextSkillGeneralizationPolicy {
    max_input_abstractions: usize,
    max_pair_evaluations: usize,
    max_steps: usize,
    max_generalizations: usize,
    minimum_source_pair_count: usize,
    minimum_success_confidence: CognitiveSignal,
    minimum_step_confidence: CognitiveSignal,
}

impl CrossContextSkillGeneralizationPolicy {
    pub fn new(
        max_input_abstractions: usize,
        max_pair_evaluations: usize,
        max_steps: usize,
        max_generalizations: usize,
        minimum_source_pair_count: usize,
        minimum_success_confidence: CognitiveSignal,
        minimum_step_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if max_input_abstractions < 2
            || max_pair_evaluations == 0
            || max_steps == 0
            || max_generalizations == 0
            || minimum_source_pair_count == 0
            || minimum_success_confidence == CognitiveSignal::zero()
            || minimum_step_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            max_input_abstractions,
            max_pair_evaluations,
            max_steps,
            max_generalizations,
            minimum_source_pair_count,
            minimum_success_confidence,
            minimum_step_confidence,
        })
    }

    pub fn max_input_abstractions(self) -> usize {
        self.max_input_abstractions
    }

    pub fn max_pair_evaluations(self) -> usize {
        self.max_pair_evaluations
    }

    pub fn max_steps(self) -> usize {
        self.max_steps
    }

    pub fn max_generalizations(self) -> usize {
        self.max_generalizations
    }

    pub fn minimum_source_pair_count(self) -> usize {
        self.minimum_source_pair_count
    }

    pub fn minimum_success_confidence(self) -> CognitiveSignal {
        self.minimum_success_confidence
    }

    pub fn minimum_step_confidence(self) -> CognitiveSignal {
        self.minimum_step_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossContextSkillGeneralizationEvidence {
    schema: CrossContextSkillSchema,
    source_abstraction_pair_count: usize,
    source_support_sum: usize,
    success_confidence_floor: CognitiveSignal,
    step_confidence_floor: CognitiveSignal,
}

impl CrossContextSkillGeneralizationEvidence {
    fn floor(a: CognitiveSignal, b: CognitiveSignal) -> CognitiveSignal {
        if a.value() <= b.value() {
            a
        } else {
            b
        }
    }

    fn new(
        schema: CrossContextSkillSchema,
        left: &StructuralSkillAbstractionEvidence,
        right: &StructuralSkillAbstractionEvidence,
    ) -> Self {
        Self {
            schema,
            source_abstraction_pair_count: 1,
            source_support_sum: left
                .source_support_sum()
                .saturating_add(right.source_support_sum()),
            success_confidence_floor: Self::floor(
                left.success_confidence_floor(),
                right.success_confidence_floor(),
            ),
            step_confidence_floor: Self::floor(
                left.step_confidence_floor(),
                right.step_confidence_floor(),
            ),
        }
    }

    fn observe(
        &mut self,
        left: &StructuralSkillAbstractionEvidence,
        right: &StructuralSkillAbstractionEvidence,
    ) {
        self.source_abstraction_pair_count = self.source_abstraction_pair_count.saturating_add(1);

        self.source_support_sum = self.source_support_sum.saturating_add(
            left.source_support_sum()
                .saturating_add(right.source_support_sum()),
        );

        self.success_confidence_floor = Self::floor(
            self.success_confidence_floor,
            Self::floor(
                left.success_confidence_floor(),
                right.success_confidence_floor(),
            ),
        );

        self.step_confidence_floor = Self::floor(
            self.step_confidence_floor,
            Self::floor(left.step_confidence_floor(), right.step_confidence_floor()),
        );
    }

    pub fn schema(&self) -> &CrossContextSkillSchema {
        &self.schema
    }

    pub fn source_abstraction_pair_count(&self) -> usize {
        self.source_abstraction_pair_count
    }

    pub fn source_support_sum(&self) -> usize {
        self.source_support_sum
    }

    pub fn success_confidence_floor(&self) -> CognitiveSignal {
        self.success_confidence_floor
    }

    pub fn step_confidence_floor(&self) -> CognitiveSignal {
        self.step_confidence_floor
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossContextSkillGeneralizationResult {
    input_abstraction_count: usize,
    unique_abstraction_count: usize,
    considered_abstraction_count: usize,
    abstraction_frontier_truncated: bool,
    rejected_support_count: usize,
    rejected_step_bound_count: usize,
    rejected_threshold_count: usize,
    pair_evaluation_count: usize,
    pair_evaluation_truncated: bool,
    incompatible_structure_count: usize,
    generalizations_before_frontier: usize,
    generalization_frontier_truncated: bool,
    generalizations: Vec<CrossContextSkillGeneralizationEvidence>,
}

impl CrossContextSkillGeneralizationResult {
    pub fn input_abstraction_count(&self) -> usize {
        self.input_abstraction_count
    }

    pub fn unique_abstraction_count(&self) -> usize {
        self.unique_abstraction_count
    }

    pub fn considered_abstraction_count(&self) -> usize {
        self.considered_abstraction_count
    }

    pub fn abstraction_frontier_truncated(&self) -> bool {
        self.abstraction_frontier_truncated
    }

    pub fn rejected_support_count(&self) -> usize {
        self.rejected_support_count
    }

    pub fn rejected_step_bound_count(&self) -> usize {
        self.rejected_step_bound_count
    }

    pub fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }

    pub fn pair_evaluation_count(&self) -> usize {
        self.pair_evaluation_count
    }

    pub fn pair_evaluation_truncated(&self) -> bool {
        self.pair_evaluation_truncated
    }

    pub fn incompatible_structure_count(&self) -> usize {
        self.incompatible_structure_count
    }

    pub fn generalizations_before_frontier(&self) -> usize {
        self.generalizations_before_frontier
    }

    pub fn generalization_frontier_truncated(&self) -> bool {
        self.generalization_frontier_truncated
    }

    pub fn generalizations(&self) -> &[CrossContextSkillGeneralizationEvidence] {
        &self.generalizations
    }

    pub fn generalization_count(&self) -> usize {
        self.generalizations.len()
    }

    pub fn abstained(&self) -> bool {
        self.generalizations.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CrossContextSkillGeneralization;

impl CrossContextSkillGeneralization {
    fn evidence_order(
        a: &StructuralSkillAbstractionEvidence,
        b: &StructuralSkillAbstractionEvidence,
    ) -> std::cmp::Ordering {
        b.source_pair_count()
            .cmp(&a.source_pair_count())
            .then_with(|| b.source_support_sum().cmp(&a.source_support_sum()))
            .then_with(|| {
                b.success_confidence_floor()
                    .value()
                    .cmp(&a.success_confidence_floor().value())
            })
            .then_with(|| format!("{:?}", a.abstraction()).cmp(&format!("{:?}", b.abstraction())))
    }

    fn canonical(id: usize, map: &mut Vec<(usize, usize)>) -> usize {
        if let Some((_, canonical)) = map.iter().find(|(source, _)| *source == id) {
            return *canonical;
        }

        let canonical = map.len();
        map.push((id, canonical));
        canonical
    }

    fn term(
        left: &StructuralSkillTerm,
        right: &StructuralSkillTerm,
        left_map: &mut Vec<(usize, usize)>,
        right_map: &mut Vec<(usize, usize)>,
        contexts: &mut Vec<(CognitiveStructure, CognitiveStructure)>,
    ) -> Option<GeneralizedSkillTerm> {
        match (left, right) {
            (StructuralSkillTerm::Invariant(a), StructuralSkillTerm::Invariant(b)) => {
                if a == b {
                    Some(GeneralizedSkillTerm::Invariant(a.clone()))
                } else {
                    let id = if let Some(index) = contexts
                        .iter()
                        .position(|(x, y)| (x == a && y == b) || (x == b && y == a))
                    {
                        index
                    } else {
                        let index = contexts.len();
                        contexts.push((a.clone(), b.clone()));
                        index
                    };

                    Some(GeneralizedSkillTerm::ContextVariable(id))
                }
            }

            (StructuralSkillTerm::Variable(a), StructuralSkillTerm::Variable(b)) => {
                let left_id = Self::canonical(*a, left_map);

                let right_id = Self::canonical(*b, right_map);

                if left_id == right_id {
                    Some(GeneralizedSkillTerm::StructuralVariable(left_id))
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    fn generalize_pair(
        left: &StructuralSkillAbstraction,
        right: &StructuralSkillAbstraction,
    ) -> Option<CrossContextSkillSchema> {
        if left.step_count() != right.step_count() {
            return None;
        }

        let mut left_map = Vec::new();
        let mut right_map = Vec::new();
        let mut contexts = Vec::new();

        let initial_state = Self::term(
            left.initial_state(),
            right.initial_state(),
            &mut left_map,
            &mut right_map,
            &mut contexts,
        )?;

        let goal_identity = Self::term(
            left.goal_identity(),
            right.goal_identity(),
            &mut left_map,
            &mut right_map,
            &mut contexts,
        )?;

        let mut steps = Vec::new();

        for (a, b) in left.steps().iter().zip(right.steps()) {
            steps.push(CrossContextSkillStep {
                required_state: Self::term(
                    a.required_state(),
                    b.required_state(),
                    &mut left_map,
                    &mut right_map,
                    &mut contexts,
                )?,
                action: Self::term(
                    a.action(),
                    b.action(),
                    &mut left_map,
                    &mut right_map,
                    &mut contexts,
                )?,
                observed_outcome: Self::term(
                    a.observed_outcome(),
                    b.observed_outcome(),
                    &mut left_map,
                    &mut right_map,
                    &mut contexts,
                )?,
            });
        }

        Some(CrossContextSkillSchema {
            initial_state,
            goal_identity,
            steps,
            structural_variable_count: left_map.len(),
            context_variable_count: contexts.len(),
        })
    }

    fn generalized_order(
        a: &CrossContextSkillGeneralizationEvidence,
        b: &CrossContextSkillGeneralizationEvidence,
    ) -> std::cmp::Ordering {
        b.source_abstraction_pair_count()
            .cmp(&a.source_abstraction_pair_count())
            .then_with(|| b.source_support_sum().cmp(&a.source_support_sum()))
            .then_with(|| {
                b.success_confidence_floor()
                    .value()
                    .cmp(&a.success_confidence_floor().value())
            })
            .then_with(|| format!("{:?}", a.schema()).cmp(&format!("{:?}", b.schema())))
    }

    pub fn generalize(
        abstractions: &[StructuralSkillAbstractionEvidence],
        policy: CrossContextSkillGeneralizationPolicy,
    ) -> CrossContextSkillGeneralizationResult {
        let input_abstraction_count = abstractions.len();

        let mut ranked = abstractions.to_vec();

        ranked.sort_by(Self::evidence_order);

        ranked.dedup_by(|a, b| a.abstraction() == b.abstraction());

        let unique_abstraction_count = ranked.len();

        ranked.truncate(policy.max_input_abstractions());

        let considered_abstraction_count = ranked.len();

        let mut rejected_support_count = 0;
        let mut rejected_step_bound_count = 0;
        let mut rejected_threshold_count = 0;

        let eligible: Vec<_> = ranked
            .into_iter()
            .filter(|x| {
                if x.source_pair_count() < policy.minimum_source_pair_count() {
                    rejected_support_count += 1;
                    return false;
                }

                if x.abstraction().step_count() > policy.max_steps() {
                    rejected_step_bound_count += 1;
                    return false;
                }

                if x.success_confidence_floor().value()
                    < policy.minimum_success_confidence().value()
                    || x.step_confidence_floor().value() < policy.minimum_step_confidence().value()
                {
                    rejected_threshold_count += 1;
                    return false;
                }

                true
            })
            .collect();

        let possible = eligible
            .len()
            .saturating_mul(eligible.len().saturating_sub(1))
            / 2;

        let mut pair_evaluation_count = 0;
        let mut incompatible_structure_count = 0;

        let mut generalizations: Vec<CrossContextSkillGeneralizationEvidence> = Vec::new();

        'outer: for i in 0..eligible.len() {
            for j in (i + 1)..eligible.len() {
                if pair_evaluation_count >= policy.max_pair_evaluations() {
                    break 'outer;
                }

                pair_evaluation_count += 1;

                let left = &eligible[i];
                let right = &eligible[j];

                let Some(schema) = Self::generalize_pair(left.abstraction(), right.abstraction())
                else {
                    incompatible_structure_count += 1;
                    continue;
                };

                if let Some(existing) = generalizations.iter_mut().find(|x| x.schema() == &schema) {
                    existing.observe(left, right);
                } else {
                    generalizations.push(CrossContextSkillGeneralizationEvidence::new(
                        schema, left, right,
                    ));
                }
            }
        }

        generalizations.sort_by(Self::generalized_order);

        let before = generalizations.len();

        generalizations.truncate(policy.max_generalizations());

        CrossContextSkillGeneralizationResult {
            input_abstraction_count,
            unique_abstraction_count,
            considered_abstraction_count,
            abstraction_frontier_truncated: unique_abstraction_count > considered_abstraction_count,
            rejected_support_count,
            rejected_step_bound_count,
            rejected_threshold_count,
            pair_evaluation_count,
            pair_evaluation_truncated: possible > pair_evaluation_count,
            incompatible_structure_count,
            generalizations_before_frontier: before,
            generalization_frontier_truncated: before > generalizations.len(),
            generalizations,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalCrossContextSkillGeneralization;

impl UniversalCrossContextSkillGeneralization {
    pub fn evaluate(
        abstractions: &[StructuralSkillAbstractionEvidence],
        policy: CrossContextSkillGeneralizationPolicy,
    ) -> CrossContextSkillGeneralizationResult {
        CrossContextSkillGeneralization::generalize(abstractions, policy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompressedSkillTerm {
    InvariantRef(usize),
    StructuralSlot(usize),
    ContextSlot(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompressedSkillStep {
    required_state: CompressedSkillTerm,
    action: CompressedSkillTerm,
    observed_outcome: CompressedSkillTerm,
}

impl CompressedSkillStep {
    pub fn required_state(&self) -> &CompressedSkillTerm {
        &self.required_state
    }

    pub fn action(&self) -> &CompressedSkillTerm {
        &self.action
    }

    pub fn observed_outcome(&self) -> &CompressedSkillTerm {
        &self.observed_outcome
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillCompressionBounds {
    max_input_generalizations: usize,
    max_generalization_evaluations: usize,
    max_steps: usize,
    max_records: usize,
}

impl SkillCompressionBounds {
    pub fn new(
        max_input_generalizations: usize,
        max_generalization_evaluations: usize,
        max_steps: usize,
        max_records: usize,
    ) -> Option<Self> {
        if max_input_generalizations == 0
            || max_generalization_evaluations == 0
            || max_steps == 0
            || max_records == 0
        {
            return None;
        }

        Some(Self {
            max_input_generalizations,
            max_generalization_evaluations,
            max_steps,
            max_records,
        })
    }

    pub fn max_input_generalizations(self) -> usize {
        self.max_input_generalizations
    }

    pub fn max_generalization_evaluations(self) -> usize {
        self.max_generalization_evaluations
    }

    pub fn max_steps(self) -> usize {
        self.max_steps
    }

    pub fn max_records(self) -> usize {
        self.max_records
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillCompressionThresholds {
    minimum_source_pair_count: usize,
    minimum_success_confidence: CognitiveSignal,
    minimum_step_confidence: CognitiveSignal,
    minimum_compression_gain: usize,
}

impl SkillCompressionThresholds {
    pub fn new(
        minimum_source_pair_count: usize,
        minimum_success_confidence: CognitiveSignal,
        minimum_step_confidence: CognitiveSignal,
        minimum_compression_gain: usize,
    ) -> Option<Self> {
        if minimum_source_pair_count == 0
            || minimum_success_confidence == CognitiveSignal::zero()
            || minimum_step_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_source_pair_count,
            minimum_success_confidence,
            minimum_step_confidence,
            minimum_compression_gain,
        })
    }

    pub fn minimum_source_pair_count(self) -> usize {
        self.minimum_source_pair_count
    }

    pub fn minimum_success_confidence(self) -> CognitiveSignal {
        self.minimum_success_confidence
    }

    pub fn minimum_step_confidence(self) -> CognitiveSignal {
        self.minimum_step_confidence
    }

    pub fn minimum_compression_gain(self) -> usize {
        self.minimum_compression_gain
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillCompressionPolicy {
    bounds: SkillCompressionBounds,
    thresholds: SkillCompressionThresholds,
}

impl SkillCompressionPolicy {
    pub fn new(bounds: SkillCompressionBounds, thresholds: SkillCompressionThresholds) -> Self {
        Self { bounds, thresholds }
    }

    pub fn bounds(self) -> SkillCompressionBounds {
        self.bounds
    }

    pub fn thresholds(self) -> SkillCompressionThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompressedSkillRecord {
    invariant_dictionary: Vec<CognitiveStructure>,
    initial_state: CompressedSkillTerm,
    goal_identity: CompressedSkillTerm,
    steps: Vec<CompressedSkillStep>,
    structural_slot_count: usize,
    context_slot_count: usize,
    invariant_occurrence_count: usize,
    compression_gain: usize,
    source_generalization_count: usize,
    source_support_sum: usize,
    success_confidence_floor: CognitiveSignal,
    step_confidence_floor: CognitiveSignal,
}

impl CompressedSkillRecord {
    pub fn invariant_dictionary(&self) -> &[CognitiveStructure] {
        &self.invariant_dictionary
    }

    pub fn initial_state(&self) -> &CompressedSkillTerm {
        &self.initial_state
    }

    pub fn goal_identity(&self) -> &CompressedSkillTerm {
        &self.goal_identity
    }

    pub fn steps(&self) -> &[CompressedSkillStep] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn structural_slot_count(&self) -> usize {
        self.structural_slot_count
    }

    pub fn context_slot_count(&self) -> usize {
        self.context_slot_count
    }

    pub fn invariant_occurrence_count(&self) -> usize {
        self.invariant_occurrence_count
    }

    pub fn compression_gain(&self) -> usize {
        self.compression_gain
    }

    pub fn source_generalization_count(&self) -> usize {
        self.source_generalization_count
    }

    pub fn source_support_sum(&self) -> usize {
        self.source_support_sum
    }

    pub fn success_confidence_floor(&self) -> CognitiveSignal {
        self.success_confidence_floor
    }

    pub fn step_confidence_floor(&self) -> CognitiveSignal {
        self.step_confidence_floor
    }

    fn expand_term(&self, term: &CompressedSkillTerm) -> Option<GeneralizedSkillTerm> {
        match term {
            CompressedSkillTerm::InvariantRef(index) => self
                .invariant_dictionary
                .get(*index)
                .cloned()
                .map(GeneralizedSkillTerm::Invariant),
            CompressedSkillTerm::StructuralSlot(index) => {
                Some(GeneralizedSkillTerm::StructuralVariable(*index))
            }
            CompressedSkillTerm::ContextSlot(index) => {
                Some(GeneralizedSkillTerm::ContextVariable(*index))
            }
        }
    }

    pub fn semantically_matches(&self, schema: &CrossContextSkillSchema) -> bool {
        if self.step_count() != schema.step_count()
            || self.structural_slot_count != schema.structural_variable_count()
            || self.context_slot_count != schema.context_variable_count()
        {
            return false;
        }

        if self.expand_term(&self.initial_state).as_ref() != Some(schema.initial_state())
            || self.expand_term(&self.goal_identity).as_ref() != Some(schema.goal_identity())
        {
            return false;
        }

        self.steps
            .iter()
            .zip(schema.steps())
            .all(|(compressed, original)| {
                self.expand_term(compressed.required_state()).as_ref()
                    == Some(original.required_state())
                    && self.expand_term(compressed.action()).as_ref() == Some(original.action())
                    && self.expand_term(compressed.observed_outcome()).as_ref()
                        == Some(original.observed_outcome())
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillCompressionResult {
    input_generalization_count: usize,
    unique_generalization_count: usize,
    considered_generalization_count: usize,
    input_frontier_truncated: bool,
    evaluation_count: usize,
    evaluation_frontier_truncated: bool,
    rejected_support_count: usize,
    rejected_step_bound_count: usize,
    rejected_threshold_count: usize,
    rejected_gain_count: usize,
    records_before_frontier: usize,
    record_frontier_truncated: bool,
    records: Vec<CompressedSkillRecord>,
}

impl SkillCompressionResult {
    pub fn input_generalization_count(&self) -> usize {
        self.input_generalization_count
    }

    pub fn unique_generalization_count(&self) -> usize {
        self.unique_generalization_count
    }

    pub fn considered_generalization_count(&self) -> usize {
        self.considered_generalization_count
    }

    pub fn input_frontier_truncated(&self) -> bool {
        self.input_frontier_truncated
    }

    pub fn evaluation_count(&self) -> usize {
        self.evaluation_count
    }

    pub fn evaluation_frontier_truncated(&self) -> bool {
        self.evaluation_frontier_truncated
    }

    pub fn rejected_support_count(&self) -> usize {
        self.rejected_support_count
    }

    pub fn rejected_step_bound_count(&self) -> usize {
        self.rejected_step_bound_count
    }

    pub fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }

    pub fn rejected_gain_count(&self) -> usize {
        self.rejected_gain_count
    }

    pub fn records_before_frontier(&self) -> usize {
        self.records_before_frontier
    }

    pub fn record_frontier_truncated(&self) -> bool {
        self.record_frontier_truncated
    }

    pub fn records(&self) -> &[CompressedSkillRecord] {
        &self.records
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn abstained(&self) -> bool {
        self.records.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LossControlledSkillCompression;

impl LossControlledSkillCompression {
    fn evidence_order(
        a: &CrossContextSkillGeneralizationEvidence,
        b: &CrossContextSkillGeneralizationEvidence,
    ) -> std::cmp::Ordering {
        b.source_abstraction_pair_count()
            .cmp(&a.source_abstraction_pair_count())
            .then_with(|| b.source_support_sum().cmp(&a.source_support_sum()))
            .then_with(|| {
                b.success_confidence_floor()
                    .value()
                    .cmp(&a.success_confidence_floor().value())
            })
            .then_with(|| format!("{:?}", a.schema()).cmp(&format!("{:?}", b.schema())))
    }

    fn compress_term(
        term: &GeneralizedSkillTerm,
        dictionary: &mut Vec<CognitiveStructure>,
        invariant_occurrences: &mut usize,
    ) -> CompressedSkillTerm {
        match term {
            GeneralizedSkillTerm::Invariant(value) => {
                *invariant_occurrences = invariant_occurrences.saturating_add(1);

                let index =
                    if let Some(index) = dictionary.iter().position(|existing| existing == value) {
                        index
                    } else {
                        let index = dictionary.len();
                        dictionary.push(value.clone());
                        index
                    };

                CompressedSkillTerm::InvariantRef(index)
            }

            GeneralizedSkillTerm::StructuralVariable(id) => {
                CompressedSkillTerm::StructuralSlot(*id)
            }

            GeneralizedSkillTerm::ContextVariable(id) => CompressedSkillTerm::ContextSlot(*id),
        }
    }

    fn compress(evidence: &CrossContextSkillGeneralizationEvidence) -> CompressedSkillRecord {
        let schema = evidence.schema();

        let mut dictionary = Vec::new();
        let mut invariant_occurrences = 0;

        let initial_state = Self::compress_term(
            schema.initial_state(),
            &mut dictionary,
            &mut invariant_occurrences,
        );

        let goal_identity = Self::compress_term(
            schema.goal_identity(),
            &mut dictionary,
            &mut invariant_occurrences,
        );

        let steps = schema
            .steps()
            .iter()
            .map(|step| CompressedSkillStep {
                required_state: Self::compress_term(
                    step.required_state(),
                    &mut dictionary,
                    &mut invariant_occurrences,
                ),
                action: Self::compress_term(
                    step.action(),
                    &mut dictionary,
                    &mut invariant_occurrences,
                ),
                observed_outcome: Self::compress_term(
                    step.observed_outcome(),
                    &mut dictionary,
                    &mut invariant_occurrences,
                ),
            })
            .collect();

        let compression_gain = invariant_occurrences.saturating_sub(dictionary.len());

        CompressedSkillRecord {
            invariant_dictionary: dictionary,
            initial_state,
            goal_identity,
            steps,
            structural_slot_count: schema.structural_variable_count(),
            context_slot_count: schema.context_variable_count(),
            invariant_occurrence_count: invariant_occurrences,
            compression_gain,
            source_generalization_count: evidence.source_abstraction_pair_count(),
            source_support_sum: evidence.source_support_sum(),
            success_confidence_floor: evidence.success_confidence_floor(),
            step_confidence_floor: evidence.step_confidence_floor(),
        }
    }

    fn record_order(a: &CompressedSkillRecord, b: &CompressedSkillRecord) -> std::cmp::Ordering {
        b.compression_gain()
            .cmp(&a.compression_gain())
            .then_with(|| {
                b.source_generalization_count()
                    .cmp(&a.source_generalization_count())
            })
            .then_with(|| b.source_support_sum().cmp(&a.source_support_sum()))
            .then_with(|| format!("{a:?}").cmp(&format!("{b:?}")))
    }

    pub fn compress_all(
        generalizations: &[CrossContextSkillGeneralizationEvidence],
        policy: SkillCompressionPolicy,
    ) -> SkillCompressionResult {
        let bounds = policy.bounds();
        let thresholds = policy.thresholds();

        let input_generalization_count = generalizations.len();

        let mut ranked = generalizations.to_vec();

        ranked.sort_by(Self::evidence_order);

        ranked.dedup_by(|a, b| a.schema() == b.schema());

        let unique_generalization_count = ranked.len();

        ranked.truncate(bounds.max_input_generalizations());

        let considered_generalization_count = ranked.len();

        let mut evaluation_count = 0;
        let mut evaluation_frontier_truncated = false;
        let mut rejected_support_count = 0;
        let mut rejected_step_bound_count = 0;
        let mut rejected_threshold_count = 0;
        let mut rejected_gain_count = 0;
        let mut records = Vec::new();

        for evidence in ranked {
            if evaluation_count >= bounds.max_generalization_evaluations() {
                evaluation_frontier_truncated = true;
                break;
            }

            evaluation_count += 1;

            if evidence.source_abstraction_pair_count() < thresholds.minimum_source_pair_count() {
                rejected_support_count += 1;
                continue;
            }

            if evidence.schema().step_count() > bounds.max_steps() {
                rejected_step_bound_count += 1;
                continue;
            }

            if evidence.success_confidence_floor().value()
                < thresholds.minimum_success_confidence().value()
                || evidence.step_confidence_floor().value()
                    < thresholds.minimum_step_confidence().value()
            {
                rejected_threshold_count += 1;
                continue;
            }

            let record = Self::compress(&evidence);

            if !record.semantically_matches(evidence.schema()) {
                continue;
            }

            if record.compression_gain() < thresholds.minimum_compression_gain() {
                rejected_gain_count += 1;
                continue;
            }

            records.push(record);
        }

        records.sort_by(Self::record_order);

        let records_before_frontier = records.len();

        records.truncate(bounds.max_records());

        SkillCompressionResult {
            input_generalization_count,
            unique_generalization_count,
            considered_generalization_count,
            input_frontier_truncated: unique_generalization_count > considered_generalization_count,
            evaluation_count,
            evaluation_frontier_truncated,
            rejected_support_count,
            rejected_step_bound_count,
            rejected_threshold_count,
            rejected_gain_count,
            records_before_frontier,
            record_frontier_truncated: records_before_frontier > records.len(),
            records,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalLossControlledSkillCompression;

impl UniversalLossControlledSkillCompression {
    pub fn evaluate(
        generalizations: &[CrossContextSkillGeneralizationEvidence],
        policy: SkillCompressionPolicy,
    ) -> SkillCompressionResult {
        LossControlledSkillCompression::compress_all(generalizations, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillReuseSlotKind {
    Structural,
    Context,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedSkillSlotBinding {
    kind: SkillReuseSlotKind,
    slot_id: usize,
    value: CognitiveStructure,
    evidence_confidence: CognitiveSignal,
}

impl GroundedSkillSlotBinding {
    pub fn new(
        kind: SkillReuseSlotKind,
        slot_id: usize,
        value: CognitiveStructure,
        evidence_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if evidence_confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            kind,
            slot_id,
            value,
            evidence_confidence,
        })
    }

    pub fn kind(&self) -> SkillReuseSlotKind {
        self.kind
    }

    pub fn slot_id(&self) -> usize {
        self.slot_id
    }

    pub fn value(&self) -> &CognitiveStructure {
        &self.value
    }

    pub fn evidence_confidence(&self) -> CognitiveSignal {
        self.evidence_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedSkillReuseRequest {
    current_state: CognitiveStructure,
    goal_identity: CognitiveStructure,
    bindings: Vec<GroundedSkillSlotBinding>,
}

impl GroundedSkillReuseRequest {
    pub fn new(
        current_state: CognitiveStructure,
        goal_identity: CognitiveStructure,
        bindings: Vec<GroundedSkillSlotBinding>,
    ) -> Self {
        Self {
            current_state,
            goal_identity,
            bindings,
        }
    }

    pub fn current_state(&self) -> &CognitiveStructure {
        &self.current_state
    }

    pub fn goal_identity(&self) -> &CognitiveStructure {
        &self.goal_identity
    }

    pub fn bindings(&self) -> &[GroundedSkillSlotBinding] {
        &self.bindings
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillReuseBounds {
    max_input_records: usize,
    max_record_evaluations: usize,
    max_bindings: usize,
    max_steps: usize,
    max_selected_plans: usize,
}

impl SkillReuseBounds {
    pub fn new(
        max_input_records: usize,
        max_record_evaluations: usize,
        max_bindings: usize,
        max_steps: usize,
        max_selected_plans: usize,
    ) -> Option<Self> {
        if max_input_records == 0
            || max_record_evaluations == 0
            || max_bindings == 0
            || max_steps == 0
            || max_selected_plans == 0
        {
            return None;
        }

        Some(Self {
            max_input_records,
            max_record_evaluations,
            max_bindings,
            max_steps,
            max_selected_plans,
        })
    }

    pub fn max_input_records(self) -> usize {
        self.max_input_records
    }

    pub fn max_record_evaluations(self) -> usize {
        self.max_record_evaluations
    }

    pub fn max_bindings(self) -> usize {
        self.max_bindings
    }

    pub fn max_steps(self) -> usize {
        self.max_steps
    }

    pub fn max_selected_plans(self) -> usize {
        self.max_selected_plans
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillReuseThresholds {
    minimum_source_generalization_count: usize,
    minimum_source_support_sum: usize,
    minimum_success_confidence: CognitiveSignal,
    minimum_step_confidence: CognitiveSignal,
    minimum_binding_confidence: CognitiveSignal,
}

impl SkillReuseThresholds {
    pub fn new(
        minimum_source_generalization_count: usize,
        minimum_source_support_sum: usize,
        minimum_success_confidence: CognitiveSignal,
        minimum_step_confidence: CognitiveSignal,
        minimum_binding_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_source_generalization_count == 0
            || minimum_source_support_sum == 0
            || minimum_success_confidence == CognitiveSignal::zero()
            || minimum_step_confidence == CognitiveSignal::zero()
            || minimum_binding_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_source_generalization_count,
            minimum_source_support_sum,
            minimum_success_confidence,
            minimum_step_confidence,
            minimum_binding_confidence,
        })
    }

    pub fn minimum_source_generalization_count(self) -> usize {
        self.minimum_source_generalization_count
    }

    pub fn minimum_source_support_sum(self) -> usize {
        self.minimum_source_support_sum
    }

    pub fn minimum_success_confidence(self) -> CognitiveSignal {
        self.minimum_success_confidence
    }

    pub fn minimum_step_confidence(self) -> CognitiveSignal {
        self.minimum_step_confidence
    }

    pub fn minimum_binding_confidence(self) -> CognitiveSignal {
        self.minimum_binding_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillReusePolicy {
    bounds: SkillReuseBounds,
    thresholds: SkillReuseThresholds,
}

impl SkillReusePolicy {
    pub fn new(bounds: SkillReuseBounds, thresholds: SkillReuseThresholds) -> Self {
        Self { bounds, thresholds }
    }

    pub fn bounds(self) -> SkillReuseBounds {
        self.bounds
    }

    pub fn thresholds(self) -> SkillReuseThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedReusableSkillStep {
    required_state: CognitiveStructure,
    action: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
}

impl GroundedReusableSkillStep {
    pub fn required_state(&self) -> &CognitiveStructure {
        &self.required_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        &self.predicted_outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedSkillReusePlan {
    source_record: CompressedSkillRecord,
    initial_state: CognitiveStructure,
    goal_identity: CognitiveStructure,
    steps: Vec<GroundedReusableSkillStep>,
    effective_confidence_floor: CognitiveSignal,
}

impl GroundedSkillReusePlan {
    pub fn source_record(&self) -> &CompressedSkillRecord {
        &self.source_record
    }

    pub fn initial_state(&self) -> &CognitiveStructure {
        &self.initial_state
    }

    pub fn goal_identity(&self) -> &CognitiveStructure {
        &self.goal_identity
    }

    pub fn steps(&self) -> &[GroundedReusableSkillStep] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn first_step(&self) -> Option<&GroundedReusableSkillStep> {
        self.steps.first()
    }

    pub fn effective_confidence_floor(&self) -> CognitiveSignal {
        self.effective_confidence_floor
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillRetrievalReuseResult {
    input_record_count: usize,
    unique_record_count: usize,
    considered_record_count: usize,
    record_frontier_truncated: bool,
    evaluation_count: usize,
    evaluation_frontier_truncated: bool,
    binding_frontier_exceeded: bool,
    binding_conflict: bool,
    binding_threshold_failed: bool,
    rejected_support_count: usize,
    rejected_step_bound_count: usize,
    rejected_evidence_count: usize,
    rejected_anchor_mismatch_count: usize,
    rejected_unresolved_count: usize,
    rejected_continuity_count: usize,
    plans_before_frontier: usize,
    plan_frontier_truncated: bool,
    plans: Vec<GroundedSkillReusePlan>,
}

impl SkillRetrievalReuseResult {
    pub fn input_record_count(&self) -> usize {
        self.input_record_count
    }
    pub fn unique_record_count(&self) -> usize {
        self.unique_record_count
    }
    pub fn considered_record_count(&self) -> usize {
        self.considered_record_count
    }
    pub fn record_frontier_truncated(&self) -> bool {
        self.record_frontier_truncated
    }
    pub fn evaluation_count(&self) -> usize {
        self.evaluation_count
    }
    pub fn evaluation_frontier_truncated(&self) -> bool {
        self.evaluation_frontier_truncated
    }
    pub fn binding_frontier_exceeded(&self) -> bool {
        self.binding_frontier_exceeded
    }
    pub fn binding_conflict(&self) -> bool {
        self.binding_conflict
    }
    pub fn binding_threshold_failed(&self) -> bool {
        self.binding_threshold_failed
    }
    pub fn rejected_support_count(&self) -> usize {
        self.rejected_support_count
    }
    pub fn rejected_step_bound_count(&self) -> usize {
        self.rejected_step_bound_count
    }
    pub fn rejected_evidence_count(&self) -> usize {
        self.rejected_evidence_count
    }
    pub fn rejected_anchor_mismatch_count(&self) -> usize {
        self.rejected_anchor_mismatch_count
    }
    pub fn rejected_unresolved_count(&self) -> usize {
        self.rejected_unresolved_count
    }
    pub fn rejected_continuity_count(&self) -> usize {
        self.rejected_continuity_count
    }
    pub fn plans_before_frontier(&self) -> usize {
        self.plans_before_frontier
    }
    pub fn plan_frontier_truncated(&self) -> bool {
        self.plan_frontier_truncated
    }
    pub fn plans(&self) -> &[GroundedSkillReusePlan] {
        &self.plans
    }
    pub fn plan_count(&self) -> usize {
        self.plans.len()
    }
    pub fn abstained(&self) -> bool {
        self.plans.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SkillRetrievalAndReuse;

impl SkillRetrievalAndReuse {
    fn floor(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        if left.value() <= right.value() {
            left
        } else {
            right
        }
    }

    fn same_semantics(left: &CompressedSkillRecord, right: &CompressedSkillRecord) -> bool {
        left.invariant_dictionary() == right.invariant_dictionary()
            && left.initial_state() == right.initial_state()
            && left.goal_identity() == right.goal_identity()
            && left.steps() == right.steps()
            && left.structural_slot_count() == right.structural_slot_count()
            && left.context_slot_count() == right.context_slot_count()
    }

    fn record_order(
        left: &CompressedSkillRecord,
        right: &CompressedSkillRecord,
    ) -> std::cmp::Ordering {
        right
            .source_generalization_count()
            .cmp(&left.source_generalization_count())
            .then_with(|| right.source_support_sum().cmp(&left.source_support_sum()))
            .then_with(|| {
                right
                    .success_confidence_floor()
                    .value()
                    .cmp(&left.success_confidence_floor().value())
            })
            .then_with(|| {
                right
                    .step_confidence_floor()
                    .value()
                    .cmp(&left.step_confidence_floor().value())
            })
            .then_with(|| right.compression_gain().cmp(&left.compression_gain()))
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    fn insert_binding(
        map: &mut Vec<(usize, CognitiveStructure)>,
        id: usize,
        value: &CognitiveStructure,
    ) -> bool {
        if let Some((_, existing)) = map.iter().find(|(slot, _)| *slot == id) {
            return existing == value;
        }

        map.push((id, value.clone()));
        true
    }

    fn resolve(
        record: &CompressedSkillRecord,
        term: &CompressedSkillTerm,
        structural: &[(usize, CognitiveStructure)],
        context: &[(usize, CognitiveStructure)],
    ) -> Option<CognitiveStructure> {
        match term {
            CompressedSkillTerm::InvariantRef(index) => {
                record.invariant_dictionary().get(*index).cloned()
            }
            CompressedSkillTerm::StructuralSlot(id) => structural
                .iter()
                .find(|(slot, _)| slot == id)
                .map(|(_, value)| value.clone()),
            CompressedSkillTerm::ContextSlot(id) => context
                .iter()
                .find(|(slot, _)| slot == id)
                .map(|(_, value)| value.clone()),
        }
    }

    fn bind_anchor(
        record: &CompressedSkillRecord,
        term: &CompressedSkillTerm,
        value: &CognitiveStructure,
        structural: &mut Vec<(usize, CognitiveStructure)>,
        context: &mut Vec<(usize, CognitiveStructure)>,
    ) -> bool {
        match term {
            CompressedSkillTerm::InvariantRef(index) => record
                .invariant_dictionary()
                .get(*index)
                .is_some_and(|invariant| invariant == value),
            CompressedSkillTerm::StructuralSlot(id) => Self::insert_binding(structural, *id, value),
            CompressedSkillTerm::ContextSlot(id) => Self::insert_binding(context, *id, value),
        }
    }

    fn plan_order(
        left: &GroundedSkillReusePlan,
        right: &GroundedSkillReusePlan,
    ) -> std::cmp::Ordering {
        right
            .source_record()
            .source_generalization_count()
            .cmp(&left.source_record().source_generalization_count())
            .then_with(|| {
                right
                    .source_record()
                    .source_support_sum()
                    .cmp(&left.source_record().source_support_sum())
            })
            .then_with(|| {
                right
                    .effective_confidence_floor()
                    .value()
                    .cmp(&left.effective_confidence_floor().value())
            })
            .then_with(|| {
                right
                    .source_record()
                    .compression_gain()
                    .cmp(&left.source_record().compression_gain())
            })
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    pub fn retrieve(
        records: &[CompressedSkillRecord],
        request: &GroundedSkillReuseRequest,
        policy: SkillReusePolicy,
    ) -> SkillRetrievalReuseResult {
        let bounds = policy.bounds();
        let thresholds = policy.thresholds();

        let input_record_count = records.len();

        if request.bindings().len() > bounds.max_bindings() {
            return SkillRetrievalReuseResult {
                input_record_count,
                unique_record_count: 0,
                considered_record_count: 0,
                record_frontier_truncated: false,
                evaluation_count: 0,
                evaluation_frontier_truncated: false,
                binding_frontier_exceeded: true,
                binding_conflict: false,
                binding_threshold_failed: false,
                rejected_support_count: 0,
                rejected_step_bound_count: 0,
                rejected_evidence_count: 0,
                rejected_anchor_mismatch_count: 0,
                rejected_unresolved_count: 0,
                rejected_continuity_count: 0,
                plans_before_frontier: 0,
                plan_frontier_truncated: false,
                plans: Vec::new(),
            };
        }

        let mut structural = Vec::new();
        let mut context = Vec::new();
        let mut binding_floor: Option<CognitiveSignal> = None;

        for binding in request.bindings() {
            if binding.evidence_confidence().value()
                < thresholds.minimum_binding_confidence().value()
            {
                return SkillRetrievalReuseResult {
                    input_record_count,
                    unique_record_count: 0,
                    considered_record_count: 0,
                    record_frontier_truncated: false,
                    evaluation_count: 0,
                    evaluation_frontier_truncated: false,
                    binding_frontier_exceeded: false,
                    binding_conflict: false,
                    binding_threshold_failed: true,
                    rejected_support_count: 0,
                    rejected_step_bound_count: 0,
                    rejected_evidence_count: 0,
                    rejected_anchor_mismatch_count: 0,
                    rejected_unresolved_count: 0,
                    rejected_continuity_count: 0,
                    plans_before_frontier: 0,
                    plan_frontier_truncated: false,
                    plans: Vec::new(),
                };
            }

            binding_floor = Some(match binding_floor {
                Some(current) => Self::floor(current, binding.evidence_confidence()),
                None => binding.evidence_confidence(),
            });

            let target = match binding.kind() {
                SkillReuseSlotKind::Structural => &mut structural,
                SkillReuseSlotKind::Context => &mut context,
            };

            if !Self::insert_binding(target, binding.slot_id(), binding.value()) {
                return SkillRetrievalReuseResult {
                    input_record_count,
                    unique_record_count: 0,
                    considered_record_count: 0,
                    record_frontier_truncated: false,
                    evaluation_count: 0,
                    evaluation_frontier_truncated: false,
                    binding_frontier_exceeded: false,
                    binding_conflict: true,
                    binding_threshold_failed: false,
                    rejected_support_count: 0,
                    rejected_step_bound_count: 0,
                    rejected_evidence_count: 0,
                    rejected_anchor_mismatch_count: 0,
                    rejected_unresolved_count: 0,
                    rejected_continuity_count: 0,
                    plans_before_frontier: 0,
                    plan_frontier_truncated: false,
                    plans: Vec::new(),
                };
            }
        }

        let mut ranked = records.to_vec();
        ranked.sort_by(Self::record_order);
        ranked.dedup_by(|a, b| Self::same_semantics(a, b));

        let unique_record_count = ranked.len();

        ranked.truncate(bounds.max_input_records());
        let considered_record_count = ranked.len();

        let mut evaluation_count = 0;
        let mut evaluation_frontier_truncated = false;
        let mut rejected_support_count = 0;
        let mut rejected_step_bound_count = 0;
        let mut rejected_evidence_count = 0;
        let mut rejected_anchor_mismatch_count = 0;
        let mut rejected_unresolved_count = 0;
        let mut rejected_continuity_count = 0;
        let mut plans = Vec::new();

        for record in ranked {
            if evaluation_count >= bounds.max_record_evaluations() {
                evaluation_frontier_truncated = true;
                break;
            }

            evaluation_count += 1;

            if record.source_generalization_count()
                < thresholds.minimum_source_generalization_count()
                || record.source_support_sum() < thresholds.minimum_source_support_sum()
            {
                rejected_support_count += 1;
                continue;
            }

            if record.step_count() > bounds.max_steps() {
                rejected_step_bound_count += 1;
                continue;
            }

            if record.success_confidence_floor().value()
                < thresholds.minimum_success_confidence().value()
                || record.step_confidence_floor().value()
                    < thresholds.minimum_step_confidence().value()
            {
                rejected_evidence_count += 1;
                continue;
            }

            let mut local_structural = structural.clone();
            let mut local_context = context.clone();

            if !Self::bind_anchor(
                &record,
                record.initial_state(),
                request.current_state(),
                &mut local_structural,
                &mut local_context,
            ) || !Self::bind_anchor(
                &record,
                record.goal_identity(),
                request.goal_identity(),
                &mut local_structural,
                &mut local_context,
            ) {
                rejected_anchor_mismatch_count += 1;
                continue;
            }

            let Some(initial_state) = Self::resolve(
                &record,
                record.initial_state(),
                &local_structural,
                &local_context,
            ) else {
                rejected_unresolved_count += 1;
                continue;
            };

            let Some(goal_identity) = Self::resolve(
                &record,
                record.goal_identity(),
                &local_structural,
                &local_context,
            ) else {
                rejected_unresolved_count += 1;
                continue;
            };

            let mut steps = Vec::new();
            let mut unresolved = false;

            for step in record.steps() {
                let Some(required_state) = Self::resolve(
                    &record,
                    step.required_state(),
                    &local_structural,
                    &local_context,
                ) else {
                    unresolved = true;
                    break;
                };

                let Some(action) =
                    Self::resolve(&record, step.action(), &local_structural, &local_context)
                else {
                    unresolved = true;
                    break;
                };

                let Some(predicted_outcome) = Self::resolve(
                    &record,
                    step.observed_outcome(),
                    &local_structural,
                    &local_context,
                ) else {
                    unresolved = true;
                    break;
                };

                steps.push(GroundedReusableSkillStep {
                    required_state,
                    action,
                    predicted_outcome,
                });
            }

            if unresolved {
                rejected_unresolved_count += 1;
                continue;
            }

            let first_matches = steps
                .first()
                .is_some_and(|step| step.required_state() == request.current_state());

            let continuous = steps
                .windows(2)
                .all(|pair| pair[0].predicted_outcome() == pair[1].required_state());

            if initial_state != *request.current_state()
                || goal_identity != *request.goal_identity()
                || !first_matches
                || !continuous
            {
                rejected_continuity_count += 1;
                continue;
            }

            let mut confidence = Self::floor(
                record.success_confidence_floor(),
                record.step_confidence_floor(),
            );

            if let Some(binding_confidence) = binding_floor {
                confidence = Self::floor(confidence, binding_confidence);
            }

            plans.push(GroundedSkillReusePlan {
                source_record: record,
                initial_state,
                goal_identity,
                steps,
                effective_confidence_floor: confidence,
            });
        }

        plans.sort_by(Self::plan_order);

        let plans_before_frontier = plans.len();
        plans.truncate(bounds.max_selected_plans());

        SkillRetrievalReuseResult {
            input_record_count,
            unique_record_count,
            considered_record_count,
            record_frontier_truncated: unique_record_count > considered_record_count,
            evaluation_count,
            evaluation_frontier_truncated,
            binding_frontier_exceeded: false,
            binding_conflict: false,
            binding_threshold_failed: false,
            rejected_support_count,
            rejected_step_bound_count,
            rejected_evidence_count,
            rejected_anchor_mismatch_count,
            rejected_unresolved_count,
            rejected_continuity_count,
            plans_before_frontier,
            plan_frontier_truncated: plans_before_frontier > plans.len(),
            plans,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalSkillRetrievalAndReuse;

impl UniversalSkillRetrievalAndReuse {
    pub fn evaluate(
        records: &[CompressedSkillRecord],
        request: &GroundedSkillReuseRequest,
        policy: SkillReusePolicy,
    ) -> SkillRetrievalReuseResult {
        SkillRetrievalAndReuse::retrieve(records, request, policy)
    }
}

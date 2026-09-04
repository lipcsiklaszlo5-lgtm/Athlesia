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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillRevisionDisposition {
    Abstain,
    Reinforce,
    Retain,
    Weaken,
    Suspend,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillExecutionObservation {
    required_state: CognitiveStructure,
    action: CognitiveStructure,
    observed_outcome: CognitiveStructure,
    evidence_confidence: CognitiveSignal,
}

impl SkillExecutionObservation {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillOutcomeFeedbackPolicy {
    max_observations: usize,
    max_step_evaluations: usize,
    minimum_observation_confidence: CognitiveSignal,
}

impl SkillOutcomeFeedbackPolicy {
    pub fn new(
        max_observations: usize,
        max_step_evaluations: usize,
        minimum_observation_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if max_observations == 0
            || max_step_evaluations == 0
            || minimum_observation_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            max_observations,
            max_step_evaluations,
            minimum_observation_confidence,
        })
    }

    pub fn max_observations(self) -> usize {
        self.max_observations
    }

    pub fn max_step_evaluations(self) -> usize {
        self.max_step_evaluations
    }

    pub fn minimum_observation_confidence(self) -> CognitiveSignal {
        self.minimum_observation_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillOutcomeFeedbackResult {
    input_observation_count: usize,
    considered_observation_count: usize,
    observation_frontier_truncated: bool,
    evaluation_count: usize,
    evaluation_frontier_truncated: bool,
    low_confidence_count: usize,
    exact_step_count: usize,
    execution_mismatch_count: usize,
    outcome_mismatch_count: usize,
    missing_plan_step_count: usize,
    extra_observation_count: usize,
    feedback_confidence_floor: Option<CognitiveSignal>,
    disposition: SkillRevisionDisposition,
}

impl SkillOutcomeFeedbackResult {
    pub fn input_observation_count(&self) -> usize {
        self.input_observation_count
    }

    pub fn considered_observation_count(&self) -> usize {
        self.considered_observation_count
    }

    pub fn observation_frontier_truncated(&self) -> bool {
        self.observation_frontier_truncated
    }

    pub fn evaluation_count(&self) -> usize {
        self.evaluation_count
    }

    pub fn evaluation_frontier_truncated(&self) -> bool {
        self.evaluation_frontier_truncated
    }

    pub fn low_confidence_count(&self) -> usize {
        self.low_confidence_count
    }

    pub fn exact_step_count(&self) -> usize {
        self.exact_step_count
    }

    pub fn execution_mismatch_count(&self) -> usize {
        self.execution_mismatch_count
    }

    pub fn outcome_mismatch_count(&self) -> usize {
        self.outcome_mismatch_count
    }

    pub fn missing_plan_step_count(&self) -> usize {
        self.missing_plan_step_count
    }

    pub fn extra_observation_count(&self) -> usize {
        self.extra_observation_count
    }

    pub fn feedback_confidence_floor(&self) -> Option<CognitiveSignal> {
        self.feedback_confidence_floor
    }

    pub fn disposition(&self) -> SkillRevisionDisposition {
        self.disposition
    }

    pub fn abstained(&self) -> bool {
        self.disposition == SkillRevisionDisposition::Abstain
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SkillOutcomeFeedbackAndRevision;

impl SkillOutcomeFeedbackAndRevision {
    fn floor(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        if left.value() <= right.value() {
            left
        } else {
            right
        }
    }

    pub fn evaluate(
        plan: &GroundedSkillReusePlan,
        observations: &[SkillExecutionObservation],
        policy: SkillOutcomeFeedbackPolicy,
    ) -> SkillOutcomeFeedbackResult {
        let input_observation_count = observations.len();

        let considered: Vec<_> = observations
            .iter()
            .take(policy.max_observations())
            .cloned()
            .collect();

        let considered_observation_count = considered.len();

        let observation_frontier_truncated = input_observation_count > considered_observation_count;

        let low_confidence_count = considered
            .iter()
            .filter(|observation| {
                observation.evidence_confidence().value()
                    < policy.minimum_observation_confidence().value()
            })
            .count();

        let feedback_confidence_floor = considered
            .iter()
            .map(SkillExecutionObservation::evidence_confidence)
            .reduce(Self::floor);

        let missing_plan_step_count = plan
            .step_count()
            .saturating_sub(considered_observation_count);

        let extra_observation_count =
            considered_observation_count.saturating_sub(plan.step_count());

        if low_confidence_count > 0 {
            return SkillOutcomeFeedbackResult {
                input_observation_count,
                considered_observation_count,
                observation_frontier_truncated,
                evaluation_count: 0,
                evaluation_frontier_truncated: false,
                low_confidence_count,
                exact_step_count: 0,
                execution_mismatch_count: 0,
                outcome_mismatch_count: 0,
                missing_plan_step_count,
                extra_observation_count,
                feedback_confidence_floor,
                disposition: SkillRevisionDisposition::Abstain,
            };
        }

        let comparable_count = plan.step_count().min(considered_observation_count);

        let evaluation_count = comparable_count.min(policy.max_step_evaluations());

        let evaluation_frontier_truncated = comparable_count > evaluation_count;

        let mut exact_step_count = 0;
        let mut execution_mismatch_count = 0;
        let mut outcome_mismatch_count = 0;

        for (expected, actual) in plan
            .steps()
            .iter()
            .zip(considered.iter())
            .take(evaluation_count)
        {
            let state_matches = expected.required_state() == actual.required_state();

            let action_matches = expected.action() == actual.action();

            let outcome_matches = expected.predicted_outcome() == actual.observed_outcome();

            if !state_matches || !action_matches {
                execution_mismatch_count += 1;
            }

            if !outcome_matches {
                outcome_mismatch_count += 1;
            }

            if state_matches && action_matches && outcome_matches {
                exact_step_count += 1;
            }
        }

        let disposition = if observation_frontier_truncated
            || evaluation_frontier_truncated
            || evaluation_count == 0
        {
            SkillRevisionDisposition::Abstain
        } else if extra_observation_count > 0 || execution_mismatch_count > 0 {
            SkillRevisionDisposition::Suspend
        } else if outcome_mismatch_count == 0 {
            if missing_plan_step_count == 0 && considered_observation_count == plan.step_count() {
                SkillRevisionDisposition::Reinforce
            } else {
                SkillRevisionDisposition::Retain
            }
        } else if outcome_mismatch_count == evaluation_count && missing_plan_step_count == 0 {
            SkillRevisionDisposition::Suspend
        } else {
            SkillRevisionDisposition::Weaken
        };

        SkillOutcomeFeedbackResult {
            input_observation_count,
            considered_observation_count,
            observation_frontier_truncated,
            evaluation_count,
            evaluation_frontier_truncated,
            low_confidence_count,
            exact_step_count,
            execution_mismatch_count,
            outcome_mismatch_count,
            missing_plan_step_count,
            extra_observation_count,
            feedback_confidence_floor,
            disposition,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalSkillOutcomeFeedbackAndRevision;

impl UniversalSkillOutcomeFeedbackAndRevision {
    pub fn evaluate(
        plan: &GroundedSkillReusePlan,
        observations: &[SkillExecutionObservation],
        policy: SkillOutcomeFeedbackPolicy,
    ) -> SkillOutcomeFeedbackResult {
        SkillOutcomeFeedbackAndRevision::evaluate(plan, observations, policy)
    }
}

#[cfg(test)]
mod skill_outcome_feedback_revision_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        CognitiveSignal::new(value).unwrap()
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn record() -> CompressedSkillRecord {
        CompressedSkillRecord {
            invariant_dictionary: vec![a(7)],
            initial_state: CompressedSkillTerm::StructuralSlot(0),
            goal_identity: CompressedSkillTerm::InvariantRef(0),
            steps: vec![
                CompressedSkillStep {
                    required_state: CompressedSkillTerm::StructuralSlot(0),
                    action: CompressedSkillTerm::StructuralSlot(1),
                    observed_outcome: CompressedSkillTerm::StructuralSlot(2),
                },
                CompressedSkillStep {
                    required_state: CompressedSkillTerm::StructuralSlot(2),
                    action: CompressedSkillTerm::InvariantRef(0),
                    observed_outcome: CompressedSkillTerm::InvariantRef(0),
                },
            ],
            structural_slot_count: 3,
            context_slot_count: 0,
            invariant_occurrence_count: 3,
            compression_gain: 2,
            source_generalization_count: 2,
            source_support_sum: 8,
            success_confidence_floor: s(900),
            step_confidence_floor: s(900),
        }
    }

    fn plan() -> GroundedSkillReusePlan {
        GroundedSkillReusePlan {
            source_record: record(),
            initial_state: a(900),
            goal_identity: a(7),
            steps: vec![
                GroundedReusableSkillStep {
                    required_state: a(900),
                    action: a(910),
                    predicted_outcome: a(1010),
                },
                GroundedReusableSkillStep {
                    required_state: a(1010),
                    action: a(7),
                    predicted_outcome: a(7),
                },
            ],
            effective_confidence_floor: s(900),
        }
    }

    fn observation(
        required_state: u64,
        action: u64,
        outcome: u64,
        confidence: u16,
    ) -> SkillExecutionObservation {
        SkillExecutionObservation::new(a(required_state), a(action), a(outcome), s(confidence))
            .unwrap()
    }

    fn exact_observations() -> Vec<SkillExecutionObservation> {
        vec![
            observation(900, 910, 1010, 900),
            observation(1010, 7, 7, 900),
        ]
    }

    fn policy() -> SkillOutcomeFeedbackPolicy {
        SkillOutcomeFeedbackPolicy::new(16, 16, s(500)).unwrap()
    }

    #[test]
    fn feedback_policy_and_observation_require_positive_evidence() {
        assert_eq!(SkillOutcomeFeedbackPolicy::new(0, 1, s(1)), None);

        assert_eq!(SkillExecutionObservation::new(a(1), a(2), a(3), s(0)), None);

        assert!(SkillOutcomeFeedbackPolicy::new(1, 1, s(1)).is_some());
    }

    #[test]
    fn complete_exact_outcome_feedback_reinforces_skill() {
        let result =
            SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &exact_observations(), policy());

        assert_eq!(result.disposition(), SkillRevisionDisposition::Reinforce);

        assert_eq!(result.exact_step_count(), 2);

        assert_eq!(result.outcome_mismatch_count(), 0);
    }

    #[test]
    fn partial_exact_feedback_retains_without_false_reinforcement() {
        let observations = vec![observation(900, 910, 1010, 900)];

        let result = SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &observations, policy());

        assert_eq!(result.disposition(), SkillRevisionDisposition::Retain);

        assert_eq!(result.missing_plan_step_count(), 1);
    }

    #[test]
    fn isolated_outcome_prediction_error_weakens_skill() {
        let observations = vec![
            observation(900, 910, 999, 900),
            observation(1010, 7, 7, 900),
        ];

        let result = SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &observations, policy());

        assert_eq!(result.disposition(), SkillRevisionDisposition::Weaken);

        assert_eq!(result.outcome_mismatch_count(), 1);

        assert_eq!(result.execution_mismatch_count(), 0);
    }

    #[test]
    fn complete_outcome_contradiction_suspends_skill() {
        let observations = vec![
            observation(900, 910, 999, 900),
            observation(1010, 7, 998, 900),
        ];

        let result = SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &observations, policy());

        assert_eq!(result.disposition(), SkillRevisionDisposition::Suspend);

        assert_eq!(result.outcome_mismatch_count(), 2);
    }

    #[test]
    fn required_state_mismatch_suspends_reuse_assumption() {
        let observations = vec![
            observation(901, 910, 1010, 900),
            observation(1010, 7, 7, 900),
        ];

        let result = SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &observations, policy());

        assert_eq!(result.disposition(), SkillRevisionDisposition::Suspend);

        assert_eq!(result.execution_mismatch_count(), 1);
    }

    #[test]
    fn executed_action_mismatch_suspends_skill_attribution() {
        let observations = vec![
            observation(900, 911, 1010, 900),
            observation(1010, 7, 7, 900),
        ];

        let result = SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &observations, policy());

        assert_eq!(result.disposition(), SkillRevisionDisposition::Suspend);

        assert_eq!(result.execution_mismatch_count(), 1);
    }

    #[test]
    fn weak_observation_confidence_causes_abstention_before_revision() {
        let observations = vec![
            observation(900, 910, 1010, 400),
            observation(1010, 7, 7, 900),
        ];

        let result = SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &observations, policy());

        assert!(result.abstained());

        assert_eq!(result.low_confidence_count(), 1);

        assert_eq!(result.evaluation_count(), 0);
    }

    #[test]
    fn unexpected_extra_execution_step_suspends_skill() {
        let mut observations = exact_observations();

        observations.push(observation(7, 88, 99, 900));

        let result = SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &observations, policy());

        assert_eq!(result.extra_observation_count(), 1);

        assert_eq!(result.disposition(), SkillRevisionDisposition::Suspend);
    }

    #[test]
    fn hard_observation_and_evaluation_frontiers_force_abstention() {
        let observations = exact_observations();

        let observation_limited = SkillOutcomeFeedbackAndRevision::evaluate(
            &plan(),
            &observations,
            SkillOutcomeFeedbackPolicy::new(1, 16, s(500)).unwrap(),
        );

        assert!(observation_limited.observation_frontier_truncated());

        assert!(observation_limited.abstained());

        let evaluation_limited = SkillOutcomeFeedbackAndRevision::evaluate(
            &plan(),
            &observations,
            SkillOutcomeFeedbackPolicy::new(16, 1, s(500)).unwrap(),
        );

        assert!(evaluation_limited.evaluation_frontier_truncated());

        assert!(evaluation_limited.abstained());
    }

    #[test]
    fn feedback_order_is_semantically_significant() {
        let mut observations = exact_observations();

        observations.reverse();

        let result = SkillOutcomeFeedbackAndRevision::evaluate(&plan(), &observations, policy());

        assert!(result.execution_mismatch_count() > 0);

        assert_eq!(result.disposition(), SkillRevisionDisposition::Suspend);
    }

    #[test]
    fn feedback_revision_is_deterministic_non_mutating_and_facade_equivalent() {
        let plan = plan();
        let before_plan = plan.clone();

        let observations = exact_observations();

        let before_observations = observations.clone();

        let p = policy();

        let direct = SkillOutcomeFeedbackAndRevision::evaluate(&plan, &observations, p);

        let facade = UniversalSkillOutcomeFeedbackAndRevision::evaluate(&plan, &observations, p);

        let repeated = UniversalSkillOutcomeFeedbackAndRevision::evaluate(&plan, &observations, p);

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(plan, before_plan);

        assert_eq!(observations, before_observations);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillMemoryAvailability {
    Active,
    Suspended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillRevisionMemoryPolicy {
    max_revision_events: usize,
    weakening_penalty: u16,
    reinforcement_recovery: u16,
    minimum_active_confidence: CognitiveSignal,
    minimum_feedback_confidence: CognitiveSignal,
}

impl SkillRevisionMemoryPolicy {
    pub fn new(
        max_revision_events: usize,
        weakening_penalty: u16,
        reinforcement_recovery: u16,
        minimum_active_confidence: CognitiveSignal,
        minimum_feedback_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if max_revision_events == 0
            || weakening_penalty == 0
            || reinforcement_recovery == 0
            || minimum_active_confidence == CognitiveSignal::zero()
            || minimum_feedback_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            max_revision_events,
            weakening_penalty,
            reinforcement_recovery,
            minimum_active_confidence,
            minimum_feedback_confidence,
        })
    }

    pub fn max_revision_events(self) -> usize {
        self.max_revision_events
    }

    pub fn weakening_penalty(self) -> u16 {
        self.weakening_penalty
    }

    pub fn reinforcement_recovery(self) -> u16 {
        self.reinforcement_recovery
    }

    pub fn minimum_active_confidence(self) -> CognitiveSignal {
        self.minimum_active_confidence
    }

    pub fn minimum_feedback_confidence(self) -> CognitiveSignal {
        self.minimum_feedback_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillRevisionMemoryEntry {
    record: CompressedSkillRecord,
    availability: SkillMemoryAvailability,
    revision_confidence_cap: CognitiveSignal,
    applied_revision_count: usize,
    reinforcement_count: usize,
    retention_count: usize,
    weakening_count: usize,
    suspension_count: usize,
}

impl SkillRevisionMemoryEntry {
    pub fn new(record: CompressedSkillRecord) -> Self {
        let revision_confidence_cap = if record.success_confidence_floor().value()
            <= record.step_confidence_floor().value()
        {
            record.success_confidence_floor()
        } else {
            record.step_confidence_floor()
        };

        Self {
            record,
            availability: SkillMemoryAvailability::Active,
            revision_confidence_cap,
            applied_revision_count: 0,
            reinforcement_count: 0,
            retention_count: 0,
            weakening_count: 0,
            suspension_count: 0,
        }
    }

    pub fn record(&self) -> &CompressedSkillRecord {
        &self.record
    }

    pub fn availability(&self) -> SkillMemoryAvailability {
        self.availability
    }

    pub fn revision_confidence_cap(&self) -> CognitiveSignal {
        self.revision_confidence_cap
    }

    pub fn applied_revision_count(&self) -> usize {
        self.applied_revision_count
    }

    pub fn reinforcement_count(&self) -> usize {
        self.reinforcement_count
    }

    pub fn retention_count(&self) -> usize {
        self.retention_count
    }

    pub fn weakening_count(&self) -> usize {
        self.weakening_count
    }

    pub fn suspension_count(&self) -> usize {
        self.suspension_count
    }

    pub fn reusable(&self) -> bool {
        self.availability == SkillMemoryAvailability::Active
    }

    pub fn provenance_confidence_ceiling(&self) -> CognitiveSignal {
        if self.record.success_confidence_floor().value()
            <= self.record.step_confidence_floor().value()
        {
            self.record.success_confidence_floor()
        } else {
            self.record.step_confidence_floor()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillRevisionApplicationResult {
    disposition: SkillRevisionDisposition,
    revision_applied: bool,
    revision_budget_exhausted: bool,
    feedback_evidence_rejected: bool,
    availability_before: SkillMemoryAvailability,
    availability_after: SkillMemoryAvailability,
    confidence_before: CognitiveSignal,
    confidence_after: CognitiveSignal,
    memory: SkillRevisionMemoryEntry,
}

impl SkillRevisionApplicationResult {
    pub fn disposition(&self) -> SkillRevisionDisposition {
        self.disposition
    }

    pub fn revision_applied(&self) -> bool {
        self.revision_applied
    }

    pub fn revision_budget_exhausted(&self) -> bool {
        self.revision_budget_exhausted
    }

    pub fn feedback_evidence_rejected(&self) -> bool {
        self.feedback_evidence_rejected
    }

    pub fn availability_before(&self) -> SkillMemoryAvailability {
        self.availability_before
    }

    pub fn availability_after(&self) -> SkillMemoryAvailability {
        self.availability_after
    }

    pub fn confidence_before(&self) -> CognitiveSignal {
        self.confidence_before
    }

    pub fn confidence_after(&self) -> CognitiveSignal {
        self.confidence_after
    }

    pub fn memory(&self) -> &SkillRevisionMemoryEntry {
        &self.memory
    }

    pub fn memory_changed(&self) -> bool {
        self.revision_applied
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SkillRevisionApplicationAndMemoryUpdate;

impl SkillRevisionApplicationAndMemoryUpdate {
    fn signal(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn unchanged(
        entry: &SkillRevisionMemoryEntry,
        disposition: SkillRevisionDisposition,
        revision_budget_exhausted: bool,
        feedback_evidence_rejected: bool,
    ) -> SkillRevisionApplicationResult {
        SkillRevisionApplicationResult {
            disposition,
            revision_applied: false,
            revision_budget_exhausted,
            feedback_evidence_rejected,
            availability_before: entry.availability(),
            availability_after: entry.availability(),
            confidence_before: entry.revision_confidence_cap(),
            confidence_after: entry.revision_confidence_cap(),
            memory: entry.clone(),
        }
    }

    pub fn apply(
        entry: &SkillRevisionMemoryEntry,
        feedback: &SkillOutcomeFeedbackResult,
        policy: SkillRevisionMemoryPolicy,
    ) -> SkillRevisionApplicationResult {
        let disposition = feedback.disposition();

        if disposition == SkillRevisionDisposition::Abstain {
            return Self::unchanged(entry, disposition, false, false);
        }

        if entry.applied_revision_count() >= policy.max_revision_events() {
            return Self::unchanged(entry, disposition, true, false);
        }

        let Some(feedback_confidence) = feedback.feedback_confidence_floor() else {
            return Self::unchanged(entry, disposition, false, true);
        };

        if feedback_confidence.value() < policy.minimum_feedback_confidence().value() {
            return Self::unchanged(entry, disposition, false, true);
        }

        let availability_before = entry.availability();
        let confidence_before = entry.revision_confidence_cap();

        let mut memory = entry.clone();

        memory.applied_revision_count = memory.applied_revision_count.saturating_add(1);

        match disposition {
            SkillRevisionDisposition::Abstain => {}

            SkillRevisionDisposition::Reinforce => {
                memory.reinforcement_count = memory.reinforcement_count.saturating_add(1);

                let ceiling = memory.provenance_confidence_ceiling().value();

                let recovered = memory
                    .revision_confidence_cap
                    .value()
                    .saturating_add(policy.reinforcement_recovery())
                    .min(ceiling);

                memory.revision_confidence_cap = Self::signal(recovered);
            }

            SkillRevisionDisposition::Retain => {
                memory.retention_count = memory.retention_count.saturating_add(1);
            }

            SkillRevisionDisposition::Weaken => {
                memory.weakening_count = memory.weakening_count.saturating_add(1);

                let reduced = memory
                    .revision_confidence_cap
                    .value()
                    .saturating_sub(policy.weakening_penalty());

                memory.revision_confidence_cap = Self::signal(reduced);

                if reduced < policy.minimum_active_confidence().value() {
                    memory.availability = SkillMemoryAvailability::Suspended;
                }
            }

            SkillRevisionDisposition::Suspend => {
                memory.suspension_count = memory.suspension_count.saturating_add(1);

                memory.availability = SkillMemoryAvailability::Suspended;
            }
        }

        SkillRevisionApplicationResult {
            disposition,
            revision_applied: true,
            revision_budget_exhausted: false,
            feedback_evidence_rejected: false,
            availability_before,
            availability_after: memory.availability(),
            confidence_before,
            confidence_after: memory.revision_confidence_cap(),
            memory,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalSkillRevisionApplicationAndMemoryUpdate;

impl UniversalSkillRevisionApplicationAndMemoryUpdate {
    pub fn evaluate(
        entry: &SkillRevisionMemoryEntry,
        feedback: &SkillOutcomeFeedbackResult,
        policy: SkillRevisionMemoryPolicy,
    ) -> SkillRevisionApplicationResult {
        SkillRevisionApplicationAndMemoryUpdate::apply(entry, feedback, policy)
    }
}

#[cfg(test)]
mod skill_revision_application_memory_update_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn record() -> CompressedSkillRecord {
        CompressedSkillRecord {
            invariant_dictionary: vec![a(7)],
            initial_state: CompressedSkillTerm::StructuralSlot(0),
            goal_identity: CompressedSkillTerm::InvariantRef(0),
            steps: vec![
                CompressedSkillStep {
                    required_state: CompressedSkillTerm::StructuralSlot(0),
                    action: CompressedSkillTerm::StructuralSlot(1),
                    observed_outcome: CompressedSkillTerm::StructuralSlot(2),
                },
                CompressedSkillStep {
                    required_state: CompressedSkillTerm::StructuralSlot(2),
                    action: CompressedSkillTerm::InvariantRef(0),
                    observed_outcome: CompressedSkillTerm::InvariantRef(0),
                },
            ],
            structural_slot_count: 3,
            context_slot_count: 0,
            invariant_occurrence_count: 3,
            compression_gain: 2,
            source_generalization_count: 2,
            source_support_sum: 8,
            success_confidence_floor: s(900),
            step_confidence_floor: s(800),
        }
    }

    fn memory() -> SkillRevisionMemoryEntry {
        SkillRevisionMemoryEntry::new(record())
    }

    fn policy() -> SkillRevisionMemoryPolicy {
        SkillRevisionMemoryPolicy::new(16, 200, 100, s(500), s(500)).unwrap()
    }

    fn feedback(
        disposition: SkillRevisionDisposition,
        confidence: u16,
    ) -> SkillOutcomeFeedbackResult {
        SkillOutcomeFeedbackResult {
            input_observation_count: 2,
            considered_observation_count: 2,
            observation_frontier_truncated: false,
            evaluation_count: 2,
            evaluation_frontier_truncated: false,
            low_confidence_count: 0,
            exact_step_count: 2,
            execution_mismatch_count: 0,
            outcome_mismatch_count: 0,
            missing_plan_step_count: 0,
            extra_observation_count: 0,
            feedback_confidence_floor: Some(s(confidence)),
            disposition,
        }
    }

    #[test]
    fn revision_policy_requires_positive_bounds_penalties_and_confidence() {
        assert_eq!(SkillRevisionMemoryPolicy::new(0, 1, 1, s(1), s(1),), None);

        assert_eq!(SkillRevisionMemoryPolicy::new(1, 0, 1, s(1), s(1),), None);

        assert_eq!(SkillRevisionMemoryPolicy::new(1, 1, 0, s(1), s(1),), None);

        assert!(SkillRevisionMemoryPolicy::new(1, 1, 1, s(1), s(1),).is_some());
    }

    #[test]
    fn memory_entry_starts_active_at_conservative_provenance_floor() {
        let entry = memory();

        assert_eq!(entry.availability(), SkillMemoryAvailability::Active);

        assert_eq!(entry.revision_confidence_cap(), s(800));

        assert_eq!(entry.provenance_confidence_ceiling(), s(800));

        assert!(entry.reusable());
    }

    #[test]
    fn reinforcement_recovers_confidence_but_never_exceeds_provenance() {
        let first = SkillRevisionApplicationAndMemoryUpdate::apply(
            &memory(),
            &feedback(SkillRevisionDisposition::Weaken, 900),
            policy(),
        );

        assert_eq!(first.confidence_after(), s(600));

        let second = SkillRevisionApplicationAndMemoryUpdate::apply(
            first.memory(),
            &feedback(SkillRevisionDisposition::Reinforce, 900),
            policy(),
        );

        assert_eq!(second.confidence_after(), s(700));

        let third = SkillRevisionApplicationAndMemoryUpdate::apply(
            second.memory(),
            &feedback(SkillRevisionDisposition::Reinforce, 900),
            policy(),
        );

        let fourth = SkillRevisionApplicationAndMemoryUpdate::apply(
            third.memory(),
            &feedback(SkillRevisionDisposition::Reinforce, 900),
            policy(),
        );

        assert_eq!(third.confidence_after(), s(800));

        assert_eq!(fourth.confidence_after(), s(800));
    }

    #[test]
    fn retain_preserves_confidence_and_availability() {
        let entry = memory();

        let result = SkillRevisionApplicationAndMemoryUpdate::apply(
            &entry,
            &feedback(SkillRevisionDisposition::Retain, 900),
            policy(),
        );

        assert!(result.revision_applied());

        assert_eq!(result.confidence_before(), result.confidence_after());

        assert_eq!(result.availability_after(), SkillMemoryAvailability::Active);

        assert_eq!(result.memory().retention_count(), 1);
    }

    #[test]
    fn weaken_applies_bounded_confidence_penalty() {
        let result = SkillRevisionApplicationAndMemoryUpdate::apply(
            &memory(),
            &feedback(SkillRevisionDisposition::Weaken, 900),
            policy(),
        );

        assert_eq!(result.confidence_before(), s(800));

        assert_eq!(result.confidence_after(), s(600));

        assert_eq!(result.memory().weakening_count(), 1);

        assert_eq!(result.availability_after(), SkillMemoryAvailability::Active);
    }

    #[test]
    fn accumulated_weakening_suspends_below_active_threshold() {
        let first = SkillRevisionApplicationAndMemoryUpdate::apply(
            &memory(),
            &feedback(SkillRevisionDisposition::Weaken, 900),
            policy(),
        );

        let second = SkillRevisionApplicationAndMemoryUpdate::apply(
            first.memory(),
            &feedback(SkillRevisionDisposition::Weaken, 900),
            policy(),
        );

        assert_eq!(second.confidence_after(), s(400));

        assert_eq!(
            second.availability_after(),
            SkillMemoryAvailability::Suspended
        );

        assert!(!second.memory().reusable());
    }

    #[test]
    fn suspend_disposition_immediately_disables_reuse() {
        let result = SkillRevisionApplicationAndMemoryUpdate::apply(
            &memory(),
            &feedback(SkillRevisionDisposition::Suspend, 900),
            policy(),
        );

        assert_eq!(
            result.availability_before(),
            SkillMemoryAvailability::Active
        );

        assert_eq!(
            result.availability_after(),
            SkillMemoryAvailability::Suspended
        );

        assert_eq!(result.memory().suspension_count(), 1);
    }

    #[test]
    fn abstain_and_low_confidence_feedback_do_not_revise_memory() {
        let entry = memory();

        let abstain = SkillRevisionApplicationAndMemoryUpdate::apply(
            &entry,
            &feedback(SkillRevisionDisposition::Abstain, 900),
            policy(),
        );

        assert!(!abstain.revision_applied());
        assert_eq!(abstain.memory(), &entry);

        let weak_evidence = SkillRevisionApplicationAndMemoryUpdate::apply(
            &entry,
            &feedback(SkillRevisionDisposition::Weaken, 400),
            policy(),
        );

        assert!(weak_evidence.feedback_evidence_rejected());

        assert!(!weak_evidence.revision_applied());

        assert_eq!(weak_evidence.memory(), &entry);
    }

    #[test]
    fn suspended_skill_is_not_implicitly_reactivated_by_reinforcement() {
        let suspended = SkillRevisionApplicationAndMemoryUpdate::apply(
            &memory(),
            &feedback(SkillRevisionDisposition::Suspend, 900),
            policy(),
        );

        let reinforced = SkillRevisionApplicationAndMemoryUpdate::apply(
            suspended.memory(),
            &feedback(SkillRevisionDisposition::Reinforce, 900),
            policy(),
        );

        assert_eq!(
            reinforced.availability_after(),
            SkillMemoryAvailability::Suspended
        );

        assert!(!reinforced.memory().reusable());
    }

    #[test]
    fn revision_never_mutates_source_provenance() {
        let entry = memory();

        let support_before = entry.record().source_support_sum();

        let generalization_before = entry.record().source_generalization_count();

        let success_before = entry.record().success_confidence_floor();

        let step_before = entry.record().step_confidence_floor();

        let revised = SkillRevisionApplicationAndMemoryUpdate::apply(
            &entry,
            &feedback(SkillRevisionDisposition::Weaken, 900),
            policy(),
        );

        assert_eq!(
            revised.memory().record().source_support_sum(),
            support_before
        );

        assert_eq!(
            revised.memory().record().source_generalization_count(),
            generalization_before
        );

        assert_eq!(
            revised.memory().record().success_confidence_floor(),
            success_before
        );

        assert_eq!(
            revised.memory().record().step_confidence_floor(),
            step_before
        );
    }

    #[test]
    fn hard_revision_budget_blocks_additional_memory_updates() {
        let p = SkillRevisionMemoryPolicy::new(1, 200, 100, s(500), s(500)).unwrap();

        let first = SkillRevisionApplicationAndMemoryUpdate::apply(
            &memory(),
            &feedback(SkillRevisionDisposition::Weaken, 900),
            p,
        );

        assert_eq!(first.memory().applied_revision_count(), 1);

        let before = first.memory().clone();

        let second = SkillRevisionApplicationAndMemoryUpdate::apply(
            first.memory(),
            &feedback(SkillRevisionDisposition::Reinforce, 900),
            p,
        );

        assert!(second.revision_budget_exhausted());

        assert!(!second.revision_applied());

        assert_eq!(second.memory(), &before);
    }

    #[test]
    fn revision_application_is_deterministic_non_mutating_and_facade_equivalent() {
        let entry = memory();
        let before = entry.clone();

        let evidence = feedback(SkillRevisionDisposition::Weaken, 900);

        let evidence_before = evidence.clone();

        let p = policy();

        let direct = SkillRevisionApplicationAndMemoryUpdate::apply(&entry, &evidence, p);

        let facade =
            UniversalSkillRevisionApplicationAndMemoryUpdate::evaluate(&entry, &evidence, p);

        let repeated =
            UniversalSkillRevisionApplicationAndMemoryUpdate::evaluate(&entry, &evidence, p);

        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(entry, before);
        assert_eq!(evidence, evidence_before);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsolidatedSkillMemoryTier {
    Hot,
    Warm,
    Cold,
    Forgotten,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillMemoryUseEvidence {
    access_count: usize,
    successful_reuse_count: usize,
    failed_reuse_count: usize,
    recency_signal: CognitiveSignal,
}

impl SkillMemoryUseEvidence {
    pub fn new(
        access_count: usize,
        successful_reuse_count: usize,
        failed_reuse_count: usize,
        recency_signal: CognitiveSignal,
    ) -> Option<Self> {
        if successful_reuse_count.saturating_add(failed_reuse_count) > access_count
            || recency_signal == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            access_count,
            successful_reuse_count,
            failed_reuse_count,
            recency_signal,
        })
    }

    pub fn access_count(&self) -> usize {
        self.access_count
    }

    pub fn successful_reuse_count(&self) -> usize {
        self.successful_reuse_count
    }

    pub fn failed_reuse_count(&self) -> usize {
        self.failed_reuse_count
    }

    pub fn recency_signal(&self) -> CognitiveSignal {
        self.recency_signal
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillMemoryConsolidationCandidate {
    memory: SkillRevisionMemoryEntry,
    use_evidence: SkillMemoryUseEvidence,
}

impl SkillMemoryConsolidationCandidate {
    pub fn new(memory: SkillRevisionMemoryEntry, use_evidence: SkillMemoryUseEvidence) -> Self {
        Self {
            memory,
            use_evidence,
        }
    }

    pub fn memory(&self) -> &SkillRevisionMemoryEntry {
        &self.memory
    }

    pub fn use_evidence(&self) -> &SkillMemoryUseEvidence {
        &self.use_evidence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillMemoryConsolidationBounds {
    max_input_entries: usize,
    max_evaluations: usize,
    max_hot_entries: usize,
    max_warm_entries: usize,
    max_cold_entries: usize,
    max_forgotten_archive_entries: usize,
}

impl SkillMemoryConsolidationBounds {
    pub fn new(
        max_input_entries: usize,
        max_evaluations: usize,
        max_hot_entries: usize,
        max_warm_entries: usize,
        max_cold_entries: usize,
        max_forgotten_archive_entries: usize,
    ) -> Option<Self> {
        if max_input_entries == 0
            || max_evaluations == 0
            || max_hot_entries == 0
            || max_warm_entries == 0
            || max_cold_entries == 0
            || max_forgotten_archive_entries == 0
        {
            return None;
        }

        Some(Self {
            max_input_entries,
            max_evaluations,
            max_hot_entries,
            max_warm_entries,
            max_cold_entries,
            max_forgotten_archive_entries,
        })
    }

    pub fn max_input_entries(self) -> usize {
        self.max_input_entries
    }

    pub fn max_evaluations(self) -> usize {
        self.max_evaluations
    }

    pub fn max_hot_entries(self) -> usize {
        self.max_hot_entries
    }

    pub fn max_warm_entries(self) -> usize {
        self.max_warm_entries
    }

    pub fn max_cold_entries(self) -> usize {
        self.max_cold_entries
    }

    pub fn max_forgotten_archive_entries(self) -> usize {
        self.max_forgotten_archive_entries
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillMemoryConsolidationThresholds {
    hot_score: CognitiveSignal,
    warm_score: CognitiveSignal,
    minimum_retention_score: CognitiveSignal,
    minimum_hot_successes: usize,
    forgetting_failure_count: usize,
}

impl SkillMemoryConsolidationThresholds {
    pub fn new(
        hot_score: CognitiveSignal,
        warm_score: CognitiveSignal,
        minimum_retention_score: CognitiveSignal,
        minimum_hot_successes: usize,
        forgetting_failure_count: usize,
    ) -> Option<Self> {
        if hot_score == CognitiveSignal::zero()
            || warm_score == CognitiveSignal::zero()
            || minimum_retention_score == CognitiveSignal::zero()
            || minimum_hot_successes == 0
            || forgetting_failure_count == 0
            || hot_score.value() < warm_score.value()
            || warm_score.value() < minimum_retention_score.value()
        {
            return None;
        }

        Some(Self {
            hot_score,
            warm_score,
            minimum_retention_score,
            minimum_hot_successes,
            forgetting_failure_count,
        })
    }

    pub fn hot_score(self) -> CognitiveSignal {
        self.hot_score
    }

    pub fn warm_score(self) -> CognitiveSignal {
        self.warm_score
    }

    pub fn minimum_retention_score(self) -> CognitiveSignal {
        self.minimum_retention_score
    }

    pub fn minimum_hot_successes(self) -> usize {
        self.minimum_hot_successes
    }

    pub fn forgetting_failure_count(self) -> usize {
        self.forgetting_failure_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillMemoryConsolidationPolicy {
    bounds: SkillMemoryConsolidationBounds,
    thresholds: SkillMemoryConsolidationThresholds,
}

impl SkillMemoryConsolidationPolicy {
    pub fn new(
        bounds: SkillMemoryConsolidationBounds,
        thresholds: SkillMemoryConsolidationThresholds,
    ) -> Self {
        Self { bounds, thresholds }
    }

    pub fn bounds(self) -> SkillMemoryConsolidationBounds {
        self.bounds
    }

    pub fn thresholds(self) -> SkillMemoryConsolidationThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsolidatedSkillMemoryEntry {
    memory: SkillRevisionMemoryEntry,
    use_evidence: SkillMemoryUseEvidence,
    tier: ConsolidatedSkillMemoryTier,
    retention_score: Option<CognitiveSignal>,
}

impl ConsolidatedSkillMemoryEntry {
    pub fn memory(&self) -> &SkillRevisionMemoryEntry {
        &self.memory
    }

    pub fn use_evidence(&self) -> &SkillMemoryUseEvidence {
        &self.use_evidence
    }

    pub fn tier(&self) -> ConsolidatedSkillMemoryTier {
        self.tier
    }

    pub fn retention_score(&self) -> Option<CognitiveSignal> {
        self.retention_score
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillMemoryConsolidationResult {
    input_entry_count: usize,
    unique_entry_count: usize,
    considered_entry_count: usize,
    input_frontier_truncated: bool,
    evaluation_count: usize,
    evaluation_frontier_truncated: bool,
    hot_before_frontier: usize,
    warm_before_frontier: usize,
    cold_before_frontier: usize,
    forgotten_before_frontier: usize,
    tier_frontier_truncated: bool,
    hot: Vec<ConsolidatedSkillMemoryEntry>,
    warm: Vec<ConsolidatedSkillMemoryEntry>,
    cold: Vec<ConsolidatedSkillMemoryEntry>,
    forgotten_archive: Vec<ConsolidatedSkillMemoryEntry>,
}

impl SkillMemoryConsolidationResult {
    pub fn input_entry_count(&self) -> usize {
        self.input_entry_count
    }

    pub fn unique_entry_count(&self) -> usize {
        self.unique_entry_count
    }

    pub fn considered_entry_count(&self) -> usize {
        self.considered_entry_count
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

    pub fn hot_before_frontier(&self) -> usize {
        self.hot_before_frontier
    }

    pub fn warm_before_frontier(&self) -> usize {
        self.warm_before_frontier
    }

    pub fn cold_before_frontier(&self) -> usize {
        self.cold_before_frontier
    }

    pub fn forgotten_before_frontier(&self) -> usize {
        self.forgotten_before_frontier
    }

    pub fn tier_frontier_truncated(&self) -> bool {
        self.tier_frontier_truncated
    }

    pub fn hot(&self) -> &[ConsolidatedSkillMemoryEntry] {
        &self.hot
    }

    pub fn warm(&self) -> &[ConsolidatedSkillMemoryEntry] {
        &self.warm
    }

    pub fn cold(&self) -> &[ConsolidatedSkillMemoryEntry] {
        &self.cold
    }

    pub fn forgotten_archive(&self) -> &[ConsolidatedSkillMemoryEntry] {
        &self.forgotten_archive
    }

    pub fn retained_count(&self) -> usize {
        self.hot
            .len()
            .saturating_add(self.warm.len())
            .saturating_add(self.cold.len())
    }

    pub fn forgotten_count(&self) -> usize {
        self.forgotten_archive.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SkillMemoryConsolidationAndForgetting;

impl SkillMemoryConsolidationAndForgetting {
    fn signal(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn same_skill(
        left: &SkillMemoryConsolidationCandidate,
        right: &SkillMemoryConsolidationCandidate,
    ) -> bool {
        left.memory().record() == right.memory().record()
    }

    fn candidate_order(
        left: &SkillMemoryConsolidationCandidate,
        right: &SkillMemoryConsolidationCandidate,
    ) -> std::cmp::Ordering {
        right
            .memory()
            .reusable()
            .cmp(&left.memory().reusable())
            .then_with(|| {
                right
                    .memory()
                    .revision_confidence_cap()
                    .value()
                    .cmp(&left.memory().revision_confidence_cap().value())
            })
            .then_with(|| {
                right
                    .use_evidence()
                    .successful_reuse_count()
                    .cmp(&left.use_evidence().successful_reuse_count())
            })
            .then_with(|| {
                right
                    .use_evidence()
                    .recency_signal()
                    .value()
                    .cmp(&left.use_evidence().recency_signal().value())
            })
            .then_with(|| {
                format!("{:?}", left.memory().record())
                    .cmp(&format!("{:?}", right.memory().record()))
            })
    }

    fn retention_score(
        memory: &SkillRevisionMemoryEntry,
        evidence: &SkillMemoryUseEvidence,
    ) -> Option<CognitiveSignal> {
        if evidence.access_count() == 0 {
            return None;
        }

        let utility = (evidence.successful_reuse_count().saturating_mul(1000)
            / evidence.access_count())
        .min(1000) as u16;

        let confidence = memory.revision_confidence_cap().value();

        let recency = evidence.recency_signal().value();

        Some(Self::signal(confidence.min(recency).min(utility)))
    }

    fn classify(
        candidate: SkillMemoryConsolidationCandidate,
        thresholds: SkillMemoryConsolidationThresholds,
    ) -> ConsolidatedSkillMemoryEntry {
        let score = Self::retention_score(candidate.memory(), candidate.use_evidence());

        let tier = if candidate.use_evidence().failed_reuse_count()
            >= thresholds.forgetting_failure_count()
        {
            ConsolidatedSkillMemoryTier::Forgotten
        } else if !candidate.memory().reusable() {
            ConsolidatedSkillMemoryTier::Cold
        } else {
            match score {
                None => ConsolidatedSkillMemoryTier::Cold,

                Some(value) if value.value() < thresholds.minimum_retention_score().value() => {
                    ConsolidatedSkillMemoryTier::Forgotten
                }

                Some(value)
                    if value.value() >= thresholds.hot_score().value()
                        && candidate.use_evidence().successful_reuse_count()
                            >= thresholds.minimum_hot_successes() =>
                {
                    ConsolidatedSkillMemoryTier::Hot
                }

                Some(value) if value.value() >= thresholds.warm_score().value() => {
                    ConsolidatedSkillMemoryTier::Warm
                }

                Some(_) => ConsolidatedSkillMemoryTier::Cold,
            }
        };

        ConsolidatedSkillMemoryEntry {
            memory: candidate.memory,
            use_evidence: candidate.use_evidence,
            tier,
            retention_score: score,
        }
    }

    fn consolidated_order(
        left: &ConsolidatedSkillMemoryEntry,
        right: &ConsolidatedSkillMemoryEntry,
    ) -> std::cmp::Ordering {
        right
            .retention_score()
            .map(|x| x.value())
            .unwrap_or(0)
            .cmp(&left.retention_score().map(|x| x.value()).unwrap_or(0))
            .then_with(|| {
                right
                    .use_evidence()
                    .successful_reuse_count()
                    .cmp(&left.use_evidence().successful_reuse_count())
            })
            .then_with(|| {
                right
                    .memory()
                    .revision_confidence_cap()
                    .value()
                    .cmp(&left.memory().revision_confidence_cap().value())
            })
            .then_with(|| {
                format!("{:?}", left.memory().record())
                    .cmp(&format!("{:?}", right.memory().record()))
            })
    }

    pub fn consolidate(
        candidates: &[SkillMemoryConsolidationCandidate],
        policy: SkillMemoryConsolidationPolicy,
    ) -> SkillMemoryConsolidationResult {
        let bounds = policy.bounds();
        let thresholds = policy.thresholds();

        let input_entry_count = candidates.len();

        let mut ranked = candidates.to_vec();

        ranked.sort_by(Self::candidate_order);

        ranked.dedup_by(|left, right| Self::same_skill(left, right));

        let unique_entry_count = ranked.len();

        ranked.truncate(bounds.max_input_entries());

        let considered_entry_count = ranked.len();

        let mut evaluation_count = 0;
        let mut evaluation_frontier_truncated = false;

        let mut hot = Vec::new();
        let mut warm = Vec::new();
        let mut cold = Vec::new();
        let mut forgotten_archive = Vec::new();

        for candidate in ranked {
            if evaluation_count >= bounds.max_evaluations() {
                evaluation_frontier_truncated = true;
                break;
            }

            evaluation_count += 1;

            let consolidated = Self::classify(candidate, thresholds);

            match consolidated.tier() {
                ConsolidatedSkillMemoryTier::Hot => {
                    hot.push(consolidated);
                }

                ConsolidatedSkillMemoryTier::Warm => {
                    warm.push(consolidated);
                }

                ConsolidatedSkillMemoryTier::Cold => {
                    cold.push(consolidated);
                }

                ConsolidatedSkillMemoryTier::Forgotten => {
                    forgotten_archive.push(consolidated);
                }
            }
        }

        hot.sort_by(Self::consolidated_order);
        warm.sort_by(Self::consolidated_order);
        cold.sort_by(Self::consolidated_order);
        forgotten_archive.sort_by(Self::consolidated_order);

        let hot_before_frontier = hot.len();
        let warm_before_frontier = warm.len();
        let cold_before_frontier = cold.len();
        let forgotten_before_frontier = forgotten_archive.len();

        hot.truncate(bounds.max_hot_entries());

        warm.truncate(bounds.max_warm_entries());

        cold.truncate(bounds.max_cold_entries());

        forgotten_archive.truncate(bounds.max_forgotten_archive_entries());

        let tier_frontier_truncated = hot_before_frontier > hot.len()
            || warm_before_frontier > warm.len()
            || cold_before_frontier > cold.len()
            || forgotten_before_frontier > forgotten_archive.len();

        SkillMemoryConsolidationResult {
            input_entry_count,
            unique_entry_count,
            considered_entry_count,
            input_frontier_truncated: unique_entry_count > considered_entry_count,
            evaluation_count,
            evaluation_frontier_truncated,
            hot_before_frontier,
            warm_before_frontier,
            cold_before_frontier,
            forgotten_before_frontier,
            tier_frontier_truncated,
            hot,
            warm,
            cold,
            forgotten_archive,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalSkillMemoryConsolidationAndForgetting;

impl UniversalSkillMemoryConsolidationAndForgetting {
    pub fn evaluate(
        candidates: &[SkillMemoryConsolidationCandidate],
        policy: SkillMemoryConsolidationPolicy,
    ) -> SkillMemoryConsolidationResult {
        SkillMemoryConsolidationAndForgetting::consolidate(candidates, policy)
    }
}

#[cfg(test)]
mod skill_memory_consolidation_forgetting_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn record(id: u64) -> CompressedSkillRecord {
        CompressedSkillRecord {
            invariant_dictionary: vec![a(id)],
            initial_state: CompressedSkillTerm::StructuralSlot(0),
            goal_identity: CompressedSkillTerm::InvariantRef(0),
            steps: vec![
                CompressedSkillStep {
                    required_state: CompressedSkillTerm::StructuralSlot(0),
                    action: CompressedSkillTerm::StructuralSlot(1),
                    observed_outcome: CompressedSkillTerm::StructuralSlot(2),
                },
                CompressedSkillStep {
                    required_state: CompressedSkillTerm::StructuralSlot(2),
                    action: CompressedSkillTerm::InvariantRef(0),
                    observed_outcome: CompressedSkillTerm::InvariantRef(0),
                },
            ],
            structural_slot_count: 3,
            context_slot_count: 0,
            invariant_occurrence_count: 3,
            compression_gain: 2,
            source_generalization_count: 2,
            source_support_sum: 8,
            success_confidence_floor: s(900),
            step_confidence_floor: s(900),
        }
    }

    fn memory(id: u64) -> SkillRevisionMemoryEntry {
        SkillRevisionMemoryEntry::new(record(id))
    }

    fn evidence(
        accesses: usize,
        successes: usize,
        failures: usize,
        recency: u16,
    ) -> SkillMemoryUseEvidence {
        SkillMemoryUseEvidence::new(accesses, successes, failures, s(recency)).unwrap()
    }

    fn candidate(
        id: u64,
        accesses: usize,
        successes: usize,
        failures: usize,
        recency: u16,
    ) -> SkillMemoryConsolidationCandidate {
        SkillMemoryConsolidationCandidate::new(
            memory(id),
            evidence(accesses, successes, failures, recency),
        )
    }

    fn thresholds() -> SkillMemoryConsolidationThresholds {
        SkillMemoryConsolidationThresholds::new(s(700), s(500), s(200), 3, 5).unwrap()
    }

    fn policy() -> SkillMemoryConsolidationPolicy {
        SkillMemoryConsolidationPolicy::new(
            SkillMemoryConsolidationBounds::new(32, 32, 32, 32, 32, 32).unwrap(),
            thresholds(),
        )
    }

    fn suspended_memory(id: u64) -> SkillRevisionMemoryEntry {
        let entry = memory(id);

        let feedback = SkillOutcomeFeedbackResult {
            input_observation_count: 2,
            considered_observation_count: 2,
            observation_frontier_truncated: false,
            evaluation_count: 2,
            evaluation_frontier_truncated: false,
            low_confidence_count: 0,
            exact_step_count: 0,
            execution_mismatch_count: 1,
            outcome_mismatch_count: 0,
            missing_plan_step_count: 0,
            extra_observation_count: 0,
            feedback_confidence_floor: Some(s(900)),
            disposition: SkillRevisionDisposition::Suspend,
        };

        let revision_policy = SkillRevisionMemoryPolicy::new(8, 100, 100, s(500), s(500)).unwrap();

        SkillRevisionApplicationAndMemoryUpdate::apply(&entry, &feedback, revision_policy)
            .memory()
            .clone()
    }

    #[test]
    fn consolidation_policy_and_use_evidence_require_valid_bounds() {
        assert_eq!(SkillMemoryConsolidationBounds::new(0, 1, 1, 1, 1, 1), None);

        assert_eq!(SkillMemoryUseEvidence::new(1, 2, 0, s(900)), None);

        assert_eq!(
            SkillMemoryConsolidationThresholds::new(s(500), s(700), s(200), 1, 1,),
            None
        );

        assert!(SkillMemoryConsolidationBounds::new(1, 1, 1, 1, 1, 1).is_some());
    }

    #[test]
    fn unused_active_skill_moves_to_cold_memory_without_forgetting() {
        let result = SkillMemoryConsolidationAndForgetting::consolidate(
            &[candidate(1, 0, 0, 0, 900)],
            policy(),
        );

        assert_eq!(result.cold().len(), 1);
        assert_eq!(result.forgotten_count(), 0);

        assert_eq!(result.cold()[0].tier(), ConsolidatedSkillMemoryTier::Cold);

        assert_eq!(result.cold()[0].retention_score(), None);
    }

    #[test]
    fn reliable_frequent_recent_skill_becomes_hot() {
        let result = SkillMemoryConsolidationAndForgetting::consolidate(
            &[candidate(2, 10, 9, 1, 900)],
            policy(),
        );

        assert_eq!(result.hot().len(), 1);

        assert_eq!(result.hot()[0].retention_score().unwrap(), s(900));
    }

    #[test]
    fn moderate_evidence_skill_becomes_warm() {
        let result = SkillMemoryConsolidationAndForgetting::consolidate(
            &[candidate(3, 10, 7, 1, 650)],
            policy(),
        );

        assert_eq!(result.warm().len(), 1);

        assert_eq!(result.warm()[0].retention_score().unwrap(), s(650));
    }

    #[test]
    fn low_but_retained_score_keeps_skill_cold() {
        let result = SkillMemoryConsolidationAndForgetting::consolidate(
            &[candidate(4, 10, 3, 1, 400)],
            policy(),
        );

        assert_eq!(result.cold().len(), 1);

        assert_eq!(result.cold()[0].retention_score().unwrap(), s(300));
    }

    #[test]
    fn insufficient_retention_score_moves_skill_to_forgotten_archive() {
        let result = SkillMemoryConsolidationAndForgetting::consolidate(
            &[candidate(5, 10, 1, 1, 900)],
            policy(),
        );

        assert_eq!(result.forgotten_count(), 1);

        assert_eq!(
            result.forgotten_archive()[0].tier(),
            ConsolidatedSkillMemoryTier::Forgotten
        );

        assert_eq!(
            result.forgotten_archive()[0].retention_score().unwrap(),
            s(100)
        );
    }

    #[test]
    fn repeated_failures_force_forgetting_despite_high_confidence() {
        let result = SkillMemoryConsolidationAndForgetting::consolidate(
            &[candidate(6, 10, 5, 5, 1000)],
            policy(),
        );

        assert_eq!(result.forgotten_count(), 1);
    }

    #[test]
    fn suspended_skill_cannot_enter_hot_or_warm_memory() {
        let candidate =
            SkillMemoryConsolidationCandidate::new(suspended_memory(7), evidence(10, 10, 0, 1000));

        let result = SkillMemoryConsolidationAndForgetting::consolidate(&[candidate], policy());

        assert!(result.hot().is_empty());
        assert!(result.warm().is_empty());
        assert_eq!(result.cold().len(), 1);
    }

    #[test]
    fn consolidation_preserves_skill_revision_and_source_provenance() {
        let candidate = candidate(8, 10, 9, 0, 900);

        let before = candidate.memory().clone();

        let result = SkillMemoryConsolidationAndForgetting::consolidate(
            std::slice::from_ref(&candidate),
            policy(),
        );

        let after = result.hot()[0].memory();

        assert_eq!(after, &before);

        assert_eq!(after.record(), before.record());
    }

    #[test]
    fn semantic_duplicate_keeps_stronger_usage_evidence_once() {
        let weak = candidate(9, 10, 5, 1, 600);

        let strong = candidate(9, 10, 9, 0, 900);

        let result = SkillMemoryConsolidationAndForgetting::consolidate(&[weak, strong], policy());

        assert_eq!(result.input_entry_count(), 2);

        assert_eq!(result.unique_entry_count(), 1);

        assert_eq!(result.hot().len(), 1);

        assert_eq!(result.hot()[0].use_evidence().successful_reuse_count(), 9);
    }

    #[test]
    fn hard_input_evaluation_and_tier_frontiers_are_enforced() {
        let items = vec![
            candidate(10, 10, 10, 0, 1000),
            candidate(11, 10, 9, 0, 900),
            candidate(12, 10, 1, 0, 900),
        ];

        let input_policy = SkillMemoryConsolidationPolicy::new(
            SkillMemoryConsolidationBounds::new(1, 32, 32, 32, 32, 32).unwrap(),
            thresholds(),
        );

        let input = SkillMemoryConsolidationAndForgetting::consolidate(&items, input_policy);

        assert_eq!(input.unique_entry_count(), 3);

        assert_eq!(input.considered_entry_count(), 1);

        assert!(input.input_frontier_truncated());

        let eval_policy = SkillMemoryConsolidationPolicy::new(
            SkillMemoryConsolidationBounds::new(32, 1, 32, 32, 32, 32).unwrap(),
            thresholds(),
        );

        let eval = SkillMemoryConsolidationAndForgetting::consolidate(&items, eval_policy);

        assert_eq!(eval.evaluation_count(), 1);

        assert!(eval.evaluation_frontier_truncated());

        let frontier_policy = SkillMemoryConsolidationPolicy::new(
            SkillMemoryConsolidationBounds::new(32, 32, 1, 32, 32, 1).unwrap(),
            thresholds(),
        );

        let frontier = SkillMemoryConsolidationAndForgetting::consolidate(
            &[
                candidate(20, 10, 10, 0, 1000),
                candidate(21, 10, 9, 0, 900),
                candidate(22, 10, 1, 0, 900),
                candidate(23, 10, 1, 0, 900),
            ],
            frontier_policy,
        );

        assert_eq!(frontier.hot_before_frontier(), 2);

        assert_eq!(frontier.forgotten_before_frontier(), 2);

        assert_eq!(frontier.hot().len(), 1);

        assert_eq!(frontier.forgotten_archive().len(), 1);

        assert!(frontier.tier_frontier_truncated());
    }

    #[test]
    fn consolidation_is_deterministic_non_mutating_and_facade_equivalent() {
        let items = vec![
            candidate(30, 10, 7, 1, 650),
            candidate(31, 10, 9, 0, 900),
            candidate(32, 10, 2, 0, 500),
        ];

        let before = items.clone();

        let mut reversed = items.clone();

        reversed.reverse();

        let p = policy();

        let direct = SkillMemoryConsolidationAndForgetting::consolidate(&items, p);

        let reordered = SkillMemoryConsolidationAndForgetting::consolidate(&reversed, p);

        let facade = UniversalSkillMemoryConsolidationAndForgetting::evaluate(&items, p);

        let repeated = UniversalSkillMemoryConsolidationAndForgetting::evaluate(&items, p);

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(items, before);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedSkillLearningCycleInput {
    memory: SkillRevisionMemoryEntry,
    reuse_request: GroundedSkillReuseRequest,
    execution_observations: Vec<SkillExecutionObservation>,
    use_evidence: SkillMemoryUseEvidence,
}

impl IntegratedSkillLearningCycleInput {
    pub fn new(
        memory: SkillRevisionMemoryEntry,
        reuse_request: GroundedSkillReuseRequest,
        execution_observations: Vec<SkillExecutionObservation>,
        use_evidence: SkillMemoryUseEvidence,
    ) -> Self {
        Self {
            memory,
            reuse_request,
            execution_observations,
            use_evidence,
        }
    }

    pub fn memory(&self) -> &SkillRevisionMemoryEntry {
        &self.memory
    }

    pub fn reuse_request(&self) -> &GroundedSkillReuseRequest {
        &self.reuse_request
    }

    pub fn execution_observations(&self) -> &[SkillExecutionObservation] {
        &self.execution_observations
    }

    pub fn use_evidence(&self) -> &SkillMemoryUseEvidence {
        &self.use_evidence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegratedSkillLearningCyclePolicy {
    reuse: SkillReusePolicy,
    feedback: SkillOutcomeFeedbackPolicy,
    revision: SkillRevisionMemoryPolicy,
    consolidation: SkillMemoryConsolidationPolicy,
}

impl IntegratedSkillLearningCyclePolicy {
    pub fn new(
        reuse: SkillReusePolicy,
        feedback: SkillOutcomeFeedbackPolicy,
        revision: SkillRevisionMemoryPolicy,
        consolidation: SkillMemoryConsolidationPolicy,
    ) -> Self {
        Self {
            reuse,
            feedback,
            revision,
            consolidation,
        }
    }

    pub fn reuse(self) -> SkillReusePolicy {
        self.reuse
    }

    pub fn feedback(self) -> SkillOutcomeFeedbackPolicy {
        self.feedback
    }

    pub fn revision(self) -> SkillRevisionMemoryPolicy {
        self.revision
    }

    pub fn consolidation(self) -> SkillMemoryConsolidationPolicy {
        self.consolidation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedSkillLearningCycleResult {
    retrieval: SkillRetrievalReuseResult,
    selected_plan: Option<GroundedSkillReusePlan>,
    feedback: Option<SkillOutcomeFeedbackResult>,
    revision: Option<SkillRevisionApplicationResult>,
    updated_memory: SkillRevisionMemoryEntry,
    updated_use_evidence: SkillMemoryUseEvidence,
    consolidation: SkillMemoryConsolidationResult,
}

impl IntegratedSkillLearningCycleResult {
    pub fn retrieval(&self) -> &SkillRetrievalReuseResult {
        &self.retrieval
    }

    pub fn selected_plan(&self) -> Option<&GroundedSkillReusePlan> {
        self.selected_plan.as_ref()
    }

    pub fn feedback(&self) -> Option<&SkillOutcomeFeedbackResult> {
        self.feedback.as_ref()
    }

    pub fn revision(&self) -> Option<&SkillRevisionApplicationResult> {
        self.revision.as_ref()
    }

    pub fn updated_memory(&self) -> &SkillRevisionMemoryEntry {
        &self.updated_memory
    }

    pub fn updated_use_evidence(&self) -> &SkillMemoryUseEvidence {
        &self.updated_use_evidence
    }

    pub fn consolidation(&self) -> &SkillMemoryConsolidationResult {
        &self.consolidation
    }

    pub fn reused_skill(&self) -> bool {
        self.selected_plan.is_some()
    }

    pub fn revised_memory(&self) -> bool {
        self.revision
            .as_ref()
            .is_some_and(SkillRevisionApplicationResult::revision_applied)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IntegratedSkillLearningCycle;

impl IntegratedSkillLearningCycle {
    fn update_use_evidence(
        evidence: &SkillMemoryUseEvidence,
        disposition: SkillRevisionDisposition,
    ) -> SkillMemoryUseEvidence {
        let mut successes = evidence.successful_reuse_count();

        let mut failures = evidence.failed_reuse_count();

        match disposition {
            SkillRevisionDisposition::Reinforce | SkillRevisionDisposition::Retain => {
                successes = successes.saturating_add(1);
            }

            SkillRevisionDisposition::Weaken | SkillRevisionDisposition::Suspend => {
                failures = failures.saturating_add(1);
            }

            SkillRevisionDisposition::Abstain => {}
        }

        SkillMemoryUseEvidence::new(
            evidence.access_count().saturating_add(1),
            successes,
            failures,
            evidence.recency_signal(),
        )
        .unwrap()
    }

    pub fn run(
        input: &IntegratedSkillLearningCycleInput,
        policy: IntegratedSkillLearningCyclePolicy,
    ) -> IntegratedSkillLearningCycleResult {
        let records = if input.memory().reusable() {
            vec![input.memory().record().clone()]
        } else {
            Vec::new()
        };

        let retrieval =
            SkillRetrievalAndReuse::retrieve(&records, input.reuse_request(), policy.reuse());

        let selected_plan = retrieval.plans().first().cloned();

        let (feedback, revision, updated_memory, updated_use_evidence) =
            if let Some(plan) = selected_plan.as_ref() {
                let feedback = SkillOutcomeFeedbackAndRevision::evaluate(
                    plan,
                    input.execution_observations(),
                    policy.feedback(),
                );

                let revision = SkillRevisionApplicationAndMemoryUpdate::apply(
                    input.memory(),
                    &feedback,
                    policy.revision(),
                );

                let use_evidence =
                    Self::update_use_evidence(input.use_evidence(), feedback.disposition());

                (
                    Some(feedback),
                    Some(revision.clone()),
                    revision.memory().clone(),
                    use_evidence,
                )
            } else {
                (
                    None,
                    None,
                    input.memory().clone(),
                    input.use_evidence().clone(),
                )
            };

        let consolidation_candidate = SkillMemoryConsolidationCandidate::new(
            updated_memory.clone(),
            updated_use_evidence.clone(),
        );

        let consolidation = SkillMemoryConsolidationAndForgetting::consolidate(
            &[consolidation_candidate],
            policy.consolidation(),
        );

        IntegratedSkillLearningCycleResult {
            retrieval,
            selected_plan,
            feedback,
            revision,
            updated_memory,
            updated_use_evidence,
            consolidation,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalIntegratedSkillLearningCycle;

impl UniversalIntegratedSkillLearningCycle {
    pub fn evaluate(
        input: &IntegratedSkillLearningCycleInput,
        policy: IntegratedSkillLearningCyclePolicy,
    ) -> IntegratedSkillLearningCycleResult {
        IntegratedSkillLearningCycle::run(input, policy)
    }
}

#[cfg(test)]
mod integrated_skill_learning_cycle_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn record() -> CompressedSkillRecord {
        CompressedSkillRecord {
            invariant_dictionary: vec![a(7)],
            initial_state: CompressedSkillTerm::StructuralSlot(0),
            goal_identity: CompressedSkillTerm::InvariantRef(0),
            steps: vec![
                CompressedSkillStep {
                    required_state: CompressedSkillTerm::StructuralSlot(0),
                    action: CompressedSkillTerm::StructuralSlot(1),
                    observed_outcome: CompressedSkillTerm::StructuralSlot(2),
                },
                CompressedSkillStep {
                    required_state: CompressedSkillTerm::StructuralSlot(2),
                    action: CompressedSkillTerm::InvariantRef(0),
                    observed_outcome: CompressedSkillTerm::InvariantRef(0),
                },
            ],
            structural_slot_count: 3,
            context_slot_count: 0,
            invariant_occurrence_count: 3,
            compression_gain: 2,
            source_generalization_count: 2,
            source_support_sum: 8,
            success_confidence_floor: s(900),
            step_confidence_floor: s(900),
        }
    }

    fn memory() -> SkillRevisionMemoryEntry {
        SkillRevisionMemoryEntry::new(record())
    }

    fn binding(id: usize, value: u64) -> GroundedSkillSlotBinding {
        GroundedSkillSlotBinding::new(SkillReuseSlotKind::Structural, id, a(value), s(900)).unwrap()
    }

    fn request() -> GroundedSkillReuseRequest {
        GroundedSkillReuseRequest::new(a(900), a(7), vec![binding(1, 910), binding(2, 1010)])
    }

    fn observation(
        state: u64,
        action: u64,
        outcome: u64,
        confidence: u16,
    ) -> SkillExecutionObservation {
        SkillExecutionObservation::new(a(state), a(action), a(outcome), s(confidence)).unwrap()
    }

    fn exact_observations() -> Vec<SkillExecutionObservation> {
        vec![
            observation(900, 910, 1010, 900),
            observation(1010, 7, 7, 900),
        ]
    }

    fn use_evidence(accesses: usize, successes: usize, failures: usize) -> SkillMemoryUseEvidence {
        SkillMemoryUseEvidence::new(accesses, successes, failures, s(900)).unwrap()
    }

    fn consolidation_thresholds() -> SkillMemoryConsolidationThresholds {
        SkillMemoryConsolidationThresholds::new(s(700), s(500), s(200), 3, 5).unwrap()
    }

    fn policy() -> IntegratedSkillLearningCyclePolicy {
        IntegratedSkillLearningCyclePolicy::new(
            SkillReusePolicy::new(
                SkillReuseBounds::new(16, 16, 16, 16, 1).unwrap(),
                SkillReuseThresholds::new(1, 1, s(500), s(500), s(500)).unwrap(),
            ),
            SkillOutcomeFeedbackPolicy::new(16, 16, s(500)).unwrap(),
            SkillRevisionMemoryPolicy::new(16, 200, 100, s(500), s(500)).unwrap(),
            SkillMemoryConsolidationPolicy::new(
                SkillMemoryConsolidationBounds::new(16, 16, 16, 16, 16, 16).unwrap(),
                consolidation_thresholds(),
            ),
        )
    }

    fn input(
        memory: SkillRevisionMemoryEntry,
        observations: Vec<SkillExecutionObservation>,
        usage: SkillMemoryUseEvidence,
    ) -> IntegratedSkillLearningCycleInput {
        IntegratedSkillLearningCycleInput::new(memory, request(), observations, usage)
    }

    fn suspension_feedback() -> SkillOutcomeFeedbackResult {
        SkillOutcomeFeedbackResult {
            input_observation_count: 2,
            considered_observation_count: 2,
            observation_frontier_truncated: false,
            evaluation_count: 2,
            evaluation_frontier_truncated: false,
            low_confidence_count: 0,
            exact_step_count: 0,
            execution_mismatch_count: 1,
            outcome_mismatch_count: 0,
            missing_plan_step_count: 0,
            extra_observation_count: 0,
            feedback_confidence_floor: Some(s(900)),
            disposition: SkillRevisionDisposition::Suspend,
        }
    }

    #[test]
    fn exact_success_closes_retrieval_feedback_revision_and_consolidation_loop() {
        let cycle = IntegratedSkillLearningCycle::run(
            &input(memory(), exact_observations(), use_evidence(9, 8, 0)),
            policy(),
        );

        assert!(cycle.reused_skill());

        assert_eq!(
            cycle.feedback().unwrap().disposition(),
            SkillRevisionDisposition::Reinforce
        );

        assert!(cycle.revised_memory());

        assert_eq!(cycle.updated_use_evidence().access_count(), 10);

        assert_eq!(cycle.updated_use_evidence().successful_reuse_count(), 9);

        assert_eq!(cycle.consolidation().hot().len(), 1);
    }

    #[test]
    fn prediction_error_flows_into_persistent_weakening_and_failure_usage() {
        let observations = vec![
            observation(900, 910, 999, 900),
            observation(1010, 7, 7, 900),
        ];

        let cycle = IntegratedSkillLearningCycle::run(
            &input(memory(), observations, use_evidence(5, 4, 0)),
            policy(),
        );

        assert_eq!(
            cycle.feedback().unwrap().disposition(),
            SkillRevisionDisposition::Weaken
        );

        assert_eq!(cycle.updated_memory().revision_confidence_cap(), s(700));

        assert_eq!(cycle.updated_use_evidence().failed_reuse_count(), 1);
    }

    #[test]
    fn execution_mismatch_suspends_memory_and_moves_it_out_of_active_tiers() {
        let observations = vec![
            observation(901, 910, 1010, 900),
            observation(1010, 7, 7, 900),
        ];

        let cycle = IntegratedSkillLearningCycle::run(
            &input(memory(), observations, use_evidence(5, 4, 0)),
            policy(),
        );

        assert_eq!(
            cycle.updated_memory().availability(),
            SkillMemoryAvailability::Suspended
        );

        assert!(cycle.consolidation().hot().is_empty());

        assert!(cycle.consolidation().warm().is_empty());

        assert_eq!(cycle.consolidation().cold().len(), 1);
    }

    #[test]
    fn unresolved_retrieval_abstains_without_feedback_or_memory_mutation() {
        let no_bindings = GroundedSkillReuseRequest::new(a(900), a(7), Vec::new());

        let original = memory();

        let input = IntegratedSkillLearningCycleInput::new(
            original.clone(),
            no_bindings,
            exact_observations(),
            use_evidence(4, 3, 0),
        );

        let cycle = IntegratedSkillLearningCycle::run(&input, policy());

        assert!(!cycle.reused_skill());

        assert!(cycle.feedback().is_none());

        assert!(cycle.revision().is_none());

        assert_eq!(cycle.updated_memory(), &original);

        assert_eq!(cycle.updated_use_evidence(), input.use_evidence());
    }

    #[test]
    fn suspended_memory_is_excluded_before_retrieval() {
        let suspended = SkillRevisionApplicationAndMemoryUpdate::apply(
            &memory(),
            &suspension_feedback(),
            SkillRevisionMemoryPolicy::new(8, 100, 100, s(500), s(500)).unwrap(),
        )
        .memory()
        .clone();

        let cycle = IntegratedSkillLearningCycle::run(
            &input(
                suspended.clone(),
                exact_observations(),
                use_evidence(5, 4, 0),
            ),
            policy(),
        );

        assert_eq!(cycle.retrieval().input_record_count(), 0);

        assert!(!cycle.reused_skill());

        assert_eq!(cycle.updated_memory(), &suspended);
    }

    #[test]
    fn low_confidence_feedback_does_not_modify_memory_but_records_access() {
        let observations = vec![
            observation(900, 910, 1010, 400),
            observation(1010, 7, 7, 900),
        ];

        let original = memory();

        let cycle = IntegratedSkillLearningCycle::run(
            &input(original.clone(), observations, use_evidence(5, 4, 0)),
            policy(),
        );

        assert_eq!(
            cycle.feedback().unwrap().disposition(),
            SkillRevisionDisposition::Abstain
        );

        assert!(!cycle.revised_memory());

        assert_eq!(cycle.updated_memory(), &original);

        assert_eq!(cycle.updated_use_evidence().access_count(), 6);

        assert_eq!(cycle.updated_use_evidence().successful_reuse_count(), 4);

        assert_eq!(cycle.updated_use_evidence().failed_reuse_count(), 0);
    }

    #[test]
    fn revision_budget_exhaustion_survives_integrated_cycle() {
        let limited_policy = SkillRevisionMemoryPolicy::new(1, 200, 100, s(500), s(500)).unwrap();

        let weakened_feedback = SkillOutcomeFeedbackResult {
            input_observation_count: 2,
            considered_observation_count: 2,
            observation_frontier_truncated: false,
            evaluation_count: 2,
            evaluation_frontier_truncated: false,
            low_confidence_count: 0,
            exact_step_count: 1,
            execution_mismatch_count: 0,
            outcome_mismatch_count: 1,
            missing_plan_step_count: 0,
            extra_observation_count: 0,
            feedback_confidence_floor: Some(s(900)),
            disposition: SkillRevisionDisposition::Weaken,
        };

        let exhausted = SkillRevisionApplicationAndMemoryUpdate::apply(
            &memory(),
            &weakened_feedback,
            limited_policy,
        )
        .memory()
        .clone();

        let p = IntegratedSkillLearningCyclePolicy::new(
            policy().reuse(),
            policy().feedback(),
            limited_policy,
            policy().consolidation(),
        );

        let cycle = IntegratedSkillLearningCycle::run(
            &input(
                exhausted.clone(),
                exact_observations(),
                use_evidence(5, 3, 1),
            ),
            p,
        );

        assert!(cycle.revision().unwrap().revision_budget_exhausted());

        assert_eq!(cycle.updated_memory(), &exhausted);
    }

    #[test]
    fn integrated_cycle_never_mutates_compressed_skill_provenance() {
        let original = memory();

        let provenance = original.record().clone();

        let observations = vec![
            observation(900, 910, 999, 900),
            observation(1010, 7, 7, 900),
        ];

        let cycle = IntegratedSkillLearningCycle::run(
            &input(original, observations, use_evidence(5, 4, 0)),
            policy(),
        );

        assert_eq!(cycle.updated_memory().record(), &provenance);
    }

    #[test]
    fn consolidation_consumes_the_post_revision_memory_state() {
        let observations = vec![
            observation(901, 910, 1010, 900),
            observation(1010, 7, 7, 900),
        ];

        let cycle = IntegratedSkillLearningCycle::run(
            &input(memory(), observations, use_evidence(10, 9, 0)),
            policy(),
        );

        assert_eq!(
            cycle.updated_memory().availability(),
            SkillMemoryAvailability::Suspended
        );

        assert_eq!(
            cycle.consolidation().cold()[0].memory().availability(),
            SkillMemoryAvailability::Suspended
        );
    }

    #[test]
    fn retrieval_step_bound_prevents_downstream_revision() {
        let bounded_reuse = SkillReusePolicy::new(
            SkillReuseBounds::new(16, 16, 16, 1, 1).unwrap(),
            policy().reuse().thresholds(),
        );

        let p = IntegratedSkillLearningCyclePolicy::new(
            bounded_reuse,
            policy().feedback(),
            policy().revision(),
            policy().consolidation(),
        );

        let original = memory();

        let cycle = IntegratedSkillLearningCycle::run(
            &input(
                original.clone(),
                exact_observations(),
                use_evidence(5, 4, 0),
            ),
            p,
        );

        assert_eq!(cycle.retrieval().rejected_step_bound_count(), 1);

        assert!(cycle.feedback().is_none());

        assert!(cycle.revision().is_none());

        assert_eq!(cycle.updated_memory(), &original);
    }

    #[test]
    fn repeated_failures_can_flow_through_cycle_into_forgetting() {
        let observations = vec![
            observation(900, 910, 999, 900),
            observation(1010, 7, 7, 900),
        ];

        let cycle = IntegratedSkillLearningCycle::run(
            &input(memory(), observations, use_evidence(9, 4, 4)),
            policy(),
        );

        assert_eq!(cycle.updated_use_evidence().failed_reuse_count(), 5);

        assert_eq!(cycle.consolidation().forgotten_count(), 1);

        assert_eq!(
            cycle.consolidation().forgotten_archive()[0].tier(),
            ConsolidatedSkillMemoryTier::Forgotten
        );
    }

    #[test]
    fn integrated_cycle_is_deterministic_non_mutating_and_facade_equivalent() {
        let input = input(memory(), exact_observations(), use_evidence(9, 8, 0));

        let before = input.clone();

        let p = policy();

        let direct = IntegratedSkillLearningCycle::run(&input, p);

        let facade = UniversalIntegratedSkillLearningCycle::evaluate(&input, p);

        let repeated = UniversalIntegratedSkillLearningCycle::evaluate(&input, p);

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(input, before);
    }
}

// ============================================================================
// T0-A — GROUNDED AUTONOMOUS SKILL CORRESPONDENCE
// ============================================================================
//
// A compressed skill may contain structural and contextual slots whose values
// are unknown in a novel grounded situation. Correspondence inference binds
// those slots only from observed execution evidence and explicit state/goal
// anchors. Conflicting evidence causes abstention rather than arbitrary
// correspondence selection.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroundedSkillCorrespondencePolicy {
    max_records: usize,
    max_observations: usize,
    max_bindings: usize,
    minimum_observation_confidence: CognitiveSignal,
}

impl GroundedSkillCorrespondencePolicy {
    pub fn new(
        max_records: usize,
        max_observations: usize,
        max_bindings: usize,
        minimum_observation_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if max_records == 0
            || max_observations == 0
            || max_bindings == 0
            || minimum_observation_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            max_records,
            max_observations,
            max_bindings,
            minimum_observation_confidence,
        })
    }

    pub fn max_records(self) -> usize {
        self.max_records
    }

    pub fn max_observations(self) -> usize {
        self.max_observations
    }

    pub fn max_bindings(self) -> usize {
        self.max_bindings
    }

    pub fn minimum_observation_confidence(self) -> CognitiveSignal {
        self.minimum_observation_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedSkillCorrespondenceResult {
    input_record_count: usize,
    considered_record_count: usize,
    record_frontier_exceeded: bool,
    input_observation_count: usize,
    considered_observation_count: usize,
    rejected_low_confidence_count: usize,
    conflicting_evidence: bool,
    requests: Vec<GroundedSkillReuseRequest>,
}

impl GroundedSkillCorrespondenceResult {
    pub fn input_record_count(&self) -> usize {
        self.input_record_count
    }

    pub fn considered_record_count(&self) -> usize {
        self.considered_record_count
    }

    pub fn record_frontier_exceeded(&self) -> bool {
        self.record_frontier_exceeded
    }

    pub fn input_observation_count(&self) -> usize {
        self.input_observation_count
    }

    pub fn considered_observation_count(&self) -> usize {
        self.considered_observation_count
    }

    pub fn rejected_low_confidence_count(&self) -> usize {
        self.rejected_low_confidence_count
    }

    pub fn conflicting_evidence(&self) -> bool {
        self.conflicting_evidence
    }

    pub fn requests(&self) -> &[GroundedSkillReuseRequest] {
        &self.requests
    }

    pub fn request_count(&self) -> usize {
        self.requests.len()
    }

    pub fn abstained(&self) -> bool {
        self.requests.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GroundedCorrespondenceBinding {
    kind: SkillReuseSlotKind,
    id: usize,
    value: CognitiveStructure,
    confidence: CognitiveSignal,
}

pub struct GroundedSkillCorrespondenceInference;

impl GroundedSkillCorrespondenceInference {
    fn floor(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        if left.value() <= right.value() {
            left
        } else {
            right
        }
    }

    fn bind_term(
        bindings: &mut Vec<GroundedCorrespondenceBinding>,
        term: &CompressedSkillTerm,
        value: &CognitiveStructure,
        confidence: CognitiveSignal,
        max_bindings: usize,
    ) -> bool {
        let (kind, id) = match term {
            CompressedSkillTerm::StructuralSlot(id) => (SkillReuseSlotKind::Structural, *id),
            CompressedSkillTerm::ContextSlot(id) => (SkillReuseSlotKind::Context, *id),
            CompressedSkillTerm::InvariantRef(_) => return true,
        };

        if let Some(existing) = bindings
            .iter_mut()
            .find(|binding| binding.kind == kind && binding.id == id)
        {
            if existing.value != *value {
                return false;
            }

            existing.confidence = Self::floor(existing.confidence, confidence);
            return true;
        }

        if bindings.len() >= max_bindings {
            return false;
        }

        bindings.push(GroundedCorrespondenceBinding {
            kind,
            id,
            value: value.clone(),
            confidence,
        });

        true
    }

    fn binding_order(
        left: &GroundedCorrespondenceBinding,
        right: &GroundedCorrespondenceBinding,
    ) -> std::cmp::Ordering {
        let left_kind = match left.kind {
            SkillReuseSlotKind::Structural => 0usize,
            SkillReuseSlotKind::Context => 1usize,
        };

        let right_kind = match right.kind {
            SkillReuseSlotKind::Structural => 0usize,
            SkillReuseSlotKind::Context => 1usize,
        };

        left_kind
            .cmp(&right_kind)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| format!("{:?}", left.value).cmp(&format!("{:?}", right.value)))
    }

    pub fn infer(
        records: &[CompressedSkillRecord],
        current_state: &CognitiveStructure,
        goal_identity: &CognitiveStructure,
        observations: &[SkillExecutionObservation],
        policy: GroundedSkillCorrespondencePolicy,
    ) -> GroundedSkillCorrespondenceResult {
        let input_record_count = records.len();
        let input_observation_count = observations.len();

        let mut canonical_records: Vec<_> = records.iter().collect();

        canonical_records
            .sort_by(|left, right| LossControlledSkillCompression::record_order(left, right));

        let record_frontier_exceeded = canonical_records.len() > policy.max_records();

        canonical_records.truncate(policy.max_records());

        let considered_records = canonical_records;
        let considered_record_count = considered_records.len();

        struct GroundedPrefixCorrespondenceCluster {
            action: CognitiveStructure,
            outcome: CognitiveStructure,
            support_count: usize,
            confidence_floor: CognitiveSignal,
        }

        let mut rejected_low_confidence_count = 0usize;
        let mut clusters: Vec<GroundedPrefixCorrespondenceCluster> = Vec::new();
        let mut cluster_frontier_exceeded = false;

        for observation in observations {
            if observation.evidence_confidence().value()
                < policy.minimum_observation_confidence().value()
            {
                rejected_low_confidence_count = rejected_low_confidence_count.saturating_add(1);
                continue;
            }

            if observation.required_state() != current_state {
                continue;
            }

            if let Some(existing) = clusters.iter_mut().find(|cluster| {
                cluster.action == *observation.action()
                    && cluster.outcome == *observation.observed_outcome()
            }) {
                existing.support_count = existing.support_count.saturating_add(1);
                existing.confidence_floor =
                    Self::floor(existing.confidence_floor, observation.evidence_confidence());
                continue;
            }

            if clusters.len() >= policy.max_observations() {
                cluster_frontier_exceeded = true;
                continue;
            }

            clusters.push(GroundedPrefixCorrespondenceCluster {
                action: observation.action().clone(),
                outcome: observation.observed_outcome().clone(),
                support_count: 1,
                confidence_floor: observation.evidence_confidence(),
            });
        }

        // Canonical structural order is established before representative
        // allocation so input presentation order cannot influence tie-breaking.
        clusters.sort_by(|left, right| {
            left.action
                .cmp(&right.action)
                .then_with(|| left.outcome.cmp(&right.outcome))
        });

        let total_qualified_support = clusters
            .iter()
            .map(|cluster| cluster.support_count)
            .sum::<usize>();

        let representative_budget = policy.max_observations().min(total_qualified_support);

        if !cluster_frontier_exceeded
            && !clusters.is_empty()
            && representative_budget >= clusters.len()
        {
            // Every observed alternative receives one representative first.
            // Remaining capacity is apportioned according to reproducible
            // support, preserving contrasts instead of taking an input prefix.
            let mandatory_support = clusters.len();
            let extra_budget = representative_budget.saturating_sub(mandatory_support);
            let total_extra_support = total_qualified_support.saturating_sub(mandatory_support);

            let mut retained_support = vec![1usize; clusters.len()];

            if extra_budget > 0 && total_extra_support > 0 {
                let mut allocated_extra = 0usize;
                let mut remainders = Vec::with_capacity(clusters.len());

                for (index, cluster) in clusters.iter().enumerate() {
                    let extra_support = cluster.support_count.saturating_sub(1);
                    let numerator = extra_support.saturating_mul(extra_budget);

                    let allocated = numerator / total_extra_support;
                    let remainder = numerator % total_extra_support;

                    retained_support[index] = retained_support[index].saturating_add(allocated);

                    allocated_extra = allocated_extra.saturating_add(allocated);

                    remainders.push((index, remainder));
                }

                let leftover = extra_budget.saturating_sub(allocated_extra);

                remainders
                    .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

                for (index, _) in remainders.into_iter().take(leftover) {
                    retained_support[index] = retained_support[index].saturating_add(1);
                }
            }

            for (cluster, retained) in clusters.iter_mut().zip(retained_support) {
                cluster.support_count = retained;
            }
        }

        let considered_observation_count = if cluster_frontier_exceeded {
            policy.max_observations()
        } else {
            clusters
                .iter()
                .map(|cluster| cluster.support_count)
                .sum::<usize>()
        };

        clusters.sort_by(|left, right| {
            right
                .support_count
                .cmp(&left.support_count)
                .then_with(|| left.action.cmp(&right.action))
                .then_with(|| left.outcome.cmp(&right.outcome))
        });

        let conflicting_evidence = cluster_frontier_exceeded
            || clusters
                .get(1)
                .is_some_and(|runner_up| runner_up.support_count == clusters[0].support_count);

        let selected_cluster = if conflicting_evidence {
            None
        } else {
            clusters.first()
        };

        let mut requests = Vec::new();

        if !record_frontier_exceeded {
            if let Some(cluster) = selected_cluster {
                for record in considered_records {
                    let Some(first_step) = record.steps().first() else {
                        continue;
                    };

                    let confidence = cluster.confidence_floor;
                    let mut bindings = Vec::new();

                    let consistent = Self::bind_term(
                        &mut bindings,
                        record.initial_state(),
                        current_state,
                        confidence,
                        policy.max_bindings(),
                    ) && Self::bind_term(
                        &mut bindings,
                        record.goal_identity(),
                        goal_identity,
                        confidence,
                        policy.max_bindings(),
                    ) && Self::bind_term(
                        &mut bindings,
                        first_step.required_state(),
                        current_state,
                        confidence,
                        policy.max_bindings(),
                    ) && Self::bind_term(
                        &mut bindings,
                        first_step.action(),
                        &cluster.action,
                        confidence,
                        policy.max_bindings(),
                    ) && Self::bind_term(
                        &mut bindings,
                        first_step.observed_outcome(),
                        &cluster.outcome,
                        confidence,
                        policy.max_bindings(),
                    );

                    if !consistent || bindings.is_empty() {
                        continue;
                    }

                    bindings.sort_by(Self::binding_order);

                    let grounded: Vec<_> = bindings
                        .into_iter()
                        .filter_map(|binding| {
                            GroundedSkillSlotBinding::new(
                                binding.kind,
                                binding.id,
                                binding.value,
                                binding.confidence,
                            )
                        })
                        .collect();

                    if grounded.is_empty() {
                        continue;
                    }

                    requests.push(GroundedSkillReuseRequest::new(
                        current_state.clone(),
                        goal_identity.clone(),
                        grounded,
                    ));
                }
            }
        }

        GroundedSkillCorrespondenceResult {
            input_record_count,
            considered_record_count,
            record_frontier_exceeded,
            input_observation_count,
            considered_observation_count,
            rejected_low_confidence_count,
            conflicting_evidence,
            requests,
        }
    }
}

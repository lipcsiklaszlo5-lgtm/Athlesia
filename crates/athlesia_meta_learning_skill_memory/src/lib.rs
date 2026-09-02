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

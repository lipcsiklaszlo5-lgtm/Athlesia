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

use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardSummary,
};
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ArcAgi3BlindBenchmarkRunId(String);

impl ArcAgi3BlindBenchmarkRunId {
    pub fn new(value: String) -> Result<Self, ArcAgi3BlindBenchmarkFoundationError> {
        if value.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkFoundationError::EmptyRunId);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkAgentIdentity {
    name: String,
    version: String,
    source_revision: String,
}

impl ArcAgi3BlindBenchmarkAgentIdentity {
    pub fn new(
        name: String,
        version: String,
        source_revision: String,
    ) -> Result<Self, ArcAgi3BlindBenchmarkFoundationError> {
        if name.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkFoundationError::EmptyAgentName);
        }

        if version.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkFoundationError::EmptyAgentVersion);
        }

        if source_revision.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkFoundationError::EmptySourceRevision);
        }

        Ok(Self {
            name,
            version,
            source_revision,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkPolicy {
    max_cognitive_steps_per_episode: usize,
}

impl ArcAgi3BlindBenchmarkPolicy {
    pub fn new(max_cognitive_steps_per_episode: usize) -> Option<Self> {
        if max_cognitive_steps_per_episode == 0 {
            return None;
        }

        Some(Self {
            max_cognitive_steps_per_episode,
        })
    }

    pub fn max_cognitive_steps_per_episode(self) -> usize {
        self.max_cognitive_steps_per_episode
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkSpec {
    run_id: ArcAgi3BlindBenchmarkRunId,
    agent: ArcAgi3BlindBenchmarkAgentIdentity,
    policy: ArcAgi3BlindBenchmarkPolicy,
}

impl ArcAgi3BlindBenchmarkSpec {
    pub fn new(
        run_id: ArcAgi3BlindBenchmarkRunId,
        agent: ArcAgi3BlindBenchmarkAgentIdentity,
        policy: ArcAgi3BlindBenchmarkPolicy,
    ) -> Self {
        Self {
            run_id,
            agent,
            policy,
        }
    }

    pub fn run_id(&self) -> &ArcAgi3BlindBenchmarkRunId {
        &self.run_id
    }

    pub fn agent(&self) -> &ArcAgi3BlindBenchmarkAgentIdentity {
        &self.agent
    }

    pub fn policy(&self) -> ArcAgi3BlindBenchmarkPolicy {
        self.policy
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkEpisodeObservation {
    episode_index: usize,
    game_id: ArcAgi3GameId,
    termination: ArcAgi3BoundedEpisodeTermination,
    completed_cognitive_steps: usize,
}

impl ArcAgi3BlindBenchmarkEpisodeObservation {
    pub fn episode_index(&self) -> usize {
        self.episode_index
    }

    pub fn game_id(&self) -> &ArcAgi3GameId {
        &self.game_id
    }

    pub fn termination(&self) -> ArcAgi3BoundedEpisodeTermination {
        self.termination
    }

    pub fn completed_cognitive_steps(&self) -> usize {
        self.completed_cognitive_steps
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3BlindBenchmarkStatus {
    Recording,
    Finalized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArcAgi3BlindBenchmarkFoundationError {
    EmptyRunId,
    EmptyAgentName,
    EmptyAgentVersion,
    EmptySourceRevision,
    BenchmarkAlreadyFinalized,
    CardIdentityMismatch { expected: String, observed: String },
    CompetitionModeMismatch,
}

#[derive(Debug)]
pub struct ArcAgi3BlindBenchmarkLedger {
    spec: ArcAgi3BlindBenchmarkSpec,
    card_id: ArcAgi3ScorecardId,
    episodes: Vec<ArcAgi3BlindBenchmarkEpisodeObservation>,
    final_summary: Option<ArcAgi3ScorecardSummary>,
}

impl ArcAgi3BlindBenchmarkLedger {
    pub fn new(spec: ArcAgi3BlindBenchmarkSpec, card_id: ArcAgi3ScorecardId) -> Self {
        Self {
            spec,
            card_id,
            episodes: Vec::new(),
            final_summary: None,
        }
    }

    pub fn spec(&self) -> &ArcAgi3BlindBenchmarkSpec {
        &self.spec
    }

    pub fn card_id(&self) -> &ArcAgi3ScorecardId {
        &self.card_id
    }

    pub fn status(&self) -> ArcAgi3BlindBenchmarkStatus {
        if self.final_summary.is_some() {
            ArcAgi3BlindBenchmarkStatus::Finalized
        } else {
            ArcAgi3BlindBenchmarkStatus::Recording
        }
    }

    pub fn episodes(&self) -> &[ArcAgi3BlindBenchmarkEpisodeObservation] {
        &self.episodes
    }

    pub fn final_summary(&self) -> Option<&ArcAgi3ScorecardSummary> {
        self.final_summary.as_ref()
    }

    pub fn server_score(&self) -> Option<f64> {
        self.final_summary
            .as_ref()
            .map(ArcAgi3ScorecardSummary::score)
    }

    pub fn record_episode(
        &mut self,
        game_id: ArcAgi3GameId,
        termination: ArcAgi3BoundedEpisodeTermination,
        completed_cognitive_steps: usize,
    ) -> Result<&ArcAgi3BlindBenchmarkEpisodeObservation, ArcAgi3BlindBenchmarkFoundationError>
    {
        if self.final_summary.is_some() {
            return Err(ArcAgi3BlindBenchmarkFoundationError::BenchmarkAlreadyFinalized);
        }

        let episode_index = self.episodes.len();

        self.episodes.push(ArcAgi3BlindBenchmarkEpisodeObservation {
            episode_index,
            game_id,
            termination,
            completed_cognitive_steps,
        });

        Ok(&self.episodes[episode_index])
    }

    pub fn finalize(
        &mut self,
        summary: ArcAgi3ScorecardSummary,
    ) -> Result<&ArcAgi3ScorecardSummary, ArcAgi3BlindBenchmarkFoundationError> {
        if self.final_summary.is_some() {
            return Err(ArcAgi3BlindBenchmarkFoundationError::BenchmarkAlreadyFinalized);
        }

        if summary.card_id() != &self.card_id {
            return Err(ArcAgi3BlindBenchmarkFoundationError::CardIdentityMismatch {
                expected: self.card_id.as_str().to_string(),
                observed: summary.card_id().as_str().to_string(),
            });
        }

        if summary.competition_mode() == Some(false) {
            return Err(ArcAgi3BlindBenchmarkFoundationError::CompetitionModeMismatch);
        }

        self.final_summary = Some(summary);

        Ok(self
            .final_summary
            .as_ref()
            .expect("summary was stored immediately before access"))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3BlindBenchmarkFoundation;

impl UniversalArcAgi3BlindBenchmarkFoundation {
    pub fn ledger(
        spec: ArcAgi3BlindBenchmarkSpec,
        card_id: ArcAgi3ScorecardId,
    ) -> ArcAgi3BlindBenchmarkLedger {
        ArcAgi3BlindBenchmarkLedger::new(spec, card_id)
    }
}

pub mod execution_runtime;

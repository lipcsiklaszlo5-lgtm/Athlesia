use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::ArcAgi3ScorecardSummary;
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;

use crate::{ArcAgi3BlindBenchmarkFoundationError, ArcAgi3BlindBenchmarkLedger};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkExecutionRequest {
    episode_index: usize,
    max_cognitive_steps_per_episode: usize,
}

impl ArcAgi3BlindBenchmarkExecutionRequest {
    fn new(episode_index: usize, max_cognitive_steps_per_episode: usize) -> Self {
        Self {
            episode_index,
            max_cognitive_steps_per_episode,
        }
    }

    pub fn episode_index(self) -> usize {
        self.episode_index
    }

    pub fn max_cognitive_steps_per_episode(self) -> usize {
        self.max_cognitive_steps_per_episode
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkExecutedEpisode {
    game_id: ArcAgi3GameId,
    termination: ArcAgi3BoundedEpisodeTermination,
    completed_cognitive_steps: usize,
}

impl ArcAgi3BlindBenchmarkExecutedEpisode {
    pub fn new(
        game_id: ArcAgi3GameId,
        termination: ArcAgi3BoundedEpisodeTermination,
        completed_cognitive_steps: usize,
    ) -> Self {
        Self {
            game_id,
            termination,
            completed_cognitive_steps,
        }
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

#[derive(Debug)]
pub enum ArcAgi3BlindBenchmarkHarnessEvent {
    Episode(ArcAgi3BlindBenchmarkExecutedEpisode),
    Finalized(ArcAgi3ScorecardSummary),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3BlindBenchmarkExecutionStatus {
    Ready,
    Running,
    Faulted,
    Finalized,
}

#[derive(Debug)]
pub enum ArcAgi3BlindBenchmarkExecutionError<E> {
    RuntimeNotReady(ArcAgi3BlindBenchmarkExecutionStatus),
    EpisodeStepBudgetExceeded {
        episode_index: usize,
        observed_steps: usize,
        maximum_steps: usize,
    },
    Harness(E),
    Foundation(ArcAgi3BlindBenchmarkFoundationError),
}

pub struct ArcAgi3BlindBenchmarkExecutionRuntime {
    ledger: ArcAgi3BlindBenchmarkLedger,
    status: ArcAgi3BlindBenchmarkExecutionStatus,
}

impl ArcAgi3BlindBenchmarkExecutionRuntime {
    pub fn new(ledger: ArcAgi3BlindBenchmarkLedger) -> Self {
        Self {
            ledger,
            status: ArcAgi3BlindBenchmarkExecutionStatus::Ready,
        }
    }

    pub fn ledger(&self) -> &ArcAgi3BlindBenchmarkLedger {
        &self.ledger
    }

    pub fn status(&self) -> ArcAgi3BlindBenchmarkExecutionStatus {
        self.status
    }

    pub fn run_with<E, F>(
        &mut self,
        mut harness: F,
    ) -> Result<&ArcAgi3ScorecardSummary, ArcAgi3BlindBenchmarkExecutionError<E>>
    where
        F: FnMut(
            ArcAgi3BlindBenchmarkExecutionRequest,
        ) -> Result<ArcAgi3BlindBenchmarkHarnessEvent, E>,
    {
        if self.status != ArcAgi3BlindBenchmarkExecutionStatus::Ready {
            return Err(ArcAgi3BlindBenchmarkExecutionError::RuntimeNotReady(
                self.status,
            ));
        }

        self.status = ArcAgi3BlindBenchmarkExecutionStatus::Running;

        loop {
            let episode_index = self.ledger.episodes().len();

            let maximum_steps = self
                .ledger
                .spec()
                .policy()
                .max_cognitive_steps_per_episode();

            let request = ArcAgi3BlindBenchmarkExecutionRequest::new(episode_index, maximum_steps);

            let event = match harness(request) {
                Ok(event) => event,
                Err(error) => {
                    self.status = ArcAgi3BlindBenchmarkExecutionStatus::Faulted;

                    return Err(ArcAgi3BlindBenchmarkExecutionError::Harness(error));
                }
            };

            match event {
                ArcAgi3BlindBenchmarkHarnessEvent::Episode(episode) => {
                    let observed_steps = episode.completed_cognitive_steps();

                    if observed_steps > maximum_steps {
                        self.status = ArcAgi3BlindBenchmarkExecutionStatus::Faulted;

                        return Err(
                            ArcAgi3BlindBenchmarkExecutionError::EpisodeStepBudgetExceeded {
                                episode_index,
                                observed_steps,
                                maximum_steps,
                            },
                        );
                    }

                    let result = self.ledger.record_episode(
                        episode.game_id,
                        episode.termination,
                        observed_steps,
                    );

                    if let Err(error) = result {
                        self.status = ArcAgi3BlindBenchmarkExecutionStatus::Faulted;

                        return Err(ArcAgi3BlindBenchmarkExecutionError::Foundation(error));
                    }
                }

                ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary) => {
                    if let Err(error) = self.ledger.finalize(summary) {
                        self.status = ArcAgi3BlindBenchmarkExecutionStatus::Faulted;

                        return Err(ArcAgi3BlindBenchmarkExecutionError::Foundation(error));
                    }

                    self.status = ArcAgi3BlindBenchmarkExecutionStatus::Finalized;

                    return Ok(self
                        .ledger
                        .final_summary()
                        .expect("ledger finalization succeeded immediately before access"));
                }
            }
        }
    }

    pub fn into_ledger(self) -> ArcAgi3BlindBenchmarkLedger {
        self.ledger
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3BlindBenchmarkExecutionRuntime;

impl UniversalArcAgi3BlindBenchmarkExecutionRuntime {
    pub fn runtime(ledger: ArcAgi3BlindBenchmarkLedger) -> ArcAgi3BlindBenchmarkExecutionRuntime {
        ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger)
    }

    pub fn run_with<E, F>(
        runtime: &mut ArcAgi3BlindBenchmarkExecutionRuntime,
        harness: F,
    ) -> Result<&ArcAgi3ScorecardSummary, ArcAgi3BlindBenchmarkExecutionError<E>>
    where
        F: FnMut(
            ArcAgi3BlindBenchmarkExecutionRequest,
        ) -> Result<ArcAgi3BlindBenchmarkHarnessEvent, E>,
    {
        runtime.run_with(harness)
    }
}

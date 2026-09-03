use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::ArcAgi3ScorecardSummary;
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;

use crate::execution_runtime::{
    ArcAgi3BlindBenchmarkExecutedEpisode, ArcAgi3BlindBenchmarkExecutionError,
    ArcAgi3BlindBenchmarkExecutionRequest, ArcAgi3BlindBenchmarkExecutionRuntime,
    ArcAgi3BlindBenchmarkHarnessEvent,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkHarnessRequest {
    episode_index: usize,
    max_cognitive_steps_per_episode: usize,
}

impl ArcAgi3BlindBenchmarkHarnessRequest {
    pub fn from_execution_request(request: ArcAgi3BlindBenchmarkExecutionRequest) -> Self {
        Self {
            episode_index: request.episode_index(),
            max_cognitive_steps_per_episode: request.max_cognitive_steps_per_episode(),
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
pub struct ArcAgi3BlindBenchmarkHarnessEpisode {
    request_episode_index: usize,
    request_max_cognitive_steps_per_episode: usize,
    game_id: ArcAgi3GameId,
    termination: ArcAgi3BoundedEpisodeTermination,
    completed_cognitive_steps: usize,
}

impl ArcAgi3BlindBenchmarkHarnessEpisode {
    pub fn new(
        request_episode_index: usize,
        request_max_cognitive_steps_per_episode: usize,
        game_id: ArcAgi3GameId,
        termination: ArcAgi3BoundedEpisodeTermination,
        completed_cognitive_steps: usize,
    ) -> Self {
        Self {
            request_episode_index,
            request_max_cognitive_steps_per_episode,
            game_id,
            termination,
            completed_cognitive_steps,
        }
    }

    pub fn request_episode_index(&self) -> usize {
        self.request_episode_index
    }

    pub fn request_max_cognitive_steps_per_episode(&self) -> usize {
        self.request_max_cognitive_steps_per_episode
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
pub struct ArcAgi3BlindBenchmarkHarnessFinalization {
    request_episode_index: usize,
    request_max_cognitive_steps_per_episode: usize,
    summary: ArcAgi3ScorecardSummary,
}

impl ArcAgi3BlindBenchmarkHarnessFinalization {
    pub fn new(
        request_episode_index: usize,
        request_max_cognitive_steps_per_episode: usize,
        summary: ArcAgi3ScorecardSummary,
    ) -> Self {
        Self {
            request_episode_index,
            request_max_cognitive_steps_per_episode,
            summary,
        }
    }

    pub fn request_episode_index(&self) -> usize {
        self.request_episode_index
    }

    pub fn request_max_cognitive_steps_per_episode(&self) -> usize {
        self.request_max_cognitive_steps_per_episode
    }

    pub fn summary(&self) -> &ArcAgi3ScorecardSummary {
        &self.summary
    }
}

#[derive(Debug)]
pub enum ArcAgi3BlindBenchmarkHarnessResponse {
    Episode(ArcAgi3BlindBenchmarkHarnessEpisode),
    Finalized(ArcAgi3BlindBenchmarkHarnessFinalization),
}

pub trait ArcAgi3BlindBenchmarkExternalHarness {
    type Error;

    fn next(
        &mut self,
        request: ArcAgi3BlindBenchmarkHarnessRequest,
    ) -> Result<ArcAgi3BlindBenchmarkHarnessResponse, Self::Error>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum ArcAgi3BlindBenchmarkHarnessBridgeError<E> {
    Harness(E),
    EpisodeIndexMismatch { expected: usize, observed: usize },
    StepBudgetMismatch { expected: usize, observed: usize },
}

pub struct ArcAgi3BlindBenchmarkHarnessBridge<H> {
    harness: H,
}

impl<H> ArcAgi3BlindBenchmarkHarnessBridge<H>
where
    H: ArcAgi3BlindBenchmarkExternalHarness,
{
    pub fn new(harness: H) -> Self {
        Self { harness }
    }

    pub fn harness(&self) -> &H {
        &self.harness
    }

    pub fn into_harness(self) -> H {
        self.harness
    }

    pub fn next_event(
        &mut self,
        execution_request: ArcAgi3BlindBenchmarkExecutionRequest,
    ) -> Result<ArcAgi3BlindBenchmarkHarnessEvent, ArcAgi3BlindBenchmarkHarnessBridgeError<H::Error>>
    {
        let expected_episode_index = execution_request.episode_index();

        let expected_step_budget = execution_request.max_cognitive_steps_per_episode();

        let harness_request =
            ArcAgi3BlindBenchmarkHarnessRequest::from_execution_request(execution_request);

        let response = self
            .harness
            .next(harness_request)
            .map_err(ArcAgi3BlindBenchmarkHarnessBridgeError::Harness)?;

        match response {
            ArcAgi3BlindBenchmarkHarnessResponse::Episode(episode) => {
                if episode.request_episode_index() != expected_episode_index {
                    return Err(
                        ArcAgi3BlindBenchmarkHarnessBridgeError::EpisodeIndexMismatch {
                            expected: expected_episode_index,
                            observed: episode.request_episode_index(),
                        },
                    );
                }

                if episode.request_max_cognitive_steps_per_episode() != expected_step_budget {
                    return Err(
                        ArcAgi3BlindBenchmarkHarnessBridgeError::StepBudgetMismatch {
                            expected: expected_step_budget,
                            observed: episode.request_max_cognitive_steps_per_episode(),
                        },
                    );
                }

                Ok(ArcAgi3BlindBenchmarkHarnessEvent::Episode(
                    ArcAgi3BlindBenchmarkExecutedEpisode::new(
                        episode.game_id,
                        episode.termination,
                        episode.completed_cognitive_steps,
                    ),
                ))
            }

            ArcAgi3BlindBenchmarkHarnessResponse::Finalized(finalization) => {
                if finalization.request_episode_index() != expected_episode_index {
                    return Err(
                        ArcAgi3BlindBenchmarkHarnessBridgeError::EpisodeIndexMismatch {
                            expected: expected_episode_index,
                            observed: finalization.request_episode_index(),
                        },
                    );
                }

                if finalization.request_max_cognitive_steps_per_episode() != expected_step_budget {
                    return Err(
                        ArcAgi3BlindBenchmarkHarnessBridgeError::StepBudgetMismatch {
                            expected: expected_step_budget,
                            observed: finalization.request_max_cognitive_steps_per_episode(),
                        },
                    );
                }

                Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(
                    finalization.summary,
                ))
            }
        }
    }
}

pub fn run_blind_benchmark_with_harness<'a, H>(
    runtime: &'a mut ArcAgi3BlindBenchmarkExecutionRuntime,
    bridge: &mut ArcAgi3BlindBenchmarkHarnessBridge<H>,
) -> Result<
    &'a ArcAgi3ScorecardSummary,
    ArcAgi3BlindBenchmarkExecutionError<ArcAgi3BlindBenchmarkHarnessBridgeError<H::Error>>,
>
where
    H: ArcAgi3BlindBenchmarkExternalHarness,
{
    runtime.run_with(|request| bridge.next_event(request))
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3BlindBenchmarkHarnessBridge;

impl UniversalArcAgi3BlindBenchmarkHarnessBridge {
    pub fn bridge<H>(harness: H) -> ArcAgi3BlindBenchmarkHarnessBridge<H>
    where
        H: ArcAgi3BlindBenchmarkExternalHarness,
    {
        ArcAgi3BlindBenchmarkHarnessBridge::new(harness)
    }

    pub fn run<'a, H>(
        runtime: &'a mut ArcAgi3BlindBenchmarkExecutionRuntime,
        bridge: &mut ArcAgi3BlindBenchmarkHarnessBridge<H>,
    ) -> Result<
        &'a ArcAgi3ScorecardSummary,
        ArcAgi3BlindBenchmarkExecutionError<ArcAgi3BlindBenchmarkHarnessBridgeError<H::Error>>,
    >
    where
        H: ArcAgi3BlindBenchmarkExternalHarness,
    {
        run_blind_benchmark_with_harness(runtime, bridge)
    }
}

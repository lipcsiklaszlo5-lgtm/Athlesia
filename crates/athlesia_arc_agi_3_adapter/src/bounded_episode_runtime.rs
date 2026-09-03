use crate::environment_transport_boundary::{
    ArcAgi3EnvironmentTransport, ArcAgi3TransportFailureDisposition,
};
use crate::live_environment_runtime::{
    ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError, ArcAgi3LiveEnvironmentRuntime,
    ArcAgi3LiveEnvironmentStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcAgi3BoundedEpisodePolicy {
    max_cognitive_steps: usize,
}

impl ArcAgi3BoundedEpisodePolicy {
    pub fn new(max_cognitive_steps: usize) -> Option<Self> {
        if max_cognitive_steps == 0 {
            return None;
        }

        Some(Self {
            max_cognitive_steps,
        })
    }

    pub fn max_cognitive_steps(self) -> usize {
        self.max_cognitive_steps
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3BoundedEpisodeTermination {
    Won,
    GameOver,
    StepBudgetExhausted,
}

impl ArcAgi3BoundedEpisodeTermination {
    pub fn is_terminal_environment_state(self) -> bool {
        matches!(self, Self::Won | Self::GameOver)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BoundedEpisodeResult {
    termination: ArcAgi3BoundedEpisodeTermination,
    steps: Vec<ArcAgi3LiveCognitiveStep>,
    starting_completed_cognitive_step_count: u64,
    ending_completed_cognitive_step_count: u64,
    final_status: ArcAgi3LiveEnvironmentStatus,
}

impl ArcAgi3BoundedEpisodeResult {
    pub fn termination(&self) -> ArcAgi3BoundedEpisodeTermination {
        self.termination
    }

    pub fn steps(&self) -> &[ArcAgi3LiveCognitiveStep] {
        &self.steps
    }

    pub fn completed_steps_in_episode(&self) -> usize {
        self.steps.len()
    }

    pub fn starting_completed_cognitive_step_count(&self) -> u64 {
        self.starting_completed_cognitive_step_count
    }

    pub fn ending_completed_cognitive_step_count(&self) -> u64 {
        self.ending_completed_cognitive_step_count
    }

    pub fn final_status(&self) -> ArcAgi3LiveEnvironmentStatus {
        self.final_status
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ArcAgi3BoundedEpisodeError {
    RuntimeNotRunnable(ArcAgi3LiveEnvironmentStatus),
    RuntimeFaultedPending(Option<ArcAgi3TransportFailureDisposition>),
    StepFailed {
        completed_steps_in_episode: usize,
        error: ArcAgi3LiveEnvironmentError,
    },
    StepHistoryLengthOverflow,
    ExpectedStepCounterOverflow,
    StepCounterMismatch {
        expected: u64,
        actual: u64,
    },
    UnexpectedPostStepStatus(ArcAgi3LiveEnvironmentStatus),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3BoundedEpisodeRuntime;

impl ArcAgi3BoundedEpisodeRuntime {
    fn terminal_result(
        runtime_status: ArcAgi3LiveEnvironmentStatus,
        starting_completed_cognitive_step_count: u64,
        steps: Vec<ArcAgi3LiveCognitiveStep>,
        ending_completed_cognitive_step_count: u64,
    ) -> Option<ArcAgi3BoundedEpisodeResult> {
        let termination = match runtime_status {
            ArcAgi3LiveEnvironmentStatus::Won => ArcAgi3BoundedEpisodeTermination::Won,
            ArcAgi3LiveEnvironmentStatus::GameOver => ArcAgi3BoundedEpisodeTermination::GameOver,
            ArcAgi3LiveEnvironmentStatus::NotStarted
            | ArcAgi3LiveEnvironmentStatus::Active
            | ArcAgi3LiveEnvironmentStatus::FaultedPending => {
                return None;
            }
        };

        Some(ArcAgi3BoundedEpisodeResult {
            termination,
            steps,
            starting_completed_cognitive_step_count,
            ending_completed_cognitive_step_count,
            final_status: runtime_status,
        })
    }

    fn verify_episode_step_counter<T>(
        runtime: &ArcAgi3LiveEnvironmentRuntime<T>,
        starting_completed_cognitive_step_count: u64,
        completed_steps_in_episode: usize,
    ) -> Result<(), ArcAgi3BoundedEpisodeError>
    where
        T: ArcAgi3EnvironmentTransport,
    {
        let completed_steps_in_episode = u64::try_from(completed_steps_in_episode)
            .map_err(|_| ArcAgi3BoundedEpisodeError::StepHistoryLengthOverflow)?;

        let expected = starting_completed_cognitive_step_count
            .checked_add(completed_steps_in_episode)
            .ok_or(ArcAgi3BoundedEpisodeError::ExpectedStepCounterOverflow)?;

        let actual = runtime.completed_cognitive_step_count();

        if actual != expected {
            return Err(ArcAgi3BoundedEpisodeError::StepCounterMismatch { expected, actual });
        }

        Ok(())
    }

    pub fn run_with<T, F>(
        runtime: &mut ArcAgi3LiveEnvironmentRuntime<T>,
        policy: ArcAgi3BoundedEpisodePolicy,
        mut execute_step: F,
    ) -> Result<ArcAgi3BoundedEpisodeResult, ArcAgi3BoundedEpisodeError>
    where
        T: ArcAgi3EnvironmentTransport,
        F: FnMut(
            &mut ArcAgi3LiveEnvironmentRuntime<T>,
        ) -> Result<ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError>,
    {
        let starting_completed_cognitive_step_count = runtime.completed_cognitive_step_count();

        match runtime.status() {
            ArcAgi3LiveEnvironmentStatus::Won | ArcAgi3LiveEnvironmentStatus::GameOver => {
                return Ok(Self::terminal_result(
                    runtime.status(),
                    starting_completed_cognitive_step_count,
                    Vec::new(),
                    runtime.completed_cognitive_step_count(),
                )
                .expect("terminal status has terminal result"));
            }

            ArcAgi3LiveEnvironmentStatus::NotStarted => {
                return Err(ArcAgi3BoundedEpisodeError::RuntimeNotRunnable(
                    ArcAgi3LiveEnvironmentStatus::NotStarted,
                ));
            }

            ArcAgi3LiveEnvironmentStatus::FaultedPending => {
                return Err(ArcAgi3BoundedEpisodeError::RuntimeFaultedPending(
                    runtime.fault_disposition(),
                ));
            }

            ArcAgi3LiveEnvironmentStatus::Active => {}
        }

        let mut steps = Vec::with_capacity(policy.max_cognitive_steps());

        while steps.len() < policy.max_cognitive_steps() {
            let status_before_step = runtime.status();

            match status_before_step {
                ArcAgi3LiveEnvironmentStatus::Won | ArcAgi3LiveEnvironmentStatus::GameOver => {
                    return Ok(Self::terminal_result(
                        status_before_step,
                        starting_completed_cognitive_step_count,
                        steps,
                        runtime.completed_cognitive_step_count(),
                    )
                    .expect("terminal status has terminal result"));
                }

                ArcAgi3LiveEnvironmentStatus::FaultedPending => {
                    return Err(ArcAgi3BoundedEpisodeError::RuntimeFaultedPending(
                        runtime.fault_disposition(),
                    ));
                }

                ArcAgi3LiveEnvironmentStatus::NotStarted => {
                    return Err(ArcAgi3BoundedEpisodeError::RuntimeNotRunnable(
                        ArcAgi3LiveEnvironmentStatus::NotStarted,
                    ));
                }

                ArcAgi3LiveEnvironmentStatus::Active => {}
            }

            let completed_before_failure = steps.len();

            let step =
                execute_step(runtime).map_err(|error| ArcAgi3BoundedEpisodeError::StepFailed {
                    completed_steps_in_episode: completed_before_failure,
                    error,
                })?;

            steps.push(step);

            Self::verify_episode_step_counter(
                runtime,
                starting_completed_cognitive_step_count,
                steps.len(),
            )?;

            match runtime.status() {
                ArcAgi3LiveEnvironmentStatus::Won | ArcAgi3LiveEnvironmentStatus::GameOver => {
                    return Ok(Self::terminal_result(
                        runtime.status(),
                        starting_completed_cognitive_step_count,
                        steps,
                        runtime.completed_cognitive_step_count(),
                    )
                    .expect("terminal status has terminal result"));
                }

                ArcAgi3LiveEnvironmentStatus::Active => {}

                ArcAgi3LiveEnvironmentStatus::NotStarted
                | ArcAgi3LiveEnvironmentStatus::FaultedPending => {
                    return Err(ArcAgi3BoundedEpisodeError::UnexpectedPostStepStatus(
                        runtime.status(),
                    ));
                }
            }
        }

        Ok(ArcAgi3BoundedEpisodeResult {
            termination: ArcAgi3BoundedEpisodeTermination::StepBudgetExhausted,
            steps,
            starting_completed_cognitive_step_count,
            ending_completed_cognitive_step_count: runtime.completed_cognitive_step_count(),
            final_status: runtime.status(),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3BoundedEpisodeRuntime;

impl UniversalArcAgi3BoundedEpisodeRuntime {
    pub fn run_with<T, F>(
        runtime: &mut ArcAgi3LiveEnvironmentRuntime<T>,
        policy: ArcAgi3BoundedEpisodePolicy,
        execute_step: F,
    ) -> Result<ArcAgi3BoundedEpisodeResult, ArcAgi3BoundedEpisodeError>
    where
        T: ArcAgi3EnvironmentTransport,
        F: FnMut(
            &mut ArcAgi3LiveEnvironmentRuntime<T>,
        ) -> Result<ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError>,
    {
        ArcAgi3BoundedEpisodeRuntime::run_with(runtime, policy, execute_step)
    }
}

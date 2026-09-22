//! Bounded lifecycle for caller-provided successor decisions, not cognitive authority.
use crate::environment_transport_boundary::{
    ArcAgi3EnvironmentTransport, ArcAgi3TransportFailureDisposition,
};
use crate::live_environment_runtime::{
    ArcAgi3LiveEnvironmentError, ArcAgi3LiveEnvironmentRuntime, ArcAgi3LiveEnvironmentStatus,
    ArcAgi3LiveUnifiedStep,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcAgi3SuccessorEpisodePolicy {
    max_decision_attempts: usize,
    max_consecutive_abstentions: usize,
}

impl ArcAgi3SuccessorEpisodePolicy {
    pub fn new(max_decision_attempts: usize, max_consecutive_abstentions: usize) -> Option<Self> {
        if max_decision_attempts == 0 || max_consecutive_abstentions == 0 {
            return None;
        }
        Some(Self {
            max_decision_attempts,
            max_consecutive_abstentions,
        })
    }

    pub fn max_decision_attempts(self) -> usize {
        self.max_decision_attempts
    }
    pub fn max_consecutive_abstentions(self) -> usize {
        self.max_consecutive_abstentions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3SuccessorEpisodeTermination {
    Won,
    GameOver,
    DecisionBudgetExhausted,
    ConsecutiveAbstentionBudgetExhausted,
}

/// Constant-size summary; step history belongs to an optional caller-owned sink.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcAgi3SuccessorEpisodeResult {
    termination: ArcAgi3SuccessorEpisodeTermination,
    decision_attempts: usize,
    executed_steps: usize,
    abstentions: usize,
    starting_completed_cognitive_step_count: u64,
    ending_completed_cognitive_step_count: u64,
    final_status: ArcAgi3LiveEnvironmentStatus,
}

impl ArcAgi3SuccessorEpisodeResult {
    pub fn termination(&self) -> ArcAgi3SuccessorEpisodeTermination {
        self.termination
    }
    pub fn decision_attempts(&self) -> usize {
        self.decision_attempts
    }
    pub fn executed_steps(&self) -> usize {
        self.executed_steps
    }
    pub fn abstentions(&self) -> usize {
        self.abstentions
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
pub enum ArcAgi3SuccessorEpisodeError {
    RuntimeNotRunnable(ArcAgi3LiveEnvironmentStatus),
    RuntimeFaultedPending(Option<ArcAgi3TransportFailureDisposition>),
    RuntimeHasPendingCommand,
    StepFailed {
        decision_attempts: usize,
        executed_steps: usize,
        error: ArcAgi3LiveEnvironmentError,
    },
    DecisionCounterOverflow,
    ExecutedStepCounterOverflow,
    AbstentionCounterOverflow,
    ExpectedCompletedCounterOverflow,
    CompletedCognitiveStepMismatch {
        expected: u64,
        actual: u64,
    },
    ReturnedStepCounterMismatch {
        expected: u64,
        actual: u64,
    },
    AbstentionChangedCompletedStepCount {
        before: u64,
        after: u64,
    },
    AbstentionCreatedPendingCommand,
    ExecutedStepLeftPendingCommand,
    UnexpectedReset {
        before: u64,
        after: u64,
    },
    UnexpectedPostStepStatus(ArcAgi3LiveEnvironmentStatus),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3SuccessorEpisodeRuntime;

impl ArcAgi3SuccessorEpisodeRuntime {
    /// Each callback must perform exactly one existing successor decision attempt.
    /// Capture a B3A sink in the callback to stream traces. No requests or records
    /// are synthesized here; caller-native possibilities/beliefs remain required.
    pub fn run_with<T, F>(
        runtime: &mut ArcAgi3LiveEnvironmentRuntime<T>,
        policy: ArcAgi3SuccessorEpisodePolicy,
        mut execute_attempt: F,
    ) -> Result<ArcAgi3SuccessorEpisodeResult, ArcAgi3SuccessorEpisodeError>
    where
        T: ArcAgi3EnvironmentTransport,
        F: FnMut(
            &mut ArcAgi3LiveEnvironmentRuntime<T>,
        ) -> Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError>,
    {
        use ArcAgi3LiveEnvironmentStatus as Status;
        use ArcAgi3SuccessorEpisodeError as Error;
        use ArcAgi3SuccessorEpisodeTermination as Termination;

        let starting_count = runtime.completed_cognitive_step_count();
        let mut result = ArcAgi3SuccessorEpisodeResult {
            termination: Termination::DecisionBudgetExhausted,
            decision_attempts: 0,
            executed_steps: 0,
            abstentions: 0,
            starting_completed_cognitive_step_count: starting_count,
            ending_completed_cognitive_step_count: starting_count,
            final_status: runtime.status(),
        };
        let mut consecutive_abstentions = 0usize;

        while result.decision_attempts < policy.max_decision_attempts() {
            match runtime.status() {
                Status::Won => {
                    result.termination = Termination::Won;
                    return Ok(result);
                }
                Status::GameOver => {
                    result.termination = Termination::GameOver;
                    return Ok(result);
                }
                Status::FaultedPending => {
                    return Err(Error::RuntimeFaultedPending(runtime.fault_disposition()))
                }
                Status::NotStarted => return Err(Error::RuntimeNotRunnable(Status::NotStarted)),
                Status::Active => {}
            }
            if runtime.cognitive_runtime().session().has_pending_command() {
                return Err(Error::RuntimeHasPendingCommand);
            }

            let before_count = runtime.completed_cognitive_step_count();
            let before_resets = runtime.completed_reset_count();
            result.decision_attempts = result
                .decision_attempts
                .checked_add(1)
                .ok_or(Error::DecisionCounterOverflow)?;
            let step = execute_attempt(runtime).map_err(|error| Error::StepFailed {
                decision_attempts: result.decision_attempts,
                executed_steps: result.executed_steps,
                error,
            })?;
            let after_count = runtime.completed_cognitive_step_count();
            let pending = runtime.cognitive_runtime().session().has_pending_command();
            let abstained = step.is_none();

            match step {
                Some(step) => {
                    result.executed_steps = result
                        .executed_steps
                        .checked_add(1)
                        .ok_or(Error::ExecutedStepCounterOverflow)?;
                    consecutive_abstentions = 0;
                    let executed = u64::try_from(result.executed_steps)
                        .map_err(|_| Error::ExecutedStepCounterOverflow)?;
                    let expected = starting_count
                        .checked_add(executed)
                        .ok_or(Error::ExpectedCompletedCounterOverflow)?;
                    if after_count != expected {
                        return Err(Error::CompletedCognitiveStepMismatch {
                            expected,
                            actual: after_count,
                        });
                    }
                    if step.completed_cognitive_step_count() != expected {
                        return Err(Error::ReturnedStepCounterMismatch {
                            expected,
                            actual: step.completed_cognitive_step_count(),
                        });
                    }
                    if pending {
                        return Err(Error::ExecutedStepLeftPendingCommand);
                    }
                }
                None => {
                    if after_count != before_count {
                        return Err(Error::AbstentionChangedCompletedStepCount {
                            before: before_count,
                            after: after_count,
                        });
                    }
                    if pending {
                        return Err(Error::AbstentionCreatedPendingCommand);
                    }
                    result.abstentions = result
                        .abstentions
                        .checked_add(1)
                        .ok_or(Error::AbstentionCounterOverflow)?;
                    consecutive_abstentions = consecutive_abstentions
                        .checked_add(1)
                        .ok_or(Error::AbstentionCounterOverflow)?;
                }
            }
            if runtime.completed_reset_count() != before_resets {
                return Err(Error::UnexpectedReset {
                    before: before_resets,
                    after: runtime.completed_reset_count(),
                });
            }
            result.ending_completed_cognitive_step_count = after_count;
            result.final_status = runtime.status();
            match result.final_status {
                Status::Active => {}
                Status::Won if !abstained => {
                    result.termination = Termination::Won;
                    return Ok(result);
                }
                Status::GameOver if !abstained => {
                    result.termination = Termination::GameOver;
                    return Ok(result);
                }
                status => return Err(Error::UnexpectedPostStepStatus(status)),
            }
            if consecutive_abstentions >= policy.max_consecutive_abstentions() {
                result.termination = Termination::ConsecutiveAbstentionBudgetExhausted;
                return Ok(result);
            }
        }
        Ok(result)
    }
}

use crate::execution_runtime::{
    ArcAgi3BlindBenchmarkExecutionRuntime, ArcAgi3BlindBenchmarkExecutionStatus,
};
use crate::run_manifest::ArcAgi3BlindBenchmarkRunManifest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArcAgi3BlindBenchmarkRunBindingError {
    RuntimeNotReady(ArcAgi3BlindBenchmarkExecutionStatus),
    RuntimeHasEpisodeHistory { observed_episodes: usize },
    RuntimeLedgerAlreadyFinalized,
    RunIdMismatch { manifest: String, runtime: String },
    AgentNameMismatch { manifest: String, runtime: String },
    AgentVersionMismatch { manifest: String, runtime: String },
    AgentSourceRevisionMismatch { manifest: String, runtime: String },
    EpisodeStepBudgetMismatch { manifest: usize, runtime: usize },
}

pub struct ArcAgi3BlindBenchmarkBoundRun {
    manifest: ArcAgi3BlindBenchmarkRunManifest,
    runtime: ArcAgi3BlindBenchmarkExecutionRuntime,
}

impl ArcAgi3BlindBenchmarkBoundRun {
    pub fn manifest(&self) -> &ArcAgi3BlindBenchmarkRunManifest {
        &self.manifest
    }

    pub fn runtime(&self) -> &ArcAgi3BlindBenchmarkExecutionRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut ArcAgi3BlindBenchmarkExecutionRuntime {
        &mut self.runtime
    }

    pub fn into_parts(
        self,
    ) -> (
        ArcAgi3BlindBenchmarkRunManifest,
        ArcAgi3BlindBenchmarkExecutionRuntime,
    ) {
        (self.manifest, self.runtime)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkRunBinding;

impl ArcAgi3BlindBenchmarkRunBinding {
    pub fn validate(
        manifest: &ArcAgi3BlindBenchmarkRunManifest,
        runtime: &ArcAgi3BlindBenchmarkExecutionRuntime,
    ) -> Result<(), ArcAgi3BlindBenchmarkRunBindingError> {
        let status = runtime.status();

        if status != ArcAgi3BlindBenchmarkExecutionStatus::Ready {
            return Err(ArcAgi3BlindBenchmarkRunBindingError::RuntimeNotReady(
                status,
            ));
        }

        let ledger = runtime.ledger();

        if !ledger.episodes().is_empty() {
            return Err(
                ArcAgi3BlindBenchmarkRunBindingError::RuntimeHasEpisodeHistory {
                    observed_episodes: ledger.episodes().len(),
                },
            );
        }

        if ledger.final_summary().is_some() {
            return Err(ArcAgi3BlindBenchmarkRunBindingError::RuntimeLedgerAlreadyFinalized);
        }

        let manifest_spec = manifest.spec();

        let runtime_spec = ledger.spec();

        let manifest_run_id = manifest_spec.run_id().as_str();

        let runtime_run_id = runtime_spec.run_id().as_str();

        if manifest_run_id != runtime_run_id {
            return Err(ArcAgi3BlindBenchmarkRunBindingError::RunIdMismatch {
                manifest: manifest_run_id.to_string(),
                runtime: runtime_run_id.to_string(),
            });
        }

        let manifest_agent = manifest_spec.agent();

        let runtime_agent = runtime_spec.agent();

        if manifest_agent.name() != runtime_agent.name() {
            return Err(ArcAgi3BlindBenchmarkRunBindingError::AgentNameMismatch {
                manifest: manifest_agent.name().to_string(),
                runtime: runtime_agent.name().to_string(),
            });
        }

        if manifest_agent.version() != runtime_agent.version() {
            return Err(ArcAgi3BlindBenchmarkRunBindingError::AgentVersionMismatch {
                manifest: manifest_agent.version().to_string(),
                runtime: runtime_agent.version().to_string(),
            });
        }

        if manifest_agent.source_revision() != runtime_agent.source_revision() {
            return Err(
                ArcAgi3BlindBenchmarkRunBindingError::AgentSourceRevisionMismatch {
                    manifest: manifest_agent.source_revision().to_string(),
                    runtime: runtime_agent.source_revision().to_string(),
                },
            );
        }

        let manifest_budget = manifest_spec.policy().max_cognitive_steps_per_episode();

        let runtime_budget = runtime_spec.policy().max_cognitive_steps_per_episode();

        if manifest_budget != runtime_budget {
            return Err(
                ArcAgi3BlindBenchmarkRunBindingError::EpisodeStepBudgetMismatch {
                    manifest: manifest_budget,
                    runtime: runtime_budget,
                },
            );
        }

        Ok(())
    }

    pub fn bind(
        manifest: ArcAgi3BlindBenchmarkRunManifest,
        runtime: ArcAgi3BlindBenchmarkExecutionRuntime,
    ) -> Result<ArcAgi3BlindBenchmarkBoundRun, ArcAgi3BlindBenchmarkRunBindingError> {
        Self::validate(&manifest, &runtime)?;

        Ok(ArcAgi3BlindBenchmarkBoundRun { manifest, runtime })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3BlindBenchmarkRunBinding;

impl UniversalArcAgi3BlindBenchmarkRunBinding {
    pub fn validate(
        manifest: &ArcAgi3BlindBenchmarkRunManifest,
        runtime: &ArcAgi3BlindBenchmarkExecutionRuntime,
    ) -> Result<(), ArcAgi3BlindBenchmarkRunBindingError> {
        ArcAgi3BlindBenchmarkRunBinding::validate(manifest, runtime)
    }

    pub fn bind(
        manifest: ArcAgi3BlindBenchmarkRunManifest,
        runtime: ArcAgi3BlindBenchmarkExecutionRuntime,
    ) -> Result<ArcAgi3BlindBenchmarkBoundRun, ArcAgi3BlindBenchmarkRunBindingError> {
        ArcAgi3BlindBenchmarkRunBinding::bind(manifest, runtime)
    }
}

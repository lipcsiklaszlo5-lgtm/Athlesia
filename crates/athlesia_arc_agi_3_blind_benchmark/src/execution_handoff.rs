use athlesia_arc_agi_3_adapter::competition_session_runtime::ArcAgi3ScorecardSummary;

use crate::execution_runtime::{
    ArcAgi3BlindBenchmarkExecutionError, ArcAgi3BlindBenchmarkExecutionStatus,
};
use crate::harness_bridge::{
    run_blind_benchmark_with_harness, ArcAgi3BlindBenchmarkExternalHarness,
    ArcAgi3BlindBenchmarkHarnessBridge, ArcAgi3BlindBenchmarkHarnessBridgeError,
};
use crate::run_binding::ArcAgi3BlindBenchmarkBoundRun;
use crate::run_manifest::ArcAgi3BlindBenchmarkRunManifest;

pub struct ArcAgi3BlindBenchmarkExecutionHandoff<H> {
    bound_run: ArcAgi3BlindBenchmarkBoundRun,
    bridge: ArcAgi3BlindBenchmarkHarnessBridge<H>,
}

impl<H> ArcAgi3BlindBenchmarkExecutionHandoff<H>
where
    H: ArcAgi3BlindBenchmarkExternalHarness,
{
    pub fn new(
        bound_run: ArcAgi3BlindBenchmarkBoundRun,
        bridge: ArcAgi3BlindBenchmarkHarnessBridge<H>,
    ) -> Self {
        Self { bound_run, bridge }
    }

    pub fn manifest(&self) -> &ArcAgi3BlindBenchmarkRunManifest {
        self.bound_run.manifest()
    }

    pub fn execution_status(&self) -> ArcAgi3BlindBenchmarkExecutionStatus {
        self.bound_run.runtime().status()
    }

    pub fn bridge(&self) -> &ArcAgi3BlindBenchmarkHarnessBridge<H> {
        &self.bridge
    }

    pub fn execute(
        &mut self,
    ) -> Result<
        &ArcAgi3ScorecardSummary,
        ArcAgi3BlindBenchmarkExecutionError<ArcAgi3BlindBenchmarkHarnessBridgeError<H::Error>>,
    > {
        run_blind_benchmark_with_harness(self.bound_run.runtime_mut(), &mut self.bridge)
    }

    pub fn into_parts(
        self,
    ) -> (
        ArcAgi3BlindBenchmarkBoundRun,
        ArcAgi3BlindBenchmarkHarnessBridge<H>,
    ) {
        (self.bound_run, self.bridge)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3BlindBenchmarkExecutionHandoff;

impl UniversalArcAgi3BlindBenchmarkExecutionHandoff {
    pub fn handoff<H>(
        bound_run: ArcAgi3BlindBenchmarkBoundRun,
        bridge: ArcAgi3BlindBenchmarkHarnessBridge<H>,
    ) -> ArcAgi3BlindBenchmarkExecutionHandoff<H>
    where
        H: ArcAgi3BlindBenchmarkExternalHarness,
    {
        ArcAgi3BlindBenchmarkExecutionHandoff::new(bound_run, bridge)
    }

    pub fn execute<H>(
        handoff: &mut ArcAgi3BlindBenchmarkExecutionHandoff<H>,
    ) -> Result<
        &ArcAgi3ScorecardSummary,
        ArcAgi3BlindBenchmarkExecutionError<ArcAgi3BlindBenchmarkHarnessBridgeError<H::Error>>,
    >
    where
        H: ArcAgi3BlindBenchmarkExternalHarness,
    {
        handoff.execute()
    }
}

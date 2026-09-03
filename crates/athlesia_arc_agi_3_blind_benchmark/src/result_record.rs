use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardSummary,
};

use crate::execution_runtime::ArcAgi3BlindBenchmarkExecutionStatus;
use crate::run_binding::ArcAgi3BlindBenchmarkBoundRun;
use crate::run_manifest::ArcAgi3BlindBenchmarkRunManifest;
use crate::{ArcAgi3BlindBenchmarkEpisodeObservation, ArcAgi3BlindBenchmarkLedger};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArcAgi3BlindBenchmarkResultRecordError {
    RuntimeNotFinalized(ArcAgi3BlindBenchmarkExecutionStatus),
}

pub struct ArcAgi3BlindBenchmarkResultRecord {
    manifest: ArcAgi3BlindBenchmarkRunManifest,
    ledger: ArcAgi3BlindBenchmarkLedger,
}

impl ArcAgi3BlindBenchmarkResultRecord {
    pub fn from_bound_run(
        bound_run: ArcAgi3BlindBenchmarkBoundRun,
    ) -> Result<Self, ArcAgi3BlindBenchmarkResultRecordError> {
        let status = bound_run.runtime().status();

        if status != ArcAgi3BlindBenchmarkExecutionStatus::Finalized {
            return Err(ArcAgi3BlindBenchmarkResultRecordError::RuntimeNotFinalized(
                status,
            ));
        }

        let (manifest, runtime) = bound_run.into_parts();

        Ok(Self {
            manifest,
            ledger: runtime.into_ledger(),
        })
    }

    pub fn manifest(&self) -> &ArcAgi3BlindBenchmarkRunManifest {
        &self.manifest
    }

    pub fn ledger(&self) -> &ArcAgi3BlindBenchmarkLedger {
        &self.ledger
    }

    pub fn scorecard_id(&self) -> &ArcAgi3ScorecardId {
        self.ledger.card_id()
    }

    pub fn episodes(&self) -> &[ArcAgi3BlindBenchmarkEpisodeObservation] {
        self.ledger.episodes()
    }

    pub fn final_summary(&self) -> Option<&ArcAgi3ScorecardSummary> {
        self.ledger.final_summary()
    }

    pub fn server_score(&self) -> Option<f64> {
        self.ledger.server_score()
    }

    pub fn into_parts(
        self,
    ) -> (
        ArcAgi3BlindBenchmarkRunManifest,
        ArcAgi3BlindBenchmarkLedger,
    ) {
        (self.manifest, self.ledger)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3BlindBenchmarkResultRecord;

impl UniversalArcAgi3BlindBenchmarkResultRecord {
    pub fn record(
        bound_run: ArcAgi3BlindBenchmarkBoundRun,
    ) -> Result<ArcAgi3BlindBenchmarkResultRecord, ArcAgi3BlindBenchmarkResultRecordError> {
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(bound_run)
    }
}

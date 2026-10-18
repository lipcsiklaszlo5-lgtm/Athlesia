//! Bounded Bring-Your-Own-Data causal transfer proof.
//!
//! This adapter deliberately accepts a narrow, inspectable CSV contract. It
//! never guesses missing values, never trains on target holdouts, and never
//! turns ambiguity into success. The first target episode is a structural
//! mapping witness; every later target episode is an independent holdout.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::cross_domain_causal_core::{
    CanonicalCausalGraph, CausalEdge, CausalMapping, CausalNode, CrossDomainCausalCore, Digest,
    EdgeKind, Literal, MappingBudget, MappingVerdict, NodeId, NodeKind, ObservationVerdict, State,
    TransferStatus, TransitionExample,
};

pub const MAX_INPUT_BYTES: usize = 256 * 1024;
pub const MAX_EPISODES: usize = 128;
pub const MAX_STATE_FIELDS: usize = 16;
const MAX_LABEL_BYTES: usize = 64;
const MAPPING_BUDGET: u64 = 16_384;
const SOURCE_BASE: NodeId = 1;
const TARGET_BASE: NodeId = 10_001;
const OPERATION_TAG: u64 = 0x4154_484c_4553_4941;
const EFFECT_TAG: u64 = 0x4546_4645_4354;
const CONTEXT_FALSE_TAG: u64 = 0x434f_4e54_0000;
const CONTEXT_TRUE_TAG: u64 = 0x434f_4e54_0001;
const REPORT_SCHEMA: &str = "athlesia/byod-causal-transfer/2";
const PROOF_SCHEMA: &str = "athlesia/byod-break-test-proof/1";
const VERIFICATION_SCHEMA: &str = "athlesia/byod-break-test-verification/1";
const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ByodTransferRequest {
    pub source_csv: String,
    pub target_csv: String,
}

impl ByodTransferRequest {
    pub fn new(source_csv: impl Into<String>, target_csv: impl Into<String>) -> Self {
        Self {
            source_csv: source_csv.into(),
            target_csv: target_csv.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ByodVerdict {
    Verified,
    Contradicted,
    Abstain,
    Refused,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ByodReasonCode {
    Verified,
    SourceMalformed,
    TargetMalformed,
    SourceEvidenceInsufficient,
    TargetMappingEvidenceInsufficient,
    TargetHoldoutEvidenceInsufficient,
    FieldCountMismatch,
    StructuralRoleAmbiguous,
    MappingAmbiguous,
    MappingNoMatch,
    MappingBudgetExceeded,
    SourceModelRejected,
    TransferNotAdmitted,
    TargetContradiction,
    TargetObservationAbstained,
    TargetRevisionRequired,
    ProofMismatch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ByodStage {
    pub name: String,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ByodMapping {
    pub source_role: String,
    pub target_role: String,
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub type_tag: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ByodEpisodeResult {
    pub episode: u64,
    pub before_digest: Digest,
    pub before_id: String,
    pub predicted_after_digest: Option<Digest>,
    pub predicted_after_id: Option<String>,
    pub observed_after_digest: Digest,
    pub observed_after_id: String,
    pub matched: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ByodTransferReport {
    pub schema: String,
    pub status: ByodVerdict,
    pub reason_code: ByodReasonCode,
    pub reason: String,
    pub source_records: usize,
    pub target_records: usize,
    pub target_mapping_records: usize,
    pub target_holdout_records: usize,
    pub field_count: usize,
    pub source_action: String,
    pub target_action: String,
    pub source_effect: Option<String>,
    pub target_effect: Option<String>,
    pub source_input_digest: Digest,
    pub target_input_digest: Digest,
    pub source_graph_digest: Option<Digest>,
    pub target_graph_digest: Option<Digest>,
    pub mapping_digest: Option<Digest>,
    pub source_model_digest: Option<Digest>,
    pub target_training_updates: usize,
    pub target_verified_records: usize,
    pub conflicting_episode: Option<u64>,
    pub evidence_admitted: bool,
    /// Always false in P13. Execution still belongs to the separate P9 lease boundary.
    pub execution_authority_issued: bool,
    pub ledger_verified: bool,
    pub mapping: Vec<ByodMapping>,
    pub episodes: Vec<ByodEpisodeResult>,
    pub stages: Vec<ByodStage>,
}

impl ByodTransferReport {
    pub fn succeeded(&self) -> bool {
        self.status == ByodVerdict::Verified
            && self.evidence_admitted
            && !self.execution_authority_issued
            && self.target_training_updates == 0
            && self.target_holdout_records > 0
            && self.target_verified_records == self.target_holdout_records
            && self.ledger_verified
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ByodProofBundle {
    pub schema: String,
    pub request: ByodTransferRequest,
    pub report: ByodTransferReport,
    pub proof_digest: Digest,
    /// Exact hexadecimal representation for JavaScript-safe display and replay.
    pub proof_id: String,
    pub integrity_algorithm: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ByodProofVerification {
    pub schema: String,
    pub verified: bool,
    pub proof_id: String,
    pub status: ByodVerdict,
    pub reason_code: ByodReasonCode,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CsvEpisode {
    episode: u64,
    before: Vec<bool>,
    after: Vec<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CsvDomain {
    action: String,
    fields: Vec<String>,
    episodes: Vec<CsvEpisode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParseFailure {
    Empty,
    TooLarge,
    UnsupportedCsv,
    Header,
    FieldBudget,
    DuplicateField,
    InvalidLabel,
    RowWidth,
    InvalidEpisode,
    DuplicateEpisode,
    InvalidBoolean,
    MixedActions,
    EpisodeBudget,
}

fn mix(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

fn digest_bytes(bytes: &[u8]) -> Digest {
    bytes.iter().fold(FNV_OFFSET, |state, byte| {
        (state ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}

fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LABEL_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

fn parse_csv(input: &str) -> Result<CsvDomain, ParseFailure> {
    if input.is_empty() {
        return Err(ParseFailure::Empty);
    }
    if input.len() > MAX_INPUT_BYTES {
        return Err(ParseFailure::TooLarge);
    }
    if input.contains(['"', '\0']) {
        return Err(ParseFailure::UnsupportedCsv);
    }

    let normalized = input.replace("\r\n", "\n");
    let mut lines: Vec<&str> = normalized.split('\n').collect();
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    if lines.len() < 2 || lines.iter().any(|line| line.trim().is_empty()) {
        return Err(ParseFailure::Empty);
    }

    let headers: Vec<String> = lines[0]
        .split(',')
        .map(|value| value.trim().to_owned())
        .collect();
    if headers.len() < 6 || headers[0] != "episode" || headers[1] != "action" {
        return Err(ParseFailure::Header);
    }

    let mut before = BTreeMap::<String, usize>::new();
    let mut after = BTreeMap::<String, usize>::new();
    for (index, header) in headers.iter().enumerate().skip(2) {
        let (prefix, label) = header.split_once('.').ok_or(ParseFailure::Header)?;
        if !valid_label(label) {
            return Err(ParseFailure::InvalidLabel);
        }
        let destination = match prefix {
            "before" => &mut before,
            "after" => &mut after,
            _ => return Err(ParseFailure::Header),
        };
        if destination.insert(label.to_owned(), index).is_some() {
            return Err(ParseFailure::DuplicateField);
        }
    }
    if before.is_empty() || before.keys().ne(after.keys()) {
        return Err(ParseFailure::Header);
    }
    if before.len() > MAX_STATE_FIELDS {
        return Err(ParseFailure::FieldBudget);
    }
    let fields: Vec<String> = before.keys().cloned().collect();
    let mut episodes = Vec::new();
    let mut seen = BTreeSet::new();
    let mut action: Option<String> = None;

    for line in lines.into_iter().skip(1) {
        if episodes.len() == MAX_EPISODES {
            return Err(ParseFailure::EpisodeBudget);
        }
        let cells: Vec<&str> = line.split(',').map(str::trim).collect();
        if cells.len() != headers.len() {
            return Err(ParseFailure::RowWidth);
        }
        let episode = cells[0]
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or(ParseFailure::InvalidEpisode)?;
        if !seen.insert(episode) {
            return Err(ParseFailure::DuplicateEpisode);
        }
        if !valid_label(cells[1]) {
            return Err(ParseFailure::InvalidLabel);
        }
        if let Some(expected) = &action {
            if expected != cells[1] {
                return Err(ParseFailure::MixedActions);
            }
        } else {
            action = Some(cells[1].to_owned());
        }
        let mut before_values = Vec::with_capacity(fields.len());
        let mut after_values = Vec::with_capacity(fields.len());
        for field in &fields {
            before_values
                .push(parse_bool(cells[before[field]]).ok_or(ParseFailure::InvalidBoolean)?);
            after_values.push(parse_bool(cells[after[field]]).ok_or(ParseFailure::InvalidBoolean)?);
        }
        episodes.push(CsvEpisode {
            episode,
            before: before_values,
            after: after_values,
        });
    }

    Ok(CsvDomain {
        action: action.ok_or(ParseFailure::Empty)?,
        fields,
        episodes,
    })
}

fn structural_graph(
    domain: &CsvDomain,
    base: NodeId,
) -> Result<(CanonicalCausalGraph, usize), ByodReasonCode> {
    let witness = domain
        .episodes
        .first()
        .ok_or(ByodReasonCode::TargetMappingEvidenceInsufficient)?;
    let changed: Vec<usize> = witness
        .before
        .iter()
        .zip(&witness.after)
        .enumerate()
        .filter_map(|(index, (before, after))| (before != after).then_some(index))
        .collect();
    if changed.len() != 1 {
        return Err(ByodReasonCode::StructuralRoleAmbiguous);
    }
    let effect = changed[0];
    let mut nodes = vec![CausalNode {
        id: base,
        kind: NodeKind::Operation,
        type_tag: OPERATION_TAG,
    }];
    let mut edges = Vec::with_capacity(domain.fields.len());
    for (index, value) in witness.before.iter().copied().enumerate() {
        let node = base + index as NodeId + 1;
        let type_tag = if index == effect {
            EFFECT_TAG
        } else if value {
            CONTEXT_TRUE_TAG
        } else {
            CONTEXT_FALSE_TAG
        };
        nodes.push(CausalNode {
            id: node,
            kind: NodeKind::StateField,
            type_tag,
        });
        edges.push(CausalEdge {
            from: base,
            to: node,
            kind: if index == effect {
                EdgeKind::Writes
            } else {
                EdgeKind::Requires
            },
        });
    }
    CanonicalCausalGraph::new(nodes, edges)
        .map(|graph| (graph, effect))
        .map_err(|_| ByodReasonCode::StructuralRoleAmbiguous)
}

fn state(values: &[bool], base: NodeId) -> State {
    State::new(
        values
            .iter()
            .copied()
            .enumerate()
            .map(|(index, value)| Literal::new(u64::from(base + index as NodeId + 1), value)),
    )
}

fn examples(domain: &CsvDomain, base: NodeId) -> Vec<TransitionExample> {
    domain
        .episodes
        .iter()
        .map(|episode| {
            TransitionExample::new(
                episode.episode,
                u64::from(base),
                state(&episode.before, base),
                state(&episode.after, base),
            )
        })
        .collect()
}

fn role_name(domain: &CsvDomain, base: NodeId, node: NodeId) -> Option<String> {
    if node == base {
        return Some(format!("action:{}", domain.action));
    }
    let index = node.checked_sub(base + 1)? as usize;
    domain.fields.get(index).cloned()
}

fn mapping_digest(mapping: &CausalMapping) -> Digest {
    mapping.pairs().iter().fold(FNV_OFFSET, |digest, pair| {
        mix(mix(digest, u64::from(pair.0)), u64::from(pair.1))
    })
}

fn render_mapping(
    mapping: &CausalMapping,
    source: &CsvDomain,
    target: &CsvDomain,
    source_graph: &CanonicalCausalGraph,
) -> Vec<ByodMapping> {
    mapping
        .pairs()
        .iter()
        .filter_map(|(source_node, target_node)| {
            let node = source_graph
                .nodes()
                .iter()
                .find(|node| node.id == *source_node)?;
            Some(ByodMapping {
                source_role: role_name(source, SOURCE_BASE, *source_node)?,
                target_role: role_name(target, TARGET_BASE, *target_node)?,
                source_node: *source_node,
                target_node: *target_node,
                type_tag: node.type_tag,
            })
        })
        .collect()
}

fn blank_report(request: &ByodTransferRequest) -> ByodTransferReport {
    ByodTransferReport {
        schema: REPORT_SCHEMA.to_owned(),
        status: ByodVerdict::Refused,
        reason_code: ByodReasonCode::SourceMalformed,
        reason: "input has not passed the bounded parser".to_owned(),
        source_records: 0,
        target_records: 0,
        target_mapping_records: 0,
        target_holdout_records: 0,
        field_count: 0,
        source_action: String::new(),
        target_action: String::new(),
        source_effect: None,
        target_effect: None,
        source_input_digest: digest_bytes(request.source_csv.as_bytes()),
        target_input_digest: digest_bytes(request.target_csv.as_bytes()),
        source_graph_digest: None,
        target_graph_digest: None,
        mapping_digest: None,
        source_model_digest: None,
        target_training_updates: 0,
        target_verified_records: 0,
        conflicting_episode: None,
        evidence_admitted: false,
        execution_authority_issued: false,
        ledger_verified: false,
        mapping: Vec::new(),
        episodes: Vec::new(),
        stages: Vec::new(),
    }
}

fn stop(
    report: &mut ByodTransferReport,
    status: ByodVerdict,
    code: ByodReasonCode,
    reason: impl Into<String>,
    stage: &str,
) {
    report.status = status;
    report.reason_code = code;
    report.reason = reason.into();
    report.stages.push(ByodStage {
        name: stage.to_owned(),
        detail: report.reason.clone(),
    });
}

pub fn run_byod_transfer(request: &ByodTransferRequest) -> ByodTransferReport {
    let mut report = blank_report(request);
    let source = match parse_csv(&request.source_csv) {
        Ok(value) => value,
        Err(error) => {
            stop(
                &mut report,
                ByodVerdict::Refused,
                ByodReasonCode::SourceMalformed,
                format!("source CSV rejected: {error:?}"),
                "REFUSED",
            );
            return report;
        }
    };
    report.source_records = source.episodes.len();
    report.source_action = source.action.clone();
    report.field_count = source.fields.len();
    report.stages.push(ByodStage {
        name: "INGEST_SOURCE".to_owned(),
        detail: format!("{} bounded source episodes accepted", source.episodes.len()),
    });
    if source.episodes.len() < 2 {
        stop(
            &mut report,
            ByodVerdict::Abstain,
            ByodReasonCode::SourceEvidenceInsufficient,
            "at least two source intervention episodes are required",
            "ABSTAIN",
        );
        return report;
    }

    let target = match parse_csv(&request.target_csv) {
        Ok(value) => value,
        Err(error) => {
            stop(
                &mut report,
                ByodVerdict::Refused,
                ByodReasonCode::TargetMalformed,
                format!("target CSV rejected: {error:?}"),
                "REFUSED",
            );
            return report;
        }
    };
    report.target_records = target.episodes.len();
    report.target_mapping_records = usize::from(!target.episodes.is_empty());
    report.target_holdout_records = target.episodes.len().saturating_sub(1);
    report.target_action = target.action.clone();
    report.stages.push(ByodStage {
        name: "INGEST_TARGET".to_owned(),
        detail: format!(
            "one mapping witness and {} sealed holdouts accepted",
            report.target_holdout_records
        ),
    });
    if source.fields.len() != target.fields.len() {
        stop(
            &mut report,
            ByodVerdict::Refused,
            ByodReasonCode::FieldCountMismatch,
            "source and target state dimensions differ",
            "REFUSED",
        );
        return report;
    }

    let (source_graph, source_effect) = match structural_graph(&source, SOURCE_BASE) {
        Ok(value) => value,
        Err(code) => {
            stop(
                &mut report,
                ByodVerdict::Abstain,
                code,
                "source mapping witness does not expose one unique changed role",
                "ABSTAIN",
            );
            return report;
        }
    };
    let (target_graph, target_effect) = match structural_graph(&target, TARGET_BASE) {
        Ok(value) => value,
        Err(code) => {
            stop(
                &mut report,
                ByodVerdict::Abstain,
                code,
                "target mapping witness does not expose one unique changed role",
                "ABSTAIN",
            );
            return report;
        }
    };
    report.source_effect = source.fields.get(source_effect).cloned();
    report.target_effect = target.fields.get(target_effect).cloned();
    report.source_graph_digest = Some(source_graph.digest());
    report.target_graph_digest = Some(target_graph.digest());

    let mut core = CrossDomainCausalCore::new();
    if let Err(reason) = core.learn_source(&examples(&source, SOURCE_BASE)) {
        stop(
            &mut report,
            ByodVerdict::Refused,
            ByodReasonCode::SourceModelRejected,
            format!("source evidence cannot freeze one causal rule: {reason}"),
            "REFUSED",
        );
        return report;
    }
    report.source_model_digest = Some(core.source().model_digest());
    report.stages.push(ByodStage {
        name: "FREEZE_SOURCE".to_owned(),
        detail: "source effect frozen from interventions; no neural weights".to_owned(),
    });

    let budget = MappingBudget::new(MAPPING_BUDGET).expect("positive constant budget");
    let mapping = match source_graph.find_unique_mapping(&target_graph, budget) {
        MappingVerdict::Unique(mapping) => mapping,
        MappingVerdict::Ambiguous { .. } => {
            stop(
                &mut report,
                ByodVerdict::Abstain,
                ByodReasonCode::MappingAmbiguous,
                "more than one structural role mapping fits the evidence",
                "ABSTAIN",
            );
            return report;
        }
        MappingVerdict::ResourceLimit { .. } => {
            stop(
                &mut report,
                ByodVerdict::Abstain,
                ByodReasonCode::MappingBudgetExceeded,
                "bounded structural search ended before uniqueness was proved",
                "ABSTAIN",
            );
            return report;
        }
        MappingVerdict::NoMatch => {
            stop(
                &mut report,
                ByodVerdict::Refused,
                ByodReasonCode::MappingNoMatch,
                "source structure has no compatible target mapping",
                "REFUSED",
            );
            return report;
        }
    };
    report.mapping_digest = Some(mapping_digest(&mapping));
    report.mapping = render_mapping(&mapping, &source, &target, &source_graph);
    report.stages.push(ByodStage {
        name: "MAP".to_owned(),
        detail: format!(
            "one bijective typed mapping proved in {} search states",
            mapping.visited_states()
        ),
    });

    if core.transfer(&source_graph, &target_graph, u64::from(TARGET_BASE), budget)
        != TransferStatus::StructuralCandidate
    {
        stop(
            &mut report,
            ByodVerdict::Refused,
            ByodReasonCode::TransferNotAdmitted,
            "the frozen rule did not enter the target as a structural candidate",
            "REFUSED",
        );
        return report;
    }
    report.stages.push(ByodStage {
        name: "ZERO_SHOT".to_owned(),
        detail: "frozen rule installed with target training updates = 0".to_owned(),
    });

    if report.target_holdout_records == 0 {
        report.ledger_verified = core.ledger().verify();
        stop(
            &mut report,
            ByodVerdict::Abstain,
            ByodReasonCode::TargetHoldoutEvidenceInsufficient,
            "the mapping witness is present, but no independent target holdout remains",
            "ABSTAIN",
        );
        return report;
    }

    for episode in examples(&target, TARGET_BASE).into_iter().skip(1) {
        let predicted = core
            .target()
            .and_then(|learner| learner.predict(episode.action, &episode.before));
        let before_digest = episode.before.digest();
        let predicted_after_digest = predicted.as_ref().map(State::digest);
        let observed_after_digest = episode.after.digest();
        let mut result = ByodEpisodeResult {
            episode: episode.episode,
            before_digest,
            before_id: format!("{before_digest:016x}"),
            predicted_after_digest,
            predicted_after_id: predicted_after_digest.map(|value| format!("{value:016x}")),
            observed_after_digest,
            observed_after_id: format!("{observed_after_digest:016x}"),
            matched: predicted.as_ref() == Some(&episode.after),
        };
        match core.target_trial(episode.clone()) {
            ObservationVerdict::Confirmed => {
                result.matched = true;
                report.target_verified_records += 1;
                report.episodes.push(result);
            }
            ObservationVerdict::ContradictionFound { .. } => {
                result.matched = false;
                report.episodes.push(result);
                report.conflicting_episode = Some(episode.episode);
                report.ledger_verified = core.ledger().verify();
                stop(
                    &mut report,
                    ByodVerdict::Contradicted,
                    ByodReasonCode::TargetContradiction,
                    format!(
                        "held-out target episode {} contradicts the frozen prediction",
                        episode.episode
                    ),
                    "CONTRADICT",
                );
                return report;
            }
            ObservationVerdict::Revised { .. } => {
                report.episodes.push(result);
                report.ledger_verified = core.ledger().verify();
                stop(
                    &mut report,
                    ByodVerdict::Abstain,
                    ByodReasonCode::TargetRevisionRequired,
                    "target evidence requested revision; the zero-shot proof was withheld",
                    "ABSTAIN",
                );
                return report;
            }
            ObservationVerdict::Abstain => {
                report.episodes.push(result);
                report.ledger_verified = core.ledger().verify();
                stop(
                    &mut report,
                    ByodVerdict::Abstain,
                    ByodReasonCode::TargetObservationAbstained,
                    "the target learner could not issue a prediction for a holdout",
                    "ABSTAIN",
                );
                return report;
            }
        }
    }

    report.ledger_verified = core.ledger().verify();
    if report.target_verified_records == report.target_holdout_records && report.ledger_verified {
        report.status = ByodVerdict::Verified;
        report.reason_code = ByodReasonCode::Verified;
        report.reason = format!(
            "{} independent target holdouts matched with zero target training",
            report.target_verified_records
        );
        report.evidence_admitted = true;
        report.stages.push(ByodStage {
            name: "VERIFIED".to_owned(),
            detail: report.reason.clone(),
        });
    } else {
        stop(
            &mut report,
            ByodVerdict::Abstain,
            ByodReasonCode::TargetObservationAbstained,
            "not every target holdout was verified",
            "ABSTAIN",
        );
    }
    report
}

fn proof_digest(request: &ByodTransferRequest, report: &ByodTransferReport) -> Digest {
    let mut digest = digest_bytes(b"ATHLESIA_BYOD_BREAK_TEST_PROOF_V1");
    digest = mix(digest, request.source_csv.len() as u64);
    digest = mix(digest, digest_bytes(request.source_csv.as_bytes()));
    digest = mix(digest, request.target_csv.len() as u64);
    digest = mix(digest, digest_bytes(request.target_csv.as_bytes()));
    let report_json = serde_json::to_vec(report).expect("serializable proof report");
    digest = mix(digest, report_json.len() as u64);
    mix(digest, digest_bytes(&report_json)).max(1)
}

pub fn create_byod_proof(request: ByodTransferRequest) -> ByodProofBundle {
    let report = run_byod_transfer(&request);
    let proof_digest = proof_digest(&request, &report);
    ByodProofBundle {
        schema: PROOF_SCHEMA.to_owned(),
        request,
        report,
        proof_digest,
        proof_id: format!("{proof_digest:016x}"),
        integrity_algorithm: "FNV1A64 deterministic replay digest; not a signature".to_owned(),
    }
}

pub fn verify_byod_proof(bundle: &ByodProofBundle) -> ByodProofVerification {
    let expected = create_byod_proof(bundle.request.clone());
    let verified = bundle.schema == PROOF_SCHEMA
        && bundle.report == expected.report
        && bundle.proof_digest == expected.proof_digest
        && bundle.proof_id == expected.proof_id
        && bundle.integrity_algorithm == expected.integrity_algorithm;
    ByodProofVerification {
        schema: VERIFICATION_SCHEMA.to_owned(),
        verified,
        proof_id: expected.proof_id,
        status: expected.report.status,
        reason_code: if verified {
            expected.report.reason_code
        } else {
            ByodReasonCode::ProofMismatch
        },
        reason: if verified {
            "exact input bytes and deterministic Rust result reproduced".to_owned()
        } else {
            "bundle differs from deterministic recomputation".to_owned()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "episode,action,before.power,before.coolant,before.open,after.power,after.coolant,after.open\n1,press,1,0,0,1,0,1\n2,press,0,1,0,0,1,1\n3,press,1,1,0,1,1,1\n";
    const TARGET: &str = "episode,action,after.commit,before.replica,after.quorum,before.commit,after.replica,before.quorum\n1,write,1,0,1,0,0,1\n2,write,1,1,0,0,1,0\n3,write,1,0,0,0,0,0\n";

    fn request(target: &str) -> ByodTransferRequest {
        ByodTransferRequest::new(SOURCE, target)
    }

    #[test]
    fn user_data_is_ingested_and_transferred_without_target_training() {
        let report = run_byod_transfer(&request(TARGET));
        assert_eq!(report.status, ByodVerdict::Verified);
        assert!(report.succeeded());
        assert_eq!(report.target_training_updates, 0);
        assert_eq!(report.target_verified_records, 2);
        assert!(!report.execution_authority_issued);
        assert!(report.ledger_verified);
    }

    #[test]
    fn harmless_holdout_context_edit_remains_verified_but_changes_proof() {
        let changed = TARGET.replace("2,write,1,1,0,0,1,0", "2,write,1,0,0,0,0,0");
        let original = create_byod_proof(request(TARGET));
        let edited = create_byod_proof(request(&changed));
        assert_eq!(edited.report.status, ByodVerdict::Verified);
        assert_ne!(original.proof_id, edited.proof_id);
    }

    #[test]
    fn changed_holdout_outcome_is_a_named_contradiction() {
        let changed = TARGET.replace("3,write,1,0,0,0,0,0", "3,write,0,0,0,0,0,0");
        let report = run_byod_transfer(&request(&changed));
        assert_eq!(report.status, ByodVerdict::Contradicted);
        assert_eq!(report.reason_code, ByodReasonCode::TargetContradiction);
        assert_eq!(report.conflicting_episode, Some(3));
        assert!(!report.evidence_admitted);
    }

    #[test]
    fn removing_every_holdout_abstains_after_mapping() {
        let target = TARGET.lines().take(2).collect::<Vec<_>>().join("\n") + "\n";
        let report = run_byod_transfer(&request(&target));
        assert_eq!(report.status, ByodVerdict::Abstain);
        assert_eq!(
            report.reason_code,
            ByodReasonCode::TargetHoldoutEvidenceInsufficient
        );
        assert!(report.mapping_digest.is_some());
    }

    #[test]
    fn ambiguous_structural_roles_abstain_instead_of_guessing() {
        let source = "episode,action,before.a,before.b,before.out,after.a,after.b,after.out\n1,go,1,1,0,1,1,1\n2,go,0,0,0,0,0,1\n";
        let target = "episode,action,before.x,before.y,before.z,after.x,after.y,after.z\n1,do,1,1,0,1,1,1\n2,do,0,0,0,0,0,1\n";
        let report = run_byod_transfer(&ByodTransferRequest::new(source, target));
        assert_eq!(report.status, ByodVerdict::Abstain);
        assert_eq!(report.reason_code, ByodReasonCode::MappingAmbiguous);
    }

    #[test]
    fn parser_rejects_unbounded_or_partial_pairs() {
        let partial = "episode,action,before.x,after.y\n1,go,0,1\n";
        let report = run_byod_transfer(&ByodTransferRequest::new(partial, TARGET));
        assert_eq!(report.status, ByodVerdict::Refused);
        assert_eq!(report.reason_code, ByodReasonCode::SourceMalformed);
    }

    #[test]
    fn target_labels_and_column_order_do_not_carry_meaning() {
        let renamed = TARGET
            .replace("commit", "omega")
            .replace("replica", "alpha")
            .replace("quorum", "beta");
        let report = run_byod_transfer(&request(&renamed));
        assert_eq!(report.status, ByodVerdict::Verified);
        assert_eq!(report.target_effect.as_deref(), Some("omega"));
    }

    #[test]
    fn proof_is_replay_deterministic_and_tamper_evident() {
        let first = create_byod_proof(request(TARGET));
        let second = create_byod_proof(request(TARGET));
        assert_eq!(first, second);
        assert!(verify_byod_proof(&first).verified);
        let mut tampered = first;
        tampered.report.reason.push('!');
        assert!(!verify_byod_proof(&tampered).verified);
    }
}

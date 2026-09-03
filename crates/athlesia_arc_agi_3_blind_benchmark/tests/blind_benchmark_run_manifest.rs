use athlesia_arc_agi_3_blind_benchmark::run_manifest::*;
use athlesia_arc_agi_3_blind_benchmark::{
    ArcAgi3BlindBenchmarkAgentIdentity, ArcAgi3BlindBenchmarkPolicy, ArcAgi3BlindBenchmarkRunId,
    ArcAgi3BlindBenchmarkSpec,
};

const SOURCE_REVISION: &str = "5165e8385b16386b91d8c1543160543fee006e5f";

const CONFIG_FINGERPRINT: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn spec() -> ArcAgi3BlindBenchmarkSpec {
    ArcAgi3BlindBenchmarkSpec::new(
        ArcAgi3BlindBenchmarkRunId::new("blind-manifest-run".to_string()).unwrap(),
        ArcAgi3BlindBenchmarkAgentIdentity::new(
            "athlesia".to_string(),
            "m53-manifest".to_string(),
            SOURCE_REVISION.to_string(),
        )
        .unwrap(),
        ArcAgi3BlindBenchmarkPolicy::new(64).unwrap(),
    )
}

fn harness_identity() -> ArcAgi3BlindBenchmarkHarnessIdentity {
    ArcAgi3BlindBenchmarkHarnessIdentity::new(
        "official-blind-harness".to_string(),
        "2026.09".to_string(),
    )
    .unwrap()
}

fn build_identity() -> ArcAgi3BlindBenchmarkBuildIdentity {
    ArcAgi3BlindBenchmarkBuildIdentity::new(
        SOURCE_REVISION.to_string(),
        "x86_64-unknown-linux-gnu".to_string(),
        "release".to_string(),
    )
    .unwrap()
}

fn protocol_identity() -> ArcAgi3BlindBenchmarkProtocolIdentity {
    ArcAgi3BlindBenchmarkProtocolIdentity::new(
        "arc-agi-3-blind-evaluation".to_string(),
        "2026-09".to_string(),
    )
    .unwrap()
}

fn fingerprint() -> ArcAgi3BlindBenchmarkConfigurationFingerprint {
    ArcAgi3BlindBenchmarkConfigurationFingerprint::new(CONFIG_FINGERPRINT.to_string()).unwrap()
}

fn manifest(deterministic_seed: u64) -> ArcAgi3BlindBenchmarkRunManifest {
    ArcAgi3BlindBenchmarkRunManifest::new(
        spec(),
        harness_identity(),
        build_identity(),
        protocol_identity(),
        fingerprint(),
        deterministic_seed,
    )
    .unwrap()
}

#[test]
fn harness_identity_requires_nonempty_name_and_version() {
    assert_eq!(
        ArcAgi3BlindBenchmarkHarnessIdentity::new(" ".to_string(), "1".to_string(),),
        Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyHarnessName,),
    );

    assert_eq!(
        ArcAgi3BlindBenchmarkHarnessIdentity::new("harness".to_string(), "".to_string(),),
        Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyHarnessVersion,),
    );
}

#[test]
fn build_identity_requires_source_revision_target_and_profile() {
    assert_eq!(
        ArcAgi3BlindBenchmarkBuildIdentity::new(
            "".to_string(),
            "target".to_string(),
            "release".to_string(),
        ),
        Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyBuildSourceRevision,),
    );

    assert_eq!(
        ArcAgi3BlindBenchmarkBuildIdentity::new(
            SOURCE_REVISION.to_string(),
            " ".to_string(),
            "release".to_string(),
        ),
        Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyBuildTarget,),
    );

    assert_eq!(
        ArcAgi3BlindBenchmarkBuildIdentity::new(
            SOURCE_REVISION.to_string(),
            "target".to_string(),
            "".to_string(),
        ),
        Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyBuildProfile,),
    );
}

#[test]
fn protocol_identity_requires_nonempty_name_and_revision() {
    assert_eq!(
        ArcAgi3BlindBenchmarkProtocolIdentity::new("".to_string(), "1".to_string(),),
        Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyProtocolName,),
    );

    assert_eq!(
        ArcAgi3BlindBenchmarkProtocolIdentity::new("protocol".to_string(), " ".to_string(),),
        Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyProtocolRevision,),
    );
}

#[test]
fn configuration_fingerprint_requires_exact_lowercase_sha256_shape() {
    for invalid in [
        "",
        "abc",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0",
        "0123456789ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef",
        "g123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ] {
        assert_eq!(
            ArcAgi3BlindBenchmarkConfigurationFingerprint::new(invalid.to_string(),),
            Err(ArcAgi3BlindBenchmarkRunManifestError::InvalidConfigurationFingerprint,),
        );
    }
}

#[test]
fn valid_configuration_fingerprint_is_preserved_exactly() {
    let fingerprint = fingerprint();

    assert_eq!(fingerprint.as_str(), CONFIG_FINGERPRINT,);
}

#[test]
fn source_revision_mismatch_is_rejected_transactionally() {
    let wrong_build = ArcAgi3BlindBenchmarkBuildIdentity::new(
        "different-revision".to_string(),
        "x86_64-unknown-linux-gnu".to_string(),
        "release".to_string(),
    )
    .unwrap();

    let result = ArcAgi3BlindBenchmarkRunManifest::new(
        spec(),
        harness_identity(),
        wrong_build,
        protocol_identity(),
        fingerprint(),
        7,
    );

    assert_eq!(
        result,
        Err(
            ArcAgi3BlindBenchmarkRunManifestError::SourceRevisionMismatch {
                expected: SOURCE_REVISION.to_string(),
                observed: "different-revision".to_string(),
            },
        ),
    );
}

#[test]
fn manifest_preserves_exact_run_and_agent_identity() {
    let manifest = manifest(41);

    assert_eq!(manifest.spec().run_id().as_str(), "blind-manifest-run",);

    assert_eq!(manifest.spec().agent().name(), "athlesia",);

    assert_eq!(manifest.spec().agent().version(), "m53-manifest",);

    assert_eq!(manifest.spec().agent().source_revision(), SOURCE_REVISION,);
}

#[test]
fn manifest_preserves_exact_harness_identity() {
    let manifest = manifest(41);

    assert_eq!(manifest.harness_identity().name(), "official-blind-harness",);

    assert_eq!(manifest.harness_identity().version(), "2026.09",);
}

#[test]
fn manifest_preserves_exact_build_identity() {
    let manifest = manifest(41);

    assert_eq!(manifest.build_identity().source_revision(), SOURCE_REVISION,);

    assert_eq!(
        manifest.build_identity().target(),
        "x86_64-unknown-linux-gnu",
    );

    assert_eq!(manifest.build_identity().profile(), "release",);
}

#[test]
fn manifest_preserves_exact_protocol_identity() {
    let manifest = manifest(41);

    assert_eq!(
        manifest.protocol_identity().name(),
        "arc-agi-3-blind-evaluation",
    );

    assert_eq!(manifest.protocol_identity().revision(), "2026-09",);
}

#[test]
fn manifest_preserves_exact_deterministic_seed_including_zero() {
    assert_eq!(manifest(0,).deterministic_seed(), 0,);

    assert_eq!(manifest(u64::MAX,).deterministic_seed(), u64::MAX,);
}

#[test]
fn manifest_preserves_exact_episode_step_budget_from_frozen_spec() {
    let manifest = manifest(1);

    assert_eq!(
        manifest.spec().policy().max_cognitive_steps_per_episode(),
        64,
    );
}

#[test]
fn manifest_has_no_mutation_surface_after_construction() {
    let source = include_str!("../src/run_manifest.rs");

    let manifest_start = source
        .find("pub struct ArcAgi3BlindBenchmarkRunManifest")
        .unwrap();

    let facade_start = source
        .find("pub struct UniversalArcAgi3BlindBenchmarkRunManifest")
        .unwrap();

    let manifest_source = &source[manifest_start..facade_start];

    assert!(!manifest_source.contains("&mut self"));

    assert!(!manifest_source.contains("pub fn set_"));

    assert!(!manifest_source.contains("pub fn update"));
}

#[test]
fn run_manifest_contains_no_hidden_catalog_execution_transport_action_or_score_authority() {
    let source = include_str!("../src/run_manifest.rs");

    for forbidden in [
        "game_catalog",
        "environment_catalog",
        "hidden_games",
        "hidden_environments",
        "evaluation_games",
        "public_games",
        "RHAE",
        "baseline_actions",
        ".powi(",
        "Action1",
        "Action2",
        "Action3",
        "Action4",
        "Action5",
        "Action6",
        "Action7",
        "reqwest",
        "ureq",
        "TcpStream",
        "UdpSocket",
        "ArcAgi3CompetitionSession",
        "ArcAgi3RestTransport",
        "ArcAgi3BlindBenchmarkExecutionRuntime",
        "ArcAgi3BlindBenchmarkExternalHarness",
        "open_scorecard",
        "close_scorecard",
        "record_episode",
        ".finalize(",
        "begin_reset",
        ".reset(",
        "retry(",
        "sleep(",
    ] {
        assert!(
            !source.contains(forbidden,),
            "M53 run manifest leaked forbidden benchmark authority: {forbidden}",
        );
    }
}

#[test]
fn universal_facade_matches_direct_run_manifest_construction() {
    let direct = ArcAgi3BlindBenchmarkRunManifest::new(
        spec(),
        harness_identity(),
        build_identity(),
        protocol_identity(),
        fingerprint(),
        99,
    )
    .unwrap();

    let facade = UniversalArcAgi3BlindBenchmarkRunManifest::manifest(
        spec(),
        harness_identity(),
        build_identity(),
        protocol_identity(),
        fingerprint(),
        99,
    )
    .unwrap();

    assert_eq!(direct, facade,);
}

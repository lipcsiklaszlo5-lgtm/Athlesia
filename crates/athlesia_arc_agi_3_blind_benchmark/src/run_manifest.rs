use crate::ArcAgi3BlindBenchmarkSpec;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArcAgi3BlindBenchmarkRunManifestError {
    EmptyHarnessName,
    EmptyHarnessVersion,
    EmptyBuildSourceRevision,
    EmptyBuildTarget,
    EmptyBuildProfile,
    EmptyProtocolName,
    EmptyProtocolRevision,
    InvalidConfigurationFingerprint,
    SourceRevisionMismatch { expected: String, observed: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkHarnessIdentity {
    name: String,
    version: String,
}

impl ArcAgi3BlindBenchmarkHarnessIdentity {
    pub fn new(
        name: String,
        version: String,
    ) -> Result<Self, ArcAgi3BlindBenchmarkRunManifestError> {
        if name.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyHarnessName);
        }

        if version.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyHarnessVersion);
        }

        Ok(Self { name, version })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkBuildIdentity {
    source_revision: String,
    target: String,
    profile: String,
}

impl ArcAgi3BlindBenchmarkBuildIdentity {
    pub fn new(
        source_revision: String,
        target: String,
        profile: String,
    ) -> Result<Self, ArcAgi3BlindBenchmarkRunManifestError> {
        if source_revision.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyBuildSourceRevision);
        }

        if target.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyBuildTarget);
        }

        if profile.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyBuildProfile);
        }

        Ok(Self {
            source_revision,
            target,
            profile,
        })
    }

    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn profile(&self) -> &str {
        &self.profile
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkProtocolIdentity {
    name: String,
    revision: String,
}

impl ArcAgi3BlindBenchmarkProtocolIdentity {
    pub fn new(
        name: String,
        revision: String,
    ) -> Result<Self, ArcAgi3BlindBenchmarkRunManifestError> {
        if name.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyProtocolName);
        }

        if revision.trim().is_empty() {
            return Err(ArcAgi3BlindBenchmarkRunManifestError::EmptyProtocolRevision);
        }

        Ok(Self { name, revision })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkConfigurationFingerprint(String);

impl ArcAgi3BlindBenchmarkConfigurationFingerprint {
    pub fn new(value: String) -> Result<Self, ArcAgi3BlindBenchmarkRunManifestError> {
        let valid = value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));

        if !valid {
            return Err(ArcAgi3BlindBenchmarkRunManifestError::InvalidConfigurationFingerprint);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3BlindBenchmarkRunManifest {
    spec: ArcAgi3BlindBenchmarkSpec,
    harness_identity: ArcAgi3BlindBenchmarkHarnessIdentity,
    build_identity: ArcAgi3BlindBenchmarkBuildIdentity,
    protocol_identity: ArcAgi3BlindBenchmarkProtocolIdentity,
    configuration_fingerprint: ArcAgi3BlindBenchmarkConfigurationFingerprint,
    deterministic_seed: u64,
}

impl ArcAgi3BlindBenchmarkRunManifest {
    pub fn new(
        spec: ArcAgi3BlindBenchmarkSpec,
        harness_identity: ArcAgi3BlindBenchmarkHarnessIdentity,
        build_identity: ArcAgi3BlindBenchmarkBuildIdentity,
        protocol_identity: ArcAgi3BlindBenchmarkProtocolIdentity,
        configuration_fingerprint: ArcAgi3BlindBenchmarkConfigurationFingerprint,
        deterministic_seed: u64,
    ) -> Result<Self, ArcAgi3BlindBenchmarkRunManifestError> {
        let expected_revision = spec.agent().source_revision();

        let observed_revision = build_identity.source_revision();

        if expected_revision != observed_revision {
            return Err(
                ArcAgi3BlindBenchmarkRunManifestError::SourceRevisionMismatch {
                    expected: expected_revision.to_string(),
                    observed: observed_revision.to_string(),
                },
            );
        }

        Ok(Self {
            spec,
            harness_identity,
            build_identity,
            protocol_identity,
            configuration_fingerprint,
            deterministic_seed,
        })
    }

    pub fn spec(&self) -> &ArcAgi3BlindBenchmarkSpec {
        &self.spec
    }

    pub fn harness_identity(&self) -> &ArcAgi3BlindBenchmarkHarnessIdentity {
        &self.harness_identity
    }

    pub fn build_identity(&self) -> &ArcAgi3BlindBenchmarkBuildIdentity {
        &self.build_identity
    }

    pub fn protocol_identity(&self) -> &ArcAgi3BlindBenchmarkProtocolIdentity {
        &self.protocol_identity
    }

    pub fn configuration_fingerprint(&self) -> &ArcAgi3BlindBenchmarkConfigurationFingerprint {
        &self.configuration_fingerprint
    }

    pub fn deterministic_seed(&self) -> u64 {
        self.deterministic_seed
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3BlindBenchmarkRunManifest;

impl UniversalArcAgi3BlindBenchmarkRunManifest {
    pub fn manifest(
        spec: ArcAgi3BlindBenchmarkSpec,
        harness_identity: ArcAgi3BlindBenchmarkHarnessIdentity,
        build_identity: ArcAgi3BlindBenchmarkBuildIdentity,
        protocol_identity: ArcAgi3BlindBenchmarkProtocolIdentity,
        configuration_fingerprint: ArcAgi3BlindBenchmarkConfigurationFingerprint,
        deterministic_seed: u64,
    ) -> Result<ArcAgi3BlindBenchmarkRunManifest, ArcAgi3BlindBenchmarkRunManifestError> {
        ArcAgi3BlindBenchmarkRunManifest::new(
            spec,
            harness_identity,
            build_identity,
            protocol_identity,
            configuration_fingerprint,
            deterministic_seed,
        )
    }
}

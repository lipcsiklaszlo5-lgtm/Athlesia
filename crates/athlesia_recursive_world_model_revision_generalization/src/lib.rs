use std::collections::BTreeMap;

use athlesia_recursive::RecursiveUnit;
use athlesia_recursive_world_model::RecursiveWorldRule;
use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;
use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionGeneralizationThreshold {
    minimum_support: usize,
}

impl RecursiveWorldRevisionGeneralizationThreshold {
    pub fn new(minimum_support: usize, observation_count: usize) -> Option<Self> {
        if minimum_support < 2 || minimum_support > observation_count {
            return None;
        }

        Some(Self { minimum_support })
    }

    pub fn minimum_support(&self) -> usize {
        self.minimum_support
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizationInput {
    target: RecursiveWorldRule,
    observations: RecursiveWorldRevisionInductionObservationSet,
    threshold: RecursiveWorldRevisionGeneralizationThreshold,
}

impl RecursiveWorldRevisionGeneralizationInput {
    pub fn new(
        target: RecursiveWorldRule,
        observations: RecursiveWorldRevisionInductionObservationSet,
        threshold: RecursiveWorldRevisionGeneralizationThreshold,
    ) -> Option<Self> {
        if threshold.minimum_support() > observations.len() {
            return None;
        }

        Some(Self {
            target,
            observations,
            threshold,
        })
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.observations
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionGeneralizationThreshold {
        self.threshold
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizedStructure {
    target: RecursiveWorldRule,
    observations: RecursiveWorldRevisionInductionObservationSet,
    threshold: RecursiveWorldRevisionGeneralizationThreshold,
    generalized_observation: RecursiveWorldRevisionDiscoveryObservation,
    premise_support: BTreeMap<RecursiveUnit, usize>,
    conclusion_support: BTreeMap<RecursiveUnit, usize>,
}

impl RecursiveWorldRevisionGeneralizedStructure {
    pub fn generalize(input: RecursiveWorldRevisionGeneralizationInput) -> Option<Self> {
        let mut premise_support = BTreeMap::<RecursiveUnit, usize>::new();

        let mut conclusion_support = BTreeMap::<RecursiveUnit, usize>::new();

        for observation in input.observations().observations() {
            for unit in observation.premises() {
                let count = premise_support.entry(unit.clone()).or_insert(0);

                *count = count.saturating_add(1);
            }

            for unit in observation.conclusions() {
                let count = conclusion_support.entry(unit.clone()).or_insert(0);

                *count = count.saturating_add(1);
            }
        }

        let minimum_support = input.threshold().minimum_support();

        let generalized_premises = premise_support
            .iter()
            .filter_map(|(unit, support)| {
                if *support >= minimum_support {
                    Some(unit.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let generalized_conclusions = conclusion_support
            .iter()
            .filter_map(|(unit, support)| {
                if *support >= minimum_support {
                    Some(unit.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let generalized_observation = RecursiveWorldRevisionDiscoveryObservation::new(
            generalized_premises,
            generalized_conclusions,
        )?;

        Some(Self {
            target: input.target().clone(),
            observations: input.observations().clone(),
            threshold: input.threshold(),
            generalized_observation,
            premise_support,
            conclusion_support,
        })
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.observations
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionGeneralizationThreshold {
        self.threshold
    }

    pub fn generalized_observation(&self) -> &RecursiveWorldRevisionDiscoveryObservation {
        &self.generalized_observation
    }

    pub fn generalized_premises(&self) -> &[RecursiveUnit] {
        self.generalized_observation.premises()
    }

    pub fn generalized_conclusions(&self) -> &[RecursiveUnit] {
        self.generalized_observation.conclusions()
    }

    pub fn premise_support(&self, unit: &RecursiveUnit) -> usize {
        self.premise_support.get(unit).copied().unwrap_or(0)
    }

    pub fn conclusion_support(&self, unit: &RecursiveUnit) -> usize {
        self.conclusion_support.get(unit).copied().unwrap_or(0)
    }

    pub fn support_count(&self) -> usize {
        self.observations.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizer;

impl RecursiveWorldRevisionGeneralizer {
    pub fn generalize(
        input: RecursiveWorldRevisionGeneralizationInput,
    ) -> Option<RecursiveWorldRevisionGeneralizedStructure> {
        RecursiveWorldRevisionGeneralizedStructure::generalize(input)
    }
}

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryHypothesis;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizationDiscoveryBridge {
    generalized: RecursiveWorldRevisionGeneralizedStructure,
    hypothesis: RecursiveWorldRevisionDiscoveryHypothesis,
}

impl RecursiveWorldRevisionGeneralizationDiscoveryBridge {
    pub fn new(generalized: RecursiveWorldRevisionGeneralizedStructure) -> Option<Self> {
        let hypothesis = RecursiveWorldRevisionDiscoveryHypothesis::discover(
            generalized.target().clone(),
            generalized.generalized_observation().clone(),
        )?;

        Some(Self {
            generalized,
            hypothesis,
        })
    }

    pub fn generalized(&self) -> &RecursiveWorldRevisionGeneralizedStructure {
        &self.generalized
    }

    pub fn hypothesis(&self) -> &RecursiveWorldRevisionDiscoveryHypothesis {
        &self.hypothesis
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        self.hypothesis.target()
    }

    pub fn replacement(&self) -> &RecursiveWorldRule {
        self.hypothesis.replacement()
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionGeneralizationThreshold {
        self.generalized.threshold()
    }

    pub fn support_count(&self) -> usize {
        self.generalized.support_count()
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.generalized.observations()
    }

    pub fn premise_support(&self, unit: &RecursiveUnit) -> usize {
        self.generalized.premise_support(unit)
    }

    pub fn conclusion_support(&self, unit: &RecursiveUnit) -> usize {
        self.generalized.conclusion_support(unit)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizationDiscoveryBridgeBuilder;

impl RecursiveWorldRevisionGeneralizationDiscoveryBridgeBuilder {
    pub fn build(
        generalized: RecursiveWorldRevisionGeneralizedStructure,
    ) -> Option<RecursiveWorldRevisionGeneralizationDiscoveryBridge> {
        RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized)
    }
}

use athlesia_recursive_world_model::RecursiveWorldModel;

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryHypothesisSet, RecursiveWorldRevisionDiscoveryValidation,
    RecursiveWorldRevisionDiscoveryValidator,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionGeneralizationValidationStatus {
    DiscoveryUnavailable,
    Rejected,
    Accepted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizationValidation {
    generalized: RecursiveWorldRevisionGeneralizedStructure,
    bridge: Option<RecursiveWorldRevisionGeneralizationDiscoveryBridge>,
    discovery_validation: Option<RecursiveWorldRevisionDiscoveryValidation>,
    status: RecursiveWorldRevisionGeneralizationValidationStatus,
}

impl RecursiveWorldRevisionGeneralizationValidation {
    pub fn new(
        model: &RecursiveWorldModel,
        generalized: RecursiveWorldRevisionGeneralizedStructure,
    ) -> Self {
        let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized.clone());

        let Some(bridge_value) = bridge.clone() else {
            return Self {
                generalized,
                bridge: None,
                discovery_validation: None,
                status: RecursiveWorldRevisionGeneralizationValidationStatus::DiscoveryUnavailable,
            };
        };

        let discovery_validation = RecursiveWorldRevisionDiscoveryValidator::validate(
            model,
            RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![bridge_value
                .hypothesis()
                .clone()]),
        );

        let status = if discovery_validation.accepted_count() == 1 {
            RecursiveWorldRevisionGeneralizationValidationStatus::Accepted
        } else {
            RecursiveWorldRevisionGeneralizationValidationStatus::Rejected
        };

        Self {
            generalized,
            bridge,
            discovery_validation: Some(discovery_validation),
            status,
        }
    }

    pub fn generalized(&self) -> &RecursiveWorldRevisionGeneralizedStructure {
        &self.generalized
    }

    pub fn bridge(&self) -> Option<&RecursiveWorldRevisionGeneralizationDiscoveryBridge> {
        self.bridge.as_ref()
    }

    pub fn discovery_validation(&self) -> Option<&RecursiveWorldRevisionDiscoveryValidation> {
        self.discovery_validation.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionGeneralizationValidationStatus {
        self.status
    }

    pub fn is_accepted(&self) -> bool {
        self.status == RecursiveWorldRevisionGeneralizationValidationStatus::Accepted
    }

    pub fn is_rejected(&self) -> bool {
        self.status == RecursiveWorldRevisionGeneralizationValidationStatus::Rejected
    }

    pub fn is_discovery_unavailable(&self) -> bool {
        self.status == RecursiveWorldRevisionGeneralizationValidationStatus::DiscoveryUnavailable
    }

    pub fn accepted_hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.discovery_validation
            .as_ref()
            .and_then(|validation| validation.accepted_hypotheses().first())
    }

    pub fn rejected_hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.discovery_validation
            .as_ref()
            .and_then(|validation| validation.rejected_hypotheses().first())
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionGeneralizationThreshold {
        self.generalized.threshold()
    }

    pub fn support_count(&self) -> usize {
        self.generalized.support_count()
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.generalized.observations()
    }

    pub fn premise_support(&self, unit: &RecursiveUnit) -> usize {
        self.generalized.premise_support(unit)
    }

    pub fn conclusion_support(&self, unit: &RecursiveUnit) -> usize {
        self.generalized.conclusion_support(unit)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizationValidator;

impl RecursiveWorldRevisionGeneralizationValidator {
    pub fn validate(
        model: &RecursiveWorldModel,
        generalized: RecursiveWorldRevisionGeneralizedStructure,
    ) -> RecursiveWorldRevisionGeneralizationValidation {
        RecursiveWorldRevisionGeneralizationValidation::new(model, generalized)
    }
}

use athlesia_recursive_world_model_evidence::RecursiveWorldEvidenceState;

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryEvidenceScope, RecursiveWorldRevisionDiscoveryEvidenceScoper,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionGeneralizationEvidenceStatus {
    DiscoveryUnavailable,
    Rejected,
    Inactive,
    Active,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizationEvidenceScope {
    validation: RecursiveWorldRevisionGeneralizationValidation,
    discovery_scope: Option<RecursiveWorldRevisionDiscoveryEvidenceScope>,
    status: RecursiveWorldRevisionGeneralizationEvidenceStatus,
}

impl RecursiveWorldRevisionGeneralizationEvidenceScope {
    pub fn new(
        model: &RecursiveWorldModel,
        evidence_state: &RecursiveWorldEvidenceState,
        generalized: RecursiveWorldRevisionGeneralizedStructure,
    ) -> Self {
        let validation = RecursiveWorldRevisionGeneralizationValidation::new(model, generalized);

        if validation.is_discovery_unavailable() {
            return Self {
                validation,
                discovery_scope: None,
                status: RecursiveWorldRevisionGeneralizationEvidenceStatus::DiscoveryUnavailable,
            };
        }

        if validation.is_rejected() {
            return Self {
                validation,
                discovery_scope: None,
                status: RecursiveWorldRevisionGeneralizationEvidenceStatus::Rejected,
            };
        }

        let hypothesis = validation
            .accepted_hypothesis()
            .expect("accepted generalization validation must expose one discovery hypothesis")
            .clone();

        let discovery_scope = RecursiveWorldRevisionDiscoveryEvidenceScoper::scope(
            model,
            evidence_state,
            RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis]),
        );

        let status = if discovery_scope.active_count() == 1 {
            RecursiveWorldRevisionGeneralizationEvidenceStatus::Active
        } else {
            RecursiveWorldRevisionGeneralizationEvidenceStatus::Inactive
        };

        Self {
            validation,
            discovery_scope: Some(discovery_scope),
            status,
        }
    }

    pub fn validation(&self) -> &RecursiveWorldRevisionGeneralizationValidation {
        &self.validation
    }

    pub fn discovery_scope(&self) -> Option<&RecursiveWorldRevisionDiscoveryEvidenceScope> {
        self.discovery_scope.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionGeneralizationEvidenceStatus {
        self.status
    }

    pub fn is_active(&self) -> bool {
        self.status == RecursiveWorldRevisionGeneralizationEvidenceStatus::Active
    }

    pub fn is_inactive(&self) -> bool {
        self.status == RecursiveWorldRevisionGeneralizationEvidenceStatus::Inactive
    }

    pub fn is_rejected(&self) -> bool {
        self.status == RecursiveWorldRevisionGeneralizationEvidenceStatus::Rejected
    }

    pub fn is_discovery_unavailable(&self) -> bool {
        self.status == RecursiveWorldRevisionGeneralizationEvidenceStatus::DiscoveryUnavailable
    }

    pub fn pressured_rule(&self) -> Option<&RecursiveWorldRule> {
        self.discovery_scope
            .as_ref()
            .and_then(|scope| scope.pressured_rule())
    }

    pub fn active_hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.discovery_scope
            .as_ref()
            .and_then(|scope| scope.active_hypotheses().first())
    }

    pub fn inactive_hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.discovery_scope
            .as_ref()
            .and_then(|scope| scope.inactive_hypotheses().first())
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionGeneralizationThreshold {
        self.validation.threshold()
    }

    pub fn support_count(&self) -> usize {
        self.validation.support_count()
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.validation.source_observations()
    }

    pub fn premise_support(&self, unit: &RecursiveUnit) -> usize {
        self.validation.premise_support(unit)
    }

    pub fn conclusion_support(&self, unit: &RecursiveUnit) -> usize {
        self.validation.conclusion_support(unit)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizationEvidenceScoper;

impl RecursiveWorldRevisionGeneralizationEvidenceScoper {
    pub fn scope(
        model: &RecursiveWorldModel,
        evidence_state: &RecursiveWorldEvidenceState,
        generalized: RecursiveWorldRevisionGeneralizedStructure,
    ) -> RecursiveWorldRevisionGeneralizationEvidenceScope {
        RecursiveWorldRevisionGeneralizationEvidenceScope::new(model, evidence_state, generalized)
    }
}

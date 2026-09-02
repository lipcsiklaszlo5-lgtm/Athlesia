use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IntegratedCognitiveLayer {
    PerceptualGrounding,
    UniversalDomainLearning,
    ExecutiveAgency,
    MetaLearningSkillMemory,
    AutonomousExperimentation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedLayerContribution {
    layer: IntegratedCognitiveLayer,
    anchor_state: CognitiveStructure,
    result_state: CognitiveStructure,
    provenance: CognitiveStructure,
    confidence: CognitiveSignal,
    compute_cost: CognitiveSignal,
}

impl IntegratedLayerContribution {
    pub fn new(
        layer: IntegratedCognitiveLayer,
        anchor_state: CognitiveStructure,
        result_state: CognitiveStructure,
        provenance: CognitiveStructure,
        confidence: CognitiveSignal,
        compute_cost: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            layer,
            anchor_state,
            result_state,
            provenance,
            confidence,
            compute_cost,
        })
    }

    pub fn layer(&self) -> IntegratedCognitiveLayer {
        self.layer
    }

    pub fn anchor_state(&self) -> &CognitiveStructure {
        &self.anchor_state
    }

    pub fn result_state(&self) -> &CognitiveStructure {
        &self.result_state
    }

    pub fn provenance(&self) -> &CognitiveStructure {
        &self.provenance
    }

    pub fn confidence(&self) -> CognitiveSignal {
        self.confidence
    }

    pub fn compute_cost(&self) -> CognitiveSignal {
        self.compute_cost
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegratedAgentBounds {
    max_input_contributions: usize,
    max_total_compute: u32,
}

impl IntegratedAgentBounds {
    pub fn new(max_input_contributions: usize, max_total_compute: u32) -> Option<Self> {
        if max_input_contributions == 0 || max_input_contributions > 5 || max_total_compute == 0 {
            return None;
        }

        Some(Self {
            max_input_contributions,
            max_total_compute,
        })
    }

    pub fn max_input_contributions(self) -> usize {
        self.max_input_contributions
    }

    pub fn max_total_compute(self) -> u32 {
        self.max_total_compute
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegratedAgentThresholds {
    minimum_contribution_confidence: CognitiveSignal,
}

impl IntegratedAgentThresholds {
    pub fn new(minimum_contribution_confidence: CognitiveSignal) -> Option<Self> {
        if minimum_contribution_confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            minimum_contribution_confidence,
        })
    }

    pub fn minimum_contribution_confidence(self) -> CognitiveSignal {
        self.minimum_contribution_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegratedAgentPolicy {
    bounds: IntegratedAgentBounds,
    thresholds: IntegratedAgentThresholds,
}

impl IntegratedAgentPolicy {
    pub fn new(bounds: IntegratedAgentBounds, thresholds: IntegratedAgentThresholds) -> Self {
        Self { bounds, thresholds }
    }

    pub fn bounds(self) -> IntegratedAgentBounds {
        self.bounds
    }

    pub fn thresholds(self) -> IntegratedAgentThresholds {
        self.thresholds
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegratedAgentFoundationStatus {
    Integrated,
    NoQualifyingContributions,
    InputFrontierExceeded,
    ConflictingProvenance,
    DuplicateLayerContribution,
    ComputeBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedCognitiveFrame {
    anchor_state: CognitiveStructure,
    contributions: Vec<IntegratedLayerContribution>,
    total_compute: u32,
}

impl IntegratedCognitiveFrame {
    pub fn anchor_state(&self) -> &CognitiveStructure {
        &self.anchor_state
    }

    pub fn contributions(&self) -> &[IntegratedLayerContribution] {
        &self.contributions
    }

    pub fn contribution_count(&self) -> usize {
        self.contributions.len()
    }

    pub fn total_compute(&self) -> u32 {
        self.total_compute
    }

    pub fn contribution(
        &self,
        layer: IntegratedCognitiveLayer,
    ) -> Option<&IntegratedLayerContribution> {
        self.contributions
            .iter()
            .find(|contribution| contribution.layer() == layer)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedAgentFoundationResult {
    status: IntegratedAgentFoundationStatus,
    input_contribution_count: usize,
    unique_contribution_count: usize,
    qualifying_contribution_count: usize,
    rejected_anchor_count: usize,
    rejected_confidence_count: usize,
    frame: Option<IntegratedCognitiveFrame>,
}

impl IntegratedAgentFoundationResult {
    pub fn status(&self) -> IntegratedAgentFoundationStatus {
        self.status
    }

    pub fn input_contribution_count(&self) -> usize {
        self.input_contribution_count
    }

    pub fn unique_contribution_count(&self) -> usize {
        self.unique_contribution_count
    }

    pub fn qualifying_contribution_count(&self) -> usize {
        self.qualifying_contribution_count
    }

    pub fn rejected_anchor_count(&self) -> usize {
        self.rejected_anchor_count
    }

    pub fn rejected_confidence_count(&self) -> usize {
        self.rejected_confidence_count
    }

    pub fn frame(&self) -> Option<&IntegratedCognitiveFrame> {
        self.frame.as_ref()
    }

    pub fn integrated(&self) -> bool {
        self.status == IntegratedAgentFoundationStatus::Integrated
    }

    pub fn abstained(&self) -> bool {
        self.frame.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IntegratedCognitiveAgentFoundation;

impl IntegratedCognitiveAgentFoundation {
    fn empty(
        status: IntegratedAgentFoundationStatus,
        input_contribution_count: usize,
        unique_contribution_count: usize,
        qualifying_contribution_count: usize,
        rejected_anchor_count: usize,
        rejected_confidence_count: usize,
    ) -> IntegratedAgentFoundationResult {
        IntegratedAgentFoundationResult {
            status,
            input_contribution_count,
            unique_contribution_count,
            qualifying_contribution_count,
            rejected_anchor_count,
            rejected_confidence_count,
            frame: None,
        }
    }

    pub fn integrate(
        anchor_state: &CognitiveStructure,
        contributions: &[IntegratedLayerContribution],
        policy: IntegratedAgentPolicy,
    ) -> IntegratedAgentFoundationResult {
        let bounds = policy.bounds();
        let thresholds = policy.thresholds();

        let input_contribution_count = contributions.len();

        if input_contribution_count > bounds.max_input_contributions() {
            return Self::empty(
                IntegratedAgentFoundationStatus::InputFrontierExceeded,
                input_contribution_count,
                0,
                0,
                0,
                0,
            );
        }

        let mut ordered = contributions.to_vec();

        ordered.sort_by(|left, right| {
            format!("{:?}", left.provenance())
                .cmp(&format!("{:?}", right.provenance()))
                .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
        });

        let mut canonical: Vec<IntegratedLayerContribution> = Vec::new();

        for contribution in ordered {
            if let Some(existing) = canonical
                .iter()
                .find(|existing| existing.provenance() == contribution.provenance())
            {
                if existing != &contribution {
                    return Self::empty(
                        IntegratedAgentFoundationStatus::ConflictingProvenance,
                        input_contribution_count,
                        canonical.len(),
                        0,
                        0,
                        0,
                    );
                }

                continue;
            }

            canonical.push(contribution);
        }

        let unique_contribution_count = canonical.len();

        let mut rejected_anchor_count = 0usize;

        let mut rejected_confidence_count = 0usize;

        let mut qualifying: Vec<IntegratedLayerContribution> = canonical
            .into_iter()
            .filter(|contribution| {
                if contribution.anchor_state() != anchor_state {
                    rejected_anchor_count += 1;
                    return false;
                }

                if contribution.confidence().value()
                    < thresholds.minimum_contribution_confidence().value()
                {
                    rejected_confidence_count += 1;
                    return false;
                }

                true
            })
            .collect();

        let qualifying_contribution_count = qualifying.len();

        if qualifying.is_empty() {
            return Self::empty(
                IntegratedAgentFoundationStatus::NoQualifyingContributions,
                input_contribution_count,
                unique_contribution_count,
                qualifying_contribution_count,
                rejected_anchor_count,
                rejected_confidence_count,
            );
        }

        qualifying.sort_by_key(IntegratedLayerContribution::layer);

        for index in 1..qualifying.len() {
            if qualifying[index - 1].layer() == qualifying[index].layer() {
                return Self::empty(
                    IntegratedAgentFoundationStatus::DuplicateLayerContribution,
                    input_contribution_count,
                    unique_contribution_count,
                    qualifying_contribution_count,
                    rejected_anchor_count,
                    rejected_confidence_count,
                );
            }
        }

        let total_compute = qualifying.iter().fold(0u32, |total, contribution| {
            total.saturating_add(u32::from(contribution.compute_cost().value()))
        });

        if total_compute > bounds.max_total_compute() {
            return Self::empty(
                IntegratedAgentFoundationStatus::ComputeBudgetExceeded,
                input_contribution_count,
                unique_contribution_count,
                qualifying_contribution_count,
                rejected_anchor_count,
                rejected_confidence_count,
            );
        }

        IntegratedAgentFoundationResult {
            status: IntegratedAgentFoundationStatus::Integrated,
            input_contribution_count,
            unique_contribution_count,
            qualifying_contribution_count,
            rejected_anchor_count,
            rejected_confidence_count,
            frame: Some(IntegratedCognitiveFrame {
                anchor_state: anchor_state.clone(),
                contributions: qualifying,
                total_compute,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalIntegratedCognitiveAgentFoundation;

impl UniversalIntegratedCognitiveAgentFoundation {
    pub fn evaluate(
        anchor_state: &CognitiveStructure,
        contributions: &[IntegratedLayerContribution],
        policy: IntegratedAgentPolicy,
    ) -> IntegratedAgentFoundationResult {
        IntegratedCognitiveAgentFoundation::integrate(anchor_state, contributions, policy)
    }
}

#[cfg(test)]
mod integrated_cognitive_agent_foundation_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn contribution(
        layer: IntegratedCognitiveLayer,
        anchor: u64,
        result: u64,
        provenance: u64,
        confidence: u16,
        cost: u16,
    ) -> IntegratedLayerContribution {
        IntegratedLayerContribution::new(
            layer,
            a(anchor),
            a(result),
            a(provenance),
            s(confidence),
            s(cost),
        )
        .unwrap()
    }

    fn policy() -> IntegratedAgentPolicy {
        IntegratedAgentPolicy::new(
            IntegratedAgentBounds::new(5, 3000).unwrap(),
            IntegratedAgentThresholds::new(s(500)).unwrap(),
        )
    }

    #[test]
    fn foundation_contract_requires_positive_confidence_and_bounded_resources() {
        assert_eq!(
            IntegratedLayerContribution::new(
                IntegratedCognitiveLayer::PerceptualGrounding,
                a(1),
                a(2),
                a(3),
                s(0),
                s(100),
            ),
            None
        );

        assert_eq!(IntegratedAgentBounds::new(0, 1000,), None);

        assert_eq!(IntegratedAgentBounds::new(6, 1000,), None);

        assert_eq!(IntegratedAgentThresholds::new(s(0),), None);
    }

    #[test]
    fn grounded_cross_layer_contributions_form_integrated_cognitive_frame() {
        let result = IntegratedCognitiveAgentFoundation::integrate(
            &a(1),
            &[
                contribution(
                    IntegratedCognitiveLayer::PerceptualGrounding,
                    1,
                    10,
                    100,
                    900,
                    200,
                ),
                contribution(
                    IntegratedCognitiveLayer::UniversalDomainLearning,
                    1,
                    20,
                    101,
                    800,
                    300,
                ),
            ],
            policy(),
        );

        assert!(result.integrated());

        assert_eq!(result.frame().unwrap().contribution_count(), 2);

        assert_eq!(result.frame().unwrap().anchor_state(), &a(1));
    }

    #[test]
    fn contribution_from_different_world_state_is_rejected() {
        let result = IntegratedCognitiveAgentFoundation::integrate(
            &a(1),
            &[contribution(
                IntegratedCognitiveLayer::PerceptualGrounding,
                2,
                10,
                100,
                900,
                200,
            )],
            policy(),
        );

        assert_eq!(
            result.status(),
            IntegratedAgentFoundationStatus::NoQualifyingContributions
        );

        assert_eq!(result.rejected_anchor_count(), 1);
    }

    #[test]
    fn low_confidence_layer_output_cannot_enter_integrated_frame() {
        let result = IntegratedCognitiveAgentFoundation::integrate(
            &a(1),
            &[contribution(
                IntegratedCognitiveLayer::ExecutiveAgency,
                1,
                10,
                100,
                400,
                200,
            )],
            policy(),
        );

        assert_eq!(
            result.status(),
            IntegratedAgentFoundationStatus::NoQualifyingContributions
        );

        assert_eq!(result.rejected_confidence_count(), 1);
    }

    #[test]
    fn exact_duplicate_provenance_is_deduplicated_without_compute_inflation() {
        let item = contribution(
            IntegratedCognitiveLayer::PerceptualGrounding,
            1,
            10,
            100,
            900,
            200,
        );

        let result =
            IntegratedCognitiveAgentFoundation::integrate(&a(1), &[item.clone(), item], policy());

        assert_eq!(result.input_contribution_count(), 2);

        assert_eq!(result.unique_contribution_count(), 1);

        assert_eq!(result.frame().unwrap().total_compute(), 200);
    }

    #[test]
    fn conflicting_reuse_of_exact_provenance_abstains_atomically() {
        let result = IntegratedCognitiveAgentFoundation::integrate(
            &a(1),
            &[
                contribution(
                    IntegratedCognitiveLayer::PerceptualGrounding,
                    1,
                    10,
                    100,
                    900,
                    200,
                ),
                contribution(
                    IntegratedCognitiveLayer::PerceptualGrounding,
                    1,
                    11,
                    100,
                    900,
                    200,
                ),
            ],
            policy(),
        );

        assert_eq!(
            result.status(),
            IntegratedAgentFoundationStatus::ConflictingProvenance
        );

        assert!(result.frame().is_none());
    }

    #[test]
    fn multiple_authoritative_outputs_from_same_layer_abstain() {
        let result = IntegratedCognitiveAgentFoundation::integrate(
            &a(1),
            &[
                contribution(
                    IntegratedCognitiveLayer::ExecutiveAgency,
                    1,
                    10,
                    100,
                    900,
                    200,
                ),
                contribution(
                    IntegratedCognitiveLayer::ExecutiveAgency,
                    1,
                    11,
                    101,
                    900,
                    200,
                ),
            ],
            policy(),
        );

        assert_eq!(
            result.status(),
            IntegratedAgentFoundationStatus::DuplicateLayerContribution
        );

        assert!(result.abstained());
    }

    #[test]
    fn hard_compute_budget_abstains_without_partial_frame() {
        let bounded = IntegratedAgentPolicy::new(
            IntegratedAgentBounds::new(5, 300).unwrap(),
            IntegratedAgentThresholds::new(s(500)).unwrap(),
        );

        let result = IntegratedCognitiveAgentFoundation::integrate(
            &a(1),
            &[
                contribution(
                    IntegratedCognitiveLayer::PerceptualGrounding,
                    1,
                    10,
                    100,
                    900,
                    200,
                ),
                contribution(
                    IntegratedCognitiveLayer::UniversalDomainLearning,
                    1,
                    20,
                    101,
                    900,
                    200,
                ),
            ],
            bounded,
        );

        assert_eq!(
            result.status(),
            IntegratedAgentFoundationStatus::ComputeBudgetExceeded
        );

        assert!(result.frame().is_none());
    }

    #[test]
    fn exact_cognitive_state_identity_is_semantic_authority() {
        let result = IntegratedCognitiveAgentFoundation::integrate(
            &a(10),
            &[
                contribution(
                    IntegratedCognitiveLayer::MetaLearningSkillMemory,
                    11,
                    20,
                    100,
                    900,
                    100,
                ),
                contribution(
                    IntegratedCognitiveLayer::AutonomousExperimentation,
                    10,
                    30,
                    101,
                    900,
                    100,
                ),
            ],
            policy(),
        );

        assert_eq!(result.qualifying_contribution_count(), 1);

        assert_eq!(
            result.frame().unwrap().contributions()[0].layer(),
            IntegratedCognitiveLayer::AutonomousExperimentation
        );
    }

    #[test]
    fn canonical_layer_order_is_independent_of_input_order() {
        let first = contribution(
            IntegratedCognitiveLayer::AutonomousExperimentation,
            1,
            50,
            105,
            900,
            100,
        );

        let second = contribution(
            IntegratedCognitiveLayer::PerceptualGrounding,
            1,
            10,
            101,
            900,
            100,
        );

        let third = contribution(
            IntegratedCognitiveLayer::ExecutiveAgency,
            1,
            30,
            103,
            900,
            100,
        );

        let result =
            IntegratedCognitiveAgentFoundation::integrate(&a(1), &[first, second, third], policy());

        let layers: Vec<IntegratedCognitiveLayer> = result
            .frame()
            .unwrap()
            .contributions()
            .iter()
            .map(IntegratedLayerContribution::layer)
            .collect();

        assert_eq!(
            layers,
            vec![
                IntegratedCognitiveLayer::PerceptualGrounding,
                IntegratedCognitiveLayer::ExecutiveAgency,
                IntegratedCognitiveLayer::AutonomousExperimentation,
            ]
        );
    }

    #[test]
    fn all_five_cognitive_layers_fit_one_bounded_integrated_frame() {
        let result = IntegratedCognitiveAgentFoundation::integrate(
            &a(1),
            &[
                contribution(
                    IntegratedCognitiveLayer::PerceptualGrounding,
                    1,
                    10,
                    100,
                    900,
                    100,
                ),
                contribution(
                    IntegratedCognitiveLayer::UniversalDomainLearning,
                    1,
                    20,
                    101,
                    900,
                    100,
                ),
                contribution(
                    IntegratedCognitiveLayer::ExecutiveAgency,
                    1,
                    30,
                    102,
                    900,
                    100,
                ),
                contribution(
                    IntegratedCognitiveLayer::MetaLearningSkillMemory,
                    1,
                    40,
                    103,
                    900,
                    100,
                ),
                contribution(
                    IntegratedCognitiveLayer::AutonomousExperimentation,
                    1,
                    50,
                    104,
                    900,
                    100,
                ),
            ],
            policy(),
        );

        assert!(result.integrated());

        assert_eq!(result.frame().unwrap().contribution_count(), 5);

        assert_eq!(result.frame().unwrap().total_compute(), 500);
    }

    #[test]
    fn foundation_is_order_invariant_non_mutating_deterministic_and_facade_equivalent() {
        let contributions = vec![
            contribution(
                IntegratedCognitiveLayer::PerceptualGrounding,
                1,
                10,
                100,
                900,
                100,
            ),
            contribution(
                IntegratedCognitiveLayer::UniversalDomainLearning,
                1,
                20,
                101,
                800,
                200,
            ),
            contribution(
                IntegratedCognitiveLayer::ExecutiveAgency,
                1,
                30,
                102,
                700,
                300,
            ),
        ];

        let before = contributions.clone();

        let mut reversed = contributions.clone();

        reversed.reverse();

        let p = policy();

        let direct = IntegratedCognitiveAgentFoundation::integrate(&a(1), &contributions, p);

        let reordered = IntegratedCognitiveAgentFoundation::integrate(&a(1), &reversed, p);

        let facade =
            UniversalIntegratedCognitiveAgentFoundation::evaluate(&a(1), &contributions, p);

        let repeated =
            UniversalIntegratedCognitiveAgentFoundation::evaluate(&a(1), &contributions, p);

        assert_eq!(direct, reordered);

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(contributions, before);
    }
}

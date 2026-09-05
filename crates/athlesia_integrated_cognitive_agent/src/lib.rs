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

use athlesia_core_knowledge_perceptual_grounding::{
    CoreKnowledgePerceptualWorld, IntegratedPerceptualWorldContext, IntegratedPerceptualWorldInput,
    IntegratedPerceptualWorldResult,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualGroundingIngestionRequest {
    anchor_state: CognitiveStructure,
    grounded_state: CognitiveStructure,
    provenance: CognitiveStructure,
    confidence: CognitiveSignal,
    compute_cost: CognitiveSignal,
}

impl PerceptualGroundingIngestionRequest {
    pub fn new(
        anchor_state: CognitiveStructure,
        grounded_state: CognitiveStructure,
        provenance: CognitiveStructure,
        confidence: CognitiveSignal,
        compute_cost: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            anchor_state,
            grounded_state,
            provenance,
            confidence,
            compute_cost,
        })
    }

    pub fn anchor_state(&self) -> &CognitiveStructure {
        &self.anchor_state
    }

    pub fn grounded_state(&self) -> &CognitiveStructure {
        &self.grounded_state
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
pub struct PerceptualGroundingIngestionBounds {
    max_previous_frame_elements: usize,
    max_current_frame_elements: usize,
    max_selected_outputs: usize,
    max_dependency_rejections: usize,
}

impl PerceptualGroundingIngestionBounds {
    pub fn new(
        max_previous_frame_elements: usize,
        max_current_frame_elements: usize,
        max_selected_outputs: usize,
        max_dependency_rejections: usize,
    ) -> Option<Self> {
        if max_previous_frame_elements == 0 || max_current_frame_elements == 0 {
            return None;
        }

        Some(Self {
            max_previous_frame_elements,
            max_current_frame_elements,
            max_selected_outputs,
            max_dependency_rejections,
        })
    }

    pub fn max_previous_frame_elements(self) -> usize {
        self.max_previous_frame_elements
    }

    pub fn max_current_frame_elements(self) -> usize {
        self.max_current_frame_elements
    }

    pub fn max_selected_outputs(self) -> usize {
        self.max_selected_outputs
    }

    pub fn max_dependency_rejections(self) -> usize {
        self.max_dependency_rejections
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PerceptualGroundingIngestionPolicy {
    bounds: PerceptualGroundingIngestionBounds,
}

impl PerceptualGroundingIngestionPolicy {
    pub fn new(bounds: PerceptualGroundingIngestionBounds) -> Self {
        Self { bounds }
    }

    pub fn bounds(self) -> PerceptualGroundingIngestionBounds {
        self.bounds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualGroundingDigest {
    previous_observation_index: u64,
    current_observation_index: u64,
    previous_frame_element_count: usize,
    current_frame_element_count: usize,
    selected_previous_scene_count: usize,
    selected_current_scene_count: usize,
    selected_persistence_count: usize,
    selected_topology_count: usize,
    selected_change_count: usize,
    selected_action_consequence_count: usize,
    rejected_dependency_count: usize,
}

impl PerceptualGroundingDigest {
    pub fn from_world(
        input: &IntegratedPerceptualWorldInput,
        world: &IntegratedPerceptualWorldResult,
    ) -> Self {
        let rejected_dependency_count = world
            .rejected_persistence_dependency_count()
            .saturating_add(world.rejected_topology_dependency_count())
            .saturating_add(world.rejected_change_dependency_count())
            .saturating_add(world.rejected_action_consequence_dependency_count());

        Self {
            previous_observation_index: input.previous_frame().observation_index(),
            current_observation_index: input.current_frame().observation_index(),
            previous_frame_element_count: input.previous_frame().element_count(),
            current_frame_element_count: input.current_frame().element_count(),
            selected_previous_scene_count: world.previous_scene().selected_count(),
            selected_current_scene_count: world.current_scene().selected_count(),
            selected_persistence_count: world.persistence().selected_count(),
            selected_topology_count: world.topology().selected_count(),
            selected_change_count: world.changes().selected_count(),
            selected_action_consequence_count: world.action_consequences().selected_count(),
            rejected_dependency_count,
        }
    }

    pub fn previous_observation_index(&self) -> u64 {
        self.previous_observation_index
    }

    pub fn current_observation_index(&self) -> u64 {
        self.current_observation_index
    }

    pub fn previous_frame_element_count(&self) -> usize {
        self.previous_frame_element_count
    }

    pub fn current_frame_element_count(&self) -> usize {
        self.current_frame_element_count
    }

    pub fn selected_previous_scene_count(&self) -> usize {
        self.selected_previous_scene_count
    }

    pub fn selected_current_scene_count(&self) -> usize {
        self.selected_current_scene_count
    }

    pub fn selected_persistence_count(&self) -> usize {
        self.selected_persistence_count
    }

    pub fn selected_topology_count(&self) -> usize {
        self.selected_topology_count
    }

    pub fn selected_change_count(&self) -> usize {
        self.selected_change_count
    }

    pub fn selected_action_consequence_count(&self) -> usize {
        self.selected_action_consequence_count
    }

    pub fn rejected_dependency_count(&self) -> usize {
        self.rejected_dependency_count
    }

    pub fn total_selected_outputs(&self) -> usize {
        self.selected_previous_scene_count
            .saturating_add(self.selected_current_scene_count)
            .saturating_add(self.selected_persistence_count)
            .saturating_add(self.selected_topology_count)
            .saturating_add(self.selected_change_count)
            .saturating_add(self.selected_action_consequence_count)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerceptualGroundingIngestionStatus {
    Ingested,
    FrameFrontierExceeded,
    SelectedOutputFrontierExceeded,
    DependencyRejectionFrontierExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualGroundingIngestionResult {
    status: PerceptualGroundingIngestionStatus,
    digest: Option<PerceptualGroundingDigest>,
    contribution: Option<IntegratedLayerContribution>,
}

impl PerceptualGroundingIngestionResult {
    pub fn status(&self) -> PerceptualGroundingIngestionStatus {
        self.status
    }

    pub fn digest(&self) -> Option<&PerceptualGroundingDigest> {
        self.digest.as_ref()
    }

    pub fn contribution(&self) -> Option<&IntegratedLayerContribution> {
        self.contribution.as_ref()
    }

    pub fn ingested(&self) -> bool {
        self.status == PerceptualGroundingIngestionStatus::Ingested
    }

    pub fn abstained(&self) -> bool {
        self.contribution.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousPerceptualGroundingIngestion;

impl AutonomousPerceptualGroundingIngestion {
    fn abstain(
        status: PerceptualGroundingIngestionStatus,
        digest: Option<PerceptualGroundingDigest>,
    ) -> PerceptualGroundingIngestionResult {
        PerceptualGroundingIngestionResult {
            status,
            digest,
            contribution: None,
        }
    }

    pub fn ingest(
        request: &PerceptualGroundingIngestionRequest,
        input: &IntegratedPerceptualWorldInput,
        context: IntegratedPerceptualWorldContext,
        policy: PerceptualGroundingIngestionPolicy,
    ) -> PerceptualGroundingIngestionResult {
        let bounds = policy.bounds();

        if input.previous_frame().element_count() > bounds.max_previous_frame_elements()
            || input.current_frame().element_count() > bounds.max_current_frame_elements()
        {
            return Self::abstain(
                PerceptualGroundingIngestionStatus::FrameFrontierExceeded,
                None,
            );
        }

        let world = CoreKnowledgePerceptualWorld::evaluate(input, context);

        let digest = PerceptualGroundingDigest::from_world(input, &world);

        if digest.total_selected_outputs() > bounds.max_selected_outputs() {
            return Self::abstain(
                PerceptualGroundingIngestionStatus::SelectedOutputFrontierExceeded,
                Some(digest),
            );
        }

        if digest.rejected_dependency_count() > bounds.max_dependency_rejections() {
            return Self::abstain(
                PerceptualGroundingIngestionStatus::DependencyRejectionFrontierExceeded,
                Some(digest),
            );
        }

        let contribution = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::PerceptualGrounding,
            request.anchor_state().clone(),
            request.grounded_state().clone(),
            request.provenance().clone(),
            request.confidence(),
            request.compute_cost(),
        )
        .expect("ingestion request enforces positive confidence");

        PerceptualGroundingIngestionResult {
            status: PerceptualGroundingIngestionStatus::Ingested,
            digest: Some(digest),
            contribution: Some(contribution),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalPerceptualGroundingIngestion;

impl UniversalPerceptualGroundingIngestion {
    pub fn evaluate(
        request: &PerceptualGroundingIngestionRequest,
        input: &IntegratedPerceptualWorldInput,
        context: IntegratedPerceptualWorldContext,
        policy: PerceptualGroundingIngestionPolicy,
    ) -> PerceptualGroundingIngestionResult {
        AutonomousPerceptualGroundingIngestion::ingest(request, input, context, policy)
    }
}

#[cfg(test)]
mod perceptual_grounding_ingestion_tests {
    use super::*;

    use athlesia_core_knowledge_perceptual_grounding::{
        ActionConsequencePolicy, IntegratedPerceptualWorld, IntegratedPerceptualWorldCandidates,
        PerceptualChangePolicy, PerceptualElement, PerceptualElementHandle, PerceptualFrame,
        PerceptualGroundingPolicy, PersistenceTrackingPolicy, TopologicalRelationPolicy,
    };

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

    fn frame(observation: u64, elements: &[(u64, u64)]) -> PerceptualFrame {
        PerceptualFrame::new(
            observation,
            elements
                .iter()
                .map(|(handle, signature)| {
                    PerceptualElement::new(PerceptualElementHandle::new(*handle), a(*signature))
                })
                .collect(),
        )
        .unwrap()
    }

    fn input(previous: &[(u64, u64)], current: &[(u64, u64)]) -> IntegratedPerceptualWorldInput {
        IntegratedPerceptualWorldInput::new(
            frame(1, previous),
            frame(2, current),
            IntegratedPerceptualWorldCandidates::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        )
        .unwrap()
    }

    fn context() -> IntegratedPerceptualWorldContext {
        IntegratedPerceptualWorldContext::new(
            PerceptualGroundingPolicy::new(8, 8).unwrap(),
            PersistenceTrackingPolicy::new(8, 8, 16).unwrap(),
            TopologicalRelationPolicy::new(8, 16).unwrap(),
            PerceptualChangePolicy::new(8, 16).unwrap(),
            ActionConsequencePolicy::new(8, 8, 16).unwrap(),
        )
    }

    fn request() -> PerceptualGroundingIngestionRequest {
        PerceptualGroundingIngestionRequest::new(a(1000), a(1001), a(9000), s(900), s(200)).unwrap()
    }

    fn policy() -> PerceptualGroundingIngestionPolicy {
        PerceptualGroundingIngestionPolicy::new(
            PerceptualGroundingIngestionBounds::new(8, 8, 32, 32).unwrap(),
        )
    }

    #[test]
    fn perceptual_ingestion_contract_requires_positive_confidence_and_frame_bounds() {
        assert_eq!(
            PerceptualGroundingIngestionRequest::new(a(1), a(2), a(3), s(0), s(10),),
            None
        );

        assert_eq!(PerceptualGroundingIngestionBounds::new(0, 8, 32, 32,), None);

        assert_eq!(PerceptualGroundingIngestionBounds::new(8, 0, 32, 32,), None);
    }

    #[test]
    fn real_m46_world_evaluation_is_ingested_into_perceptual_layer() {
        let world_input = input(&[(1, 101)], &[(2, 102)]);

        let result = AutonomousPerceptualGroundingIngestion::ingest(
            &request(),
            &world_input,
            context(),
            policy(),
        );

        assert!(result.ingested());

        let digest = result.digest().unwrap();

        assert_eq!(digest.previous_observation_index(), 1);

        assert_eq!(digest.current_observation_index(), 2);

        assert_eq!(digest.previous_frame_element_count(), 1);

        assert_eq!(digest.current_frame_element_count(), 1);
    }

    #[test]
    fn perceptual_contribution_preserves_exact_agent_state_and_provenance_identity() {
        let req = PerceptualGroundingIngestionRequest::new(a(500), a(501), a(999), s(850), s(175))
            .unwrap();

        let result = AutonomousPerceptualGroundingIngestion::ingest(
            &req,
            &input(&[(1, 10)], &[(2, 20)]),
            context(),
            policy(),
        );

        let contribution = result.contribution().unwrap();

        assert_eq!(
            contribution.layer(),
            IntegratedCognitiveLayer::PerceptualGrounding
        );

        assert_eq!(contribution.anchor_state(), &a(500));

        assert_eq!(contribution.result_state(), &a(501));

        assert_eq!(contribution.provenance(), &a(999));

        assert_eq!(contribution.confidence(), s(850));

        assert_eq!(contribution.compute_cost(), s(175));
    }

    #[test]
    fn strictly_forward_m46_frame_order_remains_authoritative() {
        let previous = frame(2, &[(1, 10)]);

        let current = frame(1, &[(2, 20)]);

        let world_input = IntegratedPerceptualWorldInput::new(
            previous,
            current,
            IntegratedPerceptualWorldCandidates::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        );

        assert_eq!(world_input, None);
    }

    #[test]
    fn hard_frame_frontier_abstains_before_m46_world_evaluation() {
        let bounded = PerceptualGroundingIngestionPolicy::new(
            PerceptualGroundingIngestionBounds::new(1, 1, 32, 32).unwrap(),
        );

        let result = AutonomousPerceptualGroundingIngestion::ingest(
            &request(),
            &input(&[(1, 10), (2, 20)], &[(3, 30)]),
            context(),
            bounded,
        );

        assert_eq!(
            result.status(),
            PerceptualGroundingIngestionStatus::FrameFrontierExceeded
        );

        assert!(result.digest().is_none());

        assert!(result.contribution().is_none());
    }

    #[test]
    fn empty_m46_candidate_world_does_not_fabricate_grounded_hypotheses() {
        let result = AutonomousPerceptualGroundingIngestion::ingest(
            &request(),
            &input(&[(1, 10)], &[(2, 20)]),
            context(),
            policy(),
        );

        let digest = result.digest().unwrap();

        assert_eq!(digest.selected_previous_scene_count(), 0);

        assert_eq!(digest.selected_current_scene_count(), 0);

        assert_eq!(digest.selected_persistence_count(), 0);

        assert_eq!(digest.selected_topology_count(), 0);

        assert_eq!(digest.selected_change_count(), 0);

        assert_eq!(digest.selected_action_consequence_count(), 0);

        assert_eq!(digest.total_selected_outputs(), 0);
    }

    #[test]
    fn dependency_closed_empty_world_reports_zero_dependency_resurrection() {
        let result = AutonomousPerceptualGroundingIngestion::ingest(
            &request(),
            &input(&[(1, 10)], &[(2, 20)]),
            context(),
            policy(),
        );

        assert_eq!(result.digest().unwrap().rejected_dependency_count(), 0);

        assert!(result.ingested());
    }

    #[test]
    fn core_and_integrated_m46_facades_produce_identical_perceptual_digest() {
        let world_input = input(&[(1, 10)], &[(2, 20)]);

        let core = CoreKnowledgePerceptualWorld::evaluate(&world_input, context());

        let integrated = IntegratedPerceptualWorld::evaluate(&world_input, context());

        let core_digest = PerceptualGroundingDigest::from_world(&world_input, &core);

        let integrated_digest = PerceptualGroundingDigest::from_world(&world_input, &integrated);

        assert_eq!(core_digest, integrated_digest);
    }

    #[test]
    fn opaque_perceptual_signatures_are_not_given_domain_semantics_by_m51() {
        let first = AutonomousPerceptualGroundingIngestion::ingest(
            &request(),
            &input(&[(1, 100)], &[(2, 200)]),
            context(),
            policy(),
        );

        let second = AutonomousPerceptualGroundingIngestion::ingest(
            &request(),
            &input(&[(1, 700)], &[(2, 900)]),
            context(),
            policy(),
        );

        assert_eq!(first.digest(), second.digest());

        assert_eq!(first.contribution(), second.contribution());
    }

    #[test]
    fn m46_frame_canonicalization_makes_ingestion_element_order_invariant() {
        let first = input(&[(2, 20), (1, 10)], &[(4, 40), (3, 30)]);

        let second = input(&[(1, 10), (2, 20)], &[(3, 30), (4, 40)]);

        assert_eq!(first.previous_frame(), second.previous_frame());

        assert_eq!(first.current_frame(), second.current_frame());

        let left =
            AutonomousPerceptualGroundingIngestion::ingest(&request(), &first, context(), policy());

        let right = AutonomousPerceptualGroundingIngestion::ingest(
            &request(),
            &second,
            context(),
            policy(),
        );

        assert_eq!(left, right);
    }

    #[test]
    fn real_m46_perceptual_contribution_enters_integrated_agent_frame() {
        let perceptual = AutonomousPerceptualGroundingIngestion::ingest(
            &request(),
            &input(&[(1, 10)], &[(2, 20)]),
            context(),
            policy(),
        );

        let learned = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::UniversalDomainLearning,
            a(1000),
            a(2000),
            a(9001),
            s(800),
            s(150),
        )
        .unwrap();

        let integrated = IntegratedCognitiveAgentFoundation::integrate(
            &a(1000),
            &[perceptual.contribution().unwrap().clone(), learned],
            IntegratedAgentPolicy::new(
                IntegratedAgentBounds::new(5, 1000).unwrap(),
                IntegratedAgentThresholds::new(s(500)).unwrap(),
            ),
        );

        assert!(integrated.integrated());

        assert!(
            integrated
                .frame()
                .unwrap()
                .contribution(IntegratedCognitiveLayer::PerceptualGrounding)
                .is_some()
        );
    }

    #[test]
    fn perceptual_ingestion_is_deterministic_non_mutating_and_facade_equivalent() {
        let world_input = input(&[(2, 20), (1, 10)], &[(4, 40), (3, 30)]);

        let before_previous = world_input.previous_frame().clone();

        let before_current = world_input.current_frame().clone();

        let req = request();

        let p = policy();

        let direct =
            AutonomousPerceptualGroundingIngestion::ingest(&req, &world_input, context(), p);

        let facade =
            UniversalPerceptualGroundingIngestion::evaluate(&req, &world_input, context(), p);

        let repeated =
            UniversalPerceptualGroundingIngestion::evaluate(&req, &world_input, context(), p);

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(world_input.previous_frame(), &before_previous);

        assert_eq!(world_input.current_frame(), &before_current);
    }
}

use athlesia_universal_domain_learning::{
    CompressedDomainModel, GroundedInterventionalCausalHypothesis, IntegratedDomainModelPolicy,
    IntegratedDomainModelResult, UniversalIntegratedDomainModel,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniversalDomainLearningIngestionRequest {
    domain: CognitiveStructure,
    anchor_state: CognitiveStructure,
    learned_state: CognitiveStructure,
    provenance: CognitiveStructure,
    confidence: CognitiveSignal,
    compute_cost: CognitiveSignal,
}

impl UniversalDomainLearningIngestionRequest {
    pub fn new(
        domain: CognitiveStructure,
        anchor_state: CognitiveStructure,
        learned_state: CognitiveStructure,
        provenance: CognitiveStructure,
        confidence: CognitiveSignal,
        compute_cost: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            domain,
            anchor_state,
            learned_state,
            provenance,
            confidence,
            compute_cost,
        })
    }

    pub fn domain(&self) -> &CognitiveStructure {
        &self.domain
    }

    pub fn anchor_state(&self) -> &CognitiveStructure {
        &self.anchor_state
    }

    pub fn learned_state(&self) -> &CognitiveStructure {
        &self.learned_state
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
pub struct UniversalDomainLearningIngestionBounds {
    max_input_local_hypotheses: usize,
    max_input_transferred_models: usize,
    max_relations: usize,
    max_rejected_domain_mismatches: usize,
}

impl UniversalDomainLearningIngestionBounds {
    pub fn new(
        max_input_local_hypotheses: usize,
        max_input_transferred_models: usize,
        max_relations: usize,
        max_rejected_domain_mismatches: usize,
    ) -> Option<Self> {
        if max_input_local_hypotheses == 0
            || max_input_transferred_models == 0
            || max_relations == 0
        {
            return None;
        }

        Some(Self {
            max_input_local_hypotheses,
            max_input_transferred_models,
            max_relations,
            max_rejected_domain_mismatches,
        })
    }

    pub fn max_input_local_hypotheses(self) -> usize {
        self.max_input_local_hypotheses
    }

    pub fn max_input_transferred_models(self) -> usize {
        self.max_input_transferred_models
    }

    pub fn max_relations(self) -> usize {
        self.max_relations
    }

    pub fn max_rejected_domain_mismatches(self) -> usize {
        self.max_rejected_domain_mismatches
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UniversalDomainLearningIngestionPolicy {
    bounds: UniversalDomainLearningIngestionBounds,
}

impl UniversalDomainLearningIngestionPolicy {
    pub fn new(bounds: UniversalDomainLearningIngestionBounds) -> Self {
        Self { bounds }
    }

    pub fn bounds(self) -> UniversalDomainLearningIngestionBounds {
        self.bounds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniversalDomainLearningDigest {
    domain: CognitiveStructure,
    input_local_hypothesis_count: usize,
    considered_local_hypothesis_count: usize,
    local_frontier_truncated: bool,
    input_transferred_model_count: usize,
    matching_transferred_model_count: usize,
    considered_transferred_model_count: usize,
    transferred_frontier_truncated: bool,
    rejected_target_domain_mismatch: usize,
    admitted_before_frontier: usize,
    relation_count: usize,
}

impl UniversalDomainLearningDigest {
    pub fn from_model(model: &IntegratedDomainModelResult) -> Self {
        Self {
            domain: model.domain().clone(),
            input_local_hypothesis_count: model.input_local_hypothesis_count(),
            considered_local_hypothesis_count: model.considered_local_hypothesis_count(),
            local_frontier_truncated: model.local_frontier_truncated(),
            input_transferred_model_count: model.input_transferred_model_count(),
            matching_transferred_model_count: model.matching_transferred_model_count(),
            considered_transferred_model_count: model.considered_transferred_model_count(),
            transferred_frontier_truncated: model.transferred_frontier_truncated(),
            rejected_target_domain_mismatch: model.rejected_target_domain_mismatch(),
            admitted_before_frontier: model.admitted_before_frontier(),
            relation_count: model.relation_count(),
        }
    }

    pub fn domain(&self) -> &CognitiveStructure {
        &self.domain
    }

    pub fn input_local_hypothesis_count(&self) -> usize {
        self.input_local_hypothesis_count
    }

    pub fn considered_local_hypothesis_count(&self) -> usize {
        self.considered_local_hypothesis_count
    }

    pub fn local_frontier_truncated(&self) -> bool {
        self.local_frontier_truncated
    }

    pub fn input_transferred_model_count(&self) -> usize {
        self.input_transferred_model_count
    }

    pub fn matching_transferred_model_count(&self) -> usize {
        self.matching_transferred_model_count
    }

    pub fn considered_transferred_model_count(&self) -> usize {
        self.considered_transferred_model_count
    }

    pub fn transferred_frontier_truncated(&self) -> bool {
        self.transferred_frontier_truncated
    }

    pub fn rejected_target_domain_mismatch(&self) -> usize {
        self.rejected_target_domain_mismatch
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn relation_count(&self) -> usize {
        self.relation_count
    }

    pub fn frontier_truncated(&self) -> bool {
        self.local_frontier_truncated || self.transferred_frontier_truncated
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniversalDomainLearningIngestionStatus {
    Ingested,
    InputFrontierExceeded,
    RelationFrontierExceeded,
    DomainMismatchFrontierExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniversalDomainLearningIngestionResult {
    status: UniversalDomainLearningIngestionStatus,
    digest: Option<UniversalDomainLearningDigest>,
    contribution: Option<IntegratedLayerContribution>,
}

impl UniversalDomainLearningIngestionResult {
    pub fn status(&self) -> UniversalDomainLearningIngestionStatus {
        self.status
    }

    pub fn digest(&self) -> Option<&UniversalDomainLearningDigest> {
        self.digest.as_ref()
    }

    pub fn contribution(&self) -> Option<&IntegratedLayerContribution> {
        self.contribution.as_ref()
    }

    pub fn ingested(&self) -> bool {
        self.status == UniversalDomainLearningIngestionStatus::Ingested
    }

    pub fn abstained(&self) -> bool {
        self.contribution.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousUniversalDomainLearningIngestion;

impl AutonomousUniversalDomainLearningIngestion {
    fn abstain(
        status: UniversalDomainLearningIngestionStatus,
        digest: Option<UniversalDomainLearningDigest>,
    ) -> UniversalDomainLearningIngestionResult {
        UniversalDomainLearningIngestionResult {
            status,
            digest,
            contribution: None,
        }
    }

    pub fn ingest(
        request: &UniversalDomainLearningIngestionRequest,
        local: &[GroundedInterventionalCausalHypothesis],
        transferred: &[CompressedDomainModel],
        domain_policy: IntegratedDomainModelPolicy,
        policy: UniversalDomainLearningIngestionPolicy,
    ) -> UniversalDomainLearningIngestionResult {
        let bounds = policy.bounds();

        if local.len() > bounds.max_input_local_hypotheses()
            || transferred.len() > bounds.max_input_transferred_models()
        {
            return Self::abstain(
                UniversalDomainLearningIngestionStatus::InputFrontierExceeded,
                None,
            );
        }

        let model = UniversalIntegratedDomainModel::evaluate(
            request.domain(),
            local,
            transferred,
            domain_policy,
        );

        let digest = UniversalDomainLearningDigest::from_model(&model);

        if digest.relation_count() > bounds.max_relations() {
            return Self::abstain(
                UniversalDomainLearningIngestionStatus::RelationFrontierExceeded,
                Some(digest),
            );
        }

        if digest.rejected_target_domain_mismatch() > bounds.max_rejected_domain_mismatches() {
            return Self::abstain(
                UniversalDomainLearningIngestionStatus::DomainMismatchFrontierExceeded,
                Some(digest),
            );
        }

        let contribution = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::UniversalDomainLearning,
            request.anchor_state().clone(),
            request.learned_state().clone(),
            request.provenance().clone(),
            request.confidence(),
            request.compute_cost(),
        )
        .expect("domain-learning request enforces positive confidence");

        UniversalDomainLearningIngestionResult {
            status: UniversalDomainLearningIngestionStatus::Ingested,
            digest: Some(digest),
            contribution: Some(contribution),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalDomainLearningAgentIngestion;

impl UniversalDomainLearningAgentIngestion {
    pub fn evaluate(
        request: &UniversalDomainLearningIngestionRequest,
        local: &[GroundedInterventionalCausalHypothesis],
        transferred: &[CompressedDomainModel],
        domain_policy: IntegratedDomainModelPolicy,
        policy: UniversalDomainLearningIngestionPolicy,
    ) -> UniversalDomainLearningIngestionResult {
        AutonomousUniversalDomainLearningIngestion::ingest(
            request,
            local,
            transferred,
            domain_policy,
            policy,
        )
    }
}

#[cfg(test)]
mod universal_domain_learning_ingestion_tests {
    use super::*;

    use athlesia_universal_domain_learning::IntegratedDomainModel;

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

    fn request() -> UniversalDomainLearningIngestionRequest {
        UniversalDomainLearningIngestionRequest::new(
            a(7000),
            a(1000),
            a(1002),
            a(9100),
            s(900),
            s(250),
        )
        .unwrap()
    }

    fn domain_policy() -> IntegratedDomainModelPolicy {
        IntegratedDomainModelPolicy::new(8, 8, 16).unwrap()
    }

    fn ingestion_policy() -> UniversalDomainLearningIngestionPolicy {
        UniversalDomainLearningIngestionPolicy::new(
            UniversalDomainLearningIngestionBounds::new(8, 8, 16, 8).unwrap(),
        )
    }

    #[test]
    fn domain_learning_ingestion_contract_requires_positive_confidence_and_hard_bounds() {
        assert_eq!(
            UniversalDomainLearningIngestionRequest::new(a(1), a(2), a(3), a(4), s(0), s(10),),
            None
        );

        assert_eq!(
            UniversalDomainLearningIngestionBounds::new(0, 8, 16, 8,),
            None
        );

        assert_eq!(
            UniversalDomainLearningIngestionBounds::new(8, 0, 16, 8,),
            None
        );

        assert_eq!(
            UniversalDomainLearningIngestionBounds::new(8, 8, 0, 8,),
            None
        );

        assert_eq!(IntegratedDomainModelPolicy::new(0, 8, 16,), None);
    }

    #[test]
    fn real_m47_integrated_domain_model_is_ingested() {
        let result = AutonomousUniversalDomainLearningIngestion::ingest(
            &request(),
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        assert!(result.ingested());

        assert!(result.digest().is_some());

        assert!(result.contribution().is_some());
    }

    #[test]
    fn m47_integrated_model_preserves_exact_domain_identity() {
        let req = request();

        let result = AutonomousUniversalDomainLearningIngestion::ingest(
            &req,
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        assert_eq!(result.digest().unwrap().domain(), req.domain());

        assert_eq!(result.digest().unwrap().domain(), &a(7000));
    }

    #[test]
    fn empty_m47_model_does_not_fabricate_domain_relations() {
        let result = AutonomousUniversalDomainLearningIngestion::ingest(
            &request(),
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        let digest = result.digest().unwrap();

        assert_eq!(digest.input_local_hypothesis_count(), 0);

        assert_eq!(digest.input_transferred_model_count(), 0);

        assert_eq!(digest.relation_count(), 0);

        assert_eq!(digest.admitted_before_frontier(), 0);
    }

    #[test]
    fn empty_m47_model_reports_no_truncation_or_domain_mismatch() {
        let result = AutonomousUniversalDomainLearningIngestion::ingest(
            &request(),
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        let digest = result.digest().unwrap();

        assert_eq!(digest.considered_local_hypothesis_count(), 0);

        assert_eq!(digest.matching_transferred_model_count(), 0);

        assert_eq!(digest.considered_transferred_model_count(), 0);

        assert!(!digest.local_frontier_truncated());

        assert!(!digest.transferred_frontier_truncated());

        assert!(!digest.frontier_truncated());

        assert_eq!(digest.rejected_target_domain_mismatch(), 0);
    }

    #[test]
    fn domain_learning_contribution_preserves_exact_agent_state_and_provenance() {
        let req = UniversalDomainLearningIngestionRequest::new(
            a(777),
            a(500),
            a(501),
            a(999),
            s(850),
            s(175),
        )
        .unwrap();

        let result = AutonomousUniversalDomainLearningIngestion::ingest(
            &req,
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        let contribution = result.contribution().unwrap();

        assert_eq!(
            contribution.layer(),
            IntegratedCognitiveLayer::UniversalDomainLearning
        );

        assert_eq!(contribution.anchor_state(), &a(500));

        assert_eq!(contribution.result_state(), &a(501));

        assert_eq!(contribution.provenance(), &a(999));

        assert_eq!(contribution.confidence(), s(850));

        assert_eq!(contribution.compute_cost(), s(175));
    }

    #[test]
    fn opaque_domain_identity_changes_identity_without_inventing_semantics() {
        let first_request = UniversalDomainLearningIngestionRequest::new(
            a(111),
            a(1000),
            a(1002),
            a(9100),
            s(900),
            s(250),
        )
        .unwrap();

        let second_request = UniversalDomainLearningIngestionRequest::new(
            a(999),
            a(1000),
            a(1002),
            a(9100),
            s(900),
            s(250),
        )
        .unwrap();

        let first = AutonomousUniversalDomainLearningIngestion::ingest(
            &first_request,
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        let second = AutonomousUniversalDomainLearningIngestion::ingest(
            &second_request,
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        assert_ne!(
            first.digest().unwrap().domain(),
            second.digest().unwrap().domain()
        );

        assert_eq!(
            first.digest().unwrap().relation_count(),
            second.digest().unwrap().relation_count()
        );

        assert_eq!(first.contribution(), second.contribution());
    }

    #[test]
    fn universal_and_direct_m47_facades_produce_identical_domain_digest() {
        let domain = a(7000);

        let direct = IntegratedDomainModel::build(&domain, &[], &[], domain_policy());

        let universal =
            UniversalIntegratedDomainModel::evaluate(&domain, &[], &[], domain_policy());

        let direct_digest = UniversalDomainLearningDigest::from_model(&direct);

        let universal_digest = UniversalDomainLearningDigest::from_model(&universal);

        assert_eq!(direct_digest, universal_digest);
    }

    #[test]
    fn domain_learning_contribution_combines_with_real_perceptual_layer_in_agent_frame() {
        let learned = AutonomousUniversalDomainLearningIngestion::ingest(
            &request(),
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        let perceptual = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::PerceptualGrounding,
            a(1000),
            a(1001),
            a(9000),
            s(900),
            s(200),
        )
        .unwrap();

        let integrated = IntegratedCognitiveAgentFoundation::integrate(
            &a(1000),
            &[perceptual, learned.contribution().unwrap().clone()],
            IntegratedAgentPolicy::new(
                IntegratedAgentBounds::new(5, 1000).unwrap(),
                IntegratedAgentThresholds::new(s(500)).unwrap(),
            ),
        );

        assert!(integrated.integrated());

        let frame = integrated.frame().unwrap();

        assert!(
            frame
                .contribution(IntegratedCognitiveLayer::PerceptualGrounding)
                .is_some()
        );

        assert!(
            frame
                .contribution(IntegratedCognitiveLayer::UniversalDomainLearning)
                .is_some()
        );
    }

    #[test]
    fn mismatched_agent_anchor_rejects_domain_learning_from_integrated_frame() {
        let learned = AutonomousUniversalDomainLearningIngestion::ingest(
            &request(),
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        let integrated = IntegratedCognitiveAgentFoundation::integrate(
            &a(9999),
            &[learned.contribution().unwrap().clone()],
            IntegratedAgentPolicy::new(
                IntegratedAgentBounds::new(5, 1000).unwrap(),
                IntegratedAgentThresholds::new(s(500)).unwrap(),
            ),
        );

        assert_eq!(
            integrated.status(),
            IntegratedAgentFoundationStatus::NoQualifyingContributions
        );

        assert_eq!(integrated.rejected_anchor_count(), 1);
    }

    #[test]
    fn exact_cross_layer_provenance_conflict_abstains_atomically() {
        let learned_request = UniversalDomainLearningIngestionRequest::new(
            a(7000),
            a(1000),
            a(1002),
            a(9000),
            s(900),
            s(250),
        )
        .unwrap();

        let learned = AutonomousUniversalDomainLearningIngestion::ingest(
            &learned_request,
            &[],
            &[],
            domain_policy(),
            ingestion_policy(),
        );

        let perceptual = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::PerceptualGrounding,
            a(1000),
            a(1001),
            a(9000),
            s(900),
            s(200),
        )
        .unwrap();

        let integrated = IntegratedCognitiveAgentFoundation::integrate(
            &a(1000),
            &[perceptual, learned.contribution().unwrap().clone()],
            IntegratedAgentPolicy::new(
                IntegratedAgentBounds::new(5, 1000).unwrap(),
                IntegratedAgentThresholds::new(s(500)).unwrap(),
            ),
        );

        assert_eq!(
            integrated.status(),
            IntegratedAgentFoundationStatus::ConflictingProvenance
        );

        assert!(integrated.frame().is_none());
    }

    #[test]
    fn domain_learning_ingestion_is_deterministic_non_mutating_and_facade_equivalent() {
        let local: Vec<GroundedInterventionalCausalHypothesis> = Vec::new();

        let transferred: Vec<CompressedDomainModel> = Vec::new();

        let req = request();

        let direct = AutonomousUniversalDomainLearningIngestion::ingest(
            &req,
            &local,
            &transferred,
            domain_policy(),
            ingestion_policy(),
        );

        let facade = UniversalDomainLearningAgentIngestion::evaluate(
            &req,
            &local,
            &transferred,
            domain_policy(),
            ingestion_policy(),
        );

        let repeated = UniversalDomainLearningAgentIngestion::evaluate(
            &req,
            &local,
            &transferred,
            domain_policy(),
            ingestion_policy(),
        );

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert!(local.is_empty());

        assert!(transferred.is_empty());
    }
}

use athlesia_executive_agency::{
    IntegratedExecutiveControlDecision, IntegratedExecutiveControlResult,
    IntegratedExecutiveSelection, IntegratedExecutiveSelectionSource,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutiveAgencyIngestionRequest {
    anchor_state: CognitiveStructure,
    executive_state: CognitiveStructure,
    provenance: CognitiveStructure,
    confidence: CognitiveSignal,
    compute_cost: CognitiveSignal,
}

impl ExecutiveAgencyIngestionRequest {
    pub fn new(
        anchor_state: CognitiveStructure,
        executive_state: CognitiveStructure,
        provenance: CognitiveStructure,
        confidence: CognitiveSignal,
        compute_cost: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            anchor_state,
            executive_state,
            provenance,
            confidence,
            compute_cost,
        })
    }

    pub fn anchor_state(&self) -> &CognitiveStructure {
        &self.anchor_state
    }

    pub fn executive_state(&self) -> &CognitiveStructure {
        &self.executive_state
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutiveAgencyDigest {
    decision: IntegratedExecutiveControlDecision,
    selection_source: Option<IntegratedExecutiveSelectionSource>,
    goal_identity: Option<CognitiveStructure>,
    action: Option<CognitiveStructure>,
    predicted_outcome: Option<CognitiveStructure>,
    intention_step_index: Option<usize>,
    control_value: Option<CognitiveSignal>,
}

impl ExecutiveAgencyDigest {
    pub fn from_parts(
        decision: IntegratedExecutiveControlDecision,
        selection: Option<&IntegratedExecutiveSelection>,
    ) -> Self {
        Self {
            decision,
            selection_source: selection.map(IntegratedExecutiveSelection::source),
            goal_identity: selection.map(|value| value.goal_identity().clone()),
            action: selection.map(|value| value.action().clone()),
            predicted_outcome: selection.map(|value| value.predicted_outcome().clone()),
            intention_step_index: selection
                .and_then(IntegratedExecutiveSelection::intention_step_index),
            control_value: selection.map(IntegratedExecutiveSelection::control_value),
        }
    }

    pub fn from_result(result: &IntegratedExecutiveControlResult) -> Self {
        Self::from_parts(result.decision(), result.selection())
    }

    pub fn decision(&self) -> IntegratedExecutiveControlDecision {
        self.decision
    }

    pub fn selection_source(&self) -> Option<IntegratedExecutiveSelectionSource> {
        self.selection_source
    }

    pub fn goal_identity(&self) -> Option<&CognitiveStructure> {
        self.goal_identity.as_ref()
    }

    pub fn action(&self) -> Option<&CognitiveStructure> {
        self.action.as_ref()
    }

    pub fn predicted_outcome(&self) -> Option<&CognitiveStructure> {
        self.predicted_outcome.as_ref()
    }

    pub fn intention_step_index(&self) -> Option<usize> {
        self.intention_step_index
    }

    pub fn control_value(&self) -> Option<CognitiveSignal> {
        self.control_value
    }

    pub fn has_selection(&self) -> bool {
        self.selection_source.is_some()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutiveAgencyIngestionStatus {
    Ingested,
    MissingExecutionSelection,
    UnexpectedNonExecutionSelection,
    SelectionSourceMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutiveAgencyIngestionResult {
    status: ExecutiveAgencyIngestionStatus,
    digest: Option<ExecutiveAgencyDigest>,
    contribution: Option<IntegratedLayerContribution>,
}

impl ExecutiveAgencyIngestionResult {
    pub fn status(&self) -> ExecutiveAgencyIngestionStatus {
        self.status
    }

    pub fn digest(&self) -> Option<&ExecutiveAgencyDigest> {
        self.digest.as_ref()
    }

    pub fn contribution(&self) -> Option<&IntegratedLayerContribution> {
        self.contribution.as_ref()
    }

    pub fn ingested(&self) -> bool {
        self.status == ExecutiveAgencyIngestionStatus::Ingested
    }

    pub fn abstained(&self) -> bool {
        self.contribution.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousExecutiveAgencyIngestion;

impl AutonomousExecutiveAgencyIngestion {
    pub fn decision_requires_selection(decision: IntegratedExecutiveControlDecision) -> bool {
        matches!(
            decision,
            IntegratedExecutiveControlDecision::ExecuteCurrent
                | IntegratedExecutiveControlDecision::ExecuteReplacement
                | IntegratedExecutiveControlDecision::ExecuteExploration
        )
    }

    pub fn expected_selection_source(
        decision: IntegratedExecutiveControlDecision,
    ) -> Option<IntegratedExecutiveSelectionSource> {
        match decision {
            IntegratedExecutiveControlDecision::ExecuteCurrent => {
                Some(IntegratedExecutiveSelectionSource::CurrentIntention)
            }

            IntegratedExecutiveControlDecision::ExecuteReplacement => {
                Some(IntegratedExecutiveSelectionSource::ReplacementIntention)
            }

            IntegratedExecutiveControlDecision::ExecuteExploration => {
                Some(IntegratedExecutiveSelectionSource::Exploration)
            }

            IntegratedExecutiveControlDecision::Stop
            | IntegratedExecutiveControlDecision::Reconsider
            | IntegratedExecutiveControlDecision::NoViableOption => None,
        }
    }

    fn abstain(
        status: ExecutiveAgencyIngestionStatus,
        digest: ExecutiveAgencyDigest,
    ) -> ExecutiveAgencyIngestionResult {
        ExecutiveAgencyIngestionResult {
            status,
            digest: Some(digest),
            contribution: None,
        }
    }

    pub fn ingest_parts(
        request: &ExecutiveAgencyIngestionRequest,
        decision: IntegratedExecutiveControlDecision,
        selection: Option<&IntegratedExecutiveSelection>,
    ) -> ExecutiveAgencyIngestionResult {
        let digest = ExecutiveAgencyDigest::from_parts(decision, selection);

        let expected_source = Self::expected_selection_source(decision);

        match (expected_source, digest.selection_source()) {
            (Some(_), None) => {
                return Self::abstain(
                    ExecutiveAgencyIngestionStatus::MissingExecutionSelection,
                    digest,
                );
            }

            (None, Some(_)) => {
                return Self::abstain(
                    ExecutiveAgencyIngestionStatus::UnexpectedNonExecutionSelection,
                    digest,
                );
            }

            (Some(expected), Some(actual)) if expected != actual => {
                return Self::abstain(
                    ExecutiveAgencyIngestionStatus::SelectionSourceMismatch,
                    digest,
                );
            }

            _ => {}
        }

        let contribution = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::ExecutiveAgency,
            request.anchor_state().clone(),
            request.executive_state().clone(),
            request.provenance().clone(),
            request.confidence(),
            request.compute_cost(),
        )
        .expect("executive ingestion request enforces positive confidence");

        ExecutiveAgencyIngestionResult {
            status: ExecutiveAgencyIngestionStatus::Ingested,
            digest: Some(digest),
            contribution: Some(contribution),
        }
    }

    pub fn ingest(
        request: &ExecutiveAgencyIngestionRequest,
        result: &IntegratedExecutiveControlResult,
    ) -> ExecutiveAgencyIngestionResult {
        Self::ingest_parts(request, result.decision(), result.selection())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalExecutiveAgencyIngestion;

impl UniversalExecutiveAgencyIngestion {
    pub fn evaluate(
        request: &ExecutiveAgencyIngestionRequest,
        result: &IntegratedExecutiveControlResult,
    ) -> ExecutiveAgencyIngestionResult {
        AutonomousExecutiveAgencyIngestion::ingest(request, result)
    }

    pub fn evaluate_parts(
        request: &ExecutiveAgencyIngestionRequest,
        decision: IntegratedExecutiveControlDecision,
        selection: Option<&IntegratedExecutiveSelection>,
    ) -> ExecutiveAgencyIngestionResult {
        AutonomousExecutiveAgencyIngestion::ingest_parts(request, decision, selection)
    }
}

#[cfg(test)]
mod executive_agency_ingestion_tests {
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

    fn request() -> ExecutiveAgencyIngestionRequest {
        ExecutiveAgencyIngestionRequest::new(a(1000), a(1003), a(9200), s(900), s(225)).unwrap()
    }

    #[test]
    fn executive_ingestion_request_requires_positive_confidence() {
        assert_eq!(
            ExecutiveAgencyIngestionRequest::new(a(1), a(2), a(3), s(0), s(10),),
            None
        );

        let valid = request();

        assert_eq!(valid.anchor_state(), &a(1000));

        assert_eq!(valid.executive_state(), &a(1003));

        assert_eq!(valid.provenance(), &a(9200));
    }

    #[test]
    fn real_m48_integrated_control_result_adapter_is_compile_time_bound() {
        let adapter: fn(&IntegratedExecutiveControlResult) -> ExecutiveAgencyDigest =
            ExecutiveAgencyDigest::from_result;

        let ingest: fn(
            &ExecutiveAgencyIngestionRequest,
            &IntegratedExecutiveControlResult,
        ) -> ExecutiveAgencyIngestionResult = AutonomousExecutiveAgencyIngestion::ingest;

        let facade: fn(
            &ExecutiveAgencyIngestionRequest,
            &IntegratedExecutiveControlResult,
        ) -> ExecutiveAgencyIngestionResult = UniversalExecutiveAgencyIngestion::evaluate;

        let _ = (adapter, ingest, facade);
    }

    #[test]
    fn m48_execution_decisions_require_exact_selection_presence() {
        assert!(
            AutonomousExecutiveAgencyIngestion::decision_requires_selection(
                IntegratedExecutiveControlDecision::ExecuteCurrent
            )
        );

        assert!(
            AutonomousExecutiveAgencyIngestion::decision_requires_selection(
                IntegratedExecutiveControlDecision::ExecuteReplacement
            )
        );

        assert!(
            AutonomousExecutiveAgencyIngestion::decision_requires_selection(
                IntegratedExecutiveControlDecision::ExecuteExploration
            )
        );

        assert!(
            !AutonomousExecutiveAgencyIngestion::decision_requires_selection(
                IntegratedExecutiveControlDecision::Stop
            )
        );

        assert!(
            !AutonomousExecutiveAgencyIngestion::decision_requires_selection(
                IntegratedExecutiveControlDecision::Reconsider
            )
        );

        assert!(
            !AutonomousExecutiveAgencyIngestion::decision_requires_selection(
                IntegratedExecutiveControlDecision::NoViableOption
            )
        );
    }

    #[test]
    fn m48_execute_decisions_bind_to_exact_frozen_selection_sources() {
        assert_eq!(
            AutonomousExecutiveAgencyIngestion::expected_selection_source(
                IntegratedExecutiveControlDecision::ExecuteCurrent
            ),
            Some(IntegratedExecutiveSelectionSource::CurrentIntention)
        );

        assert_eq!(
            AutonomousExecutiveAgencyIngestion::expected_selection_source(
                IntegratedExecutiveControlDecision::ExecuteReplacement
            ),
            Some(IntegratedExecutiveSelectionSource::ReplacementIntention)
        );

        assert_eq!(
            AutonomousExecutiveAgencyIngestion::expected_selection_source(
                IntegratedExecutiveControlDecision::ExecuteExploration
            ),
            Some(IntegratedExecutiveSelectionSource::Exploration)
        );
    }

    #[test]
    fn stop_decision_ingests_without_fabricating_action_selection() {
        let result = AutonomousExecutiveAgencyIngestion::ingest_parts(
            &request(),
            IntegratedExecutiveControlDecision::Stop,
            None,
        );

        assert!(result.ingested());

        let digest = result.digest().unwrap();

        assert_eq!(digest.decision(), IntegratedExecutiveControlDecision::Stop);

        assert!(!digest.has_selection());

        assert_eq!(digest.action(), None);

        assert_eq!(digest.predicted_outcome(), None);
    }

    #[test]
    fn reconsider_decision_preserves_exact_agent_state_and_provenance() {
        let req =
            ExecutiveAgencyIngestionRequest::new(a(500), a(503), a(999), s(850), s(175)).unwrap();

        let result = AutonomousExecutiveAgencyIngestion::ingest_parts(
            &req,
            IntegratedExecutiveControlDecision::Reconsider,
            None,
        );

        let contribution = result.contribution().unwrap();

        assert_eq!(
            contribution.layer(),
            IntegratedCognitiveLayer::ExecutiveAgency
        );

        assert_eq!(contribution.anchor_state(), &a(500));

        assert_eq!(contribution.result_state(), &a(503));

        assert_eq!(contribution.provenance(), &a(999));

        assert_eq!(contribution.confidence(), s(850));

        assert_eq!(contribution.compute_cost(), s(175));
    }

    #[test]
    fn no_viable_option_is_preserved_as_explicit_executive_state() {
        let result = AutonomousExecutiveAgencyIngestion::ingest_parts(
            &request(),
            IntegratedExecutiveControlDecision::NoViableOption,
            None,
        );

        assert!(result.ingested());

        assert_eq!(
            result.digest().unwrap().decision(),
            IntegratedExecutiveControlDecision::NoViableOption
        );

        assert!(result.digest().unwrap().selection_source().is_none());
    }

    #[test]
    fn execution_decisions_without_frozen_m48_selection_abstain_atomically() {
        for decision in [
            IntegratedExecutiveControlDecision::ExecuteCurrent,
            IntegratedExecutiveControlDecision::ExecuteReplacement,
            IntegratedExecutiveControlDecision::ExecuteExploration,
        ] {
            let result =
                AutonomousExecutiveAgencyIngestion::ingest_parts(&request(), decision, None);

            assert_eq!(
                result.status(),
                ExecutiveAgencyIngestionStatus::MissingExecutionSelection
            );

            assert!(result.abstained());

            assert!(result.contribution().is_none());

            assert_eq!(result.digest().unwrap().decision(), decision);
        }
    }

    #[test]
    fn executive_ingestion_facade_is_deterministic_for_reconsideration() {
        let req = request();

        let direct = AutonomousExecutiveAgencyIngestion::ingest_parts(
            &req,
            IntegratedExecutiveControlDecision::Reconsider,
            None,
        );

        let facade = UniversalExecutiveAgencyIngestion::evaluate_parts(
            &req,
            IntegratedExecutiveControlDecision::Reconsider,
            None,
        );

        let repeated = UniversalExecutiveAgencyIngestion::evaluate_parts(
            &req,
            IntegratedExecutiveControlDecision::Reconsider,
            None,
        );

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);
    }

    #[test]
    fn executive_contribution_coexists_with_perception_and_domain_learning() {
        let executive = AutonomousExecutiveAgencyIngestion::ingest_parts(
            &request(),
            IntegratedExecutiveControlDecision::Stop,
            None,
        );

        let perceptual = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::PerceptualGrounding,
            a(1000),
            a(1001),
            a(9000),
            s(900),
            s(200),
        )
        .unwrap();

        let domain = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::UniversalDomainLearning,
            a(1000),
            a(1002),
            a(9100),
            s(900),
            s(250),
        )
        .unwrap();

        let integrated = IntegratedCognitiveAgentFoundation::integrate(
            &a(1000),
            &[
                perceptual,
                domain,
                executive.contribution().unwrap().clone(),
            ],
            IntegratedAgentPolicy::new(
                IntegratedAgentBounds::new(5, 2000).unwrap(),
                IntegratedAgentThresholds::new(s(500)).unwrap(),
            ),
        );

        assert!(integrated.integrated());

        let frame = integrated.frame().unwrap();

        assert!(
            frame
                .contribution(IntegratedCognitiveLayer::PerceptualGrounding)
                .is_some()
        );

        assert!(
            frame
                .contribution(IntegratedCognitiveLayer::UniversalDomainLearning)
                .is_some()
        );

        assert!(
            frame
                .contribution(IntegratedCognitiveLayer::ExecutiveAgency)
                .is_some()
        );
    }

    #[test]
    fn executive_cross_layer_provenance_collision_remains_atomic() {
        let executive_request =
            ExecutiveAgencyIngestionRequest::new(a(1000), a(1003), a(9000), s(900), s(225))
                .unwrap();

        let executive = AutonomousExecutiveAgencyIngestion::ingest_parts(
            &executive_request,
            IntegratedExecutiveControlDecision::Reconsider,
            None,
        );

        let perceptual = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::PerceptualGrounding,
            a(1000),
            a(1001),
            a(9000),
            s(900),
            s(200),
        )
        .unwrap();

        let integrated = IntegratedCognitiveAgentFoundation::integrate(
            &a(1000),
            &[perceptual, executive.contribution().unwrap().clone()],
            IntegratedAgentPolicy::new(
                IntegratedAgentBounds::new(5, 2000).unwrap(),
                IntegratedAgentThresholds::new(s(500)).unwrap(),
            ),
        );

        assert_eq!(
            integrated.status(),
            IntegratedAgentFoundationStatus::ConflictingProvenance
        );

        assert!(integrated.frame().is_none());
    }

    #[test]
    fn opaque_executive_state_identity_is_never_given_domain_specific_meaning() {
        let first_request =
            ExecutiveAgencyIngestionRequest::new(a(1000), a(111), a(9200), s(900), s(225)).unwrap();

        let second_request =
            ExecutiveAgencyIngestionRequest::new(a(1000), a(999), a(9200), s(900), s(225)).unwrap();

        let first = AutonomousExecutiveAgencyIngestion::ingest_parts(
            &first_request,
            IntegratedExecutiveControlDecision::Stop,
            None,
        );

        let second = AutonomousExecutiveAgencyIngestion::ingest_parts(
            &second_request,
            IntegratedExecutiveControlDecision::Stop,
            None,
        );

        assert_eq!(first.digest(), second.digest());

        assert_ne!(
            first.contribution().unwrap().result_state(),
            second.contribution().unwrap().result_state()
        );

        assert_eq!(
            first.contribution().unwrap().layer(),
            IntegratedCognitiveLayer::ExecutiveAgency
        );

        assert_eq!(
            second.contribution().unwrap().layer(),
            IntegratedCognitiveLayer::ExecutiveAgency
        );
    }
}

use athlesia_meta_learning_skill_memory::{
    IntegratedSkillLearningCycleResult, SkillMemoryAvailability,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetaLearningSkillMemoryIngestionRequest {
    anchor_state: CognitiveStructure,
    memory_state: CognitiveStructure,
    provenance: CognitiveStructure,
    confidence: CognitiveSignal,
    compute_cost: CognitiveSignal,
}

impl MetaLearningSkillMemoryIngestionRequest {
    pub fn new(
        anchor_state: CognitiveStructure,
        memory_state: CognitiveStructure,
        provenance: CognitiveStructure,
        confidence: CognitiveSignal,
        compute_cost: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            anchor_state,
            memory_state,
            provenance,
            confidence,
            compute_cost,
        })
    }

    pub fn anchor_state(&self) -> &CognitiveStructure {
        &self.anchor_state
    }

    pub fn memory_state(&self) -> &CognitiveStructure {
        &self.memory_state
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
pub struct SkillLearningChainDigest {
    reused_skill: bool,
    selected_plan_present: bool,
    feedback_present: bool,
    revision_present: bool,
    revision_applied: bool,
    revised_memory: bool,
}

impl SkillLearningChainDigest {
    pub fn new(
        reused_skill: bool,
        selected_plan_present: bool,
        feedback_present: bool,
        revision_present: bool,
        revision_applied: bool,
        revised_memory: bool,
    ) -> Self {
        Self {
            reused_skill,
            selected_plan_present,
            feedback_present,
            revision_present,
            revision_applied,
            revised_memory,
        }
    }

    pub fn reused_skill(self) -> bool {
        self.reused_skill
    }

    pub fn selected_plan_present(self) -> bool {
        self.selected_plan_present
    }

    pub fn feedback_present(self) -> bool {
        self.feedback_present
    }

    pub fn revision_present(self) -> bool {
        self.revision_present
    }

    pub fn revision_applied(self) -> bool {
        self.revision_applied
    }

    pub fn revised_memory(self) -> bool {
        self.revised_memory
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillMemoryStateDigest {
    availability: SkillMemoryAvailability,
    reusable: bool,
    retained_count: usize,
    forgotten_count: usize,
}

impl SkillMemoryStateDigest {
    pub fn new(
        availability: SkillMemoryAvailability,
        reusable: bool,
        retained_count: usize,
        forgotten_count: usize,
    ) -> Self {
        Self {
            availability,
            reusable,
            retained_count,
            forgotten_count,
        }
    }

    pub fn availability(self) -> SkillMemoryAvailability {
        self.availability
    }

    pub fn reusable(self) -> bool {
        self.reusable
    }

    pub fn retained_count(self) -> usize {
        self.retained_count
    }

    pub fn forgotten_count(self) -> usize {
        self.forgotten_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillMemoryFrontierDigest {
    input_frontier_truncated: bool,
    evaluation_frontier_truncated: bool,
    tier_frontier_truncated: bool,
}

impl SkillMemoryFrontierDigest {
    pub fn new(
        input_frontier_truncated: bool,
        evaluation_frontier_truncated: bool,
        tier_frontier_truncated: bool,
    ) -> Self {
        Self {
            input_frontier_truncated,
            evaluation_frontier_truncated,
            tier_frontier_truncated,
        }
    }

    pub fn input_frontier_truncated(self) -> bool {
        self.input_frontier_truncated
    }

    pub fn evaluation_frontier_truncated(self) -> bool {
        self.evaluation_frontier_truncated
    }

    pub fn tier_frontier_truncated(self) -> bool {
        self.tier_frontier_truncated
    }

    pub fn any_truncated(self) -> bool {
        self.input_frontier_truncated
            || self.evaluation_frontier_truncated
            || self.tier_frontier_truncated
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetaLearningSkillMemoryDigest {
    chain: SkillLearningChainDigest,
    memory: SkillMemoryStateDigest,
    frontier: SkillMemoryFrontierDigest,
}

impl MetaLearningSkillMemoryDigest {
    pub fn new(
        chain: SkillLearningChainDigest,
        memory: SkillMemoryStateDigest,
        frontier: SkillMemoryFrontierDigest,
    ) -> Self {
        Self {
            chain,
            memory,
            frontier,
        }
    }

    pub fn from_result(result: &IntegratedSkillLearningCycleResult) -> Self {
        let revision = result.revision();

        let consolidation = result.consolidation();

        Self {
            chain: SkillLearningChainDigest::new(
                result.reused_skill(),
                result.selected_plan().is_some(),
                result.feedback().is_some(),
                revision.is_some(),
                revision
                    .map(|value| value.revision_applied())
                    .unwrap_or(false),
                result.revised_memory(),
            ),
            memory: SkillMemoryStateDigest::new(
                result.updated_memory().availability(),
                result.updated_memory().reusable(),
                consolidation.retained_count(),
                consolidation.forgotten_count(),
            ),
            frontier: SkillMemoryFrontierDigest::new(
                consolidation.input_frontier_truncated(),
                consolidation.evaluation_frontier_truncated(),
                consolidation.tier_frontier_truncated(),
            ),
        }
    }

    pub fn chain(self) -> SkillLearningChainDigest {
        self.chain
    }

    pub fn memory(self) -> SkillMemoryStateDigest {
        self.memory
    }

    pub fn frontier(self) -> SkillMemoryFrontierDigest {
        self.frontier
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetaLearningSkillMemoryIngestionStatus {
    Ingested,
    ReuseSelectionMismatch,
    LearningChainMismatch,
    RevisionStateMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetaLearningSkillMemoryIngestionResult {
    status: MetaLearningSkillMemoryIngestionStatus,
    digest: MetaLearningSkillMemoryDigest,
    contribution: Option<IntegratedLayerContribution>,
}

impl MetaLearningSkillMemoryIngestionResult {
    pub fn status(&self) -> MetaLearningSkillMemoryIngestionStatus {
        self.status
    }

    pub fn digest(&self) -> MetaLearningSkillMemoryDigest {
        self.digest
    }

    pub fn contribution(&self) -> Option<&IntegratedLayerContribution> {
        self.contribution.as_ref()
    }

    pub fn ingested(&self) -> bool {
        self.status == MetaLearningSkillMemoryIngestionStatus::Ingested
    }

    pub fn abstained(&self) -> bool {
        self.contribution.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousMetaLearningSkillMemoryIngestion;

impl AutonomousMetaLearningSkillMemoryIngestion {
    fn abstain(
        status: MetaLearningSkillMemoryIngestionStatus,
        digest: MetaLearningSkillMemoryDigest,
    ) -> MetaLearningSkillMemoryIngestionResult {
        MetaLearningSkillMemoryIngestionResult {
            status,
            digest,
            contribution: None,
        }
    }

    pub fn ingest_digest(
        request: &MetaLearningSkillMemoryIngestionRequest,
        digest: MetaLearningSkillMemoryDigest,
    ) -> MetaLearningSkillMemoryIngestionResult {
        let chain = digest.chain();

        if chain.reused_skill() != chain.selected_plan_present() {
            return Self::abstain(
                MetaLearningSkillMemoryIngestionStatus::ReuseSelectionMismatch,
                digest,
            );
        }

        if chain.selected_plan_present() != chain.feedback_present()
            || chain.selected_plan_present() != chain.revision_present()
        {
            return Self::abstain(
                MetaLearningSkillMemoryIngestionStatus::LearningChainMismatch,
                digest,
            );
        }

        if (chain.revision_applied() || chain.revised_memory()) && !chain.revision_present() {
            return Self::abstain(
                MetaLearningSkillMemoryIngestionStatus::RevisionStateMismatch,
                digest,
            );
        }

        let contribution = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::MetaLearningSkillMemory,
            request.anchor_state().clone(),
            request.memory_state().clone(),
            request.provenance().clone(),
            request.confidence(),
            request.compute_cost(),
        )
        .expect("skill-memory ingestion request enforces positive confidence");

        MetaLearningSkillMemoryIngestionResult {
            status: MetaLearningSkillMemoryIngestionStatus::Ingested,
            digest,
            contribution: Some(contribution),
        }
    }

    pub fn ingest(
        request: &MetaLearningSkillMemoryIngestionRequest,
        result: &IntegratedSkillLearningCycleResult,
    ) -> MetaLearningSkillMemoryIngestionResult {
        Self::ingest_digest(request, MetaLearningSkillMemoryDigest::from_result(result))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalMetaLearningSkillMemoryIngestion;

impl UniversalMetaLearningSkillMemoryIngestion {
    pub fn evaluate(
        request: &MetaLearningSkillMemoryIngestionRequest,
        result: &IntegratedSkillLearningCycleResult,
    ) -> MetaLearningSkillMemoryIngestionResult {
        AutonomousMetaLearningSkillMemoryIngestion::ingest(request, result)
    }

    pub fn evaluate_digest(
        request: &MetaLearningSkillMemoryIngestionRequest,
        digest: MetaLearningSkillMemoryDigest,
    ) -> MetaLearningSkillMemoryIngestionResult {
        AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(request, digest)
    }
}

#[cfg(test)]
mod meta_learning_skill_memory_ingestion_tests {
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

    fn request() -> MetaLearningSkillMemoryIngestionRequest {
        MetaLearningSkillMemoryIngestionRequest::new(a(1000), a(1004), a(9300), s(900), s(200))
            .unwrap()
    }

    fn digest(
        reused: bool,
        selected: bool,
        feedback: bool,
        revision: bool,
        revision_applied: bool,
        revised_memory: bool,
        availability: SkillMemoryAvailability,
    ) -> MetaLearningSkillMemoryDigest {
        MetaLearningSkillMemoryDigest::new(
            SkillLearningChainDigest::new(
                reused,
                selected,
                feedback,
                revision,
                revision_applied,
                revised_memory,
            ),
            SkillMemoryStateDigest::new(
                availability,
                availability == SkillMemoryAvailability::Active,
                1,
                0,
            ),
            SkillMemoryFrontierDigest::new(false, false, false),
        )
    }

    #[test]
    fn skill_memory_ingestion_request_requires_positive_confidence() {
        assert_eq!(
            MetaLearningSkillMemoryIngestionRequest::new(a(1), a(2), a(3), s(0), s(10),),
            None
        );

        let req = request();

        assert_eq!(req.anchor_state(), &a(1000));

        assert_eq!(req.memory_state(), &a(1004));

        assert_eq!(req.provenance(), &a(9300));
    }

    #[test]
    fn real_m49_integrated_cycle_result_adapter_is_compile_time_bound() {
        let adapter: fn(&IntegratedSkillLearningCycleResult) -> MetaLearningSkillMemoryDigest =
            MetaLearningSkillMemoryDigest::from_result;

        let ingestion: fn(
            &MetaLearningSkillMemoryIngestionRequest,
            &IntegratedSkillLearningCycleResult,
        ) -> MetaLearningSkillMemoryIngestionResult =
            AutonomousMetaLearningSkillMemoryIngestion::ingest;

        let facade: fn(
            &MetaLearningSkillMemoryIngestionRequest,
            &IntegratedSkillLearningCycleResult,
        ) -> MetaLearningSkillMemoryIngestionResult =
            UniversalMetaLearningSkillMemoryIngestion::evaluate;

        let _ = (adapter, ingestion, facade);
    }

    #[test]
    fn unresolved_retrieval_ingests_without_fabricating_skill_reuse() {
        let result = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &request(),
            digest(
                false,
                false,
                false,
                false,
                false,
                false,
                SkillMemoryAvailability::Active,
            ),
        );

        assert!(result.ingested());

        let chain = result.digest().chain();

        assert!(!chain.reused_skill());

        assert!(!chain.selected_plan_present());

        assert!(!chain.feedback_present());

        assert!(!chain.revision_present());
    }

    #[test]
    fn reuse_requires_exact_selected_plan_presence() {
        let result = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &request(),
            digest(
                true,
                false,
                false,
                false,
                false,
                false,
                SkillMemoryAvailability::Active,
            ),
        );

        assert_eq!(
            result.status(),
            MetaLearningSkillMemoryIngestionStatus::ReuseSelectionMismatch
        );

        assert!(result.abstained());
    }

    #[test]
    fn selected_reuse_requires_feedback_and_revision_chain() {
        let result = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &request(),
            digest(
                true,
                true,
                true,
                false,
                false,
                false,
                SkillMemoryAvailability::Active,
            ),
        );

        assert_eq!(
            result.status(),
            MetaLearningSkillMemoryIngestionStatus::LearningChainMismatch
        );

        assert!(result.contribution().is_none());
    }

    #[test]
    fn revision_state_cannot_exist_without_revision_result() {
        let result = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &request(),
            digest(
                false,
                false,
                false,
                false,
                true,
                true,
                SkillMemoryAvailability::Active,
            ),
        );

        assert_eq!(
            result.status(),
            MetaLearningSkillMemoryIngestionStatus::RevisionStateMismatch
        );

        assert!(result.abstained());
    }

    #[test]
    fn successful_skill_learning_chain_preserves_memory_state() {
        let result = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &request(),
            digest(
                true,
                true,
                true,
                true,
                true,
                true,
                SkillMemoryAvailability::Active,
            ),
        );

        assert!(result.ingested());

        let memory = result.digest().memory();

        assert_eq!(memory.availability(), SkillMemoryAvailability::Active);

        assert!(memory.reusable());

        assert_eq!(memory.retained_count(), 1);

        assert_eq!(memory.forgotten_count(), 0);
    }

    #[test]
    fn suspended_skill_memory_remains_explicit_and_non_reusable() {
        let result = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &request(),
            digest(
                true,
                true,
                true,
                true,
                true,
                true,
                SkillMemoryAvailability::Suspended,
            ),
        );

        assert!(result.ingested());

        let memory = result.digest().memory();

        assert_eq!(memory.availability(), SkillMemoryAvailability::Suspended);

        assert!(!memory.reusable());
    }

    #[test]
    fn skill_memory_contribution_preserves_exact_agent_state_and_provenance() {
        let req =
            MetaLearningSkillMemoryIngestionRequest::new(a(500), a(504), a(999), s(850), s(175))
                .unwrap();

        let result = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &req,
            digest(
                false,
                false,
                false,
                false,
                false,
                false,
                SkillMemoryAvailability::Active,
            ),
        );

        let contribution = result.contribution().unwrap();

        assert_eq!(
            contribution.layer(),
            IntegratedCognitiveLayer::MetaLearningSkillMemory
        );

        assert_eq!(contribution.anchor_state(), &a(500));

        assert_eq!(contribution.result_state(), &a(504));

        assert_eq!(contribution.provenance(), &a(999));
    }

    #[test]
    fn perception_domain_executive_and_skill_memory_share_one_agent_frame() {
        let skill = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &request(),
            digest(
                false,
                false,
                false,
                false,
                false,
                false,
                SkillMemoryAvailability::Active,
            ),
        );

        let contributions = vec![
            IntegratedLayerContribution::new(
                IntegratedCognitiveLayer::PerceptualGrounding,
                a(1000),
                a(1001),
                a(9000),
                s(900),
                s(200),
            )
            .unwrap(),
            IntegratedLayerContribution::new(
                IntegratedCognitiveLayer::UniversalDomainLearning,
                a(1000),
                a(1002),
                a(9100),
                s(900),
                s(250),
            )
            .unwrap(),
            IntegratedLayerContribution::new(
                IntegratedCognitiveLayer::ExecutiveAgency,
                a(1000),
                a(1003),
                a(9200),
                s(900),
                s(225),
            )
            .unwrap(),
            skill.contribution().unwrap().clone(),
        ];

        let integrated = IntegratedCognitiveAgentFoundation::integrate(
            &a(1000),
            &contributions,
            IntegratedAgentPolicy::new(
                IntegratedAgentBounds::new(5, 2000).unwrap(),
                IntegratedAgentThresholds::new(s(500)).unwrap(),
            ),
        );

        assert!(integrated.integrated());

        let frame = integrated.frame().unwrap();

        assert!(
            frame
                .contribution(IntegratedCognitiveLayer::MetaLearningSkillMemory)
                .is_some()
        );
    }

    #[test]
    fn skill_memory_cross_layer_provenance_collision_remains_atomic() {
        let req =
            MetaLearningSkillMemoryIngestionRequest::new(a(1000), a(1004), a(9000), s(900), s(200))
                .unwrap();

        let skill = AutonomousMetaLearningSkillMemoryIngestion::ingest_digest(
            &req,
            digest(
                false,
                false,
                false,
                false,
                false,
                false,
                SkillMemoryAvailability::Active,
            ),
        );

        let perceptual = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::PerceptualGrounding,
            a(1000),
            a(1001),
            a(9000),
            s(900),
            s(200),
        )
        .unwrap();

        let integrated = IntegratedCognitiveAgentFoundation::integrate(
            &a(1000),
            &[perceptual, skill.contribution().unwrap().clone()],
            IntegratedAgentPolicy::new(
                IntegratedAgentBounds::new(5, 2000).unwrap(),
                IntegratedAgentThresholds::new(s(500)).unwrap(),
            ),
        );

        assert_eq!(
            integrated.status(),
            IntegratedAgentFoundationStatus::ConflictingProvenance
        );

        assert!(integrated.frame().is_none());
    }

    #[test]
    fn skill_memory_ingestion_is_deterministic_and_opaque_state_preserving() {
        let first_request =
            MetaLearningSkillMemoryIngestionRequest::new(a(1000), a(111), a(9300), s(900), s(200))
                .unwrap();

        let second_request =
            MetaLearningSkillMemoryIngestionRequest::new(a(1000), a(999), a(9300), s(900), s(200))
                .unwrap();

        let d = digest(
            false,
            false,
            false,
            false,
            false,
            false,
            SkillMemoryAvailability::Active,
        );

        let first = UniversalMetaLearningSkillMemoryIngestion::evaluate_digest(&first_request, d);

        let repeated =
            UniversalMetaLearningSkillMemoryIngestion::evaluate_digest(&first_request, d);

        let second = UniversalMetaLearningSkillMemoryIngestion::evaluate_digest(&second_request, d);

        assert_eq!(first, repeated);

        assert_eq!(first.digest(), second.digest());

        assert_ne!(
            first.contribution().unwrap().result_state(),
            second.contribution().unwrap().result_state()
        );

        assert!(!first.digest().frontier().any_truncated());
    }
}

use athlesia_autonomous_active_experimentation::{
    ExperimentContinuationBasis, IntegratedAutonomousExperimentationResult,
    IntegratedAutonomousExperimentationStatus, StopContinueExperimentationDecision,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomousExperimentationIngestionRequest {
    anchor_state: CognitiveStructure,
    experimentation_state: CognitiveStructure,
    provenance: CognitiveStructure,
    confidence: CognitiveSignal,
    compute_cost: CognitiveSignal,
}

impl AutonomousExperimentationIngestionRequest {
    pub fn new(
        anchor_state: CognitiveStructure,
        experimentation_state: CognitiveStructure,
        provenance: CognitiveStructure,
        confidence: CognitiveSignal,
        compute_cost: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            anchor_state,
            experimentation_state,
            provenance,
            confidence,
            compute_cost,
        })
    }

    pub fn anchor_state(&self) -> &CognitiveStructure {
        &self.anchor_state
    }

    pub fn experimentation_state(&self) -> &CognitiveStructure {
        &self.experimentation_state
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
pub struct AutonomousExperimentationLifecycleDigest {
    status: IntegratedAutonomousExperimentationStatus,
    continuing: bool,
    stopped: bool,
    abstained: bool,
}

impl AutonomousExperimentationLifecycleDigest {
    pub fn new(
        status: IntegratedAutonomousExperimentationStatus,
        continuing: bool,
        stopped: bool,
        abstained: bool,
    ) -> Self {
        Self {
            status,
            continuing,
            stopped,
            abstained,
        }
    }

    pub fn status(self) -> IntegratedAutonomousExperimentationStatus {
        self.status
    }

    pub fn continuing(self) -> bool {
        self.continuing
    }

    pub fn stopped(self) -> bool {
        self.stopped
    }

    pub fn abstained(self) -> bool {
        self.abstained
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutonomousExperimentationPipelineDigest {
    proposal_present: bool,
    learning_progress_present: bool,
    sequence_planning_present: bool,
    control_present: bool,
    next_plan_present: bool,
    next_experiment_present: bool,
}

impl AutonomousExperimentationPipelineDigest {
    pub fn new(
        proposal_present: bool,
        learning_progress_present: bool,
        sequence_planning_present: bool,
        control_present: bool,
        next_plan_present: bool,
        next_experiment_present: bool,
    ) -> Self {
        Self {
            proposal_present,
            learning_progress_present,
            sequence_planning_present,
            control_present,
            next_plan_present,
            next_experiment_present,
        }
    }

    pub fn proposal_present(self) -> bool {
        self.proposal_present
    }

    pub fn learning_progress_present(self) -> bool {
        self.learning_progress_present
    }

    pub fn sequence_planning_present(self) -> bool {
        self.sequence_planning_present
    }

    pub fn control_present(self) -> bool {
        self.control_present
    }

    pub fn next_plan_present(self) -> bool {
        self.next_plan_present
    }

    pub fn next_experiment_present(self) -> bool {
        self.next_experiment_present
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutonomousExperimentationControlDigest {
    decision: Option<StopContinueExperimentationDecision>,
    continuation_basis: Option<ExperimentContinuationBasis>,
    current_experiment_cycle: Option<usize>,
}

impl AutonomousExperimentationControlDigest {
    pub fn new(
        decision: Option<StopContinueExperimentationDecision>,
        continuation_basis: Option<ExperimentContinuationBasis>,
        current_experiment_cycle: Option<usize>,
    ) -> Self {
        Self {
            decision,
            continuation_basis,
            current_experiment_cycle,
        }
    }

    pub fn decision(self) -> Option<StopContinueExperimentationDecision> {
        self.decision
    }

    pub fn continuation_basis(self) -> Option<ExperimentContinuationBasis> {
        self.continuation_basis
    }

    pub fn current_experiment_cycle(self) -> Option<usize> {
        self.current_experiment_cycle
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutonomousExperimentationDigest {
    lifecycle: AutonomousExperimentationLifecycleDigest,
    pipeline: AutonomousExperimentationPipelineDigest,
    control: AutonomousExperimentationControlDigest,
}

impl AutonomousExperimentationDigest {
    pub fn new(
        lifecycle: AutonomousExperimentationLifecycleDigest,
        pipeline: AutonomousExperimentationPipelineDigest,
        control: AutonomousExperimentationControlDigest,
    ) -> Self {
        Self {
            lifecycle,
            pipeline,
            control,
        }
    }

    pub fn from_result(result: &IntegratedAutonomousExperimentationResult) -> Self {
        let control = result.control();

        Self {
            lifecycle: AutonomousExperimentationLifecycleDigest::new(
                result.status(),
                result.continuing(),
                result.stopped(),
                result.abstained(),
            ),
            pipeline: AutonomousExperimentationPipelineDigest::new(
                true,
                result.learning_progress().is_some(),
                result.sequence_planning().is_some(),
                control.is_some(),
                result.next_plan().is_some(),
                result.next_experiment().is_some(),
            ),
            control: AutonomousExperimentationControlDigest::new(
                control.map(|value| value.decision()),
                control.and_then(|value| value.continuation_basis()),
                control.map(|value| value.current_experiment_cycle()),
            ),
        }
    }

    pub fn lifecycle(self) -> AutonomousExperimentationLifecycleDigest {
        self.lifecycle
    }

    pub fn pipeline(self) -> AutonomousExperimentationPipelineDigest {
        self.pipeline
    }

    pub fn control(self) -> AutonomousExperimentationControlDigest {
        self.control
    }

    pub fn status(self) -> IntegratedAutonomousExperimentationStatus {
        self.lifecycle.status()
    }

    pub fn continuing(self) -> bool {
        self.lifecycle.continuing()
    }

    pub fn stopped(self) -> bool {
        self.lifecycle.stopped()
    }

    pub fn abstained(self) -> bool {
        self.lifecycle.abstained()
    }

    pub fn control_present(self) -> bool {
        self.pipeline.control_present()
    }

    pub fn next_plan_present(self) -> bool {
        self.pipeline.next_plan_present()
    }

    pub fn next_experiment_present(self) -> bool {
        self.pipeline.next_experiment_present()
    }

    pub fn control_decision(self) -> Option<StopContinueExperimentationDecision> {
        self.control.decision()
    }

    pub fn continuation_basis(self) -> Option<ExperimentContinuationBasis> {
        self.control.continuation_basis()
    }

    pub fn current_experiment_cycle(self) -> Option<usize> {
        self.control.current_experiment_cycle()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutonomousExperimentationIngestionStatus {
    Ingested,
    LifecycleStateMismatch,
    ContinuationWithoutControl,
    ContinuationWithoutPlan,
    ContinuationWithoutExperiment,
    ControlPresenceMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomousExperimentationIngestionResult {
    status: AutonomousExperimentationIngestionStatus,
    digest: AutonomousExperimentationDigest,
    contribution: Option<IntegratedLayerContribution>,
}

impl AutonomousExperimentationIngestionResult {
    pub fn status(&self) -> AutonomousExperimentationIngestionStatus {
        self.status
    }

    pub fn digest(&self) -> AutonomousExperimentationDigest {
        self.digest
    }

    pub fn contribution(&self) -> Option<&IntegratedLayerContribution> {
        self.contribution.as_ref()
    }

    pub fn ingested(&self) -> bool {
        self.status == AutonomousExperimentationIngestionStatus::Ingested
    }

    pub fn abstained(&self) -> bool {
        self.contribution.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousExperimentationIngestion;

impl AutonomousExperimentationIngestion {
    fn abstain(
        status: AutonomousExperimentationIngestionStatus,
        digest: AutonomousExperimentationDigest,
    ) -> AutonomousExperimentationIngestionResult {
        AutonomousExperimentationIngestionResult {
            status,
            digest,
            contribution: None,
        }
    }

    pub fn ingest_digest(
        request: &AutonomousExperimentationIngestionRequest,
        digest: AutonomousExperimentationDigest,
    ) -> AutonomousExperimentationIngestionResult {
        let lifecycle_count = [digest.continuing(), digest.stopped(), digest.abstained()]
            .into_iter()
            .filter(|state| *state)
            .count();

        if lifecycle_count != 1 {
            return Self::abstain(
                AutonomousExperimentationIngestionStatus::LifecycleStateMismatch,
                digest,
            );
        }

        if digest.continuing() && !digest.control_present() {
            return Self::abstain(
                AutonomousExperimentationIngestionStatus::ContinuationWithoutControl,
                digest,
            );
        }

        if digest.continuing() && !digest.next_plan_present() {
            return Self::abstain(
                AutonomousExperimentationIngestionStatus::ContinuationWithoutPlan,
                digest,
            );
        }

        if digest.continuing() && !digest.next_experiment_present() {
            return Self::abstain(
                AutonomousExperimentationIngestionStatus::ContinuationWithoutExperiment,
                digest,
            );
        }

        if digest.control_present() != digest.control_decision().is_some() {
            return Self::abstain(
                AutonomousExperimentationIngestionStatus::ControlPresenceMismatch,
                digest,
            );
        }

        let contribution = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::AutonomousExperimentation,
            request.anchor_state().clone(),
            request.experimentation_state().clone(),
            request.provenance().clone(),
            request.confidence(),
            request.compute_cost(),
        )
        .expect("autonomous experimentation request enforces positive confidence");

        AutonomousExperimentationIngestionResult {
            status: AutonomousExperimentationIngestionStatus::Ingested,
            digest,
            contribution: Some(contribution),
        }
    }

    pub fn ingest(
        request: &AutonomousExperimentationIngestionRequest,
        result: &IntegratedAutonomousExperimentationResult,
    ) -> AutonomousExperimentationIngestionResult {
        Self::ingest_digest(
            request,
            AutonomousExperimentationDigest::from_result(result),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousExperimentationIngestion;

impl UniversalAutonomousExperimentationIngestion {
    pub fn evaluate(
        request: &AutonomousExperimentationIngestionRequest,
        result: &IntegratedAutonomousExperimentationResult,
    ) -> AutonomousExperimentationIngestionResult {
        AutonomousExperimentationIngestion::ingest(request, result)
    }

    pub fn evaluate_digest(
        request: &AutonomousExperimentationIngestionRequest,
        digest: AutonomousExperimentationDigest,
    ) -> AutonomousExperimentationIngestionResult {
        AutonomousExperimentationIngestion::ingest_digest(request, digest)
    }
}

#[cfg(test)]
mod autonomous_experimentation_ingestion_tests {
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

    fn request() -> AutonomousExperimentationIngestionRequest {
        AutonomousExperimentationIngestionRequest::new(a(1000), a(1005), a(9400), s(900), s(225))
            .unwrap()
    }

    fn digest(
        lifecycle: AutonomousExperimentationLifecycleDigest,
        pipeline: AutonomousExperimentationPipelineDigest,
        control: AutonomousExperimentationControlDigest,
    ) -> AutonomousExperimentationDigest {
        AutonomousExperimentationDigest::new(lifecycle, pipeline, control)
    }

    fn continuing_digest() -> AutonomousExperimentationDigest {
        digest(
            AutonomousExperimentationLifecycleDigest::new(
                IntegratedAutonomousExperimentationStatus::ContinueExperimentation,
                true,
                false,
                false,
            ),
            AutonomousExperimentationPipelineDigest::new(true, true, true, true, true, true),
            AutonomousExperimentationControlDigest::new(
                Some(StopContinueExperimentationDecision::ContinueExperimentation),
                Some(ExperimentContinuationBasis::ExpectedInformationGain),
                Some(1),
            ),
        )
    }

    fn stopped_digest(
        status: IntegratedAutonomousExperimentationStatus,
        decision: StopContinueExperimentationDecision,
    ) -> AutonomousExperimentationDigest {
        digest(
            AutonomousExperimentationLifecycleDigest::new(status, false, true, false),
            AutonomousExperimentationPipelineDigest::new(true, true, true, true, false, false),
            AutonomousExperimentationControlDigest::new(Some(decision), None, Some(2)),
        )
    }

    fn abstained_digest(
        status: IntegratedAutonomousExperimentationStatus,
    ) -> AutonomousExperimentationDigest {
        digest(
            AutonomousExperimentationLifecycleDigest::new(status, false, false, true),
            AutonomousExperimentationPipelineDigest::new(true, false, false, false, false, false),
            AutonomousExperimentationControlDigest::new(None, None, None),
        )
    }

    #[test]
    fn autonomous_experimentation_request_and_real_m50_adapter_contract_are_valid() {
        assert_eq!(
            AutonomousExperimentationIngestionRequest::new(a(1), a(2), a(3), s(0), s(10),),
            None
        );

        let req = request();

        assert_eq!(req.anchor_state(), &a(1000));

        assert_eq!(req.experimentation_state(), &a(1005));

        let adapter: fn(
            &IntegratedAutonomousExperimentationResult,
        ) -> AutonomousExperimentationDigest = AutonomousExperimentationDigest::from_result;

        let ingestion: fn(
            &AutonomousExperimentationIngestionRequest,
            &IntegratedAutonomousExperimentationResult,
        ) -> AutonomousExperimentationIngestionResult = AutonomousExperimentationIngestion::ingest;

        let facade: fn(
            &AutonomousExperimentationIngestionRequest,
            &IntegratedAutonomousExperimentationResult,
        ) -> AutonomousExperimentationIngestionResult =
            UniversalAutonomousExperimentationIngestion::evaluate;

        let _ = (adapter, ingestion, facade);
    }

    #[test]
    fn continuing_experimentation_preserves_m50_control_authority() {
        let result =
            AutonomousExperimentationIngestion::ingest_digest(&request(), continuing_digest());

        assert!(result.ingested());

        let d = result.digest();

        assert_eq!(
            d.status(),
            IntegratedAutonomousExperimentationStatus::ContinueExperimentation
        );

        assert!(d.continuing());

        assert_eq!(
            d.control_decision(),
            Some(StopContinueExperimentationDecision::ContinueExperimentation)
        );

        assert_eq!(
            d.continuation_basis(),
            Some(ExperimentContinuationBasis::ExpectedInformationGain)
        );
    }

    #[test]
    fn continuing_experimentation_requires_control_result() {
        let d = digest(
            AutonomousExperimentationLifecycleDigest::new(
                IntegratedAutonomousExperimentationStatus::ContinueExperimentation,
                true,
                false,
                false,
            ),
            AutonomousExperimentationPipelineDigest::new(true, true, true, false, true, true),
            AutonomousExperimentationControlDigest::new(None, None, None),
        );

        let result = AutonomousExperimentationIngestion::ingest_digest(&request(), d);

        assert_eq!(
            result.status(),
            AutonomousExperimentationIngestionStatus::ContinuationWithoutControl
        );

        assert!(result.abstained());
    }

    #[test]
    fn continuing_experimentation_requires_selected_plan() {
        let d = digest(
            AutonomousExperimentationLifecycleDigest::new(
                IntegratedAutonomousExperimentationStatus::ContinueExperimentation,
                true,
                false,
                false,
            ),
            AutonomousExperimentationPipelineDigest::new(true, true, true, true, false, true),
            AutonomousExperimentationControlDigest::new(
                Some(StopContinueExperimentationDecision::ContinueExperimentation),
                Some(ExperimentContinuationBasis::ExpectedInformationGain),
                Some(1),
            ),
        );

        let result = AutonomousExperimentationIngestion::ingest_digest(&request(), d);

        assert_eq!(
            result.status(),
            AutonomousExperimentationIngestionStatus::ContinuationWithoutPlan
        );
    }

    #[test]
    fn continuing_experimentation_requires_next_grounded_experiment() {
        let d = digest(
            AutonomousExperimentationLifecycleDigest::new(
                IntegratedAutonomousExperimentationStatus::ContinueExperimentation,
                true,
                false,
                false,
            ),
            AutonomousExperimentationPipelineDigest::new(true, true, true, true, true, false),
            AutonomousExperimentationControlDigest::new(
                Some(StopContinueExperimentationDecision::ContinueExperimentation),
                Some(ExperimentContinuationBasis::ExpectedInformationGain),
                Some(1),
            ),
        );

        let result = AutonomousExperimentationIngestion::ingest_digest(&request(), d);

        assert_eq!(
            result.status(),
            AutonomousExperimentationIngestionStatus::ContinuationWithoutExperiment
        );
    }

    #[test]
    fn resolved_belief_space_stops_without_fabricating_next_experiment() {
        let result = AutonomousExperimentationIngestion::ingest_digest(
            &request(),
            stopped_digest(
                IntegratedAutonomousExperimentationStatus::StopResolved,
                StopContinueExperimentationDecision::StopResolved,
            ),
        );

        assert!(result.ingested());

        let d = result.digest();

        assert!(d.stopped());

        assert!(!d.next_plan_present());

        assert!(!d.next_experiment_present());
    }

    #[test]
    fn experiment_budget_exhaustion_remains_explicit_stop_state() {
        let result = AutonomousExperimentationIngestion::ingest_digest(
            &request(),
            stopped_digest(
                IntegratedAutonomousExperimentationStatus::StopExperimentBudgetExhausted,
                StopContinueExperimentationDecision::StopExperimentBudgetExhausted,
            ),
        );

        assert!(result.ingested());

        assert_eq!(
            result.digest().status(),
            IntegratedAutonomousExperimentationStatus::StopExperimentBudgetExhausted
        );

        assert_eq!(result.digest().current_experiment_cycle(), Some(2));
    }

    #[test]
    fn upstream_m50_abstention_does_not_manufacture_experiment_control() {
        let result = AutonomousExperimentationIngestion::ingest_digest(
            &request(),
            abstained_digest(IntegratedAutonomousExperimentationStatus::AbstainSequencePlanning),
        );

        assert!(result.ingested());

        let d = result.digest();

        assert!(d.abstained());

        assert!(!d.control_present());

        assert!(!d.next_experiment_present());
    }

    #[test]
    fn invalid_lifecycle_and_control_presence_abstain_atomically() {
        let invalid_lifecycle = digest(
            AutonomousExperimentationLifecycleDigest::new(
                IntegratedAutonomousExperimentationStatus::ContinueExperimentation,
                true,
                true,
                false,
            ),
            AutonomousExperimentationPipelineDigest::new(true, true, true, true, true, true),
            AutonomousExperimentationControlDigest::new(
                Some(StopContinueExperimentationDecision::ContinueExperimentation),
                None,
                Some(1),
            ),
        );

        let lifecycle_result =
            AutonomousExperimentationIngestion::ingest_digest(&request(), invalid_lifecycle);

        assert_eq!(
            lifecycle_result.status(),
            AutonomousExperimentationIngestionStatus::LifecycleStateMismatch
        );

        let invalid_control = digest(
            AutonomousExperimentationLifecycleDigest::new(
                IntegratedAutonomousExperimentationStatus::StopResolved,
                false,
                true,
                false,
            ),
            AutonomousExperimentationPipelineDigest::new(true, true, true, false, false, false),
            AutonomousExperimentationControlDigest::new(
                Some(StopContinueExperimentationDecision::StopResolved),
                None,
                Some(2),
            ),
        );

        let control_result =
            AutonomousExperimentationIngestion::ingest_digest(&request(), invalid_control);

        assert_eq!(
            control_result.status(),
            AutonomousExperimentationIngestionStatus::ControlPresenceMismatch
        );
    }

    #[test]
    fn experimentation_contribution_preserves_exact_agent_state_and_provenance() {
        let req =
            AutonomousExperimentationIngestionRequest::new(a(500), a(505), a(999), s(850), s(175))
                .unwrap();

        let result = AutonomousExperimentationIngestion::ingest_digest(&req, continuing_digest());

        let contribution = result.contribution().unwrap();

        assert_eq!(
            contribution.layer(),
            IntegratedCognitiveLayer::AutonomousExperimentation
        );

        assert_eq!(contribution.anchor_state(), &a(500));

        assert_eq!(contribution.result_state(), &a(505));

        assert_eq!(contribution.provenance(), &a(999));

        assert_eq!(contribution.confidence(), s(850));

        assert_eq!(contribution.compute_cost(), s(175));
    }

    #[test]
    fn all_five_layers_integrate_and_cross_layer_provenance_remains_atomic() {
        let experimentation =
            AutonomousExperimentationIngestion::ingest_digest(&request(), continuing_digest());

        let contributions = vec![
            IntegratedLayerContribution::new(
                IntegratedCognitiveLayer::PerceptualGrounding,
                a(1000),
                a(1001),
                a(9000),
                s(900),
                s(200),
            )
            .unwrap(),
            IntegratedLayerContribution::new(
                IntegratedCognitiveLayer::UniversalDomainLearning,
                a(1000),
                a(1002),
                a(9100),
                s(900),
                s(250),
            )
            .unwrap(),
            IntegratedLayerContribution::new(
                IntegratedCognitiveLayer::ExecutiveAgency,
                a(1000),
                a(1003),
                a(9200),
                s(900),
                s(225),
            )
            .unwrap(),
            IntegratedLayerContribution::new(
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                a(1000),
                a(1004),
                a(9300),
                s(900),
                s(200),
            )
            .unwrap(),
            experimentation.contribution().unwrap().clone(),
        ];

        let policy = IntegratedAgentPolicy::new(
            IntegratedAgentBounds::new(5, 2000).unwrap(),
            IntegratedAgentThresholds::new(s(500)).unwrap(),
        );

        let integrated =
            IntegratedCognitiveAgentFoundation::integrate(&a(1000), &contributions, policy);

        assert!(integrated.integrated());

        let frame = integrated.frame().unwrap();

        for layer in [
            IntegratedCognitiveLayer::PerceptualGrounding,
            IntegratedCognitiveLayer::UniversalDomainLearning,
            IntegratedCognitiveLayer::ExecutiveAgency,
            IntegratedCognitiveLayer::MetaLearningSkillMemory,
            IntegratedCognitiveLayer::AutonomousExperimentation,
        ] {
            assert!(frame.contribution(layer).is_some());
        }

        let conflicting_request = AutonomousExperimentationIngestionRequest::new(
            a(1000),
            a(1005),
            a(9000),
            s(900),
            s(225),
        )
        .unwrap();

        let conflicting_experimentation = AutonomousExperimentationIngestion::ingest_digest(
            &conflicting_request,
            continuing_digest(),
        );

        let perceptual = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::PerceptualGrounding,
            a(1000),
            a(1001),
            a(9000),
            s(900),
            s(200),
        )
        .unwrap();

        let conflict = IntegratedCognitiveAgentFoundation::integrate(
            &a(1000),
            &[
                perceptual,
                conflicting_experimentation.contribution().unwrap().clone(),
            ],
            policy,
        );

        assert_eq!(
            conflict.status(),
            IntegratedAgentFoundationStatus::ConflictingProvenance
        );

        assert!(conflict.frame().is_none());
    }

    #[test]
    fn experimentation_ingestion_is_deterministic_and_opaque_state_preserving() {
        let first_request = AutonomousExperimentationIngestionRequest::new(
            a(1000),
            a(111),
            a(9400),
            s(900),
            s(225),
        )
        .unwrap();

        let second_request = AutonomousExperimentationIngestionRequest::new(
            a(1000),
            a(999),
            a(9400),
            s(900),
            s(225),
        )
        .unwrap();

        let d = continuing_digest();

        let first = UniversalAutonomousExperimentationIngestion::evaluate_digest(&first_request, d);

        let repeated =
            UniversalAutonomousExperimentationIngestion::evaluate_digest(&first_request, d);

        let second =
            UniversalAutonomousExperimentationIngestion::evaluate_digest(&second_request, d);

        assert_eq!(first, repeated);

        assert_eq!(first.digest(), second.digest());

        assert_ne!(
            first.contribution().unwrap().result_state(),
            second.contribution().unwrap().result_state()
        );

        assert_eq!(
            first.digest().pipeline().proposal_present(),
            second.digest().pipeline().proposal_present()
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CognitiveCyclePhase {
    layer: IntegratedCognitiveLayer,
    result_state: CognitiveStructure,
    provenance: CognitiveStructure,
    confidence: CognitiveSignal,
    compute_cost: CognitiveSignal,
}

impl CognitiveCyclePhase {
    fn from_contribution(contribution: &IntegratedLayerContribution) -> Self {
        Self {
            layer: contribution.layer(),
            result_state: contribution.result_state().clone(),
            provenance: contribution.provenance().clone(),
            confidence: contribution.confidence(),
            compute_cost: contribution.compute_cost(),
        }
    }

    pub fn layer(&self) -> IntegratedCognitiveLayer {
        self.layer
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
pub enum IntegratedCognitiveCycleStatus {
    Integrated,
    FoundationRejected,
    MissingRequiredLayer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedCognitiveCycleResult {
    status: IntegratedCognitiveCycleStatus,
    foundation_status: IntegratedAgentFoundationStatus,
    anchor_state: CognitiveStructure,
    phases: Vec<CognitiveCyclePhase>,
}

impl IntegratedCognitiveCycleResult {
    pub fn status(&self) -> IntegratedCognitiveCycleStatus {
        self.status
    }

    pub fn foundation_status(&self) -> &IntegratedAgentFoundationStatus {
        &self.foundation_status
    }

    pub fn anchor_state(&self) -> &CognitiveStructure {
        &self.anchor_state
    }

    pub fn phases(&self) -> &[CognitiveCyclePhase] {
        &self.phases
    }

    pub fn phase_count(&self) -> usize {
        self.phases.len()
    }

    pub fn phase(&self, layer: IntegratedCognitiveLayer) -> Option<&CognitiveCyclePhase> {
        self.phases.iter().find(|phase| phase.layer() == layer)
    }

    pub fn integrated(&self) -> bool {
        self.status == IntegratedCognitiveCycleStatus::Integrated
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IntegratedCognitiveCycle;

impl IntegratedCognitiveCycle {
    pub const PHASE_COUNT: usize = 5;

    pub fn required_layers() -> [IntegratedCognitiveLayer; Self::PHASE_COUNT] {
        [
            IntegratedCognitiveLayer::PerceptualGrounding,
            IntegratedCognitiveLayer::UniversalDomainLearning,
            IntegratedCognitiveLayer::ExecutiveAgency,
            IntegratedCognitiveLayer::MetaLearningSkillMemory,
            IntegratedCognitiveLayer::AutonomousExperimentation,
        ]
    }

    pub fn run(
        anchor_state: &CognitiveStructure,
        contributions: &[IntegratedLayerContribution],
        policy: IntegratedAgentPolicy,
    ) -> IntegratedCognitiveCycleResult {
        let foundation =
            IntegratedCognitiveAgentFoundation::integrate(anchor_state, contributions, policy);

        let foundation_status = foundation.status();

        let Some(frame) = foundation.frame() else {
            return IntegratedCognitiveCycleResult {
                status: IntegratedCognitiveCycleStatus::FoundationRejected,
                foundation_status,
                anchor_state: anchor_state.clone(),
                phases: Vec::new(),
            };
        };

        let mut phases = Vec::with_capacity(Self::PHASE_COUNT);

        for layer in Self::required_layers() {
            let Some(contribution) = frame.contribution(layer) else {
                return IntegratedCognitiveCycleResult {
                    status: IntegratedCognitiveCycleStatus::MissingRequiredLayer,
                    foundation_status,
                    anchor_state: anchor_state.clone(),
                    phases: Vec::new(),
                };
            };

            phases.push(CognitiveCyclePhase::from_contribution(contribution));
        }

        IntegratedCognitiveCycleResult {
            status: IntegratedCognitiveCycleStatus::Integrated,
            foundation_status,
            anchor_state: anchor_state.clone(),
            phases,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalIntegratedCognitiveCycle;

impl UniversalIntegratedCognitiveCycle {
    pub fn evaluate(
        anchor_state: &CognitiveStructure,
        contributions: &[IntegratedLayerContribution],
        policy: IntegratedAgentPolicy,
    ) -> IntegratedCognitiveCycleResult {
        IntegratedCognitiveCycle::run(anchor_state, contributions, policy)
    }
}

#[cfg(test)]
mod integrated_cognitive_cycle_tests {
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

    fn policy() -> IntegratedAgentPolicy {
        IntegratedAgentPolicy::new(
            IntegratedAgentBounds::new(5, 2000).unwrap(),
            IntegratedAgentThresholds::new(s(500)).unwrap(),
        )
    }

    fn contribution(
        layer: IntegratedCognitiveLayer,
        result_state: u64,
        provenance: u64,
        confidence: u16,
        compute_cost: u16,
    ) -> IntegratedLayerContribution {
        IntegratedLayerContribution::new(
            layer,
            a(1000),
            a(result_state),
            a(provenance),
            s(confidence),
            s(compute_cost),
        )
        .unwrap()
    }

    fn five() -> Vec<IntegratedLayerContribution> {
        vec![
            contribution(
                IntegratedCognitiveLayer::PerceptualGrounding,
                1001,
                9001,
                900,
                200,
            ),
            contribution(
                IntegratedCognitiveLayer::UniversalDomainLearning,
                1002,
                9002,
                900,
                250,
            ),
            contribution(
                IntegratedCognitiveLayer::ExecutiveAgency,
                1003,
                9003,
                900,
                225,
            ),
            contribution(
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                1004,
                9004,
                900,
                200,
            ),
            contribution(
                IntegratedCognitiveLayer::AutonomousExperimentation,
                1005,
                9005,
                900,
                225,
            ),
        ]
    }

    #[test]
    fn cognitive_cycle_requires_all_five_frozen_layers() {
        let mut contributions = five();

        contributions.pop();

        let result = IntegratedCognitiveCycle::run(&a(1000), &contributions, policy());

        assert_eq!(
            result.status(),
            IntegratedCognitiveCycleStatus::MissingRequiredLayer
        );

        assert!(result.phases().is_empty());
    }

    #[test]
    fn cognitive_cycle_uses_fixed_canonical_phase_order() {
        let result = IntegratedCognitiveCycle::run(&a(1000), &five(), policy());

        assert!(result.integrated());

        let layers: Vec<IntegratedCognitiveLayer> =
            result.phases().iter().map(|phase| phase.layer()).collect();

        assert_eq!(layers, IntegratedCognitiveCycle::required_layers().to_vec());
    }

    #[test]
    fn cognitive_cycle_preserves_exact_layer_result_states() {
        let result = IntegratedCognitiveCycle::run(&a(1000), &five(), policy());

        for (layer, expected) in [
            (IntegratedCognitiveLayer::PerceptualGrounding, 1001),
            (IntegratedCognitiveLayer::UniversalDomainLearning, 1002),
            (IntegratedCognitiveLayer::ExecutiveAgency, 1003),
            (IntegratedCognitiveLayer::MetaLearningSkillMemory, 1004),
            (IntegratedCognitiveLayer::AutonomousExperimentation, 1005),
        ] {
            assert_eq!(result.phase(layer).unwrap().result_state(), &a(expected));
        }
    }

    #[test]
    fn cognitive_cycle_preserves_exact_provenance_confidence_and_compute_signals() {
        let result = IntegratedCognitiveCycle::run(&a(1000), &five(), policy());

        let perceptual = result
            .phase(IntegratedCognitiveLayer::PerceptualGrounding)
            .unwrap();

        let domain = result
            .phase(IntegratedCognitiveLayer::UniversalDomainLearning)
            .unwrap();

        let executive = result
            .phase(IntegratedCognitiveLayer::ExecutiveAgency)
            .unwrap();

        let memory = result
            .phase(IntegratedCognitiveLayer::MetaLearningSkillMemory)
            .unwrap();

        let experimentation = result
            .phase(IntegratedCognitiveLayer::AutonomousExperimentation)
            .unwrap();

        assert_eq!(executive.provenance(), &a(9003));

        assert_eq!(executive.confidence(), s(900));

        assert_eq!(perceptual.compute_cost(), s(200));

        assert_eq!(domain.compute_cost(), s(250));

        assert_eq!(executive.compute_cost(), s(225));

        assert_eq!(memory.compute_cost(), s(200));

        assert_eq!(experimentation.compute_cost(), s(225));
    }

    #[test]
    fn mismatched_anchor_is_filtered_and_cycle_abstains_on_missing_layer() {
        let mut contributions = five();

        contributions[0] = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::PerceptualGrounding,
            a(9999),
            a(1001),
            a(9001),
            s(900),
            s(200),
        )
        .unwrap();

        let result = IntegratedCognitiveCycle::run(&a(1000), &contributions, policy());

        assert_eq!(
            result.status(),
            IntegratedCognitiveCycleStatus::MissingRequiredLayer
        );

        assert_eq!(result.phase_count(), 0);
    }

    #[test]
    fn duplicate_layer_is_rejected_before_cycle_materialization() {
        let mut contributions = five();

        contributions[4] = contribution(
            IntegratedCognitiveLayer::PerceptualGrounding,
            7777,
            9777,
            900,
            100,
        );

        let result = IntegratedCognitiveCycle::run(&a(1000), &contributions, policy());

        assert_eq!(
            result.status(),
            IntegratedCognitiveCycleStatus::FoundationRejected
        );

        assert_eq!(
            result.foundation_status(),
            &IntegratedAgentFoundationStatus::DuplicateLayerContribution
        );

        assert_eq!(result.phase_count(), 0);
    }

    #[test]
    fn cross_layer_provenance_collision_remains_atomic() {
        let mut contributions = five();

        contributions[4] = contribution(
            IntegratedCognitiveLayer::AutonomousExperimentation,
            1005,
            9001,
            900,
            225,
        );

        let result = IntegratedCognitiveCycle::run(&a(1000), &contributions, policy());

        assert_eq!(
            result.status(),
            IntegratedCognitiveCycleStatus::FoundationRejected
        );

        assert_eq!(
            result.foundation_status(),
            &IntegratedAgentFoundationStatus::ConflictingProvenance
        );

        assert!(result.phases().is_empty());
    }

    #[test]
    fn compute_budget_failure_prevents_partial_cycle() {
        let contributions = five()
            .into_iter()
            .map(|item| {
                IntegratedLayerContribution::new(
                    item.layer(),
                    item.anchor_state().clone(),
                    item.result_state().clone(),
                    item.provenance().clone(),
                    item.confidence(),
                    s(500),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        let result = IntegratedCognitiveCycle::run(&a(1000), &contributions, policy());

        assert_eq!(
            result.status(),
            IntegratedCognitiveCycleStatus::FoundationRejected
        );

        assert_eq!(
            result.foundation_status(),
            &IntegratedAgentFoundationStatus::ComputeBudgetExceeded
        );

        assert_eq!(result.phase_count(), 0);
    }

    #[test]
    fn low_confidence_layer_cannot_manufacture_complete_cycle() {
        let mut contributions = five();

        contributions[3] = contribution(
            IntegratedCognitiveLayer::MetaLearningSkillMemory,
            1004,
            9004,
            400,
            200,
        );

        let result = IntegratedCognitiveCycle::run(&a(1000), &contributions, policy());

        assert_eq!(
            result.status(),
            IntegratedCognitiveCycleStatus::MissingRequiredLayer
        );

        assert_eq!(result.phase_count(), 0);
    }

    #[test]
    fn equal_opaque_result_states_remain_distinct_by_cognitive_layer() {
        let contributions = IntegratedCognitiveCycle::required_layers()
            .into_iter()
            .enumerate()
            .map(|(index, layer)| contribution(layer, 4444, 9100 + index as u64, 900, 100))
            .collect::<Vec<_>>();

        let result = IntegratedCognitiveCycle::run(&a(1000), &contributions, policy());

        assert!(result.integrated());

        assert_eq!(result.phase_count(), 5);

        for phase in result.phases() {
            assert_eq!(phase.result_state(), &a(4444));
        }
    }

    #[test]
    fn input_order_cannot_change_canonical_cycle() {
        let original = five();

        let mut reversed = original.clone();

        reversed.reverse();

        let first = IntegratedCognitiveCycle::run(&a(1000), &original, policy());

        let second = IntegratedCognitiveCycle::run(&a(1000), &reversed, policy());

        assert_eq!(first, second);
    }

    #[test]
    fn cognitive_cycle_is_deterministic_non_mutating_and_facade_equivalent() {
        let contributions = five();

        let before = contributions.clone();

        let direct = IntegratedCognitiveCycle::run(&a(1000), &contributions, policy());

        let facade =
            UniversalIntegratedCognitiveCycle::evaluate(&a(1000), &contributions, policy());

        let repeated =
            UniversalIntegratedCognitiveCycle::evaluate(&a(1000), &contributions, policy());

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(contributions, before);

        assert_eq!(facade.anchor_state(), &a(1000));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CognitiveCycleTransitionAuthority {
    PreserveAnchor,
    AdoptLayer(IntegratedCognitiveLayer),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CognitiveCycleStateTransitionRequest {
    expected_anchor_state: CognitiveStructure,
    authority: CognitiveCycleTransitionAuthority,
    expected_authority_provenance: Option<CognitiveStructure>,
}

impl CognitiveCycleStateTransitionRequest {
    pub fn new(
        expected_anchor_state: CognitiveStructure,
        authority: CognitiveCycleTransitionAuthority,
        expected_authority_provenance: Option<CognitiveStructure>,
    ) -> Option<Self> {
        let provenance_contract_valid = match authority {
            CognitiveCycleTransitionAuthority::PreserveAnchor => {
                expected_authority_provenance.is_none()
            }
            CognitiveCycleTransitionAuthority::AdoptLayer(_) => {
                expected_authority_provenance.is_some()
            }
        };

        if !provenance_contract_valid {
            return None;
        }

        Some(Self {
            expected_anchor_state,
            authority,
            expected_authority_provenance,
        })
    }

    pub fn expected_anchor_state(&self) -> &CognitiveStructure {
        &self.expected_anchor_state
    }

    pub fn authority(&self) -> CognitiveCycleTransitionAuthority {
        self.authority
    }

    pub fn expected_authority_provenance(&self) -> Option<&CognitiveStructure> {
        self.expected_authority_provenance.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CognitiveCycleStateTransitionStatus {
    Transitioned,
    PreservedAnchor,
    CycleNotIntegrated,
    AnchorMismatch,
    AuthorityLayerMissing,
    AuthorityProvenanceMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CognitiveCycleStateTransitionResult {
    status: CognitiveCycleStateTransitionStatus,
    authority: CognitiveCycleTransitionAuthority,
    previous_anchor_state: CognitiveStructure,
    next_anchor_state: Option<CognitiveStructure>,
    authority_provenance: Option<CognitiveStructure>,
}

impl CognitiveCycleStateTransitionResult {
    pub fn status(&self) -> CognitiveCycleStateTransitionStatus {
        self.status
    }

    pub fn authority(&self) -> CognitiveCycleTransitionAuthority {
        self.authority
    }

    pub fn previous_anchor_state(&self) -> &CognitiveStructure {
        &self.previous_anchor_state
    }

    pub fn next_anchor_state(&self) -> Option<&CognitiveStructure> {
        self.next_anchor_state.as_ref()
    }

    pub fn authority_provenance(&self) -> Option<&CognitiveStructure> {
        self.authority_provenance.as_ref()
    }

    pub fn accepted(&self) -> bool {
        matches!(
            self.status,
            CognitiveCycleStateTransitionStatus::Transitioned
                | CognitiveCycleStateTransitionStatus::PreservedAnchor
        )
    }

    pub fn transitioned(&self) -> bool {
        self.status == CognitiveCycleStateTransitionStatus::Transitioned
    }

    pub fn preserved_anchor(&self) -> bool {
        self.status == CognitiveCycleStateTransitionStatus::PreservedAnchor
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CognitiveCycleStateTransition;

impl CognitiveCycleStateTransition {
    fn reject(
        cycle: &IntegratedCognitiveCycleResult,
        request: &CognitiveCycleStateTransitionRequest,
        status: CognitiveCycleStateTransitionStatus,
    ) -> CognitiveCycleStateTransitionResult {
        CognitiveCycleStateTransitionResult {
            status,
            authority: request.authority(),
            previous_anchor_state: cycle.anchor_state().clone(),
            next_anchor_state: None,
            authority_provenance: None,
        }
    }

    pub fn apply(
        cycle: &IntegratedCognitiveCycleResult,
        request: &CognitiveCycleStateTransitionRequest,
    ) -> CognitiveCycleStateTransitionResult {
        if !cycle.integrated() {
            return Self::reject(
                cycle,
                request,
                CognitiveCycleStateTransitionStatus::CycleNotIntegrated,
            );
        }

        if cycle.anchor_state() != request.expected_anchor_state() {
            return Self::reject(
                cycle,
                request,
                CognitiveCycleStateTransitionStatus::AnchorMismatch,
            );
        }

        match request.authority() {
            CognitiveCycleTransitionAuthority::PreserveAnchor => {
                CognitiveCycleStateTransitionResult {
                    status: CognitiveCycleStateTransitionStatus::PreservedAnchor,
                    authority: request.authority(),
                    previous_anchor_state: cycle.anchor_state().clone(),
                    next_anchor_state: Some(cycle.anchor_state().clone()),
                    authority_provenance: None,
                }
            }

            CognitiveCycleTransitionAuthority::AdoptLayer(layer) => {
                let Some(phase) = cycle.phase(layer) else {
                    return Self::reject(
                        cycle,
                        request,
                        CognitiveCycleStateTransitionStatus::AuthorityLayerMissing,
                    );
                };

                let Some(expected_provenance) = request.expected_authority_provenance() else {
                    return Self::reject(
                        cycle,
                        request,
                        CognitiveCycleStateTransitionStatus::AuthorityProvenanceMismatch,
                    );
                };

                if phase.provenance() != expected_provenance {
                    return Self::reject(
                        cycle,
                        request,
                        CognitiveCycleStateTransitionStatus::AuthorityProvenanceMismatch,
                    );
                }

                CognitiveCycleStateTransitionResult {
                    status: CognitiveCycleStateTransitionStatus::Transitioned,
                    authority: request.authority(),
                    previous_anchor_state: cycle.anchor_state().clone(),
                    next_anchor_state: Some(phase.result_state().clone()),
                    authority_provenance: Some(phase.provenance().clone()),
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalCognitiveCycleStateTransition;

impl UniversalCognitiveCycleStateTransition {
    pub fn evaluate(
        cycle: &IntegratedCognitiveCycleResult,
        request: &CognitiveCycleStateTransitionRequest,
    ) -> CognitiveCycleStateTransitionResult {
        CognitiveCycleStateTransition::apply(cycle, request)
    }
}

#[cfg(test)]
mod cognitive_cycle_state_transition_tests {
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

    fn policy() -> IntegratedAgentPolicy {
        IntegratedAgentPolicy::new(
            IntegratedAgentBounds::new(5, 2000).unwrap(),
            IntegratedAgentThresholds::new(s(500)).unwrap(),
        )
    }

    fn contribution(
        layer: IntegratedCognitiveLayer,
        result_state: u64,
        provenance: u64,
    ) -> IntegratedLayerContribution {
        IntegratedLayerContribution::new(
            layer,
            a(1000),
            a(result_state),
            a(provenance),
            s(900),
            s(200),
        )
        .unwrap()
    }

    fn contributions() -> Vec<IntegratedLayerContribution> {
        vec![
            contribution(IntegratedCognitiveLayer::PerceptualGrounding, 1001, 9001),
            contribution(
                IntegratedCognitiveLayer::UniversalDomainLearning,
                1002,
                9002,
            ),
            contribution(IntegratedCognitiveLayer::ExecutiveAgency, 1003, 9003),
            contribution(
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                1004,
                9004,
            ),
            contribution(
                IntegratedCognitiveLayer::AutonomousExperimentation,
                1005,
                9005,
            ),
        ]
    }

    fn cycle() -> IntegratedCognitiveCycleResult {
        IntegratedCognitiveCycle::run(&a(1000), &contributions(), policy())
    }

    fn adopt(
        layer: IntegratedCognitiveLayer,
        provenance: u64,
    ) -> CognitiveCycleStateTransitionRequest {
        CognitiveCycleStateTransitionRequest::new(
            a(1000),
            CognitiveCycleTransitionAuthority::AdoptLayer(layer),
            Some(a(provenance)),
        )
        .unwrap()
    }

    #[test]
    fn transition_request_requires_authority_specific_provenance_contract() {
        assert!(
            CognitiveCycleStateTransitionRequest::new(
                a(1000),
                CognitiveCycleTransitionAuthority::PreserveAnchor,
                None,
            )
            .is_some()
        );

        assert!(
            CognitiveCycleStateTransitionRequest::new(
                a(1000),
                CognitiveCycleTransitionAuthority::PreserveAnchor,
                Some(a(9000)),
            )
            .is_none()
        );

        assert!(
            CognitiveCycleStateTransitionRequest::new(
                a(1000),
                CognitiveCycleTransitionAuthority::AdoptLayer(
                    IntegratedCognitiveLayer::ExecutiveAgency,
                ),
                None,
            )
            .is_none()
        );
    }

    #[test]
    fn preserve_anchor_is_an_explicit_accepted_transition_decision() {
        let request = CognitiveCycleStateTransitionRequest::new(
            a(1000),
            CognitiveCycleTransitionAuthority::PreserveAnchor,
            None,
        )
        .unwrap();

        let result = CognitiveCycleStateTransition::apply(&cycle(), &request);

        assert!(result.accepted());

        assert!(result.preserved_anchor());

        assert_eq!(result.next_anchor_state(), Some(&a(1000)));

        assert_eq!(result.authority_provenance(), None);
    }

    #[test]
    fn every_frozen_layer_can_be_explicit_transition_authority() {
        for (layer, provenance, expected_state) in [
            (IntegratedCognitiveLayer::PerceptualGrounding, 9001, 1001),
            (
                IntegratedCognitiveLayer::UniversalDomainLearning,
                9002,
                1002,
            ),
            (IntegratedCognitiveLayer::ExecutiveAgency, 9003, 1003),
            (
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                9004,
                1004,
            ),
            (
                IntegratedCognitiveLayer::AutonomousExperimentation,
                9005,
                1005,
            ),
        ] {
            let result = CognitiveCycleStateTransition::apply(&cycle(), &adopt(layer, provenance));

            assert!(result.transitioned());

            assert_eq!(result.next_anchor_state(), Some(&a(expected_state)));

            assert_eq!(
                result.authority(),
                CognitiveCycleTransitionAuthority::AdoptLayer(layer)
            );
        }
    }

    #[test]
    fn transition_preserves_exact_authority_provenance() {
        let result = CognitiveCycleStateTransition::apply(
            &cycle(),
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        );

        assert_eq!(result.authority_provenance(), Some(&a(9003)));

        assert_eq!(result.previous_anchor_state(), &a(1000));

        assert_eq!(result.next_anchor_state(), Some(&a(1003)));
    }

    #[test]
    fn stale_or_wrong_authority_provenance_rejects_transition_atomically() {
        let result = CognitiveCycleStateTransition::apply(
            &cycle(),
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9999),
        );

        assert_eq!(
            result.status(),
            CognitiveCycleStateTransitionStatus::AuthorityProvenanceMismatch
        );

        assert!(!result.accepted());

        assert!(result.next_anchor_state().is_none());
    }

    #[test]
    fn stale_expected_anchor_rejects_transition_atomically() {
        let request = CognitiveCycleStateTransitionRequest::new(
            a(9999),
            CognitiveCycleTransitionAuthority::AdoptLayer(
                IntegratedCognitiveLayer::ExecutiveAgency,
            ),
            Some(a(9003)),
        )
        .unwrap();

        let result = CognitiveCycleStateTransition::apply(&cycle(), &request);

        assert_eq!(
            result.status(),
            CognitiveCycleStateTransitionStatus::AnchorMismatch
        );

        assert!(result.next_anchor_state().is_none());
    }

    #[test]
    fn rejected_cognitive_cycle_cannot_advance_agent_state() {
        let mut incomplete = contributions();

        incomplete.pop();

        let rejected = IntegratedCognitiveCycle::run(&a(1000), &incomplete, policy());

        assert!(!rejected.integrated());

        let result = CognitiveCycleStateTransition::apply(
            &rejected,
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        );

        assert_eq!(
            result.status(),
            CognitiveCycleStateTransitionStatus::CycleNotIntegrated
        );

        assert!(result.next_anchor_state().is_none());
    }

    #[test]
    fn low_confidence_filtered_cycle_cannot_create_partial_transition() {
        let mut input = contributions();

        input[2] = IntegratedLayerContribution::new(
            IntegratedCognitiveLayer::ExecutiveAgency,
            a(1000),
            a(1003),
            a(9003),
            s(400),
            s(200),
        )
        .unwrap();

        let rejected = IntegratedCognitiveCycle::run(&a(1000), &input, policy());

        let result = CognitiveCycleStateTransition::apply(
            &rejected,
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        );

        assert_eq!(
            result.status(),
            CognitiveCycleStateTransitionStatus::CycleNotIntegrated
        );

        assert!(!result.accepted());
    }

    #[test]
    fn transition_authority_selects_exact_opaque_state_without_merging_layers() {
        let executive = CognitiveCycleStateTransition::apply(
            &cycle(),
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        );

        let experimentation = CognitiveCycleStateTransition::apply(
            &cycle(),
            &adopt(IntegratedCognitiveLayer::AutonomousExperimentation, 9005),
        );

        assert_eq!(executive.next_anchor_state(), Some(&a(1003)));

        assert_eq!(experimentation.next_anchor_state(), Some(&a(1005)));

        assert_ne!(
            executive.next_anchor_state(),
            experimentation.next_anchor_state()
        );
    }

    #[test]
    fn cycle_input_order_cannot_change_authorized_transition() {
        let original = contributions();

        let mut reversed = original.clone();

        reversed.reverse();

        let first_cycle = IntegratedCognitiveCycle::run(&a(1000), &original, policy());

        let second_cycle = IntegratedCognitiveCycle::run(&a(1000), &reversed, policy());

        let request = adopt(IntegratedCognitiveLayer::MetaLearningSkillMemory, 9004);

        let first = CognitiveCycleStateTransition::apply(&first_cycle, &request);

        let second = CognitiveCycleStateTransition::apply(&second_cycle, &request);

        assert_eq!(first, second);
    }

    #[test]
    fn transition_does_not_mutate_integrated_cycle() {
        let cycle = cycle();

        let before = cycle.clone();

        let _ = CognitiveCycleStateTransition::apply(
            &cycle,
            &adopt(IntegratedCognitiveLayer::UniversalDomainLearning, 9002),
        );

        assert_eq!(cycle, before);
    }

    #[test]
    fn transition_is_deterministic_and_universal_facade_equivalent() {
        let cycle = cycle();

        let request = adopt(IntegratedCognitiveLayer::AutonomousExperimentation, 9005);

        let direct = CognitiveCycleStateTransition::apply(&cycle, &request);

        let facade = UniversalCognitiveCycleStateTransition::evaluate(&cycle, &request);

        let repeated = UniversalCognitiveCycleStateTransition::evaluate(&cycle, &request);

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(facade.next_anchor_state(), Some(&a(1005)));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClosedLoopAgentStepStatus {
    Advanced,
    Preserved,
    RejectedCycle,
    RejectedTransition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedLoopAgentStepResult {
    status: ClosedLoopAgentStepStatus,
    cycle: IntegratedCognitiveCycleResult,
    transition: CognitiveCycleStateTransitionResult,
}

impl ClosedLoopAgentStepResult {
    pub fn status(&self) -> ClosedLoopAgentStepStatus {
        self.status
    }

    pub fn cycle(&self) -> &IntegratedCognitiveCycleResult {
        &self.cycle
    }

    pub fn transition(&self) -> &CognitiveCycleStateTransitionResult {
        &self.transition
    }

    pub fn previous_anchor_state(&self) -> &CognitiveStructure {
        self.transition.previous_anchor_state()
    }

    pub fn next_anchor_state(&self) -> Option<&CognitiveStructure> {
        self.transition.next_anchor_state()
    }

    pub fn advanced(&self) -> bool {
        self.status == ClosedLoopAgentStepStatus::Advanced
    }

    pub fn preserved(&self) -> bool {
        self.status == ClosedLoopAgentStepStatus::Preserved
    }

    pub fn rejected(&self) -> bool {
        matches!(
            self.status,
            ClosedLoopAgentStepStatus::RejectedCycle
                | ClosedLoopAgentStepStatus::RejectedTransition
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ClosedLoopAgentStep;

impl ClosedLoopAgentStep {
    pub fn run(
        anchor_state: &CognitiveStructure,
        contributions: &[IntegratedLayerContribution],
        cycle_policy: IntegratedAgentPolicy,
        transition_request: &CognitiveCycleStateTransitionRequest,
    ) -> ClosedLoopAgentStepResult {
        let cycle = IntegratedCognitiveCycle::run(anchor_state, contributions, cycle_policy);

        let transition = CognitiveCycleStateTransition::apply(&cycle, transition_request);

        let status = if !cycle.integrated() {
            ClosedLoopAgentStepStatus::RejectedCycle
        } else {
            match transition.status() {
                CognitiveCycleStateTransitionStatus::Transitioned => {
                    ClosedLoopAgentStepStatus::Advanced
                }

                CognitiveCycleStateTransitionStatus::PreservedAnchor => {
                    ClosedLoopAgentStepStatus::Preserved
                }

                _ => ClosedLoopAgentStepStatus::RejectedTransition,
            }
        };

        ClosedLoopAgentStepResult {
            status,
            cycle,
            transition,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalClosedLoopAgentStep;

impl UniversalClosedLoopAgentStep {
    pub fn evaluate(
        anchor_state: &CognitiveStructure,
        contributions: &[IntegratedLayerContribution],
        cycle_policy: IntegratedAgentPolicy,
        transition_request: &CognitiveCycleStateTransitionRequest,
    ) -> ClosedLoopAgentStepResult {
        ClosedLoopAgentStep::run(
            anchor_state,
            contributions,
            cycle_policy,
            transition_request,
        )
    }
}

#[cfg(test)]
mod closed_loop_agent_step_tests {
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

    fn policy() -> IntegratedAgentPolicy {
        IntegratedAgentPolicy::new(
            IntegratedAgentBounds::new(5, 2000).unwrap(),
            IntegratedAgentThresholds::new(s(500)).unwrap(),
        )
    }

    fn contribution(
        layer: IntegratedCognitiveLayer,
        result_state: u64,
        provenance: u64,
        confidence: u16,
        compute_cost: u16,
    ) -> IntegratedLayerContribution {
        IntegratedLayerContribution::new(
            layer,
            a(1000),
            a(result_state),
            a(provenance),
            s(confidence),
            s(compute_cost),
        )
        .unwrap()
    }

    fn contributions() -> Vec<IntegratedLayerContribution> {
        vec![
            contribution(
                IntegratedCognitiveLayer::PerceptualGrounding,
                1001,
                9001,
                900,
                200,
            ),
            contribution(
                IntegratedCognitiveLayer::UniversalDomainLearning,
                1002,
                9002,
                900,
                250,
            ),
            contribution(
                IntegratedCognitiveLayer::ExecutiveAgency,
                1003,
                9003,
                900,
                225,
            ),
            contribution(
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                1004,
                9004,
                900,
                200,
            ),
            contribution(
                IntegratedCognitiveLayer::AutonomousExperimentation,
                1005,
                9005,
                900,
                225,
            ),
        ]
    }

    fn adopt(
        layer: IntegratedCognitiveLayer,
        provenance: u64,
    ) -> CognitiveCycleStateTransitionRequest {
        CognitiveCycleStateTransitionRequest::new(
            a(1000),
            CognitiveCycleTransitionAuthority::AdoptLayer(layer),
            Some(a(provenance)),
        )
        .unwrap()
    }

    fn preserve() -> CognitiveCycleStateTransitionRequest {
        CognitiveCycleStateTransitionRequest::new(
            a(1000),
            CognitiveCycleTransitionAuthority::PreserveAnchor,
            None,
        )
        .unwrap()
    }

    #[test]
    fn closed_loop_step_advances_by_explicit_executive_authority() {
        let result = ClosedLoopAgentStep::run(
            &a(1000),
            &contributions(),
            policy(),
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        );

        assert!(result.advanced());

        assert_eq!(result.status(), ClosedLoopAgentStepStatus::Advanced);

        assert_eq!(result.previous_anchor_state(), &a(1000));

        assert_eq!(result.next_anchor_state(), Some(&a(1003)));
    }

    #[test]
    fn preserve_anchor_is_successful_closed_loop_step() {
        let result = ClosedLoopAgentStep::run(&a(1000), &contributions(), policy(), &preserve());

        assert!(result.preserved());

        assert!(!result.advanced());

        assert_eq!(result.next_anchor_state(), Some(&a(1000)));

        assert_eq!(
            result.transition().authority(),
            CognitiveCycleTransitionAuthority::PreserveAnchor
        );
    }

    #[test]
    fn rejected_cycle_cannot_emit_next_anchor() {
        let mut input = contributions();

        input.pop();

        let result = ClosedLoopAgentStep::run(
            &a(1000),
            &input,
            policy(),
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        );

        assert_eq!(result.status(), ClosedLoopAgentStepStatus::RejectedCycle);

        assert!(result.rejected());

        assert!(result.next_anchor_state().is_none());

        assert_eq!(
            result.transition().status(),
            CognitiveCycleStateTransitionStatus::CycleNotIntegrated
        );
    }

    #[test]
    fn stale_transition_anchor_rejects_after_valid_cycle() {
        let request = CognitiveCycleStateTransitionRequest::new(
            a(9999),
            CognitiveCycleTransitionAuthority::AdoptLayer(
                IntegratedCognitiveLayer::ExecutiveAgency,
            ),
            Some(a(9003)),
        )
        .unwrap();

        let result = ClosedLoopAgentStep::run(&a(1000), &contributions(), policy(), &request);

        assert!(result.cycle().integrated());

        assert_eq!(
            result.status(),
            ClosedLoopAgentStepStatus::RejectedTransition
        );

        assert_eq!(
            result.transition().status(),
            CognitiveCycleStateTransitionStatus::AnchorMismatch
        );

        assert!(result.next_anchor_state().is_none());
    }

    #[test]
    fn stale_transition_provenance_rejects_after_valid_cycle() {
        let result = ClosedLoopAgentStep::run(
            &a(1000),
            &contributions(),
            policy(),
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9999),
        );

        assert!(result.cycle().integrated());

        assert_eq!(
            result.status(),
            ClosedLoopAgentStepStatus::RejectedTransition
        );

        assert_eq!(
            result.transition().status(),
            CognitiveCycleStateTransitionStatus::AuthorityProvenanceMismatch
        );

        assert!(result.next_anchor_state().is_none());
    }

    #[test]
    fn every_layer_can_drive_exact_closed_loop_transition() {
        for (layer, provenance, state) in [
            (IntegratedCognitiveLayer::PerceptualGrounding, 9001, 1001),
            (
                IntegratedCognitiveLayer::UniversalDomainLearning,
                9002,
                1002,
            ),
            (IntegratedCognitiveLayer::ExecutiveAgency, 9003, 1003),
            (
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                9004,
                1004,
            ),
            (
                IntegratedCognitiveLayer::AutonomousExperimentation,
                9005,
                1005,
            ),
        ] {
            let result = ClosedLoopAgentStep::run(
                &a(1000),
                &contributions(),
                policy(),
                &adopt(layer, provenance),
            );

            assert!(result.advanced());

            assert_eq!(result.next_anchor_state(), Some(&a(state)));
        }
    }

    #[test]
    fn closed_loop_preserves_transition_authority_and_provenance() {
        let result = ClosedLoopAgentStep::run(
            &a(1000),
            &contributions(),
            policy(),
            &adopt(IntegratedCognitiveLayer::MetaLearningSkillMemory, 9004),
        );

        assert_eq!(
            result.transition().authority(),
            CognitiveCycleTransitionAuthority::AdoptLayer(
                IntegratedCognitiveLayer::MetaLearningSkillMemory
            )
        );

        assert_eq!(result.transition().authority_provenance(), Some(&a(9004)));

        assert_eq!(result.next_anchor_state(), Some(&a(1004)));
    }

    #[test]
    fn cycle_input_order_cannot_change_closed_loop_step_result() {
        let original = contributions();

        let mut reversed = original.clone();

        reversed.reverse();

        let request = adopt(IntegratedCognitiveLayer::AutonomousExperimentation, 9005);

        let first = ClosedLoopAgentStep::run(&a(1000), &original, policy(), &request);

        let second = ClosedLoopAgentStep::run(&a(1000), &reversed, policy(), &request);

        assert_eq!(first, second);
    }

    #[test]
    fn low_confidence_filtering_rejects_step_atomically() {
        let mut input = contributions();

        input[2] = contribution(
            IntegratedCognitiveLayer::ExecutiveAgency,
            1003,
            9003,
            400,
            225,
        );

        let result = ClosedLoopAgentStep::run(
            &a(1000),
            &input,
            policy(),
            &adopt(IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        );

        assert_eq!(result.status(), ClosedLoopAgentStepStatus::RejectedCycle);

        assert_eq!(
            result.cycle().status(),
            IntegratedCognitiveCycleStatus::MissingRequiredLayer
        );

        assert!(result.next_anchor_state().is_none());
    }

    #[test]
    fn duplicate_layer_rejection_remains_visible_through_step() {
        let mut input = contributions();

        input[4] = contribution(
            IntegratedCognitiveLayer::PerceptualGrounding,
            7777,
            9777,
            900,
            100,
        );

        let result = ClosedLoopAgentStep::run(&a(1000), &input, policy(), &preserve());

        assert_eq!(result.status(), ClosedLoopAgentStepStatus::RejectedCycle);

        assert_eq!(
            result.cycle().foundation_status(),
            &IntegratedAgentFoundationStatus::DuplicateLayerContribution
        );

        assert!(result.next_anchor_state().is_none());
    }

    #[test]
    fn closed_loop_step_does_not_mutate_inputs() {
        let input = contributions();

        let before = input.clone();

        let request = adopt(IntegratedCognitiveLayer::UniversalDomainLearning, 9002);

        let request_before = request.clone();

        let result = ClosedLoopAgentStep::run(&a(1000), &input, policy(), &request);

        assert!(result.advanced());

        assert_eq!(input, before);

        assert_eq!(request, request_before);
    }

    #[test]
    fn closed_loop_step_is_deterministic_and_universal_facade_equivalent() {
        let input = contributions();

        let request = adopt(IntegratedCognitiveLayer::AutonomousExperimentation, 9005);

        let direct = ClosedLoopAgentStep::run(&a(1000), &input, policy(), &request);

        let facade = UniversalClosedLoopAgentStep::evaluate(&a(1000), &input, policy(), &request);

        let repeated = UniversalClosedLoopAgentStep::evaluate(&a(1000), &input, policy(), &request);

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(facade.status(), ClosedLoopAgentStepStatus::Advanced);

        assert_eq!(facade.next_anchor_state(), Some(&a(1005)));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurrentAgentStepInput {
    contributions: Vec<IntegratedLayerContribution>,
    transition_request: CognitiveCycleStateTransitionRequest,
}

impl RecurrentAgentStepInput {
    pub fn new(
        contributions: Vec<IntegratedLayerContribution>,
        transition_request: CognitiveCycleStateTransitionRequest,
    ) -> Self {
        Self {
            contributions,
            transition_request,
        }
    }

    pub fn contributions(&self) -> &[IntegratedLayerContribution] {
        &self.contributions
    }

    pub fn transition_request(&self) -> &CognitiveCycleStateTransitionRequest {
        &self.transition_request
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedRecurrentAgentLoopPolicy {
    max_steps: usize,
}

impl BoundedRecurrentAgentLoopPolicy {
    pub fn new(max_steps: usize) -> Option<Self> {
        if max_steps == 0 {
            return None;
        }

        Some(Self { max_steps })
    }

    pub fn max_steps(self) -> usize {
        self.max_steps
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedRecurrentAgentLoopStatus {
    Completed,
    InputFrontierExceeded,
    StepRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedRecurrentAgentLoopResult {
    status: BoundedRecurrentAgentLoopStatus,
    initial_anchor_state: CognitiveStructure,
    final_anchor_state: Option<CognitiveStructure>,
    steps: Vec<ClosedLoopAgentStepResult>,
    completed_step_count: usize,
    rejected_step_index: Option<usize>,
}

impl BoundedRecurrentAgentLoopResult {
    pub fn status(&self) -> BoundedRecurrentAgentLoopStatus {
        self.status
    }

    pub fn initial_anchor_state(&self) -> &CognitiveStructure {
        &self.initial_anchor_state
    }

    pub fn final_anchor_state(&self) -> Option<&CognitiveStructure> {
        self.final_anchor_state.as_ref()
    }

    pub fn steps(&self) -> &[ClosedLoopAgentStepResult] {
        &self.steps
    }

    pub fn executed_step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn completed_step_count(&self) -> usize {
        self.completed_step_count
    }

    pub fn rejected_step_index(&self) -> Option<usize> {
        self.rejected_step_index
    }

    pub fn completed(&self) -> bool {
        self.status == BoundedRecurrentAgentLoopStatus::Completed
    }

    pub fn frontier_exceeded(&self) -> bool {
        self.status == BoundedRecurrentAgentLoopStatus::InputFrontierExceeded
    }

    pub fn rejected(&self) -> bool {
        self.status == BoundedRecurrentAgentLoopStatus::StepRejected
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BoundedRecurrentAgentLoop;

impl BoundedRecurrentAgentLoop {
    pub fn run(
        initial_anchor_state: &CognitiveStructure,
        step_inputs: &[RecurrentAgentStepInput],
        cycle_policy: IntegratedAgentPolicy,
        loop_policy: BoundedRecurrentAgentLoopPolicy,
    ) -> BoundedRecurrentAgentLoopResult {
        if step_inputs.len() > loop_policy.max_steps() {
            return BoundedRecurrentAgentLoopResult {
                status: BoundedRecurrentAgentLoopStatus::InputFrontierExceeded,
                initial_anchor_state: initial_anchor_state.clone(),
                final_anchor_state: None,
                steps: Vec::new(),
                completed_step_count: 0,
                rejected_step_index: None,
            };
        }

        let mut current_anchor = initial_anchor_state.clone();

        let mut steps = Vec::with_capacity(step_inputs.len());

        let mut completed_step_count = 0;

        for (index, input) in step_inputs.iter().enumerate() {
            let result = ClosedLoopAgentStep::run(
                &current_anchor,
                input.contributions(),
                cycle_policy,
                input.transition_request(),
            );

            if result.rejected() {
                steps.push(result);

                return BoundedRecurrentAgentLoopResult {
                    status: BoundedRecurrentAgentLoopStatus::StepRejected,
                    initial_anchor_state: initial_anchor_state.clone(),
                    final_anchor_state: Some(current_anchor),
                    steps,
                    completed_step_count,
                    rejected_step_index: Some(index),
                };
            }

            let Some(next_anchor) = result.next_anchor_state().cloned() else {
                steps.push(result);

                return BoundedRecurrentAgentLoopResult {
                    status: BoundedRecurrentAgentLoopStatus::StepRejected,
                    initial_anchor_state: initial_anchor_state.clone(),
                    final_anchor_state: Some(current_anchor),
                    steps,
                    completed_step_count,
                    rejected_step_index: Some(index),
                };
            };

            current_anchor = next_anchor;

            completed_step_count += 1;

            steps.push(result);
        }

        BoundedRecurrentAgentLoopResult {
            status: BoundedRecurrentAgentLoopStatus::Completed,
            initial_anchor_state: initial_anchor_state.clone(),
            final_anchor_state: Some(current_anchor),
            steps,
            completed_step_count,
            rejected_step_index: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalBoundedRecurrentAgentLoop;

impl UniversalBoundedRecurrentAgentLoop {
    pub fn evaluate(
        initial_anchor_state: &CognitiveStructure,
        step_inputs: &[RecurrentAgentStepInput],
        cycle_policy: IntegratedAgentPolicy,
        loop_policy: BoundedRecurrentAgentLoopPolicy,
    ) -> BoundedRecurrentAgentLoopResult {
        BoundedRecurrentAgentLoop::run(initial_anchor_state, step_inputs, cycle_policy, loop_policy)
    }
}

#[cfg(test)]
mod bounded_recurrent_agent_loop_tests {
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

    fn cycle_policy() -> IntegratedAgentPolicy {
        IntegratedAgentPolicy::new(
            IntegratedAgentBounds::new(5, 2000).unwrap(),
            IntegratedAgentThresholds::new(s(500)).unwrap(),
        )
    }

    fn loop_policy(max_steps: usize) -> BoundedRecurrentAgentLoopPolicy {
        BoundedRecurrentAgentLoopPolicy::new(max_steps).unwrap()
    }

    fn contribution(
        anchor: u64,
        layer: IntegratedCognitiveLayer,
        result_state: u64,
        provenance: u64,
    ) -> IntegratedLayerContribution {
        IntegratedLayerContribution::new(
            layer,
            a(anchor),
            a(result_state),
            a(provenance),
            s(900),
            s(200),
        )
        .unwrap()
    }

    fn contributions(
        anchor: u64,
        result_base: u64,
        provenance_base: u64,
    ) -> Vec<IntegratedLayerContribution> {
        vec![
            contribution(
                anchor,
                IntegratedCognitiveLayer::PerceptualGrounding,
                result_base + 1,
                provenance_base + 1,
            ),
            contribution(
                anchor,
                IntegratedCognitiveLayer::UniversalDomainLearning,
                result_base + 2,
                provenance_base + 2,
            ),
            contribution(
                anchor,
                IntegratedCognitiveLayer::ExecutiveAgency,
                result_base + 3,
                provenance_base + 3,
            ),
            contribution(
                anchor,
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                result_base + 4,
                provenance_base + 4,
            ),
            contribution(
                anchor,
                IntegratedCognitiveLayer::AutonomousExperimentation,
                result_base + 5,
                provenance_base + 5,
            ),
        ]
    }

    fn adopt(
        anchor: u64,
        layer: IntegratedCognitiveLayer,
        provenance: u64,
    ) -> CognitiveCycleStateTransitionRequest {
        CognitiveCycleStateTransitionRequest::new(
            a(anchor),
            CognitiveCycleTransitionAuthority::AdoptLayer(layer),
            Some(a(provenance)),
        )
        .unwrap()
    }

    fn preserve(anchor: u64) -> CognitiveCycleStateTransitionRequest {
        CognitiveCycleStateTransitionRequest::new(
            a(anchor),
            CognitiveCycleTransitionAuthority::PreserveAnchor,
            None,
        )
        .unwrap()
    }

    fn first_step() -> RecurrentAgentStepInput {
        RecurrentAgentStepInput::new(
            contributions(1000, 1000, 9000),
            adopt(1000, IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        )
    }

    fn second_step() -> RecurrentAgentStepInput {
        RecurrentAgentStepInput::new(
            contributions(1003, 2000, 9100),
            adopt(
                1003,
                IntegratedCognitiveLayer::AutonomousExperimentation,
                9105,
            ),
        )
    }

    #[test]
    fn recurrent_loop_policy_requires_positive_hard_step_bound() {
        assert_eq!(BoundedRecurrentAgentLoopPolicy::new(0), None);

        assert_eq!(
            BoundedRecurrentAgentLoopPolicy::new(3).unwrap().max_steps(),
            3
        );
    }

    #[test]
    fn empty_recurrent_sequence_completes_without_changing_anchor() {
        let result = BoundedRecurrentAgentLoop::run(&a(1000), &[], cycle_policy(), loop_policy(3));

        assert!(result.completed());

        assert_eq!(result.executed_step_count(), 0);

        assert_eq!(result.completed_step_count(), 0);

        assert_eq!(result.final_anchor_state(), Some(&a(1000)));
    }

    #[test]
    fn accepted_next_anchor_drives_the_following_agent_step() {
        let result = BoundedRecurrentAgentLoop::run(
            &a(1000),
            &[first_step(), second_step()],
            cycle_policy(),
            loop_policy(2),
        );

        assert!(result.completed());

        assert_eq!(result.completed_step_count(), 2);

        assert_eq!(result.steps()[0].next_anchor_state(), Some(&a(1003)));

        assert_eq!(result.steps()[1].previous_anchor_state(), &a(1003));

        assert_eq!(result.final_anchor_state(), Some(&a(2005)));
    }

    #[test]
    fn preserved_anchor_remains_authoritative_for_next_step() {
        let first = RecurrentAgentStepInput::new(contributions(1000, 1000, 9000), preserve(1000));

        let second = RecurrentAgentStepInput::new(
            contributions(1000, 2000, 9100),
            adopt(
                1000,
                IntegratedCognitiveLayer::UniversalDomainLearning,
                9102,
            ),
        );

        let result = BoundedRecurrentAgentLoop::run(
            &a(1000),
            &[first, second],
            cycle_policy(),
            loop_policy(2),
        );

        assert!(result.completed());

        assert!(result.steps()[0].preserved());

        assert_eq!(result.steps()[1].previous_anchor_state(), &a(1000));

        assert_eq!(result.final_anchor_state(), Some(&a(2002)));
    }

    #[test]
    fn hard_step_frontier_rejects_entire_sequence_before_execution() {
        let result = BoundedRecurrentAgentLoop::run(
            &a(1000),
            &[first_step(), second_step()],
            cycle_policy(),
            loop_policy(1),
        );

        assert!(result.frontier_exceeded());

        assert_eq!(result.executed_step_count(), 0);

        assert_eq!(result.completed_step_count(), 0);

        assert_eq!(result.final_anchor_state(), None);
    }

    #[test]
    fn stale_second_step_contributions_halt_after_first_committed_step() {
        let stale_second = RecurrentAgentStepInput::new(
            contributions(1000, 2000, 9100),
            adopt(
                1003,
                IntegratedCognitiveLayer::AutonomousExperimentation,
                9105,
            ),
        );

        let result = BoundedRecurrentAgentLoop::run(
            &a(1000),
            &[first_step(), stale_second],
            cycle_policy(),
            loop_policy(2),
        );

        assert!(result.rejected());

        assert_eq!(result.completed_step_count(), 1);

        assert_eq!(result.rejected_step_index(), Some(1));

        assert_eq!(result.final_anchor_state(), Some(&a(1003)));

        assert_eq!(
            result.steps()[1].status(),
            ClosedLoopAgentStepStatus::RejectedCycle
        );
    }

    #[test]
    fn stale_second_transition_anchor_halts_after_first_committed_step() {
        let stale_second = RecurrentAgentStepInput::new(
            contributions(1003, 2000, 9100),
            adopt(
                1000,
                IntegratedCognitiveLayer::AutonomousExperimentation,
                9105,
            ),
        );

        let result = BoundedRecurrentAgentLoop::run(
            &a(1000),
            &[first_step(), stale_second],
            cycle_policy(),
            loop_policy(2),
        );

        assert!(result.rejected());

        assert_eq!(result.completed_step_count(), 1);

        assert_eq!(result.final_anchor_state(), Some(&a(1003)));

        assert_eq!(
            result.steps()[1].status(),
            ClosedLoopAgentStepStatus::RejectedTransition
        );
    }

    #[test]
    fn rejection_at_first_step_preserves_initial_stable_anchor() {
        let stale = RecurrentAgentStepInput::new(
            contributions(9999, 1000, 9000),
            adopt(1000, IntegratedCognitiveLayer::ExecutiveAgency, 9003),
        );

        let result =
            BoundedRecurrentAgentLoop::run(&a(1000), &[stale], cycle_policy(), loop_policy(1));

        assert!(result.rejected());

        assert_eq!(result.completed_step_count(), 0);

        assert_eq!(result.rejected_step_index(), Some(0));

        assert_eq!(result.final_anchor_state(), Some(&a(1000)));
    }

    #[test]
    fn rejection_halts_loop_before_any_later_step_executes() {
        let stale_second = RecurrentAgentStepInput::new(
            contributions(1000, 2000, 9100),
            adopt(
                1003,
                IntegratedCognitiveLayer::AutonomousExperimentation,
                9105,
            ),
        );

        let unreachable_third = RecurrentAgentStepInput::new(
            contributions(2005, 3000, 9200),
            adopt(2005, IntegratedCognitiveLayer::ExecutiveAgency, 9203),
        );

        let result = BoundedRecurrentAgentLoop::run(
            &a(1000),
            &[first_step(), stale_second, unreachable_third],
            cycle_policy(),
            loop_policy(3),
        );

        assert!(result.rejected());

        assert_eq!(result.executed_step_count(), 2);

        assert_eq!(result.completed_step_count(), 1);

        assert_eq!(result.rejected_step_index(), Some(1));
    }

    #[test]
    fn contribution_order_inside_each_step_cannot_change_recurrent_result() {
        let first = first_step();

        let second = second_step();

        let mut first_reversed = first.contributions().to_vec();

        first_reversed.reverse();

        let mut second_reversed = second.contributions().to_vec();

        second_reversed.reverse();

        let reordered = [
            RecurrentAgentStepInput::new(first_reversed, first.transition_request().clone()),
            RecurrentAgentStepInput::new(second_reversed, second.transition_request().clone()),
        ];

        let canonical = BoundedRecurrentAgentLoop::run(
            &a(1000),
            &[first, second],
            cycle_policy(),
            loop_policy(2),
        );

        let reversed =
            BoundedRecurrentAgentLoop::run(&a(1000), &reordered, cycle_policy(), loop_policy(2));

        assert_eq!(canonical, reversed);
    }

    #[test]
    fn recurrent_loop_does_not_mutate_supplied_step_inputs() {
        let inputs = vec![first_step(), second_step()];

        let before = inputs.clone();

        let result =
            BoundedRecurrentAgentLoop::run(&a(1000), &inputs, cycle_policy(), loop_policy(2));

        assert!(result.completed());

        assert_eq!(inputs, before);
    }

    #[test]
    fn recurrent_loop_is_deterministic_and_universal_facade_equivalent() {
        let inputs = vec![first_step(), second_step()];

        let direct =
            BoundedRecurrentAgentLoop::run(&a(1000), &inputs, cycle_policy(), loop_policy(2));

        let facade = UniversalBoundedRecurrentAgentLoop::evaluate(
            &a(1000),
            &inputs,
            cycle_policy(),
            loop_policy(2),
        );

        let repeated = UniversalBoundedRecurrentAgentLoop::evaluate(
            &a(1000),
            &inputs,
            cycle_policy(),
            loop_policy(2),
        );

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(facade.initial_anchor_state(), &a(1000));

        assert_eq!(facade.final_anchor_state(), Some(&a(2005)));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurrentFeedbackEvidence {
    predecessor_step_index: usize,
    predecessor_next_anchor_state: CognitiveStructure,
    predecessor_authority_provenance: Option<CognitiveStructure>,
}

impl RecurrentFeedbackEvidence {
    pub fn new(
        predecessor_step_index: usize,
        predecessor_next_anchor_state: CognitiveStructure,
        predecessor_authority_provenance: Option<CognitiveStructure>,
    ) -> Self {
        Self {
            predecessor_step_index,
            predecessor_next_anchor_state,
            predecessor_authority_provenance,
        }
    }

    pub fn predecessor_step_index(&self) -> usize {
        self.predecessor_step_index
    }

    pub fn predecessor_next_anchor_state(&self) -> &CognitiveStructure {
        &self.predecessor_next_anchor_state
    }

    pub fn predecessor_authority_provenance(&self) -> Option<&CognitiveStructure> {
        self.predecessor_authority_provenance.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedbackBoundRecurrentAgentStepInput {
    step: RecurrentAgentStepInput,
    feedback: Option<RecurrentFeedbackEvidence>,
}

impl FeedbackBoundRecurrentAgentStepInput {
    pub fn new(step: RecurrentAgentStepInput, feedback: Option<RecurrentFeedbackEvidence>) -> Self {
        Self { step, feedback }
    }

    pub fn step(&self) -> &RecurrentAgentStepInput {
        &self.step
    }

    pub fn feedback(&self) -> Option<&RecurrentFeedbackEvidence> {
        self.feedback.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecurrentFeedbackClosureStatus {
    Completed,
    InputFrontierExceeded,
    FirstStepCarriesFeedback,
    MissingFeedback,
    FeedbackStepIndexMismatch,
    FeedbackAnchorMismatch,
    FeedbackProvenanceMismatch,
    StepRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurrentFeedbackClosureResult {
    status: RecurrentFeedbackClosureStatus,
    initial_anchor_state: CognitiveStructure,
    final_anchor_state: Option<CognitiveStructure>,
    steps: Vec<ClosedLoopAgentStepResult>,
    completed_step_count: usize,
    rejected_step_index: Option<usize>,
}

impl RecurrentFeedbackClosureResult {
    pub fn status(&self) -> RecurrentFeedbackClosureStatus {
        self.status
    }

    pub fn initial_anchor_state(&self) -> &CognitiveStructure {
        &self.initial_anchor_state
    }

    pub fn final_anchor_state(&self) -> Option<&CognitiveStructure> {
        self.final_anchor_state.as_ref()
    }

    pub fn steps(&self) -> &[ClosedLoopAgentStepResult] {
        &self.steps
    }

    pub fn executed_step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn completed_step_count(&self) -> usize {
        self.completed_step_count
    }

    pub fn rejected_step_index(&self) -> Option<usize> {
        self.rejected_step_index
    }

    pub fn completed(&self) -> bool {
        self.status == RecurrentFeedbackClosureStatus::Completed
    }

    pub fn rejected(&self) -> bool {
        !self.completed()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecurrentFeedbackClosure;

impl RecurrentFeedbackClosure {
    fn structural_rejection(
        status: RecurrentFeedbackClosureStatus,
        initial_anchor_state: &CognitiveStructure,
        current_anchor_state: Option<CognitiveStructure>,
        steps: Vec<ClosedLoopAgentStepResult>,
        completed_step_count: usize,
        rejected_step_index: Option<usize>,
    ) -> RecurrentFeedbackClosureResult {
        RecurrentFeedbackClosureResult {
            status,
            initial_anchor_state: initial_anchor_state.clone(),
            final_anchor_state: current_anchor_state,
            steps,
            completed_step_count,
            rejected_step_index,
        }
    }

    pub fn run(
        initial_anchor_state: &CognitiveStructure,
        step_inputs: &[FeedbackBoundRecurrentAgentStepInput],
        cycle_policy: IntegratedAgentPolicy,
        loop_policy: BoundedRecurrentAgentLoopPolicy,
    ) -> RecurrentFeedbackClosureResult {
        if step_inputs.len() > loop_policy.max_steps() {
            return Self::structural_rejection(
                RecurrentFeedbackClosureStatus::InputFrontierExceeded,
                initial_anchor_state,
                None,
                Vec::new(),
                0,
                None,
            );
        }

        if step_inputs
            .first()
            .and_then(|input| input.feedback())
            .is_some()
        {
            return Self::structural_rejection(
                RecurrentFeedbackClosureStatus::FirstStepCarriesFeedback,
                initial_anchor_state,
                Some(initial_anchor_state.clone()),
                Vec::new(),
                0,
                Some(0),
            );
        }

        let mut current_anchor = initial_anchor_state.clone();

        let mut steps = Vec::with_capacity(step_inputs.len());

        let mut completed_step_count = 0;

        for (index, input) in step_inputs.iter().enumerate() {
            if index > 0 {
                let Some(feedback) = input.feedback() else {
                    return Self::structural_rejection(
                        RecurrentFeedbackClosureStatus::MissingFeedback,
                        initial_anchor_state,
                        Some(current_anchor),
                        steps,
                        completed_step_count,
                        Some(index),
                    );
                };

                if feedback.predecessor_step_index() != index - 1 {
                    return Self::structural_rejection(
                        RecurrentFeedbackClosureStatus::FeedbackStepIndexMismatch,
                        initial_anchor_state,
                        Some(current_anchor),
                        steps,
                        completed_step_count,
                        Some(index),
                    );
                }

                if feedback.predecessor_next_anchor_state() != &current_anchor {
                    return Self::structural_rejection(
                        RecurrentFeedbackClosureStatus::FeedbackAnchorMismatch,
                        initial_anchor_state,
                        Some(current_anchor),
                        steps,
                        completed_step_count,
                        Some(index),
                    );
                }

                let previous = steps
                    .last()
                    .expect("index greater than zero requires a completed predecessor step");

                if feedback.predecessor_authority_provenance()
                    != previous.transition().authority_provenance()
                {
                    return Self::structural_rejection(
                        RecurrentFeedbackClosureStatus::FeedbackProvenanceMismatch,
                        initial_anchor_state,
                        Some(current_anchor),
                        steps,
                        completed_step_count,
                        Some(index),
                    );
                }
            }

            let step = input.step();

            let result = ClosedLoopAgentStep::run(
                &current_anchor,
                step.contributions(),
                cycle_policy,
                step.transition_request(),
            );

            if result.rejected() {
                steps.push(result);

                return Self::structural_rejection(
                    RecurrentFeedbackClosureStatus::StepRejected,
                    initial_anchor_state,
                    Some(current_anchor),
                    steps,
                    completed_step_count,
                    Some(index),
                );
            }

            let Some(next_anchor) = result.next_anchor_state().cloned() else {
                steps.push(result);

                return Self::structural_rejection(
                    RecurrentFeedbackClosureStatus::StepRejected,
                    initial_anchor_state,
                    Some(current_anchor),
                    steps,
                    completed_step_count,
                    Some(index),
                );
            };

            current_anchor = next_anchor;

            completed_step_count += 1;

            steps.push(result);
        }

        RecurrentFeedbackClosureResult {
            status: RecurrentFeedbackClosureStatus::Completed,
            initial_anchor_state: initial_anchor_state.clone(),
            final_anchor_state: Some(current_anchor),
            steps,
            completed_step_count,
            rejected_step_index: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalRecurrentFeedbackClosure;

impl UniversalRecurrentFeedbackClosure {
    pub fn evaluate(
        initial_anchor_state: &CognitiveStructure,
        step_inputs: &[FeedbackBoundRecurrentAgentStepInput],
        cycle_policy: IntegratedAgentPolicy,
        loop_policy: BoundedRecurrentAgentLoopPolicy,
    ) -> RecurrentFeedbackClosureResult {
        RecurrentFeedbackClosure::run(initial_anchor_state, step_inputs, cycle_policy, loop_policy)
    }
}

#[cfg(test)]
mod recurrent_feedback_closure_tests {
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

    fn cycle_policy() -> IntegratedAgentPolicy {
        IntegratedAgentPolicy::new(
            IntegratedAgentBounds::new(5, 2000).unwrap(),
            IntegratedAgentThresholds::new(s(500)).unwrap(),
        )
    }

    fn loop_policy(max_steps: usize) -> BoundedRecurrentAgentLoopPolicy {
        BoundedRecurrentAgentLoopPolicy::new(max_steps).unwrap()
    }

    fn contribution(
        anchor: u64,
        layer: IntegratedCognitiveLayer,
        result_state: u64,
        provenance: u64,
    ) -> IntegratedLayerContribution {
        IntegratedLayerContribution::new(
            layer,
            a(anchor),
            a(result_state),
            a(provenance),
            s(900),
            s(200),
        )
        .unwrap()
    }

    fn contributions(
        anchor: u64,
        result_base: u64,
        provenance_base: u64,
    ) -> Vec<IntegratedLayerContribution> {
        vec![
            contribution(
                anchor,
                IntegratedCognitiveLayer::PerceptualGrounding,
                result_base + 1,
                provenance_base + 1,
            ),
            contribution(
                anchor,
                IntegratedCognitiveLayer::UniversalDomainLearning,
                result_base + 2,
                provenance_base + 2,
            ),
            contribution(
                anchor,
                IntegratedCognitiveLayer::ExecutiveAgency,
                result_base + 3,
                provenance_base + 3,
            ),
            contribution(
                anchor,
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                result_base + 4,
                provenance_base + 4,
            ),
            contribution(
                anchor,
                IntegratedCognitiveLayer::AutonomousExperimentation,
                result_base + 5,
                provenance_base + 5,
            ),
        ]
    }

    fn adopt(
        anchor: u64,
        layer: IntegratedCognitiveLayer,
        provenance: u64,
    ) -> CognitiveCycleStateTransitionRequest {
        CognitiveCycleStateTransitionRequest::new(
            a(anchor),
            CognitiveCycleTransitionAuthority::AdoptLayer(layer),
            Some(a(provenance)),
        )
        .unwrap()
    }

    fn preserve(anchor: u64) -> CognitiveCycleStateTransitionRequest {
        CognitiveCycleStateTransitionRequest::new(
            a(anchor),
            CognitiveCycleTransitionAuthority::PreserveAnchor,
            None,
        )
        .unwrap()
    }

    fn first_step() -> FeedbackBoundRecurrentAgentStepInput {
        FeedbackBoundRecurrentAgentStepInput::new(
            RecurrentAgentStepInput::new(
                contributions(1000, 1000, 9000),
                adopt(1000, IntegratedCognitiveLayer::ExecutiveAgency, 9003),
            ),
            None,
        )
    }

    fn valid_feedback() -> RecurrentFeedbackEvidence {
        RecurrentFeedbackEvidence::new(0, a(1003), Some(a(9003)))
    }

    fn second_step(
        feedback: Option<RecurrentFeedbackEvidence>,
    ) -> FeedbackBoundRecurrentAgentStepInput {
        FeedbackBoundRecurrentAgentStepInput::new(
            RecurrentAgentStepInput::new(
                contributions(1003, 2000, 9100),
                adopt(
                    1003,
                    IntegratedCognitiveLayer::AutonomousExperimentation,
                    9105,
                ),
            ),
            feedback,
        )
    }

    #[test]
    fn first_recurrent_step_must_not_claim_predecessor_feedback() {
        let first = FeedbackBoundRecurrentAgentStepInput::new(
            first_step().step().clone(),
            Some(RecurrentFeedbackEvidence::new(0, a(1000), None)),
        );

        let result =
            RecurrentFeedbackClosure::run(&a(1000), &[first], cycle_policy(), loop_policy(1));

        assert_eq!(
            result.status(),
            RecurrentFeedbackClosureStatus::FirstStepCarriesFeedback
        );

        assert_eq!(result.executed_step_count(), 0);
    }

    #[test]
    fn subsequent_recurrent_step_requires_explicit_feedback() {
        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[first_step(), second_step(None)],
            cycle_policy(),
            loop_policy(2),
        );

        assert_eq!(
            result.status(),
            RecurrentFeedbackClosureStatus::MissingFeedback
        );

        assert_eq!(result.executed_step_count(), 1);

        assert_eq!(result.completed_step_count(), 1);
    }

    #[test]
    fn exact_predecessor_step_index_is_required() {
        let feedback = RecurrentFeedbackEvidence::new(7, a(1003), Some(a(9003)));

        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[first_step(), second_step(Some(feedback))],
            cycle_policy(),
            loop_policy(2),
        );

        assert_eq!(
            result.status(),
            RecurrentFeedbackClosureStatus::FeedbackStepIndexMismatch
        );

        assert_eq!(result.rejected_step_index(), Some(1));
    }

    #[test]
    fn exact_predecessor_next_anchor_is_required() {
        let feedback = RecurrentFeedbackEvidence::new(0, a(9999), Some(a(9003)));

        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[first_step(), second_step(Some(feedback))],
            cycle_policy(),
            loop_policy(2),
        );

        assert_eq!(
            result.status(),
            RecurrentFeedbackClosureStatus::FeedbackAnchorMismatch
        );

        assert_eq!(result.final_anchor_state(), Some(&a(1003)));
    }

    #[test]
    fn exact_predecessor_authority_provenance_is_required() {
        let feedback = RecurrentFeedbackEvidence::new(0, a(1003), Some(a(9999)));

        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[first_step(), second_step(Some(feedback))],
            cycle_policy(),
            loop_policy(2),
        );

        assert_eq!(
            result.status(),
            RecurrentFeedbackClosureStatus::FeedbackProvenanceMismatch
        );

        assert_eq!(result.completed_step_count(), 1);
    }

    #[test]
    fn valid_feedback_closes_two_step_recurrent_causal_chain() {
        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[first_step(), second_step(Some(valid_feedback()))],
            cycle_policy(),
            loop_policy(2),
        );

        assert!(result.completed());

        assert_eq!(result.completed_step_count(), 2);

        assert_eq!(result.final_anchor_state(), Some(&a(2005)));

        assert_eq!(result.steps()[1].previous_anchor_state(), &a(1003));
    }

    #[test]
    fn preserve_anchor_feedback_uses_absence_of_authority_provenance_exactly() {
        let first = FeedbackBoundRecurrentAgentStepInput::new(
            RecurrentAgentStepInput::new(contributions(1000, 1000, 9000), preserve(1000)),
            None,
        );

        let second = FeedbackBoundRecurrentAgentStepInput::new(
            RecurrentAgentStepInput::new(
                contributions(1000, 2000, 9100),
                adopt(
                    1000,
                    IntegratedCognitiveLayer::UniversalDomainLearning,
                    9102,
                ),
            ),
            Some(RecurrentFeedbackEvidence::new(0, a(1000), None)),
        );

        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[first, second],
            cycle_policy(),
            loop_policy(2),
        );

        assert!(result.completed());

        assert_eq!(result.final_anchor_state(), Some(&a(2002)));
    }

    #[test]
    fn feedback_failure_prevents_current_step_execution() {
        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[
                first_step(),
                second_step(Some(RecurrentFeedbackEvidence::new(
                    0,
                    a(9999),
                    Some(a(9003)),
                ))),
            ],
            cycle_policy(),
            loop_policy(2),
        );

        assert_eq!(result.executed_step_count(), 1);

        assert_eq!(result.completed_step_count(), 1);

        assert_eq!(result.final_anchor_state(), Some(&a(1003)));
    }

    #[test]
    fn hard_frontier_rejects_before_feedback_or_step_execution() {
        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[first_step(), second_step(Some(valid_feedback()))],
            cycle_policy(),
            loop_policy(1),
        );

        assert_eq!(
            result.status(),
            RecurrentFeedbackClosureStatus::InputFrontierExceeded
        );

        assert_eq!(result.executed_step_count(), 0);

        assert_eq!(result.final_anchor_state(), None);
    }

    #[test]
    fn valid_feedback_does_not_override_stale_current_step_inputs() {
        let stale_second = FeedbackBoundRecurrentAgentStepInput::new(
            RecurrentAgentStepInput::new(
                contributions(1000, 2000, 9100),
                adopt(
                    1003,
                    IntegratedCognitiveLayer::AutonomousExperimentation,
                    9105,
                ),
            ),
            Some(valid_feedback()),
        );

        let result = RecurrentFeedbackClosure::run(
            &a(1000),
            &[first_step(), stale_second],
            cycle_policy(),
            loop_policy(2),
        );

        assert_eq!(
            result.status(),
            RecurrentFeedbackClosureStatus::StepRejected
        );

        assert_eq!(result.executed_step_count(), 2);

        assert_eq!(result.completed_step_count(), 1);

        assert_eq!(result.final_anchor_state(), Some(&a(1003)));
    }

    #[test]
    fn feedback_closed_loop_does_not_mutate_supplied_evidence_or_steps() {
        let inputs = vec![first_step(), second_step(Some(valid_feedback()))];

        let before = inputs.clone();

        let result =
            RecurrentFeedbackClosure::run(&a(1000), &inputs, cycle_policy(), loop_policy(2));

        assert!(result.completed());

        assert_eq!(inputs, before);
    }

    #[test]
    fn feedback_closure_is_deterministic_and_universal_facade_equivalent() {
        let inputs = vec![first_step(), second_step(Some(valid_feedback()))];

        let direct =
            RecurrentFeedbackClosure::run(&a(1000), &inputs, cycle_policy(), loop_policy(2));

        let facade = UniversalRecurrentFeedbackClosure::evaluate(
            &a(1000),
            &inputs,
            cycle_policy(),
            loop_policy(2),
        );

        let repeated = UniversalRecurrentFeedbackClosure::evaluate(
            &a(1000),
            &inputs,
            cycle_policy(),
            loop_policy(2),
        );

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(facade.initial_anchor_state(), &a(1000));

        assert_eq!(facade.final_anchor_state(), Some(&a(2005)));
    }
}

pub struct OnlinePerceptualGroundingRuntime<'a> {
    request: &'a PerceptualGroundingIngestionRequest,
    input: &'a athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldInput,
    context: athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext,
    policy: PerceptualGroundingIngestionPolicy,
}

impl<'a> OnlinePerceptualGroundingRuntime<'a> {
    pub fn new(
        request: &'a PerceptualGroundingIngestionRequest,
        input: &'a athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldInput,
        context: athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext,
        policy: PerceptualGroundingIngestionPolicy,
    ) -> Self {
        Self {
            request,
            input,
            context,
            policy,
        }
    }
}

pub struct OnlineUniversalDomainRuntime<'a> {
    request: &'a UniversalDomainLearningIngestionRequest,
    local: &'a [athlesia_universal_domain_learning::GroundedInterventionalCausalHypothesis],
    transferred: &'a [athlesia_universal_domain_learning::CompressedDomainModel],
    domain_policy: athlesia_universal_domain_learning::IntegratedDomainModelPolicy,
    ingestion_policy: UniversalDomainLearningIngestionPolicy,
}

impl<'a> OnlineUniversalDomainRuntime<'a> {
    pub fn new(
        request: &'a UniversalDomainLearningIngestionRequest,
        local: &'a [athlesia_universal_domain_learning::GroundedInterventionalCausalHypothesis],
        transferred: &'a [athlesia_universal_domain_learning::CompressedDomainModel],
        domain_policy: athlesia_universal_domain_learning::IntegratedDomainModelPolicy,
        ingestion_policy: UniversalDomainLearningIngestionPolicy,
    ) -> Self {
        Self {
            request,
            local,
            transferred,
            domain_policy,
            ingestion_policy,
        }
    }
}

pub struct OnlineExecutiveAgencyRuntime<'a> {
    request: &'a ExecutiveAgencyIngestionRequest,
    context: athlesia_executive_agency::IntegratedExecutiveControlContext<'a>,
    policy: athlesia_executive_agency::IntegratedExecutiveControlPolicy,
}

impl<'a> OnlineExecutiveAgencyRuntime<'a> {
    pub fn new(
        request: &'a ExecutiveAgencyIngestionRequest,
        context: athlesia_executive_agency::IntegratedExecutiveControlContext<'a>,
        policy: athlesia_executive_agency::IntegratedExecutiveControlPolicy,
    ) -> Self {
        Self {
            request,
            context,
            policy,
        }
    }
}

pub struct OnlineSkillMemoryRuntime<'a> {
    request: &'a MetaLearningSkillMemoryIngestionRequest,
    input: &'a athlesia_meta_learning_skill_memory::IntegratedSkillLearningCycleInput,
    policy: athlesia_meta_learning_skill_memory::IntegratedSkillLearningCyclePolicy,
}

impl<'a> OnlineSkillMemoryRuntime<'a> {
    pub fn new(
        request: &'a MetaLearningSkillMemoryIngestionRequest,
        input: &'a athlesia_meta_learning_skill_memory::IntegratedSkillLearningCycleInput,
        policy: athlesia_meta_learning_skill_memory::IntegratedSkillLearningCyclePolicy,
    ) -> Self {
        Self {
            request,
            input,
            policy,
        }
    }
}

pub struct OnlineAutonomousExperimentationRuntime<'a> {
    request: &'a AutonomousExperimentationIngestionRequest,
    beliefs: &'a [athlesia_autonomous_active_experimentation::HypothesisBeliefState],
    possibilities:
        &'a [athlesia_autonomous_active_experimentation::GroundedExperimentPossibility],
    learning_samples:
        &'a [athlesia_autonomous_active_experimentation::ExperimentLearningProgressSample],
    current_experiment_cycle: usize,
    policy: athlesia_autonomous_active_experimentation::IntegratedAutonomousExperimentationPolicy,
}

impl<'a> OnlineAutonomousExperimentationRuntime<'a> {
    pub fn new(
        request: &'a AutonomousExperimentationIngestionRequest,
        beliefs: &'a [athlesia_autonomous_active_experimentation::HypothesisBeliefState],
        possibilities: &'a [athlesia_autonomous_active_experimentation::GroundedExperimentPossibility],
        learning_samples: &'a [athlesia_autonomous_active_experimentation::ExperimentLearningProgressSample],
        current_experiment_cycle: usize,
        policy: athlesia_autonomous_active_experimentation::IntegratedAutonomousExperimentationPolicy,
    ) -> Self {
        Self {
            request,
            beliefs,
            possibilities,
            learning_samples,
            current_experiment_cycle,
            policy,
        }
    }
}

pub struct OnlineCognitiveOrchestrationInput<'a> {
    perceptual: OnlinePerceptualGroundingRuntime<'a>,
    domain: OnlineUniversalDomainRuntime<'a>,
    executive: OnlineExecutiveAgencyRuntime<'a>,
    skill_memory: OnlineSkillMemoryRuntime<'a>,
    experimentation: OnlineAutonomousExperimentationRuntime<'a>,
}

impl<'a> OnlineCognitiveOrchestrationInput<'a> {
    pub fn new(
        perceptual: OnlinePerceptualGroundingRuntime<'a>,
        domain: OnlineUniversalDomainRuntime<'a>,
        executive: OnlineExecutiveAgencyRuntime<'a>,
        skill_memory: OnlineSkillMemoryRuntime<'a>,
        experimentation: OnlineAutonomousExperimentationRuntime<'a>,
    ) -> Self {
        Self {
            perceptual,
            domain,
            executive,
            skill_memory,
            experimentation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnlineCognitiveOrchestrationStatus {
    Advanced,
    Preserved,
    RequestAnchorMismatch(IntegratedCognitiveLayer),
    PerceptualGroundingRejected,
    UniversalDomainLearningRejected,
    ExecutiveAgencyRejected,
    MetaLearningSkillMemoryRejected,
    AutonomousExperimentationRejected,
    CognitiveCycleRejected,
    StateTransitionRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnlineCognitiveOrchestrationResult {
    status: OnlineCognitiveOrchestrationStatus,
    contributions: Vec<IntegratedLayerContribution>,
    executive_decision: Option<athlesia_executive_agency::IntegratedExecutiveControlDecision>,
    executive_selection: Option<athlesia_executive_agency::IntegratedExecutiveSelection>,
    skill_reused: Option<bool>,
    experimentation_status: Option<
        athlesia_autonomous_active_experimentation::IntegratedAutonomousExperimentationStatus,
    >,
    step: Option<ClosedLoopAgentStepResult>,
}

impl OnlineCognitiveOrchestrationResult {
    fn partial(
        status: OnlineCognitiveOrchestrationStatus,
        contributions: Vec<IntegratedLayerContribution>,
        executive_decision: Option<athlesia_executive_agency::IntegratedExecutiveControlDecision>,
        skill_reused: Option<bool>,
        experimentation_status: Option<
            athlesia_autonomous_active_experimentation::IntegratedAutonomousExperimentationStatus,
        >,
    ) -> Self {
        Self {
            status,
            contributions,
            executive_decision,
            executive_selection: None,
            skill_reused,
            experimentation_status,
            step: None,
        }
    }

    pub fn status(&self) -> OnlineCognitiveOrchestrationStatus {
        self.status
    }

    pub fn contributions(&self) -> &[IntegratedLayerContribution] {
        &self.contributions
    }

    pub fn contribution(
        &self,
        layer: IntegratedCognitiveLayer,
    ) -> Option<&IntegratedLayerContribution> {
        self.contributions
            .iter()
            .find(|candidate| candidate.layer() == layer)
    }

    pub fn executive_decision(
        &self,
    ) -> Option<athlesia_executive_agency::IntegratedExecutiveControlDecision> {
        self.executive_decision
    }

    pub fn executive_selection(
        &self,
    ) -> Option<&athlesia_executive_agency::IntegratedExecutiveSelection> {
        self.executive_selection.as_ref()
    }

    pub fn skill_reused(&self) -> Option<bool> {
        self.skill_reused
    }

    pub fn experimentation_status(
        &self,
    ) -> Option<athlesia_autonomous_active_experimentation::IntegratedAutonomousExperimentationStatus>
    {
        self.experimentation_status
    }

    pub fn step(&self) -> Option<&ClosedLoopAgentStepResult> {
        self.step.as_ref()
    }

    pub fn next_anchor_state(&self) -> Option<&CognitiveStructure> {
        self.step.as_ref().and_then(|step| step.next_anchor_state())
    }

    pub fn advanced(&self) -> bool {
        self.status == OnlineCognitiveOrchestrationStatus::Advanced
    }

    pub fn preserved(&self) -> bool {
        self.status == OnlineCognitiveOrchestrationStatus::Preserved
    }

    pub fn rejected(&self) -> bool {
        !self.advanced() && !self.preserved()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OnlineCognitiveOrchestration;

impl OnlineCognitiveOrchestration {
    pub fn run(
        anchor_state: &CognitiveStructure,
        input: OnlineCognitiveOrchestrationInput<'_>,
        cycle_policy: IntegratedAgentPolicy,
        transition_request: &CognitiveCycleStateTransitionRequest,
    ) -> OnlineCognitiveOrchestrationResult {
        let OnlineCognitiveOrchestrationInput {
            perceptual,
            domain,
            executive,
            skill_memory,
            experimentation,
        } = input;

        for (layer, candidate_anchor) in [
            (
                IntegratedCognitiveLayer::PerceptualGrounding,
                perceptual.request.anchor_state(),
            ),
            (
                IntegratedCognitiveLayer::UniversalDomainLearning,
                domain.request.anchor_state(),
            ),
            (
                IntegratedCognitiveLayer::ExecutiveAgency,
                executive.request.anchor_state(),
            ),
            (
                IntegratedCognitiveLayer::MetaLearningSkillMemory,
                skill_memory.request.anchor_state(),
            ),
            (
                IntegratedCognitiveLayer::AutonomousExperimentation,
                experimentation.request.anchor_state(),
            ),
        ] {
            if candidate_anchor != anchor_state {
                return OnlineCognitiveOrchestrationResult::partial(
                    OnlineCognitiveOrchestrationStatus::RequestAnchorMismatch(layer),
                    Vec::new(),
                    None,
                    None,
                    None,
                );
            }
        }

        let mut contributions = Vec::with_capacity(IntegratedCognitiveCycle::PHASE_COUNT);

        let perceptual_result = AutonomousPerceptualGroundingIngestion::ingest(
            perceptual.request,
            perceptual.input,
            perceptual.context,
            perceptual.policy,
        );

        let Some(contribution) = perceptual_result.contribution().cloned() else {
            return OnlineCognitiveOrchestrationResult::partial(
                OnlineCognitiveOrchestrationStatus::PerceptualGroundingRejected,
                contributions,
                None,
                None,
                None,
            );
        };

        contributions.push(contribution);

        let domain_result = AutonomousUniversalDomainLearningIngestion::ingest(
            domain.request,
            domain.local,
            domain.transferred,
            domain.domain_policy,
            domain.ingestion_policy,
        );

        let Some(contribution) = domain_result.contribution().cloned() else {
            return OnlineCognitiveOrchestrationResult::partial(
                OnlineCognitiveOrchestrationStatus::UniversalDomainLearningRejected,
                contributions,
                None,
                None,
                None,
            );
        };

        contributions.push(contribution);

        let executive_result = athlesia_executive_agency::IntegratedExecutiveControl::evaluate(
            executive.context,
            executive.policy,
        );

        let executive_decision = executive_result.decision();

        let executive_selection = executive_result.selection().cloned();

        let executive_ingestion =
            AutonomousExecutiveAgencyIngestion::ingest(executive.request, &executive_result);

        let Some(contribution) = executive_ingestion.contribution().cloned() else {
            return OnlineCognitiveOrchestrationResult::partial(
                OnlineCognitiveOrchestrationStatus::ExecutiveAgencyRejected,
                contributions,
                Some(executive_decision),
                None,
                None,
            );
        };

        contributions.push(contribution);

        let skill_result = athlesia_meta_learning_skill_memory::IntegratedSkillLearningCycle::run(
            skill_memory.input,
            skill_memory.policy,
        );

        let skill_reused = skill_result.reused_skill();

        let skill_ingestion =
            AutonomousMetaLearningSkillMemoryIngestion::ingest(skill_memory.request, &skill_result);

        let Some(contribution) = skill_ingestion.contribution().cloned() else {
            return OnlineCognitiveOrchestrationResult::partial(
                OnlineCognitiveOrchestrationStatus::MetaLearningSkillMemoryRejected,
                contributions,
                Some(executive_decision),
                Some(skill_reused),
                None,
            );
        };

        contributions.push(contribution);

        let experimentation_result =
            athlesia_autonomous_active_experimentation::
                AutonomousIntegratedExperimentationCycle::run_cycle(
                    anchor_state,
                    experimentation.beliefs,
                    experimentation.possibilities,
                    experimentation.learning_samples,
                    experimentation.current_experiment_cycle,
                    experimentation.policy,
                );

        let experimentation_status = experimentation_result.status();

        let experimentation_ingestion = AutonomousExperimentationIngestion::ingest(
            experimentation.request,
            &experimentation_result,
        );

        let Some(contribution) = experimentation_ingestion.contribution().cloned() else {
            return OnlineCognitiveOrchestrationResult::partial(
                OnlineCognitiveOrchestrationStatus::AutonomousExperimentationRejected,
                contributions,
                Some(executive_decision),
                Some(skill_reused),
                Some(experimentation_status),
            );
        };

        contributions.push(contribution);

        let step = ClosedLoopAgentStep::run(
            anchor_state,
            &contributions,
            cycle_policy,
            transition_request,
        );

        let status = match step.status() {
            ClosedLoopAgentStepStatus::Advanced => OnlineCognitiveOrchestrationStatus::Advanced,
            ClosedLoopAgentStepStatus::Preserved => OnlineCognitiveOrchestrationStatus::Preserved,
            ClosedLoopAgentStepStatus::RejectedCycle => {
                OnlineCognitiveOrchestrationStatus::CognitiveCycleRejected
            }
            ClosedLoopAgentStepStatus::RejectedTransition => {
                OnlineCognitiveOrchestrationStatus::StateTransitionRejected
            }
        };

        OnlineCognitiveOrchestrationResult {
            status,
            contributions,
            executive_decision: Some(executive_decision),
            executive_selection,
            skill_reused: Some(skill_reused),
            experimentation_status: Some(experimentation_status),
            step: Some(step),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalOnlineCognitiveOrchestration;

impl UniversalOnlineCognitiveOrchestration {
    pub fn evaluate(
        anchor_state: &CognitiveStructure,
        input: OnlineCognitiveOrchestrationInput<'_>,
        cycle_policy: IntegratedAgentPolicy,
        transition_request: &CognitiveCycleStateTransitionRequest,
    ) -> OnlineCognitiveOrchestrationResult {
        OnlineCognitiveOrchestration::run(anchor_state, input, cycle_policy, transition_request)
    }
}

// ============================================================================
// M51 — ONLINE RECURRENT COGNITIVE CONTROL LOOP
// ============================================================================

pub struct OnlineRecurrentCognitiveStepInput<'a> {
    runtime: OnlineCognitiveOrchestrationInput<'a>,
    transition_request: &'a CognitiveCycleStateTransitionRequest,
    feedback: Option<RecurrentFeedbackEvidence>,
}

impl<'a> OnlineRecurrentCognitiveStepInput<'a> {
    pub fn new(
        runtime: OnlineCognitiveOrchestrationInput<'a>,
        transition_request: &'a CognitiveCycleStateTransitionRequest,
        feedback: Option<RecurrentFeedbackEvidence>,
    ) -> Self {
        Self {
            runtime,
            transition_request,
            feedback,
        }
    }

    pub fn transition_request(&self) -> &CognitiveCycleStateTransitionRequest {
        self.transition_request
    }

    pub fn feedback(&self) -> Option<&RecurrentFeedbackEvidence> {
        self.feedback.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnlineRecurrentCognitiveLoopStatus {
    Completed,
    StepFrontierExceeded,
    UnexpectedFirstStepFeedback,
    MissingPredecessorFeedback,
    FeedbackStepIndexMismatch,
    FeedbackAnchorMismatch,
    FeedbackAuthorityProvenanceMismatch,
    OnlineStepRejected,
}

#[derive(Debug, PartialEq)]
pub struct OnlineRecurrentCognitiveLoopResult {
    status: OnlineRecurrentCognitiveLoopStatus,
    initial_anchor_state: CognitiveStructure,
    final_anchor_state: CognitiveStructure,
    steps: Vec<OnlineCognitiveOrchestrationResult>,
    rejected_step_index: Option<usize>,
}

impl OnlineRecurrentCognitiveLoopResult {
    fn rejected_before_execution(
        status: OnlineRecurrentCognitiveLoopStatus,
        initial_anchor_state: &CognitiveStructure,
        current_anchor_state: &CognitiveStructure,
        steps: Vec<OnlineCognitiveOrchestrationResult>,
        rejected_step_index: usize,
    ) -> Self {
        Self {
            status,
            initial_anchor_state: initial_anchor_state.clone(),
            final_anchor_state: current_anchor_state.clone(),
            steps,
            rejected_step_index: Some(rejected_step_index),
        }
    }

    pub fn status(&self) -> OnlineRecurrentCognitiveLoopStatus {
        self.status
    }

    pub fn initial_anchor_state(&self) -> &CognitiveStructure {
        &self.initial_anchor_state
    }

    pub fn final_anchor_state(&self) -> &CognitiveStructure {
        &self.final_anchor_state
    }

    pub fn steps(&self) -> &[OnlineCognitiveOrchestrationResult] {
        &self.steps
    }

    pub fn executed_step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn completed_step_count(&self) -> usize {
        if self.status == OnlineRecurrentCognitiveLoopStatus::OnlineStepRejected {
            self.steps.len().saturating_sub(1)
        } else {
            self.steps.len()
        }
    }

    pub fn rejected_step_index(&self) -> Option<usize> {
        self.rejected_step_index
    }

    pub fn completed(&self) -> bool {
        self.status == OnlineRecurrentCognitiveLoopStatus::Completed
    }

    pub fn rejected(&self) -> bool {
        !self.completed()
    }
}

pub struct OnlineRecurrentCognitiveLoop;

impl OnlineRecurrentCognitiveLoop {
    pub fn run<'a>(
        initial_anchor_state: &CognitiveStructure,
        step_inputs: Vec<OnlineRecurrentCognitiveStepInput<'a>>,
        cycle_policy: IntegratedAgentPolicy,
        loop_policy: BoundedRecurrentAgentLoopPolicy,
    ) -> OnlineRecurrentCognitiveLoopResult {
        if step_inputs.len() > loop_policy.max_steps() {
            return OnlineRecurrentCognitiveLoopResult {
                status: OnlineRecurrentCognitiveLoopStatus::StepFrontierExceeded,
                initial_anchor_state: initial_anchor_state.clone(),
                final_anchor_state: initial_anchor_state.clone(),
                steps: Vec::new(),
                rejected_step_index: Some(loop_policy.max_steps()),
            };
        }

        let mut current_anchor_state = initial_anchor_state.clone();

        let mut completed_steps: Vec<OnlineCognitiveOrchestrationResult> =
            Vec::with_capacity(step_inputs.len());

        let mut predecessor_anchor: Option<CognitiveStructure> = None;

        let mut predecessor_authority_provenance: Option<CognitiveStructure> = None;

        for (step_index, step_input) in step_inputs.into_iter().enumerate() {
            if step_index == 0 {
                if step_input.feedback.is_some() {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::UnexpectedFirstStepFeedback,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                }
            } else {
                let Some(feedback) = step_input.feedback.as_ref() else {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::MissingPredecessorFeedback,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                };

                if feedback.predecessor_step_index() != step_index - 1 {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::FeedbackStepIndexMismatch,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                }

                if feedback.predecessor_next_anchor_state()
                    != predecessor_anchor
                        .as_ref()
                        .expect("accepted predecessor must retain anchor")
                {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::FeedbackAnchorMismatch,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                }

                if feedback.predecessor_authority_provenance()
                    != predecessor_authority_provenance.as_ref()
                {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::FeedbackAuthorityProvenanceMismatch,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                }
            }

            let authority_provenance = step_input
                .transition_request
                .expected_authority_provenance()
                .cloned();

            let result = OnlineCognitiveOrchestration::run(
                &current_anchor_state,
                step_input.runtime,
                cycle_policy,
                step_input.transition_request,
            );

            if result.rejected() {
                completed_steps.push(result);

                return OnlineRecurrentCognitiveLoopResult {
                    status: OnlineRecurrentCognitiveLoopStatus::OnlineStepRejected,
                    initial_anchor_state: initial_anchor_state.clone(),
                    final_anchor_state: current_anchor_state,
                    steps: completed_steps,
                    rejected_step_index: Some(step_index),
                };
            }

            let Some(next_anchor_state) = result.next_anchor_state().cloned() else {
                completed_steps.push(result);

                return OnlineRecurrentCognitiveLoopResult {
                    status: OnlineRecurrentCognitiveLoopStatus::OnlineStepRejected,
                    initial_anchor_state: initial_anchor_state.clone(),
                    final_anchor_state: current_anchor_state,
                    steps: completed_steps,
                    rejected_step_index: Some(step_index),
                };
            };

            predecessor_anchor = Some(next_anchor_state.clone());

            predecessor_authority_provenance = authority_provenance;

            current_anchor_state = next_anchor_state;

            completed_steps.push(result);
        }

        OnlineRecurrentCognitiveLoopResult {
            status: OnlineRecurrentCognitiveLoopStatus::Completed,
            initial_anchor_state: initial_anchor_state.clone(),
            final_anchor_state: current_anchor_state,
            steps: completed_steps,
            rejected_step_index: None,
        }
    }
}

pub struct UniversalOnlineRecurrentCognitiveLoop;

impl UniversalOnlineRecurrentCognitiveLoop {
    pub fn evaluate<'a>(
        initial_anchor_state: &CognitiveStructure,
        step_inputs: Vec<OnlineRecurrentCognitiveStepInput<'a>>,
        cycle_policy: IntegratedAgentPolicy,
        loop_policy: BoundedRecurrentAgentLoopPolicy,
    ) -> OnlineRecurrentCognitiveLoopResult {
        OnlineRecurrentCognitiveLoop::run(
            initial_anchor_state,
            step_inputs,
            cycle_policy,
            loop_policy,
        )
    }
}

// ============================================================================
// M51 — ENVIRONMENT INTERACTION BOUNDARY
// ============================================================================

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvironmentActionDispatchStatus {
    Ready,
    OnlineStepRejected,
    ExecutionNotAuthorized,
    MissingExecutiveSelection,
    SelectionDecisionMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentActionDispatch {
    source_anchor_state: CognitiveStructure,
    selection: athlesia_executive_agency::IntegratedExecutiveSelection,
}

impl EnvironmentActionDispatch {
    pub fn source_anchor_state(&self) -> &CognitiveStructure {
        &self.source_anchor_state
    }

    pub fn selection(&self) -> &athlesia_executive_agency::IntegratedExecutiveSelection {
        &self.selection
    }

    pub fn action(&self) -> &CognitiveStructure {
        self.selection.action()
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        self.selection.predicted_outcome()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentActionDispatchResult {
    status: EnvironmentActionDispatchStatus,
    dispatch: Option<EnvironmentActionDispatch>,
}

impl EnvironmentActionDispatchResult {
    fn rejection(status: EnvironmentActionDispatchStatus) -> Self {
        Self {
            status,
            dispatch: None,
        }
    }

    pub fn status(&self) -> EnvironmentActionDispatchStatus {
        self.status
    }

    pub fn dispatch(&self) -> Option<&EnvironmentActionDispatch> {
        self.dispatch.as_ref()
    }

    pub fn ready(&self) -> bool {
        self.status == EnvironmentActionDispatchStatus::Ready
    }

    pub fn rejected(&self) -> bool {
        !self.ready()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentInteractionObservation {
    event_index: u64,
    observed_outcome: CognitiveStructure,
    confidence: CognitiveSignal,
}

impl EnvironmentInteractionObservation {
    pub fn new(
        event_index: u64,
        observed_outcome: CognitiveStructure,
        confidence: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            event_index,
            observed_outcome,
            confidence,
        })
    }

    pub fn event_index(&self) -> u64 {
        self.event_index
    }

    pub fn observed_outcome(&self) -> &CognitiveStructure {
        &self.observed_outcome
    }

    pub fn confidence(&self) -> CognitiveSignal {
        self.confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentInteractionEvidence {
    action_observation: athlesia_core_knowledge_perceptual_grounding::ActionObservation,
    execution_observation: athlesia_executive_agency::GroundedExecutionObservation,
    experiment_observation:
        athlesia_autonomous_active_experimentation::ExperimentOutcomeObservation,
}

impl EnvironmentInteractionEvidence {
    pub fn action_observation(
        &self,
    ) -> &athlesia_core_knowledge_perceptual_grounding::ActionObservation {
        &self.action_observation
    }

    pub fn execution_observation(
        &self,
    ) -> &athlesia_executive_agency::GroundedExecutionObservation {
        &self.execution_observation
    }

    pub fn experiment_observation(
        &self,
    ) -> &athlesia_autonomous_active_experimentation::ExperimentOutcomeObservation {
        &self.experiment_observation
    }

    pub fn self_generated(
        source_state: &CognitiveStructure,
        action: &CognitiveStructure,
        observation: &EnvironmentInteractionObservation,
    ) -> Option<Self> {
        let action_observation =
            athlesia_core_knowledge_perceptual_grounding::ActionObservation::new(
                observation.event_index(),
                athlesia_core_knowledge_perceptual_grounding::ActionSource::SelfGenerated,
                action.clone(),
            );

        let execution_observation = athlesia_executive_agency::GroundedExecutionObservation::new(
            source_state.clone(),
            action.clone(),
            observation.observed_outcome().clone(),
            observation.confidence(),
        );

        let experiment_observation =
            athlesia_autonomous_active_experimentation::ExperimentOutcomeObservation::new(
                source_state.clone(),
                action.clone(),
                observation.observed_outcome().clone(),
                observation.confidence(),
            )?;

        Some(Self {
            action_observation,
            execution_observation,
            experiment_observation,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnvironmentInteractionBoundary;

impl EnvironmentInteractionBoundary {
    pub fn dispatch(
        result: &OnlineCognitiveOrchestrationResult,
    ) -> EnvironmentActionDispatchResult {
        if result.rejected() {
            return EnvironmentActionDispatchResult::rejection(
                EnvironmentActionDispatchStatus::OnlineStepRejected,
            );
        }

        let Some(decision) = result.executive_decision() else {
            return EnvironmentActionDispatchResult::rejection(
                EnvironmentActionDispatchStatus::ExecutionNotAuthorized,
            );
        };

        let executable = matches!(
            decision,
            athlesia_executive_agency::IntegratedExecutiveControlDecision::ExecuteCurrent
                | athlesia_executive_agency::IntegratedExecutiveControlDecision::ExecuteReplacement
                | athlesia_executive_agency::IntegratedExecutiveControlDecision::ExecuteExploration
        );

        if !executable {
            return EnvironmentActionDispatchResult::rejection(
                EnvironmentActionDispatchStatus::ExecutionNotAuthorized,
            );
        }

        let Some(selection) = result.executive_selection() else {
            return EnvironmentActionDispatchResult::rejection(
                EnvironmentActionDispatchStatus::MissingExecutiveSelection,
            );
        };

        let decision_matches_selection = matches!(
            (decision, selection.source()),
            (
                athlesia_executive_agency::IntegratedExecutiveControlDecision::ExecuteCurrent,
                athlesia_executive_agency::IntegratedExecutiveSelectionSource::CurrentIntention,
            ) | (
                athlesia_executive_agency::IntegratedExecutiveControlDecision::ExecuteReplacement,
                athlesia_executive_agency::IntegratedExecutiveSelectionSource::ReplacementIntention,
            ) | (
                athlesia_executive_agency::IntegratedExecutiveControlDecision::ExecuteExploration,
                athlesia_executive_agency::IntegratedExecutiveSelectionSource::Exploration,
            )
        );

        if !decision_matches_selection {
            return EnvironmentActionDispatchResult::rejection(
                EnvironmentActionDispatchStatus::SelectionDecisionMismatch,
            );
        }

        let Some(step) = result.step() else {
            return EnvironmentActionDispatchResult::rejection(
                EnvironmentActionDispatchStatus::OnlineStepRejected,
            );
        };

        EnvironmentActionDispatchResult {
            status: EnvironmentActionDispatchStatus::Ready,
            dispatch: Some(EnvironmentActionDispatch {
                source_anchor_state: step.previous_anchor_state().clone(),
                selection: selection.clone(),
            }),
        }
    }

    pub fn bind_observation(
        dispatch: &EnvironmentActionDispatch,
        observation: &EnvironmentInteractionObservation,
    ) -> Option<EnvironmentInteractionEvidence> {
        let action_observation =
            athlesia_core_knowledge_perceptual_grounding::ActionObservation::new(
                observation.event_index(),
                athlesia_core_knowledge_perceptual_grounding::ActionSource::SelfGenerated,
                dispatch.action().clone(),
            );

        let execution_observation = athlesia_executive_agency::GroundedExecutionObservation::new(
            dispatch.source_anchor_state().clone(),
            dispatch.action().clone(),
            observation.observed_outcome().clone(),
            observation.confidence(),
        );

        let experiment_observation =
            athlesia_autonomous_active_experimentation::ExperimentOutcomeObservation::new(
                dispatch.source_anchor_state().clone(),
                dispatch.action().clone(),
                observation.observed_outcome().clone(),
                observation.confidence(),
            )?;

        Some(EnvironmentInteractionEvidence {
            action_observation,
            execution_observation,
            experiment_observation,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalEnvironmentInteractionBoundary;

impl UniversalEnvironmentInteractionBoundary {
    pub fn dispatch(
        result: &OnlineCognitiveOrchestrationResult,
    ) -> EnvironmentActionDispatchResult {
        EnvironmentInteractionBoundary::dispatch(result)
    }

    pub fn bind_observation(
        dispatch: &EnvironmentActionDispatch,
        observation: &EnvironmentInteractionObservation,
    ) -> Option<EnvironmentInteractionEvidence> {
        EnvironmentInteractionBoundary::bind_observation(dispatch, observation)
    }
}

// === ATHLESIA DOMAIN-GENERAL AUTONOMOUS SELF-BOOTSTRAP ONLINE RUNTIME BEGIN ===

pub use athlesia_autonomous_cognitive_self_bootstrap as autonomous_cognitive_self_bootstrap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnlineAutonomousSelfBootstrapDigest {
    status: autonomous_cognitive_self_bootstrap::SelfBootstrapStatus,
    objective_kind: autonomous_cognitive_self_bootstrap::BootstrapObjectiveKind,
    source_state: athlesia_mindstone_sparse_cognition::CognitiveStructure,
    target_state: Option<athlesia_mindstone_sparse_cognition::CognitiveStructure>,
    selected_action: Option<athlesia_mindstone_sparse_cognition::CognitiveStructure>,
    predicted_outcome: Option<athlesia_mindstone_sparse_cognition::CognitiveStructure>,
    selected_confidence: Option<autonomous_cognitive_self_bootstrap::BootstrapSignal>,
    selected_information_gain: Option<autonomous_cognitive_self_bootstrap::BootstrapSignal>,
    selected_controllability: Option<autonomous_cognitive_self_bootstrap::BootstrapSignal>,
    selected_execution_cost: Option<autonomous_cognitive_self_bootstrap::BootstrapSignal>,
    candidate_frontier_len: usize,
    rejected_source_state_count: usize,
    rejected_unauthorized_action_count: usize,
    rejected_threshold_count: usize,
    duplicate_affordance_count: usize,
    duplicate_hypothesis_count: usize,
    frontier_truncated: bool,
}

impl OnlineAutonomousSelfBootstrapDigest {
    fn from_result(result: &autonomous_cognitive_self_bootstrap::SelfBootstrapResult) -> Self {
        let selected = result.selected();

        Self {
            status: result.status(),
            objective_kind: result.objective().kind(),
            source_state: result.objective().source_state().clone(),
            target_state: result.objective().target_state().cloned(),
            selected_action: selected.map(|candidate| candidate.hypothesis().action().clone()),
            predicted_outcome: selected
                .map(|candidate| candidate.hypothesis().predicted_outcome().clone()),
            selected_confidence: selected.map(|candidate| candidate.hypothesis().confidence()),
            selected_information_gain: selected
                .map(|candidate| candidate.hypothesis().information_gain()),
            selected_controllability: selected
                .map(|candidate| candidate.hypothesis().controllability()),
            selected_execution_cost: selected
                .map(|candidate| candidate.hypothesis().execution_cost()),
            candidate_frontier_len: result.candidate_frontier().len(),
            rejected_source_state_count: result.rejected_source_state_count(),
            rejected_unauthorized_action_count: result.rejected_unauthorized_action_count(),
            rejected_threshold_count: result.rejected_threshold_count(),
            duplicate_affordance_count: result.duplicate_affordance_count(),
            duplicate_hypothesis_count: result.duplicate_hypothesis_count(),
            frontier_truncated: result.frontier_truncated(),
        }
    }

    pub const fn status(&self) -> autonomous_cognitive_self_bootstrap::SelfBootstrapStatus {
        self.status
    }

    pub const fn objective_kind(
        &self,
    ) -> autonomous_cognitive_self_bootstrap::BootstrapObjectiveKind {
        self.objective_kind
    }

    pub fn source_state(&self) -> &athlesia_mindstone_sparse_cognition::CognitiveStructure {
        &self.source_state
    }

    pub fn target_state(&self) -> Option<&athlesia_mindstone_sparse_cognition::CognitiveStructure> {
        self.target_state.as_ref()
    }

    pub fn selected_action(
        &self,
    ) -> Option<&athlesia_mindstone_sparse_cognition::CognitiveStructure> {
        self.selected_action.as_ref()
    }

    pub fn predicted_outcome(
        &self,
    ) -> Option<&athlesia_mindstone_sparse_cognition::CognitiveStructure> {
        self.predicted_outcome.as_ref()
    }

    pub const fn selected_confidence(
        &self,
    ) -> Option<autonomous_cognitive_self_bootstrap::BootstrapSignal> {
        self.selected_confidence
    }

    pub const fn selected_information_gain(
        &self,
    ) -> Option<autonomous_cognitive_self_bootstrap::BootstrapSignal> {
        self.selected_information_gain
    }

    pub const fn selected_controllability(
        &self,
    ) -> Option<autonomous_cognitive_self_bootstrap::BootstrapSignal> {
        self.selected_controllability
    }

    pub const fn selected_execution_cost(
        &self,
    ) -> Option<autonomous_cognitive_self_bootstrap::BootstrapSignal> {
        self.selected_execution_cost
    }

    pub const fn candidate_frontier_len(&self) -> usize {
        self.candidate_frontier_len
    }

    pub const fn rejected_source_state_count(&self) -> usize {
        self.rejected_source_state_count
    }

    pub const fn rejected_unauthorized_action_count(&self) -> usize {
        self.rejected_unauthorized_action_count
    }

    pub const fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }

    pub const fn duplicate_affordance_count(&self) -> usize {
        self.duplicate_affordance_count
    }

    pub const fn duplicate_hypothesis_count(&self) -> usize {
        self.duplicate_hypothesis_count
    }

    pub const fn frontier_truncated(&self) -> bool {
        self.frontier_truncated
    }

    pub const fn has_selected_action(&self) -> bool {
        self.selected_action.is_some()
    }

    pub const fn requires_model_expansion(&self) -> bool {
        matches!(
            self.status,
            autonomous_cognitive_self_bootstrap::SelfBootstrapStatus::ModelExpansionRequired
        )
    }

    pub const fn is_complete(&self) -> bool {
        matches!(
            self.status,
            autonomous_cognitive_self_bootstrap::SelfBootstrapStatus::Complete
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OnlineAutonomousCognitiveSelfBootstrapRuntime;

impl OnlineAutonomousCognitiveSelfBootstrapRuntime {
    pub fn evaluate(
        input: &autonomous_cognitive_self_bootstrap::SelfBootstrapInput,
        policy: autonomous_cognitive_self_bootstrap::SelfBootstrapPolicy,
    ) -> Result<
        OnlineAutonomousSelfBootstrapDigest,
        autonomous_cognitive_self_bootstrap::SelfBootstrapError,
    > {
        let result =
            autonomous_cognitive_self_bootstrap::UniversalAutonomousCognitiveSelfBootstrap::
                evaluate(input, policy)?;

        Ok(OnlineAutonomousSelfBootstrapDigest::from_result(&result))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalOnlineAutonomousCognitiveSelfBootstrap;

impl UniversalOnlineAutonomousCognitiveSelfBootstrap {
    pub fn evaluate(
        input: &autonomous_cognitive_self_bootstrap::SelfBootstrapInput,
        policy: autonomous_cognitive_self_bootstrap::SelfBootstrapPolicy,
    ) -> Result<
        OnlineAutonomousSelfBootstrapDigest,
        autonomous_cognitive_self_bootstrap::SelfBootstrapError,
    > {
        OnlineAutonomousCognitiveSelfBootstrapRuntime::evaluate(input, policy)
    }
}

// === ATHLESIA DOMAIN-GENERAL AUTONOMOUS SELF-BOOTSTRAP ONLINE RUNTIME END ===

// === ATHLESIA DOMAIN-GENERAL AUTONOMOUS EXECUTIVE CONTEXT SYNTHESIS BEGIN ===

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnlineAutonomousColdStartExplorationError {
    BootstrapDecisionNotSelected,
    MissingBootstrapEvidence,
    ColdStartExplorationRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OnlineAutonomousColdStartExplorationPolicy {
    goal_priority: athlesia_mindstone_sparse_cognition::CognitiveSignal,
    cold_start: athlesia_executive_agency::ColdStartExplorationPolicy,
}

impl OnlineAutonomousColdStartExplorationPolicy {
    pub fn new(
        goal_priority: athlesia_mindstone_sparse_cognition::CognitiveSignal,
        cold_start: athlesia_executive_agency::ColdStartExplorationPolicy,
    ) -> Self {
        Self {
            goal_priority,
            cold_start,
        }
    }

    pub fn goal_priority(self) -> athlesia_mindstone_sparse_cognition::CognitiveSignal {
        self.goal_priority
    }

    pub fn cold_start(self) -> athlesia_executive_agency::ColdStartExplorationPolicy {
        self.cold_start
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnlineAutonomousColdStartExplorationBundle {
    goal: athlesia_executive_agency::ExecutiveGoal,
    candidate: athlesia_executive_agency::GroundedExplorationCandidate,
    result: athlesia_executive_agency::ColdStartExplorationResult,
}

impl OnlineAutonomousColdStartExplorationBundle {
    pub fn goal(&self) -> &athlesia_executive_agency::ExecutiveGoal {
        &self.goal
    }

    pub fn candidate(&self) -> &athlesia_executive_agency::GroundedExplorationCandidate {
        &self.candidate
    }

    pub fn result(&self) -> &athlesia_executive_agency::ColdStartExplorationResult {
        &self.result
    }

    pub fn selected_exploration(&self) -> &athlesia_executive_agency::RankedExplorationCandidate {
        self.result
            .selected_exploration()
            .expect("successful cold-start bundle retains its grounded selection")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OnlineAutonomousColdStartExplorationSynthesis;

impl OnlineAutonomousColdStartExplorationSynthesis {
    fn cognitive_signal(
        signal: autonomous_cognitive_self_bootstrap::BootstrapSignal,
    ) -> athlesia_mindstone_sparse_cognition::CognitiveSignal {
        athlesia_mindstone_sparse_cognition::CognitiveSignal::new(signal.value())
            .expect("bootstrap signal stays on the cognitive signal scale")
    }

    pub fn synthesize(
        digest: &OnlineAutonomousSelfBootstrapDigest,
        policy: OnlineAutonomousColdStartExplorationPolicy,
    ) -> Result<OnlineAutonomousColdStartExplorationBundle, OnlineAutonomousColdStartExplorationError>
    {
        if digest.status() != autonomous_cognitive_self_bootstrap::SelfBootstrapStatus::Selected {
            return Err(OnlineAutonomousColdStartExplorationError::BootstrapDecisionNotSelected);
        }

        let Some(action) = digest.selected_action().cloned() else {
            return Err(OnlineAutonomousColdStartExplorationError::MissingBootstrapEvidence);
        };

        let Some(predicted_outcome) = digest.predicted_outcome().cloned() else {
            return Err(OnlineAutonomousColdStartExplorationError::MissingBootstrapEvidence);
        };

        let Some(target_state) = digest.target_state().cloned() else {
            return Err(OnlineAutonomousColdStartExplorationError::MissingBootstrapEvidence);
        };

        let Some(confidence) = digest.selected_confidence() else {
            return Err(OnlineAutonomousColdStartExplorationError::MissingBootstrapEvidence);
        };

        let Some(information_gain) = digest.selected_information_gain() else {
            return Err(OnlineAutonomousColdStartExplorationError::MissingBootstrapEvidence);
        };

        let Some(controllability) = digest.selected_controllability() else {
            return Err(OnlineAutonomousColdStartExplorationError::MissingBootstrapEvidence);
        };

        let Some(execution_cost) = digest.selected_execution_cost() else {
            return Err(OnlineAutonomousColdStartExplorationError::MissingBootstrapEvidence);
        };

        let confidence = Self::cognitive_signal(confidence);

        let information_gain = Self::cognitive_signal(information_gain);

        let controllability = Self::cognitive_signal(controllability);

        let execution_cost = Self::cognitive_signal(execution_cost);

        let goal = athlesia_executive_agency::ExecutiveGoal::new(
            target_state.clone(),
            policy.goal_priority(),
            athlesia_mindstone_sparse_cognition::CognitiveSignal::zero(),
        );

        let exploration_signals = athlesia_executive_agency::ExplorationSignals::new(
            information_gain,
            athlesia_mindstone_sparse_cognition::CognitiveSignal::zero(),
            controllability,
            confidence,
            execution_cost,
        );

        let candidate = athlesia_executive_agency::GroundedExplorationCandidate::new(
            target_state,
            action,
            predicted_outcome,
            exploration_signals,
        );

        let result = athlesia_executive_agency::UniversalColdStartExplorationController::evaluate(
            &goal,
            std::slice::from_ref(&candidate),
            policy.cold_start(),
        );

        if !result.selected() {
            return Err(OnlineAutonomousColdStartExplorationError::ColdStartExplorationRejected);
        }

        Ok(OnlineAutonomousColdStartExplorationBundle {
            goal,
            candidate,
            result,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalOnlineAutonomousColdStartExplorationSynthesis;

impl UniversalOnlineAutonomousColdStartExplorationSynthesis {
    pub fn evaluate(
        digest: &OnlineAutonomousSelfBootstrapDigest,
        policy: OnlineAutonomousColdStartExplorationPolicy,
    ) -> Result<OnlineAutonomousColdStartExplorationBundle, OnlineAutonomousColdStartExplorationError>
    {
        OnlineAutonomousColdStartExplorationSynthesis::synthesize(digest, policy)
    }
}

// === ATHLESIA DOMAIN-GENERAL AUTONOMOUS EXECUTIVE CONTEXT SYNTHESIS END ===

// ============================================================================
// E5D — PERCEPTUAL DOMAIN-LEARNING EVIDENCE BRIDGE
// ============================================================================
//
// This bridge closes one previously missing endogenous path:
//
// selected grounded M46 perception
// -> exact M47 before/after fact snapshots
// -> exact self-generated action as transformation
// -> controlled interventional M47 evidence.
//
// The environment's observed outcome remains environment evidence. It is not
// inserted into either state snapshot unless perception independently grounds
// the same CognitiveStructure as a perceptual fact.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PerceptualDomainLearningEvidenceBridgeStatus {
    Bridged,
    PerceptualProjectionRejected,
    ActionSourceNotSelfGenerated,
    ActionOutsidePerceptualWindow,
    InconsistentEnvironmentEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerceptualDomainLearningEvidenceBridgeResult {
    status: PerceptualDomainLearningEvidenceBridgeStatus,
    projection_status:
        athlesia_core_knowledge_perceptual_grounding::GroundedPerceptualStateProjectionStatus,
    projection:
        Option<athlesia_core_knowledge_perceptual_grounding::GroundedPerceptualStateProjection>,
    controlled_evidence:
        Option<athlesia_universal_domain_learning::InterventionalTransformationEpisode>,
}

impl PerceptualDomainLearningEvidenceBridgeResult {
    fn rejected(
        status: PerceptualDomainLearningEvidenceBridgeStatus,
        projection_status:
            athlesia_core_knowledge_perceptual_grounding::GroundedPerceptualStateProjectionStatus,
        projection: Option<
            athlesia_core_knowledge_perceptual_grounding::GroundedPerceptualStateProjection,
        >,
    ) -> Self {
        Self {
            status,
            projection_status,
            projection,
            controlled_evidence: None,
        }
    }

    pub fn status(&self) -> PerceptualDomainLearningEvidenceBridgeStatus {
        self.status
    }

    pub fn projection_status(
        &self,
    ) -> athlesia_core_knowledge_perceptual_grounding::GroundedPerceptualStateProjectionStatus {
        self.projection_status
    }

    pub fn projection(
        &self,
    ) -> Option<&athlesia_core_knowledge_perceptual_grounding::GroundedPerceptualStateProjection>
    {
        self.projection.as_ref()
    }

    pub fn controlled_evidence(
        &self,
    ) -> Option<&athlesia_universal_domain_learning::InterventionalTransformationEpisode> {
        self.controlled_evidence.as_ref()
    }

    pub fn episode(
        &self,
    ) -> Option<&athlesia_universal_domain_learning::GroundedTransformationEpisode> {
        self.controlled_evidence
            .as_ref()
            .map(athlesia_universal_domain_learning::InterventionalTransformationEpisode::episode)
    }

    pub fn bridged(&self) -> bool {
        self.status == PerceptualDomainLearningEvidenceBridgeStatus::Bridged
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct PerceptualDomainLearningEvidenceBridge;

impl PerceptualDomainLearningEvidenceBridge {
    pub fn derive(
        input: &athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldInput,
        context: athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext,
        environment_evidence: &EnvironmentInteractionEvidence,
    ) -> PerceptualDomainLearningEvidenceBridgeResult {
        let projection_result =
            athlesia_core_knowledge_perceptual_grounding::
                UniversalGroundedPerceptualStateProjection::evaluate(
                    input,
                    context,
                );

        let projection_status = projection_result.status();

        let Some(projection) = projection_result.projection().cloned() else {
            return PerceptualDomainLearningEvidenceBridgeResult::rejected(
                PerceptualDomainLearningEvidenceBridgeStatus::PerceptualProjectionRejected,
                projection_status,
                None,
            );
        };

        let action_observation = environment_evidence.action_observation();

        if action_observation.source()
            != athlesia_core_knowledge_perceptual_grounding::ActionSource::SelfGenerated
        {
            return PerceptualDomainLearningEvidenceBridgeResult::rejected(
                PerceptualDomainLearningEvidenceBridgeStatus::ActionSourceNotSelfGenerated,
                projection_status,
                Some(projection),
            );
        }

        /*
         * Causal binding authority belongs to the environment/perception
         * boundary that selected this perceptual transition.
         *
         * action_observation.event_index() identifies the interaction event.
         * PerceptualFrame::observation_index() identifies a perceptual frame.
         *
         * They are intentionally independent clocks and therefore must never
         * be numerically compared here.
         */
        let execution = environment_evidence.execution_observation();
        let experiment = environment_evidence.experiment_observation();

        let evidence_consistent = execution.observed_action() == action_observation.descriptor()
            && experiment.action() == action_observation.descriptor()
            && execution.observed_state() == experiment.source_state()
            && execution.observed_outcome() == experiment.observed_outcome()
            && execution.observation_confidence() == experiment.confidence();

        if !evidence_consistent {
            return PerceptualDomainLearningEvidenceBridgeResult::rejected(
                PerceptualDomainLearningEvidenceBridgeStatus::InconsistentEnvironmentEvidence,
                projection_status,
                Some(projection),
            );
        }

        let before = athlesia_universal_domain_learning::GroundedStateSnapshot::new(
            projection.previous_facts().to_vec(),
        )
        .expect("projected previous perceptual state contains grounded facts");

        let after = athlesia_universal_domain_learning::GroundedStateSnapshot::new(
            projection.current_facts().to_vec(),
        )
        .expect("projected current perceptual state contains grounded facts");

        let episode = athlesia_universal_domain_learning::GroundedTransformationEpisode::new(
            before,
            after,
            action_observation.descriptor().clone(),
        );

        let controlled_evidence =
            athlesia_universal_domain_learning::InterventionalTransformationEpisode::controlled(
                episode,
            );

        PerceptualDomainLearningEvidenceBridgeResult {
            status: PerceptualDomainLearningEvidenceBridgeStatus::Bridged,
            projection_status,
            projection: Some(projection),
            controlled_evidence: Some(controlled_evidence),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalPerceptualDomainLearningEvidenceBridge;

impl UniversalPerceptualDomainLearningEvidenceBridge {
    pub fn evaluate(
        input: &athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldInput,
        context: athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext,
        environment_evidence: &EnvironmentInteractionEvidence,
    ) -> PerceptualDomainLearningEvidenceBridgeResult {
        PerceptualDomainLearningEvidenceBridge::derive(input, context, environment_evidence)
    }
}

#[cfg(test)]
mod perceptual_domain_learning_evidence_bridge_tests {
    use super::*;

    use athlesia_core_knowledge_perceptual_grounding::{
        ActionConsequencePolicy, ActionObservation, ActionSource,
        IntegratedPerceptualWorldCandidates, IntegratedPerceptualWorldContext,
        IntegratedPerceptualWorldInput, ObjectHypothesis, ObjecthoodEvidence,
        PerceptualChangePolicy, PerceptualElement, PerceptualElementHandle, PerceptualFrame,
        PerceptualGroundingPolicy, PersistenceTrackingPolicy, SceneInterpretation,
        TopologicalRelationPolicy,
    };

    fn s(value: u16) -> CognitiveSignal {
        CognitiveSignal::new(value).expect("test signal must be positive and bounded")
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn objecthood() -> ObjecthoodEvidence {
        ObjecthoodEvidence::new(s(900), s(900), s(900), s(900), s(900), s(900))
    }

    fn frame(observation_index: u64, elements: &[(u64, u64)]) -> PerceptualFrame {
        PerceptualFrame::new(
            observation_index,
            elements
                .iter()
                .map(|(handle, signature)| {
                    PerceptualElement::new(PerceptualElementHandle::new(*handle), a(*signature))
                })
                .collect(),
        )
        .expect("test frame is valid")
    }

    fn scene(handles: &[u64]) -> SceneInterpretation {
        SceneInterpretation::new(
            vec![
                ObjectHypothesis::new(
                    handles
                        .iter()
                        .copied()
                        .map(PerceptualElementHandle::new)
                        .collect(),
                    objecthood(),
                )
                .expect("test object is valid"),
            ],
            s(900),
        )
        .expect("test scene is valid")
    }

    fn perceptual_input() -> IntegratedPerceptualWorldInput {
        IntegratedPerceptualWorldInput::new(
            frame(1, &[(1001, 10), (1002, 20)]),
            frame(3, &[(1001, 10), (1002, 30)]),
            IntegratedPerceptualWorldCandidates::new(
                vec![scene(&[1001, 1002])],
                vec![scene(&[1001, 1002])],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        )
        .expect("test perceptual input is valid")
    }

    fn empty_perceptual_input() -> IntegratedPerceptualWorldInput {
        IntegratedPerceptualWorldInput::new(
            frame(1, &[(1001, 10)]),
            frame(3, &[(1001, 20)]),
            IntegratedPerceptualWorldCandidates::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        )
        .expect("test perceptual input is valid")
    }

    fn context() -> IntegratedPerceptualWorldContext {
        IntegratedPerceptualWorldContext::new(
            PerceptualGroundingPolicy::new(8, 8).expect("scene policy is valid"),
            PersistenceTrackingPolicy::new(8, 8, 16).expect("persistence policy is valid"),
            TopologicalRelationPolicy::new(8, 16).expect("topology policy is valid"),
            PerceptualChangePolicy::new(8, 16).expect("change policy is valid"),
            ActionConsequencePolicy::new(8, 8, 16).expect("action consequence policy is valid"),
        )
    }

    fn environment_evidence(
        event_index: u64,
        source: ActionSource,
        action: u64,
        execution_action: u64,
        outcome: u64,
    ) -> EnvironmentInteractionEvidence {
        EnvironmentInteractionEvidence {
            action_observation: ActionObservation::new(event_index, source, a(action)),
            execution_observation: athlesia_executive_agency::GroundedExecutionObservation::new(
                a(9000),
                a(execution_action),
                a(outcome),
                s(900),
            ),
            experiment_observation:
                athlesia_autonomous_active_experimentation::ExperimentOutcomeObservation::new(
                    a(9000),
                    a(action),
                    a(outcome),
                    s(900),
                )
                .expect("experiment observation is valid"),
        }
    }

    #[test]
    fn exact_grounded_perception_becomes_controlled_m47_learning_evidence() {
        let input = perceptual_input();

        let evidence = environment_evidence(2, ActionSource::SelfGenerated, 500, 500, 600);

        let result =
            UniversalPerceptualDomainLearningEvidenceBridge::evaluate(&input, context(), &evidence);

        assert_eq!(
            result.status(),
            PerceptualDomainLearningEvidenceBridgeStatus::Bridged
        );

        assert_eq!(
            result.projection_status(),
            athlesia_core_knowledge_perceptual_grounding::
                GroundedPerceptualStateProjectionStatus::Projected
        );

        let controlled = result
            .controlled_evidence()
            .expect("bridged result contains controlled evidence");

        assert!(controlled.is_controlled());

        let episode = controlled.episode();

        assert_eq!(episode.transformation(), &a(500));
        assert!(episode.before().contains_fact(&a(10)));
        assert!(episode.before().contains_fact(&a(20)));
        assert!(episode.after().contains_fact(&a(10)));
        assert!(episode.after().contains_fact(&a(30)));
    }

    #[test]
    fn environment_outcome_and_action_are_not_fabricated_into_state_facts() {
        let input = perceptual_input();

        let evidence = environment_evidence(2, ActionSource::SelfGenerated, 500, 500, 600);

        let result = PerceptualDomainLearningEvidenceBridge::derive(&input, context(), &evidence);

        let episode = result.episode().expect("bridged result contains episode");

        for structure in [a(500), a(600), a(9000)] {
            assert!(!episode.before().contains_fact(&structure));
            assert!(!episode.after().contains_fact(&structure));
        }

        assert_eq!(episode.transformation(), &a(500));
    }

    #[test]
    fn interaction_event_identity_is_independent_of_perceptual_frame_identity() {
        let input = perceptual_input();

        /*
         * Event identity belongs to the environment interaction clock.
         * It is deliberately far outside the perceptual frame indices used
         * by perceptual_input(), proving that no cross-clock comparison
         * participates in semantic validation.
         */
        let evidence = environment_evidence(90_000, ActionSource::SelfGenerated, 500, 500, 600);

        let result = PerceptualDomainLearningEvidenceBridge::derive(&input, context(), &evidence);

        assert_eq!(
            result.status(),
            PerceptualDomainLearningEvidenceBridgeStatus::Bridged
        );

        assert!(
            result.controlled_evidence().is_some(),
            "caller-authoritative causal transition must not be rejected by unrelated frame indices"
        );

        assert_eq!(
            result
                .controlled_evidence()
                .expect("bridged evidence exists")
                .episode()
                .transformation(),
            &a(500),
        );

        assert_eq!(
            evidence.action_observation().event_index(),
            90_000,
            "interaction identity remains exact provenance"
        );
    }

    #[test]
    fn externally_observed_action_is_not_promoted_to_controlled_intervention() {
        let input = perceptual_input();

        let evidence = environment_evidence(2, ActionSource::ObservedExternal, 500, 500, 600);

        let result = PerceptualDomainLearningEvidenceBridge::derive(&input, context(), &evidence);

        assert_eq!(
            result.status(),
            PerceptualDomainLearningEvidenceBridgeStatus::ActionSourceNotSelfGenerated
        );

        assert!(result.controlled_evidence().is_none());
    }

    #[test]
    fn inconsistent_cross_layer_environment_evidence_is_rejected() {
        let input = perceptual_input();

        let evidence = environment_evidence(2, ActionSource::SelfGenerated, 500, 501, 600);

        let result = PerceptualDomainLearningEvidenceBridge::derive(&input, context(), &evidence);

        assert_eq!(
            result.status(),
            PerceptualDomainLearningEvidenceBridgeStatus::InconsistentEnvironmentEvidence
        );

        assert!(result.controlled_evidence().is_none());
    }

    #[test]
    fn missing_grounded_scene_prevents_domain_learning_evidence_creation() {
        let input = empty_perceptual_input();

        let evidence = environment_evidence(2, ActionSource::SelfGenerated, 500, 500, 600);

        let result = PerceptualDomainLearningEvidenceBridge::derive(&input, context(), &evidence);

        assert_eq!(
            result.status(),
            PerceptualDomainLearningEvidenceBridgeStatus::PerceptualProjectionRejected
        );

        assert_eq!(
            result.projection_status(),
            athlesia_core_knowledge_perceptual_grounding::
                GroundedPerceptualStateProjectionStatus::MissingPreviousScene
        );

        assert!(result.projection().is_none());
        assert!(result.controlled_evidence().is_none());
    }

    #[test]
    fn bridge_is_deterministic_and_does_not_mutate_perception_or_evidence() {
        let input = perceptual_input();

        let evidence = environment_evidence(2, ActionSource::SelfGenerated, 500, 500, 600);

        let input_before = input.clone();
        let evidence_before = evidence.clone();

        let direct = PerceptualDomainLearningEvidenceBridge::derive(&input, context(), &evidence);

        let facade =
            UniversalPerceptualDomainLearningEvidenceBridge::evaluate(&input, context(), &evidence);

        let repeated =
            UniversalPerceptualDomainLearningEvidenceBridge::evaluate(&input, context(), &evidence);

        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(input, input_before);
        assert_eq!(evidence, evidence_before);
    }
}

// ============================================================================
// E5E — ENDOGENOUS TRANSITION SCHEMA LEARNING CLOSURE
// ============================================================================
//
// This layer accumulates only controlled transformation episodes produced by
// the E5D grounded perceptual evidence bridge.
//
// Hypothesis seeds are intentionally absent.
//
// Existing M47 TransitionSchemaInduction receives:
//   - endogenous grounded transformation episodes,
//   - an empty invariant seed frontier,
//   - a bounded domain-general induction policy.
//
// A single transformation is not sufficient evidence for a schema because
// M47 requires positive association lift relative to the global evidence
// baseline. Distinct experienced transformations provide the contrast needed
// for a transformation-specific predictive hypothesis.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EndogenousTransitionSchemaLearningPolicy {
    max_evidence_episodes: usize,
    schema_policy: athlesia_universal_domain_learning::TransitionSchemaPolicy,
}

impl EndogenousTransitionSchemaLearningPolicy {
    pub fn new(
        max_evidence_episodes: usize,
        schema_policy: athlesia_universal_domain_learning::TransitionSchemaPolicy,
    ) -> Option<Self> {
        if max_evidence_episodes == 0 {
            return None;
        }

        Some(Self {
            max_evidence_episodes,
            schema_policy,
        })
    }

    pub fn max_evidence_episodes(self) -> usize {
        self.max_evidence_episodes
    }

    pub fn schema_policy(self) -> athlesia_universal_domain_learning::TransitionSchemaPolicy {
        self.schema_policy
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EndogenousTransitionSchemaLearningState {
    episodes: Vec<athlesia_universal_domain_learning::GroundedTransformationEpisode>,
}

impl EndogenousTransitionSchemaLearningState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn episodes(&self) -> &[athlesia_universal_domain_learning::GroundedTransformationEpisode] {
        &self.episodes
    }

    pub fn episode_count(&self) -> usize {
        self.episodes.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EndogenousTransitionSchemaLearningStatus {
    EvidenceRejected,
    EvidenceFrontierExceeded,
    AccumulatingContrast,
    NoAdmittedHypothesis,
    HypothesisInduced,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndogenousTransitionSchemaLearningResult {
    status: EndogenousTransitionSchemaLearningStatus,
    bridge_status: PerceptualDomainLearningEvidenceBridgeStatus,
    state: EndogenousTransitionSchemaLearningState,
    induction: Option<athlesia_universal_domain_learning::TransitionSchemaInductionResult>,
}

impl EndogenousTransitionSchemaLearningResult {
    pub fn status(&self) -> EndogenousTransitionSchemaLearningStatus {
        self.status
    }

    pub fn bridge_status(&self) -> PerceptualDomainLearningEvidenceBridgeStatus {
        self.bridge_status
    }

    pub fn state(&self) -> &EndogenousTransitionSchemaLearningState {
        &self.state
    }

    pub fn induction(
        &self,
    ) -> Option<&athlesia_universal_domain_learning::TransitionSchemaInductionResult> {
        self.induction.as_ref()
    }

    pub fn selected_hypotheses(
        &self,
    ) -> &[athlesia_universal_domain_learning::GroundedTransitionSchemaHypothesis] {
        match &self.induction {
            Some(induction) => induction.selected(),
            None => &[],
        }
    }

    pub fn hypothesis_count(&self) -> usize {
        self.selected_hypotheses().len()
    }

    pub fn hypothesis_induced(&self) -> bool {
        self.status == EndogenousTransitionSchemaLearningStatus::HypothesisInduced
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EndogenousTransitionSchemaLearningCycle;

impl EndogenousTransitionSchemaLearningCycle {
    pub fn observe(
        state: &EndogenousTransitionSchemaLearningState,
        input: &athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldInput,
        context: athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext,
        environment_evidence: &EnvironmentInteractionEvidence,
        policy: EndogenousTransitionSchemaLearningPolicy,
    ) -> EndogenousTransitionSchemaLearningResult {
        let bridge =
            PerceptualDomainLearningEvidenceBridge::derive(input, context, environment_evidence);

        let bridge_status = bridge.status();

        let Some(controlled_evidence) = bridge.controlled_evidence() else {
            return EndogenousTransitionSchemaLearningResult {
                status: EndogenousTransitionSchemaLearningStatus::EvidenceRejected,
                bridge_status,
                state: state.clone(),
                induction: None,
            };
        };

        if state.episode_count() >= policy.max_evidence_episodes() {
            return EndogenousTransitionSchemaLearningResult {
                status: EndogenousTransitionSchemaLearningStatus::EvidenceFrontierExceeded,
                bridge_status,
                state: state.clone(),
                induction: None,
            };
        }

        let mut next_state = state.clone();

        next_state
            .episodes
            .push(controlled_evidence.episode().clone());

        let induction =
            athlesia_universal_domain_learning::UniversalTransitionSchemaInduction::evaluate(
                next_state.episodes(),
                &[],
                policy.schema_policy(),
            );

        let status = if induction.selected_count() > 0 {
            EndogenousTransitionSchemaLearningStatus::HypothesisInduced
        } else if induction.transformation_count() < 2 {
            EndogenousTransitionSchemaLearningStatus::AccumulatingContrast
        } else {
            EndogenousTransitionSchemaLearningStatus::NoAdmittedHypothesis
        };

        EndogenousTransitionSchemaLearningResult {
            status,
            bridge_status,
            state: next_state,
            induction: Some(induction),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalEndogenousTransitionSchemaLearningCycle;

impl UniversalEndogenousTransitionSchemaLearningCycle {
    pub fn evaluate(
        state: &EndogenousTransitionSchemaLearningState,
        input: &athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldInput,
        context: athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext,
        environment_evidence: &EnvironmentInteractionEvidence,
        policy: EndogenousTransitionSchemaLearningPolicy,
    ) -> EndogenousTransitionSchemaLearningResult {
        EndogenousTransitionSchemaLearningCycle::observe(
            state,
            input,
            context,
            environment_evidence,
            policy,
        )
    }
}

// -----------------------------------------------------------------------------
// Persistent online cognitive learning state
// -----------------------------------------------------------------------------
//
// This is the first retained state owner in the modern M51 online cognition
// path.
//
// The underlying learning semantics remain owned by M47 and by the existing
// EndogenousTransitionSchemaLearningCycle. M51 only owns lifecycle continuity:
// accepted environment evidence from one interaction must remain available to
// later inference instead of being discarded with the orchestration result.
//
// Additional cognitive state families can be integrated here only after their
// existing update/authority contracts are preserved and behaviorally verified.

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OnlinePersistentCognitiveState {
    transition_schema_learning: EndogenousTransitionSchemaLearningState,
    perceptual_temporal_evidence:
        athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidenceState,
    perceptual_grouping_behavior_evidence:
        athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingBehaviorEvidenceState,
    perceptual_grouping_appearance_evidence:
        athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingAppearanceEvidenceState,
}

impl OnlinePersistentCognitiveState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transition_schema_learning(&self) -> &EndogenousTransitionSchemaLearningState {
        &self.transition_schema_learning
    }

    pub fn transition_episode_count(&self) -> usize {
        self.transition_schema_learning.episode_count()
    }

    pub fn perceptual_temporal_evidence(
        &self,
    ) -> &athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidenceState
    {
        &self.perceptual_temporal_evidence
    }

    pub fn perceptual_temporal_record_count(&self) -> usize {
        self.perceptual_temporal_evidence.record_count()
    }

    pub fn retain_perceptual_observation_result(
        &mut self,
        result: &athlesia_core_knowledge_perceptual_grounding::PerceptualProposalObservationResult,
    ) {
        self.perceptual_temporal_evidence.observe(result);
    }

    pub fn perceptual_grouping_behavior_evidence(
        &self,
    ) -> &athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingBehaviorEvidenceState
    {
        &self.perceptual_grouping_behavior_evidence
    }

    pub fn perceptual_grouping_behavior_record_count(&self) -> usize {
        self.perceptual_grouping_behavior_evidence.record_count()
    }

    pub fn retain_perceptual_grouping_behavior_result(
        &mut self,
        result: &athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingBehaviorObservationResult,
    ) {
        self.perceptual_grouping_behavior_evidence.observe(result);
    }

    pub fn perceptual_grouping_appearance_evidence(
        &self,
    ) -> &athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingAppearanceEvidenceState
    {
        &self.perceptual_grouping_appearance_evidence
    }

    pub fn perceptual_grouping_appearance_record_count(&self) -> usize {
        self.perceptual_grouping_appearance_evidence.record_count()
    }

    pub fn retain_perceptual_grouping_appearance_result(
        &mut self,
        result: &athlesia_core_knowledge_perceptual_grounding::
            PerceptualGroupingAppearanceObservationResult,
    ) {
        self.perceptual_grouping_appearance_evidence.observe(result);
    }

    pub fn observe_environment_transition(
        &mut self,
        input: &athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldInput,
        context: athlesia_core_knowledge_perceptual_grounding::IntegratedPerceptualWorldContext,
        environment_evidence: &EnvironmentInteractionEvidence,
        policy: EndogenousTransitionSchemaLearningPolicy,
    ) -> EndogenousTransitionSchemaLearningResult {
        let result = EndogenousTransitionSchemaLearningCycle::observe(
            &self.transition_schema_learning,
            input,
            context,
            environment_evidence,
            policy,
        );

        self.transition_schema_learning = result.state().clone();

        result
    }
}

#[cfg(test)]
mod retained_grouping_behavior_owner_tests {
    use super::*;

    use athlesia_core_knowledge_perceptual_grounding::{
        PerceptualElement, PerceptualElementHandle, PerceptualFrame,
        PerceptualGroupingBehaviorObservation, PerceptualGroupingBehaviorRetentionPolicy,
        PerceptualGroupingBehaviorSupportStatus, PerceptualGroupingCandidate,
        PerceptualGroupingCandidateKind, PerceptualObjectProposal, PerceptualProposalObservation,
    };

    fn frame(observation_index: u64, values: &[(u64, u64)]) -> PerceptualFrame {
        PerceptualFrame::new(
            observation_index,
            values
                .iter()
                .map(|(handle, signature)| {
                    PerceptualElement::new(
                        PerceptualElementHandle::new(*handle),
                        CognitiveStructure::atom(*signature),
                    )
                })
                .collect(),
        )
        .expect("test frame is valid")
    }

    fn atomic(handle: u64) -> PerceptualObjectProposal {
        PerceptualObjectProposal::new(vec![PerceptualElementHandle::new(handle)])
            .expect("atomic proposal is valid")
    }

    fn grouping() -> PerceptualGroupingCandidate {
        PerceptualGroupingCandidate::new(
            vec![
                PerceptualElementHandle::new(1),
                PerceptualElementHandle::new(2),
            ],
            PerceptualGroupingCandidateKind::PairwiseRelation,
        )
        .expect("grouping candidate is valid")
    }

    fn result(
        previous: &PerceptualFrame,
        current: &PerceptualFrame,
        grouping: &PerceptualGroupingCandidate,
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingBehaviorObservationResult
    {
        let atomic_result =
            PerceptualProposalObservation::observe(previous, current, &[atomic(1), atomic(2)]);

        PerceptualGroupingBehaviorObservation::observe(&[grouping.clone()], &atomic_result)
    }

    #[test]
    fn persistent_cognitive_state_retains_grouping_behavior_across_later_observations() {
        let grouping = grouping();

        let policy = PerceptualGroupingBehaviorRetentionPolicy::new(2, 1).unwrap();

        let f1 = frame(1, &[(1, 10), (2, 20)]);
        let f2 = frame(2, &[(1, 11), (2, 21)]);
        let f3 = frame(3, &[(1, 12), (2, 22)]);

        let mut state = OnlinePersistentCognitiveState::new();

        assert_eq!(state.perceptual_grouping_behavior_record_count(), 0);

        state.retain_perceptual_grouping_behavior_result(&result(&f1, &f2, &grouping));

        assert_eq!(state.perceptual_grouping_behavior_record_count(), 1);

        assert_eq!(
            state
                .perceptual_grouping_behavior_evidence()
                .support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::InsufficientCommonChangeEvidence
        );

        state.retain_perceptual_grouping_behavior_result(&result(&f2, &f3, &grouping));

        assert_eq!(
            state
                .perceptual_grouping_behavior_evidence()
                .support_status(&grouping, policy),
            PerceptualGroupingBehaviorSupportStatus::Supported,
            "later observation must change grouping support through the same M51 persistent owner"
        );

        let record = state
            .perceptual_grouping_behavior_evidence()
            .record(&grouping)
            .expect("retained grouping record must exist");

        assert_eq!(record.observation_count(), 2);
        assert_eq!(record.uniform_changed_count(), 2);
        assert_eq!(record.mixed_count(), 0);
    }
}

#[cfg(test)]
mod endogenous_transition_schema_learning_tests {
    use super::*;

    use athlesia_core_knowledge_perceptual_grounding::{
        ActionConsequencePolicy, ActionObservation, ActionSource,
        IntegratedPerceptualWorldCandidates, IntegratedPerceptualWorldContext,
        IntegratedPerceptualWorldInput, ObjectHypothesis, ObjecthoodEvidence,
        PerceptualChangePolicy, PerceptualElement, PerceptualElementHandle, PerceptualFrame,
        PerceptualGroundingPolicy, PersistenceTrackingPolicy, SceneInterpretation,
        TopologicalRelationPolicy,
    };

    fn s(value: u16) -> CognitiveSignal {
        CognitiveSignal::new(value).expect("test signal must be positive and bounded")
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn objecthood() -> ObjecthoodEvidence {
        ObjecthoodEvidence::new(s(900), s(900), s(900), s(900), s(900), s(900))
    }

    fn frame(observation_index: u64, elements: &[(u64, u64)]) -> PerceptualFrame {
        PerceptualFrame::new(
            observation_index,
            elements
                .iter()
                .map(|(handle, signature)| {
                    PerceptualElement::new(PerceptualElementHandle::new(*handle), a(*signature))
                })
                .collect(),
        )
        .expect("test frame is valid")
    }

    fn scene(handles: &[u64]) -> SceneInterpretation {
        SceneInterpretation::new(
            vec![
                ObjectHypothesis::new(
                    handles
                        .iter()
                        .copied()
                        .map(PerceptualElementHandle::new)
                        .collect(),
                    objecthood(),
                )
                .expect("test object hypothesis is valid"),
            ],
            s(900),
        )
        .expect("test scene is valid")
    }

    fn perceptual_input(
        previous_index: u64,
        current_index: u64,
        effect_present_after: bool,
    ) -> IntegratedPerceptualWorldInput {
        let previous_frame = frame(previous_index, &[(1001, 10)]);

        let previous_scene = scene(&[1001]);

        let (current_frame, current_scene) = if effect_present_after {
            (
                frame(current_index, &[(1001, 10), (1002, 20)]),
                scene(&[1001, 1002]),
            )
        } else {
            (frame(current_index, &[(1001, 10)]), scene(&[1001]))
        };

        IntegratedPerceptualWorldInput::new(
            previous_frame,
            current_frame,
            IntegratedPerceptualWorldCandidates::new(
                vec![previous_scene],
                vec![current_scene],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        )
        .expect("test perceptual input is valid")
    }

    fn context() -> IntegratedPerceptualWorldContext {
        IntegratedPerceptualWorldContext::new(
            PerceptualGroundingPolicy::new(8, 8).expect("scene policy is valid"),
            PersistenceTrackingPolicy::new(8, 8, 16).expect("persistence policy is valid"),
            TopologicalRelationPolicy::new(8, 16).expect("topology policy is valid"),
            PerceptualChangePolicy::new(8, 16).expect("change policy is valid"),
            ActionConsequencePolicy::new(8, 8, 16).expect("action consequence policy is valid"),
        )
    }

    fn environment_evidence(
        event_index: u64,
        source: ActionSource,
        action: u64,
        outcome: u64,
    ) -> EnvironmentInteractionEvidence {
        EnvironmentInteractionEvidence {
            action_observation: ActionObservation::new(event_index, source, a(action)),
            execution_observation: athlesia_executive_agency::GroundedExecutionObservation::new(
                a(9000),
                a(action),
                a(outcome),
                s(900),
            ),
            experiment_observation:
                athlesia_autonomous_active_experimentation::ExperimentOutcomeObservation::new(
                    a(9000),
                    a(action),
                    a(outcome),
                    s(900),
                )
                .expect("experiment observation is valid"),
        }
    }

    fn policy(max_evidence_episodes: usize) -> EndogenousTransitionSchemaLearningPolicy {
        let schema_policy = athlesia_universal_domain_learning::TransitionSchemaPolicy::new(
            1,
            s(1000),
            s(1),
            256,
            64,
        )
        .expect("schema policy is valid");

        EndogenousTransitionSchemaLearningPolicy::new(max_evidence_episodes, schema_policy)
            .expect("learning policy is valid")
    }

    #[test]
    fn one_self_generated_transformation_does_not_self_confirm_a_schema() {
        let state = EndogenousTransitionSchemaLearningState::new();

        let input = perceptual_input(1, 3, true);

        let evidence = environment_evidence(2, ActionSource::SelfGenerated, 500, 600);

        let result = UniversalEndogenousTransitionSchemaLearningCycle::evaluate(
            &state,
            &input,
            context(),
            &evidence,
            policy(8),
        );

        assert_eq!(
            result.status(),
            EndogenousTransitionSchemaLearningStatus::AccumulatingContrast
        );

        assert_eq!(result.state().episode_count(), 1);
        assert_eq!(result.hypothesis_count(), 0);

        let induction = result
            .induction()
            .expect("accepted evidence produces an induction result");

        assert_eq!(induction.transformation_count(), 1);
        assert_eq!(induction.invariant_seeded_fact_count(), 0);
        assert_eq!(induction.selected_count(), 0);
    }

    #[test]
    fn contrasting_self_generated_actions_induce_a_predictive_schema_without_host_seeds() {
        let initial = EndogenousTransitionSchemaLearningState::new();

        let action_a_input = perceptual_input(1, 3, true);

        let action_a_evidence = environment_evidence(2, ActionSource::SelfGenerated, 500, 600);

        let first = EndogenousTransitionSchemaLearningCycle::observe(
            &initial,
            &action_a_input,
            context(),
            &action_a_evidence,
            policy(8),
        );

        assert_eq!(
            first.status(),
            EndogenousTransitionSchemaLearningStatus::AccumulatingContrast
        );

        let action_b_input = perceptual_input(4, 6, false);

        let action_b_evidence = environment_evidence(5, ActionSource::SelfGenerated, 501, 601);

        let second = UniversalEndogenousTransitionSchemaLearningCycle::evaluate(
            first.state(),
            &action_b_input,
            context(),
            &action_b_evidence,
            policy(8),
        );

        assert_eq!(
            second.status(),
            EndogenousTransitionSchemaLearningStatus::HypothesisInduced
        );

        assert_eq!(second.state().episode_count(), 2);

        let induction = second
            .induction()
            .expect("contrasting evidence produces induction");

        assert_eq!(induction.transformation_count(), 2);
        assert_eq!(induction.invariant_seeded_fact_count(), 0);

        let hypothesis = second
            .selected_hypotheses()
            .iter()
            .find(|hypothesis| {
                hypothesis.transformation() == &a(500)
                    && hypothesis.effect_kind()
                        == athlesia_universal_domain_learning::TransitionEffectKind::Added
                    && hypothesis.fact() == &a(20)
            })
            .expect("action A must predict the perceptually grounded added fact");

        assert_eq!(hypothesis.support_count(), 1);
        assert_eq!(hypothesis.transformation_opportunity_count(), 1);
        assert_eq!(hypothesis.counterexample_count(), 0);
        assert_eq!(hypothesis.global_support_count(), 1);
        assert_eq!(hypothesis.global_opportunity_count(), 2);
        assert_eq!(hypothesis.precision(), s(1000));
        assert_eq!(hypothesis.baseline_rate(), s(500));
        assert_eq!(hypothesis.association_lift(), s(500));
    }

    #[test]
    fn identical_effects_across_distinct_actions_do_not_create_spurious_specificity() {
        let initial = EndogenousTransitionSchemaLearningState::new();

        let first = EndogenousTransitionSchemaLearningCycle::observe(
            &initial,
            &perceptual_input(1, 3, true),
            context(),
            &environment_evidence(2, ActionSource::SelfGenerated, 500, 600),
            policy(8),
        );

        let second = EndogenousTransitionSchemaLearningCycle::observe(
            first.state(),
            &perceptual_input(4, 6, true),
            context(),
            &environment_evidence(5, ActionSource::SelfGenerated, 501, 601),
            policy(8),
        );

        assert_eq!(
            second.status(),
            EndogenousTransitionSchemaLearningStatus::NoAdmittedHypothesis
        );

        assert_eq!(second.state().episode_count(), 2);
        assert_eq!(second.hypothesis_count(), 0);

        let induction = second
            .induction()
            .expect("accepted evidence produces induction");

        assert_eq!(induction.transformation_count(), 2);
        assert_eq!(induction.invariant_seeded_fact_count(), 0);
    }

    #[test]
    fn bounded_evidence_frontier_refuses_hidden_unbounded_memory_growth() {
        let initial = EndogenousTransitionSchemaLearningState::new();

        let first = EndogenousTransitionSchemaLearningCycle::observe(
            &initial,
            &perceptual_input(1, 3, true),
            context(),
            &environment_evidence(2, ActionSource::SelfGenerated, 500, 600),
            policy(1),
        );

        assert_eq!(first.state().episode_count(), 1);

        let second = EndogenousTransitionSchemaLearningCycle::observe(
            first.state(),
            &perceptual_input(4, 6, false),
            context(),
            &environment_evidence(5, ActionSource::SelfGenerated, 501, 601),
            policy(1),
        );

        assert_eq!(
            second.status(),
            EndogenousTransitionSchemaLearningStatus::EvidenceFrontierExceeded
        );

        assert_eq!(second.state().episode_count(), 1);
        assert!(second.induction().is_none());
    }

    #[test]
    fn non_self_generated_evidence_is_not_admitted_to_endogenous_learning_memory() {
        let state = EndogenousTransitionSchemaLearningState::new();

        let result = EndogenousTransitionSchemaLearningCycle::observe(
            &state,
            &perceptual_input(1, 3, true),
            context(),
            &environment_evidence(2, ActionSource::ObservedExternal, 500, 600),
            policy(8),
        );

        assert_eq!(
            result.status(),
            EndogenousTransitionSchemaLearningStatus::EvidenceRejected
        );

        assert_eq!(
            result.bridge_status(),
            PerceptualDomainLearningEvidenceBridgeStatus::ActionSourceNotSelfGenerated
        );

        assert_eq!(result.state().episode_count(), 0);
        assert!(result.induction().is_none());
        assert_eq!(result.hypothesis_count(), 0);
    }

    #[test]
    fn persistent_cognitive_state_carries_environment_learning_into_later_inference() {
        let mut cognitive_state = OnlinePersistentCognitiveState::new();

        assert_eq!(cognitive_state.transition_episode_count(), 0);

        let action_a_input = perceptual_input(1, 3, true);
        let action_a_evidence = environment_evidence(2, ActionSource::SelfGenerated, 500, 600);

        let first = cognitive_state.observe_environment_transition(
            &action_a_input,
            context(),
            &action_a_evidence,
            policy(8),
        );

        assert_eq!(
            first.status(),
            EndogenousTransitionSchemaLearningStatus::AccumulatingContrast
        );
        assert_eq!(first.state().episode_count(), 1);
        assert_eq!(first.hypothesis_count(), 0);

        assert_eq!(
            cognitive_state.transition_episode_count(),
            1,
            "accepted environment experience must survive the first cognitive update"
        );
        assert_eq!(
            cognitive_state.transition_schema_learning(),
            first.state(),
            "M51 persistent state must retain the exact M47 learning state"
        );

        let action_b_input = perceptual_input(4, 6, false);
        let action_b_evidence = environment_evidence(5, ActionSource::SelfGenerated, 501, 601);

        let second = cognitive_state.observe_environment_transition(
            &action_b_input,
            context(),
            &action_b_evidence,
            policy(8),
        );

        assert_eq!(
            second.status(),
            EndogenousTransitionSchemaLearningStatus::HypothesisInduced,
            "later inference must consume evidence retained from the earlier interaction"
        );
        assert_eq!(second.state().episode_count(), 2);
        assert_eq!(cognitive_state.transition_episode_count(), 2);

        let learned = second
            .selected_hypotheses()
            .iter()
            .find(|hypothesis| {
                hypothesis.transformation() == &a(500)
                    && hypothesis.effect_kind()
                        == athlesia_universal_domain_learning::TransitionEffectKind::Added
                    && hypothesis.fact() == &a(20)
            })
            .expect(
                "retained first experience plus contrasting second experience \
                 must induce the previously established predictive schema",
            );

        assert_eq!(learned.support_count(), 1);
        assert_eq!(learned.transformation_opportunity_count(), 1);
        assert_eq!(learned.counterexample_count(), 0);
        assert_eq!(learned.global_support_count(), 1);
        assert_eq!(learned.global_opportunity_count(), 2);
        assert_eq!(learned.precision(), s(1000));
        assert_eq!(learned.baseline_rate(), s(500));
        assert_eq!(learned.association_lift(), s(500));

        assert_eq!(
            cognitive_state.transition_schema_learning(),
            second.state(),
            "the newly induced state must itself become the retained state for future cycles"
        );
    }
}

// ============================================================================
// ATHLESIA ONLINE GROUNDED EPISODIC TRANSFER BRIDGE
// ============================================================================

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnlineGroundedEpisodicTransferMemory {
    target_initial_state: CognitiveStructure,
    target_goal_identity: CognitiveStructure,
    source_episode: Option<athlesia_meta_learning_skill_memory::GroundedSkillEpisode>,
    observations: Vec<athlesia_meta_learning_skill_memory::SkillExecutionObservation>,
}

impl OnlineGroundedEpisodicTransferMemory {
    pub fn new(
        target_initial_state: CognitiveStructure,
        target_goal_identity: CognitiveStructure,
    ) -> Self {
        Self {
            target_initial_state,
            target_goal_identity,
            source_episode: None,
            observations: Vec::new(),
        }
    }

    pub fn target_initial_state(&self) -> &CognitiveStructure {
        &self.target_initial_state
    }

    pub fn target_goal_identity(&self) -> &CognitiveStructure {
        &self.target_goal_identity
    }

    pub fn source_episode(
        &self,
    ) -> Option<&athlesia_meta_learning_skill_memory::GroundedSkillEpisode> {
        self.source_episode.as_ref()
    }

    pub fn observations(
        &self,
    ) -> &[athlesia_meta_learning_skill_memory::SkillExecutionObservation] {
        &self.observations
    }

    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    pub fn remember_source_episode(
        &mut self,
        episode: athlesia_meta_learning_skill_memory::GroundedSkillEpisode,
    ) -> bool {
        if self.source_episode.is_some() {
            return false;
        }

        self.source_episode = Some(episode);
        true
    }

    pub fn record_environment_evidence(
        &mut self,
        evidence: &EnvironmentInteractionEvidence,
    ) -> bool {
        let execution = evidence.execution_observation();

        let Some(observation) = athlesia_meta_learning_skill_memory::SkillExecutionObservation::new(
            execution.observed_state().clone(),
            execution.observed_action().clone(),
            execution.observed_outcome().clone(),
            execution.observation_confidence(),
        ) else {
            return false;
        };

        self.observations.push(observation);
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OnlineGroundedEpisodicTransferPolicy {
    analogy: athlesia_meta_learning_skill_memory::GroundedEpisodicAnalogyPolicy,
    executive: athlesia_executive_agency::EpistemicExecutiveControlPolicy,
}

impl OnlineGroundedEpisodicTransferPolicy {
    pub fn new(
        analogy: athlesia_meta_learning_skill_memory::GroundedEpisodicAnalogyPolicy,
        executive: athlesia_executive_agency::EpistemicExecutiveControlPolicy,
    ) -> Self {
        Self { analogy, executive }
    }

    pub fn analogy(self) -> athlesia_meta_learning_skill_memory::GroundedEpisodicAnalogyPolicy {
        self.analogy
    }

    pub fn executive(self) -> athlesia_executive_agency::EpistemicExecutiveControlPolicy {
        self.executive
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnlineGroundedEpisodicTransferStatus {
    NoSourceEpisode,
    AnalogyAbstained,
    ExecutiveRejected,
    Authorized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnlineGroundedEpisodicTransferResult {
    status: OnlineGroundedEpisodicTransferStatus,
    analogy: Option<athlesia_meta_learning_skill_memory::GroundedEpisodicAnalogyResult>,
    authorization: Option<athlesia_executive_agency::EpistemicExecutiveAuthorizationResult>,
}

impl OnlineGroundedEpisodicTransferResult {
    pub fn status(&self) -> OnlineGroundedEpisodicTransferStatus {
        self.status
    }

    pub fn analogy(
        &self,
    ) -> Option<&athlesia_meta_learning_skill_memory::GroundedEpisodicAnalogyResult> {
        self.analogy.as_ref()
    }

    pub fn authorization(
        &self,
    ) -> Option<&athlesia_executive_agency::EpistemicExecutiveAuthorizationResult> {
        self.authorization.as_ref()
    }

    pub fn selection(&self) -> Option<&athlesia_executive_agency::EpistemicExecutiveSelection> {
        self.authorization
            .as_ref()
            .and_then(|authorization| authorization.selection())
    }

    pub fn authorized(&self) -> bool {
        self.status == OnlineGroundedEpisodicTransferStatus::Authorized
    }

    pub fn abstained(&self) -> bool {
        matches!(
            self.status,
            OnlineGroundedEpisodicTransferStatus::NoSourceEpisode
                | OnlineGroundedEpisodicTransferStatus::AnalogyAbstained
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OnlineGroundedEpisodicTransferRuntime;

impl OnlineGroundedEpisodicTransferRuntime {
    pub fn evaluate(
        memory: &OnlineGroundedEpisodicTransferMemory,
        current_state: &CognitiveStructure,
        policy: OnlineGroundedEpisodicTransferPolicy,
    ) -> OnlineGroundedEpisodicTransferResult {
        let Some(source_episode) = memory.source_episode() else {
            return OnlineGroundedEpisodicTransferResult {
                status: OnlineGroundedEpisodicTransferStatus::NoSourceEpisode,
                analogy: None,
                authorization: None,
            };
        };

        let analogy =
            athlesia_meta_learning_skill_memory::GroundedEpisodicAnalogyTransfer::infer_next(
                source_episode,
                memory.target_initial_state(),
                memory.target_goal_identity(),
                memory.observations(),
                policy.analogy(),
            );

        if analogy.candidate_count() != 1 {
            return OnlineGroundedEpisodicTransferResult {
                status: OnlineGroundedEpisodicTransferStatus::AnalogyAbstained,
                analogy: Some(analogy),
                authorization: None,
            };
        }

        let candidate = &analogy.candidates()[0];

        let step = athlesia_executive_agency::EpistemicExecutableIntentionStep::new(
            candidate.required_state().clone(),
            candidate.action().clone(),
            candidate.predicted_outcome().cloned(),
            candidate.evidence_confidence_floor(),
        )
        .expect("grounded analogy candidate retains positive evidence confidence");

        let authorization = athlesia_executive_agency::EpistemicExecutiveControl::authorize(
            memory.target_goal_identity(),
            current_state,
            step,
            policy.executive(),
        );

        let status = if authorization.authorized() {
            OnlineGroundedEpisodicTransferStatus::Authorized
        } else {
            OnlineGroundedEpisodicTransferStatus::ExecutiveRejected
        };

        OnlineGroundedEpisodicTransferResult {
            status,
            analogy: Some(analogy),
            authorization: Some(authorization),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalOnlineGroundedEpisodicTransferRuntime;

impl UniversalOnlineGroundedEpisodicTransferRuntime {
    pub fn evaluate(
        memory: &OnlineGroundedEpisodicTransferMemory,
        current_state: &CognitiveStructure,
        policy: OnlineGroundedEpisodicTransferPolicy,
    ) -> OnlineGroundedEpisodicTransferResult {
        OnlineGroundedEpisodicTransferRuntime::evaluate(memory, current_state, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EpistemicEnvironmentActionDispatchStatus {
    Ready,
    ExecutiveNotAuthorized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicEnvironmentActionDispatch {
    source_anchor_state: CognitiveStructure,
    selection: athlesia_executive_agency::EpistemicExecutiveSelection,
}

impl EpistemicEnvironmentActionDispatch {
    pub fn source_anchor_state(&self) -> &CognitiveStructure {
        &self.source_anchor_state
    }

    pub fn selection(&self) -> &athlesia_executive_agency::EpistemicExecutiveSelection {
        &self.selection
    }

    pub fn action(&self) -> &CognitiveStructure {
        self.selection.action()
    }

    pub fn predicted_outcome(&self) -> Option<&CognitiveStructure> {
        self.selection.predicted_outcome()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicEnvironmentActionDispatchResult {
    status: EpistemicEnvironmentActionDispatchStatus,
    dispatch: Option<EpistemicEnvironmentActionDispatch>,
}

impl EpistemicEnvironmentActionDispatchResult {
    pub fn status(&self) -> EpistemicEnvironmentActionDispatchStatus {
        self.status
    }

    pub fn dispatch(&self) -> Option<&EpistemicEnvironmentActionDispatch> {
        self.dispatch.as_ref()
    }

    pub fn ready(&self) -> bool {
        self.status == EpistemicEnvironmentActionDispatchStatus::Ready
    }
}

impl EnvironmentInteractionBoundary {
    pub fn dispatch_epistemic(
        source_anchor_state: &CognitiveStructure,
        authorization: &athlesia_executive_agency::EpistemicExecutiveAuthorizationResult,
    ) -> EpistemicEnvironmentActionDispatchResult {
        let Some(selection) = authorization.selection() else {
            return EpistemicEnvironmentActionDispatchResult {
                status: EpistemicEnvironmentActionDispatchStatus::ExecutiveNotAuthorized,
                dispatch: None,
            };
        };

        EpistemicEnvironmentActionDispatchResult {
            status: EpistemicEnvironmentActionDispatchStatus::Ready,
            dispatch: Some(EpistemicEnvironmentActionDispatch {
                source_anchor_state: source_anchor_state.clone(),
                selection: selection.clone(),
            }),
        }
    }

    pub fn bind_epistemic_observation(
        dispatch: &EpistemicEnvironmentActionDispatch,
        observation: &EnvironmentInteractionObservation,
    ) -> Option<EnvironmentInteractionEvidence> {
        let action_observation =
            athlesia_core_knowledge_perceptual_grounding::ActionObservation::new(
                observation.event_index(),
                athlesia_core_knowledge_perceptual_grounding::ActionSource::SelfGenerated,
                dispatch.action().clone(),
            );

        let execution_observation = athlesia_executive_agency::GroundedExecutionObservation::new(
            dispatch.source_anchor_state().clone(),
            dispatch.action().clone(),
            observation.observed_outcome().clone(),
            observation.confidence(),
        );

        let experiment_observation =
            athlesia_autonomous_active_experimentation::ExperimentOutcomeObservation::new(
                dispatch.source_anchor_state().clone(),
                dispatch.action().clone(),
                observation.observed_outcome().clone(),
                observation.confidence(),
            )?;

        Some(EnvironmentInteractionEvidence {
            action_observation,
            execution_observation,
            experiment_observation,
        })
    }
}

impl UniversalEnvironmentInteractionBoundary {
    pub fn dispatch_epistemic(
        source_anchor_state: &CognitiveStructure,
        authorization: &athlesia_executive_agency::EpistemicExecutiveAuthorizationResult,
    ) -> EpistemicEnvironmentActionDispatchResult {
        EnvironmentInteractionBoundary::dispatch_epistemic(source_anchor_state, authorization)
    }

    pub fn bind_epistemic_observation(
        dispatch: &EpistemicEnvironmentActionDispatch,
        observation: &EnvironmentInteractionObservation,
    ) -> Option<EnvironmentInteractionEvidence> {
        EnvironmentInteractionBoundary::bind_epistemic_observation(dispatch, observation)
    }
}

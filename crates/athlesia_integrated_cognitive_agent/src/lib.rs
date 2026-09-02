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

        assert!(integrated
            .frame()
            .unwrap()
            .contribution(IntegratedCognitiveLayer::PerceptualGrounding)
            .is_some());
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

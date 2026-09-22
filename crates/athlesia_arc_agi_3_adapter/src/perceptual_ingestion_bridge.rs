use crate::{
    cognitive_protocol_bridge::{ArcAgi3CognitiveCodecError, ArcAgi3CognitiveProtocolBridge},
    ArcAgi3Grid, ArcAgi3Observation,
};
use athlesia_core_knowledge_perceptual_grounding::{
    IntegratedPerceptualWorldCandidates, IntegratedPerceptualWorldContext,
    IntegratedPerceptualWorldInput, PerceptualElement, PerceptualElementHandle, PerceptualFrame,
};
use athlesia_integrated_cognitive_agent::{
    OnlinePerceptualGroundingRuntime, PerceptualGroundingIngestionPolicy,
    PerceptualGroundingIngestionRequest,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

const TAG_GRID_GEOMETRY: u64 = 0xA352_1001;
const TAG_GRID_CELL: u64 = 0xA352_1002;
const TAG_PERCEPTUAL_PROVENANCE: u64 = 0xA352_1010;

const GEOMETRY_HANDLE: u64 = 0;
const GRID_STRIDE: u64 = 64;
const FIRST_CELL_HANDLE: u64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3PerceptualElementSignature {
    Geometry { width: u8, height: u8 },
    Cell { x: u8, y: u8, value: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3PerceptualBridgeError {
    ObservationIndexOverflow,
    InvalidPerceptualFrame,
    InvalidIntegratedInput,
    InvalidElementSignature,
    InvalidIngestionRequest,
    IntegerOutOfRange,
    Codec(ArcAgi3CognitiveCodecError),
}

impl From<ArcAgi3CognitiveCodecError> for ArcAgi3PerceptualBridgeError {
    fn from(error: ArcAgi3CognitiveCodecError) -> Self {
        Self::Codec(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3PerceptualProjection {
    frames: Vec<PerceptualFrame>,
    transitions: Vec<IntegratedPerceptualWorldInput>,
    causal_environment_transition: Option<IntegratedPerceptualWorldInput>,
    next_observation_index: u64,
}

impl ArcAgi3PerceptualProjection {
    pub fn frames(&self) -> &[PerceptualFrame] {
        &self.frames
    }

    pub fn transitions(&self) -> &[IntegratedPerceptualWorldInput] {
        &self.transitions
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    pub fn causal_environment_transition(&self) -> Option<&IntegratedPerceptualWorldInput> {
        self.causal_environment_transition.as_ref()
    }

    pub fn latest_frame(&self) -> &PerceptualFrame {
        &self.frames[self.frames.len() - 1]
    }

    pub fn next_observation_index(&self) -> u64 {
        self.next_observation_index
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3PerceptualIngestionBridge;

impl ArcAgi3PerceptualIngestionBridge {
    fn atom(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn ordered(fields: Vec<CognitiveStructure>) -> CognitiveStructure {
        CognitiveStructure::Ordered(fields)
    }

    fn geometry_signature(width: u8, height: u8) -> CognitiveStructure {
        Self::ordered(vec![
            Self::atom(TAG_GRID_GEOMETRY),
            Self::atom(u64::from(width)),
            Self::atom(u64::from(height)),
        ])
    }

    fn cell_signature(x: u8, y: u8, value: u8) -> CognitiveStructure {
        Self::ordered(vec![
            Self::atom(TAG_GRID_CELL),
            Self::atom(u64::from(x)),
            Self::atom(u64::from(y)),
            Self::atom(u64::from(value)),
        ])
    }

    fn atom_value(structure: &CognitiveStructure) -> Result<u64, ArcAgi3PerceptualBridgeError> {
        match structure {
            CognitiveStructure::Atom(value) => Ok(*value),
            _ => Err(ArcAgi3PerceptualBridgeError::InvalidElementSignature),
        }
    }

    fn as_u8(value: u64) -> Result<u8, ArcAgi3PerceptualBridgeError> {
        u8::try_from(value).map_err(|_| ArcAgi3PerceptualBridgeError::IntegerOutOfRange)
    }

    pub fn geometry_handle() -> PerceptualElementHandle {
        PerceptualElementHandle::new(GEOMETRY_HANDLE)
    }

    pub fn cell_handle(x: u8, y: u8) -> PerceptualElementHandle {
        let handle = FIRST_CELL_HANDLE + u64::from(y) * GRID_STRIDE + u64::from(x);

        PerceptualElementHandle::new(handle)
    }

    pub fn decode_handle_coordinate(handle: PerceptualElementHandle) -> Option<(u8, u8)> {
        let value = handle.value();

        if value < FIRST_CELL_HANDLE {
            return None;
        }

        let offset = value - FIRST_CELL_HANDLE;

        if offset >= GRID_STRIDE * GRID_STRIDE {
            return None;
        }

        let x = u8::try_from(offset % GRID_STRIDE).ok()?;
        let y = u8::try_from(offset / GRID_STRIDE).ok()?;

        Some((x, y))
    }

    pub fn decode_element_signature(
        signature: &CognitiveStructure,
    ) -> Result<ArcAgi3PerceptualElementSignature, ArcAgi3PerceptualBridgeError> {
        let fields = match signature {
            CognitiveStructure::Ordered(fields) => fields,
            _ => {
                return Err(ArcAgi3PerceptualBridgeError::InvalidElementSignature);
            }
        };

        let tag = fields
            .first()
            .ok_or(ArcAgi3PerceptualBridgeError::InvalidElementSignature)
            .and_then(Self::atom_value)?;

        match tag {
            TAG_GRID_GEOMETRY => {
                if fields.len() != 3 {
                    return Err(ArcAgi3PerceptualBridgeError::InvalidElementSignature);
                }

                let width = Self::as_u8(Self::atom_value(&fields[1])?)?;

                let height = Self::as_u8(Self::atom_value(&fields[2])?)?;

                Ok(ArcAgi3PerceptualElementSignature::Geometry { width, height })
            }

            TAG_GRID_CELL => {
                if fields.len() != 4 {
                    return Err(ArcAgi3PerceptualBridgeError::InvalidElementSignature);
                }

                let x = Self::as_u8(Self::atom_value(&fields[1])?)?;

                let y = Self::as_u8(Self::atom_value(&fields[2])?)?;

                let value = Self::as_u8(Self::atom_value(&fields[3])?)?;

                Ok(ArcAgi3PerceptualElementSignature::Cell { x, y, value })
            }

            _ => Err(ArcAgi3PerceptualBridgeError::InvalidElementSignature),
        }
    }

    pub fn project_grid(
        grid: &ArcAgi3Grid,
        observation_index: u64,
    ) -> Result<PerceptualFrame, ArcAgi3PerceptualBridgeError> {
        let width = u8::try_from(grid.width())
            .map_err(|_| ArcAgi3PerceptualBridgeError::IntegerOutOfRange)?;

        let height = u8::try_from(grid.height())
            .map_err(|_| ArcAgi3PerceptualBridgeError::IntegerOutOfRange)?;

        let mut elements = Vec::with_capacity(grid.cells().len() + 1);

        elements.push(PerceptualElement::new(
            Self::geometry_handle(),
            Self::geometry_signature(width, height),
        ));

        for y in 0..height {
            for x in 0..width {
                let value = grid
                    .cell(usize::from(x), usize::from(y))
                    .ok_or(ArcAgi3PerceptualBridgeError::InvalidPerceptualFrame)?;

                elements.push(PerceptualElement::new(
                    Self::cell_handle(x, y),
                    Self::cell_signature(x, y, value),
                ));
            }
        }

        PerceptualFrame::new(observation_index, elements)
            .ok_or(ArcAgi3PerceptualBridgeError::InvalidPerceptualFrame)
    }

    pub fn atomic_object_proposals(
        frame: &PerceptualFrame,
        max_proposals: usize,
    ) -> Option<athlesia_core_knowledge_perceptual_grounding::AtomicPerceptualProposalResult> {
        let policy =
            athlesia_core_knowledge_perceptual_grounding::AtomicPerceptualProposalPolicy::new(
                max_proposals,
            )?;

        Some(
            athlesia_core_knowledge_perceptual_grounding::AtomicPerceptualProposalGeneration::generate(
                frame,
                &[Self::geometry_handle()],
                policy,
            ),
        )
    }

    pub fn atomic_transition_evidence(
        previous_frame: &PerceptualFrame,
        current_frame: &PerceptualFrame,
        max_proposals_per_frame: usize,
    ) -> Option<athlesia_core_knowledge_perceptual_grounding::PerceptualProposalObservationResult>
    {
        let previous = Self::atomic_object_proposals(previous_frame, max_proposals_per_frame)?;

        let current = Self::atomic_object_proposals(current_frame, max_proposals_per_frame)?;

        let mut proposals = previous.proposals().to_vec();

        for proposal in current.proposals() {
            if proposals.binary_search(proposal).is_err() {
                proposals.push(proposal.clone());
                proposals.sort();
            }
        }

        Some(
            athlesia_core_knowledge_perceptual_grounding::PerceptualProposalObservation::observe(
                previous_frame,
                current_frame,
                &proposals,
            ),
        )
    }

    pub fn accumulate_atomic_transition_evidence(
        state: &mut athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidenceState,
        previous_frame: &PerceptualFrame,
        current_frame: &PerceptualFrame,
        max_proposals_per_frame: usize,
    ) -> Option<athlesia_core_knowledge_perceptual_grounding::PerceptualProposalObservationResult>
    {
        let result = Self::atomic_transition_evidence(
            previous_frame,
            current_frame,
            max_proposals_per_frame,
        )?;

        state.observe(&result);

        Some(result)
    }

    pub fn temporally_supported_grid_grouping_candidates(
        temporal_state: &athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidenceState,
        frame: &PerceptualFrame,
        temporal_policy: athlesia_core_knowledge_perceptual_grounding::PerceptualProposalTemporalEvidencePolicy,
        grouping_policy: athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingGenerationPolicy,
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingGenerationResult {
        let mut supported_cells = temporal_state
            .supported_records(temporal_policy)
            .into_iter()
            .filter_map(|record| {
                if record.proposal().member_count() != 1 {
                    return None;
                }

                let handle = record.proposal().members()[0];

                if handle == Self::geometry_handle() || !frame.contains_handle(handle) {
                    return None;
                }

                let (x, y) = Self::decode_handle_coordinate(handle)?;

                Some((handle, x, y))
            })
            .collect::<Vec<_>>();

        supported_cells.sort_by_key(|(handle, _, _)| *handle);
        supported_cells.dedup_by_key(|(handle, _, _)| *handle);

        let supported_by_coordinate = supported_cells
            .iter()
            .map(|(handle, x, y)| ((*x, *y), *handle))
            .collect::<std::collections::BTreeMap<_, _>>();

        let relation_probe_limit = grouping_policy.max_relations().saturating_add(1);

        let mut relations = Vec::new();

        'coordinates: for (&(x, y), &left_handle) in &supported_by_coordinate {
            let neighbor_coordinates = [
                x.checked_add(1).map(|right_x| (right_x, y)),
                y.checked_add(1).map(|down_y| (x, down_y)),
            ];

            for coordinate in neighbor_coordinates.into_iter().flatten() {
                let Some(&right_handle) = supported_by_coordinate.get(&coordinate) else {
                    continue;
                };

                relations.push(
                    athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingRelation::new(
                        left_handle,
                        right_handle,
                    )
                    .expect("distinct orthogonal grid cells form a valid structural relation"),
                );

                if relations.len() >= relation_probe_limit {
                    break 'coordinates;
                }
            }
        }

        athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingFrontierGeneration::generate(
            frame,
            temporal_state,
            temporal_policy,
            &relations,
            grouping_policy,
        )
    }

    /*
     * Evidence-neutral appearance grouping.
     *
     * This discovers maximal 4-connected regions whose currently observed
     * cell appearance is identical.
     *
     * It does NOT claim objecthood, persistence, causality, value, agency,
     * reward or task semantics. It is merely an alternative perceptual
     * proposal family, analogous to generic visual connected-component
     * segmentation.
     *
     * Singletons remain available through atomic temporal evidence but are
     * not promoted into a grouping because PerceptualGroupingCandidate
     * deliberately requires at least two members.
     */
    pub fn appearance_coherent_grid_grouping_candidates(
        frame: &PerceptualFrame,
    ) -> Vec<athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate> {
        use athlesia_core_knowledge_perceptual_grounding::{
            PerceptualGroupingCandidate, PerceptualGroupingCandidateKind,
        };

        let mut cells =
            std::collections::BTreeMap::<(u8, u8), (PerceptualElementHandle, u8)>::new();

        for element in frame.elements() {
            let Ok(decoded) = Self::decode_element_signature(element.signature()) else {
                continue;
            };

            let ArcAgi3PerceptualElementSignature::Cell { x, y, value } = decoded else {
                continue;
            };

            if Self::decode_handle_coordinate(element.handle()) != Some((x, y)) {
                continue;
            }

            cells.insert((x, y), (element.handle(), value));
        }

        let mut remaining = cells
            .keys()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();

        let mut candidates = Vec::new();

        while let Some(seed) = remaining.iter().next().copied() {
            remaining.remove(&seed);

            let Some((_, seed_value)) = cells.get(&seed).copied() else {
                continue;
            };

            let mut frontier = vec![seed];
            let mut members = Vec::new();

            while let Some((x, y)) = frontier.pop() {
                let Some((handle, value)) = cells.get(&(x, y)).copied() else {
                    continue;
                };

                if value != seed_value {
                    continue;
                }

                members.push(handle);

                let neighbors = [
                    x.checked_sub(1).map(|nx| (nx, y)),
                    x.checked_add(1).map(|nx| (nx, y)),
                    y.checked_sub(1).map(|ny| (x, ny)),
                    y.checked_add(1).map(|ny| (x, ny)),
                ];

                for neighbor in neighbors.into_iter().flatten() {
                    let Some((_, neighbor_value)) = cells.get(&neighbor).copied() else {
                        continue;
                    };

                    if neighbor_value != seed_value {
                        continue;
                    }

                    if remaining.remove(&neighbor) {
                        frontier.push(neighbor);
                    }
                }
            }

            if let Some(candidate) = PerceptualGroupingCandidate::new(
                members,
                PerceptualGroupingCandidateKind::ConnectedComponent,
            ) {
                candidates.push(candidate);
            }
        }

        candidates.sort();
        candidates.dedup();

        candidates
    }

    pub fn grouping_visual_objecthood_evidence(
        frame: &PerceptualFrame,
        candidate: &athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate,
    ) -> Option<(bool, bool)> {
        if !candidate.is_grounded_in(frame) {
            return None;
        }

        let mut members = std::collections::BTreeMap::<(u8, u8), u8>::new();

        for &handle in candidate.members() {
            let element = frame.element(handle)?;

            let decoded = Self::decode_element_signature(element.signature()).ok()?;

            let ArcAgi3PerceptualElementSignature::Cell { x, y, value } = decoded else {
                return None;
            };

            if Self::decode_handle_coordinate(handle) != Some((x, y)) {
                return None;
            }

            members.insert((x, y), value);
        }

        if members.len() != candidate.member_count() {
            return None;
        }

        let shared_value = members.values().next().copied()?;

        let appearance_cohesion = members.values().all(|value| *value == shared_value);

        if !appearance_cohesion {
            return Some((false, false));
        }

        let mut exterior_neighbor_count = 0_usize;
        let mut every_exterior_neighbor_contrasts = true;

        for &(x, y) in members.keys() {
            let neighbor_coordinates = [
                x.checked_sub(1).map(|nx| (nx, y)),
                x.checked_add(1).map(|nx| (nx, y)),
                y.checked_sub(1).map(|ny| (x, ny)),
                y.checked_add(1).map(|ny| (x, ny)),
            ];

            for coordinate in neighbor_coordinates.into_iter().flatten() {
                if members.contains_key(&coordinate) {
                    continue;
                }

                let handle = Self::cell_handle(coordinate.0, coordinate.1);

                let Some(element) = frame.element(handle) else {
                    continue;
                };

                let decoded = Self::decode_element_signature(element.signature()).ok()?;

                let ArcAgi3PerceptualElementSignature::Cell { value, .. } = decoded else {
                    return None;
                };

                exterior_neighbor_count = exterior_neighbor_count.saturating_add(1);

                if value == shared_value {
                    every_exterior_neighbor_contrasts = false;
                }
            }
        }

        let contrast_boundary = exterior_neighbor_count > 0 && every_exterior_neighbor_contrasts;

        Some((appearance_cohesion, contrast_boundary))
    }

    pub fn grouping_appearance_observation(
        frame: &PerceptualFrame,
        candidates: &[athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingCandidate],
    ) -> athlesia_core_knowledge_perceptual_grounding::PerceptualGroupingAppearanceObservationResult
    {
        let evidence = candidates
            .iter()
            .filter_map(|candidate| {
                let (appearance_cohesion_supported, contrast_boundary_supported) =
                    Self::grouping_visual_objecthood_evidence(frame, candidate)?;

                Some(
                    athlesia_core_knowledge_perceptual_grounding::
                        PerceptualGroupingAppearanceObservationEvidence::new(
                            candidate.clone(),
                            appearance_cohesion_supported,
                            contrast_boundary_supported,
                        ),
                )
            })
            .collect();

        athlesia_core_knowledge_perceptual_grounding::
            PerceptualGroupingAppearanceObservationResult::new(
                evidence,
            )
    }

    pub fn empty_world_candidates() -> IntegratedPerceptualWorldCandidates {
        IntegratedPerceptualWorldCandidates::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    fn transition(
        previous: PerceptualFrame,
        current: PerceptualFrame,
    ) -> Result<IntegratedPerceptualWorldInput, ArcAgi3PerceptualBridgeError> {
        IntegratedPerceptualWorldInput::new(previous, current, Self::empty_world_candidates())
            .ok_or(ArcAgi3PerceptualBridgeError::InvalidIntegratedInput)
    }

    pub fn project_observation(
        observation: &ArcAgi3Observation,
        first_observation_index: u64,
        previous_frame: Option<&PerceptualFrame>,
    ) -> Result<ArcAgi3PerceptualProjection, ArcAgi3PerceptualBridgeError> {
        let frame_count = u64::try_from(observation.frames().frame_count())
            .map_err(|_| ArcAgi3PerceptualBridgeError::IntegerOutOfRange)?;

        let next_observation_index = first_observation_index
            .checked_add(frame_count)
            .ok_or(ArcAgi3PerceptualBridgeError::ObservationIndexOverflow)?;

        let mut frames = Vec::with_capacity(observation.frames().frame_count());

        for (offset, grid) in observation.frames().frames().iter().enumerate() {
            let offset = u64::try_from(offset)
                .map_err(|_| ArcAgi3PerceptualBridgeError::IntegerOutOfRange)?;

            let index = first_observation_index
                .checked_add(offset)
                .ok_or(ArcAgi3PerceptualBridgeError::ObservationIndexOverflow)?;

            frames.push(Self::project_grid(grid, index)?);
        }

        let transition_capacity =
            frames.len().saturating_sub(1) + usize::from(previous_frame.is_some());

        let mut transitions = Vec::with_capacity(transition_capacity);

        let causal_environment_transition = match previous_frame {
            Some(previous) => {
                let transition = Self::transition(previous.clone(), frames[0].clone())?;

                transitions.push(transition.clone());

                Some(transition)
            }
            None => None,
        };

        for pair in frames.windows(2) {
            transitions.push(Self::transition(pair[0].clone(), pair[1].clone())?);
        }

        Ok(ArcAgi3PerceptualProjection {
            frames,
            transitions,
            causal_environment_transition,
            next_observation_index,
        })
    }

    pub fn build_ingestion_request(
        anchor_state: CognitiveStructure,
        observation: &ArcAgi3Observation,
        confidence: CognitiveSignal,
        compute_cost: CognitiveSignal,
    ) -> Result<PerceptualGroundingIngestionRequest, ArcAgi3PerceptualBridgeError> {
        let grounded_state = ArcAgi3CognitiveProtocolBridge::encode_observation(observation);

        let provenance = Self::ordered(vec![
            Self::atom(TAG_PERCEPTUAL_PROVENANCE),
            grounded_state.clone(),
        ]);

        PerceptualGroundingIngestionRequest::new(
            anchor_state,
            grounded_state,
            provenance,
            confidence,
            compute_cost,
        )
        .ok_or(ArcAgi3PerceptualBridgeError::InvalidIngestionRequest)
    }

    pub fn online_runtime<'a>(
        request: &'a PerceptualGroundingIngestionRequest,
        input: &'a IntegratedPerceptualWorldInput,
        context: IntegratedPerceptualWorldContext,
        policy: PerceptualGroundingIngestionPolicy,
    ) -> OnlinePerceptualGroundingRuntime<'a> {
        OnlinePerceptualGroundingRuntime::new(request, input, context, policy)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3PerceptualIngestionBridge;

impl UniversalArcAgi3PerceptualIngestionBridge {
    pub fn project_observation(
        observation: &ArcAgi3Observation,
        first_observation_index: u64,
        previous_frame: Option<&PerceptualFrame>,
    ) -> Result<ArcAgi3PerceptualProjection, ArcAgi3PerceptualBridgeError> {
        ArcAgi3PerceptualIngestionBridge::project_observation(
            observation,
            first_observation_index,
            previous_frame,
        )
    }
}

#[cfg(test)]
mod appearance_coherent_grouping_tests {
    use super::*;

    #[test]
    fn maximal_same_appearance_components_are_global_not_prefix_local() {
        let grid = ArcAgi3Grid::from_rows(vec![
            vec![1, 1, 0, 2],
            vec![1, 0, 0, 2],
            vec![3, 3, 3, 2],
            vec![4, 0, 5, 5],
        ])
        .unwrap();

        let frame = ArcAgi3PerceptualIngestionBridge::project_grid(&grid, 1).unwrap();

        let candidates =
            ArcAgi3PerceptualIngestionBridge::appearance_coherent_grid_grouping_candidates(&frame);

        let member_sets = candidates
            .iter()
            .map(|candidate| {
                candidate
                    .members()
                    .iter()
                    .filter_map(|handle| {
                        ArcAgi3PerceptualIngestionBridge::decode_handle_coordinate(*handle)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        /*
         * Candidate members are canonically ordered by perceptual handle.
         * The assertion below is about SET membership, not handle encoding.
         * Normalize decoded coordinates before comparing expectations.
         */
        let member_sets = member_sets
            .into_iter()
            .map(|mut members| {
                members.sort_unstable();
                members
            })
            .collect::<Vec<_>>();

        assert!(member_sets.contains(&vec![(0, 0), (0, 1), (1, 0),],),);

        assert!(
            member_sets.contains(&vec![(3, 0), (3, 1), (3, 2),],),
            "appearance proposal generation must reach the far side of the frame",
        );

        assert!(member_sets.contains(&vec![(0, 2), (1, 2), (2, 2),],),);

        assert!(member_sets.contains(&vec![(2, 3), (3, 3),],),);

        /*
         * Isolated singleton appearances are not falsely upgraded into
         * grouping candidates.
         */
        assert!(member_sets.iter().all(|members| { members.len() >= 2 },),);
    }

    #[test]
    fn maximal_component_satisfies_exact_visual_boundary_while_internal_pair_does_not() {
        let grid = ArcAgi3Grid::from_rows(vec![
            vec![0, 0, 0, 0],
            vec![0, 7, 7, 0],
            vec![0, 7, 7, 0],
            vec![0, 0, 0, 0],
        ])
        .unwrap();

        let frame = ArcAgi3PerceptualIngestionBridge::project_grid(&grid, 1).unwrap();

        let components =
            ArcAgi3PerceptualIngestionBridge::appearance_coherent_grid_grouping_candidates(&frame);

        let object = components
            .iter()
            .find(|candidate| candidate.member_count() == 4)
            .expect("2x2 appearance component must be proposed");

        assert_eq!(
            ArcAgi3PerceptualIngestionBridge::grouping_visual_objecthood_evidence(&frame, object,),
            Some((true, true)),
        );

        let pair =
            athlesia_core_knowledge_perceptual_grounding::
                PerceptualGroupingCandidate::new(
                    vec![
                        ArcAgi3PerceptualIngestionBridge::
                            cell_handle(1, 1),
                        ArcAgi3PerceptualIngestionBridge::
                            cell_handle(2, 1),
                    ],
                    athlesia_core_knowledge_perceptual_grounding::
                        PerceptualGroupingCandidateKind::
                            PairwiseRelation,
                )
                .unwrap();

        assert_eq!(
            ArcAgi3PerceptualIngestionBridge::grouping_visual_objecthood_evidence(&frame, &pair,),
            Some((true, false)),
            "partial fragments must not fabricate a closed visual boundary",
        );
    }
}

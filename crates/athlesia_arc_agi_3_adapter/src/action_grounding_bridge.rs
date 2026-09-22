use crate::{
    cognitive_protocol_bridge::{ArcAgi3CognitiveCodecError, ArcAgi3CognitiveProtocolBridge},
    ArcAgi3Action, ArcAgi3ActionAuthorizationStatus, ArcAgi3ActionId, ArcAgi3Observation,
    ArcAgi3Protocol,
};
use athlesia_autonomous_active_experimentation::AutonomousExperimentProposal;
use athlesia_executive_agency::{ExecutiveGoal, GroundedExecutiveActionCandidate};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3ActionGroundingError {
    Codec(ArcAgi3CognitiveCodecError),
    ExecutiveResetForbidden,
    GameNotActive,
    ActionUnavailable,
    SourceStateMismatch,
}

impl From<ArcAgi3CognitiveCodecError> for ArcAgi3ActionGroundingError {
    fn from(error: ArcAgi3CognitiveCodecError) -> Self {
        Self::Codec(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3AuthorizedExecutiveCandidate {
    action: ArcAgi3Action,
    candidate: GroundedExecutiveActionCandidate,
}

impl ArcAgi3AuthorizedExecutiveCandidate {
    pub fn action(&self) -> ArcAgi3Action {
        self.action
    }

    pub fn candidate(&self) -> &GroundedExecutiveActionCandidate {
        &self.candidate
    }

    pub fn into_candidate(self) -> GroundedExecutiveActionCandidate {
        self.candidate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3AuthorizedExperimentProposal {
    action: ArcAgi3Action,
    proposal: AutonomousExperimentProposal,
}

impl ArcAgi3AuthorizedExperimentProposal {
    pub fn action(&self) -> ArcAgi3Action {
        self.action
    }

    pub fn proposal(&self) -> &AutonomousExperimentProposal {
        &self.proposal
    }

    pub fn into_proposal(self) -> AutonomousExperimentProposal {
        self.proposal
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3ActionGroundingBridge;

impl ArcAgi3ActionGroundingBridge {
    fn authorize_environment_action(
        observation: &ArcAgi3Observation,
        action_structure: &CognitiveStructure,
    ) -> Result<ArcAgi3Action, ArcAgi3ActionGroundingError> {
        let action = ArcAgi3CognitiveProtocolBridge::decode_action(action_structure)?;

        if action.id() == ArcAgi3ActionId::Reset {
            return Err(ArcAgi3ActionGroundingError::ExecutiveResetForbidden);
        }

        match ArcAgi3Protocol::authorize_action(observation, action).status() {
            ArcAgi3ActionAuthorizationStatus::AuthorizedAction => Ok(action),

            ArcAgi3ActionAuthorizationStatus::AuthorizedReset => {
                Err(ArcAgi3ActionGroundingError::ExecutiveResetForbidden)
            }

            ArcAgi3ActionAuthorizationStatus::GameNotActive => {
                Err(ArcAgi3ActionGroundingError::GameNotActive)
            }

            ArcAgi3ActionAuthorizationStatus::ActionUnavailable => {
                Err(ArcAgi3ActionGroundingError::ActionUnavailable)
            }
        }
    }

    pub fn authorize_executive_candidate(
        observation: &ArcAgi3Observation,
        candidate: &GroundedExecutiveActionCandidate,
    ) -> Result<ArcAgi3AuthorizedExecutiveCandidate, ArcAgi3ActionGroundingError> {
        let action = Self::authorize_environment_action(observation, candidate.action())?;

        Ok(ArcAgi3AuthorizedExecutiveCandidate {
            action,
            candidate: candidate.clone(),
        })
    }

    pub fn authorize_experiment_proposal(
        observation: &ArcAgi3Observation,
        expected_source_state: &CognitiveStructure,
        proposal: &AutonomousExperimentProposal,
    ) -> Result<ArcAgi3AuthorizedExperimentProposal, ArcAgi3ActionGroundingError> {
        /*
         * Source-state provenance is generic cognition.
         *
         * ARC maps the generic fail-closed provenance error onto its
         * existing public grounding error surface, then performs only
         * protocol-specific action authorization.
         */
        athlesia_integrated_cognitive_agent::
            OnlinePersistentCognitiveState::
                validate_experiment_proposal_source_state(
                    expected_source_state,
                    proposal,
                )
                .map_err(
                    |error| {
                        match error {
                            athlesia_integrated_cognitive_agent::
                                ExperimentProposalSourceBindingError::
                                    SourceStateMismatch =>
                            {
                                ArcAgi3ActionGroundingError::
                                    SourceStateMismatch
                            }
                        }
                    },
                )?;

        let action = Self::authorize_environment_action(observation, proposal.action())?;

        Ok(ArcAgi3AuthorizedExperimentProposal {
            action,
            proposal: proposal.clone(),
        })
    }

    pub fn ground_experiment_for_goal(
        observation: &ArcAgi3Observation,
        expected_source_state: &CognitiveStructure,
        goal: &ExecutiveGoal,
        goal_alignment: CognitiveSignal,
        proposal: &AutonomousExperimentProposal,
    ) -> Result<GroundedExecutiveActionCandidate, ArcAgi3ActionGroundingError> {
        /*
         * Generic source-state provenance is validated by M51 through
         * authorize_experiment_proposal().
         *
         * ARC-specific authority is restricted to:
         *
         * - cognitive-action decoding,
         * - RESET exclusion,
         * - current ARC action availability.
         *
         * M50 evidence interpretation and M48 candidate construction
         * also remain in the integrated cognitive layer.
         */
        let authorized =
            Self::authorize_experiment_proposal(observation, expected_source_state, proposal)?;

        Ok(
            athlesia_integrated_cognitive_agent::
                OnlinePersistentCognitiveState::
                    experiment_proposal_executive_candidate(
                        goal,
                        goal_alignment,
                        authorized.proposal(),
                    ),
        )
    }

    pub fn ground_belief_driven_proposal_frontier_for_goal(
        observation: &ArcAgi3Observation,
        expected_source_state: &CognitiveStructure,
        goal: &ExecutiveGoal,
        goal_alignment: CognitiveSignal,
        result: &athlesia_autonomous_active_experimentation::BeliefDrivenExperimentProposalResult,
    ) -> Result<Vec<GroundedExecutiveActionCandidate>, ArcAgi3ActionGroundingError> {
        result
            .generated()
            .iter()
            .map(|candidate| {
                Self::ground_experiment_for_goal(
                    observation,
                    expected_source_state,
                    goal,
                    goal_alignment,
                    candidate.experiment(),
                )
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3ActionGroundingBridge;

impl UniversalArcAgi3ActionGroundingBridge {
    pub fn authorize_executive_candidate(
        observation: &ArcAgi3Observation,
        candidate: &GroundedExecutiveActionCandidate,
    ) -> Result<ArcAgi3AuthorizedExecutiveCandidate, ArcAgi3ActionGroundingError> {
        ArcAgi3ActionGroundingBridge::authorize_executive_candidate(observation, candidate)
    }

    pub fn ground_experiment_for_goal(
        observation: &ArcAgi3Observation,
        expected_source_state: &CognitiveStructure,
        goal: &ExecutiveGoal,
        goal_alignment: CognitiveSignal,
        proposal: &AutonomousExperimentProposal,
    ) -> Result<GroundedExecutiveActionCandidate, ArcAgi3ActionGroundingError> {
        ArcAgi3ActionGroundingBridge::ground_experiment_for_goal(
            observation,
            expected_source_state,
            goal,
            goal_alignment,
            proposal,
        )
    }

    pub fn ground_belief_driven_proposal_frontier_for_goal(
        observation: &ArcAgi3Observation,
        expected_source_state: &CognitiveStructure,
        goal: &ExecutiveGoal,
        goal_alignment: CognitiveSignal,
        result: &athlesia_autonomous_active_experimentation::BeliefDrivenExperimentProposalResult,
    ) -> Result<Vec<GroundedExecutiveActionCandidate>, ArcAgi3ActionGroundingError> {
        ArcAgi3ActionGroundingBridge::ground_belief_driven_proposal_frontier_for_goal(
            observation,
            expected_source_state,
            goal,
            goal_alignment,
            result,
        )
    }
}

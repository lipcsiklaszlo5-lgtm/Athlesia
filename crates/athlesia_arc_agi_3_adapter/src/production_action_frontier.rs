// ============================================================================
// ATHLESIA B4A — PRODUCTION ARC-AGI-3 ACTION AFFORDANCE FRONTIER
// ============================================================================
//
// Protocol responsibility only.
//
// The observation already states which ACTION IDs are available.
//
// This module converts that protocol affordance statement into the complete
// concrete action frontier:
//
// ACTION1..ACTION5, ACTION7
//     -> exactly one discrete concrete action each.
//
// ACTION6
//     -> one concrete coordinate action for every visible cell in the current
//        frame.
//
// No cognitive value is assigned here.
//
// No action is ranked here.
//
// No prediction, confidence, utility, EIG, controllability or goal semantics
// are created here.
//
// Coordinate enumeration is exhaustive over the visible grid rather than
// heuristic sampling.

use crate::{ArcAgi3Action, ArcAgi3ActionId, ArcAgi3Observation, ARC_AGI_3_MAX_GRID_DIMENSION};

pub const ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER: usize =
    ARC_AGI_3_MAX_GRID_DIMENSION * ARC_AGI_3_MAX_GRID_DIMENSION + 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3ProductionActionFrontierError {
    UnexpectedResetAffordance,
    InvalidDiscreteAction,
    InvalidCoordinateAction,
    FrontierOverflow { actual: usize, maximum: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3ProductionActionFrontier {
    actions: Vec<ArcAgi3Action>,
    coordinate_candidate_count: usize,
}

impl ArcAgi3ProductionActionFrontier {
    pub fn from_observation(
        observation: &ArcAgi3Observation,
    ) -> Result<Self, ArcAgi3ProductionActionFrontierError> {
        /*
         * A terminal/non-active observation has no executable production
         * action frontier regardless of any stale protocol affordance data.
         */
        if !observation.awaiting_action() {
            return Ok(Self {
                actions: Vec::new(),
                coordinate_candidate_count: 0,
            });
        }

        let frame = observation.frames().latest();

        let mut actions = Vec::new();

        let mut coordinate_candidate_count = 0usize;

        for id in observation.available_actions().actions().iter().copied() {
            match id {
                ArcAgi3ActionId::Action6 => {
                    /*
                     * ACTION6 protocol semantics require a coordinate.
                     *
                     * Enumerate every currently visible cell. No subset is
                     * preferred and no object/goal assumption is introduced.
                     */
                    for y in 0..frame.height() {
                        for x in 0..frame.width() {
                            let x = u8::try_from(x).map_err(|_| {
                                ArcAgi3ProductionActionFrontierError::InvalidCoordinateAction
                            })?;

                            let y = u8::try_from(y).map_err(|_| {
                                ArcAgi3ProductionActionFrontierError::InvalidCoordinateAction
                            })?;

                            let action = ArcAgi3Action::coordinate(x, y).ok_or(
                                ArcAgi3ProductionActionFrontierError::InvalidCoordinateAction,
                            )?;

                            actions.push(action);

                            coordinate_candidate_count = coordinate_candidate_count
                                .checked_add(1)
                                .ok_or(ArcAgi3ProductionActionFrontierError::FrontierOverflow {
                                    actual: usize::MAX,
                                    maximum: ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
                                })?;
                        }
                    }
                }

                ArcAgi3ActionId::Reset => {
                    /*
                     * ArcAgi3AvailableActions already rejects RESET.
                     *
                     * Keep this defensive failure anyway so future protocol
                     * changes cannot silently turn reset into ordinary
                     * cognitive authority.
                     */
                    return Err(ArcAgi3ProductionActionFrontierError::UnexpectedResetAffordance);
                }

                ArcAgi3ActionId::Action1
                | ArcAgi3ActionId::Action2
                | ArcAgi3ActionId::Action3
                | ArcAgi3ActionId::Action4
                | ArcAgi3ActionId::Action5
                | ArcAgi3ActionId::Action7 => {
                    let action = ArcAgi3Action::discrete(id)
                        .ok_or(ArcAgi3ProductionActionFrontierError::InvalidDiscreteAction)?;

                    actions.push(action);
                }
            }

            if actions.len() > ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER {
                return Err(ArcAgi3ProductionActionFrontierError::FrontierOverflow {
                    actual: actions.len(),
                    maximum: ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
                });
            }
        }

        Ok(Self {
            actions,
            coordinate_candidate_count,
        })
    }

    pub fn actions(&self) -> &[ArcAgi3Action] {
        &self.actions
    }

    pub fn concrete_action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn coordinate_candidate_count(&self) -> usize {
        self.coordinate_candidate_count
    }

    pub fn contains_coordinate_actions(&self) -> bool {
        self.coordinate_candidate_count > 0
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3ProductionActionFrontier;

impl UniversalArcAgi3ProductionActionFrontier {
    pub fn evaluate(
        observation: &ArcAgi3Observation,
    ) -> Result<ArcAgi3ProductionActionFrontier, ArcAgi3ProductionActionFrontierError> {
        ArcAgi3ProductionActionFrontier::from_observation(observation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ArcAgi3ActionAuthorizationStatus, ArcAgi3AvailableActions, ArcAgi3FrameSequence,
        ArcAgi3GameId, ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Protocol,
    };

    fn grid(width: usize, height: usize) -> ArcAgi3Grid {
        ArcAgi3Grid::from_rows((0..height).map(|_| vec![0; width]).collect()).unwrap()
    }

    fn observation(width: usize, height: usize, ids: Vec<ArcAgi3ActionId>) -> ArcAgi3Observation {
        ArcAgi3Observation::new(
            ArcAgi3GameId::new("b4a-test".to_string()).unwrap(),
            ArcAgi3GameState::NotFinished,
            ArcAgi3FrameSequence::new(vec![grid(width, height)]).unwrap(),
            0,
            1,
            ArcAgi3AvailableActions::new(ids).unwrap(),
            None,
        )
    }

    #[test]
    fn b4a_discrete_protocol_actions_map_exactly_once() {
        let observation = observation(
            3,
            2,
            vec![
                ArcAgi3ActionId::Action1,
                ArcAgi3ActionId::Action3,
                ArcAgi3ActionId::Action5,
                ArcAgi3ActionId::Action7,
            ],
        );

        let frontier = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        assert_eq!(
            frontier.actions(),
            &[
                ArcAgi3Action::discrete(ArcAgi3ActionId::Action1,).unwrap(),
                ArcAgi3Action::discrete(ArcAgi3ActionId::Action3,).unwrap(),
                ArcAgi3Action::discrete(ArcAgi3ActionId::Action5,).unwrap(),
                ArcAgi3Action::discrete(ArcAgi3ActionId::Action7,).unwrap(),
            ],
        );

        assert_eq!(frontier.coordinate_candidate_count(), 0,);
    }

    #[test]
    fn b4a_action6_expands_to_every_visible_cell() {
        let observation = observation(3, 2, vec![ArcAgi3ActionId::Action6]);

        let frontier = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        assert_eq!(frontier.concrete_action_count(), 6,);

        assert_eq!(frontier.coordinate_candidate_count(), 6,);

        assert_eq!(
            frontier.actions(),
            &[
                ArcAgi3Action::coordinate(0, 0).unwrap(),
                ArcAgi3Action::coordinate(1, 0).unwrap(),
                ArcAgi3Action::coordinate(2, 0).unwrap(),
                ArcAgi3Action::coordinate(0, 1).unwrap(),
                ArcAgi3Action::coordinate(1, 1).unwrap(),
                ArcAgi3Action::coordinate(2, 1).unwrap(),
            ],
        );
    }

    #[test]
    fn b4a_mixed_frontier_preserves_protocol_order_and_coordinate_expansion() {
        let observation = observation(
            2,
            2,
            vec![
                ArcAgi3ActionId::Action7,
                ArcAgi3ActionId::Action6,
                ArcAgi3ActionId::Action1,
            ],
        );

        let frontier = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        assert_eq!(
            frontier.actions(),
            &[
                ArcAgi3Action::discrete(ArcAgi3ActionId::Action1,).unwrap(),
                ArcAgi3Action::coordinate(0, 0).unwrap(),
                ArcAgi3Action::coordinate(1, 0).unwrap(),
                ArcAgi3Action::coordinate(0, 1).unwrap(),
                ArcAgi3Action::coordinate(1, 1).unwrap(),
                ArcAgi3Action::discrete(ArcAgi3ActionId::Action7,).unwrap(),
            ],
        );
    }

    #[test]
    fn b4a_every_emitted_action_is_protocol_authorized() {
        let observation = observation(
            4,
            3,
            vec![
                ArcAgi3ActionId::Action1,
                ArcAgi3ActionId::Action2,
                ArcAgi3ActionId::Action6,
                ArcAgi3ActionId::Action7,
            ],
        );

        let frontier = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        assert!(!frontier.is_empty(),);

        for action in frontier.actions() {
            assert_eq!(
                ArcAgi3Protocol::authorize_action(&observation, *action,).status(),
                ArcAgi3ActionAuthorizationStatus::AuthorizedAction,
            );
        }
    }

    #[test]
    fn b4a_full_64_by_64_action6_frontier_is_complete_and_bounded() {
        let observation = observation(
            ARC_AGI_3_MAX_GRID_DIMENSION,
            ARC_AGI_3_MAX_GRID_DIMENSION,
            vec![ArcAgi3ActionId::Action6],
        );

        let frontier = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        assert_eq!(frontier.concrete_action_count(), 4096,);

        assert_eq!(frontier.coordinate_candidate_count(), 4096,);

        assert_eq!(
            frontier.actions().first(),
            Some(&ArcAgi3Action::coordinate(0, 0).unwrap(),),
        );

        assert_eq!(
            frontier.actions().last(),
            Some(&ArcAgi3Action::coordinate(63, 63).unwrap(),),
        );

        assert!(frontier.concrete_action_count() <= ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,);
    }

    #[test]
    fn b4a_all_action_ids_fit_maximum_frontier_bound() {
        let observation = observation(
            64,
            64,
            vec![
                ArcAgi3ActionId::Action1,
                ArcAgi3ActionId::Action2,
                ArcAgi3ActionId::Action3,
                ArcAgi3ActionId::Action4,
                ArcAgi3ActionId::Action5,
                ArcAgi3ActionId::Action6,
                ArcAgi3ActionId::Action7,
            ],
        );

        let frontier = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        assert_eq!(
            frontier.concrete_action_count(),
            ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
        );

        assert_eq!(frontier.coordinate_candidate_count(), 4096,);
    }

    #[test]
    fn b4a_terminal_observation_has_no_production_actions() {
        let mut observation = observation(
            2,
            2,
            vec![ArcAgi3ActionId::Action1, ArcAgi3ActionId::Action6],
        );

        observation = ArcAgi3Observation::new(
            observation.game_id().clone(),
            ArcAgi3GameState::Win,
            observation.frames().clone(),
            observation.levels_completed(),
            observation.win_levels(),
            observation.available_actions().clone(),
            observation.last_action(),
        );

        let frontier = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        assert!(frontier.is_empty(),);

        assert_eq!(frontier.coordinate_candidate_count(), 0,);
    }

    #[test]
    fn b4a_frontier_construction_is_deterministic_and_non_mutating() {
        let observation = observation(
            3,
            3,
            vec![
                ArcAgi3ActionId::Action7,
                ArcAgi3ActionId::Action6,
                ArcAgi3ActionId::Action2,
            ],
        );

        let snapshot = observation.clone();

        let first = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        let second = ArcAgi3ProductionActionFrontier::from_observation(&observation).unwrap();

        assert_eq!(first, second,);

        assert_eq!(observation, snapshot,);
    }
}

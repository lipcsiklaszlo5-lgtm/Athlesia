// ============================================================================
// ATHLESIA B4B1 — PRODUCTION SUCCESSOR RUNTIME
// ============================================================================
//
// Production responsibility:
//
// current ARC observation
//     -> B4A complete concrete protocol action frontier
//     -> evidence-faithful successor authority
//     -> existing live environment execution
//     -> real environment response
//     -> retained cognition
//
// The production caller does NOT provide:
//
// - candidate action lists;
// - native experiment possibilities;
// - hypothesis beliefs;
// - proposal policy;
// - action rankings;
// - ACTION6 coordinates.
//
// B4A is the sole owner of the concrete production action frontier.
//
// IMPORTANT RESOURCE-BOUND DISTINCTION:
//
// The complete protocol frontier can contain 4102 concrete actions
// (4096 ACTION6 cells + six discrete actions).
//
// Therefore only ACTION-FRONTIER traversal/selection bounds are widened to
// ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER.
//
// Hypothesis/version-space, forecast discrimination and retained evidence
// budgets remain exactly caller-supplied internal cognitive resource limits.

use crate::cognitive_interaction_runtime::ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest;
use crate::environment_transport_boundary::ArcAgi3EnvironmentTransport;
use crate::live_environment_runtime::{
    ArcAgi3LiveEnvironmentError, ArcAgi3LiveEnvironmentRuntime,
    ArcAgi3LiveEvidenceFaithfulSuccessorActionRequest, ArcAgi3LiveUnifiedStep,
};
use crate::production_action_frontier::{
    ArcAgi3ProductionActionFrontier, ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
};
use crate::ArcAgi3Action;
use athlesia_mindstone_sparse_cognition::CognitiveSignal;

#[derive(Clone, Copy, Debug)]
pub struct ArcAgi3ProductionSuccessorPolicy<'a> {
    goal: &'a athlesia_executive_agency::ExecutiveGoal,
    goal_alignment: CognitiveSignal,
    exploitation_execution_cost: CognitiveSignal,

    version_policy: athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy,

    discrimination_policy:
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy,

    expectation_policy:
        athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy,

    exploitation_policy: athlesia_executive_agency::ExecutiveAgencyPolicy,

    feedback_confidence: CognitiveSignal,
}

impl<'a> ArcAgi3ProductionSuccessorPolicy<'a> {
    pub fn new(
        goal: &'a athlesia_executive_agency::ExecutiveGoal,
        goal_alignment: CognitiveSignal,
        exploitation_execution_cost: CognitiveSignal,
        version_policy: athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy,
        discrimination_policy:
            athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy,
        expectation_policy:
            athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy,
        exploitation_policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
        feedback_confidence: CognitiveSignal,
    ) -> Option<Self> {
        /*
         * Preserve exploitation semantics and output limits.
         *
         * Only the two fields that bound traversal of the concrete action
         * frontier are widened for the complete B4A production frontier.
         */
        let exploitation_policy = athlesia_executive_agency::ExecutiveAgencyPolicy::new(
            exploitation_policy.max_goals(),
            ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
            ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
            exploitation_policy.max_selected_intents(),
            exploitation_policy.weights(),
            exploitation_policy.thresholds(),
        )?;

        Some(Self {
            goal,
            goal_alignment,
            exploitation_execution_cost,
            version_policy,
            discrimination_policy,
            expectation_policy,
            exploitation_policy,
            feedback_confidence,
        })
    }

    pub fn goal(self) -> &'a athlesia_executive_agency::ExecutiveGoal {
        self.goal
    }

    pub fn goal_alignment(self) -> CognitiveSignal {
        self.goal_alignment
    }

    pub fn exploitation_execution_cost(self) -> CognitiveSignal {
        self.exploitation_execution_cost
    }

    pub fn version_policy(
        self,
    ) -> athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy {
        self.version_policy
    }

    pub fn discrimination_policy(
        self,
    ) -> athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy {
        self.discrimination_policy
    }

    pub fn expectation_policy(
        self,
    ) -> athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy {
        self.expectation_policy
    }

    pub fn exploitation_policy(self) -> athlesia_executive_agency::ExecutiveAgencyPolicy {
        self.exploitation_policy
    }

    pub fn feedback_confidence(self) -> CognitiveSignal {
        self.feedback_confidence
    }

    pub fn priority_policy(
        self,
    ) -> athlesia_autonomous_active_experimentation::EmpiricalEpistemicActionPriorityPolicy {
        athlesia_autonomous_active_experimentation::EmpiricalEpistemicActionPriorityPolicy::new(
            ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
        )
        .expect("production action frontier bound is nonzero")
    }

    pub fn epistemic_policy(self) -> athlesia_executive_agency::EpistemicExecutiveSelectionPolicy {
        athlesia_executive_agency::EpistemicExecutiveSelectionPolicy::new(
            ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
        )
        .expect("production action frontier bound is nonzero")
    }

    pub fn ignorance_policy(
        self,
    ) -> athlesia_executive_agency::IgnoranceExplorationSelectionPolicy {
        athlesia_executive_agency::IgnoranceExplorationSelectionPolicy::new(
            ARC_AGI_3_MAX_PRODUCTION_ACTION_FRONTIER,
        )
        .expect("production action frontier bound is nonzero")
    }

    fn executive_request_for_frontier<'b>(
        self,
        candidate_actions: &'b [ArcAgi3Action],
    ) -> ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest<'b>
    where
        'a: 'b,
    {
        ArcAgi3EvidenceFaithfulSuccessorExecutiveRequest {
            candidate_actions,

            goal: self.goal,

            goal_alignment: self.goal_alignment,

            exploitation_execution_cost: self.exploitation_execution_cost,

            /*
             * Internal hypothesis/model/evidence budgets remain untouched.
             */
            version_policy: self.version_policy,

            discrimination_policy: self.discrimination_policy,

            expectation_policy: self.expectation_policy,

            /*
             * These three policies consume the concrete action frontier
             * directly and therefore use the full B4A bound.
             */
            priority_policy: self.priority_policy(),

            exploitation_policy: self.exploitation_policy,

            epistemic_policy: self.epistemic_policy(),

            ignorance_policy: self.ignorance_policy(),
        }
    }
}

impl<T> ArcAgi3LiveEnvironmentRuntime<T>
where
    T: ArcAgi3EnvironmentTransport,
{
    /// Production successor step.
    ///
    /// The current LIVE observation is read on every call.
    /// No candidate action list can be supplied by the caller.
    pub fn execute_production_evidence_faithful_successor(
        &mut self,
        policy: ArcAgi3ProductionSuccessorPolicy<'_>,
    ) -> Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError> {
        let frontier = ArcAgi3ProductionActionFrontier::from_observation(
            self.cognitive_runtime().observation(),
        )?;

        let request = ArcAgi3LiveEvidenceFaithfulSuccessorActionRequest::new(
            policy.executive_request_for_frontier(frontier.actions()),
            policy.feedback_confidence(),
        );

        self.execute_evidence_faithful_successor(request)
    }

    /// Passive-trace production successor step.
    ///
    /// Trace recording remains observational only; action authority is still
    /// produced by the exact evidence-faithful successor path.
    pub fn execute_production_evidence_faithful_successor_with_trace(
        &mut self,
        policy: ArcAgi3ProductionSuccessorPolicy<'_>,
        sink: &mut impl crate::cognitive_trace::ArcAgi3CognitiveTraceSink,
    ) -> Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError> {
        let frontier = ArcAgi3ProductionActionFrontier::from_observation(
            self.cognitive_runtime().observation(),
        )?;

        let request = ArcAgi3LiveEvidenceFaithfulSuccessorActionRequest::new(
            policy.executive_request_for_frontier(frontier.actions()),
            policy.feedback_confidence(),
        );

        self.execute_evidence_faithful_successor_with_trace(request, sink)
    }
}

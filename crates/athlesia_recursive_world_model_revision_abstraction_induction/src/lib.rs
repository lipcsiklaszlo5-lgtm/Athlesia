use std::collections::BTreeSet;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionInductionSide {
    Premise,
    Conclusion,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionSubstitutionWitness {
    side: RecursiveWorldRevisionAbstractionInductionSide,
    first_observation: RecursiveWorldRevisionDiscoveryObservation,
    second_observation: RecursiveWorldRevisionDiscoveryObservation,
    first_unit: RecursiveUnit,
    second_unit: RecursiveUnit,
    shared_units: Vec<RecursiveUnit>,
    fixed_opposite_units: Vec<RecursiveUnit>,
}

impl RecursiveWorldRevisionAbstractionSubstitutionWitness {
    pub fn discover(
        first_observation: RecursiveWorldRevisionDiscoveryObservation,
        second_observation: RecursiveWorldRevisionDiscoveryObservation,
    ) -> Option<Self> {
        if first_observation == second_observation {
            return None;
        }

        if let Some(witness) = Self::discover_on_side(
            RecursiveWorldRevisionAbstractionInductionSide::Premise,
            first_observation.clone(),
            second_observation.clone(),
        ) {
            return Some(witness);
        }

        Self::discover_on_side(
            RecursiveWorldRevisionAbstractionInductionSide::Conclusion,
            first_observation,
            second_observation,
        )
    }

    fn discover_on_side(
        side: RecursiveWorldRevisionAbstractionInductionSide,
        first_observation: RecursiveWorldRevisionDiscoveryObservation,
        second_observation: RecursiveWorldRevisionDiscoveryObservation,
    ) -> Option<Self> {
        let (first_variable, second_variable, first_fixed, second_fixed) = match side {
            RecursiveWorldRevisionAbstractionInductionSide::Premise => (
                first_observation.premises(),
                second_observation.premises(),
                first_observation.conclusions(),
                second_observation.conclusions(),
            ),
            RecursiveWorldRevisionAbstractionInductionSide::Conclusion => (
                first_observation.conclusions(),
                second_observation.conclusions(),
                first_observation.premises(),
                second_observation.premises(),
            ),
        };

        if first_fixed != second_fixed {
            return None;
        }

        let first_set = first_variable.iter().cloned().collect::<BTreeSet<_>>();

        let second_set = second_variable.iter().cloned().collect::<BTreeSet<_>>();

        let first_only = first_set
            .difference(&second_set)
            .cloned()
            .collect::<Vec<_>>();

        let second_only = second_set
            .difference(&first_set)
            .cloned()
            .collect::<Vec<_>>();

        if first_only.len() != 1 || second_only.len() != 1 {
            return None;
        }

        let shared_units = first_set
            .intersection(&second_set)
            .cloned()
            .collect::<Vec<_>>();

        let mut fixed_opposite_units = first_fixed.to_vec();
        fixed_opposite_units.sort();
        fixed_opposite_units.dedup();

        let mut ordered_observations = [first_observation, second_observation];
        ordered_observations.sort();

        let mut substituted_units = [first_only[0].clone(), second_only[0].clone()];
        substituted_units.sort();

        Some(Self {
            side,
            first_observation: ordered_observations[0].clone(),
            second_observation: ordered_observations[1].clone(),
            first_unit: substituted_units[0].clone(),
            second_unit: substituted_units[1].clone(),
            shared_units,
            fixed_opposite_units,
        })
    }

    pub fn side(&self) -> RecursiveWorldRevisionAbstractionInductionSide {
        self.side
    }

    pub fn first_observation(&self) -> &RecursiveWorldRevisionDiscoveryObservation {
        &self.first_observation
    }

    pub fn second_observation(&self) -> &RecursiveWorldRevisionDiscoveryObservation {
        &self.second_observation
    }

    pub fn first_unit(&self) -> &RecursiveUnit {
        &self.first_unit
    }

    pub fn second_unit(&self) -> &RecursiveUnit {
        &self.second_unit
    }

    pub fn shared_units(&self) -> &[RecursiveUnit] {
        &self.shared_units
    }

    pub fn fixed_opposite_units(&self) -> &[RecursiveUnit] {
        &self.fixed_opposite_units
    }

    pub fn abstraction_class(&self) -> RecursiveWorldRevisionAbstractionClass {
        RecursiveWorldRevisionAbstractionClass::new(vec![
            self.first_unit.clone(),
            self.second_unit.clone(),
        ])
        .expect("substitution witness always contains two distinct units")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionSubstitutionWitnessSet {
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
}

impl RecursiveWorldRevisionAbstractionSubstitutionWitnessSet {
    pub fn discover(observations: RecursiveWorldRevisionInductionObservationSet) -> Option<Self> {
        let source = observations.observations();
        let mut witnesses = Vec::new();

        for left_index in 0..source.len() {
            for right_index in (left_index + 1)..source.len() {
                if let Some(witness) =
                    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
                        source[left_index].clone(),
                        source[right_index].clone(),
                    )
                {
                    witnesses.push(witness);
                }
            }
        }

        witnesses.sort();
        witnesses.dedup();

        if witnesses.is_empty() {
            return None;
        }

        Some(Self { witnesses })
    }

    pub fn witnesses(&self) -> &[RecursiveWorldRevisionAbstractionSubstitutionWitness] {
        &self.witnesses
    }

    pub fn len(&self) -> usize {
        self.witnesses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.witnesses.is_empty()
    }

    pub fn premise_witnesses(&self) -> Vec<&RecursiveWorldRevisionAbstractionSubstitutionWitness> {
        self.witnesses
            .iter()
            .filter(|witness| {
                witness.side() == RecursiveWorldRevisionAbstractionInductionSide::Premise
            })
            .collect()
    }

    pub fn conclusion_witnesses(
        &self,
    ) -> Vec<&RecursiveWorldRevisionAbstractionSubstitutionWitness> {
        self.witnesses
            .iter()
            .filter(|witness| {
                witness.side() == RecursiveWorldRevisionAbstractionInductionSide::Conclusion
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionSubstitutionDiscoverer;

impl RecursiveWorldRevisionAbstractionSubstitutionDiscoverer {
    pub fn discover(
        observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Option<RecursiveWorldRevisionAbstractionSubstitutionWitnessSet> {
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::discover(observations)
    }
}

use std::collections::BTreeMap;

impl RecursiveWorldRevisionAbstractionSubstitutionWitnessSet {
    pub fn new(
        mut witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
    ) -> Option<Self> {
        witnesses.sort();
        witnesses.dedup();

        if witnesses.is_empty() {
            return None;
        }

        Some(Self { witnesses })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionAbstractionInductionContext {
    side: RecursiveWorldRevisionAbstractionInductionSide,
    shared_units: Vec<RecursiveUnit>,
    fixed_opposite_units: Vec<RecursiveUnit>,
}

impl RecursiveWorldRevisionAbstractionInductionContext {
    pub fn from_witness(witness: &RecursiveWorldRevisionAbstractionSubstitutionWitness) -> Self {
        Self {
            side: witness.side(),
            shared_units: witness.shared_units().to_vec(),
            fixed_opposite_units: witness.fixed_opposite_units().to_vec(),
        }
    }

    pub fn side(&self) -> RecursiveWorldRevisionAbstractionInductionSide {
        self.side
    }

    pub fn shared_units(&self) -> &[RecursiveUnit] {
        &self.shared_units
    }

    pub fn fixed_opposite_units(&self) -> &[RecursiveUnit] {
        &self.fixed_opposite_units
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionInducedClass {
    context: RecursiveWorldRevisionAbstractionInductionContext,
    abstraction_class: RecursiveWorldRevisionAbstractionClass,
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
}

impl RecursiveWorldRevisionAbstractionInducedClass {
    pub fn context(&self) -> &RecursiveWorldRevisionAbstractionInductionContext {
        &self.context
    }

    pub fn abstraction_class(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.abstraction_class
    }

    pub fn witnesses(&self) -> &[RecursiveWorldRevisionAbstractionSubstitutionWitness] {
        &self.witnesses
    }

    pub fn witness_count(&self) -> usize {
        self.witnesses.len()
    }

    pub fn side(&self) -> RecursiveWorldRevisionAbstractionInductionSide {
        self.context.side()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionInducedClassSet {
    classes: Vec<RecursiveWorldRevisionAbstractionInducedClass>,
}

impl RecursiveWorldRevisionAbstractionInducedClassSet {
    pub fn induce(
        witness_set: RecursiveWorldRevisionAbstractionSubstitutionWitnessSet,
    ) -> Option<Self> {
        let mut grouped = BTreeMap::<
            RecursiveWorldRevisionAbstractionInductionContext,
            Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
        >::new();

        for witness in witness_set.witnesses() {
            let context = RecursiveWorldRevisionAbstractionInductionContext::from_witness(witness);

            grouped.entry(context).or_default().push(witness.clone());
        }

        let mut classes = Vec::new();

        for (context, mut witnesses) in grouped {
            witnesses.sort();
            witnesses.dedup();

            let mut members = BTreeSet::<RecursiveUnit>::new();

            let mut witnessed_pairs = BTreeSet::<(RecursiveUnit, RecursiveUnit)>::new();

            for witness in &witnesses {
                members.insert(witness.first_unit().clone());

                members.insert(witness.second_unit().clone());

                let mut pair = [witness.first_unit().clone(), witness.second_unit().clone()];

                pair.sort();

                witnessed_pairs.insert((pair[0].clone(), pair[1].clone()));
            }

            if members.len() < 2 {
                continue;
            }

            let member_vec = members.iter().cloned().collect::<Vec<_>>();

            let mut complete = true;

            'pair_check: for left in 0..member_vec.len() {
                for right in (left + 1)..member_vec.len() {
                    if !witnessed_pairs
                        .contains(&(member_vec[left].clone(), member_vec[right].clone()))
                    {
                        complete = false;
                        break 'pair_check;
                    }
                }
            }

            if !complete {
                continue;
            }

            let abstraction_class = RecursiveWorldRevisionAbstractionClass::new(member_vec)
                .expect("complete substitution clique contains at least two distinct units");

            classes.push(RecursiveWorldRevisionAbstractionInducedClass {
                context,
                abstraction_class,
                witnesses,
            });
        }

        classes.sort_by(|left, right| {
            left.context
                .cmp(&right.context)
                .then_with(|| left.abstraction_class.cmp(&right.abstraction_class))
        });

        if classes.is_empty() {
            return None;
        }

        Some(Self { classes })
    }

    pub fn classes(&self) -> &[RecursiveWorldRevisionAbstractionInducedClass] {
        &self.classes
    }

    pub fn len(&self) -> usize {
        self.classes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    pub fn premise_classes(&self) -> Vec<&RecursiveWorldRevisionAbstractionInducedClass> {
        self.classes
            .iter()
            .filter(|induced| {
                induced.side() == RecursiveWorldRevisionAbstractionInductionSide::Premise
            })
            .collect()
    }

    pub fn conclusion_classes(&self) -> Vec<&RecursiveWorldRevisionAbstractionInducedClass> {
        self.classes
            .iter()
            .filter(|induced| {
                induced.side() == RecursiveWorldRevisionAbstractionInductionSide::Conclusion
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionClassInducer;

impl RecursiveWorldRevisionAbstractionClassInducer {
    pub fn induce(
        witness_set: RecursiveWorldRevisionAbstractionSubstitutionWitnessSet,
    ) -> Option<RecursiveWorldRevisionAbstractionInducedClassSet> {
        RecursiveWorldRevisionAbstractionInducedClassSet::induce(witness_set)
    }
}

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionVocabulary;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionResolvedClass {
    abstraction_class: RecursiveWorldRevisionAbstractionClass,
    contexts: Vec<RecursiveWorldRevisionAbstractionInductionContext>,
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
}

impl RecursiveWorldRevisionAbstractionResolvedClass {
    pub fn abstraction_class(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.abstraction_class
    }

    pub fn contexts(&self) -> &[RecursiveWorldRevisionAbstractionInductionContext] {
        &self.contexts
    }

    pub fn witnesses(&self) -> &[RecursiveWorldRevisionAbstractionSubstitutionWitness] {
        &self.witnesses
    }

    pub fn context_count(&self) -> usize {
        self.contexts.len()
    }

    pub fn witness_count(&self) -> usize {
        self.witnesses.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionVocabularyConflict {
    first: RecursiveWorldRevisionAbstractionClass,
    second: RecursiveWorldRevisionAbstractionClass,
    overlap: Vec<RecursiveUnit>,
}

impl RecursiveWorldRevisionAbstractionVocabularyConflict {
    pub fn first(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.first
    }

    pub fn second(&self) -> &RecursiveWorldRevisionAbstractionClass {
        &self.second
    }

    pub fn overlap(&self) -> &[RecursiveUnit] {
        &self.overlap
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionVocabularyResolution {
    source: RecursiveWorldRevisionAbstractionInducedClassSet,
    resolved_classes: Vec<RecursiveWorldRevisionAbstractionResolvedClass>,
    conflicted_classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    conflicts: Vec<RecursiveWorldRevisionAbstractionVocabularyConflict>,
    vocabulary: Option<RecursiveWorldRevisionAbstractionVocabulary>,
}

impl RecursiveWorldRevisionAbstractionVocabularyResolution {
    pub fn resolve(source: RecursiveWorldRevisionAbstractionInducedClassSet) -> Self {
        let mut by_identity = BTreeMap::<
            RecursiveWorldRevisionAbstractionClass,
            (
                BTreeSet<RecursiveWorldRevisionAbstractionInductionContext>,
                BTreeSet<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
            ),
        >::new();

        for induced in source.classes() {
            let entry = by_identity
                .entry(induced.abstraction_class().clone())
                .or_insert_with(|| (BTreeSet::new(), BTreeSet::new()));

            entry.0.insert(induced.context().clone());

            for witness in induced.witnesses() {
                entry.1.insert(witness.clone());
            }
        }

        let identities = by_identity.keys().cloned().collect::<Vec<_>>();

        let mut conflicted = BTreeSet::<RecursiveWorldRevisionAbstractionClass>::new();

        let mut conflicts = Vec::<RecursiveWorldRevisionAbstractionVocabularyConflict>::new();

        for left_index in 0..identities.len() {
            for right_index in (left_index + 1)..identities.len() {
                let left = &identities[left_index];

                let right = &identities[right_index];

                let right_members = right.members().iter().cloned().collect::<BTreeSet<_>>();

                let overlap = left
                    .members()
                    .iter()
                    .filter(|unit| right_members.contains(*unit))
                    .cloned()
                    .collect::<Vec<_>>();

                if overlap.is_empty() {
                    continue;
                }

                conflicted.insert(left.clone());

                conflicted.insert(right.clone());

                conflicts.push(RecursiveWorldRevisionAbstractionVocabularyConflict {
                    first: left.clone(),
                    second: right.clone(),
                    overlap,
                });
            }
        }

        conflicts.sort_by(|left, right| {
            left.first
                .cmp(&right.first)
                .then_with(|| left.second.cmp(&right.second))
        });

        let mut resolved_classes = Vec::<RecursiveWorldRevisionAbstractionResolvedClass>::new();

        for (abstraction_class, (contexts, witnesses)) in by_identity {
            if conflicted.contains(&abstraction_class) {
                continue;
            }

            resolved_classes.push(RecursiveWorldRevisionAbstractionResolvedClass {
                abstraction_class,
                contexts: contexts.into_iter().collect(),
                witnesses: witnesses.into_iter().collect(),
            });
        }

        resolved_classes
            .sort_by(|left, right| left.abstraction_class.cmp(&right.abstraction_class));

        let conflicted_classes = conflicted.into_iter().collect::<Vec<_>>();

        let vocabulary = if resolved_classes.is_empty() {
            None
        } else {
            RecursiveWorldRevisionAbstractionVocabulary::new(
                resolved_classes
                    .iter()
                    .map(|resolved| resolved.abstraction_class().clone())
                    .collect(),
            )
        };

        Self {
            source,
            resolved_classes,
            conflicted_classes,
            conflicts,
            vocabulary,
        }
    }

    pub fn source(&self) -> &RecursiveWorldRevisionAbstractionInducedClassSet {
        &self.source
    }

    pub fn resolved_classes(&self) -> &[RecursiveWorldRevisionAbstractionResolvedClass] {
        &self.resolved_classes
    }

    pub fn conflicted_classes(&self) -> &[RecursiveWorldRevisionAbstractionClass] {
        &self.conflicted_classes
    }

    pub fn conflicts(&self) -> &[RecursiveWorldRevisionAbstractionVocabularyConflict] {
        &self.conflicts
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.vocabulary.as_ref()
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    pub fn resolved_count(&self) -> usize {
        self.resolved_classes.len()
    }

    pub fn conflicted_count(&self) -> usize {
        self.conflicted_classes.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionVocabularyResolver;

impl RecursiveWorldRevisionAbstractionVocabularyResolver {
    pub fn resolve(
        source: RecursiveWorldRevisionAbstractionInducedClassSet,
    ) -> RecursiveWorldRevisionAbstractionVocabularyResolution {
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(source)
    }
}

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionProjection;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionInductionProjectionStatus {
    SubstitutionUnavailable,
    InductionUnavailable,
    VocabularyUnavailable,
    ProjectionUnavailable,
    Projected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionInductionProjectionBridge {
    source_observations: RecursiveWorldRevisionInductionObservationSet,
    witness_set: Option<RecursiveWorldRevisionAbstractionSubstitutionWitnessSet>,
    induced_classes: Option<RecursiveWorldRevisionAbstractionInducedClassSet>,
    resolution: Option<RecursiveWorldRevisionAbstractionVocabularyResolution>,
    projection: Option<RecursiveWorldRevisionAbstractionProjection>,
    status: RecursiveWorldRevisionAbstractionInductionProjectionStatus,
}

impl RecursiveWorldRevisionAbstractionInductionProjectionBridge {
    pub fn project(source_observations: RecursiveWorldRevisionInductionObservationSet) -> Self {
        let witness_set = RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::discover(
            source_observations.clone(),
        );

        let Some(witness_set_value) = witness_set.clone() else {
            return Self {
                source_observations,
                witness_set: None,
                induced_classes: None,
                resolution: None,
                projection: None,
                status:
                    RecursiveWorldRevisionAbstractionInductionProjectionStatus::
                        SubstitutionUnavailable,
            };
        };

        let induced_classes =
            RecursiveWorldRevisionAbstractionInducedClassSet::induce(witness_set_value);

        let Some(induced_classes_value) = induced_classes.clone() else {
            return Self {
                source_observations,
                witness_set,
                induced_classes: None,
                resolution: None,
                projection: None,
                status:
                    RecursiveWorldRevisionAbstractionInductionProjectionStatus::InductionUnavailable,
            };
        };

        let resolution =
            RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_classes_value);

        let Some(vocabulary) = resolution.vocabulary().cloned() else {
            return Self {
                source_observations,
                witness_set,
                induced_classes,
                resolution: Some(resolution),
                projection: None,
                status:
                    RecursiveWorldRevisionAbstractionInductionProjectionStatus::
                        VocabularyUnavailable,
            };
        };

        let projection = RecursiveWorldRevisionAbstractionProjection::project(
            vocabulary,
            source_observations.clone(),
        );

        let status = if projection.is_some() {
            RecursiveWorldRevisionAbstractionInductionProjectionStatus::Projected
        } else {
            RecursiveWorldRevisionAbstractionInductionProjectionStatus::ProjectionUnavailable
        };

        Self {
            source_observations,
            witness_set,
            induced_classes,
            resolution: Some(resolution),
            projection,
            status,
        }
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.source_observations
    }

    pub fn witness_set(&self) -> Option<&RecursiveWorldRevisionAbstractionSubstitutionWitnessSet> {
        self.witness_set.as_ref()
    }

    pub fn induced_classes(&self) -> Option<&RecursiveWorldRevisionAbstractionInducedClassSet> {
        self.induced_classes.as_ref()
    }

    pub fn resolution(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabularyResolution> {
        self.resolution.as_ref()
    }

    pub fn projection(&self) -> Option<&RecursiveWorldRevisionAbstractionProjection> {
        self.projection.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionInductionProjectionStatus {
        self.status
    }

    pub fn is_projected(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionInductionProjectionStatus::Projected
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.resolution
            .as_ref()
            .and_then(|resolution| resolution.vocabulary())
    }

    pub fn conflicts(&self) -> &[RecursiveWorldRevisionAbstractionVocabularyConflict] {
        self.resolution
            .as_ref()
            .map(|resolution| resolution.conflicts())
            .unwrap_or(&[])
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionInductionProjector;

impl RecursiveWorldRevisionAbstractionInductionProjector {
    pub fn project(
        source_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionInductionProjectionBridge {
        RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(source_observations)
    }
}

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionConsensus;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionInductionConsensusStatus {
    ProjectionUnavailable,
    ConsensusUnavailable,
    ConsensusDerived,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionInductionConsensusBridge {
    projection_bridge: RecursiveWorldRevisionAbstractionInductionProjectionBridge,
    consensus: Option<RecursiveWorldRevisionAbstractionConsensus>,
    status: RecursiveWorldRevisionAbstractionInductionConsensusStatus,
}

impl RecursiveWorldRevisionAbstractionInductionConsensusBridge {
    pub fn derive(source_observations: RecursiveWorldRevisionInductionObservationSet) -> Self {
        let projection_bridge = RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(
            source_observations,
        );

        let Some(projection) = projection_bridge.projection().cloned() else {
            return Self {
                projection_bridge,
                consensus: None,
                status:
                    RecursiveWorldRevisionAbstractionInductionConsensusStatus::ProjectionUnavailable,
            };
        };

        let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection);

        let status = if consensus.is_some() {
            RecursiveWorldRevisionAbstractionInductionConsensusStatus::ConsensusDerived
        } else {
            RecursiveWorldRevisionAbstractionInductionConsensusStatus::ConsensusUnavailable
        };

        Self {
            projection_bridge,
            consensus,
            status,
        }
    }

    pub fn projection_bridge(&self) -> &RecursiveWorldRevisionAbstractionInductionProjectionBridge {
        &self.projection_bridge
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.consensus.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionInductionConsensusStatus {
        self.status
    }

    pub fn is_consensus_derived(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionInductionConsensusStatus::ConsensusDerived
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.projection_bridge.source_observations()
    }

    pub fn witness_set(&self) -> Option<&RecursiveWorldRevisionAbstractionSubstitutionWitnessSet> {
        self.projection_bridge.witness_set()
    }

    pub fn induced_classes(&self) -> Option<&RecursiveWorldRevisionAbstractionInducedClassSet> {
        self.projection_bridge.induced_classes()
    }

    pub fn resolution(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabularyResolution> {
        self.projection_bridge.resolution()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.projection_bridge.vocabulary()
    }

    pub fn projection(&self) -> Option<&RecursiveWorldRevisionAbstractionProjection> {
        self.projection_bridge.projection()
    }

    pub fn conflicts(&self) -> &[RecursiveWorldRevisionAbstractionVocabularyConflict] {
        self.projection_bridge.conflicts()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionInductionConsensusBuilder;

impl RecursiveWorldRevisionAbstractionInductionConsensusBuilder {
    pub fn derive(
        source_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionInductionConsensusBridge {
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(source_observations)
    }
}

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractionRealization,
    RecursiveWorldRevisionAbstractionRealizationStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionInductionRealizationStatus {
    ConsensusUnavailable,
    Ambiguous,
    Deterministic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionInductionRealizationBridge {
    consensus_bridge: RecursiveWorldRevisionAbstractionInductionConsensusBridge,
    realization: Option<RecursiveWorldRevisionAbstractionRealization>,
    status: RecursiveWorldRevisionAbstractionInductionRealizationStatus,
}

impl RecursiveWorldRevisionAbstractionInductionRealizationBridge {
    pub fn realize(source_observations: RecursiveWorldRevisionInductionObservationSet) -> Self {
        let consensus_bridge =
            RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(source_observations);

        let Some(consensus) = consensus_bridge.consensus().cloned() else {
            return Self {
                consensus_bridge,
                realization: None,
                status:
                    RecursiveWorldRevisionAbstractionInductionRealizationStatus::
                        ConsensusUnavailable,
            };
        };

        let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus);

        let status = match realization.status() {
            RecursiveWorldRevisionAbstractionRealizationStatus::Ambiguous => {
                RecursiveWorldRevisionAbstractionInductionRealizationStatus::Ambiguous
            }
            RecursiveWorldRevisionAbstractionRealizationStatus::Deterministic => {
                RecursiveWorldRevisionAbstractionInductionRealizationStatus::Deterministic
            }
        };

        Self {
            consensus_bridge,
            realization: Some(realization),
            status,
        }
    }

    pub fn consensus_bridge(&self) -> &RecursiveWorldRevisionAbstractionInductionConsensusBridge {
        &self.consensus_bridge
    }

    pub fn realization(&self) -> Option<&RecursiveWorldRevisionAbstractionRealization> {
        self.realization.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionInductionRealizationStatus {
        self.status
    }

    pub fn is_ambiguous(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionInductionRealizationStatus::Ambiguous
    }

    pub fn is_deterministic(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionInductionRealizationStatus::Deterministic
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.realization
            .as_ref()
            .and_then(|realization| realization.realized_observation())
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.consensus_bridge.consensus()
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.consensus_bridge.source_observations()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.consensus_bridge.vocabulary()
    }

    pub fn premise_witnesses(
        &self,
        class: &RecursiveWorldRevisionAbstractionClass,
    ) -> &[RecursiveUnit] {
        self.realization
            .as_ref()
            .map(|realization| realization.premise_witnesses(class))
            .unwrap_or(&[])
    }

    pub fn conclusion_witnesses(
        &self,
        class: &RecursiveWorldRevisionAbstractionClass,
    ) -> &[RecursiveUnit] {
        self.realization
            .as_ref()
            .map(|realization| realization.conclusion_witnesses(class))
            .unwrap_or(&[])
    }

    pub fn conflicts(&self) -> &[RecursiveWorldRevisionAbstractionVocabularyConflict] {
        self.consensus_bridge.conflicts()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionInductionRealizer;

impl RecursiveWorldRevisionAbstractionInductionRealizer {
    pub fn realize(
        source_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionInductionRealizationBridge {
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(source_observations)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionTransferStatus {
    ConsensusUnavailable,
    Ambiguous,
    Deterministic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransfer {
    induction: RecursiveWorldRevisionAbstractionInductionConsensusBridge,
    transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    premise_witnesses: BTreeMap<RecursiveWorldRevisionAbstractionClass, Vec<RecursiveUnit>>,
    conclusion_witnesses: BTreeMap<RecursiveWorldRevisionAbstractionClass, Vec<RecursiveUnit>>,
    realized_observation: Option<RecursiveWorldRevisionDiscoveryObservation>,
    status: RecursiveWorldRevisionAbstractionTransferStatus,
}

impl RecursiveWorldRevisionAbstractionTransfer {
    pub fn transfer(
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        let induction = RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(
            induction_observations,
        );

        let Some(consensus) = induction.consensus() else {
            return Self {
                induction,
                transfer_observations,
                premise_witnesses: BTreeMap::new(),
                conclusion_witnesses: BTreeMap::new(),
                realized_observation: None,
                status: RecursiveWorldRevisionAbstractionTransferStatus::ConsensusUnavailable,
            };
        };

        let mut premise_sets =
            BTreeMap::<RecursiveWorldRevisionAbstractionClass, BTreeSet<RecursiveUnit>>::new();

        let mut conclusion_sets =
            BTreeMap::<RecursiveWorldRevisionAbstractionClass, BTreeSet<RecursiveUnit>>::new();

        for class in consensus.premise_classes() {
            premise_sets.insert(class.clone(), BTreeSet::new());
        }

        for class in consensus.conclusion_classes() {
            conclusion_sets.insert(class.clone(), BTreeSet::new());
        }

        for observation in transfer_observations.observations() {
            for concrete in observation.premises() {
                for (class, witnesses) in &mut premise_sets {
                    if class.contains(concrete) {
                        witnesses.insert(concrete.clone());
                    }
                }
            }

            for concrete in observation.conclusions() {
                for (class, witnesses) in &mut conclusion_sets {
                    if class.contains(concrete) {
                        witnesses.insert(concrete.clone());
                    }
                }
            }
        }

        let premise_witnesses = premise_sets
            .into_iter()
            .map(|(class, witnesses)| (class, witnesses.into_iter().collect::<Vec<_>>()))
            .collect::<BTreeMap<_, _>>();

        let conclusion_witnesses = conclusion_sets
            .into_iter()
            .map(|(class, witnesses)| (class, witnesses.into_iter().collect::<Vec<_>>()))
            .collect::<BTreeMap<_, _>>();

        let premise_deterministic = premise_witnesses
            .values()
            .all(|witnesses| witnesses.len() == 1);

        let conclusion_deterministic = conclusion_witnesses
            .values()
            .all(|witnesses| witnesses.len() == 1);

        let realized_observation = if premise_deterministic && conclusion_deterministic {
            let premises = consensus
                .premise_classes()
                .iter()
                .map(|class| premise_witnesses[class][0].clone())
                .collect::<Vec<_>>();

            let conclusions = consensus
                .conclusion_classes()
                .iter()
                .map(|class| conclusion_witnesses[class][0].clone())
                .collect::<Vec<_>>();

            RecursiveWorldRevisionDiscoveryObservation::new(premises, conclusions)
        } else {
            None
        };

        let status = if realized_observation.is_some() {
            RecursiveWorldRevisionAbstractionTransferStatus::Deterministic
        } else {
            RecursiveWorldRevisionAbstractionTransferStatus::Ambiguous
        };

        Self {
            induction,
            transfer_observations,
            premise_witnesses,
            conclusion_witnesses,
            realized_observation,
            status,
        }
    }

    pub fn induction(&self) -> &RecursiveWorldRevisionAbstractionInductionConsensusBridge {
        &self.induction
    }

    pub fn induction_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.induction.source_observations()
    }

    pub fn transfer_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.transfer_observations
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.induction.consensus()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.induction.vocabulary()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionTransferStatus {
        self.status
    }

    pub fn is_deterministic(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionTransferStatus::Deterministic
    }

    pub fn is_ambiguous(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionTransferStatus::Ambiguous
    }

    pub fn premise_witnesses(
        &self,
        class: &RecursiveWorldRevisionAbstractionClass,
    ) -> &[RecursiveUnit] {
        self.premise_witnesses
            .get(class)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn conclusion_witnesses(
        &self,
        class: &RecursiveWorldRevisionAbstractionClass,
    ) -> &[RecursiveUnit] {
        self.conclusion_witnesses
            .get(class)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.realized_observation.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferEngine;

impl RecursiveWorldRevisionAbstractionTransferEngine {
    pub fn transfer(
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionTransfer {
        RecursiveWorldRevisionAbstractionTransfer::transfer(
            induction_observations,
            transfer_observations,
        )
    }
}

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryHypothesis;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionTransferDiscoveryStatus {
    TransferUnavailable,
    DiscoveryUnavailable,
    Discovered,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferDiscoveryBridge {
    target: RecursiveWorldRule,
    transfer: RecursiveWorldRevisionAbstractionTransfer,
    hypothesis: Option<RecursiveWorldRevisionDiscoveryHypothesis>,
    status: RecursiveWorldRevisionAbstractionTransferDiscoveryStatus,
}

impl RecursiveWorldRevisionAbstractionTransferDiscoveryBridge {
    pub fn discover(
        target: RecursiveWorldRule,
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
            induction_observations,
            transfer_observations,
        );

        let Some(realized_observation) = transfer.realized_observation().cloned() else {
            return Self {
                target,
                transfer,
                hypothesis: None,
                status:
                    RecursiveWorldRevisionAbstractionTransferDiscoveryStatus::TransferUnavailable,
            };
        };

        let hypothesis = RecursiveWorldRevisionDiscoveryHypothesis::discover(
            target.clone(),
            realized_observation,
        );

        let status = if hypothesis.is_some() {
            RecursiveWorldRevisionAbstractionTransferDiscoveryStatus::Discovered
        } else {
            RecursiveWorldRevisionAbstractionTransferDiscoveryStatus::DiscoveryUnavailable
        };

        Self {
            target,
            transfer,
            hypothesis,
            status,
        }
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn transfer(&self) -> &RecursiveWorldRevisionAbstractionTransfer {
        &self.transfer
    }

    pub fn hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.hypothesis.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionTransferDiscoveryStatus {
        self.status
    }

    pub fn is_discovered(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionTransferDiscoveryStatus::Discovered
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.transfer.realized_observation()
    }

    pub fn replacement(&self) -> Option<&RecursiveWorldRule> {
        self.hypothesis
            .as_ref()
            .map(|hypothesis| hypothesis.replacement())
    }

    pub fn induction_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.transfer.induction_observations()
    }

    pub fn transfer_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.transfer.transfer_observations()
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.transfer.consensus()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.transfer.vocabulary()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferDiscoveryBuilder;

impl RecursiveWorldRevisionAbstractionTransferDiscoveryBuilder {
    pub fn discover(
        target: RecursiveWorldRule,
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionTransferDiscoveryBridge {
        RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
            target,
            induction_observations,
            transfer_observations,
        )
    }
}

use athlesia_recursive_world_model::RecursiveWorldModel;

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryHypothesisSet, RecursiveWorldRevisionDiscoveryValidation,
    RecursiveWorldRevisionDiscoveryValidator,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionTransferValidationStatus {
    DiscoveryUnavailable,
    Rejected,
    Accepted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferValidation {
    model: RecursiveWorldModel,
    discovery: RecursiveWorldRevisionAbstractionTransferDiscoveryBridge,
    validation: Option<RecursiveWorldRevisionDiscoveryValidation>,
    status: RecursiveWorldRevisionAbstractionTransferValidationStatus,
}

impl RecursiveWorldRevisionAbstractionTransferValidation {
    pub fn validate(
        model: RecursiveWorldModel,
        target: RecursiveWorldRule,
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        let discovery = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
            target,
            induction_observations,
            transfer_observations,
        );

        let Some(hypothesis) = discovery.hypothesis().cloned() else {
            return Self {
                model,
                discovery,
                validation: None,
                status:
                    RecursiveWorldRevisionAbstractionTransferValidationStatus::DiscoveryUnavailable,
            };
        };

        let hypothesis_set = RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis]);

        let validation = RecursiveWorldRevisionDiscoveryValidator::validate(&model, hypothesis_set);

        let status = if validation.accepted_count() == 1 {
            RecursiveWorldRevisionAbstractionTransferValidationStatus::Accepted
        } else {
            RecursiveWorldRevisionAbstractionTransferValidationStatus::Rejected
        };

        Self {
            model,
            discovery,
            validation: Some(validation),
            status,
        }
    }

    pub fn model(&self) -> &RecursiveWorldModel {
        &self.model
    }

    pub fn discovery(&self) -> &RecursiveWorldRevisionAbstractionTransferDiscoveryBridge {
        &self.discovery
    }

    pub fn validation(&self) -> Option<&RecursiveWorldRevisionDiscoveryValidation> {
        self.validation.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionTransferValidationStatus {
        self.status
    }

    pub fn is_accepted(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionTransferValidationStatus::Accepted
    }

    pub fn is_rejected(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionTransferValidationStatus::Rejected
    }

    pub fn accepted_hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.validation
            .as_ref()
            .and_then(|validation| validation.accepted_hypotheses().first())
    }

    pub fn rejected_hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.validation
            .as_ref()
            .and_then(|validation| validation.rejected_hypotheses().first())
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        self.discovery.target()
    }

    pub fn hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.discovery.hypothesis()
    }

    pub fn replacement(&self) -> Option<&RecursiveWorldRule> {
        self.discovery.replacement()
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.discovery.realized_observation()
    }

    pub fn induction_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.discovery.induction_observations()
    }

    pub fn transfer_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.discovery.transfer_observations()
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.discovery.consensus()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.discovery.vocabulary()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferValidator;

impl RecursiveWorldRevisionAbstractionTransferValidator {
    pub fn validate(
        model: RecursiveWorldModel,
        target: RecursiveWorldRule,
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionTransferValidation {
        RecursiveWorldRevisionAbstractionTransferValidation::validate(
            model,
            target,
            induction_observations,
            transfer_observations,
        )
    }
}

use athlesia_recursive_world_model_evidence::RecursiveWorldEvidenceState;

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryEvidenceScope, RecursiveWorldRevisionDiscoveryEvidenceScoper,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus {
    DiscoveryUnavailable,
    Rejected,
    Inactive,
    Active,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferEvidenceScope {
    validation: RecursiveWorldRevisionAbstractionTransferValidation,
    evidence_state: RecursiveWorldEvidenceState,
    scope: Option<RecursiveWorldRevisionDiscoveryEvidenceScope>,
    status: RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus,
}

impl RecursiveWorldRevisionAbstractionTransferEvidenceScope {
    pub fn scope(
        model: RecursiveWorldModel,
        evidence_state: RecursiveWorldEvidenceState,
        target: RecursiveWorldRule,
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
            model.clone(),
            target,
            induction_observations,
            transfer_observations,
        );

        match validation.status() {
            RecursiveWorldRevisionAbstractionTransferValidationStatus::DiscoveryUnavailable => {
                return Self {
                    validation,
                    evidence_state,
                    scope: None,
                    status:
                        RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::
                            DiscoveryUnavailable,
                };
            }

            RecursiveWorldRevisionAbstractionTransferValidationStatus::Rejected => {
                return Self {
                    validation,
                    evidence_state,
                    scope: None,
                    status: RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Rejected,
                };
            }

            RecursiveWorldRevisionAbstractionTransferValidationStatus::Accepted => {}
        }

        let hypothesis = validation
            .accepted_hypothesis()
            .cloned()
            .expect("accepted transfer validation contains exactly one accepted hypothesis");

        let hypothesis_set = RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis]);

        let scope = RecursiveWorldRevisionDiscoveryEvidenceScoper::scope(
            &model,
            &evidence_state,
            hypothesis_set,
        );

        let status = if scope.active_count() == 1 {
            RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Active
        } else {
            RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Inactive
        };

        Self {
            validation,
            evidence_state,
            scope: Some(scope),
            status,
        }
    }

    pub fn validation(&self) -> &RecursiveWorldRevisionAbstractionTransferValidation {
        &self.validation
    }

    pub fn evidence_state(&self) -> &RecursiveWorldEvidenceState {
        &self.evidence_state
    }

    pub fn scope_result(&self) -> Option<&RecursiveWorldRevisionDiscoveryEvidenceScope> {
        self.scope.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus {
        self.status
    }

    pub fn is_active(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Active
    }

    pub fn is_inactive(&self) -> bool {
        self.status == RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Inactive
    }

    pub fn active_hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.scope
            .as_ref()
            .and_then(|scope| scope.active_hypotheses().first())
    }

    pub fn pressured_rule(&self) -> Option<&RecursiveWorldRule> {
        self.scope.as_ref().and_then(|scope| scope.pressured_rule())
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        self.validation.target()
    }

    pub fn hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.validation.hypothesis()
    }

    pub fn replacement(&self) -> Option<&RecursiveWorldRule> {
        self.validation.replacement()
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.validation.realized_observation()
    }

    pub fn induction_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.validation.induction_observations()
    }

    pub fn transfer_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.validation.transfer_observations()
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.validation.consensus()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.validation.vocabulary()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferEvidenceScoper;

impl RecursiveWorldRevisionAbstractionTransferEvidenceScoper {
    pub fn scope(
        model: RecursiveWorldModel,
        evidence_state: RecursiveWorldEvidenceState,
        target: RecursiveWorldRule,
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> RecursiveWorldRevisionAbstractionTransferEvidenceScope {
        RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
            model,
            evidence_state,
            target,
            induction_observations,
            transfer_observations,
        )
    }
}

use athlesia_recursive_world_model::RecursiveWorldRevisionBudget;

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryCycle, RecursiveWorldRevisionDiscoveryCycleResult,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionAbstractionTransferCycleStatus {
    DiscoveryUnavailable,
    Rejected,
    Inactive,
    ActiveNoRevision,
    Revised,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferCycleResult {
    scope: RecursiveWorldRevisionAbstractionTransferEvidenceScope,
    cycle: Option<RecursiveWorldRevisionDiscoveryCycleResult>,
    status: RecursiveWorldRevisionAbstractionTransferCycleStatus,
}

impl RecursiveWorldRevisionAbstractionTransferCycleResult {
    pub fn evaluate(
        model: RecursiveWorldModel,
        evidence_state: RecursiveWorldEvidenceState,
        target: RecursiveWorldRule,
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
        budget: RecursiveWorldRevisionBudget,
    ) -> Self {
        let scope = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
            model.clone(),
            evidence_state.clone(),
            target,
            induction_observations,
            transfer_observations,
        );

        match scope.status() {
            RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::DiscoveryUnavailable => {
                return Self {
                    scope,
                    cycle: None,
                    status:
                        RecursiveWorldRevisionAbstractionTransferCycleStatus::DiscoveryUnavailable,
                };
            }

            RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Rejected => {
                return Self {
                    scope,
                    cycle: None,
                    status: RecursiveWorldRevisionAbstractionTransferCycleStatus::Rejected,
                };
            }

            RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Inactive
            | RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Active => {}
        }

        let hypothesis = scope
            .validation()
            .accepted_hypothesis()
            .cloned()
            .expect("validated transfer scope contains exactly one accepted hypothesis");

        let hypothesis_set = RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis]);

        let cycle = RecursiveWorldRevisionDiscoveryCycle::evaluate(
            &model,
            &evidence_state,
            hypothesis_set,
            budget,
        );

        let status = match scope.status() {
            RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Inactive => {
                RecursiveWorldRevisionAbstractionTransferCycleStatus::Inactive
            }

            RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Active => {
                if cycle.has_revision() {
                    RecursiveWorldRevisionAbstractionTransferCycleStatus::Revised
                } else {
                    RecursiveWorldRevisionAbstractionTransferCycleStatus::ActiveNoRevision
                }
            }

            RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::DiscoveryUnavailable
            | RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Rejected => {
                unreachable!("terminal transfer scope status returned before cycle evaluation")
            }
        };

        Self {
            scope,
            cycle: Some(cycle),
            status,
        }
    }

    pub fn scope(&self) -> &RecursiveWorldRevisionAbstractionTransferEvidenceScope {
        &self.scope
    }

    pub fn cycle(&self) -> Option<&RecursiveWorldRevisionDiscoveryCycleResult> {
        self.cycle.as_ref()
    }

    pub fn status(&self) -> RecursiveWorldRevisionAbstractionTransferCycleStatus {
        self.status
    }

    pub fn has_revision(&self) -> bool {
        self.cycle
            .as_ref()
            .map(RecursiveWorldRevisionDiscoveryCycleResult::has_revision)
            .unwrap_or(false)
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        self.scope.target()
    }

    pub fn hypothesis(&self) -> Option<&RecursiveWorldRevisionDiscoveryHypothesis> {
        self.scope.hypothesis()
    }

    pub fn realized_observation(&self) -> Option<&RecursiveWorldRevisionDiscoveryObservation> {
        self.scope.realized_observation()
    }

    pub fn induction_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.scope.induction_observations()
    }

    pub fn transfer_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.scope.transfer_observations()
    }

    pub fn consensus(&self) -> Option<&RecursiveWorldRevisionAbstractionConsensus> {
        self.scope.consensus()
    }

    pub fn vocabulary(&self) -> Option<&RecursiveWorldRevisionAbstractionVocabulary> {
        self.scope.vocabulary()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionAbstractionTransferCycle;

impl RecursiveWorldRevisionAbstractionTransferCycle {
    pub fn evaluate(
        model: RecursiveWorldModel,
        evidence_state: RecursiveWorldEvidenceState,
        target: RecursiveWorldRule,
        induction_observations: RecursiveWorldRevisionInductionObservationSet,
        transfer_observations: RecursiveWorldRevisionInductionObservationSet,
        budget: RecursiveWorldRevisionBudget,
    ) -> RecursiveWorldRevisionAbstractionTransferCycleResult {
        RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
            model,
            evidence_state,
            target,
            induction_observations,
            transfer_observations,
            budget,
        )
    }
}

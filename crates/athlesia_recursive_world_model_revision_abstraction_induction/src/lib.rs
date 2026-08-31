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

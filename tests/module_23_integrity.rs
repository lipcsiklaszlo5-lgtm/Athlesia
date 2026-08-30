use athlesia::{
    ActiveInferenceEngine, ConceptConsolidator, ConceptMemory, Encoder, ExperimentGenerator,
    ExperimentSelector, HypothesisInducer, PartialStructuralState, PredictionEngine,
    PredictionEvaluator, PredictionOutcome, PredictiveStructuralModel, PrimitiveDiscovery,
    PrimitiveSignature, RecognitionEngine, RelationKind, RelationalStructure, Role,
    StructuralConcept, StructuralSequence,
};

fn learn_memory<T>(values: &[T]) -> ConceptMemory
where
    T: Eq + std::hash::Hash,
{
    let encoder = Encoder::new();
    let sequence = encoder.encode(values);

    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::default().discover(&relations);

    let hypotheses = HypothesisInducer::default().induce(&primitives);

    let mut memory = ConceptMemory::new();

    ConceptConsolidator::new().consolidate_into(&hypotheses, &mut memory);

    memory
}

fn canonical_concept() -> StructuralConcept {
    StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 2)])
}

fn canonical_model() -> PredictiveStructuralModel {
    let encoder = Encoder::new();

    let sequence = encoder.encode(&[1, 2, 1, 2, 3]);

    let structure = RelationalStructure::from_sequence(&sequence);

    PredictiveStructuralModel::from_example(&canonical_concept(), &structure).unwrap()
}

#[test]
fn encoder_determinism() {
    let encoder = Encoder::new();

    let values = [1, 2, 1, 2, 3];

    assert_eq!(encoder.encode(&values), encoder.encode(&values));
}

#[test]
fn encoder_value_invariance() {
    let encoder = Encoder::new();

    assert_eq!(
        encoder.encode(&[1, 2, 1, 2, 3,]),
        encoder.encode(&[847, 13, 847, 13, 999,])
    );
}

#[test]
fn positive_transfer() {
    let memory = learn_memory(&[1, 2, 1, 2, 3]);

    let encoder = Encoder::new();

    let novel = encoder.encode(&[847, 13, 847, 13, 999]);

    let structure = RelationalStructure::from_sequence(&novel);

    let result = RecognitionEngine::default().recognize(&memory, &structure);

    assert_eq!(result.count(), 1);
}

#[test]
fn prediction_generation() {
    let model = canonical_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let predictions = PredictionEngine::new().predict(&model, &state).unwrap();

    assert_eq!(predictions.len(), 2);

    assert_eq!(predictions[0].reference(), 0);

    assert_eq!(predictions[0].target(), 2);

    assert_eq!(predictions[1].reference(), 1);

    assert_eq!(predictions[1].target(), 3);
}

#[test]
fn experiment_generation() {
    let model = canonical_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    assert_eq!(experiments.len(), 2);
    assert_eq!(experiments[0].target(), 2);
    assert_eq!(experiments[1].target(), 3);
}

#[test]
fn experiment_selection() {
    let model = canonical_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let selected = ExperimentSelector::new().select(&experiments).unwrap();

    assert_eq!(selected.target(), 2);
}

#[test]
fn prediction_confirmation() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let model = canonical_model();

    let evaluation = PredictionEvaluator::new()
        .evaluate(model.rules()[0], &observation)
        .unwrap();

    assert_eq!(evaluation.outcome(), PredictionOutcome::Confirmed);
}

#[test]
fn prediction_violation() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 999, 13, 777]);

    let model = canonical_model();

    let evaluation = PredictionEvaluator::new()
        .evaluate(model.rules()[0], &observation)
        .unwrap();

    assert_eq!(evaluation.outcome(), PredictionOutcome::Violated);
}

#[test]
fn negative_rejection() {
    let memory = learn_memory(&[1, 2, 1, 2, 3]);

    let encoder = Encoder::new();

    let negative = encoder.encode(&[10, 20, 30, 20, 40]);

    let structure = RelationalStructure::from_sequence(&negative);

    let result = RecognitionEngine::default().recognize(&memory, &structure);

    assert!(result.is_empty());
}

#[test]
fn no_partial_match() {
    let memory = learn_memory(&[1, 2, 1, 2, 3]);

    let encoder = Encoder::new();

    let partial = encoder.encode(&[10, 20, 10, 20]);

    let structure = RelationalStructure::from_sequence(&partial);

    let result = RecognitionEngine::default().recognize(&memory, &structure);

    assert!(
        result.is_empty(),
        "A shorter partial structure must not satisfy a complete learned concept"
    );
}

#[test]
fn extra_tail_rejection() {
    let memory = learn_memory(&[1, 2, 1, 2, 3]);

    let encoder = Encoder::new();

    let extended = encoder.encode(&[10, 20, 10, 20, 30, 40]);

    let structure = RelationalStructure::from_sequence(&extended);

    let result = RecognitionEngine::default().recognize(&memory, &structure);

    assert!(
        result.is_empty(),
        "A longer structure with a matching interior pattern must not satisfy the exact concept"
    );
}

#[test]
fn anti_memorization() {
    let memory = learn_memory(&[847, 13, 847, 13, 999]);

    let concepts: Vec<StructuralConcept> = memory.concepts().cloned().collect();

    assert_eq!(concepts.len(), 1);

    assert_eq!(
        concepts[0].signatures(),
        &[PrimitiveSignature::new(RelationKind::Equal, 2,),]
    );
}

#[test]
fn memory_purity() {
    let memory = learn_memory(&[847, 13, 847, 13, 999]);

    for concept in memory.concepts() {
        for signature in concept.signatures() {
            assert_eq!(signature.kind(), RelationKind::Equal);

            assert!(signature.span() > 0);
        }
    }
}

#[test]
fn role_id_independence() {
    let canonical = StructuralSequence::new(vec![
        Role::new(0),
        Role::new(1),
        Role::new(0),
        Role::new(1),
        Role::new(2),
    ]);

    let renamed = StructuralSequence::new(vec![
        Role::new(9),
        Role::new(3),
        Role::new(9),
        Role::new(3),
        Role::new(8),
    ]);

    let first = RelationalStructure::from_sequence(&canonical);

    let second = RelationalStructure::from_sequence(&renamed);

    assert_eq!(first, second);
}

#[test]
fn repeated_execution_stability() {
    let model = canonical_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let engine = ActiveInferenceEngine::new();

    let first = engine.step(&model, &state, &observation);

    let second = engine.step(&model, &state, &observation);

    assert_eq!(first, second);
}

#[test]
fn training_mutation_isolation() {
    let mut training = vec![1, 2, 1, 2, 3];

    let memory = learn_memory(&training);

    training.fill(999);

    let encoder = Encoder::new();

    let novel = encoder.encode(&[847, 13, 847, 13, 777]);

    let structure = RelationalStructure::from_sequence(&novel);

    let result = RecognitionEngine::default().recognize(&memory, &structure);

    assert_eq!(result.count(), 1);
}

#[test]
fn selector_no_value_dependency() {
    let first_encoder = Encoder::new();

    let first_sequence = first_encoder.encode(&[1, 2, 1, 2, 3]);

    let second_sequence = first_encoder.encode(&[847, 13, 847, 13, 999]);

    let first_structure = RelationalStructure::from_sequence(&first_sequence);

    let second_structure = RelationalStructure::from_sequence(&second_sequence);

    let concept = canonical_concept();

    let first_model = PredictiveStructuralModel::from_example(&concept, &first_structure).unwrap();

    let second_model =
        PredictiveStructuralModel::from_example(&concept, &second_structure).unwrap();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let generator = ExperimentGenerator::new();

    let first_candidates = generator.generate(&first_model, &state).unwrap();

    let second_candidates = generator.generate(&second_model, &state).unwrap();

    let selector = ExperimentSelector::new();

    assert_eq!(
        selector.select(&first_candidates),
        selector.select(&second_candidates)
    );
}

#[test]
fn information_gain_is_structural() {
    let model = canonical_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    for candidate in candidates {
        assert_eq!(
            candidate.information_gain(),
            candidate.supporting_rules().len()
        );
    }
}

#[test]
fn active_state_transition() {
    let model = canonical_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let transition = ActiveInferenceEngine::new()
        .step(&model, &state, &observation)
        .unwrap();

    assert_eq!(transition.before().observed_count(), 2);

    assert_eq!(transition.after().observed_count(), 3);

    assert_eq!(transition.selected().target(), 2);

    assert_eq!(transition.confirmed_count(), 1);
}

use athlesia::{Encoder, PredictionEvaluator, PredictionOutcome, PredictionRule};

#[test]
fn equal_prediction_is_confirmed() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let rule = PredictionRule::new_equal(0, 2);

    let evaluation = PredictionEvaluator::new()
        .evaluate(rule, &observation)
        .unwrap();

    assert_eq!(evaluation.outcome(), PredictionOutcome::Confirmed);

    assert!(evaluation.is_confirmed());
    assert!(!evaluation.is_violated());
}

#[test]
fn unequal_completion_violates_prediction() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 999, 13, 777]);

    let rule = PredictionRule::new_equal(0, 2);

    let evaluation = PredictionEvaluator::new()
        .evaluate(rule, &observation)
        .unwrap();

    assert_eq!(evaluation.outcome(), PredictionOutcome::Violated);

    assert!(evaluation.is_violated());
    assert!(!evaluation.is_confirmed());
}

#[test]
fn evaluation_preserves_prediction_rule() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[10, 20, 10, 20, 30]);

    let rule = PredictionRule::new_equal(1, 3);

    let evaluation = PredictionEvaluator::new()
        .evaluate(rule, &observation)
        .unwrap();

    assert_eq!(evaluation.rule(), rule);
}

#[test]
fn multiple_predictions_can_all_be_confirmed() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let rules = [
        PredictionRule::new_equal(0, 2),
        PredictionRule::new_equal(1, 3),
    ];

    let evaluations = PredictionEvaluator::new().evaluate_all(&rules, &observation);

    assert_eq!(evaluations.len(), 2);

    assert!(evaluations
        .iter()
        .all(|evaluation| { evaluation.is_confirmed() }));
}

#[test]
fn mixed_confirmation_and_violation_are_preserved() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[100, 200, 100, 999, 777]);

    let rules = [
        PredictionRule::new_equal(0, 2),
        PredictionRule::new_equal(1, 3),
    ];

    let evaluations = PredictionEvaluator::new().evaluate_all(&rules, &observation);

    assert_eq!(evaluations.len(), 2);

    assert_eq!(evaluations[0].outcome(), PredictionOutcome::Confirmed);

    assert_eq!(evaluations[1].outcome(), PredictionOutcome::Violated);
}

#[test]
fn prediction_violation_is_explicit() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[1, 2, 3, 2, 4]);

    let rule = PredictionRule::new_equal(0, 2);

    let evaluation = PredictionEvaluator::new()
        .evaluate(rule, &observation)
        .unwrap();

    assert_eq!(evaluation.outcome(), PredictionOutcome::Violated);
}

#[test]
fn evaluation_is_value_invariant() {
    let encoder = Encoder::new();

    let first = encoder.encode(&[1, 2, 1, 2, 3]);

    let second = encoder.encode(&[847, 13, 847, 13, 999]);

    let rule = PredictionRule::new_equal(0, 2);

    let evaluator = PredictionEvaluator::new();

    let first_result = evaluator.evaluate(rule, &first).unwrap();

    let second_result = evaluator.evaluate(rule, &second).unwrap();

    assert_eq!(first_result, second_result);
}

#[test]
fn violation_is_value_invariant() {
    let encoder = Encoder::new();

    let first = encoder.encode(&[1, 2, 3, 2, 4]);

    let second = encoder.encode(&[847, 13, 999, 13, 777]);

    let rule = PredictionRule::new_equal(0, 2);

    let evaluator = PredictionEvaluator::new();

    assert_eq!(
        evaluator.evaluate(rule, &first,),
        evaluator.evaluate(rule, &second,)
    );
}

#[test]
fn evaluation_is_deterministic() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[1, 2, 1, 2, 3]);

    let rule = PredictionRule::new_equal(0, 2);

    let evaluator = PredictionEvaluator::new();

    let first = evaluator.evaluate(rule, &observation);

    let second = evaluator.evaluate(rule, &observation);

    assert_eq!(first, second);
}

#[test]
fn missing_target_cannot_be_evaluated() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[1, 2]);

    let rule = PredictionRule::new_equal(0, 2);

    let result = PredictionEvaluator::new().evaluate(rule, &observation);

    assert!(result.is_none());
}

#[test]
fn missing_reference_cannot_be_evaluated() {
    let encoder = Encoder::new();

    let observation = encoder.encode::<i32>(&[]);

    let rule = PredictionRule::new_equal(0, 2);

    let result = PredictionEvaluator::new().evaluate(rule, &observation);

    assert!(result.is_none());
}

#[test]
fn evaluation_contains_no_concrete_value() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let rule = PredictionRule::new_equal(0, 2);

    let evaluation = PredictionEvaluator::new()
        .evaluate(rule, &observation)
        .unwrap();

    assert_eq!(evaluation.rule(), PredictionRule::new_equal(0, 2,));

    assert_eq!(evaluation.outcome(), PredictionOutcome::Confirmed);
}

#[test]
fn evaluate_all_preserves_rule_order() {
    let encoder = Encoder::new();

    let observation = encoder.encode(&[1, 2, 1, 2, 3]);

    let rules = [
        PredictionRule::new_equal(1, 3),
        PredictionRule::new_equal(0, 2),
    ];

    let evaluations = PredictionEvaluator::new().evaluate_all(&rules, &observation);

    assert_eq!(evaluations[0].rule(), rules[0]);

    assert_eq!(evaluations[1].rule(), rules[1]);
}

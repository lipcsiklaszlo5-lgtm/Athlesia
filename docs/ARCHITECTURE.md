
Architecture

Athlesia separates observation, structure, learning, prediction and active observation into explicit components.

Processing pipeline
Observation
   |
Encoder
   |
StructuralSequence
   |
RelationalStructure
   |
PrimitiveDiscovery
   |
StructuralPrimitive
   |
HypothesisInducer
   |
StructuralHypothesis
   |
ConceptConsolidator
   |
ConceptMemory

The active path is:

ConceptMemory
   |
PredictiveStructuralModel
   |
PredictionEngine
   |
ExperimentGenerator
   |
ExperimentSelector
   |
PredictionEvaluator
   |
ActiveInferenceEngine
Structural encoding

The encoder replaces concrete values with first-occurrence roles.

[50, 80, 50, 80, 12]

becomes

[R0, R1, R0, R1, R2]

This preserves equality structure while removing concrete identity.

Structural concepts

A learned concept contains:

primitive signatures
+
structural extent

It does not require the original training values.

Structural extent prevents a learned complete structure from matching shorter or longer observations merely because they contain the same internal pattern.

Prediction

A predictive model stores structural rules such as:

0 -> 2 : Equal
1 -> 3 : Equal

A partial state can therefore produce predictions about unknown positions.

Active observation

Prediction targets become experiment candidates.

Candidates are scored using structural information gain.

The current score is the number of structural predictions tested by an observation.

Selection is deterministic.

Evaluation

Predictions are evaluated as:

Confirmed

or:

Violated

A violation remains explicit evidence and is not treated as equivalent to the absence of a concept.

Determinism

The baseline currently uses deterministic:

role assignment
relation ordering
concept storage
hypothesis ranking
prediction ordering
experiment ordering
tie-breaking

This makes the behaviour reproducible and easy to test.

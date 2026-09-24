# Evidence principles

These principles govern the Open Intelligence experiment and any follow-up benchmark.

## Declare the prior

List what the agent receives and what it must infer. In v4, the feature vocabulary is fixed to five geometric relations; the agent learns their role association through interaction. Do not describe those features as discovered from raw perception.

## Freeze the question before scoring

Define the task, acceptance criteria, seed ranges, and primary metric before running the final split. Do not use final held-out outcomes to change the policy.

## Compare like with like

Use paired world specifications for trained and control agents. Keep a cold model with the same fixed priors, plus a simple random baseline. Report success and action cost; near-ceiling success alone can hide useful differences.

## Treat leakage as a testable property

Keep training, held-out, and distribution-shift seeds and surface tokens disjoint. Remap action IDs. Do not expose hidden state, labels, goal coordinates, or family IDs to the policy.

## Publish the failure boundary

Show every task family and distribution-shift result. In v4, role reversal makes the trained model slower than cold; this negative transfer belongs in the abstract and the application.

## Separate evidence from ambition

A synthetic benchmark result does not establish general intelligence, broad causal reasoning, ARC-AGI performance, or integration with separate Rust components. State the exact system and environment measured.

## Make reproduction cheap

Prefer local deterministic tests, standard-library dependencies, explicit commands, and machine-readable outputs. A reviewer should be able to run the visible showcase first and the full benchmark afterward.

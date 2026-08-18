#!/usr/bin/env python3
"""Manhattan Kernel — jelenlegi kód audit a Cognitive Controller beillesztéséhez."""
import os, pathlib, re

print("=" * 80)
print("1. CRATE DEPENDENCY MAP")
print("=" * 80)

workspace_toml = pathlib.Path("Cargo.toml").read_text()
print(workspace_toml)

print("\n" + "=" * 80)
print("2. RUNTIME CALL GRAPH — solve_arc_json -> executor")
print("=" * 80)

kernel_lib = pathlib.Path("crates/athlesia-kernel/src/lib.rs").read_text()

# Kinyeri a solve_arc_json függvényt
start = kernel_lib.find("pub fn solve_arc_json")
end = kernel_lib.find("\n}\n", start)
if end == -1:
    end = len(kernel_lib)
solve_arc_json = kernel_lib[start:end]
print(solve_arc_json)

print("\n" + "=" * 80)
print("3. ADATFOLYAM A CORE ENGINE-BEN (solve_with_steps)")
print("=" * 80)

core_lib = pathlib.Path("crates/athlesia-core/src/lib.rs").read_text()
start = core_lib.find("pub fn solve_with_steps")
end = core_lib.find("\n    }\n", start)
if end == -1:
    end = len(core_lib)
solve_with_steps = core_lib[start:end]
print(solve_with_steps)

print("\n" + "=" * 80)
print("4. A JELENLEGI FŐ HIÁNYOSSÁGOK (a specifikáció alapján)")
print("=" * 80)

# Megnézzük, melyik függvények hiányoznak a futási ciklusból
missing = [
    "CognitiveDecision",
    "CompetenceEstimate",
    "VerificationReport",
    "PredictionError",
    "Belief",
    "expected_information_gain",
    "decompose_dimensions",
    "MetaGrid",
]

for term in missing:
    found = False
    for rs_file in pathlib.Path("crates").rglob("*.rs"):
        content = rs_file.read_text()
        if term in content:
            found = True
            break
    status = "VAN" if found else "HIÁNYZIK"
    print(f"  - {term:30s} : {status}")

print("\n" + "=" * 80)
print("5. AHOVA A KOGNITÍV VEZÉRLŐT BE KELL ILLESZTENI")
print("=" * 80)
print("""
Jelenlegi útvonal:
  solve_arc_json()
    -> Agent::step()
      -> Planner::plan()
        -> Search/Synthesis
          -> Executor::run_program()
          -> Verifier::verify()   <-- bináris (Accept/Reject)

Szükséges útvonal:
  solve_arc_json()
    -> CognitiveController::decide()
      -> StructuralAnalysis::decompose()
        -> CompetenceEstimate
        -> [Solve | Explore | Guess | Abstain]
          -> Planner::plan()
            -> Search/Synthesis
              -> Executor::run_program()
              -> StructuralVerifier::report()  <-- strukturált hibajel
          -> PredictionError -> Abstraction -> BeliefUpdate
""")

print("\n" + "=" * 80)
print("6. MINIMAL PATCH TERVE")
print("=" * 80)
print("""
Fázis 1 (következő lépés):
  - CognitiveController enum és döntési logika a kernel-ben
  - HardConstraintError a types-ban
  - SimplicityScore a metalearner-ben

Fázis 2:
  - StructuralVerifier::report() a verifier-ben
  - MetaGrid integrálása a synthesis-be

Fázis 3:
  - PredictionError és Belief a world-model-ben
  - AbstractionEngine beillesztése a runtime loop-ba

NEM kezdjük egyszerre mindet. Először a Phase 1-et.
""")

print("AUDIT KÉSZ.")

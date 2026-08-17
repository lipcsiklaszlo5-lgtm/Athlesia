#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
KERNEL_LIB = os.path.join(PROJECT, "crates", "athlesia-kernel", "src", "lib.rs")
KERNEL_TEST = os.path.join(PROJECT, "crates", "athlesia-kernel", "tests", "kernel_integration_test.rs")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Kernel lib.rs frissítése: CoreEngine használata
lib_content = pathlib.Path(KERNEL_LIB).read_text()

# Import hozzáadása
lib_content = lib_content.replace(
    "use athlesia_executor::run_program;",
    "use athlesia_executor::run_program;\nuse athlesia_core::CoreEngine;"
)

# A solve_with_kernel függvény cseréje
old_fn = '''pub fn solve_with_kernel(
    input: &Grid,
    target: &Grid,
    kb: &mut KnowledgeBase,
    memory: &mut Memory,
    planner: &Planner,
    wm: &athlesia_world_model::WorldModel,
    max_depth: usize,
) -> Option<Program> {
    // 1. Próbáljuk a memóriában lévő programokat
    if let Some(program) = memory.find_program_by_input(input) {
        let mut budget = Budget { max_steps: program.len() as u64 };
        if let Ok(output) = run_program(&program, input, &mut budget) {
            if output == *target {
                return Some(program);
            }
        }
    }

    // 2. Próbáljuk a tudásbázisból a makrókat
    for m in kb.get_all_macros() {
        let mut budget = Budget { max_steps: m.program.len() as u64 };
        if let Ok(output) = run_program(&m.program, input, &mut budget) {
            if output == *target {
                memory.add_episode(input.clone(), target.clone(), m.program.clone());
                return Some(m.program.clone());
            }
        }
    }

    // 3. Tervezés a keresőmotorral
    if let Some(program) = planner.plan(input, Some(target), wm, max_depth) {
        let verifier = Verifier;
        if verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
            memory.add_episode(input.clone(), target.clone(), program.clone());
            kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
            return Some(program);
        }
    }

    None
}'''

new_fn = '''pub fn solve_with_kernel(
    input: &Grid,
    target: &Grid,
    kb: &mut KnowledgeBase,
    memory: &mut Memory,
    planner: &Planner,
    wm: &athlesia_world_model::WorldModel,
    core: &mut CoreEngine,
    max_depth: usize,
) -> Option<Program> {
    // 1. Próbáljuk a CoreEngine-t, amely a memóriát, a MetaLearnert,
    //    a Synthesis Engine-t és a Search Engine-t is használja.
    if let Some(program) = core.solve(input, target) {
        memory.add_episode(input.clone(), target.clone(), program.clone());
        kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
        return Some(program);
    }

    // 2. Ha a CoreEngine nem járt sikerrel, próbáljuk a Plannert.
    //    (pl. ha a CoreEngine nem ismerte fel, de a Planner mégis talál megoldást)
    if let Some(program) = planner.plan(input, Some(target), wm, max_depth) {
        let verifier = Verifier;
        if verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
            memory.add_episode(input.clone(), target.clone(), program.clone());
            kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
            return Some(program);
        }
    }

    None
}'''

if old_fn in lib_content:
    lib_content = lib_content.replace(old_fn, new_fn)
    write_file(KERNEL_LIB, lib_content)
    print("[INFO] solve_with_kernel most már a CoreEngine-t használja.")
else:
    print("[ERROR] Nem találtam a régi solve_with_kernel függvényt.")
    sys.exit(1)

# 2. Kernel integrációs teszt frissítése
test_content = r'''
use athlesia_kernel::solve_with_kernel;
use athlesia_core::CoreEngine;
use athlesia_memory::Memory;
use athlesia_knowledge::KnowledgeBase;
use athlesia_planner::{Planner, PlannerMode};
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

fn make_input(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[x][y] = 1;
    build_grid(rows)
}

fn make_target(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[x][y + 1] = 1;
    build_grid(rows)
}

#[test]
fn kernel_uses_core_engine_for_learning() {
    let mut kb = KnowledgeBase::new();
    let mut mem = Memory::new();
    let planner = Planner::new(PlannerMode::GoalDirected);
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let mut core = CoreEngine::new();

    let positions = [(0, 0), (1, 1), (2, 2)];
    let expected_program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    for (x, y) in positions {
        let input = make_input(x, y);
        let target = make_target(x, y);

        let program = solve_with_kernel(
            &input,
            &target,
            &mut kb,
            &mut mem,
            &planner,
            &wm,
            &mut core,
            2,
        );

        assert!(program.is_some(), "A kernelnek minden pozícióban meg kell oldania a feladatot");
        assert_eq!(program.unwrap(), expected_program, "A megoldásnak mindig Translate(1,0)-nak kell lennie");
    }

    // Három megoldott feladat után a memóriában és a tudásbázisban is nyoma kell legyen
    assert_eq!(mem.episodic.len(), 3);
    assert_eq!(mem.get_known_programs().len(), 1, "Ugyanaz a program csak egyszer tárolódik");
    assert_eq!(kb.get_all_macros().len(), 1, "A tudásbázisban egy makró legyen");
}

#[test]
fn kernel_solves_two_step_program_with_core() {
    let mut kb = KnowledgeBase::new();
    let mut mem = Memory::new();
    let planner = Planner::new(PlannerMode::GoalDirected);
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let mut core = CoreEngine::new();

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let program = solve_with_kernel(&input, &target, &mut kb, &mut mem, &planner, &wm, &mut core, 3);
    assert!(program.is_some(), "A kernelnek meg kell oldania a kétlépéses feladatot");
    assert!(program.unwrap().len() >= 2);
}
'''
write_file(KERNEL_TEST, test_content)
print("[INFO] Kernel integrációs teszt frissítve.")

# 3. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-kernel"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Kernel integrációs tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Kernel integrációs tesztek zöldek.")

# 4. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Integrate CoreEngine into kernel solve pipeline"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")

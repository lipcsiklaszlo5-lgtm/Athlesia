#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
KERNEL_LIB = os.path.join(PROJECT, "crates", "athlesia-kernel", "src", "lib.rs")
AGENT_TEST = os.path.join(PROJECT, "crates", "athlesia-kernel", "tests", "agent_learning_test.rs")
NEW_TEST = os.path.join(PROJECT, "crates", "athlesia-kernel", "tests", "goal_directed_agent_test.rs")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Agent step metódus frissítése cél-irányított módra
lib_content = pathlib.Path(KERNEL_LIB).read_text()

old_step = '''    /// Feltáró lépés: a Planner kiválaszt egy akciót a bizonytalanság alapján,
    /// majd a kiválasztott akciót hozzáadjuk a WorldModel hipotéziseihez,
    /// hogy a későbbi update tanulni tudjon belőle.
    pub fn step(&mut self, current: &Grid) -> Action {
        let program = self
            .planner
            .plan(current, None, &self.wm, 1)
            .expect("Feltáró módban mindig kell lennie akciónak");

        let (prim, params) = program[0].clone();
        let action = Action { prim, params };

        let prog = vec![(prim, params)];
        if !self.wm.hypotheses.iter().any(|h| h.program == prog) {
            self.wm.add_hypothesis(prog);
        }

        action
    }'''

new_step = '''    /// Lépés: ha adott cél, és a világmodell elég magabiztos, cél-irányított
    /// tervezést használ, egyébként feltár.
    /// A kiválasztott akciót minden esetben hozzáadjuk a hipotézisekhez.
    pub fn step(&mut self, current: &Grid, target: Option<&Grid>) -> Action {
        if let Some(goal) = target {
            // Próbáljunk cél-irányított tervet készíteni
            let goal_planner = Planner::new(PlannerMode::GoalDirected);
            if let Some(program) = goal_planner.plan(current, Some(goal), &self.wm, 3) {
                let (prim, params) = program[0].clone();
                let action = Action { prim, params };

                let prog = vec![(prim, params)];
                if !self.wm.hypotheses.iter().any(|h| h.program == prog) {
                    self.wm.add_hypothesis(prog);
                }
                return action;
            }
        }

        // Feltáró ág, ha nincs cél vagy a cél-irányított keresés kudarcot vall
        let program = self
            .planner
            .plan(current, None, &self.wm, 1)
            .expect("Feltáró módban mindig kell lennie akciónak");

        let (prim, params) = program[0].clone();
        let action = Action { prim, params };

        let prog = vec![(prim, params)];
        if !self.wm.hypotheses.iter().any(|h| h.program == prog) {
            self.wm.add_hypothesis(prog);
        }

        action
    }'''

if old_step in lib_content:
    lib_content = lib_content.replace(old_step, new_step)
    write_file(KERNEL_LIB, lib_content)
    print("[INFO] Agent step metódus frissítve cél-irányított támogatással.")
else:
    print("[ERROR] Nem találtam a régi step metódust.")
    sys.exit(1)

# 2. Meglévő agent tanuló teszt frissítése az új szignatúrához
test_content = pathlib.Path(AGENT_TEST).read_text()
test_content = test_content.replace("agent.step(&current)", "agent.step(&current, None)")
write_file(AGENT_TEST, test_content)
print("[INFO] Meglévő agent teszt frissítve.")

# 3. Új cél-irányított ágens teszt
new_test = r'''
use athlesia_kernel::Agent;
use athlesia_types::{Grid, PrimName, Params, Budget};
use athlesia_executor::run_program;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn agent_uses_goal_directed_plan_after_learning() {
    let start = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let mut agent = Agent::new(start.clone());

    // Tanuljuk meg a Translate(1,0) szabályt ugyanazzal a szabállyal,
    // mint amit a környezet használ.
    let rule = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let mut current = start;
    for _ in 0..5 {
        let _action = agent.step(&current, None);
        let mut budget = Budget { max_steps: 1 };
        let next = run_program(&rule, &current, &mut budget).unwrap();
        agent.update(&current, &next);
        current = next;
    }

    // Most egy új, ismeretlen pozíciójú bemenetet és célt adunk.
    let new_start = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 1, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    // A cél-irányított lépésnek azonnal a Translate(1,0) akciót kell adnia.
    let action = agent.step(&new_start, Some(&target));

    assert_eq!(action.prim, PrimName::Translate);
    match action.params {
        Params::Translate(dx, dy) => {
            assert_eq!(dx, 1);
            assert_eq!(dy, 0);
        }
        _ => panic!("Nem Translate akciót kaptunk"),
    }
}
'''
write_file(NEW_TEST, new_test)
print("[INFO] Cél-irányított ágens teszt hozzáadva.")

# 4. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-kernel"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Kernel cél-irányított ágens tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Kernel cél-irányított ágens tesztek zöldek.")

# 5. Git commit és push a szülőből
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add goal-directed agent mode"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")

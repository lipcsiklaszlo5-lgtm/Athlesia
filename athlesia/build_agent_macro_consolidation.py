#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
KERNEL_LIB = os.path.join(PROJECT, "crates", "athlesia-kernel", "src", "lib.rs")
CORE_LIB = os.path.join(PROJECT, "crates", "athlesia-core", "src", "lib.rs")
MEMORY_LIB = os.path.join(PROJECT, "crates", "athlesia-memory", "src", "lib.rs")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Memory bővítése: consolidate_known_programs metódus
mem_content = pathlib.Path(MEMORY_LIB).read_text()
if "pub fn consolidate_known_programs" not in mem_content:
    mem_add = '''
    /// Hosszú távú memóriába emeli az epizodikus memóriában lévő összes programot.
    /// Ez a játékok közötti tanulás alapja.
    pub fn consolidate_known_programs(&mut self) {
        for ep in &self.episodic {
            self.long_term.add_program(ep.program.clone());
        }
    }
'''
    marker = "    pub fn get_known_programs"
    insertion = mem_content.find(marker)
    if insertion == -1:
        print("[ERROR] Memory marker hiányzik.")
        sys.exit(1)
    mem_content = mem_content[:insertion] + mem_add + mem_content[insertion:]
    write_file(MEMORY_LIB, mem_content)
    print("[INFO] Memory consolidation metódus hozzáadva.")
else:
    print("[INFO] Memory consolidation már létezik.")

# 2. Knowledge Base bővítése: add_program_as_macro
# Már van add_macro, ezért csak használjuk.

# 3. Kernel Agent bővítése: consolidate_learned_macros
kernel_content = pathlib.Path(KERNEL_LIB).read_text()
if "pub fn consolidate_learned_macros" not in kernel_content:
    agent_add = '''
    /// A WorldModel megerősített hipotéziseit makróként átemeli a Knowledge Base-be,
    /// és a memóriába is. Ez a játékon belüli tanulás lezárása.
    pub fn consolidate_learned_macros(&mut self, kb: &mut KnowledgeBase, memory: &mut Memory) {
        for hyp in &self.wm.hypotheses {
            if hyp.status == HypothesisStatus::Confirmed {
                kb.add_macro(format!("learned_{}", kb.get_all_macros().len()), hyp.program.clone());
                memory.long_term.add_program(hyp.program.clone());
            }
        }
    }
'''
    marker = "    /// A környezet megfigyelése után frissíti a WorldModel-t."
    insertion = kernel_content.find(marker)
    if insertion == -1:
        print("[ERROR] Kernel Agent marker hiányzik.")
        sys.exit(1)
    kernel_content = kernel_content[:insertion] + agent_add + kernel_content[insertion:]
    write_file(KERNEL_LIB, kernel_content)
    print("[INFO] Agent consolidate_learned_macros metódus hozzáadva.")
else:
    print("[INFO] Agent consolidation már létezik.")

# 4. Új teszt: többkörös tanulás makró-átemeléssel
test_content = r'''
use athlesia_kernel::Agent;
use athlesia_knowledge::KnowledgeBase;
use athlesia_memory::Memory;
use athlesia_types::{Grid, PrimName, Params, Budget};
use athlesia_executor::run_program;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn consolidates_learned_macro_for_next_level() {
    let start = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let mut agent = Agent::new(start.clone());
    let mut kb = KnowledgeBase::new();
    let mut memory = Memory::new();

    // A környezet szabálya: jobbra tolás (Translate(1,0))
    let rule = vec![(PrimName::Translate, Params::Translate(1, 0))];

    // Első kör: tanulás és megerősítés
    let mut current = start;
    for _ in 0..5 {
        let _action = agent.step(&current, None);
        let mut budget = Budget { max_steps: 1 };
        let next = run_program(&rule, &current, &mut budget).unwrap();
        agent.update(&current, &next);
        current = next;
    }

    // Ellenőrizzük, hogy a hipotézis megerősített
    assert!(agent.wm.hypotheses.iter().any(|h| h.status == athlesia_world_model::HypothesisStatus::Confirmed));

    // Makrók konszolidálása a tudásbázisba
    agent.consolidate_learned_macros(&mut kb, &mut memory);

    assert!(kb.get_all_macros().len() > 0, "Legalább egy makrónak be kell kerülnie a tudásbázisba");
    assert!(memory.get_known_programs().len() > 0, "A memóriába is be kell kerülnie a megtanult szabálynak");
}
'''
write_file(
    os.path.join(PROJECT, "crates", "athlesia-kernel", "tests", "agent_macro_consolidation_test.rs"),
    test_content
)
print("[INFO] Többkörös tanulás teszt hozzáadva.")

# 5. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-kernel", "--test", "agent_macro_consolidation_test"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Agent makró konszolidáció teszt nem ment át.")
    sys.exit(1)
print("\n[SUCCESS] Agent makró konszolidáció teszt zöld.")

# 6. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add macro consolidation for multi-level learning"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")

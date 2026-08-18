#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. kernel lib.rs beolvasása
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

# 2. Import hozzáadása az AbstractionEngine-hez
import_marker = "use athlesia_core::CoreEngine;"
if "use athlesia_abstraction::AbstractionEngine;" not in s:
    s = s.replace(import_marker, import_marker + "\nuse athlesia_abstraction::AbstractionEngine;")

# 3. Agent struct implementációjának bővítése: abstract_from_episodes metódus
agent_impl_start = "impl Agent {"
agent_impl_end = "/// Teljes megoldási pipeline egy ARC feladat JSON-re."
start_idx = s.find(agent_impl_start)
end_idx = s.find(agent_impl_end)
if start_idx == -1 or end_idx == -1:
    print("[ERROR] Az Agent impl blokk vagy a solve_arc_json nem található.")
    sys.exit(1)

# Új metódus, amit az Agent impl elejére szúrunk be
new_method = '''    /// Az epizodikus memóriában lévő programokból absztrakció kinyerése.
    /// Az AbstractionEngine a gyakori programmintákat makrókká emeli,
    /// és a tudásbázisba valamint a CoreEngine ismert programjai közé teszi.
    pub fn abstract_from_episodes(&mut self) {
        let solved_programs: Vec<Program> = self
            .memory
            .episodic
            .iter()
            .map(|ep| ep.program.clone())
            .collect();

        if solved_programs.len() < 2 {
            return;
        }

        let added = AbstractionEngine::extract_macros(&solved_programs, &mut self.kb, 2);
        if added > 0 {
            // Az új makrókat a CoreEngine ismert programjaihoz adjuk,
            // hogy a későbbi megoldások során használhatók legyenek.
            let macros: Vec<Program> = self
                .kb
                .get_all_macros()
                .iter()
                .map(|m| m.program.clone())
                .collect();
            for program in macros {
                if !self.core.known_programs.contains(&program) {
                    self.core.known_programs.push(program);
                }
            }
        }
    }

'''
s = s[:start_idx] + agent_impl_start + "\n" + new_method + s[start_idx + len(agent_impl_start):]

write_file(p, s)
print("[1] abstract_from_episodes metódus hozzáadva az Agent-hez.")

# 4. solve_arc_json végére hívás beillesztése
old_return = "    (predicted, test_expected)\n}"
new_return = "    agent.abstract_from_episodes();\n\n    (predicted, test_expected)\n}"
count = s.count(old_return)
if count != 1:
    print(f"[ERROR] A solve_arc_json vége nem egyértelmű (előfordulás: {count}).")
    sys.exit(1)
s = s.replace(old_return, new_return)
write_file(p, s)
print("[2] solve_arc_json végén abstract_from_episodes hívása.")

# 5. Új teszt létrehozása
test_code = r'''
use athlesia_kernel::{Agent, grid_from_rows};
use athlesia_types::{PrimName, Params};

#[test]
fn abstraction_from_episodes_adds_macro() {
    let mut agent = Agent::new(grid_from_rows(&vec![vec![0; 2]; 2]));

    let program = vec![(
        PrimName::BlockMap,
        Params::BlockMap(2, 2, vec![0, 0, 0, 0]),
    )];

    // Két példa ugyanarra a BlockMap programra
    agent.memory.append_episode(
        grid_from_rows(&vec![vec![1, 2], vec![3, 4]]),
        grid_from_rows(&vec![
            vec![1, 2, 1, 2],
            vec![3, 4, 3, 4],
            vec![1, 2, 1, 2],
            vec![3, 4, 3, 4],
        ]),
        program.clone(),
    );
    agent.memory.append_episode(
        grid_from_rows(&vec![vec![5, 6], vec![7, 8]]),
        grid_from_rows(&vec![
            vec![5, 6, 5, 6],
            vec![7, 8, 7, 8],
            vec![5, 6, 5, 6],
            vec![7, 8, 7, 8],
        ]),
        program.clone(),
    );

    agent.abstract_from_episodes();

    // Ellenőrizzük, hogy a makró bekerült a KB-be
    assert!(
        agent.kb.get_all_macros().iter().any(|m| m.program == program),
        "A makró nem található a tudásbázisban"
    );

    // Ellenőrizzük, hogy a CoreEngine ismeri a makrót
    assert!(agent.core.known_programs.contains(&program));
}
'''
write_file("crates/athlesia-kernel/tests/abstraction_learning_test.rs", test_code)
print("[3] abstraction_learning_test.rs létrehozva.")

# 6. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Kernel tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Phase 6 tesztek zöldek.")

# 7. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 6: abstract macros from episodic memory via AbstractionEngine"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")

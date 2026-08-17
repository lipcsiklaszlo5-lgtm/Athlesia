#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Típusok: Budget kiegészítése max_depth-tel, ExecError bővítése
p = pathlib.Path("crates/athlesia-types/src/lib.rs")
s = p.read_text()

old_budget = '''#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub max_steps: u64,
}'''
new_budget = '''#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub max_steps: u64,
    pub max_depth: u32,
}'''
if old_budget in s:
    s = s.replace(old_budget, new_budget)
else:
    print("[ERROR] Budget struct nem található")
    sys.exit(1)

old_error = '''#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError {
    BudgetExceeded,
}'''
new_error = '''#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError {
    BudgetExceeded,
    DepthExceeded,
}'''
if old_error in s:
    s = s.replace(old_error, new_error)
else:
    print("[ERROR] ExecError enum nem található")
    sys.exit(1)

write_file(p, s)
print("[1] Types Budget és ExecError bővítve.")

# 2. Executor run_program frissítése a max_depth ellenőrzéssel
p = pathlib.Path("crates/athlesia-executor/src/lib.rs")
s = p.read_text()

old_run = '''pub fn run_program(program: &Program, input: &Grid, budget: &mut Budget) -> Result<Grid, ExecError> {
    let mut current = input.clone();

    for (name, params) in program {
        if budget.max_steps == 0 {
            return Err(ExecError::BudgetExceeded);
        }
        current = apply_primitive(&current, name, params);
        budget.max_steps -= 1;
    }

    Ok(current)
}'''
new_run = '''pub fn run_program(program: &Program, input: &Grid, budget: &mut Budget) -> Result<Grid, ExecError> {
    if budget.max_depth == 0 {
        return Err(ExecError::DepthExceeded);
    }

    let mut current = input.clone();

    for (i, (name, params)) in program.iter().enumerate() {
        if budget.max_steps == 0 {
            return Err(ExecError::BudgetExceeded);
        }
        if i as u32 >= budget.max_depth {
            return Err(ExecError::DepthExceeded);
        }
        current = apply_primitive(&current, name, params);
        budget.max_steps -= 1;
    }

    Ok(current)
}'''
if old_run in s:
    s = s.replace(old_run, new_run)
else:
    print("[ERROR] run_program blokk nem található")
    sys.exit(1)

# Az összes Budget { max_steps: X } literált frissíteni kell a kódban
# Ezt a tesztfájlokban és a lib.rs-ben is meg kell tenni, de most csak az executorban.
s = s.replace("Budget { max_steps: 1 }", "Budget { max_steps: 1, max_depth: 100 }")
s = s.replace("Budget { max_steps: d as u64 }", "Budget { max_steps: d as u64, max_depth: 100 }")
s = s.replace("Budget { max_steps: 0 }", "Budget { max_steps: 0, max_depth: 100 }")

write_file(p, s)
print("[2] Executor run_program frissítve.")

# 3. Tesztek frissítése a Budget új mezőjével
for test_file in pathlib.Path("crates").rglob("tests/*.rs"):
    content = test_file.read_text()
    if "Budget { max_steps:" in content:
        # Hozzáadjuk a max_depth-et, ha hiányzik
        content = content.replace("Budget { max_steps: 1 }", "Budget { max_steps: 1, max_depth: 100 }")
        content = content.replace("Budget { max_steps: 1000 }", "Budget { max_steps: 1000, max_depth: 100 }")
        content = content.replace("Budget { max_steps: 1, max_depth: 100 }", "Budget { max_steps: 1, max_depth: 100 }")
        write_file(test_file, content)
        print(f"[3] Teszt frissítve: {test_file}")

# 4. Executor tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-executor"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Executor tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Executor tesztek zöldek.")

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize Execution Engine with Budget max_depth and DepthExceeded"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")

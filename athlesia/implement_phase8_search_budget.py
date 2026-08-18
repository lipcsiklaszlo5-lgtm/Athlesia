#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Search crate bővítése
p = pathlib.Path("crates/athlesia-search/src/lib.rs")
s = p.read_text()

# SearchTelemetry és SearchDecision definíciók a fájl elejére
if "pub struct SearchTelemetry" not in s:
    telemetry_def = r'''
/// Keresési telemetria: a kereső futásának élő adatai.
#[derive(Debug, Clone)]
pub struct SearchTelemetry {
    pub nodes_expanded: u64,
    pub current_depth: u32,
    pub branching_factor: f32,
    pub best_score: f32,
    pub previous_best_score: f32,
    pub score_delta: f32,
    pub hypotheses_tested: u64,
    pub high_confidence_hits: u64,
    pub estimated_remaining_cost: f32,
    // Belső állapot az abort döntéshez
    stagnation_counter: u32,
    patience_window: u32,
    max_possible_score: f32,
}

/// Keresési döntés: folytatás, metszés vagy leállítás.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDecision {
    Continue,
    Prune,
    Abort,
}

impl SearchTelemetry {
    /// Új telemetria adott maximális pontszámmal (cellaegyezések maximuma).
    pub fn new(max_possible_score: f32) -> Self {
        Self {
            nodes_expanded: 0,
            current_depth: 0,
            branching_factor: 0.0,
            best_score: 0.0,
            previous_best_score: 0.0,
            score_delta: 0.0,
            hypotheses_tested: 0,
            high_confidence_hits: 0,
            estimated_remaining_cost: 0.0,
            stagnation_counter: 0,
            patience_window: 20,
            max_possible_score,
        }
    }

    /// Egy csomópont kiterjesztésének rögzítése.
    /// - `depth`: az aktuális mélység
    /// - `score`: az aktuális csomópont pontszáma (cellaegyezés)
    /// - `candidates_count`: a kiterjesztésből származó gyermekek száma
    /// - `remaining_cost`: becsült hátralévő költség (pl. mismatch)
    pub fn record_expansion(&mut self, depth: u32, score: f32, candidates_count: usize, remaining_cost: f32) {
        self.nodes_expanded += 1;
        self.hypotheses_tested += 1;
        self.current_depth = depth;

        // Átlagos elágazási tényező frissítése
        let n = self.nodes_expanded as f32;
        self.branching_factor = if n > 1.0 {
            (self.branching_factor * (n - 1.0) + candidates_count as f32) / n
        } else {
            candidates_count as f32
        };

        self.previous_best_score = self.best_score;
        self.best_score = self.best_score.max(score);
        self.score_delta = self.best_score - self.previous_best_score;

        self.estimated_remaining_cost = remaining_cost;

        // Magas konfidencia találat, ha a pontszám közel van a maximumhoz
        if self.max_possible_score > 0.0 && score > 0.9 * self.max_possible_score {
            self.high_confidence_hits += 1;
        }

        // Stagnálás számláló frissítése
        if self.score_delta <= 0.0 && self.nodes_expanded > 1 {
            self.stagnation_counter += 1;
        } else {
            self.stagnation_counter = 0;
        }
    }

    /// Döntés, hogy a keresést le kell-e állítani.
    /// `Abort` akkor, ha a legjobb pontszám tartósan nem javul.
    pub fn should_abort(&self) -> bool {
        self.stagnation_counter >= self.patience_window
    }

    /// A jelenlegi keresési döntés lekérdezése.
    pub fn decision(&self) -> SearchDecision {
        if self.should_abort() {
            SearchDecision::Abort
        } else if self.score_delta <= 0.0 && self.nodes_expanded > 1 {
            SearchDecision::Prune
        } else {
            SearchDecision::Continue
        }
    }
}
'''
    s = telemetry_def + s

# search_with_budget hozzáadása a score_grid után
if "pub fn search_with_budget" not in s:
    search_budget_fn = r'''

/// Keresés költségvetéssel és telemetriával.
/// A* keresést végez, gyűjti a telemetriát, és abortál, ha a fejlődés tartósan megáll.
pub fn search_with_budget(
    input: &Grid,
    target: &Grid,
    max_depth: usize,
    telemetry: &mut SearchTelemetry,
) -> Option<Program> {
    use std::collections::BinaryHeap;
    use std::cmp::Ordering;

    #[derive(Debug, Clone)]
    struct Node {
        program: Program,
        grid: Grid,
        depth: usize,
        f_score: usize,
    }

    impl PartialEq for Node {
        fn eq(&self, other: &Self) -> bool {
            self.f_score == other.f_score && self.program == other.program
        }
    }
    impl Eq for Node {}
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            other.f_score.cmp(&self.f_score)
        }
    }

    let total_cells = (target.width as usize) * (target.height as usize);
    let max_possible_score = total_cells as f32;

    fn mismatch_count(grid: &Grid, target: &Grid) -> usize {
        if grid.width != target.width || grid.height != target.height {
            return usize::MAX;
        }
        let mut mismatch = 0;
        for i in 0..grid.height as usize {
            for j in 0..grid.width as usize {
                let idx = i * grid.width as usize + j;
                let tidx = i * target.width as usize + j;
                if grid.cells[idx] != target.cells[tidx] {
                    mismatch += 1;
                }
            }
        }
        mismatch
    }

    let initial_grid = input.clone();
    let initial_program = Vec::new();
    let initial_mismatch = mismatch_count(&initial_grid, target);
    let initial_score = score_grid(&initial_grid, target);
    let mut heap = BinaryHeap::new();
    heap.push(Node {
        program: initial_program,
        grid: initial_grid,
        depth: 0,
        f_score: initial_mismatch,
    });

    while let Some(node) = heap.pop() {
        if telemetry.should_abort() {
            return None;
        }
        if node.grid == *target {
            return Some(node.program);
        }

        let score = score_grid(&node.grid, target) as f32;
        let remaining = mismatch_count(&node.grid, target) as f32;
        let candidates_count = candidate_primitives(input, target).len();
        telemetry.record_expansion(node.depth as u32, score, candidates_count, remaining);

        if node.depth >= max_depth {
            continue;
        }

        for (prim, params) in candidate_primitives(input, target) {
            let mut new_program = node.program.clone();
            new_program.push((prim, params));
            let mut budget = Budget { max_steps: new_program.len() as u64, max_depth: 100 };
            if let Ok(new_grid) = run_program(&new_program, input, &mut budget) {
                let new_depth = node.depth + 1;
                let new_mismatch = mismatch_count(&new_grid, target);
                let new_f = new_depth + new_mismatch;
                heap.push(Node {
                    program: new_program,
                    grid: new_grid,
                    depth: new_depth,
                    f_score: new_f,
                });
            }
        }
    }
    None
}
'''
    # A score_grid függvény után szúrjuk be
    anchor = "fn score_grid(grid: &Grid, target: &Grid) -> usize {"
    idx = s.find(anchor)
    if idx == -1:
        print("[ERROR] score_grid függvény nem található.")
        sys.exit(1)
    # Keressük meg a score_grid végét (a záró kapcsos zárójel előtt beszúrva)
    end_idx = s.find("}\n", idx) + 1  # a záró }
    s = s[:end_idx] + search_budget_fn + s[end_idx:]

write_file(p, s)
print("[1] Search crate bővítve: SearchTelemetry és search_with_budget hozzáadva.")

# 2. CoreEngine módosítása
p = pathlib.Path("crates/athlesia-core/src/lib.rs")
s = p.read_text()

# Import cseréje
old_import = "use athlesia_search::search;"
new_import = "use athlesia_search::{search_with_budget, SearchTelemetry};"
if old_import not in s:
    print("[ERROR] search import nem található.")
    sys.exit(1)
s = s.replace(old_import, new_import)

# 3. lépés (keresés) cseréje
old_search_block = '''        // 3. Ha a szintézis nem járt sikerrel, próbáljuk a többlépéses keresést
        if let Some(program) = search(input, target, 3) {
            steps += 1;
            if self.verifier.verify(&program, &vec![(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return (Some(program), steps);
            }
        }
'''
new_search_block = '''        // 3. Ha a szintézis nem járt sikerrel, próbáljuk a többlépéses keresést
        let max_score = (target.width as usize * target.height as usize) as f32;
        let mut telemetry = SearchTelemetry::new(max_score);
        if let Some(program) = search_with_budget(input, target, 3, &mut telemetry) {
            steps += telemetry.hypotheses_tested as usize;
            if self.verifier.verify(&program, &vec![(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                self.meta.record_search_cost_in_context(fv, id, telemetry.hypotheses_tested as f64);
                return (Some(program), steps);
            }
        }
        // Akár talált, akár nem, rögzítsük a keresés költségét a 0-s hipotézishez
        self.meta.record_search_cost_in_context(fv, 0, telemetry.hypotheses_tested as f64);
'''
if old_search_block not in s:
    print("[ERROR] Keresési blokk nem található a CoreEngine-ben.")
    sys.exit(1)
s = s.replace(old_search_block, new_search_block)

write_file(p, s)
print("[2] CoreEngine frissítve: search_with_budget használata és költségtanulás.")

# 3. Tesztek a search crate-hez
search_test = r'''
use athlesia_search::{search_with_budget, SearchTelemetry};
use athlesia_types::{Grid, Color};

fn make_grid(rows: Vec<Vec<u8>>) -> Grid {
    let height = rows.len() as u8;
    let width = if height > 0 { rows[0].len() as u8 } else { 0 };
    let mut cells = Vec::new();
    for row in &rows {
        for &c in row {
            cells.push(Color(c));
        }
    }
    Grid { width, height, cells }
}

#[test]
fn search_with_budget_aborts_when_stuck() {
    // Olyan cél, amely nem érhető el a primitívekkel, hogy a keresés elakadjon.
    let input = make_grid(vec![vec![1, 0], vec![0, 0]]);
    let target = make_grid(vec![vec![2, 3], vec![4, 5]]);

    let max_score = (target.width as usize * target.height as usize) as f32;
    let mut telemetry = SearchTelemetry::new(max_score);
    let result = search_with_budget(&input, &target, 3, &mut telemetry);

    assert!(result.is_none(), "Nem kellett volna megoldást találni");
    assert!(telemetry.should_abort(), "A keresésnek le kellett volna állnia");
    assert!(telemetry.hypotheses_tested > 0, "Telemetriának gyűjtenie kell adatot");
}
'''
write_file("crates/athlesia-search/tests/budget_abort_test.rs", search_test)
print("[3] budget_abort_test.rs létrehozva.")

# 4. Teszt a CoreEngine költségtanulásához
core_test = r'''
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, Color};
use athlesia_features::FeatureVector;

fn make_grid(rows: Vec<Vec<u8>>) -> Grid {
    let height = rows.len() as u8;
    let width = if height > 0 { rows[0].len() as u8 } else { 0 };
    let mut cells = Vec::new();
    for row in &rows {
        for &c in row {
            cells.push(Color(c));
        }
    }
    Grid { width, height, cells }
}

#[test]
fn core_learns_search_cost_after_failed_search() {
    let mut engine = CoreEngine::new();
    let input = make_grid(vec![vec![1, 0], vec![0, 0]]);
    let target = make_grid(vec![vec![2, 3], vec![4, 5]]);

    let fv = FeatureVector::default();
    let (result, _steps) = engine.solve_with_steps(&input, &target);
    assert!(result.is_none(), "Nincs megoldás erre a feladatra");

    // A keresési költségnek rögzítve kell lennie a meta-learnerben
    let cost = engine.meta.estimated_cost(fv, 0);
    assert!(cost.is_some(), "A keresési költséget meg kellett tanulni");
    assert!(cost.unwrap() > 0.0, "A költség nagyobb mint nulla");
}
'''
write_file("crates/athlesia-core/tests/search_cost_learning_test.rs", core_test)
print("[4] search_cost_learning_test.rs létrehozva.")

# 5. Search crate teszt futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-search"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Search tesztek nem mentek át.")
    sys.exit(1)

# 6. Core crate teszt futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)

# 7. Kernel tesztek futtatása
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

print("\n[SUCCESS] Phase 8 tesztek zöldek.")

# 8. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 8: search budget with telemetry and early abort; core learns search cost"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")

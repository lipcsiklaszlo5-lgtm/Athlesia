#!/usr/bin/env python3
import os, re, sys, subprocess
from pathlib import Path

# 1. Típus: Recolor 10 színre
types_path = Path("crates/athlesia-types/src/lib.rs")
types = types_path.read_text()
types = types.replace("    Recolor([Color; 4]),", "    Recolor([Color; 10]),")
types_path.write_text(types)
print("[1] Recolor 10 eleműre bővítve a types-ben.")

# 2. Minden Rust fájlban a Recolor 4 elemből 10 elemmé bővítése (identitás kiegészítés)
pattern = re.compile(r'(Params::Recolor\(\s*\[)([^\]]*?)(\]\s*\))', re.DOTALL)

for rs_file in Path("crates").rglob("*.rs"):
    content = rs_file.read_text()
    def repl(match):
        inner = match.group(2)
        elems = re.findall(r'Color\(\s*(\d+)\s*\)', inner)
        if len(elems) == 10:
            return match.group(0)
        if len(elems) == 4:
            extended = [int(e) for e in elems]
            for c in range(4, 10):
                extended.append(c)
            new_inner = ', '.join(f'Color({c})' for c in extended)
            return f'{match.group(1)}{new_inner}{match.group(3)}'
        return match.group(0)
    new_content = pattern.sub(repl, content)
    if new_content != content:
        rs_file.write_text(new_content)
        print(f"[2] Recolor bővítve: {rs_file}")

# 3. Search candidate_primitives frissítése új primitívekkel
search_path = Path("crates/athlesia-search/src/lib.rs")
search = search_path.read_text()
old_fn = search.find("fn candidate_primitives()")
if old_fn != -1:
    # Teljes függvény cseréje
    fn_start = old_fn
    fn_end = search.find("\n}\n", fn_start) + 3
    new_fn = '''fn candidate_primitives() -> Vec<(PrimName, Params)> {
    let mut v = Vec::new();

    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0), (0, 0)] {
        v.push((PrimName::Translate, Params::Translate(dx, dy)));
    }

    v.push((PrimName::ReflectH, Params::None));
    v.push((PrimName::ReflectV, Params::None));
    v.push((PrimName::Rotate90, Params::None));
    v.push((PrimName::Rotate180, Params::None));
    v.push((PrimName::Rotate270, Params::None));

    v.push((PrimName::SwapColors, Params::SwapColors(1, 2)));
    v.push((PrimName::SwapColors, Params::SwapColors(1, 3)));
    v.push((PrimName::SwapColors, Params::SwapColors(2, 3)));

    v.push((PrimName::TranslateWrap, Params::TranslateWrap(1, 0)));
    v.push((PrimName::TranslateWrap, Params::TranslateWrap(0, 1)));
    v.push((PrimName::TranslateWrap, Params::TranslateWrap(-1, 0)));
    v.push((PrimName::TranslateWrap, Params::TranslateWrap(0, -1)));

    let identity: [Color; 10] = [
        Color(0), Color(1), Color(2), Color(3), Color(4),
        Color(5), Color(6), Color(7), Color(8), Color(9),
    ];
    v.push((PrimName::Recolor, Params::Recolor(identity)));

    let swap12: [Color; 10] = [
        Color(0), Color(2), Color(1), Color(3), Color(4),
        Color(5), Color(6), Color(7), Color(8), Color(9),
    ];
    v.push((PrimName::Recolor, Params::Recolor(swap12)));

    let swap13: [Color; 10] = [
        Color(0), Color(3), Color(2), Color(1), Color(4),
        Color(5), Color(6), Color(7), Color(8), Color(9),
    ];
    v.push((PrimName::Recolor, Params::Recolor(swap13)));

    v
}\n'''
    search = search[:fn_start] + new_fn + search[fn_end:]
    search_path.write_text(search)
    print("[3] Search candidate_primitives frissítve.")
else:
    print("[WARN] candidate_primitives nem található a search-ben.")

# 4. Synthesis Recolor blokk frissítése
synthesis_path = Path("crates/athlesia-synthesis/src/lib.rs")
synthesis = synthesis_path.read_text()
old_block = synthesis.find("PrimitiveTemplate::Recolor => {")
if old_block != -1:
    start = old_block
    end = synthesis.find("\n        }", start) + len("\n        }")
    new_block = '''PrimitiveTemplate::Recolor => {
            let mut v = Vec::new();
            let perms: [[Color; 10]; 4] = [
                [Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
                [Color(2), Color(1), Color(0), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
                [Color(3), Color(2), Color(1), Color(0), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
                [Color(1), Color(2), Color(3), Color(0), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
            ];
            for perm in perms {
                v.push((PrimName::Recolor, Params::Recolor(perm)));
            }
            v
        }'''
    synthesis = synthesis[:start] + new_block + synthesis[end:]
    synthesis_path.write_text(synthesis)
    print("[4] Synthesis Recolor frissítve.")
else:
    print("[WARN] Synthesis Recolor blokk nem található.")

# 5. Hypothesis Recolor példa frissítése
hyp_path = Path("crates/athlesia-hypothesis/src/lib.rs")
hyp = hyp_path.read_text()
hyp = hyp.replace(
    "PrimName::Recolor => vec![(PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3)]))],",
    "PrimName::Recolor => vec![(PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)]))],"
)
hyp_path.write_text(hyp)
print("[5] Hypothesis Recolor frissítve.")

# 6. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-kernel", "--test", "arc_multi_test", "--", "--nocapture"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Az arc_multi_test nem ment át.")
    sys.exit(1)
print("\n[SUCCESS] Az arc_multi_test zöld.")

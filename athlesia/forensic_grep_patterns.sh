#!/bin/bash
echo "=== 1. structural_match küszöb ==="
grep -R "structural_match" crates/athlesia-kernel/src crates/athlesia-kernel/tests 2>/dev/null

echo ""
echo "=== 2. patience_window ==="
grep -R "patience_window" crates/athlesia-search/src crates/athlesia-search/tests 2>/dev/null

echo ""
echo "=== 3. gain <= / gain < ==="
grep -R "gain" crates/athlesia-abstraction/src crates/athlesia-abstraction/tests 2>/dev/null | head -20

echo ""
echo "=== 4. hidden trigger ==="
grep -R "hidden" crates/athlesia-interactive/src crates/athlesia-interactive/tests 2>/dev/null

echo ""
echo "=== 5. expand_hypotheses / sequence generation ==="
grep -R "expand_hypotheses\|generate_length\|sequence" crates 2>/dev/null | head -30

echo ""
echo "=== 6. openworld / phase13 nyomai ==="
find crates -name "*openworld*" -o -name "*phase13*" 2>/dev/null

echo ""
echo "=== 7. BlockMap / BlockMap template ==="
grep -R "BlockMap" crates/athlesia-kernel/src crates/athlesia-core/src crates/athlesia-synthesis/src 2>/dev/null | head -20

echo ""
echo "=== 8. gold / ground truth / oracle ==="
grep -R "gold\|ground_truth\|oracle\|expected_output" crates 2>/dev/null | head -20

echo ""
echo "=== 9. task_id / task identifier ==="
grep -R "task_id\|task_name\|ArcTask" crates/athlesia-kernel/src crates/athlesia-kernel/tests 2>/dev/null | head -20

echo ""
echo "=== 10. hamis implementációra utaló jelek ==="
grep -R "unwrap()\|todo!()\|unimplemented!()" crates 2>/dev/null | head -20

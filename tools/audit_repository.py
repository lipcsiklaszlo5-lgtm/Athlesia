#!/usr/bin/env python3
"""Read-only Athlesia repository audit. Python 3.11+, standard library only."""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ACTIVE = {
    'athlesia_mindstone_sparse_cognition',
    'athlesia_core_knowledge_perceptual_grounding',
    'athlesia_universal_domain_learning',
    'athlesia_autonomous_active_experimentation',
    'athlesia_executive_agency',
    'athlesia_meta_learning_skill_memory',
    'athlesia_autonomous_cognitive_self_bootstrap',
    'athlesia_integrated_cognitive_agent',
    'athlesia_arc_agi_3_adapter',
    'athlesia_arc_agi_3_blind_benchmark',
}


def git(*args: str) -> str:
    return subprocess.check_output(['git', '-C', str(ROOT), *args], text=True, stderr=subprocess.DEVNULL)


def audit() -> dict:
    warnings: list[str] = []
    errors: list[str] = []
    inventory_path = ROOT / 'tools/repo_manifests.txt'
    manifests = [v for v in inventory_path.read_text().splitlines() if v.strip()]
    if len(manifests) != 33 or manifests != sorted(set(manifests)):
        errors.append(f'Manifest inventory expected 33 sorted unique entries; found {len(manifests)}')
    entries = []
    for relative in manifests:
        filename = ROOT / relative
        if not filename.is_file():
            errors.append(f'Missing manifest: {relative}')
            continue
        try:
            with filename.open('rb') as fh:
                parsed = tomllib.load(fh)
        except Exception as exc:
            errors.append(f'Cannot parse {relative}: {exc}')
            continue
        name = parsed.get('package', {}).get('name')
        if not name:
            errors.append(f'Missing package.name: {relative}')
            continue
        dependency_names = sorted(
            name for name, value in parsed.get('dependencies', {}).items()
            if isinstance(value, dict) and 'path' in value
        )
        entries.append({
            'manifest': relative,
            'name': name,
            'line': 'active' if name in ACTIVE else 'historical_or_frozen',
            'path_dependencies': dependency_names,
        })
    seen = {e['name'] for e in entries}
    for entry in entries:
        for dependency in entry['path_dependencies']:
            if dependency not in seen:
                warnings.append(f"Undeclared-in-inventory path dependency {entry['name']} -> {dependency}")
    missing_active = sorted(ACTIVE - seen)
    if missing_active:
        errors.append('Missing expected active packages: ' + ', '.join(missing_active))
    # Only check markdown targets whose destination is local and textually stable.
    for relative in ['README.md', 'CONTRIBUTING.md', 'docs/ARCHITECTURE.md',
                     'docs/ROADMAP.md', 'docs/REPO_MAP.md', 'docs/EVIDENCE_AND_LIMITS.md',
                     'docs/RESEARCH.md']:
        p = ROOT / relative
        if not p.is_file():
            errors.append(f'Missing public navigation file: {relative}')
            continue
        for raw in re.findall(r'(?<!!)\[[^\]]+\]\(([^)]+)\)', p.read_text(encoding='utf-8')):
            link = raw.split('#', 1)[0].strip()
            if not link or link.startswith(('https:', 'http:', 'mailto:', 'data:')):
                continue
            if link.startswith('/'):
                warnings.append(f'Absolute local link in {relative}: {link}')
                continue
            if not (p.parent / link).exists():
                errors.append(f'Broken local link: {relative} -> {raw}')
    root_files = sorted(x.name for x in ROOT.iterdir() if x.is_file())
    untracked = git('ls-files', '--others', '--exclude-standard').splitlines()
    e11 = ROOT / 'experiments/e11_continuous_affordance_monitor'
    if e11.exists() and not git('ls-files', '--', 'experiments/e11_continuous_affordance_monitor').strip():
        warnings.append('E11 exists locally but is not tracked: preserve and review separately')
    scratch = [x for x in untracked if x.startswith('athlesia_') and x.endswith('.sh')]
    if scratch:
        warnings.append(f'{len(scratch)} local root pilot/finalizer script(s): no deletion or relocation performed')
    oversized = sorted(({'path': path, 'bytes': (ROOT / path).stat().st_size}
                        for path in root_files if (ROOT / path).stat().st_size >= 40000),
                       key=lambda x: -x['bytes'])
    status = git('status', '--short').splitlines()
    return {
        'head': git('rev-parse', '--short', 'HEAD').strip(),
        'branch': git('branch', '--show-current').strip(),
        'manifest_count': len(manifests),
        'active_count': sum(e['line'] == 'active' for e in entries),
        'historical_count': sum(e['line'] != 'active' for e in entries),
        'packages': entries,
        'root_files': root_files,
        'large_root_files': oversized,
        'untracked_count': len(untracked),
        'untracked_e11': e11.exists() and not git('ls-files', '--', 'experiments/e11_continuous_affordance_monitor').strip(),
        'worktree_status': status,
        'warnings': warnings,
        'errors': errors,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--json-out', help='Write a JSON audit report; no repository changes')
    args = parser.parse_args()
    try:
        report = audit()
    except Exception as exc:
        print(f'AUDIT ERROR: {exc}', file=sys.stderr)
        return 2
    print(f"ATHLESIA REPOSITORY AUDIT | {report['branch']} @ {report['head']}")
    print(f"Inventory: {report['manifest_count']} manifests | {report['active_count']} active | {report['historical_count']} historical/frozen")
    print(f"Working tree: {len(report['worktree_status'])} changes including {report['untracked_count']} untracked files")
    for issue in report['warnings']:
        print(f'WARN: {issue}')
    for issue in report['errors']:
        print(f'ERROR: {issue}')
    if args.json_out:
        path = Path(args.json_out)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
        print(f'JSON: {path}')
    return 1 if report['errors'] else 0


if __name__ == '__main__':
    raise SystemExit(main())

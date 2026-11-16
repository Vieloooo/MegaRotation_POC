import hashlib
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

root = Path('/home/jy/.codex/worktrees/1c43/megaroation_poc')
out = Path('/tmp/commander-feature03-acceptance')
sha = 'e9c15e04bc83b335668f3ecd837367aa87344a89'
vdir = root / '.build/artifacts/verification/full03' / sha
tdir = root / '.build/artifacts/simulation' / sha
v = json.loads((vdir / 'run-manifest.json').read_text())
t = json.loads((tdir / 'run-manifest.json').read_text())
checks = {}

def h(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def record(k, value):
    checks[k] = bool(value)

def git(*args):
    return subprocess.check_output(['git', *args], cwd=root)

record('candidate_head', git('rev-parse', 'HEAD').decode().strip() == sha)
record('candidate_clean', git('status', '--porcelain') == b'')
record('same_sha_verifier', v['candidate_sha'] == sha and v['head_after'] == sha and v['return_code'] == 0 and not v['git_status_before'] and not v['git_status_after'])
record('same_sha_tester', t['candidate_sha'] == sha and t['git_before']['head'] == sha and t['git_after']['head'] == sha and not t['git_before']['porcelain'] and not t['git_after']['porcelain'])
for name, digest in v['files'].items():
    record('verifier_artifact:' + name, h(vdir / name) == digest)
for name, info in t['artifacts'].items():
    record('tester_artifact:' + name, h(info['path']) == info['sha256'] and Path(info['path']).stat().st_size == info['bytes'])
record('tester_links_verified_manifest', h(t['verified_same_sha_manifest']) == t['verified_same_sha_manifest_sha256'])
record('v2_source_pin', h(root / 'formalization_v10/poc_design_zh_v2.md') == '95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab')
record('contract_pin', h(root / 'poc/docs/contracts/task-contract-mock-plan-execution-v2.md') == v['source_contract_sha256'] == t['contract_sha256'])
record('lock_pin', h(root / 'poc/Cargo.lock') == t['cargo_lock_sha256'])
record('bft_gitlink', git('ls-tree', 'HEAD', 'regional_BFT_sim').decode().split()[2] == '691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6')
record('review_to_seal_no_production_diff', git('diff', 'e30f5ed3b3352d58005f672bfebf22e9810552fb', sha, '--', 'poc/crates', 'poc/Cargo.toml', 'poc/Cargo.lock', 'poc/proto', 'poc/vendor', 'regional_BFT_sim') == b'')
record('proto_vendor_unchanged', git('diff', 'ada48a584ad4de15e302c479ee8cd41b9e2a8d9b', sha, '--', 'poc/proto', 'poc/vendor', 'regional_BFT_sim') == b'')
lock = tomllib.loads((root / 'poc/Cargo.lock').read_text())
cw = [p for p in lock['package'] if p['name'].startswith('commonware-')]
record('all_commonware_2026_10_0', all(p['version'] == '2026.10.0' for p in cw))
record('strict_edwards_pin', any(p['name'] == 'curve25519-dalek' and p['version'] == '5.0.0' for p in lock['package']))
vecdir = root / 'poc/verification/mock-plan-execution/implementation-v1'
for name, digest in v['expected_vector_sha256'].items():
    record('frozen_vector:' + name, h(vecdir / name) == digest)
for name, digest in t['additional_input_sha256'].items():
    record('tester_input:' + name, h(vecdir / name) == digest)
for item in v['reference_checks']:
    script = Path(item['command_argv'][1])
    record('reference_script:' + script.name, h(script) == item['script_sha256'])
    env = os.environ.copy()
    env['PYTHONDONTWRITEBYTECODE'] = '1'
    r = subprocess.run([sys.executable, str(script)], cwd=root, capture_output=True, env=env)
    (out / (script.stem + '.stdout')).write_bytes(r.stdout)
    (out / (script.stem + '.stderr')).write_bytes(r.stderr)
    record('reference_rerun:' + script.name, r.returncode == 0 and hashlib.sha256(r.stdout).hexdigest() == item['output_sha256'])

def parse_log(p):
    rows = re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored;', p.read_text())
    return {'passed': sum(int(r[1]) for r in rows), 'failed': sum(int(r[2]) for r in rows), 'ignored': sum(int(r[3]) for r in rows), 'all_ok': all(r[0] == 'ok' for r in rows)}

vl = parse_log(vdir / 'stdout.txt')
tl = parse_log(tdir / 'workspace.stdout.txt')
record('raw_verifier_counts', vl == {'passed': 32, 'failed': 0, 'ignored': 0, 'all_ok': True})
record('raw_tester_counts', tl == {'passed': 52, 'failed': 0, 'ignored': 11, 'all_ok': True})
record('all_tester_commands_exit_zero', all(r['return_code'] == 0 for r in t['runs']))
v_components = json.loads((vdir / 'component-evidence.json').read_text())
t_components = json.loads((tdir / 'component-evidence.json').read_text())
record('component_case_keys_unique', len({case['case'] for case in v_components}) == len(v_components) == 3 and len({case['case'] for case in t_components}) == len(t_components) == 3)
# Separate test cases are not a single event trace. Compare their exact records
# by case identity, retaining the ordering of every per-case block/transaction list.
record('component_evidence_equal_by_case', {case['case']: case for case in v_components} == {case['case']: case for case in t_components})
record('no_missing_feature_cases', sum(t['feature03_test_counts'].values()) == 32 and len([p for p in t['per_case_outcomes'] if p['status'] == 'PASS']) == 52)
record('candidate_clean_after', git('status', '--porcelain') == b'')
result = {'candidate_sha': sha, 'checks': checks, 'check_count': len(checks), 'failed': [k for k, passed in checks.items() if not passed], 'verifier_manifest_sha256': h(vdir / 'run-manifest.json'), 'tester_manifest_sha256': h(tdir / 'run-manifest.json'), 'counts': {'verifier': vl, 'tester': tl}, 'commonware_packages': len(cw), 'ignored_cases': [p['test'] for p in t['per_case_outcomes'] if p['status'] != 'PASS'], 'heavy_test_rerun': False, 'reference_checks_rerun': 4, 'python': sys.version}
(out / 'evidence-audit.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: value for k, value in result.items() if k != 'checks'}, indent=2))
assert not result['failed']

#!/usr/bin/env python3
"""Capped pure/full-artifact replay. Socket controls run separately through lead."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[3]
OUT=Path(tempfile.mkdtemp(prefix='native-fresh-independent-final-'))
HERE=ROOT/'poc/verification/native-engine-fresh'
BASE=ROOT/'poc/artifacts/verification/native-engine-fresh'

def hashes():
    files=[*ROOT.glob('poc/prototypes/i1b-runtime/src/*.rs'),*ROOT.glob('poc/prototypes/i1b-runtime/src/bin/*.rs'),ROOT/'poc/prototypes/i1b-runtime/Cargo.toml',ROOT/'poc/prototypes/i1b-runtime/Cargo.lock',ROOT/'poc/proto/native.proto',ROOT/'poc/docs/contracts/task-contract-native-engine-fresh-v1.md',*HERE.glob('*.rs'),*HERE.glob('*.py')]
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(files)}

commands=[
    ('rust',['./poc/scripts/bootstrap.sh','test','--manifest-path','prototypes/i1b-runtime/Cargo.toml','--locked','--offline','--no-fail-fast','--','--test-threads=1']),
    ('seed7',['python3',str(HERE/'check.py'),str(BASE/'candidate-6686a00-seed7'),'--output',str(OUT/'seed7.json')]),
    ('paired',['python3',str(HERE/'check.py'),str(BASE/'candidate-6686a00-pairs'),'--output',str(OUT/'paired.json')]),
    ('pair-bytes',['python3',str(HERE/'verify_pairs.py'),str(BASE/'candidate-6686a00-pairs/pairs.json'),'--output',str(OUT/'pair-bytes.json')]),
    ('mutations',['python3',str(HERE/'mutations.py')]),
    ('format',['./poc/scripts/bootstrap.sh','fmt','--manifest-path','prototypes/i1b-runtime/Cargo.toml','--all','--check']),
    ('diff',['git','diff','--check']),
]
before=hashes();results=[]
for label,command in commands:
    capped=['timeout','--kill-after=5s','60s',*command]
    result=subprocess.run(capped,cwd=ROOT,capture_output=True,timeout=67)
    (OUT/(label+'.stdout')).write_bytes(result.stdout);(OUT/(label+'.stderr')).write_bytes(result.stderr)
    row={'label':label,'command':capped,'exit_code':result.returncode};results.append(row);print(json.dumps(row),flush=True)
report={'candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'output':str(OUT),'commands':results,'source_hashes_before':before,'source_hashes_after':hashes(),'binary_hashes':{name:hashlib.sha256((ROOT/'.build/target/debug'/name).read_bytes()).hexdigest() for name in ['native-hub','native-worker','native-engine-verifier']},'scope':'fresh native two-process fixture; no restart, v10 or full I1b acceptance'}
(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n');print('REPORT',OUT/'report.json',flush=True)
raise SystemExit(any(r['exit_code'] for r in results))

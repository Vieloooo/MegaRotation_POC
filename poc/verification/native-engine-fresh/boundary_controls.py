#!/usr/bin/env python3
"""Fresh actual-worker completion boundaries; no restart/native quorum claim."""
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[3]
OUT=Path(tempfile.mkdtemp(prefix='native-fresh-completion-boundaries-'))
helper=ROOT/'.build/target/debug/native-engine-verifier'
worker=ROOT/'.build/target/debug/native-worker'
template=ROOT/'poc/artifacts/verification/native-engine-fresh/candidate-6686a00-seed7/healthy/worker-0/launch-private.json'
results=[]
for mode in ['explicit-close','lease-boundary','stop-boundary']:
    path=OUT/mode
    command=[str(helper),'drive-worker',mode,str(worker),str(template),str(path)]
    process=subprocess.Popen(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    try:stdout,stderr=process.communicate(timeout=45)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid,signal.SIGKILL);stdout,stderr=process.communicate()
    path.mkdir(exist_ok=True);(path/'launcher-stdout').write_bytes(stdout);(path/'launcher-stderr').write_bytes(stderr)
    report=json.loads((path/'report.json').read_text()) if (path/'report.json').exists() else {'status':'FAIL','error':'worker authority did not produce a report'}
    report.update(command=command,process_exit=process.returncode,artifact=str(path));results.append(report);print(json.dumps(report),flush=True)
(OUT/'report.json').write_text(json.dumps({'artifact':str(OUT),'results':results},indent=2)+'\n')
print('REPORT',OUT/'report.json',flush=True)
raise SystemExit(any(r['status']!='PASS' or r['process_exit']!=0 for r in results))

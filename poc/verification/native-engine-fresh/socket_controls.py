#!/usr/bin/env python3
"""Transport-only adversarial worker controls; these are NOT native quorum runs."""
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import tempfile
import argparse

ROOT=Path(__file__).resolve().parents[3]
OUT=Path(tempfile.mkdtemp(prefix='native-fresh-socket-controls-'))
bin_dir=OUT/'bin';bin_dir.mkdir()
hub=ROOT/'.build/target/debug/native-hub';worker=ROOT/'.build/target/debug/native-worker';helper=ROOT/'.build/target/debug/native-engine-verifier'
shutil.copy2(hub,bin_dir/'native-hub')
wrapper=bin_dir/'native-worker'
wrapper.write_text('#!/bin/sh\nexec '+shlex.quote(str(helper))+' fake-worker "$NATIVE_FRESH_FIXTURE_MODE" "$1" "$2" '+shlex.quote(str(worker))+'\n')
wrapper.chmod(0o755)
results=[]
parser=argparse.ArgumentParser();parser.add_argument('--modes',nargs='+');options=parser.parse_args()
for mode in options.modes or ['invalid-atomic','unsolicited','altered-duplicate','exact-duplicate','frame-exact','frame-over','payload-exact','payload-over']:
    case=OUT/mode;case.mkdir();run=case/'run'
    command=[str(bin_dir/'native-hub'),'--scenario','two-real-alone','--seed','7','--output',str(run)]
    environment=dict(os.environ,NATIVE_FRESH_FIXTURE_MODE=mode)
    process=subprocess.Popen(command,cwd=ROOT,env=environment,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    timed_out=False
    try:stdout,stderr=process.communicate(timeout=45)
    except subprocess.TimeoutExpired:
        timed_out=True;os.killpg(process.pid,signal.SIGKILL);stdout,stderr=process.communicate()
    (case/'stdout.log').write_bytes(stdout);(case/'stderr.log').write_bytes(stderr)
    manifest=json.loads((run/'manifest.json').read_text()) if (run/'manifest.json').exists() else {}
    rows=[json.loads(line) for line in (run/'virtual-trace.jsonl').read_text().splitlines()] if (run/'virtual-trace.jsonl').exists() else []
    counts={kind:sum(r['kind']==kind for r in rows) for kind in ['AckCommitted','AckDuplicate','ClockRequestTransit','NetworkAdmitted']}
    error=manifest.get('error') or ''
    if mode in ['exact-duplicate','frame-exact','payload-exact']:
        passed=not timed_out and process.returncode==0 and manifest.get('outcome')=='COMPLETED'
        if mode=='exact-duplicate':passed &= counts['AckDuplicate']>0
        if mode=='payload-exact':
            deliveries=[r for r in rows if r['kind']=='Grant' and r['detail'].get('input',{}).get('kind',{}).get('Deliver',{}).get('channel')==4 and len(r['detail']['input']['kind']['Deliver']['payload_bytes'])==65536]
            accepted={r['detail']['logical_step_id'] for r in rows if r['kind']=='AckCommitted' and r['detail']['status']==1}
            passed &= len(deliveries)==1 and deliveries[0]['detail']['logical_step_id'] in accepted
    else:
        passed=not timed_out and process.returncode!=0 and manifest.get('outcome')=='INFRASTRUCTURE_FAILURE'
        if mode=='invalid-atomic':passed &= counts['AckCommitted']==counts['ClockRequestTransit']==counts['NetworkAdmitted']==0 and 'TimerArm' in error
        if mode=='unsolicited':passed &= counts['AckCommitted']==0
        if mode=='altered-duplicate':passed &= counts['AckCommitted']==1 and 'duplicate' in error.lower()
        if mode=='frame-over':passed &= any(word in error.lower() for word in ['too large','length','limit'])
        if mode=='payload-over':passed &= counts['AckCommitted']==counts['ClockRequestTransit']==counts['NetworkAdmitted']==0 and 'envelope' in error.lower()
    result={'mode':mode,'status':'PASS' if passed else 'FAIL','exit_code':process.returncode,'timed_out':timed_out,'counts':counts,'error':error,'command':command,'artifact':str(case),'scope':'worker0 is adversarial verifier stub, worker1 execs the real native binary; transport/commit-control only'}
    results.append(result);print(json.dumps(result),flush=True)
report={'candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'binary_hashes':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [hub,worker,helper]},'artifact':str(OUT),'results':results}
(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('REPORT',OUT/'report.json',flush=True)
raise SystemExit(any(r['status']=='FAIL' for r in results))

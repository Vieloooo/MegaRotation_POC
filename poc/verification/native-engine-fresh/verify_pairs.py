#!/usr/bin/env python3
"""Compare complete logical records and native bytes, excluding host diagnostics."""
import argparse
import hashlib
import json
from pathlib import Path

def evidence(root):
    paths=[root/name for name in ['scenario.json','virtual-trace.jsonl','frames.bin']]
    for node in [0,1]:
        worker=root/f'worker-{node}'
        paths.extend(worker/name for name in ['root-step','root-step.head','witness','witness.head'])
        for partition in ['i1b-native-voter','body-fixture']:
            folder=worker/'native'/partition
            if folder.exists():paths.extend(p for p in folder.rglob('*') if p.is_file())
    return {str(path.relative_to(root)):hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(paths)}

def main():
    parser=argparse.ArgumentParser();parser.add_argument('pairs',type=Path);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();results=[]
    for pair in json.loads(args.pairs.read_text()):
        left,right=Path(pair['baseline']),Path(pair['paused_reverse']);a,b=evidence(left),evidence(right);ld=json.loads((left/'transport-diagnostics.json').read_text());rd=json.loads((right/'transport-diagnostics.json').read_text());equal=a==b and ld['wall_pause_ms']==0 and rd['wall_pause_ms']==2 and not ld['reverse_start'] and rd['reverse_start']
        result={'case':pair['case'],'seed':pair['seed'],'status':'PASS' if equal else 'FAIL','baseline':str(left),'paused_reverse':str(right),'files_compared':len(a),'baseline_sha256_by_file':a,'paused_sha256_by_file':b};results.append(result);print(json.dumps({k:result[k] for k in ['case','seed','status','files_compared']}),flush=True)
    args.output.write_text(json.dumps(results,indent=2)+'\n');return any(r['status']=='FAIL' for r in results)
if __name__=='__main__':raise SystemExit(main())

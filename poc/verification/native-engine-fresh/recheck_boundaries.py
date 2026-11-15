#!/usr/bin/env python3
"""Validate actual worker journals behind the completion probe's reported counts."""
import json
from pathlib import Path
import check as C

SOURCE=Path('/tmp/native-fresh-completion-boundaries-o3uig2my')
OUT=Path(__file__).with_name('boundary-journal-recheck.json')
oracle=C.Oracle();results=[]
try:
    for mode in ['explicit-close','lease-boundary','stop-boundary']:
        root=SOURCE/mode
        # Only public resolved configuration is used; the test key is not copied or emitted.
        config=json.loads((root/'launch-private.json').read_text())['resolved']
        # Launch JSON is a transport/config object, not the canonical scenario
        # serialization. Reconstruct the frozen BaseProfile field order.
        config['base']={key:config['base'][key] for key in C.BASE_FIELDS};C.config_check(config,oracle)
        records,receipts=C.journal(root/'worker/root-step',b'I1SHEAD1','I1b/root-step',config,0)
        frames=json.loads((root/'frames.json').read_text());C.need(len(frames)==2*len(records),'probe frame/journal count')
        for i,record in enumerate(records):
            grant,ack=frames[i*2]['grant'],frames[i*2+1]['ack'];logical=dict(ack,durable_step_hash=[])
            C.need(record['grant']==grant and record['ack']==logical,'actual durable worker response')
            C.need(grant['grant_hash']==oracle.ask('proto',type='grant',value=grant)['hash'] and ack['logical_ack_hash']==oracle.ask('proto',type='ack',value=ack)['hash'] and C.b(ack['durable_step_hash'])==receipts[i+1],'probe grant/Ack/root hashes')
        grant,ack=frames[-2]['grant'],frames[-1]['ack'];runtime=json.loads(bytes(ack['runtime_events_json']));app=json.loads(bytes(ack['application_events_json']))
        counts={name:sum(e['event']==name for e in app['work']) for name in ['work_completed','work_discarded','persisted']}
        signs=sum('Sign' in e['kind'] for e in ack['effects']);retired=[e for e in runtime if e['kind']=='retired']
        C.need(counts=={'work_completed':0,'work_discarded':1,'persisted':0} and signs==0,'obsolete completion not consumed/persisted/signed')
        if mode=='lease-boundary':C.need(grant['tick_us']==1100250 and any(e['reason']=='ClockUnsafe' for e in retired),'exact lease equality')
        if mode=='stop-boundary':C.need(grant['tick_us']==1799000 and any(e['reason']=='LocalStop' for e in retired),'exact original local stop')
        results.append({'mode':mode,'status':'PASS','root_steps':len(records),'completion_tick_us':grant['tick_us'],'counts':counts,'sign_effects':signs,'retirement_reasons':[e['reason'] for e in retired]})
finally:oracle.close()
OUT.write_text(json.dumps({'source':str(SOURCE),'results':results,'scope':'one genuine worker per controlled authority probe; not native quorum or restart evidence'},indent=2)+'\n')
print(json.dumps(results,indent=2))

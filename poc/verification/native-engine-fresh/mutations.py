#!/usr/bin/env python3
"""Recompute all affected Grant/Ack/root-chain hashes before semantic rejection."""
import copy
import json
from pathlib import Path
import tempfile
import shutil
import check as C

BASE=C.ROOT/'poc/artifacts/verification/native-engine-fresh/candidate-6686a00-seed7'
OUT=Path(tempfile.mkdtemp(prefix='native-fresh-semantic-mutants-'))

def ack_rows(rows):return [row['detail'] for row in rows if row['kind']=='AckCommitted']
def app(ack):return json.loads(bytes(ack['application_events_json']))
def set_app(ack,value):ack['application_events_json']=list(C.canonical(value))
def visit(value,callback):
    if isinstance(value,dict):
        callback(value)
        for item in list(value.values()):visit(item,callback)
    elif isinstance(value,list):
        if value and isinstance(value[0],int):return
        for item in value:visit(item,callback)

def normalize_like(template,value):
    if isinstance(template,dict) and isinstance(value,dict) and set(template)==set(value):return {key:normalize_like(template[key],value[key]) for key in template}
    if isinstance(template,list) and isinstance(value,list):return [normalize_like(template[i],item) if i<len(template) else item for i,item in enumerate(value)]
    return copy.deepcopy(value)

def rehash(path,rows,oracle):
    c=json.loads((path/'scenario.json').read_text());grants={}
    for row in rows:
        if row['kind']=='Grant':
            g=row['detail'];g['grant_hash']=oracle.ask('proto',type='grant',value=g)['hash'];grants[g['logical_step_id']]=g
    acks={a['logical_step_id']:a for a in ack_rows(rows)};roots=[]
    for node in [0,1]:
        old,_=C.journal(path/f'worker-{node}/root-step',b'I1SHEAD1','I1b/root-step',c,node);records=[];receipts={};previous=bytes(32)
        for prior in old:
            step=prior['grant']['logical_step_id'];a=acks[step];a['grant_hash']=grants[step]['grant_hash'];a['logical_ack_hash']=oracle.ask('proto',type='ack',value=a)['hash'];logical=copy.deepcopy(a);logical['durable_step_hash']=[]
            record={'schema':1,'sequence':prior['sequence'],'previous_record_hash':list(previous),'grant':normalize_like(prior['grant'],grants[step]),'ack':normalize_like(prior['ack'],logical)}
            previous=C.h('I1b/root-step-record/v1',C.canonical(record));a['durable_step_hash']=list(previous);receipts[record['sequence']]=previous;records.append(record)
        roots.append((records,receipts))
    frames=[]
    for row in rows:
        if row['kind'] in ['Grant','AckCommitted']:
            g=row['detail'];frames.append({'direction':0 if row['kind']=='Grant' else 1,'worker':g['binding']['worker_id'],'frame':{'frame':{'Grant' if row['kind']=='Grant' else 'Ack':g}}})
    return roots,frames

def materialize(source,target,rows,roots,frames,oracle):
    target.mkdir()
    for name in ['scenario.json','manifest.json','transport-diagnostics.json']:shutil.copy2(source/name,target/name)
    (target/'virtual-trace.jsonl').write_bytes(b''.join(C.canonical(row)+b'\n' for row in rows))
    c=json.loads((source/'scenario.json').read_text())
    for node,(records,receipts) in enumerate(roots):
        dest=target/f'worker-{node}';dest.mkdir();original=source/f'worker-{node}'
        for name in ['witness','witness.head']:shutil.copy2(original/name,dest/name)
        shutil.copytree(original/'native',dest/'native')
        data=bytearray()
        for record in records:
            payload=C.canonical(record);n=len(payload);digest=C.h('I1b/root-step-record/v1',payload)
            assert digest==receipts[record['sequence']]
            data.extend(C.u64(n)+C.u64(n^(2**64-1))+payload+digest)
        (dest/'root-step').write_bytes(data)
        head=b'I1SHEAD1'+C.u64(len(data))+C.u64(len(records))+receipts[len(records)]
        head+=C.h('I1b/root-step-head/v1',bytes(c['run_id']),bytes(c['config_hash']),C.u32(node),head)
        (dest/'root-step.head').write_bytes(head)
    raw=bytearray()
    for frame in frames:
        encoded=bytes(oracle.ask('frame-encode',direction=frame['direction'],value=frame['frame']))
        raw.extend(bytes([frame['direction'],frame['worker']])+C.u64(len(encoded))+encoded)
    (target/'frames.bin').write_bytes(raw)
    (target/'mutation-origin.json').write_text(json.dumps({'source':str(source),'kind':'synthetic rehashed semantic mutant; not a new process execution'},indent=2)+'\n')

def first_sign(rows):
    for a in ack_rows(rows):
        for e in a['effects']:
            if 'Sign' in e['kind'] and e['kind']['Sign']['action']==1:return a,e['kind']['Sign']
    raise AssertionError('sign fixture absent')
def changed_sign(rows,field,value):
    a,s=first_sign(rows);old=copy.deepcopy(s);s[field]=value(s) if callable(value) else value;data=app(a)
    for item in data['signing']:
        if item==old:item[field]=s[field]
    set_app(a,data)
def expired(rows):changed_sign(rows,'lease_end_tick_us',lambda s:s['tick_us'])
def false_cache(rows):changed_sign(rows,'action',2)
def false_witness(rows):
    selected=None
    for a in ack_rows(rows):
        for e in a['effects']:
            if 'Send' in e['kind'] and 'LocalWitness' in e['kind']['Send']['provenance']:selected=e['kind']['Send'];break
        if selected:break
    token=(selected['message_id'],selected['attempt'])
    def change(value):
        if value.get('message_id')==token[0] and value.get('attempt')==token[1] and 'provenance' in value:value['provenance']['LocalWitness']['witness_ids']=[[0x99]*32]
    visit(rows,change)
def false_worker_origin(rows):
    selected=next(e['kind']['Send'] for a in ack_rows(rows) for e in a['effects'] if 'Send' in e['kind'] and e['kind']['Send']['channel']==4);token=(selected['message_id'],selected['attempt'])
    def change(value):
        if value.get('message_id')==token[0] and value.get('attempt')==token[1] and 'provenance' in value:value['provenance']={'Scripted':{'transcript_id':[8]*32,'entry_id':[9]*32}}
    visit(rows,change)
def early_work(rows):
    for a in ack_rows(rows):
        data=app(a)
        for event in data['work']:
            if event['event']=='work_completed':event['completed_tick_us']-=1;set_app(a,data);return
def proof_without_private_body(rows):
    delivery=next(copy.deepcopy(e['kind']['Delivered']) for a in ack_rows(rows) if a['binding']['worker_id']==1 for e in a['effects'] if 'Delivered' in e['kind'])
    for a in ack_rows(rows):
        if a['binding']['worker_id']==1 and any('Proof' in e['kind'] for e in a['effects']):
            data=app(a);data['work'].append({'event':'delivery','delivered':delivery});set_app(a,data);a['effects'].append({'kind':{'Delivered':delivery}});return
def wrong_parent(rows,wrong_leader=False):
    target=None;replacement=None
    for a in ack_rows(rows):
        for e in app(a)['work']:
            if e['event']=='work_pending' and e['binding']['work']['kind']==1:
                replacement=copy.deepcopy(e['binding']);target=replacement['work']['operation_id']
                if wrong_leader:replacement['view']+=1
                else:replacement['parent_view']=replacement['view']
                break
        if replacement:break
    w=replacement['work'];w['operation_id']=list(C.h('I1b/fresh-work/v1',bytes(w['cid']),C.u32(replacement['member']),C.u32(w['kind']),C.u64(replacement['epoch']),C.u64(replacement['view']),C.u64(replacement['parent_view']),bytes(replacement['parent_digest']),bytes(replacement['requested_payload']),C.u64(replacement['request_generation'])))
    def change(value):
        if value.get('operation_id')==target:value['operation_id']=w['operation_id']
        if isinstance(value.get('work'),dict) and value['work'].get('operation_id') in [target,w['operation_id']] and 'parent_view' in value:
            value['parent_view']=replacement['parent_view'];value['view']=replacement['view']
    visit(rows,change)
    for a in ack_rows(rows):data=app(a);visit(data,change);set_app(a,data)

def main():
    oracle=C.Oracle();results=[]
    cases=[('expired-sign','healthy',expired,'sign lease/stop/cid guard'),('uncreated-cache','healthy',false_cache,'cache hit exact prior attestation'),('false-witness','healthy',false_witness,'exact committed witness reference'),('scripted-worker','healthy',false_worker_origin,'worker output origin'),('early-work','healthy',early_work,'registered timely live WorkCompleted'),('proof-without-body','proof-before-body',proof_without_private_body,'delivery requires privately available body prefix'),('wrong-work-parent','healthy',wrong_parent,'native callback parent view/context'),('wrong-proposer','healthy',lambda rows:wrong_parent(rows,True),'native proposer/elector binding')]
    try:
        for name,case,mutate,rule in cases:
            path=BASE/case;rows=[json.loads(line) for line in (path/'virtual-trace.jsonl').read_text().splitlines()];mutate(rows);roots,frames=rehash(path,rows,oracle);error=None
            target=OUT/name;materialize(path,target,rows,roots,frames,oracle)
            try:C.check(target,oracle)
            except C.Invalid as exc:error=str(exc)
            passed=error is not None and rule in error;result={'name':name,'status':'PASS' if passed else 'FAIL','expected_rule':rule,'error':error,'hashes_recomputed':['Grant','logical Ack','all subsequent root record frames and committed-head checksum','complete raw protobuf frame stream'],'native_signatures_unchanged':True};results.append(result)
            (target/'root-records.json').write_text(json.dumps([x[0] for x in roots],separators=(',',':')));(target/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)
    finally:oracle.close()
    report={'artifact':str(OUT),'candidate':'6686a009e0eeea261026dac807742efed76b17c6','results':results};(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n');print('REPORT',OUT/'report.json');return any(r['status']=='FAIL' for r in results)
if __name__=='__main__':raise SystemExit(main())

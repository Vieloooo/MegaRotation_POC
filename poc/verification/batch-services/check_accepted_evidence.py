#!/usr/bin/env python3
"""Independent actual-Inbox/DA evidence checker, prepared before feature11 code.

Inputs:
  --bundle: existing 09 raw-rpc-bundle.json schema
  --da-index: JSON {"payloads": {"0x<commitment>": "relative/file.bin", ...}}

This verifies offline consistency/signatures/DA bytes; it cannot prove that an
artifact was produced by a real node or establish finality/delivery discipline.
Those require the recorded live run and separate driver-trace/source review.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/'anvil-l1'))
import receipt_checker as r
import independent_codec as c
import frame_vectors as f


def check(bundle, index, root):
    baseline=r.Checker(bundle).run()
    receipts={r.norm(x['transactionHash']):x for x in bundle['receipts']}
    paths={k.lower():v for k,v in index['payloads'].items()}
    accepted=[]
    missing=[]
    for block in bundle['blocks']:
        for transaction in block['transactions']:
            receipt=receipts[r.norm(transaction['hash'])]
            if r.q(receipt['status'])!=1:
                continue
            for local_log, log in enumerate(receipt['logs']):
                if r.norm(log['address'])!=r.norm(bundle['config']['inbox']):
                    continue
                name, values=r.decode_event(log)
                if name!='CursorAdvanced':
                    continue
                call=r.decode_call(transaction)
                r.need(call is not None and call[0]=='submitBatch','Advanced without submitBatch')
                request_id,submission=call[1]
                prev,nxt,commitment,da,first,last,gpos,signature=submission
                position=[r.q(block['number']),r.q(transaction['transactionIndex']),local_log]
                key=c.hx(commitment)
                record={'position':position,'block_hash':block['hash'],'transaction_hash':transaction['hash'],'rpc_log_index':r.q(log['logIndex']),'request_id':c.hx(request_id),'commitment':key,'first_height':first,'last_height':last,'next_origin':nxt[1],'gpos':c.hx(gpos),'inclusion_timestamp_seconds':r.q(block['timestamp'])}
                if key not in paths:
                    missing.append({**record,'status':'WAIT_MISSING_DA'})
                    continue
                payload_path=root/paths[key]
                payload=payload_path.read_bytes()
                r.need(c.keccak256(payload)==commitment,'accepted payload commitment mismatch')
                frame=f.decode(payload)
                r.need(frame[0]['height']==first and frame[-1]['height']==last,'accepted frame range mismatch')
                r.need(frame[-1]['origin']==nxt[1],'accepted frame origin mismatch')
                r.need(first==prev[0]+1 and last==nxt[0],'accepted cursor continuity mismatch')
                accepted.append({**record,'status':'VERIFIED_PAYLOAD','payload_length':len(payload),'payload_sha256':hashlib.sha256(payload).hexdigest(),'records':f.json_records(frame),'da_certificate':c.hx(da),'certifier_signature':c.hx(signature)})
    expected_count=sum(x['kind']=='CursorAdvanced' for x in baseline['events'])
    r.need(len(accepted)+len(missing)==expected_count,'accepted event coverage mismatch')
    return {'status':'PASS' if not missing else 'INCONCLUSIVE','scope':'offline raw receipt/ABI/signature replay plus BS-FRAME-v1 DA opening; source_mode remains fixture','accepted_count':expected_count,'payloads_verified':len(accepted),'accepted':accepted,'missing':missing,'failed_receipts':baseline['failedReceipts'],'no_new_event_retries':baseline['idempotentRetries'],'not_established':['artifact live-run provenance','driver finalized and reader delivery discipline','DA write-before-receipt durability','Certifier independent G replay','full GFR/G-prime/D/ResumeCut']}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--bundle',type=Path,required=True)
    parser.add_argument('--da-index',type=Path,required=True)
    args=parser.parse_args()
    result=check(json.loads(args.bundle.read_text()),json.loads(args.da_index.read_text()),args.da_index.parent)
    print(json.dumps(result,indent=2))
    if result['status']!='PASS':
        sys.exit(2)


if __name__=='__main__':
    main()

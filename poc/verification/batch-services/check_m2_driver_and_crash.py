#!/usr/bin/env python3
"""Independent M2 command frontiers, raw transactions and durable evidence audit.

This reads recorded evidence without production imports/network access. Real
process exits and exact fsync hooks additionally require live-run/source review.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/'anvil-l1'))
import independent_codec as c
import golden_vectors as g
import receipt_checker as r


def need(ok,why):
    if not ok: raise ValueError(why)


def raw(value):
    return bytes(value) if isinstance(value,list) else c.as_bytes(value)


def rlp_bytes(value):
    if len(value)==1 and value[0]<128: return value
    return rlp_prefix(len(value),128)+value


def rlp_prefix(length,base):
    if length<56: return bytes([base+length])
    encoded=length.to_bytes((length.bit_length()+7)//8,'big')
    return bytes([base+55+len(encoded)])+encoded


def rlp_list(items):
    body=b''.join(rlp_bytes(x) for x in items)
    return rlp_prefix(len(body),192)+body


def decode_rlp(value,start=0):
    need(start<len(value),'RLP truncated')
    tag=value[start]
    if tag<128: return value[start:start+1],start+1
    if tag<184:
        size=tag-128;begin=start+1;end=begin+size
        need(end<=len(value),'RLP short string')
        result=value[begin:end]
        need(size!=1 or result[0]>=128,'RLP noncanonical byte')
        return result,end
    if tag<192:
        count=tag-183;begin=start+1
        need(begin+count<=len(value) and value[begin]!=0,'RLP long string size')
        size=int.from_bytes(value[begin:begin+count],'big');begin+=count;end=begin+size
        need(size>=56 and end<=len(value),'RLP long string bounds')
        return value[begin:end],end
    if tag<248:
        size=tag-192;begin=start+1
    else:
        count=tag-247;begin=start+1
        need(begin+count<=len(value) and value[begin]!=0,'RLP long list size')
        size=int.from_bytes(value[begin:begin+count],'big');begin+=count
        need(size>=56,'RLP long list canonical')
    end=begin+size;need(end<=len(value),'RLP list bounds')
    items=[]
    while begin<end:
        item,begin=decode_rlp(value,begin);items.append(item)
    need(begin==end,'RLP list item overflow')
    return items,end


def uint_bytes(value):
    return value.to_bytes((value.bit_length()+7)//8,'big')


def transaction(value,chain,expected_sender):
    data=raw(value);items,end=decode_rlp(data)
    need(end==len(data) and isinstance(items,list) and len(items)==9,'legacy transaction shape')
    need(all(isinstance(x,bytes) for x in items) and rlp_list(items)==data,'legacy canonical RLP')
    for i in (0,1,2,4,6,7,8): need(not items[i] or items[i][0]!=0,'legacy integer leading zero')
    v=int.from_bytes(items[6],'big');recovery=v-2*chain-35
    need(recovery in (0,1),'legacy chain/recovery id')
    signature=items[7].rjust(32,b'\0')+items[8].rjust(32,b'\0')+bytes([27+recovery])
    digest=c.keccak256(rlp_list(items[:6]+[uint_bytes(chain),b'',b'']))
    need(c.recover_address(signature,digest)==raw(expected_sender),'raw transaction signer')
    return {'hash':c.hx(c.keccak256(data)),'nonce':int.from_bytes(items[0],'big'),'to':c.hx(items[3]),'calldata':items[5]}


def cursor(value):
    return [value['height'],value['origin'],value['frames_digest'],value['source_kind'],value['source']]


def batch_tuple(value):
    return [cursor(value['prev']),cursor(value['next']),value['commitment'],value['da_certificate'],value['first_height'],value['last_height'],value['gpos'],value['certifier_signature']]


def check(root):
    load=lambda name:json.loads((root/name).read_text())
    cfg=load('driver-config.json');bundle=load('raw-rpc-bundle.json');trace=load('driver-trace.json');rpc=load('rpc-transcript.json');crashes=load('crash-trace.json')
    headers={int(b['number'],16):b for b in bundle['blocks']}
    receipts={x['transactionHash']:x for x in bundle['receipts']}
    actual={'reference_us':0,'included_height':0,'finalized_height':0,'visible_heights':{name:0 for name in cfg['readers']}}
    mutations=[];wait_reads=[];cursor_reads=[];submissions=[]
    for expected_id,row in enumerate(trace,1):
        cmd,reply=row['command'],row['reply'];op=cmd['op'];status=reply['status']
        need(cmd['id']==reply['id']==expected_id,'ordered driver command identity')
        after={key:reply[key] for key in actual}
        expected=json.loads(json.dumps(actual))
        if status=='ready':
            result=reply['result']
            if op=='tick':
                need(cmd['reference_us']>=actual['reference_us'],'tick regression');expected['reference_us']=cmd['reference_us']
            elif op=='mine':
                h=cmd['slot'];need(h==actual['included_height']+1 and actual['reference_us']>=h*cfg['slot_seconds']*1000000,'mine reference/sequence')
                expected['included_height']=h
                need(result['included_block']['header_and_transactions']==headers[h],'mined header evidence')
                mutations.extend([('evm_setNextBlockTimestamp',[hex(cfg['genesis_timestamp_seconds']+h*cfg['slot_seconds'])]),('evm_mine',[])])
            elif op=='finalize':
                h=cmd['through_height'];need(actual['finalized_height']<=h<=actual['included_height'] and actual['reference_us']>=h*cfg['slot_seconds']*1000000+cfg['finality_delay_us'],'finality boundary')
                expected['finalized_height']=h
            elif op=='deliver':
                h=cmd['through_height'];reader=cmd['reader'];need(actual['visible_heights'][reader]<=h<=actual['finalized_height'],'delivery boundary')
                expected['visible_heights'][reader]=h
            elif op=='submit':
                tx=transaction(cmd['raw'],cfg['chain_id'],cfg['submitter']);need(result['submitted_tx_hash']==tx['hash'],'submitted hash')
                mutations.append(('eth_sendRawTransaction',[cmd['raw']]))
                submissions.append(tx)
            elif op=='read':
                query=cmd['query'];h=query['height'];need(h<=actual['visible_heights'][cmd['reader']]<=actual['finalized_height'],'reader accessed invisible/unfinalized block')
                if query['kind']=='block':
                    need(result['control_coverage_through']==h and result['block']['header_and_transactions']==headers[h],'read block/coverage evidence')
                    need(result['block']['receipts']==[receipts[t['hash']] for t in headers[h]['transactions']],'read receipts')
                elif query['kind']=='cursor':
                    need(result['as_of']==h,'cursor read as_of');cursor_reads.append([h,result['written_at']['block'],result['written_at']['tx'],result['written_at']['log']])
            else: raise ValueError('unknown driver operation')
        else:
            need(status=='wait' and op=='read','unexpected nonready result')
            wait_reads.append({'height':cmd['query']['height'],'included':actual['included_height'],'finalized':actual['finalized_height'],'visible':actual['visible_heights'][cmd['reader']]})
        need(after==expected,'driver frontiers changed outside owning operation')
        actual=after
    need(wait_reads==[{'height':2,'included':1,'finalized':1,'visible':1},{'height':2,'included':2,'finalized':1,'visible':1},{'height':2,'included':2,'finalized':2,'visible':1}],'three acceptance gate waits')
    allowed={'eth_chainId','anvil_getAutomine','eth_blockNumber','eth_getBlockByNumber','eth_getTransactionReceipt','eth_call','eth_sendRawTransaction','eth_getTransactionByHash','evm_setNextBlockTimestamp','evm_mine'}
    actual_mutations=[]
    for entry in rpc:
        request=entry['request'];method=request['method'];need(method in allowed,'unexpected driver RPC')
        need(entry['response']['id']==request['id'] and entry['result']['Ok']==entry['response']['result'],'recorded RPC result/response mismatch')
        if method in ('eth_sendRawTransaction','evm_setNextBlockTimestamp','evm_mine'):actual_mutations.append((method,request['params']))
        elif method=='eth_getBlockByNumber':need(entry['result']['Ok']==headers[int(request['params'][0],16)],'RPC header mismatch')
        elif method=='eth_getTransactionReceipt':need(entry['result']['Ok']==receipts[request['params'][0]],'RPC receipt mismatch')
    need(actual_mutations==mutations,'uncontrolled or missing mutating RPC')
    expected_points={'batcher_built_durable','da_before_bytes','da_bytes_durable','da_receipt_durable','batcher_da_durable','certifier_before_signature_persist','certifier_signature_durable','batcher_certified_durable','batcher_raw_durable','raw_sent_before_reply','batcher_accepted_durable'}
    need(len(crashes)==11 and {x['boundary'] for x in crashes}==expected_points,'crash coverage')
    for case in crashes:
        need(case['pid']!=case['reopen_pid'],'role child not reopened in a different process')
        need(case['journal_keccak_after']==case['journal_keccak_reopened'],'reopen changed already persisted files')
        need(case['journal_keccak_before']['identity.json']==case['journal_keccak_after']['identity.json'],'crash changed identity')
        need(case.get('signal')=='SIGKILL' if case['boundary']=='raw_sent_before_reply' else case['exit_code']==91,'actual process termination status')
    bypoint={case['boundary']:case for case in crashes}
    files=lambda point,suffix:[name for name in bypoint[point]['journal_keccak_after'] if name.endswith(suffix)]
    need(not files('da_before_bytes','.bin') and not files('da_before_bytes','.receipt'),'DA pre-byte state')
    need(len(files('da_bytes_durable','.bin'))==1 and not files('da_bytes_durable','.receipt'),'DA byte-before-receipt boundary')
    need(len(files('da_receipt_durable','.bin'))==len(files('da_receipt_durable','.receipt'))==1,'DA durable receipt boundary')
    for point in ('da_bytes_durable','da_receipt_durable'):
        for name in files(point,'.bin')+files(point,'.receipt'):
            need(c.hx(c.keccak256((root/'da'/name).read_bytes()))==bypoint[point]['journal_keccak_after'][name],'acknowledged DA changed after restart')
    sent=bypoint['raw_sent_before_reply'];need(sent['journal_keccak_before']==sent['journal_keccak_after'],'send crash modified persisted outbox')
    tx=transaction(sent['raw'],cfg['chain_id'],cfg['submitter']);need(tx['hash']==sent['tx_hash'] and tx['nonce']==sent['nonce'],'lost-reply raw identity')
    need(sum(tx['hash']==entry['hash'] for entry in submissions)==1,'lost-reply task blindly resent with different identity')
    bc=load('batcher/state.json');cc=load('certifier/state.json')
    need(len(bc['jobs'])==3 and len(cc['signatures'])==3,'durable task/signature count')
    outcomes=[]
    auth=r.Checker(bundle)
    for identity,job in bc['jobs'].items():
        payload=bytes(job['payload']);commitment=c.keccak256(payload)
        need(raw(job['id'])==raw(identity) and commitment==raw(job['commitment']),'durable task id/data hash')
        preimage=b'PoC/BatchJob/v1'+cfg['chain_id'].to_bytes(32,'big')+raw(cfg['inbox'])+raw(cfg['config_ref'])+g.cursor_hash(cursor(job['prev']))[1]+commitment
        need(c.keccak256(preimage)==raw(identity),'job identity preimage')
        batch=job['certified'];need(batch['request_id']==identity and raw(batch['da_certificate'])==raw(job['da_certificate']),'durable certification binding')
        auth.batch_args([raw(identity),r.abi_decode([g.BATCH],c.abi_encode([g.BATCH],[batch_tuple(batch)]))[0]],raw(cfg['submitter']))
        tx=transaction(job['raw']['raw'],cfg['chain_id'],cfg['submitter'])
        need(tx['hash']==c.hx(raw(job['raw']['hash'])) and tx['nonce']==job['raw']['nonce'],'durable outbox hash/nonce')
        need(tx['calldata']==c.keccak256(g.FUNCTIONS['submitBatch'].encode())[:4]+c.abi_encode(['bytes32',g.BATCH],[raw(identity),batch_tuple(batch)]),'durable outbox attestation calldata')
        receipt=receipts[tx['hash']];terminal=job['terminal']['outcome']
        need((terminal=='accepted' and receipt['status']=='0x1') or (terminal=='reverted' and receipt['status']=='0x0'),'durable terminal contradicts real receipt')
        outcomes.append({'job_id':identity,'terminal':terminal,'tx_hash':tx['hash'],'nonce':tx['nonce']})
    need(sorted(x['terminal'] for x in outcomes)==['accepted','accepted','reverted'],'terminal preservation')
    for signed in cc['signatures'].values():
        batch=signed['result'];auth.batch_args([raw(batch['request_id']),r.abi_decode([g.BATCH],c.abi_encode([g.BATCH],[batch_tuple(batch)]))[0]],raw(cfg['submitter']))
        need(any(batch==job['certified'] for job in bc['jobs'].values()),'Certifier output differs from saved Batcher result')
    return {'status':'PASS','commands':len(trace),'mutating_rpc_count':len(mutations),'raw_transaction_signatures_checked':len(submissions),'wait_read_frontiers':wait_reads,'cursor_queries':len(cursor_reads),'final_frontiers':actual,'crash_boundaries':sorted(expected_points),'process_exits':10,'sigkill':1,'durable_jobs':outcomes,'durable_certifier_signatures':len(cc['signatures']),'limits':['offline audit relies on observed live scenario/source review for actual process/fsync boundaries','same-host process exits; no kernel crash/power loss','fixture G source; no native recovery or full-system grant/ack proof']}


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('run',type=Path);args=parser.parse_args()
    print(json.dumps(check(args.run),indent=2))

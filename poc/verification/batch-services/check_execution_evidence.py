#!/usr/bin/env python3
"""Independent 03 Apply/Kept oracle for bounded feature11 source fixtures.

No production import, subprocess, RPC, clock, or scheduler access. Accepted
records come from the separate independent receipt/DA checker; this program
does not reimplement that check or claim a fixture is actual GFR replay.
"""
from __future__ import annotations
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct

MAX=(1 << 64)-1


def need(condition, reason):
    if not condition:
        raise ValueError(reason)


def uint(value, bits=64):
    need(type(value) is int and 0 <= value < 1 << bits, f'unsigned u{bits}')
    return value


def raw(value, length=None):
    need(isinstance(value,str) and value.startswith('0x') and len(value)%2==0,'raw hex')
    result=bytes.fromhex(value[2:])
    need(length is None or len(result)==length,'raw width')
    return result


def hx(value):
    return '0x'+value.hex()


def metadata(value):
    need(set(value)=={'height','timestamp_us','origin'},'metadata fields')
    return struct.pack('>QQQ',*(uint(value[k]) for k in ('height','timestamp_us','origin')))


def accounts(value):
    need(isinstance(value,list) and len(value)<=256,'account count')
    output={}
    previous=-1
    for row in value:
        need(isinstance(row,list) and len(row)==3,'account row')
        account,nonce,balance=uint(row[0],8),uint(row[1]),uint(row[2])
        need(account>previous and (nonce!=0 or balance!=0),'noncanonical account order/zero')
        output[account]=[nonce,balance]
        previous=account
    return output


def account_rows(state):
    return [[key,*value] for key,value in sorted(state.items()) if value!=[0,0]]


def framed_hash(domain, value):
    domain=domain.encode('ascii')
    return hashlib.sha256(struct.pack('>Q',len(domain))+domain+struct.pack('>Q',len(value))+value).digest()


def state_root(meta, state):
    rows=account_rows(state)
    encoded=b'\x00\x01'+metadata(meta)+struct.pack('>I',len(rows))+b''.join(struct.pack('>BQQ',*row) for row in rows)
    return framed_hash('PoC/ExecutionState/v1',encoded)


def blobs(values):
    return struct.pack('>I',len(values))+b''.join(struct.pack('>I',len(v))+v for v in values)


def native_hash(parent, meta, dep, kept, post_root):
    return framed_hash('PoC/NativeBlock/v1',b'\x00\x01'+parent+metadata(meta)+blobs(dep)+blobs(kept)+post_root)


def apply(parent, meta, dep, transactions, byte_limit):
    # Decode all input before constructing any returned state or Kept list.
    encoded_metadata=metadata(meta)
    uint(byte_limit,32)
    need(1<=len(dep)<=1025,'DEP count')
    need(dep[0]==b'\x03'+encoded_metadata,'DEP attributes mismatch')
    credits=[]
    for item in dep[1:]:
        need(len(item)==10 and item[0]==2,'DEP credit encoding')
        credits.append(struct.unpack('>BQ',item[1:]))
    need(len(transactions)<=1024 and sum(map(len,transactions))<=min(byte_limit,65536),'ordinary bounds')
    transfers=[]
    for item in transactions:
        need(len(item)==19 and item[0]==1,'ordinary transfer encoding')
        transfers.append(struct.unpack('>BBQQ',item[1:]))
    state=copy.deepcopy(parent)
    for recipient,amount in credits:
        nonce,balance=state.get(recipient,[0,0])
        need(balance<=MAX-amount,'credit overflow')
        if nonce or balance+amount:
            state[recipient]=[nonce,balance+amount]
    kept=[]
    reasons=[]
    for index,(sender,recipient,nonce,amount) in enumerate(transfers):
        old_nonce,balance=state.get(sender,[0,0])
        target_nonce,target_balance=state.get(recipient,[0,0])
        reason=None
        if nonce!=old_nonce:
            reason='nonce_mismatch'
        elif old_nonce==MAX:
            reason='nonce_overflow'
        elif amount>balance:
            reason='insufficient_balance'
        elif sender!=recipient and target_balance>MAX-amount:
            reason='recipient_overflow'
        if reason is not None:
            reasons.append({'index':index,'reason':reason})
            continue
        if sender==recipient:
            state[sender]=[old_nonce+1,balance]
        else:
            state[sender]=[old_nonce+1,balance-amount]
            if target_nonce or target_balance+amount:
                state[recipient]=[target_nonce,target_balance+amount]
        kept.append(index)
    return state,kept,reasons


def derive_sources(source):
    need(source['schema']=='BS-EXECUTION-EVIDENCE-v1','source schema')
    need(source['source_mode']=='fixture','source must be explicitly fixture')
    need(isinstance(source['fixture_id'],str) and source['fixture_id'],'fixture identity')
    raw(source['config_ref'],32)
    byte_limit=uint(source['byte_limit'],32)
    snapshots={}
    compared_outputs=0
    for snapshot in source['snapshots']:
        gpos=hx(raw(snapshot['gpos']))
        need(1<=len(raw(gpos))<=1024 and gpos not in snapshots,'gpos bounds/duplicate snapshot')
        initial=snapshot['initial']
        current_meta=initial['metadata']
        metadata(current_meta)
        current_accounts=accounts(initial['accounts'])
        parent=raw(initial['native_parent'],32)
        blocks={}
        for block in snapshot['blocks']:
            meta=block['metadata']
            metadata(meta)
            need(current_meta['height']<MAX and meta['height']==current_meta['height']+1,'source height continuity')
            before=state_root(current_meta,current_accounts)
            need(raw(block['pre_state_root'],32)==before,'source pre-state root mismatch')
            dep=[raw(x) for x in block['dep']]
            transactions=[raw(x) for x in block['raw_transactions']]
            after,indices,rejected=apply(current_accounts,meta,dep,transactions,byte_limit)
            kept=[transactions[i] for i in indices]
            post=state_root(meta,after)
            new_native=native_hash(parent,meta,dep,kept,post)
            value={'height':meta['height'],'origin':meta['origin'],'pre_state_root':hx(before),'kept_indices':indices,'kept':[hx(x) for x in kept],'filtered':rejected,'post_accounts':account_rows(after),'post_state_root':hx(post),'native_hash':hx(new_native)}
            observed=block.get('observed')
            if observed is not None:
                need([hx(raw(x)) for x in observed['kept']]==value['kept'],'observed Kept mismatch')
                need(account_rows(accounts(observed['post_accounts']))==value['post_accounts'],'observed post accounts mismatch')
                need(raw(observed['post_state_root'],32)==post,'observed post-state root mismatch')
                need(raw(observed['native_hash'],32)==new_native,'observed native hash mismatch')
                compared_outputs+=1
            blocks[meta['height']]=value
            current_meta,current_accounts,parent=meta,after,new_native
        snapshots[gpos]=blocks
    return snapshots,compared_outputs


def check(source, accepted):
    need(accepted['status']=='PASS','accepted receipt/DA result must already PASS')
    need(len(accepted['accepted'])>0 and accepted['accepted_count']==accepted['payloads_verified']==len(accepted['accepted']),'complete nonempty accepted payload coverage')
    need(raw(source['config_ref'],32)==raw(accepted['configuration']['config_ref'],32),'source/Inbox configRef mismatch')
    snapshots,compared=derive_sources(source)
    missing=[]
    matches=[]
    for batch in accepted['accepted']:
        gpos=hx(raw(batch['gpos']))
        if gpos not in snapshots:
            missing.append({'gpos':gpos,'position':batch['position'],'reason':'missing source snapshot'})
            continue
        for record in batch['records']:
            height=record['height']
            if height not in snapshots[gpos]:
                missing.append({'gpos':gpos,'position':batch['position'],'height':height,'reason':'missing source height'})
                continue
            expected=snapshots[gpos][height]
            need(record['origin']==expected['origin'],'accepted DA origin != fixture source')
            need([hx(raw(x)) for x in record['transactions']]==expected['kept'],'accepted DA ordinary transactions != independently executed Kept')
            matches.append({'position':batch['position'],'commitment':batch['commitment'],'gpos':gpos,'height':height,'kept_count':len(expected['kept']),'filtered_count':len(expected['filtered']),'post_state_root':expected['post_state_root'],'native_hash':expected['native_hash']})
    return {'status':'PASS' if not missing else 'INCONCLUSIVE','scope':'independent 03 execution and accepted DA record fidelity; source_mode=fixture','fixture_id':source['fixture_id'],'config_ref':source['config_ref'],'snapshots':{key:list(value.values()) for key,value in snapshots.items()},'observed_poststates_compared':compared,'accepted_heights_compared':len(matches),'matches':matches,'missing':missing,'not_established':['actual GFR consensus/source validity','authenticity of initial state/fixture config','full PublicConfig or dep-fixture content-address validation','DA durability','D actual-parent derivation','receipt/finality checks must be provided by the separate accepted evidence result']}


def selftest():
    m={'height':1,'timestamp_us':1000000,'origin':0}
    attr=b'\x03'+metadata(m)
    tx=lambda a,b,n,v:b'\x01'+struct.pack('>BBQQ',a,b,n,v)
    credit=lambda a,v:b'\x02'+struct.pack('>BQ',a,v)
    cases=[]
    def good(name,rows,dep,transactions,expected,kept,limit=65536):
        original=accounts(rows)
        result,indices,reasons=apply(original,m,dep,transactions,limit)
        need(account_rows(result)==expected and indices==kept,name)
        need(original==accounts(rows),'parent mutation')
        cases.append({'name':name,'status':'PASS','accounts':expected,'kept_indices':indices,'filtered':reasons})
        return result
    h1=good('contract-h1',[[1,0,10]],[attr],[tx(1,2,0,3),tx(1,2,0,1),tx(1,2,1,100)],[[1,1,7],[2,0,3]],[0])
    good('contract-h2',account_rows(h1),[attr],[tx(1,2,1,2)],[[1,2,5],[2,0,5]],[0])
    good('contract-padding',account_rows(h1),[attr],[],[[1,1,7],[2,0,3]],[])
    good('credit-and-filter',[[0,0,10]],[attr,credit(0,5)],[tx(0,1,0,7),tx(0,1,0,1),tx(0,1,1,9),tx(0,1,1,8)],[[0,2,0],[1,0,15]],[0,3])
    good('self-at-max-balance',[[1,0,MAX]],[attr],[tx(1,1,0,MAX)],[[1,1,MAX]],[0])
    good('nonce-overflow',[[1,MAX,1]],[attr],[tx(1,2,MAX,1)],[[1,MAX,1]],[])
    good('receiver-overflow',[[1,0,1],[2,0,MAX]],[attr],[tx(1,2,0,1)],[[1,0,1],[2,0,MAX]],[])
    good('zero-new-sender',[],[attr,credit(5,0)],[tx(3,4,0,0)],[[3,1,0]],[0])
    good('self-insufficient',[[1,0,1]],[attr],[tx(1,1,0,2)],[[1,0,1]],[])
    bad=[
        ('missing-attr',[],[],[],65536),
        ('wrong-attr',[],[b'\x03'+bytes(24)],[],65536),
        ('ordinary-credit',[],[attr],[credit(1,1)],65536),
        ('DEP-transfer',[],[attr,tx(1,2,0,1)],[],65536),
        ('late-attr',[],[attr,attr],[],65536),
        ('bad-tail-after-valid',[[1,0,10]],[attr],[tx(1,2,0,3),b'\x01'],65536),
        ('credit-overflow',[[1,0,MAX]],[attr,credit(1,1)],[],65536),
        ('byte-limit',[],[attr],[tx(1,2,0,0)],18),
        ('tx-count',[],[attr],[tx(1,2,0,0)]*1025,65536),
        ('DEP-count',[],[attr]+[credit(1,0)]*1025,[],65536),
    ]
    for name,rows,dep,transactions,limit in bad:
        original=accounts(rows)
        try:
            apply(original,m,dep,transactions,limit)
        except ValueError:
            need(original==accounts(rows),'reject mutated parent')
            cases.append({'name':name,'status':'PASS_REJECT'})
        else:
            raise ValueError(name+' unexpectedly accepted')
    # In-memory schema fixtures exercise evidence joins only; these are never
    # serialized as real accepted receipts or counted as implementation evidence.
    initial_meta={'height':0,'timestamp_us':0,'origin':0}
    initial_accounts=accounts([[1,0,10]])
    initial_native=bytes([0x22])*32
    post_root=state_root(m,h1)
    first_tx=tx(1,2,0,3)
    observed_native=native_hash(initial_native,m,[attr],[first_tx],post_root)
    source={'schema':'BS-EXECUTION-EVIDENCE-v1','source_mode':'fixture','fixture_id':'reference-schema-selftest-only','config_ref':hx(bytes([0x11])*32),'byte_limit':65536,'snapshots':[{'gpos':hx(b'bs11-fixture-gpos-1'),'initial':{'metadata':initial_meta,'accounts':[[1,0,10]],'native_parent':hx(initial_native)},'blocks':[{'metadata':m,'dep':[hx(attr)],'raw_transactions':[hx(first_tx),hx(tx(1,2,0,1)),hx(tx(1,2,1,100))],'pre_state_root':hx(state_root(initial_meta,initial_accounts)),'observed':{'kept':[hx(first_tx)],'post_accounts':account_rows(h1),'post_state_root':hx(post_root),'native_hash':hx(observed_native)}}]}]}
    accepted={'status':'PASS','configuration':{'config_ref':source['config_ref']},'accepted_count':1,'payloads_verified':1,'accepted':[{'gpos':source['snapshots'][0]['gpos'],'position':[0,0,0],'commitment':hx(bytes(32)),'records':[{'height':1,'origin':0,'transactions':[hx(first_tx)]}]}]}
    joined=check(source,accepted)
    need(joined['status']=='PASS' and joined['accepted_heights_compared']==1 and joined['observed_poststates_compared']==1,'source/accepted join')
    cases.append({'name':'schema-source-accepted-join','status':'PASS','kind':'in-memory reference fixture only'})
    for name in ('config-binding','pre-root','observed-state','accepted-not-kept'):
        altered_source=copy.deepcopy(source)
        altered_accepted=copy.deepcopy(accepted)
        if name=='config-binding': altered_source['config_ref']=hx(bytes([0x33])*32)
        if name=='pre-root': altered_source['snapshots'][0]['blocks'][0]['pre_state_root']=hx(bytes(32))
        if name=='observed-state': altered_source['snapshots'][0]['blocks'][0]['observed']['post_accounts']=[[1,0,10]]
        if name=='accepted-not-kept': altered_accepted['accepted'][0]['records'][0]['transactions'].append(hx(tx(1,2,0,1)))
        try:
            check(altered_source,altered_accepted)
        except ValueError:
            cases.append({'name':name,'status':'PASS_REJECT'})
        else:
            raise ValueError(name+' accepted altered evidence')
    missing=copy.deepcopy(source)
    missing['snapshots']=[]
    need(check(missing,accepted)['status']=='INCONCLUSIVE','missing source must WAIT')
    cases.append({'name':'missing-source','status':'PASS_INCONCLUSIVE'})
    return {'status':'PASS','scope':'source reference self-check, production M2 not inspected or executed','source':'03 e9c15e04bc83b335668f3ecd837367aa87344a89; task-contract-mock-plan-execution-v2 §5.1; BS r1 §9','seed':'none (literal cases)','cases':cases}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--source',type=Path)
    parser.add_argument('--accepted-result',type=Path)
    parser.add_argument('--self-test',action='store_true')
    args=parser.parse_args()
    if args.self_test:
        result=selftest()
    elif args.source and args.accepted_result:
        result=check(json.loads(args.source.read_text()),json.loads(args.accepted_result.read_text()))
        result['source_file_sha256']=hashlib.sha256(args.source.read_bytes()).hexdigest()
        result['accepted_result_sha256']=hashlib.sha256(args.accepted_result.read_bytes()).hexdigest()
    else:
        parser.error('provide --self-test or --source and --accepted-result')
    print(json.dumps(result,indent=2))
    if result['status']!='PASS':
        raise SystemExit(2)


if __name__=='__main__':
    main()

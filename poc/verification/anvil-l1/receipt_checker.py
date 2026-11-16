#!/usr/bin/env python3
"""Offline, independent ANVIL-L1-ABI-v1 raw JSON-RPC receipt checker.

Prepared before candidate source review. Not a production reader or proof verifier.
Input is an exported evidence bundle; extraction from actual runner artifacts is
intentionally deferred until their format is available. No network or wall clock.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
_bootstrap=argparse.ArgumentParser(add_help=False)
_bootstrap.add_argument('--verification-dir',type=Path)
_bootargs,_=_bootstrap.parse_known_args()
VERIFY = _bootargs.verification_dir or Path(__file__).resolve().parent
if not (VERIFY/'independent_codec.py').is_file():
    VERIFY = Path('/home/jy/.codex/worktrees/65ef/megaroation_poc/poc/verification/anvil-l1')
sys.path.insert(0,str(VERIFY))
import independent_codec as c
import golden_vectors as g


def need(ok,why):
    if not ok: raise ValueError(why)


def raw(x): return c.as_bytes(x)
def hx(x): return c.hx(x)
def q(x): return int(x,16) if isinstance(x,str) else int(x)
def norm(x): return x.lower()
def w(data,off):
    need(off >= 0 and off+32 <= len(data),'ABI word out of bounds')
    return int.from_bytes(data[off:off+32],'big')


def decode_value(ty,data,start):
    if isinstance(ty,(tuple,list)): return decode_tuple(ty,data,start)
    if ty.startswith('uint'):
        v=w(data,start);need(v<1<<int(ty[4:] or 256),'noncanonical ABI uint');return v
    if ty=='address':
        need(w(data,start)<1<<160,'noncanonical ABI address');return data[start+12:start+32]
    if ty.startswith('bytes') and ty!='bytes':
        n=int(ty[5:]);w(data,start);need(data[start+n:start+32]==b'\0'*(32-n),'nonzero fixed-bytes padding');return data[start:start+n]
    if ty in ('bytes','string'):
        length=w(data,start);end=start+32+length
        need(end<=len(data),'dynamic bytes out of bounds')
        value=data[start+32:end]
        return value.decode('utf8') if ty=='string' else value
    raise ValueError('unsupported decode type '+str(ty))


def decode_tuple(types,data,base):
    result=[];head=base
    for ty in types:
        start=base+w(data,head) if c.dynamic(ty) else head
        result.append(decode_value(ty,data,start))
        head += 32 if c.dynamic(ty) else c.static_size(ty)
    return result


def abi_decode(types,data):
    data=raw(data); values=decode_tuple(types,data,0)
    need(c.abi_encode(types,values)==data,'noncanonical ABI offsets/padding/trailing bytes')
    return values


def jsonable(v):
    if isinstance(v,bytes):return hx(v)
    if isinstance(v,list):return [jsonable(x) for x in v]
    if isinstance(v,dict):return {k:jsonable(x) for k,x in v.items()}
    return v


CALLS={
 'submitForce':['bytes32','uint64','uint64',g.POLICY,'bytes'],
 'submitResume':['bytes32','uint64',g.POLICY,'uint64','uint64','bytes'],
 'submitBatch':['bytes32',g.BATCH],
 'overrideCursor':[g.CURSOR]}
SELECTORS={c.keccak256(g.FUNCTIONS[k].encode())[:4]:k for k in CALLS}
TOPICS={hx(c.keccak256(v.encode())):k for k,v in g.EVENTS.items()}
EVENT_LAYOUT={
 'ForceRecorded':(['address','bytes32','uint64'],['uint64',g.POLICY,'uint64']),
 'ResumeRecorded':(['address','bytes32','uint64'],[g.POLICY,'uint64','uint64','uint64']),
 'CursorInitialized':([],[g.CURSOR]),
 'CursorAdvanced':(['address','bytes32'],[g.CURSOR,g.CURSOR,'bytes32']),
 'CursorSet':([],[g.CURSOR,g.CURSOR])}


def decode_call(tx):
    data=raw(tx['input']); name=SELECTORS.get(data[:4])
    return None if name is None else (name,abi_decode(CALLS[name],data[4:]))


def decode_event(log):
    topics=log['topics']; name=TOPICS.get(norm(topics[0])) if topics else None
    need(name is not None,'unknown configured-contract event')
    indexed,types=EVENT_LAYOUT[name]
    need(len(topics)==len(indexed)+1,'event indexed-topic count')
    iv=[abi_decode([ty],topics[i+1])[0] for i,ty in enumerate(indexed)]
    return name,iv+abi_decode(types,log['data'])


def cursor_valid(v):
    height,origin,frames,kind,source=v
    need(frames==b'\0'*32,'unsupported open frame cursor')
    need(kind in (0,1,2),'bad source kind')
    if kind==0:need(source==b'','bad genesis source')
    if kind==1:need(1<=len(source)<=1024,'bad GLOBAL source length')
    if kind==2:need(len(source)==32 and int.from_bytes(source,'big')<1<<64,'bad RESUME source')


def policy_valid(p):need(p[:4]==[7,0,1,2],'unsupported P0 policy')


class Checker:
    def __init__(self,bundle):
        self.bundle=bundle; self.cfg=bundle['config'];self.cursor=None;self.written=None
        self.seq=None;self.requests={};self.events=[];self.history={};self.failed=[];self.retries=[]
        self.addresses={k:raw(self.cfg[k]) for k in ('authority','inbox')}
        self.domains={k:c.domain_separator(g.NAMES[k],'1',self.cfg['chainId'],self.addresses[k],raw(self.cfg['configRef'])) for k in self.addresses}
        self.signers={k:raw(v) for k,v in self.cfg['signers'].items()}

    def signed(self,role,kind,types,values,signature,signer):
        h=g.struct(kind,types,values)[1];d=c.typed_digest(self.domains[role],h)
        need(c.recover_address(signature,d)==self.signers[signer],kind+' signer mismatch')
        return h

    def control_args(self,name,args,sender):
        if name=='submitForce':
            rid,seq,target,p,sig=args;policy_valid(p);ph=g.struct('Policy',g.POLICY,p)[1]
            body=self.signed('authority','Force',['address','bytes32','uint64','uint64','bytes32'],[sender,rid,seq,target,ph],sig,'governance')
        else:
            rid,seq,p,target,origin,sig=args;policy_valid(p);ph=g.struct('Policy',g.POLICY,p)[1]
            body=self.signed('authority','Resume',['address','bytes32','uint64','bytes32','uint64','uint64'],[sender,rid,seq,ph,target,origin],sig,'governance')
        return rid,seq,body

    def batch_args(self,args,sender):
        rid,b=args;prev,nxt,commitment,da,first,last,gpos,signature=b
        need(sender==raw(self.cfg['submitter']),'unregistered submitter')
        cursor_valid(prev);cursor_valid(nxt)
        need(nxt[3]==1 and nxt[4]==gpos and 1<=len(gpos)<=1024,'GLOBAL source != gpos')
        need(first==prev[0]+1 and last==nxt[0] and 1<=last-first+1<=2,'invalid range')
        self.signed('inbox','DAReceipt',['bytes32'],[commitment],da,'da')
        vals=[g.cursor_hash(prev)[1],g.cursor_hash(nxt)[1],commitment,c.keccak256(da),first,last,c.keccak256(gpos)]
        body=self.signed('inbox','BatchAttestation',['bytes32']*4+['uint64','uint64','bytes32'],vals,signature,'certifier')
        return rid,body

    def receipt(self,h,tx,r,events):
        block=q(h['number']);idx=q(tx['transactionIndex']);txhash=norm(tx['hash']);sender=raw(tx['from'])
        if q(r['status'])==0:
            need(not r['logs'],'failed receipt contains surviving logs');self.failed.append(txhash);return
        call=decode_call(tx)
        if call is None:
            for name,values,pos in events:
                need(name=='CursorInitialized','unrecognized successful call with protocol events')
                need(self.cursor is None,'duplicate initialization');v=values[0];cursor_valid(v)
                need(v==[self.cfg['genesisHeight'],self.cfg['genesisOrigin'],b'\0'*32,0,b''],'genesis cursor mismatch')
                need(v[1]<=block,'future genesis origin')
                self.cursor=v;self.written=pos
                self.events.append({'kind':name,'position':pos,'cursor':jsonable(v)})
            return
        name,args=call
        if name in ('submitForce','submitResume'):
            need(raw(tx['to'])==self.addresses['authority'],'control wrong destination')
            rid,seq,body=self.control_args(name,args,sender);key=('authority',sender,rid)
        elif name=='submitBatch':
            need(raw(tx['to'])==self.addresses['inbox'],'batch wrong destination')
            rid,body=self.batch_args(args,sender);key=('inbox',sender,rid)
        else:
            raise ValueError('direct successful Override outside production contract entry')
        if key in self.requests:
            need(self.requests[key]==body,'successful conflicting req')
            need(not events,'idempotent retry emitted protocol logs');self.retries.append(txhash);return
        if name in ('submitForce','submitResume'):
            need(self.seq is None or seq>self.seq,'non-increasing control seq')
            act=q(h['timestamp'])+self.cfg['announcementSeconds'];need(act<1<<64,'act overflow')
        if name=='submitForce':
            need(len(events)==1 and events[0][0]=='ForceRecorded','Force event count')
            _,v,pos=events[0];rid,seq,target,p,sig=args
            need(v==[sender,rid,seq,target,p,act],'Force calldata/event mismatch')
            self.events.append({'kind':'ForceRecorded','position':pos,'seq':seq,'act':act})
            self.seq=seq
        elif name=='submitResume':
            need(len(events)==2 and [e[0] for e in events]==['CursorSet','ResumeRecorded'],'Set/Resume event count or order')
            rid,seq,p,target,origin,sig=args
            expected=[target,origin,b'\0'*32,2,c.abi_encode(['uint64'],[seq])]
            need(self.cursor is not None and target>self.cursor[0] and origin<=block,'Resume bounds')
            need(events[0][1]==[self.cursor,expected],'CursorSet old/next mismatch')
            need(events[1][1]==[sender,rid,seq,p,target,origin,act],'Resume calldata/event mismatch')
            self.cursor=expected;self.written=events[0][2];self.seq=seq
            self.events.append({'kind':'CursorSet','position':events[0][2],'resumePosition':events[1][2],'seq':seq,'cursor':jsonable(expected)})
            self.events.append({'kind':'ResumeRecorded','position':events[1][2],'setPosition':events[0][2],'seq':seq,'act':act})
        else:
            need(len(events)==1 and events[0][0]=='CursorAdvanced','Batch event count')
            rid,b=args;prev,nxt,commitment,*_=b
            need(prev==self.cursor,'successful stale full prev');need(nxt[1]<=block,'future next origin')
            need(events[0][1]==[sender,rid,prev,nxt,commitment],'Batch calldata/event mismatch')
            self.cursor=nxt;self.written=events[0][2]
            self.events.append({'kind':'CursorAdvanced','position':self.written,'cursor':jsonable(nxt),'commitment':hx(commitment)})
        self.requests[key]=body

    def run(self):
        headers=self.bundle['blocks'];receipts=self.bundle['receipts']
        need(headers and q(headers[0]['number'])==0,'bundle must start at Anvil genesis')
        bytx={norm(r['transactionHash']):r for r in receipts}
        need(len(bytx)==len(receipts),'duplicate receipts')
        used=set();previous=None
        for expected_number,h in enumerate(headers):
            n=q(h['number']);need(n==expected_number,'noncontiguous blocks')
            need(q(h['timestamp'])==self.cfg['genesisTimestampSeconds']+n*self.cfg['slotSeconds'],'header timestamp != slot')
            if previous:need(norm(h['parentHash'])==norm(previous['hash']),'header parent mismatch')
            global_log=0
            for tx_index,tx in enumerate(h['transactions']):
                need(isinstance(tx,dict),'full transaction object required')
                txhash=norm(tx['hash']);need(txhash in bytx,'missing receipt')
                need(txhash not in used,'repeated tx in canonical blocks');used.add(txhash);r=bytx[txhash]
                for obj in (tx,r):
                    need(q(obj['blockNumber'])==n and norm(obj['blockHash'])==norm(h['hash']),'tx/receipt wrong block')
                    need(q(obj['transactionIndex'])==tx_index,'tx/receipt index mismatch')
                need(q(r['status']) in (0,1),'invalid receipt status')
                events=[]
                for local_log,log in enumerate(r['logs']):
                    need(not log.get('removed',False),'removed log')
                    need(q(log['blockNumber'])==n and norm(log['blockHash'])==norm(h['hash']),'log wrong block')
                    need(norm(log['transactionHash'])==txhash and q(log['transactionIndex'])==tx_index,'log wrong tx')
                    need(q(log['logIndex'])==global_log,'block-global logIndex gap/conflict')
                    global_log+=1
                    addr=raw(log['address'])
                    if addr not in self.addresses.values():continue
                    name,values=decode_event(log)
                    role='authority' if name in ('ForceRecorded','ResumeRecorded') else 'inbox'
                    need(addr==self.addresses[role],'event wrong configured emitter')
                    events.append((name,values,[n,tx_index,local_log]))
                self.receipt(h,tx,r,events)
            self.history[str(n)]={'cursor':jsonable(self.cursor),'writtenAt':self.written}
            previous=h
        need(used==set(bytx),'receipts outside provided complete block prefix')
        for read in self.bundle.get('historicalCursorReads',[]):
            n=read['height'];need(str(n) in self.history,'historical read outside evidence prefix')
            cur=abi_decode([g.CURSOR],read['returnData'])[0]
            need(jsonable(cur)==self.history[str(n)]['cursor'],'historical eth_call disagrees with receipt replay')
            if 'writtenAt' in read:need(read['writtenAt']==self.history[str(n)]['writtenAt'],'historical written_at mismatch')
        return {'status':'PASS','scope':'offline RPC consistency, signatures, acceptance/events and historical cursor only',
                'blockCount':len(headers),'receiptCount':len(receipts),'failedReceipts':self.failed,
                'idempotentRetries':self.retries,'events':self.events,'history':self.history,
                'notEstablished':['Ethereum proof verification','Clock/transport integration','driver commands actually executed','finalized/delivery grant discipline','DA persistence or D/ResumeCut']}


def selftest():
    artifact=json.loads((VERIFY/'golden-vectors.json').read_text())
    for name in CALLS:
        data=artifact['calldata'][name]['arguments']
        values=abi_decode(CALLS[name],data)
        need(hx(c.abi_encode(CALLS[name],values))==data,'business roundtrip')
        try:abi_decode(CALLS[name],data+'00');raise AssertionError('trailing byte accepted')
        except ValueError:pass
    return {'status':'PASS','scope':'ABI decoder preparation only','businessCalldataShapes':len(CALLS),'noncanonicalTrailingBytesRejected':len(CALLS),'realReceiptsChecked':0}


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--verification-dir',type=Path);p.add_argument('--bundle',type=Path);p.add_argument('--self-test',action='store_true')
    a=p.parse_args()
    if a.self_test:result=selftest()
    elif a.bundle:result=Checker(json.loads(a.bundle.read_text())).run()
    else:p.error('provide --bundle or --self-test')
    print(json.dumps(result,indent=2))

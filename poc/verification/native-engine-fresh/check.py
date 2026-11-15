#!/usr/bin/env python3
"""Independent full-artifact checker for NATIVE-ENGINE/FRESH. Input is read-only."""
import argparse
import collections
import copy
import hashlib
import json
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[3]
BIN = ROOT / '.build/target/debug/native-engine-verifier'
BASE_FIELDS = 'profile package commonware scenario seed registry_hash epoch start stop cutoff epsilon lease refresh clock_transit link_delay link_jitter queue_capacity overflowing quota_per_second mailbox_capacity grpc_max_bytes payload_bound work_us payload_size virtual_crypto_us leader_timeout certification_timeout timeout_retry fetch_timeout fetch_retry height_limit sections view_retention scripted body_hold_until close_on_work lose_clock_after startup_skew stress_tick stress_count'.split()

class Invalid(Exception): pass
def need(ok, rule):
    if not ok: raise Invalid(rule)
def b(value): return bytes(value)
def u64(n): return int(n).to_bytes(8, 'big')
def u32(n): return int(n).to_bytes(4, 'big')
def h(domain, *parts):
    return hashlib.sha256(b''.join(u64(len(p))+p for p in [domain.encode(), *parts])).digest()
def canonical(value): return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode()
def one(value):
    need(isinstance(value, dict) and len(value)==1, 'required single typed variant')
    return next(iter(value.items()))
def hx(value): return b(value).hex()

class Oracle:
    def __init__(self):
        self.process = subprocess.Popen([str(BIN)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        self.cache={};self.context=None
    def ask(self, op, **data):
        request=canonical({'op':op,**data})
        if op=='init':self.context=tuple(data['namespace'])
        key=(self.context,request)
        if op=='init' or key not in self.cache:
            self.process.stdin.write(request.decode()+'\n');self.process.stdin.flush()
            reply=json.loads(self.process.stdout.readline())
            self.cache[key]=reply
        reply=self.cache[key]
        need('ok' in reply, f'native {op}: {reply.get("error")}')
        return reply['ok']
    def close(self):
        self.process.stdin.close();self.process.wait(timeout=5)

def config_check(c, oracle):
    base=c['base'];need(list(base)==BASE_FIELDS,'canonical base profile order')
    need(len(c['members'])==5 and [m['member'] for m in c['members']]==list(range(5)),'five logical members')
    expected={'profile':'native-engine-fresh-v1','package':'0.1.0','commonware':'2026.10.0+controlled-spawn-v1','epoch':1,'start':1000000,'stop':1800000,'cutoff':1900000,'epsilon':[1000,-1000],'lease':100000,'refresh':20000,'clock_transit':250,'link_delay':20000,'link_jitter':0,'queue_capacity':64,'overflowing':'drop_newest','mailbox_capacity':64,'grpc_max_bytes':1048576,'payload_bound':65536,'work_us':5000,'payload_size':256,'virtual_crypto_us':0,'leader_timeout':200000,'certification_timeout':400000,'timeout_retry':100000,'fetch_timeout':100000,'fetch_retry':20000,'sections':list(range(1,9)),'view_retention':16,'startup_skew':20000}
    for key,value in expected.items():need(base[key]==value,'resolved profile '+key)
    need(base['seed'] in [7,42,99],'approved seed')
    fields=[]
    for member in c['members']:fields.extend([u32(member['member']),b(member['identity']),b(member['bls_key']),b(member['pop'])])
    registry=h('I1b/voter-registry/v1',*fields);need(registry==b(base['registry_hash']),'registry hash')
    config=h('I1b/fresh-config/v1',canonical(base),registry);need(config==b(c['config_hash']),'config hash')
    run=h('I1b/fresh-run/v1',config);genesis=h('I1b/fresh-genesis/v1',config);cid=h('I1b/fresh-cid/v1',run,genesis,u64(1));ns=h('I1b/voter-namespace/v1',run,config,cid,u64(1))
    for key,value in [('run_id',run),('genesis',genesis),('cid',cid),('namespace',ns),('native_config_hash',h('I1b/fresh-native-config/v1',config,registry,ns)),('app_fixture_hash',h('I1b/fresh-app/v1',cid,genesis,u64(base['height_limit']),u32(256),u64(5000)))]:need(b(c[key])==value,'derived '+key)
    result=oracle.ask('init',members=c['members'],namespace=c['namespace'],epoch=1)
    need(b(result['registry_hash'])==registry and result['quorum']==4,'native registry/PoP/quorum')
    s=base['scenario'];need(s in ['healthy','two-real-alone','proof-before-body','close-propose','close-verify','close-certify','lease-loss','wrong-context','native-proof-negatives','quota','router-overflow'],'scenario')
    need(base['height_limit']==(2 if s in ['healthy','two-real-alone','lease-loss'] else 1),'height profile')
    need(base['scripted']==(s not in ['two-real-alone','quota','router-overflow']),'script profile')
    need(base['quota_per_second']==(0 if s=='router-overflow' else 64),'quota profile')
    need(base['body_hold_until']==(1250000 if s=='proof-before-body' else None),'body hold profile')
    need(base['lose_clock_after']==(1050000 if s=='lease-loss' else None),'lease-loss profile')
    need(base['close_on_work']=={'close-propose':1,'close-verify':2,'close-certify':3}.get(s),'close profile')

def block(raw, c):
    raw=b(raw);need(len(raw)==333 and raw[0]==1 and raw[1:33]==b(c['cid']) and raw[73:77]==u32(256),'canonical block framing')
    height=int.from_bytes(raw[33:41],'big');parent=raw[41:73]
    need(1<=height<=c['base']['height_limit'] and raw[77:]==bytes((height+i)%256 for i in range(256)),'canonical block height/payload')
    return {'digest':h('I1b/fixture-block/v1',raw),'height':height,'parent':parent,'bytes':raw}

def crc(raw):
    c=0xffffffff
    for x in raw:
        c^=x
        for _ in range(8):c=(c>>1)^(0x82f63b78 if c&1 else 0)
    return c^0xffffffff
def page_decode(raw):
    need(len(raw)==256,'physical page size');valid=[]
    for slot in range(2):
        at=244+6*slot;n=int.from_bytes(raw[at:at+2],'big');sum_=int.from_bytes(raw[at+2:at+6],'big')
        if 0<n<=244 and crc(raw[:n])==sum_:valid.append((n,-slot))
    need(valid,'valid page CRC');n,neg=max(valid);return -neg,n
def container_data(raw):
    need(4096<=len(raw) and raw[:8]==b'CWIK\x00\x01\x00\x00' and int.from_bytes(raw[8:12],'big')==crc(raw[:8]) and raw[12:4096]==bytes(4084),'native V1 header/app version/CRC/padding')
    return raw[4096:]
def varint(raw,at):
    begin=at;n=0;shift=0
    while at<len(raw):
        x=raw[at];at+=1;n|=(x&127)<<shift
        need(shift<35,'native u32 varint bound')
        if x<128:
            need(n<=0xffffffff and (at-begin==1 or x!=0),'canonical native varint');return n,at
        shift+=7
    return None,begin
def frames(raw, strict=False):
    need(len(raw)<=65536 and len(raw)%256==0,'bounded native image')
    logical=bytearray();pages=[]
    for at in range(0,len(raw),256):
        p=raw[at:at+256];slot,n=page_decode(p);need(at+256==len(raw) or n==244,'nonfinal full page')
        logical.extend(p[:n]);pages.append({'physical_offset':at,'selected_slot':slot,'selected_length':n,'bytes':list(p)})
    out=[];at=0
    while at<len(logical):
        n,body=varint(logical,at)
        if n is None or body+n>len(logical):need(not strict,'complete snapshot frame');break
        need(0<n<=4096 and len(out)<1024,'bounded native frame')
        end=body+n;out.append({'logical_offset':at,'frame_bytes':list(logical[at:end]),'artifact_bytes':list(logical[body:end]),'pages':pages[at//244:(end-1)//244+1]});at=end
    return out

def journal(path, magic, domain, c, worker):
    raw=path.read_bytes();head=path.with_name(path.name+'.head').read_bytes()
    need(len(head)==88 and head[:8]==magic,'journal protected head magic/size')
    need(head[56:]==h(domain+'-head/v1',b(c['run_id']),b(c['config_hash']),u32(worker),head[:56]),'journal head checksum')
    limit,seq=struct.unpack('>QQ',head[8:24]);need(limit==len(raw),'committed journal extent')
    records=[];receipts={};at=0;previous=bytes(32)
    while at<len(raw):
        need(at+16<=len(raw),'complete journal header');n,inv=struct.unpack('>QQ',raw[at:at+16]);need(n>0 and n<=16*1024*1024 and n^inv==2**64-1,'protected record length')
        end=at+48+n;need(end<=len(raw),'complete journal frame');payload=raw[at+16:at+16+n];digest=raw[at+16+n:end];need(digest==h(domain+'-record/v1',payload),'record checksum')
        rec=json.loads(payload);need(canonical(rec)==payload,'compact ordered record JSON');need(rec['schema']==1 and rec['sequence']==len(records)+1 and b(rec['previous_record_hash'])==previous,'journal sequence/hash chain')
        records.append(rec);previous=digest;receipts[rec['sequence']]=digest;at=end
    need(seq==len(records) and head[24:56]==previous,'journal committed head matches chain')
    return records,receipts

class Witness:
    def __init__(self,path,c,node,oracle):
        self.records,self.receipts=journal(path,b'I1WHEAD1','I1b/witness',c,node);self.witnesses={};self.eligible={};self.routes={};self.entry_seq={};self.certificate_facts=[];blobs={};images={};last_io=0;sealed=[]
        for record in self.records:
            need(record['identity']=={'run_id':c['run_id'],'config_hash':c['config_hash'],'worker':node},'witness ledger identity')
            for entry in record['entries']:
                kind=entry['entry'];v=entry['value']
                if kind=='policy':
                    need(record['sequence']==1 and len(record['entries'])==1,'initial witness policy')
                    need(v=={'config':{'run_id':c['run_id'],'config_hash':c['config_hash'],'cid':c['cid'],'worker':node,'member':node,'epoch':1,'generation':1,'partition':'i1b-native-voter','sections':list(range(1,9))},'registry_hash':c['base']['registry_hash'],'namespace_hash':c['namespace'],'local_signer':c['members'][node]['signer_index']},'witness policy binding');continue
                if kind in ['blob','barrier','observation']:
                    io=v['open_id'] if kind=='blob' else v['io_id'];need(io>last_io,'witness I/O chronology');last_io=io
                if kind=='blob':
                    ident=h('I1b/blob/v1',b(c['run_id']),b(c['config_hash']),u32(node),u64(1),b'i1b-native-voter',u64(v['section']),u64(v['open_id']))
                    need(ident==b(v['id']) and 1<=v['section']<=8,'witness blob identity');blobs[ident]=v
                    for old in list(images):
                        if blobs[old]['section']==v['section']:del images[old]
                    data=bytearray(v['initial_bytes']);images[ident]={'data':data,'cover':[None]*len(data),'extent':0,'extent_id':None,'uncertain':False}
                elif kind=='observation':
                    op=v['change'];tag=op['operation']
                    if tag in ['write','resize']:
                        image=images[b(op['blob_id'])];need(not image['uncertain'],'uncertain mutation')
                        if tag=='write':self.write(image,op['offset'],b(op['bytes']))
                        else:self.resize(image,op['len'])
                    elif tag=='uncertain':
                        for ident,image in images.items():
                            if op['section'] is None or blobs[ident]['section']==op['section']:image['uncertain']=True
                    elif tag=='remove':
                        need(not any(op['section'] is None or w['frame']['section']==op['section'] for w in sealed),'protected removal')
                        images={ident:image for ident,image in images.items() if op['section'] is not None and blobs[ident]['section']!=op['section']}
                    else:raise Invalid('unknown observation')
                elif kind=='barrier':
                    ident=b(v['blob_id']);image=images[ident];data=b(v['bytes']);start=v['offset'];end=start+len(data)
                    need(data and end<=65536 and not image['uncertain'],'barrier range/uncertainty')
                    expected=h('I1b/native-barrier/v1',ident,u64(v['io_id']),bytes([v['kind']]),u64(start),u64(len(data)),h('I1b/barrier-bytes/v1',data));need(expected==b(v['id']),'barrier identity')
                    if v['kind']==1:self.write(image,start,data)
                    else:need(v['kind']==2 and start==0 and data==image['data'],'full barrier current bytes/extent')
                    image['cover'][start:end]=[expected]*len(data)
                    if end>=image['extent']:image['extent']=end;image['extent_id']=expected
                elif kind=='witness':
                    ident=b(v['blob_id']);image=images[ident];f=v['frame'];need(not image['uncertain'],'uncertain witness')
                    candidates=frames(image['data']);match=next((a for a in candidates if a['logical_offset']==f['logical_offset']),None);need(match is not None and all(match[k]==f[k] for k in match),'actual current frame boundary/evidence')
                    need(f['section']==blobs[ident]['section'],'witness section')
                    refs={image['extent_id']}
                    for page in f['pages']:
                        off=page['physical_offset'];need(image['extent']>=off+256,'durable physical page extent');foot=244+page['selected_slot']*6
                        for pos in list(range(page['selected_length']))+list(range(foot,foot+6)):refs.add(image['cover'][off+pos])
                    need(None not in refs and [list(x) for x in sorted(refs)]==v['barrier_ids'],'current witness CRC byte coverage')
                    crypto=oracle.ask('artifact',bytes=f['artifact_bytes']);need(crypto['valid'] and crypto['epoch']==1 and crypto['view']==f['section'] and crypto['wire']==f['wire'],'native artifact/wire identity')
                    if f['wire'] and f['wire']['channel']==1:need(crypto['proof']['signer_index']==c['members'][node]['signer_index'],'own voter artifact signer')
                    if f['wire'] and f['wire']['channel']==2:self.certificate_facts.append((record['sequence'],crypto['proof']))
                    ah=h('I1b/native-artifact/v1',b(f['artifact_bytes']));need(ah==b(v['artifact_hash']),'artifact hash')
                    wh=h('I1b/artifact-witness/v1',b(c['run_id']),b(c['config_hash']),b(c['cid']),u64(1),ident,u64(f['section']),u64(f['logical_offset']),ah,b''.join(sorted(refs)));need(wh==b(v['id']) and wh not in self.witnesses,'witness identity/uniqueness');self.witnesses[wh]=v;sealed.append(v);self.entry_seq[wh]=record['sequence']
                elif kind=='eligible':
                    w=self.witnesses[b(v['witness_id'])];need(v['wire']==w['frame']['wire'],'eligibility exact native wire');wire=v['wire'];dh=h('I1b/native-wire/v1',u32(wire['channel']),b(wire['bytes']));need(dh==b(v['wire_hash']),'eligible wire hash');eid=h('I1b/outbox-payload/v1',b(c['run_id']),b(c['config_hash']),b(c['cid']),u64(1),u32(node),u32(wire['channel']),dh);need(eid==b(v['id']) and eid not in self.eligible,'eligible identity');self.eligible[eid]=v;self.entry_seq[eid]=record['sequence']
                elif kind=='submission':
                    pid=b(v['payload_id']);need(pid in self.eligible and 0<=v['destination']<5 and v['destination']!=node,'route recipient/payload');rid=h('I1b/outbox/v1',pid,u32(v['destination']));need(rid==b(v['id']) and rid not in self.routes,'route authorization identity');self.routes[rid]=v;self.entry_seq[rid]=record['sequence']
                else:raise Invalid('unexpected fresh witness entry '+kind)
        partition=path.parent/'native/i1b-native-voter';expected_files=set()
        for ident,image in images.items():
            name=f"{blobs[ident]['section']:016x}";expected_files.add(name);raw=(partition/name).read_bytes();need(container_data(raw)==image['data'],'actual native voter bytes equal observed image');frames(image['data'],strict=True)
        actual_files={p.name for p in partition.iterdir() if p.is_file()} if partition.exists() else set()
        need(actual_files==expected_files,'exact selected native voter file set')
    @staticmethod
    def resize(image,n):
        need(0<=n<=65536,'image extent');old=len(image['data'])
        if n<old:del image['data'][n:];del image['cover'][n:]
        else:image['data'].extend(bytes(n-old));image['cover'].extend([None]*(n-old))
        image['extent']=min(image['extent'],n)
    @staticmethod
    def write(image,start,data):
        need(start>=0 and start+len(data)<=65536,'image write bound')
        if not data:return
        if start+len(data)>len(image['data']):Witness.resize(image,start+len(data))
        for i,value in enumerate(data,start):
            if image['data'][i]!=value:image['cover'][i]=None;image['data'][i]=value
    def receipt(self,r):need(r and self.receipts.get(r['sequence'])==b(r['hash']),'witness receipt prefix')
    def output(self,e):
        p=e['provenance']['LocalWitness'];self.receipt(p['receipt']);rid=b(p['authorization_id']);need(rid in self.routes and self.entry_seq[rid]<=p['receipt']['sequence'],'committed route reference');route=self.routes[rid];payload=self.eligible[b(route['payload_id'])];need(route['destination']==e['dst_member_id'] and payload['wire']=={'channel':e['channel'],'bytes':e['payload_bytes']},'authorized exact wire/destination');need(p['witness_ids']==[payload['witness_id']] and self.entry_seq[b(payload['witness_id'])]<=p['receipt']['sequence'],'exact committed witness reference')

def envelope(e,c,oracle,allow_invalid=False):
    channel=e['channel'];need(channel in [1,2,3,4] and 0<=e['src_member_id']<5 and 0<=e['dst_member_id']<5 and e['src_member_id']!=e['dst_member_id'],'envelope channel/members')
    need(b(e['run_id'])==b(c['run_id']) and b(e['cid'])==b(c['cid']) and e['epoch']==1,'envelope context')
    need(1<=e['attempt']<=0xffffffff and len(e['payload_bytes'])<=65536,'attempt/payload bound')
    digest=h('I1b/native-wire/v1',u32(channel),b(e['payload_bytes']));need(digest==b(e['payload_hash']),'envelope payload hash')
    mid=h('I1b/native-message/v1',b(c['run_id']),b(c['config_hash']),b(c['cid']),u64(1),u32(e['src_member_id']),u32(e['dst_member_id']),u32(channel),digest);need(mid==b(e['message_id']),'message identity')
    provenance,proof=one(e['provenance']);need(provenance in ['Scripted','LocalWitness','Resolver','Body'],'typed provenance')
    if provenance=='Scripted':need(e['src_member_id']>=2 and len(proof['transcript_id'])==32 and len(proof['entry_id'])==32,'scripted origin')
    else:need(e['src_member_id']<2,'physical worker origin')
    decoded=None
    if channel in [1,2]:
        need(provenance in ['Scripted','LocalWitness'] and e['priority'],'voter provenance/priority')
        decoded=oracle.ask('vote' if channel==1 else 'certificate',bytes=e['payload_bytes'])
        if not allow_invalid:need(decoded['valid'],'native envelope signature/quorum')
        if channel==1 and not allow_invalid:need(decoded['signer_index']==c['members'][e['src_member_id']]['signer_index'],'native vote origin signer')
    elif channel==3:
        need(provenance in ['Scripted','Resolver'],'resolver provenance')
        raw=b(e['payload_bytes']);need(len(raw)>=9,'native resolver prefix');request=int.from_bytes(raw[:8],'big');tag=raw[8]
        if provenance=='Resolver':need(proof['request_id']==request and b(proof['source_record_id'])==h('I1b/resolver-source/v1',raw),'resolver source binding')
        if tag==0:need(len(raw)==17,'resolver request encoding');decoded={'operation':'request','request':request,'view':int.from_bytes(raw[9:17],'big')}
        elif tag==2:need(len(raw)==9,'resolver error encoding');decoded={'operation':'error','request':request}
        else:
            need(tag==1,'resolver variant');size,at=varint(raw,9);need(size is not None and at+size==len(raw),'resolver response frame');cert=oracle.ask('certificate',bytes=list(raw[at:]));need(cert['valid'],'resolver certificate');decoded={'operation':'response','request':request,'certificate':cert}
    else:
        need(provenance in ['Scripted','Body'],'BODY provenance');kind,packet=one(oracle.ask('body',bytes=e['payload_bytes'])['kind']);need(packet['cid']==c['cid'],'BodyPacket context')
        if kind=='Request':need(packet['request_id']>0 and len(packet['wanted_digest'])==32,'body request');content=b(packet['wanted_digest']);request=packet['request_id'];decoded={'kind':kind,'content':content,'request':request}
        else:
            need(kind in ['Push','Reply'],'body kind');value=block(packet['block_bytes'],c);request=0 if kind=='Push' else packet['request_id'];need(kind=='Push' or request>0,'body reply request');content=value['digest'];decoded={'kind':kind,'content':content,'request':request,'block':value}
        if provenance=='Body':need(b(proof['content_id'])==content and proof['request_id']==request,'body content/request provenance')
    return decoded

class Checker:
    def __init__(self,path,oracle,rows=None,roots=None,rawframes=None):
        self.path=Path(path);self.oracle=oracle;self.c=json.loads((self.path/'scenario.json').read_text());config_check(self.c,oracle)
        self.rows=rows if rows is not None else [json.loads(x) for x in (self.path/'virtual-trace.jsonl').read_text().splitlines()]
        self.roots=[];self.root_hash=[];self.w=[]
        for node in [0,1]:
            records,receipts=journal(self.path/f'worker-{node}/root-step',b'I1SHEAD1','I1b/root-step',self.c,node)
            if roots is not None:records,receipts=roots[node]
            self.roots.append(records);self.root_hash.append(receipts);self.w.append(Witness(self.path/f'worker-{node}/witness',self.c,node,oracle))
        self.rawframes=rawframes if rawframes is not None else oracle.ask('frames',path=str(self.path/'frames.bin'))
        self.metrics=collections.Counter();self.samples={};self.current=[None,None];self.closed=[False,False];self.start=[None,None];self.tasks=[set(),set()];self.sign_cache={};self.work=[{},{}];self.generations=[0,0];self.available=[set(),set()];self.persisted=[set(),set()];self.proofs=[set(),set()];self.delivered=[0,0];self.tips=[b(self.c['genesis'])]*2;self.timers={};self.attempts={};self.sent={};self.routes=collections.defaultdict(collections.deque);self.root_index=[0,0];self.body_requests=[{},{}];self.known={};self.last_key=None;self.script_seq=collections.Counter();self.script_entries={};self.script_bodies={i:set() for i in [2,3,4]};self.script_proposals={i:set() for i in [2,3,4]};self.script_signed={};self.quota={};self.first_close_work=None;self.close_ticks={};self.proof_wait=[];self.pending_grant=None;self.frames_seen=[]
        self.sign_meta={};self.witness_floor=[1,1]
        parent=b(self.c['genesis'])
        for height in range(1,self.c['base']['height_limit']+1):
            raw=bytes([1])+b(self.c['cid'])+u64(height)+parent+u32(256)+bytes((height+i)%256 for i in range(256));value=block(raw,self.c);self.known[value['digest']]=value;parent=value['digest']
    def valid_private_chain(self,node,digest):
        seen=set()
        while digest!=b(self.c['genesis']):
            if digest in seen or digest not in self.available[node] or digest not in self.known:return False
            seen.add(digest);digest=self.known[digest]['parent']
        return True
    def binding(self,value,node):need(value=={'schema_version':1,'run_id':self.c['run_id'],'config_hash':self.c['config_hash'],'worker_id':node,'member_id':node,'incarnation':1},'worker binding')
    def check_key(self,key):
        if key is None:return
        need(key['event_id']>0 and b(key['tie_rank'])==h('I1b/fresh-tie/v1',u64(self.c['base']['seed']),u64(key['event_id'])),'event tie hash')
        order=(key['tick_us'],key['causal_wave'],b(key['tie_rank']),key['event_id'])
        if self.last_key is not None:need(order>=self.last_key,'ordered hub event queue')
        self.last_key=order
    def quota_take(self,node,channel,dest,tick,accepted):
        if self.c['base']['quota_per_second']==0:return
        key=(node,channel,dest);tat=self.quota.get(key,tick)
        allowed=tick>=tat-63*15625
        need(allowed==accepted,'per-recipient/channel quota')
        if allowed:self.quota[key]=max(tat,tick)+15625
    def root_step(self,node,g,a):
        i=self.root_index[node];need(i<len(self.roots[node]),'Ack missing durable root record');record=self.roots[node][i];self.root_index[node]+=1
        expected=copy.deepcopy(a);expected['durable_step_hash']=[]
        need(record['grant']==g and record['ack']==expected,'exact grant/logical Ack in root journal')
        need(a['durable_step_sequence']==i+1 and b(a['durable_step_hash'])==self.root_hash[node][i+1],'root step receipt hash')
    def runtime(self,node,g,a,events):
        tick=g['tick_us'];step=g['logical_step_id'];need(events and events[0]=={'kind':'grant','grant':step,'tick_us':tick} and events[-1]=={'kind':'ack','grant':step},'complete runtime grant/ack audit')
        polled=set();current_io=None;issued={};ready=set();sample_input=one(g['input']['kind'])
        for e in events:
            tag=e['kind']
            if tag=='sample_accepted':
                need(sample_input[0]=='Sample','Sample accepted without delivery');s=sample_input[1];need((node,s['request_id'],s['sequence']) in self.samples and self.samples[(node,s['request_id'],s['sequence'])]==s,'actual clock service sample')
                need(s['as_of_tick_us']+250==tick and tick<s['lease_end_tick_us'],'clock service/delivery/lease ordering');need(s['node_time_at_as_of_us']==s['as_of_tick_us']+self.c['base']['epsilon'][node],'authority oscillator');self.current[node]=s
            elif tag=='task_registered':need(e['task'] not in self.tasks[node] and e['cid']==self.c['cid'],'unique controlled task');self.tasks[node].add(e['task'])
            elif tag=='poll':need(e['task'] in self.tasks[node] and e['grant']==step and current_io is None,'controlled poll identity/order');polled.add(e['task'])
            elif tag=='clock_read':
                v=e['snapshot'];need(v['cid']==self.c['cid'] and v['grant_id']==step and v['tick_us']==tick,'clock audit grant/context')
                s=self.current[node];expected=None if s is None or tick>=s['lease_end_tick_us'] else tick+self.c['base']['epsilon'][node]
                need(v['local_time_us']==expected,'single calibrated node projection');need(v['task_id']==0 or v['task_id'] in polled,'clock read controlled task')
                if s is not None:need(v['sample_sequence']==s['sequence'] and v['lease_end_tick_us']==s['lease_end_tick_us'],'clock calibration identity')
                self.metrics['clock_reads']+=1
            elif tag=='io_issued':
                identity=e['identity'];need(identity['node']==node and identity['grant']==step and identity['tick_us']==tick and identity['cid']==self.c['cid'] and identity['run_id']==self.c['run_id'] and identity['incarnation']==1 and identity['task'] in polled,'native I/O controlled binding');issued[identity['io_id']]=e
            elif tag=='io_ready':need(e['io_id'] in issued and current_io is None,'I/O ready ordering');current_io=e['io_id'];ready.add(current_io)
            elif tag in ['io_released','io_discarded']:need(current_io==e['io_id'],'I/O release custody');current_io=None
            elif tag=='witness_committed':
                need(self.w[node].receipts.get(e['sequence'])==b(e['head_hash']) and e['sequence']>self.witness_floor[node],'runtime witness receipt progression');self.witness_floor[node]=e['sequence']
            elif tag=='retired':need(e['cid']==self.c['cid'],'retired cid');self.closed[node]=True;self.close_ticks.setdefault(node,tick)
            elif tag=='failure':raise Invalid('successful run has runtime failure')
        need(current_io is None and len(ready)==len(issued),'quiescence before runtime Ack')
        return polled,issued
    def work_events(self,node,g,a,app,issued):
        tick=g['tick_us'];input_kind,input_value=one(g['input']['kind']);generated=[];audit_proofs=[];audit_delivered=[]
        for e in app['work']:
            tag=e['event']
            if tag in ['work_pending','work_started','work_completed']:
                binding=e['binding'];work=binding['work'];ident=b(work['operation_id']);need(binding['member']==node and binding['epoch']==1 and work['cid']==self.c['cid'] and work['kind'] in [1,2,3] and work['duration_us']==5000,'work context/kind/duration')
                wh=h('I1b/fresh-work/v1',b(self.c['cid']),u32(node),u32(work['kind']),u64(1),u64(binding['view']),u64(binding['parent_view']),b(binding['parent_digest']),b(binding['requested_payload']),u64(binding['request_generation']));need(wh==ident,'complete work identity')
                if tag=='work_pending':
                    need(binding['request_generation']==self.generations[node]+1 and binding['request_tick_us']==tick and ident not in self.work[node],'work request generation/tick');self.generations[node]+=1;self.work[node][ident]={'binding':binding,'due':None,'cancelled':False,'complete':False}
                    need(1<=binding['view']<=8,'native work view bound')
                    if work['kind'] in [1,2]:need(binding['parent_view']<binding['view'] and (binding['parent_view']==0)==(b(binding['parent_digest'])==b(self.c['genesis'])),'native callback parent view/context')
                    if work['kind']==1:need(binding['requested_payload']==[0]*32 and (binding['view']-1)%5==node,'native proposer/elector binding')
                    if work['kind']==3:need(binding['parent_view']==0 and binding['parent_digest']==[0]*32,'certify absent callback parent')
                else:
                    need(ident in self.work[node] and self.work[node][ident]['binding']==binding,'work original request binding');state=self.work[node][ident]
                    if tag=='work_started':
                        need(state['due'] is None and not state['cancelled'] and e['start_tick_us']==tick and e['due_tick_us']==tick+5000,'work actual start/due')
                        target=b(binding['parent_digest'] if work['kind']==1 else binding['requested_payload']);need(self.valid_private_chain(node,target),'work starts only with private body ancestry')
                        state['due']=e['due_tick_us'];generated.append(work);self.metrics['work_started']+=1
                        if self.first_close_work is None and self.c['base']['close_on_work']==work['kind']:self.first_close_work=(node,tick)
                    else:
                        need(input_kind=='WorkCompleted' and input_value==work and state['due']==tick and e['completed_tick_us']==tick and not state['cancelled'] and not state['complete'] and not self.closed[node],'registered timely live WorkCompleted')
                        state['complete']=True;self.metrics['work_completed']+=1
                        state['valid']=e['result_valid'];state['body_digest']=e['body_digest']
                    if e.get('body_digest') is not None:
                        digest=b(e['body_digest']);need(digest in self.known and b(e['body_parent'])==self.known[digest]['parent'],'work effective body/parent')
                        if work['kind']!=1:need(digest==b(binding['requested_payload']),'work payload binding')
                        if e['native_parent_view'] is not None:need(any(seq<=a['witness_receipt']['sequence'] and proof['view']==binding['view'] and proof['payload']==e['body_digest'] and proof['parent_view']==e['native_parent_view'] for seq,proof in self.w[node].certificate_facts),'work effective native parent is backed by verified certificate')
                    else:need(work['kind']==1 and self.known.get(b(binding['parent_digest']),{}).get('height')==self.c['base']['height_limit'],'only established height-limit invalidity in fixtures')
            elif tag=='work_discarded':
                ident=b(e['work']['operation_id']);need(e['cancel_tick_us']==tick,'work cancellation tick')
                if ident in self.work[node]:self.work[node][ident]['cancelled']=True
                self.metrics['work_discarded']+=1
            elif tag=='block_received':
                digest=b(e['digest']);need(digest in self.known and self.known[digest]['height']==e['height'],'received canonical body')
                if digest not in self.available[node]:
                    matching=[s for s in self.work[node].values() if s['complete'] and s['binding']['work']['kind']==1 and s['due']==tick]
                    need(matching,'private body received without network input or own build');self.available[node].add(digest)
            elif tag=='persisted':
                digest=b(e['digest']);need(digest in self.available[node],'persisted body availability');need(any(io['target']['partition']=='body-fixture' and b(io['target']['name'])==digest for io in issued.values()),'actual private native body I/O');self.persisted[node].add(digest)
                f=self.path/f'worker-{node}/native/body-fixture'/digest.hex();raw=f.read_bytes();need(container_data(raw)==self.known[digest]['bytes'],'actual persisted private body bytes')
            elif tag=='proof':
                p=e['proof'];proof=self.oracle.ask('finalization',bytes=p['finalization_bytes']);need(proof['valid'] and proof['kind']==3 and p['cid']==self.c['cid'] and p['epoch']==proof['epoch'] and p['view']==proof['view'] and p['payload_digest']==proof['payload'],'ProofObserved native finalization');digest=b(p['payload_digest']);self.proofs[node].add(digest);audit_proofs.append(p);self.metrics['proofs']+=1
                if not self.valid_private_chain(node,digest):self.proof_wait.append((node,tick))
            elif tag=='delivery':
                d=e['delivered'];value=block(d['block_bytes'],self.c);digest=value['digest'];need(d['cid']==self.c['cid'] and d['height']==value['height']==self.delivered[node]+1 and b(d['digest'])==digest and b(d['parent_digest'])==value['parent']==self.tips[node],'contiguous exact-once delivery')
                need(self.valid_private_chain(node,digest),'delivery requires privately available body prefix')
                covered=False
                for tip in self.proofs[node]:
                    if not self.valid_private_chain(node,tip):continue
                    while tip!=b(self.c['genesis']):
                        if tip==digest:covered=True;break
                        tip=self.known[tip]['parent']
                need(covered,'delivery requires actual finalized prefix proof');self.delivered[node]+=1;self.tips[node]=digest;audit_delivered.append(d);self.metrics['delivered']+=1
            elif tag in ['invalid_body','rejected_input']:self.metrics['input_rejections']+=1
            else:raise Invalid('unknown full application audit event '+tag)
        return generated,audit_proofs,audit_delivered

    def ack(self,key,a):
        need(self.pending_grant is not None,'unsolicited Ack');g=self.pending_grant;self.pending_grant=None;node=g['binding']['worker_id'];tick=g['tick_us'];self.binding(a['binding'],node)
        need(a['logical_step_id']==g['logical_step_id'] and a['event_id']==g['event_id'] and a['grant_hash']==g['grant_hash'] and a['status'] in [1,2],'Ack binding/status')
        need(a['logical_ack_hash']==self.oracle.ask('proto',type='ack',value=a)['hash'],'logical Ack hash');self.root_step(node,g,a);self.w[node].receipt(a['witness_receipt'])
        runtime=json.loads(b(a['runtime_events_json']));app=json.loads(b(a['application_events_json']));need(canonical(runtime)==b(a['runtime_events_json']) and canonical(app)==b(a['application_events_json']) and list(app)==['work','network','signing'],'complete canonical audit JSON')
        polled,issued=self.runtime(node,g,a,runtime);need(a['witness_receipt']['sequence']==self.witness_floor[node],'Ack contains current committed witness head');input_kind,input_value=one(g['input']['kind'])
        if input_kind=='Deliver' and a['status']==1:
            decoded=envelope(input_value,self.c,self.oracle)
            if input_value['channel']==4 and decoded['kind'] in ['Push','Reply']:self.available[node].add(decoded['content'])
        if input_kind=='TimerFired':
            t=input_value;known=self.timers.pop((node,t['id']),None);need(known==t and tick>=t['local_deadline_us']-self.c['base']['epsilon'][node],'original granted timer deadline')
        work,proofs,deliveries=self.work_events(node,g,a,app,issued)
        effects=collections.defaultdict(list)
        for effect in a['effects']:
            kind,value=one(effect['kind']);effects[kind].append(value)
        need(effects['Work']==work and effects['Sign']==app['signing'] and effects['Proof']==proofs and effects['Delivered']==deliveries,'typed effects equal complete application audit')
        for s in app['signing']:
            need(s['cid']==self.c['cid'] and s['epoch']==1 and s['grant_id']==g['logical_step_id'] and s['tick_us']==tick and s['task_id'] in polled and s['signer_index']==self.c['members'][node]['signer_index'],'sign controlled identity')
            need(s['kind'] in [1,2,3] and s['action'] in [1,2,3],'sign enums');cache=(node,s['kind'],s['epoch'],s['view'])
            if s['action']==3:need(not s['attestation_bytes'],'denied sign bytes');self.metrics['denied']+=1;continue
            sample=self.current[node];need(not self.closed[node] and sample is not None and s['sample_sequence']==sample['sequence'] and s['lease_end_tick_us']==sample['lease_end_tick_us'] and tick<s['lease_end_tick_us'] and s['local_time_us']==tick+self.c['base']['epsilon'][node] and s['local_stop_us']==1800000 and s['local_time_us']<1800000,'sign lease/stop/cid guard')
            verified=self.oracle.ask('attestation',bytes=s['attestation_bytes'],subject=s['subject_bytes'],kind=s['kind']);need(verified['valid'] and verified['epoch']==s['epoch'] and verified['view']==s['view'] and verified['signer_index']==s['signer_index'],'actual created/cached native attestation')
            value=(s['subject_bytes'],s['attestation_bytes'])
            if s['action']==1:need(cache not in self.sign_cache,'one underlying creation per subject');self.sign_cache[cache]=value;self.sign_meta[cache]=verified;self.metrics['created']+=1
            else:need(self.sign_cache.get(cache)==value,'cache hit exact prior attestation');self.metrics['cache_hits']+=1
            if s['kind'] in [2,3]:need((node,5-s['kind'],1,s['view']) not in self.sign_cache,'native nullify/finalize conflict')
            if s['kind']==3:need(b(verified['payload']) in self.persisted[node],'finalize only after durable certification body')
            if s['kind'] in [1,3]:
                candidates=[state for state in self.work[node].values() if state['complete'] and state.get('valid') and state.get('body_digest')==verified['payload'] and state['binding']['view']==s['view'] and state['binding']['work']['kind'] in ([1,2] if s['kind']==1 else [3])]
                need(candidates,'native sign follows matching completed application work')
                if s['kind']==1:need(any(state['binding']['parent_view']==verified['parent_view'] and b(state['binding']['parent_digest'])==self.known[b(verified['payload'])]['parent'] for state in candidates),'sign subject matches original application parent context')
        for t in effects['TimerArm']:need(t['cid']==self.c['cid'] and t['id']>0 and (node,t['id']) not in self.timers,'unique timer arm');self.timers[(node,t['id'])]=t
        for t in effects['TimerCancel']:
            if self.timers.get((node,t['id']))==t:self.timers.pop((node,t['id']))
        sends=effects['Send'];submits=[e for e in app['network'] if e['event']=='submit'];need(len(submits)==len(sends),'every actual Sender attempt audited')
        by_attempt={(b(e['message_id']),e['attempt']):e for e in sends}
        for event in app['network']:
            if event['event']=='rate_limited':self.quota_take(node,event['channel'],event['destination'],tick,False);self.metrics['rate_limited']+=1
            elif event['event']=='submit':
                e=by_attempt.get((b(event['message_id']),event['attempt']));need(e is not None and e['channel']==event['channel'] and e['dst_member_id']==event['destination'],'Sender audit/effect identity');self.quota_take(node,e['channel'],e['dst_member_id'],tick,True)
        for e in sends:
            need(e['src_member_id']==node and 'Scripted' not in e['provenance'],'worker output origin');decoded=envelope(e,self.c,self.oracle)
            if e['channel']==1:need(self.sign_cache.get((node,decoded['kind'],decoded['epoch'],decoded['view']))==(decoded['subject'],decoded['attestation']),'output vote has actual guarded worker SignAudit')
            if e['channel']==2:self.certificate_origins(decoded)
            if e['channel']==3 and decoded['operation']=='response':self.certificate_origins(decoded['certificate'])
            if e['channel'] in [1,2]:self.w[node].output(e)
            self.note_send(e,tick)
        for proof in proofs:self.certificate_origins(self.oracle.ask('finalization',bytes=proof['finalization_bytes']))
        for e in effects['Lifecycle']:
            need(e['cid']==self.c['cid'] and e['event'] in ['started','terminated'],'lifecycle context')
            if e['event']=='started':need(input_kind=='Start' and self.start[node] is None and self.current[node] is not None,'independent calibrated Enter');self.start[node]=tick
        self.metrics['acks']+=1;self.frames_seen.append({'direction':1,'worker':node,'frame':{'frame':{'Ack':a}}})
    def certificate_origins(self,proof):
        for member in self.c['members'][:2]:
            if member['signer_index'] in proof['signers']:
                key=(member['member'],proof['kind'],proof['epoch'],proof['view']);signed=self.sign_meta.get(key)
                need(signed is not None and signed['payload']==proof['payload'] and signed['parent_view']==proof['parent_view'],'certificate real signer has actual guarded worker evidence')
    def note_send(self,e,tick):
        ident=b(e['message_id']);need(e['attempt']==self.attempts.get(ident,0)+1,'one-based checked attempt sequence');self.attempts[ident]=e['attempt'];token=(ident,e['attempt']);need(token not in self.sent,'duplicate attempt');self.sent[token]=(e,tick);self.metrics['sends']+=1
    def run(self):
        injected=[];registered=[];script_certs={i:set() for i in [2,3,4]};consumed=set();run_complete=None
        for ordinal,row in enumerate(self.rows):
            try:
                tag=row['kind'];key=row['event'];detail=row['detail'];self.check_key(key);tick=key['tick_us'] if key else None
                if tag=='Registered':
                    node=detail['binding']['worker_id'];self.binding(detail['binding'],node);need(node in [0,1] and node not in registered and detail['registry_hash']==self.c['base']['registry_hash'] and detail['namespace_hash']==self.c['namespace'] and detail['profile']=='native-engine-fresh-v1','real registration');registered.append(node)
                elif tag=='ClockServiceSample':
                    s=detail;node=s['worker_id'];need(node in [0,1] and s['run_id']==self.c['run_id'] and s['incarnation']==1 and s['as_of_tick_us']==tick and s['lease_end_tick_us']==tick+100000 and s['node_time_at_as_of_us']==tick+self.c['base']['epsilon'][node] and s['uncertainty_us']==1000,'authority sample validity')
                    need(b(s['clock_domain'])==h('I1/clock-domain/v1',b(self.c['run_id']),b(self.c['config_hash']),u32(node)) and b(s['rate_profile'])==h('I1/rate/v1',b'fixed-rate-one-v1'),'clock domain/rate');self.samples[(node,s['request_id'],s['sequence'])]=s
                elif tag=='Grant':
                    g=detail;node=g['binding']['worker_id'];self.binding(g['binding'],node);need(node in registered and self.pending_grant is None,'one global outstanding grant')
                    need(g['event_id']==key['event_id'] and g['tick_us']==tick and g['causal_wave']==key['causal_wave'] and g['tie_rank']==key['tie_rank'] and g['grant_hash']==self.oracle.ask('proto',type='grant',value=g)['hash'],'grant hash/event binding')
                    need(g['logical_step_id']==self.metrics['acks']+1,'global logical step sequence');kind,value=one(g['input']['kind'])
                    if kind=='Deliver':
                        if value in injected:pass
                        else:envelope(value,self.c,self.oracle)
                    if kind=='Start':need(not self.closed[node] and self.current[node] is not None and tick<self.current[node]['lease_end_tick_us'],'live calibrated start')
                    self.pending_grant=g;self.frames_seen.append({'direction':0,'worker':node,'frame':{'frame':{'Grant':g}}})
                elif tag=='AckCommitted':
                    bad_input=self.pending_grant is not None and one(self.pending_grant['input']['kind'])[0]=='Deliver' and one(self.pending_grant['input']['kind'])[1] in injected
                    if bad_input:need(detail['status']==2,'injected malformed input must be rejected')
                    self.ack(key,detail)
                elif tag=='ScriptedSend':
                    e=detail;native=envelope(e,self.c,self.oracle);member=e['src_member_id'];need(member in [2,3,4] and e['channel']==1 and self.c['base']['scripted'],'explicit scripted endpoint')
                    provenance=e['provenance']['Scripted'];transcript=h('I1b/fresh-script/v1',b(self.c['run_id']),u32(member));need(b(provenance['transcript_id'])==transcript,'script transcript identity');entry=b(provenance['entry_id']);raw=b(e['payload_bytes'])
                    if entry not in self.script_entries:
                        self.script_seq[member]+=1;need(entry==h('I1b/fresh-script-entry/v1',transcript,u64(self.script_seq[member]),raw),'script entry identity');self.script_entries[entry]=raw
                        identity=(member,native['kind'],native['view']);need(identity not in self.script_signed,'script per-kind/view uniqueness');self.script_signed[identity]=raw
                        need((native['view'],b(native['payload'])) in self.script_proposals[member] and b(native['payload']) in self.script_bodies[member],'script received leader proposal and private body')
                        if native['kind']==3:need((native['view'],b(native['payload'])) in script_certs[member],'script finalizer saw native notarization')
                    else:need(self.script_entries[entry]==raw,'script reused entry bytes')
                    self.note_send(e,tick);self.metrics['scripted_sends']+=1
                elif tag=='InjectedNativeDelivery':
                    need(self.c['base']['scenario'] in ['wrong-context','native-proof-negatives'] and detail['src_member_id']==2,'declared native negative injection')
                    invalid=self.oracle.ask('certificate',bytes=detail['payload_bytes']);need(not invalid['valid'] and invalid['kind']==3,'wrong-kind proof is actually invalid');positive=[0,*detail['payload_bytes'][1:]];need(self.oracle.ask('certificate',bytes=positive)['valid'],'negative derived from actual valid notarization')
                    if self.c['base']['scenario']=='wrong-context':need(b(detail['cid'])==h('I1b/wrong-context-fixture/v1',b(self.c['cid'])),'wrong-context injection')
                    else:envelope(detail,self.c,self.oracle,allow_invalid=True)
                    injected.append(detail);self.metrics['injections']+=1
                    if self.c['base']['scenario']=='native-proof-negatives':self.note_send(detail,tick)
                elif tag in ['NetworkAdmitted','QueueOverflow']:
                    e=detail['envelope'] if tag=='NetworkAdmitted' else detail;envelope(e,self.c,self.oracle,allow_invalid=e in injected);token=(b(e['message_id']),e['attempt']);need(token in self.sent and token not in consumed and self.sent[token]==(e,tick),'routing only committed unique send attempts');consumed.add(token);route=(e['src_member_id'],e['dst_member_id'],e['channel']);queue=self.routes[route]
                    if tag=='QueueOverflow':need(len(queue)>=64,'drop-newest only at declared queue capacity');self.metrics['queue_overflow']+=1
                    else:
                        need(len(queue)<64,'route queue capacity');due=tick+20000
                        if e['dst_member_id']==1 and e['channel']==4 and self.c['base']['body_hold_until'] is not None:due=max(due,self.c['base']['body_hold_until'])
                        need(detail['due_tick_us']==due,'native link/body hold delay');queue.append((due,e))
                elif tag=='NetworkDelivered':
                    e=detail;route=(e['src_member_id'],e['dst_member_id'],e['channel']);need(self.routes[route],'delivery has admitted route');due,known=self.routes[route].popleft();need(e==known and tick>=due,'FIFO original delivery deadline');native=envelope(e,self.c,self.oracle,allow_invalid=e in injected);dst=e['dst_member_id'];self.metrics['network_deliveries']+=1
                    if dst>=2 and self.c['base']['scripted']:
                        if e['channel']==4 and native['kind'] in ['Push','Reply']:self.script_bodies[dst].add(native['content'])
                        if e['channel']==1 and native['kind']==1 and e['src_member_id']==(native['view']-1)%5:self.script_proposals[dst].add((native['view'],b(native['payload'])))
                        if e['channel']==2 and native['kind']==1:script_certs[dst].add((native['view'],b(native['payload'])))
                elif tag=='ClockResponseLost':need(self.c['base']['scenario']=='lease-loss' and tick>=1050000,'declared clock loss')
                elif tag in ['ClockRequestTransit','TimerSuppressed','ScriptedPeerOffline']:pass
                elif tag=='RunComplete':run_complete=detail
                elif tag in ['InfrastructureFailure']:raise Invalid('matrix run infrastructure failure')
                elif tag=='DuplicateAck':raise Invalid('unexpected duplicate trace shape; explicit control checker required')
                else:raise Invalid('unknown full trace event '+tag)
            except (Invalid,KeyError,IndexError,TypeError,ValueError) as error:
                raise Invalid(f'row {ordinal} {row.get("kind")}: {error}') from error
        need(self.pending_grant is None and registered==[0,1] and run_complete is not None,'completed two-worker run')
        need(self.root_index==[len(r) for r in self.roots],'no unacknowledged root records in completed run');need(self.frames_seen==self.rawframes,'raw protobuf frames equal full logical trace')
        need(consumed==set(self.sent),'every committed attempt routed or explicitly dropped')
        manifest=json.loads((self.path/'manifest.json').read_text());need(manifest['outcome']=='COMPLETED' and manifest['delivered_heights']==self.delivered and run_complete['delivered_heights']==self.delivered,'manifest matches independently delivered prefix')
        diagnostics=json.loads((self.path/'transport-diagnostics.json').read_text());processes=diagnostics['processes'];starts={p['worker']:p['pid'] for p in processes if 'start_wall_elapsed_us' in p};ends={p['worker']:p['exit_code'] for p in processes if 'exit_code' in p};need(set(starts)=={0,1} and len(set(starts.values()))==2 and ends=={0:0,1:0},'two real distinct successful OS processes')
        need(self.start[0]==1000500 and self.start[1]==1020500,'independent staggered calibration/start')
        scenario=self.c['base']['scenario']
        if scenario=='healthy':need(self.delivered==[2,2],'healthy two-height delivery')
        if scenario=='two-real-alone':need(self.delivered==[0,0] and not self.metrics['scripted_sends'] and self.metrics['cache_hits']>0,'two real members cannot form quorum; actual nullify cache retries')
        if scenario=='proof-before-body':need(any(n==1 and t<1250000 for n,t in self.proof_wait) and self.delivered==[1,1],'proof before private body observation')
        if scenario.startswith('close-'):need(self.first_close_work is not None and self.close_ticks.get(self.first_close_work[0])==self.first_close_work[1]+2500 and self.metrics['work_discarded']>0,'close halfway through actual native work')
        if scenario=='lease-loss':
            need(all(self.closed),'lease-loss retires both cids')
            for node in [0,1]:need(self.close_ticks[node]==self.current[node]['lease_end_tick_us'],'lease equality termination')
        if scenario in ['wrong-context','native-proof-negatives']:need(self.metrics['injections']==1 and self.metrics['input_rejections']>0,'actual rejected negative input')
        if scenario=='quota':need(self.metrics['rate_limited']>0,'quota stress hits actual limiter')
        if scenario=='router-overflow':need(self.metrics['queue_overflow']>0 and self.metrics['rate_limited']==0,'unlimited quota isolates queue overflow')
        return {'case':scenario,'seed':self.c['base']['seed'],'rows':len(self.rows),'metrics':dict(self.metrics),'delivered':self.delivered,'source':str(self.path),'logical_trace_sha256':hashlib.sha256(canonical(self.rows)).hexdigest()}

def check(path,oracle,**overrides):return Checker(path,oracle,**overrides).run()
def main():
    parser=argparse.ArgumentParser();parser.add_argument('paths',nargs='+',type=Path);parser.add_argument('--output',type=Path);args=parser.parse_args();oracle=Oracle();out=[]
    try:
        for path in args.paths:
            paths=[path] if (path/'scenario.json').is_file() else sorted(p.parent for p in path.rglob('scenario.json'))
            for case in paths:
                try:result=check(case,oracle);out.append({'status':'PASS',**result});print(json.dumps(out[-1]),flush=True)
                except Exception as e:out.append({'status':'FAIL','source':str(case),'error':str(e)});print(json.dumps(out[-1]),flush=True)
    finally:oracle.close()
    if args.output:args.output.write_text(json.dumps(out,indent=2)+'\n')
    return 1 if any(x['status']=='FAIL' for x in out) else 0
if __name__=='__main__':raise SystemExit(main())

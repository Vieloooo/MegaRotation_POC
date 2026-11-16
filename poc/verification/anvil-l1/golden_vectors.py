#!/usr/bin/env python3
"""Independent ANVIL-L1-ABI-v1 public vectors, generated only from the contract.

Generate reads/create private test-only keys in an explicit external directory;
public golden results contain only addresses, signatures, hashes and bytes.
Check re-derives every byte/digest using the independent codec, never a contract.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
from coincurve import PrivateKey
import independent_codec as c

HERE = Path(__file__).resolve().parent
POLICY = ['uint8','uint8','uint8','uint8','uint64']
CURSOR = ['uint64','uint64','bytes32','uint8','bytes']
BATCH = [CURSOR,CURSOR,'bytes32','bytes','uint64','uint64','bytes','bytes']
POLICY_SIG = '(uint8,uint8,uint8,uint8,uint64)'
CURSOR_SIG = '(uint64,uint64,bytes32,uint8,bytes)'
BATCH_SIG = '('+CURSOR_SIG+','+CURSOR_SIG+',bytes32,bytes,uint64,uint64,bytes,bytes)'
TYPE_STRINGS = {
    'Domain':'EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)',
    'Policy':'Policy(uint8 enabledMask,uint8 order0,uint8 order1,uint8 order2,uint64 l1Floor)',
    'Cursor':'Cursor(uint64 height,uint64 origin,bytes32 framesDigest,uint8 sourceKind,bytes source)',
    'Force':'Force(address caller,bytes32 requestId,uint64 controlSeq,uint64 targetBranch,bytes32 policyHash)',
    'Resume':'Resume(address caller,bytes32 requestId,uint64 controlSeq,bytes32 policyHash,uint64 targetHeight,uint64 origin)',
    'DAReceipt':'DAReceipt(bytes32 commitment)',
    'BatchAttestation':'BatchAttestation(bytes32 prevHash,bytes32 nextHash,bytes32 commitment,bytes32 daCertificateHash,uint64 firstHeight,uint64 lastHeight,bytes32 gposHash)',
}
NAMES = {'authority':'SequencerRotationControlAuthority','inbox':'SequencerRotationBatchInbox'}
FUNCTIONS = {
    'submitForce':'submitForce(bytes32,uint64,uint64,'+POLICY_SIG+',bytes)',
    'submitResume':'submitResume(bytes32,uint64,'+POLICY_SIG+',uint64,uint64,bytes)',
    'submitBatch':'submitBatch(bytes32,'+BATCH_SIG+')',
    'overrideCursor':'overrideCursor('+CURSOR_SIG+')',
    'currentCursor':'currentCursor()',
    'hashPolicy':'hashPolicy('+POLICY_SIG+')',
    'hashCursor':'hashCursor('+CURSOR_SIG+')',
    'forceDigest':'forceDigest(address,bytes32,uint64,uint64,'+POLICY_SIG+')',
    'resumeDigest':'resumeDigest(address,bytes32,uint64,'+POLICY_SIG+',uint64,uint64)',
    'daDigest':'daDigest(bytes32)',
    'batchDigest':'batchDigest('+BATCH_SIG+')',
}
EVENTS = {
    'ForceRecorded':'ForceRecorded(address,bytes32,uint64,uint64,'+POLICY_SIG+',uint64)',
    'ResumeRecorded':'ResumeRecorded(address,bytes32,uint64,'+POLICY_SIG+',uint64,uint64,uint64)',
    'CursorInitialized':'CursorInitialized('+CURSOR_SIG+')',
    'CursorAdvanced':'CursorAdvanced(address,bytes32,'+CURSOR_SIG+','+CURSOR_SIG+',bytes32)',
    'CursorSet':'CursorSet('+CURSOR_SIG+','+CURSOR_SIG+')',
}
ERRORS = ('Unauthorized','InvalidSignature','InvalidPolicy','InvalidCursor','InvalidRange','InvalidSequence','RequestConflict','StalePrev','InvalidOrigin','InvalidTarget','InvalidDA','Overflow')
FIXTURE = {
    'chainId':31337,
    'configRef':'0x'+'11'*32,
    'authority':'0x1000000000000000000000000000000000000001',
    'inbox':'0x2000000000000000000000000000000000000002',
    'caller':'0x3000000000000000000000000000000000000003',
    'policy':[7,0,1,2,0],
    'genesisCursor':[0,0,'0x'+'00'*32,0,'0x'],
    'gpos':'0x01020304',
    'payloadUtf8':'anvil-l1-v1-payload',
    'firstHeight':1,'lastHeight':2,
    'force':{'requestId':c.hx((1).to_bytes(32,'big')),'controlSeq':0,'targetBranch':7},
    'batch':{'requestId':c.hx((2).to_bytes(32,'big'))},
    'resume':{'requestId':c.hx((3).to_bytes(32,'big')),'controlSeq':2,'targetHeight':8,'origin':0},
}


def struct(name, types, values):
    preimage = c.abi_encode(['bytes32',*types],[c.keccak256(TYPE_STRINGS[name].encode()),*values])
    return preimage,c.keccak256(preimage)


def cursor_hash(value):
    return struct('Cursor',['uint64','uint64','bytes32','uint8','bytes32'],[*value[:4],c.keccak256(c.as_bytes(value[4]))])


def domain(f, role):
    addr = f[role]
    return struct('Domain',['bytes32','bytes32','uint256','address','bytes32'],
        [c.keccak256(NAMES[role].encode()),c.keccak256(b'1'),f['chainId'],addr,f['configRef']])


def build(f, signatures):
    p_pre,p_hash = struct('Policy',POLICY,f['policy'])
    next_cursor = [2,0,'0x'+'00'*32,1,f['gpos']]
    resume_cursor = [8,0,'0x'+'00'*32,2,c.hx(c.abi_encode(['uint64'],[2]))]
    prev_pre,prev_hash = cursor_hash(f['genesisCursor'])
    next_pre,next_hash = cursor_hash(next_cursor)
    res_pre,res_hash = cursor_hash(resume_cursor)
    domains = {role:domain(f,role) for role in NAMES}
    commitment = c.keccak256(f['payloadUtf8'].encode())
    da_pre,da_hash = struct('DAReceipt',['bytes32'],[commitment])
    force_values = [f['caller'],f['force']['requestId'],f['force']['controlSeq'],f['force']['targetBranch'],p_hash]
    force_types = ['address','bytes32','uint64','uint64','bytes32']
    force_pre,force_hash = struct('Force',force_types,force_values)
    resume_values = [f['caller'],f['resume']['requestId'],f['resume']['controlSeq'],p_hash,f['resume']['targetHeight'],f['resume']['origin']]
    resume_types = ['address','bytes32','uint64','bytes32','uint64','uint64']
    resume_pre,resume_hash = struct('Resume',resume_types,resume_values)
    da_sig = c.as_bytes(signatures['da'])
    batch_values = [prev_hash,next_hash,commitment,c.keccak256(da_sig),f['firstHeight'],f['lastHeight'],c.keccak256(c.as_bytes(f['gpos']))]
    batch_types = ['bytes32']*4+['uint64','uint64','bytes32']
    batch_pre,batch_hash = struct('BatchAttestation',batch_types,batch_values)
    messages = {
      'force':('Force','authority',force_pre,force_hash,force_types,force_values,'governance'),
      'resume':('Resume','authority',resume_pre,resume_hash,resume_types,resume_values,'governance'),
      'da':('DAReceipt','inbox',da_pre,da_hash,['bytes32'],[commitment],'da'),
      'batch':('BatchAttestation','inbox',batch_pre,batch_hash,batch_types,batch_values,'certifier'),
    }
    signed = {}
    for name, (type_name,role,pre,hashed,types,values,signer) in messages.items():
        digest = c.typed_digest(domains[role][1],hashed)
        signature = c.as_bytes(signatures[name])
        signed[name] = {'type':type_name,'domain':role,'structPreimage':c.hx(pre),'structHash':c.hx(hashed),
            'digestPreimage':c.hx(b'\x19\x01'+domains[role][1]+hashed),'digest':c.hx(digest),
            'signature':c.hx(signature),'signerRole':signer,'recoveredSigner':c.hx(c.recover_address(signature,digest))}
    batch_tuple = [f['genesisCursor'],next_cursor,c.hx(commitment),signatures['da'],1,2,f['gpos'],signatures['batch']]
    calldata_values = {
      'submitForce':(['bytes32','uint64','uint64',POLICY,'bytes'],[f['force']['requestId'],0,7,f['policy'],signatures['force']]),
      'submitResume':(['bytes32','uint64',POLICY,'uint64','uint64','bytes'],[f['resume']['requestId'],2,f['policy'],8,0,signatures['resume']]),
      'submitBatch':(['bytes32',BATCH],[f['batch']['requestId'],batch_tuple]),
      'overrideCursor':([CURSOR],[resume_cursor]),
    }
    result = {
      'profile':'ANVIL-L1-ABI-v1','kind':'independent-public-golden-vector; offline addresses; opaque gpos fixture',
      'fixture':f,'typeStrings':TYPE_STRINGS,'typeHashes':{k:c.hx(c.keccak256(v.encode())) for k,v in TYPE_STRINGS.items()},
      'domains':{r:{'name':NAMES[r],'preimage':c.hx(d[0]),'separator':c.hx(d[1])} for r,d in domains.items()},
      'objects':{'policy':{'preimage':c.hx(p_pre),'hash':c.hx(p_hash)},
        'prev':{'cursor':f['genesisCursor'],'preimage':c.hx(prev_pre),'hash':c.hx(prev_hash)},
        'next':{'cursor':next_cursor,'preimage':c.hx(next_pre),'hash':c.hx(next_hash)},
        'override':{'cursor':resume_cursor,'preimage':c.hx(res_pre),'hash':c.hx(res_hash)},
        'payload':{'bytes':c.hx(f['payloadUtf8'].encode()),'commitment':c.hx(commitment)}},
      'signed':signed,
      'functionSelectors':{k:{'signature':v,'selector':c.hx(c.keccak256(v.encode())[:4])} for k,v in FUNCTIONS.items()},
      'eventTopics':{k:{'signature':v,'topic0':c.hx(c.keccak256(v.encode()))} for k,v in EVENTS.items()},
      'errorSelectors':{k:c.hx(c.keccak256((k+'()').encode())[:4]) for k in ERRORS},
      'calldata':{k:{'arguments':c.hx(c.abi_encode(t,v)),
                    'full':c.hx(c.keccak256(FUNCTIONS[k].encode())[:4]+c.abi_encode(t,v))} for k,(t,v) in calldata_values.items()},
      'signatureMutations':[],
      'malformedSignatures':[],
    }
    # Each signed field, plus domain dimensions and every raw Policy/Cursor component.
    def mutation(base,label,new_hash=None,new_domain=None):
        msg = messages[base]
        d = c.typed_digest(new_domain or domains[msg[1]][1],new_hash or msg[3])
        sig = c.as_bytes(signatures[base])
        recovered = c.recover_address(sig,d)
        assert recovered != c.as_bytes(signed[base]['recoveredSigner']), label
        result['signatureMutations'].append({'base':base,'changed':label,'digest':c.hx(d),'recoveredSigner':c.hx(recovered),'expected':'REJECT_ORIGINAL_SIGNATURE'})
    for base, (tn,role,pre,hs,types,values,_) in messages.items():
        for i,(ty,value) in enumerate(zip(types,values)):
            mutated = list(values)
            if ty.startswith('uint'):
                mutated[i] = value + 1
            else:
                raw=bytearray(c.as_bytes(value)); raw[-1] ^= 1; mutated[i]=bytes(raw)
            mutation(base,'signed_field_'+str(i),struct(tn,types,mutated)[1])
        for field in ('chainId',role,'configRef'):
            altered=dict(f)
            if field=='chainId': altered[field] += 1
            else:
                raw=bytearray(c.as_bytes(altered[field]));raw[-1]^=1;altered[field]=c.hx(raw)
            mutation(base,'domain_'+field,new_domain=domain(altered,role)[1])
    for i in range(5):
        altered=list(f['policy']);altered[i]+=1
        ph=struct('Policy',POLICY,altered)[1]
        fv=list(force_values);fv[4]=ph
        rv=list(resume_values);rv[3]=ph
        mutation('force','policy_field_'+str(i),struct('Force',force_types,fv)[1])
        mutation('resume','policy_field_'+str(i),struct('Resume',resume_types,rv)[1])
    for which, original, index in [('prev',f['genesisCursor'],0),('next',next_cursor,1)]:
        for i in range(5):
            altered=list(original)
            if i in (0,1,3):altered[i]+=1
            else:
                raw=bytearray(c.as_bytes(altered[i]))
                if raw:raw[-1]^=1
                else:raw.append(1)
                altered[i]=c.hx(raw)
            bv=list(batch_values);bv[index]=cursor_hash(altered)[1]
            mutation('batch',which+'_cursor_field_'+str(i),struct('BatchAttestation',batch_types,bv)[1])
    # Explicit cross-kind and personal-sign misuse.
    mutation('force','control_type_resume',resume_hash)
    mutation('da','type_batch_attestation',batch_hash)
    for base in signed:
        item=signed[base];sig=c.as_bytes(item['signature']);digest=c.as_bytes(item['digest'])
        personal=c.keccak256(b'\x19Ethereum Signed Message:\n32'+digest)
        recovered=c.recover_address(sig,personal)
        assert recovered != c.as_bytes(item['recoveredSigner'])
        result['signatureMutations'].append({'base':base,'changed':'personal_sign_prefix','digest':c.hx(personal),'recoveredSigner':c.hx(recovered),'expected':'REJECT_ORIGINAL_SIGNATURE'})
    for base, item in signed.items():
        sig=c.as_bytes(item['signature']); digest=c.as_bytes(item['digest'])
        malformed = {
          'length64':sig[:64], 'length66':sig+b'\x00',
          'v0':sig[:64]+b'\x00', 'v1':sig[:64]+b'\x01', 'v29':sig[:64]+b'\x1d',
          'r0':b'\0'*32+sig[32:], 'rOrder':c.SECP_N.to_bytes(32,'big')+sig[32:],
          's0':sig[:32]+b'\0'*32+sig[64:],
          'sAboveHalf':sig[:32]+(c.SECP_N//2+1).to_bytes(32,'big')+sig[64:],
          'highSAlias':sig[:32]+(c.SECP_N-int.from_bytes(sig[32:64],'big')).to_bytes(32,'big')+bytes([55-sig[64]])}
        for label,bad in malformed.items():
            try:
                c.recover_address(bad,digest)
                raise AssertionError(label+' should reject')
            except ValueError:
                pass
            result['malformedSignatures'].append({'base':base,'changed':label,'signature':c.hx(bad),'expected':'REJECT_NONCANONICAL_SIGNATURE'})
    return result


def private_keys(directory):
    directory.mkdir(parents=True,exist_ok=True,mode=0o700)
    result={}
    for role in ('governance','da','certifier'):
        p=directory/(role+'.test-key')
        if not p.exists():
            fd=os.open(p,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
            with os.fdopen(fd,'wb') as f:f.write(secrets.token_bytes(32))
        raw=p.read_bytes()
        PrivateKey(raw)
        result[role]=raw
    return result


def generate(directory):
    keys=private_keys(directory)
    f=FIXTURE
    ds={r:domain(f,r)[1] for r in NAMES}
    ph=struct('Policy',POLICY,f['policy'])[1]
    commitment=c.keccak256(f['payloadUtf8'].encode())
    da_hash=struct('DAReceipt',['bytes32'],[commitment])[1]
    signatures={'da':c.hx(c.sign_digest(keys['da'],c.typed_digest(ds['inbox'],da_hash)))}
    fh=struct('Force',['address','bytes32','uint64','uint64','bytes32'],[f['caller'],f['force']['requestId'],0,7,ph])[1]
    rh=struct('Resume',['address','bytes32','uint64','bytes32','uint64','uint64'],[f['caller'],f['resume']['requestId'],2,ph,8,0])[1]
    signatures['force']=c.hx(c.sign_digest(keys['governance'],c.typed_digest(ds['authority'],fh)))
    signatures['resume']=c.hx(c.sign_digest(keys['governance'],c.typed_digest(ds['authority'],rh)))
    next_cursor=[2,0,'0x'+'00'*32,1,f['gpos']]
    bh=struct('BatchAttestation',['bytes32']*4+['uint64','uint64','bytes32'],
        [cursor_hash(f['genesisCursor'])[1],cursor_hash(next_cursor)[1],commitment,c.keccak256(c.as_bytes(signatures['da'])),1,2,c.keccak256(c.as_bytes(f['gpos']))])[1]
    signatures['batch']=c.hx(c.sign_digest(keys['certifier'],c.typed_digest(ds['inbox'],bh)))
    result=build(f,signatures)
    result['publicSigners']={r:c.hx(c.public_address(PrivateKey(k).public_key)) for r,k in keys.items()}
    for m in result['signed'].values():assert m['recoveredSigner']==result['publicSigners'][m['signerRole']]
    return result


def check(path,cast=None):
    saved=json.loads(path.read_text())
    assert saved['fixture']==FIXTURE
    signatures={k:v['signature'] for k,v in saved['signed'].items()}
    rebuilt=build(saved['fixture'],signatures)
    public=saved.pop('publicSigners')
    assert saved==rebuilt, 'golden artifact differs from independent recomputation'
    assert len(set(public.values()))==3
    for name,m in rebuilt['signed'].items():
        assert m['recoveredSigner']==public[m['signerRole']],name
    cast_checks=0
    if cast:
        data=[o['preimage'] for o in rebuilt['objects'].values() if 'preimage' in o]
        data += [d['preimage'] for d in rebuilt['domains'].values()]
        data += [m[k] for m in rebuilt['signed'].values() for k in ('structPreimage','digestPreimage')]
        for value in data:
            expected=c.hx(c.keccak256(c.as_bytes(value)))
            assert subprocess.check_output([cast,'keccak',value],text=True).strip()==expected
            cast_checks+=1
        for fn,info in rebuilt['functionSelectors'].items():
            assert subprocess.check_output([cast,'sig',info['signature']],text=True).strip()==info['selector']
            cast_checks+=1
        for info in rebuilt['eventTopics'].values():
            assert subprocess.check_output([cast,'keccak',info['signature']],text=True).strip()==info['topic0']
            cast_checks+=1
        def arg(value):
            return '('+','.join(arg(v) for v in value)+')' if isinstance(value,list) else str(value)
        f=rebuilt['fixture']
        vals={
          'submitForce':[f['force']['requestId'],0,7,f['policy'],signatures['force']],
          'submitResume':[f['resume']['requestId'],2,f['policy'],8,0,signatures['resume']],
          'submitBatch':[f['batch']['requestId'],[f['genesisCursor'],rebuilt['objects']['next']['cursor'],rebuilt['objects']['payload']['commitment'],signatures['da'],1,2,f['gpos'],signatures['batch']]],
          'overrideCursor':[rebuilt['objects']['override']['cursor']]}
        for name,values in vals.items():
            actual=subprocess.check_output([cast,'abi-encode',FUNCTIONS[name],*[arg(x) for x in values]],text=True).strip()
            assert actual==rebuilt['calldata'][name]['arguments'],name
            cast_checks+=1
    return {'status':'PASS','profile':rebuilt['profile'],'signedMessages':len(signatures),
        'fieldAndDomainMutations':len(rebuilt['signatureMutations']),
        'malformedSignatureCases':len(rebuilt['malformedSignatures']),'castCrossChecks':cast_checks,
        'goldenSha256':hashlib.sha256(path.read_bytes()).hexdigest(),
        'privateKeysRequiredForCheck':False,'candidateCodeImported':False}


if __name__=='__main__':
    p=argparse.ArgumentParser();s=p.add_subparsers(dest='command',required=True)
    g=s.add_parser('generate');g.add_argument('--private-dir',type=Path,required=True);g.add_argument('--output',type=Path,required=True)
    k=s.add_parser('check');k.add_argument('--golden',type=Path,default=HERE/'golden-vectors.json');k.add_argument('--cast')
    a=p.parse_args()
    if a.command=='generate':
        if a.private_dir.resolve().is_relative_to(HERE.parents[2]):p.error('private keys must be outside the repository')
        a.output.write_text(json.dumps(generate(a.private_dir),indent=2)+'\n')
        print(json.dumps({'status':'GENERATED','output':str(a.output),'privateKeysPrinted':False}))
    else:print(json.dumps(check(a.golden,a.cast),indent=2))

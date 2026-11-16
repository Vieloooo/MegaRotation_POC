#!/usr/bin/env python3
"""Verifier-only secp256k1 vectors for the frozen 09 ABI with BS-FRAME-v1.

Private test keys are created only in a private temporary directory. Committed
output contains public addresses/signatures only. No feature-11 code is used.
"""
from __future__ import annotations
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import secrets
import sys
import tempfile

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/'anvil-l1'))
import independent_codec as c

CURSOR='Cursor(uint64 height,uint64 origin,bytes32 framesDigest,uint8 sourceKind,bytes source)'
DA='DAReceipt(bytes32 commitment)'
BATCH='BatchAttestation(bytes32 prevHash,bytes32 nextHash,bytes32 commitment,bytes32 daCertificateHash,uint64 firstHeight,uint64 lastHeight,bytes32 gposHash)'


def cursor_hash(value):
    return c.keccak256(c.abi_encode(['bytes32','uint64','uint64','bytes32','uint8','bytes32'],[c.keccak256(CURSOR.encode()),*value[:4],c.keccak256(c.as_bytes(value[4]))]))


def domain(f):
    return c.domain_separator('SequencerRotationBatchInbox','1',f['chain_id'],f['inbox'],f['config_ref'])


def da_digest(f):
    body=c.keccak256(c.abi_encode(['bytes32','bytes32'],[c.keccak256(DA.encode()),f['commitment']]))
    return c.typed_digest(domain(f),body)


def batch_digest(f):
    body=c.keccak256(c.abi_encode(['bytes32']*5+['uint64','uint64','bytes32'],[c.keccak256(BATCH.encode()),cursor_hash(f['prev']),cursor_hash(f['next']),f['commitment'],c.keccak256(c.as_bytes(f['da_certificate'])),f['first_height'],f['last_height'],c.keccak256(c.as_bytes(f['gpos']))]))
    return c.typed_digest(domain(f),body)


def derive(public):
    f=public['fixture']
    assert c.hx(c.keccak256(c.as_bytes(f['payload'])))==f['commitment']
    d=da_digest(f)
    b=batch_digest(f)
    assert c.hx(c.recover_address(c.as_bytes(f['da_certificate']),d))==public['da_signer']
    assert c.hx(c.recover_address(c.as_bytes(public['certifier_signature']),b))==public['certifier_signer']
    mutations=[]
    def mutation(role, name, changed):
        digest=da_digest(changed) if role=='da' else batch_digest(changed)
        signature=c.as_bytes(f['da_certificate'] if role=='da' else public['certifier_signature'])
        signer=public['da_signer'] if role=='da' else public['certifier_signer']
        recovered=c.hx(c.recover_address(signature,digest))
        assert recovered!=signer
        mutations.append({'role':role,'changed':name,'digest':c.hx(digest),'recovered':recovered,'expected':'REJECT_ORIGINAL_SIGNATURE'})
    for role in ('da','certifier'):
        for key in ('chain_id','inbox','config_ref','commitment'):
            changed=copy.deepcopy(f)
            changed[key]=f[key]+1 if isinstance(f[key],int) else c.hx(bytes([c.as_bytes(f[key])[0]^1])+c.as_bytes(f[key])[1:])
            mutation(role,key,changed)
    for key in ('prev','next'):
        for i,label in enumerate(('height','origin','frames_digest','source_kind','source')):
            changed=copy.deepcopy(f)
            old=changed[key][i]
            changed[key][i]=old+1 if isinstance(old,int) else old+'01' if label=='source' else c.hx(bytes([c.as_bytes(old)[0]^1])+c.as_bytes(old)[1:])
            mutation('certifier',key+'.'+label,changed)
    for key in ('first_height','last_height','gpos','da_certificate'):
        changed=copy.deepcopy(f)
        changed[key]=f[key]+1 if isinstance(f[key],int) else f[key]+'01'
        mutation('certifier',key,changed)
    malformed=[]
    signature=c.as_bytes(public['certifier_signature'])
    variants={
      'short':signature[:-1],
      'trailing':signature+b'\x00',
      'v0':signature[:-1]+b'\x00',
      'r0':bytes(32)+signature[32:],
      's0':signature[:32]+bytes(32)+signature[64:],
      'high_s':signature[:32]+(c.SECP_N-int.from_bytes(signature[32:64],'big')).to_bytes(32,'big')+bytes([27 if signature[64]==28 else 28]),
    }
    for name,value in variants.items():
        try:
            c.recover_address(value,b)
        except (ValueError,AssertionError):
            malformed.append({'name':name,'signature':c.hx(value),'expected':'REJECT'})
        else:
            raise AssertionError('malformed signature accepted: '+name)
    return {**public,'da_digest':c.hx(d),'batch_digest':c.hx(b),'mutations':mutations,'malformed_signatures':malformed}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('command',choices=['generate','check'])
    parser.add_argument('--vectors',type=Path,default=HERE/'signature-golden-vectors.json')
    args=parser.parse_args()
    if args.command=='generate':
        frame=json.loads((HERE/'frame-golden-vectors.json').read_text())['valid'][1]
        gpos=c.hx(b'fixture:BS-FRAME-v1:gpos:1')
        f={'chain_id':31337,'inbox':'0x2000000000000000000000000000000000000002','config_ref':'0x'+'11'*32,'prev':[0,0,'0x'+'00'*32,0,'0x'],'next':[2,0,'0x'+'00'*32,1,gpos],'first_height':1,'last_height':2,'gpos':gpos,'payload':frame['payload'],'commitment':frame['commitment']}
        keydir=Path(tempfile.mkdtemp(prefix='batch-services-verifier-keys-'))
        keys={role:secrets.token_bytes(32) for role in ('da','certifier')}
        for role,key in keys.items():
            fd=os.open(keydir/(role+'.key'),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
            with os.fdopen(fd,'wb') as file:
                file.write(key)
        f['da_certificate']=c.hx(c.sign_digest(keys['da'],da_digest(f)))
        signature=c.sign_digest(keys['certifier'],batch_digest(f))
        public={'profile':'BS-FRAME-v1/ANVIL-L1-ABI-v1','kind':'offline independent real secp256k1 public vectors; fixture addresses/gpos; NOT Anvil acceptance','fixture':f,'da_signer':c.hx(c.recover_address(c.as_bytes(f['da_certificate']),da_digest(f))),'certifier_signer':c.hx(c.recover_address(signature,batch_digest(f))),'certifier_signature':c.hx(signature)}
        result=derive(public)
        args.vectors.write_text(json.dumps(result,indent=2)+'\n')
    else:
        stored=json.loads(args.vectors.read_text())
        public={k:v for k,v in stored.items() if k not in ('da_digest','batch_digest','mutations','malformed_signatures')}
        result=derive(public)
        assert stored==result,'signature vector drift'
    print(json.dumps({'status':'PASS','valid_signatures':2,'mutations':len(result['mutations']),'malformed_signatures':len(result['malformed_signatures']),'sha256':hashlib.sha256(args.vectors.read_bytes()).hexdigest()}))


if __name__=='__main__':
    main()

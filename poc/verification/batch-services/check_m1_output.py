#!/usr/bin/env python3
"""Check production consumer output with independent ABI and crypto references."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import signature_vectors as s
sys.path.insert(0,str(HERE.parent/'anvil-l1'))
import independent_codec as c
import golden_vectors as g


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    value=json.loads(args.output.read_text())
    golden=json.loads((HERE/'signature-golden-vectors.json').read_text())
    fixture=golden['fixture']
    produced=value['production_signing']
    da=s.da_digest(fixture)
    batch=s.batch_digest(fixture)
    assert produced['da_digest']==c.hx(da)
    assert produced['batch_digest']==c.hx(batch)
    assert c.hx(c.recover_address(c.as_bytes(produced['da_signature']),da))==produced['address']
    assert c.hx(c.recover_address(c.as_bytes(produced['batch_signature']),batch))==produced['address']
    fields=[fixture['prev'],fixture['next'],fixture['commitment'],fixture['da_certificate'],fixture['first_height'],fixture['last_height'],fixture['gpos'],golden['certifier_signature']]
    expected=c.keccak256(g.FUNCTIONS['submitBatch'].encode())[:4]+c.abi_encode(['bytes32',g.BATCH],[(1).to_bytes(32,'big'),fields])
    assert c.as_bytes(value['calldata'])==expected
    print(json.dumps({'status':'PASS','source_revision':value['source_revision'],'random_seed':value['random_seed'],'positive_frames':len(value['positive_frames']),'negative_frames':len(value['negative_frames']),'all_truncated_prefixes_rejected':sum(x['all_truncations_rejected'] for x in value['positive_frames']),'max_payload':value['maximum_payload_length'],'signature_mutations':len(value['signature_mutations']),'malformed_signature_checks':value['malformed_signatures'],'production_signatures_independently_recovered':2,'calldata_bytes_independently_matched':len(expected),'consumer_output_sha256':hashlib.sha256(args.output.read_bytes()).hexdigest(),'signer':produced['address']},indent=2))


if __name__=='__main__':
    main()

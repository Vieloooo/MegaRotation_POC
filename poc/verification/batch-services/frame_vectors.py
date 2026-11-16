#!/usr/bin/env python3
"""Independent BS-FRAME-v1 golden vectors, derived before the implementation.

This is a verifier-only reference, never a production encoding/signing path.
The imported 09 helper is its pre-existing independent Keccak/ABI reference.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'anvil-l1'))
import independent_codec as independent

MAX_DATA = 47146
MAX_PAYLOAD = 47175
U64 = 2**64 - 1


def tx(sender, recipient, nonce, amount):
    return struct.pack('>BBBQQ', 1, sender, recipient, nonce, amount)


def encode(records):
    assert 1 <= len(records) <= 2
    data = struct.pack('>H', len(records))
    for i, r in enumerate(records):
        assert 0 <= r['height'] <= U64 and 0 <= r['origin'] <= U64
        assert i == 0 or r['height'] == records[i - 1]['height'] + 1
        assert len(r['transactions']) <= 1024
        data += struct.pack('>QQI', r['height'], r['origin'], len(r['transactions']))
        for raw in r['transactions']:
            assert len(raw) == 19 and raw[0] == 1
            data += struct.pack('>I', len(raw)) + raw
    assert len(data) <= MAX_DATA
    return wrap(data)


def wrap(data):
    return b'P0B1' + struct.pack('>H', 1) + independent.keccak256(data)[:16] + struct.pack('>HI', 0, len(data)) + data + b'\x01'


def decode(payload):
    def require(ok, message):
        if not ok:
            raise ValueError(message)
    require(29 <= len(payload) <= MAX_PAYLOAD, 'payload bound')
    require(payload[:4] == b'P0B1', 'magic')
    require(payload[4:6] == b'\x00\x01', 'version')
    require(payload[22:24] == b'\x00\x00', 'frame number')
    size = int.from_bytes(payload[24:28], 'big')
    require(size <= MAX_DATA and len(payload) == 29 + size, 'data length')
    require(payload[-1] == 1, 'complete frame')
    data = payload[28:-1]
    require(payload[6:22] == independent.keccak256(data)[:16], 'channel id')
    offset = 0
    def take(n):
        nonlocal offset
        require(offset + n <= len(data), 'truncated record')
        raw = data[offset:offset+n]
        offset += n
        return raw
    def num(n):
        return int.from_bytes(take(n), 'big')
    count = num(2)
    require(count in (1, 2), 'record count')
    records = []
    for _ in range(count):
        height, origin, tx_count = num(8), num(8), num(4)
        require(tx_count <= 1024, 'transaction count')
        if records:
            require(records[-1]['height'] < U64 and height == records[-1]['height'] + 1, 'continuity')
        transactions = []
        for _ in range(tx_count):
            length = num(4)
            require(length == 19, 'transaction length')
            raw = take(length)
            require(raw[0] == 1, 'ordinary transfer opcode')
            transactions.append(raw)
        records.append({'height': height, 'origin': origin, 'transactions': transactions})
    require(offset == len(data), 'trailing data')
    return records


def json_records(records):
    return [{**r, 'transactions': ['0x' + x.hex() for x in r['transactions']]} for r in records]


def derive():
    inputs = {
        'empty-height-1': [{'height':1,'origin':0,'transactions':[]}],
        'kept-and-padding': [{'height':1,'origin':0,'transactions':[tx(0,1,0,7),tx(0,1,1,8)]},{'height':2,'origin':0,'transactions':[]}],
        'max-height-origin': [{'height':U64,'origin':U64,'transactions':[]}],
        'contract-single-h1': [{'height':1,'origin':0,'transactions':[tx(1,2,0,3)]}],
        'contract-h1-h2': [{'height':1,'origin':0,'transactions':[tx(1,2,0,3)]},{'height':2,'origin':0,'transactions':[tx(1,2,1,2)]}],
        'contract-h1-padding-h2': [{'height':1,'origin':0,'transactions':[tx(1,2,0,3)]},{'height':2,'origin':0,'transactions':[]}],
    }
    good = []
    for name, records in inputs.items():
        payload = encode(records)
        assert decode(payload) == records
        good.append({'name':name,'records':json_records(records),'payload':'0x'+payload.hex(),'payload_length':len(payload),'frame_data':'0x'+payload[28:-1].hex(),'frame_data_length':len(payload)-29,'channel_id':'0x'+payload[6:22].hex(),'commitment':'0x'+independent.keccak256(payload).hex(),'expected':'DECODE'})
    base = encode(inputs['kept-and-padding'])
    bad = []
    def negative(name, payload):
        try:
            decode(payload)
        except ValueError as error:
            bad.append({'name':name,'payload':'0x'+payload.hex(),'expected':'REJECT','reference_reason':str(error)})
        else:
            raise AssertionError('negative accepted: '+name)
    def mutate(offset, replacement):
        return base[:offset]+replacement+base[offset+len(replacement):]
    negative('bad-magic', mutate(0,b'Q'))
    negative('version-2', mutate(4,b'\x00\x02'))
    negative('channel-mismatch', mutate(6,bytes([base[6]^1])))
    negative('frame-number-1', mutate(22,b'\x00\x01'))
    negative('wrong-data-length', mutate(24,(len(base)-28).to_bytes(4,'big')))
    negative('length-u32-max', mutate(24,b'\xff'*4))
    negative('not-last', base[:-1]+b'\x00')
    negative('last-marker-2', base[:-1]+b'\x02')
    negative('trailing-payload', base+b'\x00')
    negative('truncated-payload', base[:-1])
    data = base[28:-1]
    negative('record-count-0', wrap(b'\x00\x00'+data[2:]))
    negative('record-count-3', wrap(b'\x00\x03'+data[2:]))
    negative('transaction-count-1025', wrap(data[:18]+(1025).to_bytes(4,'big')+data[22:]))
    negative('transaction-length-18', wrap(data[:22]+(18).to_bytes(4,'big')+data[26:]))
    negative('deposit-opcode', wrap(data[:26]+b'\x02'+data[27:]))
    negative('record-height-gap', wrap(data[:68]+(3).to_bytes(8,'big')+data[76:]))
    negative('record-height-duplicate', wrap(data[:68]+(1).to_bytes(8,'big')+data[76:]))
    negative('trailing-frame-data', wrap(data+b'\x00'))
    overflow_data=struct.pack('>HQQIQQI',2,U64,0,0,0,0,0)
    negative('height-wraparound',wrap(overflow_data))
    max_records=[{'height':h,'origin':0,'transactions':[tx(0,1,n,0) for n in range(1024)]} for h in (1,2)]
    maximum=encode(max_records)
    assert len(maximum)==MAX_PAYLOAD and decode(maximum)==max_records
    boundary={'name':'maximum-two-records','recipe':'heights=1,2; origin=0; each transactions Transfer(0,1,nonce=0..1023,amount=0)','payload_length':len(maximum),'data_length':len(maximum)-29,'payload_sha256':hashlib.sha256(maximum).hexdigest(),'commitment':'0x'+independent.keccak256(maximum).hex(),'expected':'DECODE'}
    return {'profile':'BS-FRAME-v1','kind':'independent P0 byte vectors; not native MegaETH encoding; not real G or L1 acceptance','seed':'none (literal inputs)','valid':good,'invalid':bad,'boundary':boundary}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('command',choices=['generate','check','inspect'])
    parser.add_argument('--vectors',type=Path,default=HERE/'frame-golden-vectors.json')
    parser.add_argument('--payload',type=Path)
    args=parser.parse_args()
    if args.command=='inspect':
        raw=args.payload.read_bytes()
        print(json.dumps({'records':json_records(decode(raw)),'commitment':'0x'+independent.keccak256(raw).hex(),'length':len(raw)},indent=2))
        return
    result=derive()
    if args.command=='generate':
        args.vectors.write_text(json.dumps(result,indent=2)+'\n')
    else:
        assert json.loads(args.vectors.read_text())==result,'golden vectors changed'
    print(json.dumps({'status':'PASS','valid':len(result['valid']),'invalid':len(result['invalid']),'boundary':1,'golden_sha256':hashlib.sha256(args.vectors.read_bytes()).hexdigest()}))


if __name__=='__main__':
    main()

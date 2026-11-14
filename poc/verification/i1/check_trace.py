#!/usr/bin/env python3
"""Independent I1a oracle: Python stdlib, no production transition/hash calls.

Checks frozen protobuf/hash framing, on-disk commitment, scheduler ownership,
and source-derived clock/lifecycle/timer/work/receipt invariants. It does not
establish physical power-loss durability or I1b/Commonware conformance.
"""
import argparse
import collections
import copy
import hashlib
import json
from pathlib import Path
import struct
import sys


class Violation(Exception):
    pass


def need(ok, rule):
    if not ok:
        raise Violation(rule)


def compact(obj):
    return json.dumps(obj, separators=(",", ":"), ensure_ascii=False).encode()


def hp(domain, *parts):
    h = hashlib.sha256()
    for p in (domain.encode(), *parts):
        h.update(len(p).to_bytes(8, "big"))
        h.update(p)
    return h.digest()


def b(x):
    return bytes(x)


def be(x, size=8):
    return x.to_bytes(size, "big")


def varint(x):
    need(0 <= x < 1 << 64, "protobuf integer range")
    out = bytearray()
    while x > 127:
        out.append((x & 127) | 128)
        x >>= 7
    out.append(x)
    return bytes(out)


# Field definitions copied from the frozen wire contract, not generated Rust.
SCHEMA = {
    "Empty": [],
    "Cid": [(1, "cid", "b")],
    "Fixture": [(1, "cid", "b"), (2, "stop_local_us", "s")],
    "Work": [(1, "cid", "b"), (2, "slot", "u"), (3, "subject", "b"), (4, "duration_us", "u")],
    "Timer": [(1, "timer_id", "u"), (2, "generation", "u"), (3, "cid", "b"), (4, "local_deadline_us", "s")],
    "ClockRequest": [(1, "request_id", "u")],
    "ClockSample": [(1,"run_id","b"),(2,"node_id","u"),(3,"incarnation","u"),(4,"request_id","u"),
                    (5,"clock_domain","b"),(6,"sequence","u"),(7,"as_of_tick_us","u"),(8,"node_time_at_as_of_us","s"),
                    (9,"uncertainty_us","u"),(10,"lease_end_tick_us","u"),(11,"rate_profile_ref","b")],
    "Envelope": [(1,"run_id","b"),(2,"src","u"),(3,"dst","u"),(4,"channel","u"),(5,"message_id","b"),
                 (6,"cid","b"),(7,"sent_at_tick_us","u"),(8,"payload_hash","b"),(9,"payload_bytes","b"),(10,"attempt","u")],
    "ClockRead": [(1,"cid","b"),(2,"caller","t"),(3,"tick_us","u"),(4,"node_time_us","s"),(5,"lease_end_tick_us","u")],
    "Notice": [(1,"cid","b"),(2,"reason","t"),(3,"request_id","u")],
    "SignedIntent": [(1,"cid","b"),(2,"slot","u"),(3,"subject","b"),(4,"intent_hash","b")],
    "Grant": [(1,"schema_version","u"),(2,"run_id","b"),(3,"node_id","u"),(4,"incarnation","u"),
              (5,"logical_step_id","u"),(6,"event_id","u"),(7,"tick_us","u"),(8,"causal_wave","u"),
              (9,"tie_rank","b"),(10,"input","Input"),(11,"grant_hash","b")],
    "Ack": [(1,"schema_version","u"),(2,"run_id","b"),(3,"node_id","u"),(4,"incarnation","u"),
            (5,"logical_step_id","u"),(6,"event_id","u"),(7,"grant_hash","b"),(8,"status","u"),
            (9,"effects","*Effect"),(10,"durable_journal_seq","u"),(11,"state_hash","b"),(12,"logical_ack_hash","b")],
    "RegisterRequest": [(1,"schema_version","u"),(2,"run_id","b"),(3,"node_id","u"),(4,"incarnation","u"),
                        (5,"config_hash","b"),(6,"recovered_journal_seq","u"),(7,"recovered_state_hash","b")],
}
ONEOF = {
    "Input": dict(zip(
        ["RequestSample","Sample","Create","Enter","BeginWork","WorkCompleted","ArmTimer","CancelTimer","TimerFired","Send","Deliver","Close","Pause","Resume","ReadClock","Recover","AnchorReady","CheckClock"],
        enumerate(["Empty","ClockSample","Fixture","Cid","Work","Work","Timer","Timer","Timer","Envelope","Envelope","Cid","Cid","Cid","Cid","Empty","Cid","Empty"],1))),
    "Effect": dict(zip(
        ["RequestSample","SampleAccepted","SampleRejected","ArmTimer","CancelTimer","TimerObserved","WorkStarted","Send","ClockRead","Entered","SignedIntent","Terminated","MessageReceived","Rejected","Created","AnchorReady","Paused","Resumed"],
        enumerate(["ClockRequest","ClockSample","Notice","Timer","Timer","Timer","Work","Envelope","ClockRead","Cid","SignedIntent","Notice","Envelope","Notice","Cid","Cid","Cid","Cid"],1))),
    "WorkerFrame": {"Hello": (1,"RegisterRequest"), "Ack": (2,"Ack")},
    "HubFrame": {"Grant": (1,"Grant")},
}


def tagged(n, data):
    return varint(n * 8 + 2) + varint(len(data)) + data


def proto(name, obj):
    if name in ONEOF:
        value = obj.get("kind" if name in ("Input","Effect") else "frame")
        need(isinstance(value, dict) and len(value) == 1, "oneof presence")
        variant, inner = next(iter(value.items()))
        need(variant in ONEOF[name], "unknown oneof")
        n, typ = ONEOF[name][variant]
        return tagged(n, proto(typ, inner))
    out = bytearray()
    for n, field, typ in SCHEMA[name]:
        value = obj.get(field)
        if typ.startswith("*"):
            for v in value or []:
                out.extend(tagged(n, proto(typ[1:], v)))
        elif typ in ("u", "s"):
            if value:
                encoded = value if typ == "u" else ((value << 1) ^ (value >> 63))
                out.extend(varint(n * 8) + varint(encoded))
        elif typ in ("b", "t"):
            if value:
                out.extend(tagged(n, b(value) if typ == "b" else value.encode()))
        elif value is not None:
            out.extend(tagged(n, proto(typ, value)))
    return bytes(out)


def logical_hash(name, obj):
    obj = copy.deepcopy(obj)
    obj["incarnation"] = 0
    obj["grant_hash" if name == "Grant" else "logical_ack_hash"] = []
    return hp("I1/grant/v1" if name == "Grant" else "I1/ack/v1", proto(name, obj))


def variant(x):
    need(x and x.get("kind") and len(x["kind"]) == 1, "operation presence")
    return next(iter(x["kind"].items()))


def canonical_ack(a):
    a = copy.deepcopy(a)
    a["incarnation"] = 0
    return a


def payload_digest(e):
    return hp("I1/payload/v1", b(e["payload_bytes"]))


def message_digest(e):
    return hp("I1/message/v1", b(e["run_id"]), be(e["src"],4), be(e["dst"],4), be(e["channel"],4),
              b(e["cid"]), be(e["sent_at_tick_us"]), b(e["payload_hash"]))


def intent_digest(run, config, node, intent):
    return hp("I1/intent/v1", run, config, b(intent["cid"]), be(node,4), be(intent["slot"]), b(intent["subject"]))


class Evidence:
    def __init__(self, path):
        self.path = Path(path)
        self.manifest = json.loads((self.path / "manifest.json").read_text())
        self.scenario = json.loads((self.path / "scenario.json").read_text())
        self.rows = [json.loads(s) for s in (self.path / "virtual-trace.jsonl").read_text().splitlines()]
        self.frames = (self.path / "frames.bin").read_bytes()
        self.journals = []
        self.heads = []
        for node in range(2):
            self.journals.append((self.path / f"worker-{node}/journal").read_bytes())
            self.heads.append((self.path / f"worker-{node}/journal.head").read_bytes())


class Model:
    def __init__(self, run, config, node, epsilon):
        self.run, self.config, self.node, self.epsilon = run, config, node, epsilon
        self.inc = 1
        self.sample = None
        self.sequence = 0
        self.counter = 0
        self.requests = set()
        self.prior_end = None
        self.fixtures, self.timers, self.received = {}, {}, {}
        self.views = {}
        self.signed_count = 0

    def local(self, t):
        return t + self.epsilon

    def safe(self, t):
        return self.sample is not None and t < self.sample["lease_end_tick_us"] and self.sample["incarnation"] == self.inc

    def terminate(self, key, tick, reason, expected):
        f = self.fixtures[key]
        if f["terminal"] is None:
            f["terminal"] = {"reason":reason,"tick_us":tick,"lease_end_tick_us":self.prior_end}
            for timer in self.timers.values():
                if timer["active"] and b(timer["timer"]["cid"]).hex() == key:
                    timer["active"] = False
                    expected.append(("CancelTimer",timer["timer"]))
        expected.append(("Terminated", {"cid":f["descriptor"]["cid"],"reason":f["terminal"]["reason"],"request_id":0}))

    def expire(self, tick, expected):
        if self.prior_end is not None and tick >= self.prior_end:
            for key in sorted(self.fixtures):
                f = self.fixtures[key]
                if f["entered_at_tick_us"] is not None and f["terminal"] is None:
                    self.terminate(key,tick,"ClockUnsafe",expected)

    def guard(self, cid, t, expected, callers=(), active=False):
        self.expire(t,expected)
        key = b(cid).hex()
        if not self.safe(t) or key not in self.fixtures or self.fixtures[key]["terminal"] is not None:
            return False
        f = self.fixtures[key]
        if self.local(t) >= f["descriptor"]["stop_local_us"]:
            self.terminate(key,t,"Stop",expected)
            return False
        for caller in callers:
            expected.append(("ClockRead", {"cid":cid,"caller":caller,"tick_us":t,"node_time_us":self.local(t),"lease_end_tick_us":self.prior_end}))
        return not active or (f["entered_at_tick_us"] is not None and not f["paused"])

    def envelope_ok(self,e):
        return (b(e["run_id"]) == self.run and e["src"] in (0,1) and e["dst"] in (0,1) and e["src"] != e["dst"]
                and len(e["cid"]) in (0,32) and len(e["payload_bytes"]) <= 65536
                and b(e["payload_hash"]) == payload_digest(e) and b(e["message_id"]) == message_digest(e)
                and (not e["cid"] or b(e["cid"]).hex() in self.fixtures))

    def apply(self,g,a,stored):
        op,x = variant(g["input"])
        t = g["tick_us"]
        expected=[]
        reject=False
        key=b(x.get("cid",[])).hex()
        previously_entered=self.fixtures.get(key,{}).get("entered_at_tick_us")
        if op == "Recover":
            need(g["incarnation"] > self.inc,"new incarnation requires explicit recovery")
            self.inc=g["incarnation"]
            self.expire(t,expected)
            self.sample=None
            self.requests.clear()
        else:
            need(g["incarnation"] == self.inc,"grant before recovery")
        if op in ("Recover","RequestSample"):
            self.counter+=1;self.requests.add(self.counter)
            expected.append(("RequestSample",{"request_id":self.counter}))
        elif op == "Sample":
            valid=(b(x["run_id"]) == self.run and x["node_id"] == self.node and x["incarnation"] == self.inc
                   and b(x["clock_domain"]) == hp("I1/clock-domain/v1",self.run,self.config,be(self.node,4))
                   and b(x["rate_profile_ref"]) == hp("I1/rate/v1",b"fixed-rate-one-v1")
                   and x["request_id"] in self.requests and x["sequence"] > self.sequence
                   and x["as_of_tick_us"] <= t < x["lease_end_tick_us"]
                   and x["node_time_at_as_of_us"] == self.local(x["as_of_tick_us"])
                   and abs(self.epsilon) <= x["uncertainty_us"] <= 1000)
            if valid:
                self.expire(t,expected);self.requests.remove(x["request_id"]);self.sequence=x["sequence"]
                self.sample=copy.deepcopy(x);self.prior_end=x["lease_end_tick_us"]
                expected.append(("SampleAccepted",x))
            else:
                reject=True
        elif op == "Create":
            reject=len(x["cid"])!=32 or (key in self.fixtures and self.fixtures[key]["descriptor"]!=x)
            if not reject and key not in self.fixtures:
                self.fixtures[key]={"descriptor":copy.deepcopy(x),"created_at_tick_us":t,"anchor_ready_tick_us":None,
                                    "entered_at_tick_us":None,"terminal":None,"paused":False,"signed_intents":{},"pending_work":{}}
                expected.append(("Created",{"cid":x["cid"]}))
        elif op == "AnchorReady":
            reject=key not in self.fixtures or self.fixtures[key]["terminal"] is not None
            if not reject and self.fixtures[key]["anchor_ready_tick_us"] is None:
                self.fixtures[key]["anchor_ready_tick_us"]=t;expected.append(("AnchorReady",x))
        elif op in ("Enter","ReadClock","Pause","Resume"):
            callers=("Manager","BFT") if op=="ReadClock" else ("Manager",)
            reject=not self.guard(x["cid"],t,expected,callers)
            if not reject:
                f=self.fixtures[key]
                if op=="Enter":
                    reject=f["anchor_ready_tick_us"] is None
                    if not reject and f["entered_at_tick_us"] is None:
                        f["entered_at_tick_us"]=t;expected.append(("Entered",x))
                elif op in ("Pause","Resume"):
                    reject=f["entered_at_tick_us"] is None
                    if not reject and f["paused"]!=(op=="Pause"):
                        f["paused"]=op=="Pause";expected.append(("Paused" if f["paused"] else "Resumed",x))
        elif op == "CheckClock":
            self.expire(t,expected)
        elif op == "Close":
            reject=key not in self.fixtures
            if not reject:self.terminate(key,t,"Close",expected)
        elif op == "BeginWork":
            reject=not self.guard(x["cid"],t,expected,("BFT",),True)
            if not reject:
                f=self.fixtures[key];slot=str(x["slot"])
                reject=len(x["subject"])!=32 or x["duration_us"]==0 or slot in f["signed_intents"] or slot in f["pending_work"]
                if not reject:
                    f["pending_work"][slot]={"work":copy.deepcopy(x),"due_tick_us":t+x["duration_us"]}
                    expected.append(("WorkStarted",x))
        elif op == "WorkCompleted":
            f=self.fixtures.get(key,{});slot=str(x["slot"]);pending=f.get("pending_work",{}).get(slot)
            reject=pending is None or pending["work"]!=x or pending["due_tick_us"]>t
            if not reject:
                del f["pending_work"][slot]
                reject=not self.guard(x["cid"],t,expected,("BFT",),True)
                if not reject:
                    need(slot not in f["signed_intents"],"conflicting or repeated signing coordinate")
                    intent={"cid":x["cid"],"slot":x["slot"],"subject":x["subject"]}
                    intent["intent_hash"]=list(intent_digest(self.run,self.config,self.node,intent))
                    f["signed_intents"][slot]=intent;expected.append(("SignedIntent",intent));self.signed_count+=1
        elif op in ("ArmTimer","CancelTimer","TimerFired"):
            tid=str(x["timer_id"]);old=self.timers.get(tid)
            if op=="CancelTimer":
                reject=old is None or not old["active"] or old["timer"]!=x
                if not reject:old["active"]=False;expected.append(("CancelTimer",x))
            else:
                if op=="TimerFired" and (old is None or not old["active"] or old["timer"]!=x):reject=True
                else:
                    if x["cid"]:reject=not self.guard(x["cid"],t,expected,("BFT",))
                    else:self.expire(t,expected);reject=not self.safe(t)
                    if not reject and op=="ArmTimer":
                        reject=(old is None and x["generation"]!=1) or (old is not None and (old["active"] or x["generation"]!=old["timer"]["generation"]+1))
                        if not reject:self.timers[tid]={"timer":copy.deepcopy(x),"active":True,"observed":False};expected.append(("ArmTimer",x))
                    elif not reject:
                        reject=self.local(t)<x["local_deadline_us"]
                        if not reject:old["active"]=False;old["observed"]=True;expected.append(("TimerObserved",x))
        elif op in ("Send","Deliver"):
            reject=not self.envelope_ok(x)
            if not reject and op=="Send":
                reject=x["src"]!=self.node or x["sent_at_tick_us"]!=t or x["attempt"]!=0
                if not reject:expected.append(("Send",x))
            elif not reject:
                reject=x["dst"]!=self.node or x["sent_at_tick_us"]>t
                if not reject:
                    self.expire(t,expected);mid=b(x["message_id"]).hex()
                    reject=mid in self.received
                    if not reject:
                        valid=self.safe(t)
                        self.received[mid]={"envelope":copy.deepcopy(x),"first_tick_us":t,"first_local_time_us":self.local(t) if valid else None}
                        reject=not valid
                        if valid:
                            expected.extend([("ClockRead",{"cid":x["cid"],"caller":"Receiver","tick_us":t,"node_time_us":self.local(t),"lease_end_tick_us":self.prior_end}),("MessageReceived",x)])
        else:raise Violation("unrecognized input")
        observed=[variant(e) for e in a["effects"]]
        rejection=[(k,v) for k,v in observed if k in ("Rejected","SampleRejected")]
        need(a["status"]==int(reject) and len(rejection)==int(reject),f"{op}: rejection/eligibility mismatch")
        if reject:
            k,v=rejection[0]
            need(bool(v["reason"]) and k==("SampleRejected" if op=="Sample" else "Rejected"),"rejection diagnostic")
            if op=="Sample":need(v["request_id"]==x["request_id"],"sample rejection identity")
        observed=[(k,v) for k,v in observed if k not in ("Rejected","SampleRejected")]
        # Exact values and multiplicity; dependency order is checked separately.
        norm=lambda entries:collections.Counter((k,json.dumps(v,sort_keys=True,separators=(",", ":"))) for k,v in entries)
        need(norm(observed)==norm(expected),f"{op}: output set violates independent guards")
        for i,(k,v) in enumerate(observed):
            if k=="SignedIntent":need(any(pk=="ClockRead" and pv["caller"]=="BFT" and pv["cid"]==v["cid"] for pk,pv in observed[:i]),"sign before clock guard")
        for field,value in [("runtime_incarnation",self.inc),("sample_request_counter",self.counter),("outstanding_requests",sorted(self.requests)),
                            ("accepted_sequence",self.sequence),("prior_lease_end_tick_us",self.prior_end),("sample",self.sample),
                            ("timers",self.timers),("first_received",self.received),("last_tick_us",t)]:
            need(stored[field]==value,f"{op}: durable {field} diverges from independent state")
        need(set(stored["fixtures"])==set(self.fixtures),"fixture set")
        for key,f in self.fixtures.items():
            for field,value in f.items():need(stored["fixtures"][key][field]==value,f"{op}: fixture {field} mutation")
            current_view=stored["fixtures"][key]["view"]
            if key in self.views:
                signs=[v for k,v in observed if k=="SignedIntent" and b(v["cid"]).hex()==key]
                if signs:
                    need(current_view>=max(self.views[key],signs[0]["slot"]),"fixture view regressed on signing")
                elif not (op=="Enter" and previously_entered is None and b(x["cid"]).hex()==key):
                    need(current_view==self.views[key],"fixture view reset/changed without progress")
            self.views[key]=current_view


def verify(evidence, check_processes=True):
    e=evidence;run=b(e.manifest["run_id"]);config=b(e.manifest["config_hash"]);seed=e.scenario["seed"]
    need(e.manifest["outcome"]=="COMPLETED" and e.manifest["gate"]=="I1a","not completed I1a evidence")
    need(config==hp("I1/config/v1",compact(e.scenario)) and run==hp("I1/run/v1",config),"run/config binding")
    records={};all_records=[[],[]]
    for node,raw in enumerate(e.journals):
        offset=0;previous=bytes(32);seq=0
        while offset<len(raw):
            need(len(raw)-offset>=48,"incomplete final evidence frame")
            n,complement=struct.unpack_from(">QQ",raw,offset)
            need(0<n<=16*1024*1024 and n^complement==(1<<64)-1,"journal protected length")
            payload=raw[offset+16:offset+16+n];checksum=raw[offset+16+n:offset+48+n]
            need(len(checksum)==32 and checksum==hp("I1/journal/v1",payload),"journal checksum")
            record=json.loads(payload);need(compact(record)==payload,"journal canonical bytes")
            seq+=1;a=record["ack"];s=record["resulting_state"]
            need(record["journal_seq"]==seq==a["durable_journal_seq"],"journal sequence")
            need(b(record["previous_record_hash"])==previous,"journal hash chain")
            need(a["logical_step_id"]==record["logical_step_id"] and a["grant_hash"]==record["grant_hash"],"journal Ack binding")
            need(b(a["state_hash"])==hp("I1/state/v1",compact(s)),"journal state hash")
            need(b(a["logical_ack_hash"])==logical_hash("Ack",a),"journal Ack hash")
            need(b(s["run_id"])==run and b(s["config_hash"])==config and s["node_id"]==node,"journal identity")
            need(a["node_id"]==node and b(a["run_id"])==run and a["schema_version"]==1 and a["incarnation"]>0,"Ack identity")
            key=(node,a["logical_step_id"]);need(key not in records,"duplicate journal step")
            records[key]=record;all_records[node].append(record);previous=checksum;offset+=n+48
        head=e.heads[node]
        need(len(head)==88 and head[:8]==b"I1HEAD01","journal head shape")
        need(head[56:]==hp("I1/journal-head/v1",run,config,be(node,4),head[:56]),"journal head checksum")
        need(struct.unpack_from(">QQ",head,8)==(len(raw),seq) and head[24:56]==previous,"journal committed floor")
    models=[Model(run,config,i,e.scenario["epsilon_us"][i]) for i in range(2)]
    grants={};active=None;duplicate=False;durable=None;committed={};effects=collections.deque();transport=[]
    registered=[0,0];last_key=None;queues=collections.defaultdict(collections.deque);sent={};delivered=set();counts=collections.Counter()
    timer_due={};last_served={};requests={};service_sequence=[0,0]
    for line,row in enumerate(e.rows,1):
        kind,d,event=row["kind"],row["detail"],row["event"];counts[kind]+=1
        if event is not None:
            need(b(event["tie_rank"])==hp("I1/tie/v1",be(seed),be(event["event_id"])),f"line{line}: tie rank")
            current=(event["tick_us"],event["causal_wave"],b(event["tie_rank"]),event["event_id"])
            need(last_key is None or current>=last_key,f"line{line}: event order")
            last_key=current
        if kind=="Registered":
            node=d["node_id"];need(node in (0,1) and d["incarnation"]==registered[node]+1,"registration incarnation")
            registered[node]=d["incarnation"]
            transport.append((1,node,proto("WorkerFrame",{"frame":{"Hello":d}})))
        elif kind in ("Grant","RecoverPending","GrantDuplicate"):
            g=d;node=g["node_id"];step=g["logical_step_id"]
            need(node in (0,1) and b(g["run_id"])==run and g["schema_version"]==1 and g["incarnation"]==registered[node],"grant identity")
            need(b(g["grant_hash"])==logical_hash("Grant",g),"grant hash")
            if kind=="Grant":
                need(active is None,"two globally outstanding grants");need(step not in grants,"new grant reused step")
                grants[step]=copy.deepcopy(g);active=g;duplicate=False
            elif kind=="RecoverPending":
                need(active is not None and active["logical_step_id"]==step and active["grant_hash"]==g["grant_hash"],"unauthorized recovery replay")
                active=g
            else:
                need(active is None and (node,step) in committed,"unsolicited duplicate grant")
                active=g;duplicate=True
            transport.append((0,node,proto("HubFrame",{"frame":{"Grant":g}})))
        elif kind=="StepAborted":
            need(active is not None and active["logical_step_id"]==d["step"],"abort without grant")
            need((d["node_id"],d["step"]) not in records,"aborted step became durable")
            active=None
        elif kind=="JournalDurable":
            need(active is not None and active["logical_step_id"]==d["logical_step_id"],"durability outside grant")
            durable=d
        elif kind in ("AckCommitted","AckDuplicate"):
            a=d["ack"] if kind=="AckCommitted" else d;node=a["node_id"];step=a["logical_step_id"]
            need(node in (0,1) and a["incarnation"]==registered[node] and a["schema_version"]==1 and b(a["run_id"])==run,"Ack transport/schema identity")
            need(b(a["logical_ack_hash"])==logical_hash("Ack",a),"trace Ack hash")
            transport.append((1,node,proto("WorkerFrame",{"frame":{"Ack":a}})))
            key=(node,step)
            if kind=="AckDuplicate":
                need(key in committed and canonical_ack(a)==canonical_ack(committed[key]),"altered or unsolicited duplicate Ack")
                if active is not None:need(duplicate and active["logical_step_id"]==step,"duplicate for active new step");active=None;duplicate=False
                continue
            need(active is not None and not duplicate and active["logical_step_id"]==step and active["node_id"]==node,"Ack outside outstanding grant")
            need(a["event_id"]==active["event_id"] and a["grant_hash"]==active["grant_hash"] and a["incarnation"]==active["incarnation"],"Ack grant binding")
            need(durable is not None and durable["logical_step_id"]==step and durable["journal_seq"]==a["durable_journal_seq"] and durable["state_hash"]==a["state_hash"],"missing sync-before-commit evidence")
            need(key not in committed and key in records,"commit exactly once with journal")
            record=records[key];need(canonical_ack(a)==canonical_ack(record["ack"]),"replay Ack differs from original journal")
            models[node].apply(grants[step],a,record["resulting_state"])
            committed[key]=a
            for effect in a["effects"]:effects.append((node,event,effect))
            active=None;durable=None
        elif kind=="Effect":
            need(effects and effects.popleft()==(d["node_id"],event,d["effect"]),"output outside batch or changed ordered effect")
            name,value=variant(d["effect"])
            node=d["node_id"];tick=event["tick_us"]
            if name=="Send":
                mid=b(value["message_id"])
                need(mid not in sent or sent[mid]==value,"logical message identity collision")
                sent[mid]=copy.deepcopy(value)
            elif name=="RequestSample":
                requests[(node,registered[node],value["request_id"])]=tick
            elif name=="ArmTimer":
                timer_due[(node,value["timer_id"])]=(copy.deepcopy(value),max(tick,value["local_deadline_us"]-models[node].epsilon))
            elif name in ("CancelTimer","TimerObserved"):
                slot=(node,value["timer_id"])
                need(slot in timer_due and timer_due[slot][0]==value,"timer removal without matching generation")
                del timer_due[slot]
            elif name=="SampleAccepted":
                for slot,(timer,due) in list(timer_due.items()):
                    if slot[0]==node and due is None:
                        timer_due[slot]=(timer,max(tick,timer["local_deadline_us"]-models[node].epsilon))
        elif kind=="TimerDelivery":
            slot=(d["node_id"],d["timer"]["timer_id"])
            need(slot in timer_due and timer_due[slot]==(d["timer"],event["tick_us"]),"timer did not fire at original eligible deadline")
        elif kind=="TimerSuppressed" and d["active"]:
            slot=(d["node_id"],d["timer"]["timer_id"])
            need(slot in timer_due and timer_due[slot]==(d["timer"],event["tick_us"]),"suppressed timer deadline/generation")
            timer_due[slot]=(d["timer"],None)
        elif kind=="ClockServiceSample":
            sample=d["sample"];node=sample["node_id"];key=(node,sample["incarnation"],sample["request_id"])
            need(key in requests,"sample without committed request")
            override=next((o for o in e.scenario["clock"]["overrides"] if o["node"]==node and o["request_id"]==sample["request_id"]),None)
            request_delay=override["request_us"] if override else e.scenario["clock"]["default_request_us"]
            response_delay=override["response_us"] if override else e.scenario["clock"]["default_response_us"]
            service_sequence[node]+=1
            need(event["tick_us"]==requests[key]+request_delay==sample["as_of_tick_us"],"RPC request/service time")
            need(sample["sequence"]==service_sequence[node] and sample["node_time_at_as_of_us"]==models[node].local(event["tick_us"]),"authority sample order/oscillator")
            need(sample["lease_end_tick_us"]==event["tick_us"]+e.scenario["clock"]["lease_us"] and d["one_way_response_delay_us"]==response_delay,"authority lease/RPC profile")
        elif kind=="PacketAdmitted":
            packet=d["envelope"];mid=b(packet["message_id"]);need(mid in sent,"network output bypassed committed send")
            original=copy.deepcopy(packet);original["attempt"]=0
            need(original==sent[mid],"retry changed original message bytes/identity")
            route=(packet["src"],packet["dst"],packet["channel"])
            need(len(queues[route])<64,"link capacity")
            link=next(link for link in e.scenario["links"] if link["channel"]==packet["channel"])
            jitter=int.from_bytes(hp("I1/tie/v1",be(seed),be(event["event_id"]))[:8],"big")%(link["jitter_us"]+1) if link["jitter_us"] else 0
            need(d["jitter_us"]==jitter and d["earliest_delivery_tick_us"]==event["tick_us"]+link["delay_us"]+jitter,"network delay/jitter profile")
            queues[route].append((mid,packet["attempt"],d["earliest_delivery_tick_us"]))
        elif kind in ("PacketDelivered","PacketDroppedOffline"):
            packet=d["envelope"];route=(packet["src"],packet["dst"],packet["channel"])
            need(queues[route],"delivery without admission")
            mid,attempt,due=queues[route].popleft()
            need((mid,attempt)==(b(packet["message_id"]),packet["attempt"]) and event["tick_us"]==max(due,last_served.get(route,0)),"per-link FIFO or exact latency")
            last_served[route]=event["tick_us"]
            if kind=="PacketDelivered":delivered.add((mid,attempt))
        elif kind=="QueueOverflow":
            p=d["envelope"];need(len(queues[(p["src"],p["dst"],p["channel"])])==64 and d["policy"]=="drop-newest","overflow policy")
        elif kind=="InfrastructureFailure":raise Violation("infrastructure failure is not a pass")
        elif kind=="RunComplete":need(active is None and not effects,"incomplete work at run completion")
    need(counts["RunComplete"]==1 and active is None and not effects,"missing complete trace")
    need(set(committed)==set(records),"journal/commit coverage")
    raw=e.frames;offset=0;actual=[]
    while offset<len(raw):
        need(len(raw)-offset>=10,"partial transport frame")
        direction,node=raw[offset:offset+2];n=int.from_bytes(raw[offset+2:offset+10],"big")
        need(n<=1048576 and offset+10+n<=len(raw),"transport frame bound")
        actual.append((direction,node,raw[offset+10:offset+10+n]));offset+=10+n
    # Initial Register/Hello receipt order is OS transport information. The hub
    # emits those two logical registration records in node order. Every later
    # frame must preserve the exact logical chronology, including duplicates.
    need(len(actual)==len(transport) and sorted(actual[:2],key=lambda x:x[1])==transport[:2]
         and actual[2:]==transport[2:],"raw protobuf/trace chronology")
    if check_processes:
        starts=[];ends=[]
        for node in range(2):
            history=json.loads((e.path/f"worker-{node}/process-history.json").read_text())
            ss=[h["detail"] for h in history if h["kind"]=="ProcessStarted"]
            ee=[h["detail"] for h in history if h["kind"]=="ProcessExited"]
            need(len(ss)==len(ee)==registered[node],"child cleanup history")
            need(all(x["exit_code"] in (0,71,72,73,74,75) for x in ee),"unclassified child failure")
            starts.append(ss[0]);ends.append(ee[0])
        need(starts[0]["pid"]!=starts[1]["pid"] and max(s["wall_elapsed_us"] for s in starts)<min(x["wall_elapsed_us"] for x in ends),"two concurrently live subprocesses")
    return {"outcome":"PASS","gate":"I1a-recorded-trace","scenario":e.scenario["name"],"seed":seed,
            "rows":len(e.rows),"committed_steps":len(committed),"signed_intents":sum(m.signed_count for m in models),
            "timer_deliveries":counts["TimerDelivery"],"network_deliveries":counts["PacketDelivered"],
            "trace_sha256":hashlib.sha256(compact(e.rows)).hexdigest()}


def rebuild_frames(e):
    out=bytearray()
    for r in e.rows:
        k,d=r["kind"],r["detail"]
        if k=="Registered":direction,node,name,obj=1,d["node_id"],"WorkerFrame",{"frame":{"Hello":d}}
        elif k in ("Grant","RecoverPending","GrantDuplicate"):direction,node,name,obj=0,d["node_id"],"HubFrame",{"frame":{"Grant":d}}
        elif k=="AckCommitted":direction,node,name,obj=1,d["ack"]["node_id"],"WorkerFrame",{"frame":{"Ack":d["ack"]}}
        elif k=="AckDuplicate":direction,node,name,obj=1,d["node_id"],"WorkerFrame",{"frame":{"Ack":d}}
        else:continue
        data=proto(name,obj);out.extend(bytes([direction,node])+be(len(data))+data)
    e.frames=bytes(out)


def mutation_checks(original):
    """All mutation inputs are private in-memory copies; candidate runs stay read-only."""
    results=[]
    for mode in ("missing_sync","orphan_effect","altered_duplicate","lease_extension_rehashed","post_close_intent_rehashed","sample_counter_poison_rehashed"):
        e=copy.deepcopy(original)
        if mode=="missing_sync":
            i=next(i for i,r in enumerate(e.rows) if r["kind"]=="JournalDurable");del e.rows[i]
        elif mode=="orphan_effect":
            row=copy.deepcopy(next(r for r in e.rows if r["kind"]=="Effect"));e.rows.insert(0,row)
        elif mode=="altered_duplicate":
            a=copy.deepcopy(next(r for r in e.rows if r["kind"]=="AckCommitted")["detail"]["ack"])
            a["state_hash"][0]^=1;a["logical_ack_hash"]=list(logical_hash("Ack",a))
            e.rows.append({"kind":"AckDuplicate","event":None,"detail":a});rebuild_frames(e)
        else:
            # Recompute record, state, Ack and committed-head hashes. Grant inputs
            # remain authentic, so failures must reach semantic state/output checks.
            node=0;parsed=[];raw=e.journals[node];at=0
            while at<len(raw):n=int.from_bytes(raw[at:at+8],"big");parsed.append(json.loads(raw[at+16:at+16+n]));at+=n+48
            if mode=="lease_extension_rehashed":
                target=next(r for r in parsed if r["resulting_state"]["sample"] is not None and any(variant(v)[0]=="ClockRead" for v in r["ack"]["effects"]))
                target["resulting_state"]["prior_lease_end_tick_us"]+=1
                target["resulting_state"]["sample"]["lease_end_tick_us"]+=1
            elif mode=="post_close_intent_rehashed":
                target=next(r for r in parsed if any(variant(v)[0]=="SignedIntent" for v in r["ack"]["effects"]))
                fixture=next(iter(target["resulting_state"]["fixtures"].values()))
                fixture["terminal"]={"reason":"Close","tick_us":target["resulting_state"]["last_tick_us"]-1,"lease_end_tick_us":target["resulting_state"]["prior_lease_end_tick_us"]}
            else:
                target=next(r for r in parsed if r["resulting_state"]["sample"] is not None)
                target["resulting_state"]["accepted_sequence"]=100
            prev=bytes(32);data=bytearray()
            for record in parsed:
                record["previous_record_hash"]=list(prev)
                a=record["ack"];a["state_hash"]=list(hp("I1/state/v1",compact(record["resulting_state"])))
                a["logical_ack_hash"]=list(logical_hash("Ack",a));payload=compact(record);prev=hp("I1/journal/v1",payload)
                data.extend(be(len(payload))+be(((1<<64)-1)^len(payload))+payload+prev)
                for row in e.rows:
                    if row["kind"]=="AckCommitted" and row["detail"]["ack"]["node_id"]==node and row["detail"]["ack"]["logical_step_id"]==a["logical_step_id"]:
                        row["detail"]["ack"]=copy.deepcopy(a)
                    if row["kind"]=="JournalDurable" and row["detail"]["node_id"]==node and row["detail"]["logical_step_id"]==a["logical_step_id"]:
                        row["detail"]["state_hash"]=a["state_hash"]
            e.journals[node]=bytes(data)
            head=b"I1HEAD01"+be(len(data))+be(len(parsed))+prev
            e.heads[node]=head+hp("I1/journal-head/v1",b(e.manifest["run_id"]),b(e.manifest["config_hash"]),be(node,4),head)
            rebuild_frames(e)
        try:
            verify(e,False)
        except Violation as error:
            results.append({"mutation":mode,"outcome":"REJECTED","reason":str(error)})
        else:raise Violation(f"mutation survived: {mode}")
    return results


def main():
    ap=argparse.ArgumentParser();ap.add_argument("paths",nargs="+");ap.add_argument("--mutations",action="store_true");ap.add_argument("--output",type=Path)
    args=ap.parse_args();report={"checker_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),"runs":[],"limits":["finite I1a fixtures only","no physical power-loss guarantee","no I1b or BFT safety/liveness claim"]}
    failed=False
    for path in args.paths:
        try:
            evidence=Evidence(path);result=verify(evidence);result["path"]=str(path)
            if args.mutations and evidence.scenario["name"]=="healthy":result["mutations"]=mutation_checks(evidence)
            report["runs"].append(result)
        except (Violation,KeyError,ValueError,OSError) as error:
            report["runs"].append({"outcome":"FAIL","path":str(path),"reason":str(error)});failed=True
    encoded=json.dumps(report,indent=2)
    if args.output:args.output.write_text(encoded+"\n")
    print(encoded)
    return int(failed)


if __name__=="__main__":
    sys.exit(main())

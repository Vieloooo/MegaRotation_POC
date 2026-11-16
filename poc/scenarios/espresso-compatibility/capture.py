#!/usr/bin/env python3
"""Record a bounded official dev-node run inside an isolated loopback namespace."""

import argparse
import base64
from dataclasses import dataclass
import hashlib
import http.client
import json
import os
from pathlib import Path
import re
import signal
import socket
import stat
import subprocess
import time

PORTS = {"api": 18770, "dev": 18771, "builder": 18772, "prover": 18773, "tonic": 18774, "anvil": 18775}
NAMESPACE = 160016
MARKERS = (b"EC16:non-cryptographic-publication:A:v1", b"EC16:non-cryptographic-publication:B:v1")
MAX_RESPONSE = 16 * 1024 * 1024
MAX_BUNDLE = 256 * 1024 * 1024
MAX_CAPTURE_HEIGHT = 128
SRS = "kzg10-aztec20-srs-1048584.bin"
API = "/v1/availability"
INPUT_SCHEMA = "H3-PUBLICATION-ARTIFACT-1"
GFR_PUBLICATION_CODEC = 0x47465201
MAX_INPUT_BYTES = 65_536
MAX_INPUT_MESSAGES = 11


def strict_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def reject_number(value):
    raise ValueError(f"noninteger JSON number {value!r}")


def checked_input_file(root_fd, relative, limit):
    if (type(relative) is not str or "\\" in relative or "\0" in relative
            or any(part in ("", ".", "..") for part in relative.split("/"))):
        raise ValueError("input paths must be relative regular files under the manifest directory")
    parts = relative.split("/")
    directory = os.dup(root_fd)
    try:
        for part in parts[:-1]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=directory)
            os.close(directory)
            directory = child
        fd = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=directory)
        with os.fdopen(fd, "rb") as stream:
            info = os.fstat(stream.fileno())
            if not stat.S_ISREG(info.st_mode) or info.st_size > limit:
                raise ValueError(f"input file is not a regular file within {limit} bytes: {relative!r}")
            data = stream.read(limit + 1)
        if len(data) != info.st_size or len(data) > limit:
            raise ValueError(f"input size changed during bounded read: {relative!r}")
        return data
    finally:
        os.close(directory)


def fields(value, expected, label):
    if type(value) is not dict or value.keys() != set(expected.split()):
        raise ValueError(f"invalid {label} fields")


def matches(value, pattern):
    return type(value) is str and re.fullmatch(pattern, value) is not None


def load_payload_manifest(path):
    """Return validated metadata, exact manifest bytes, immutable payloads and optional report."""
    path = Path(path)
    root = os.open(path.parent.resolve(strict=True), os.O_RDONLY | os.O_DIRECTORY)
    try:
        raw = checked_input_file(root, path.name, MAX_INPUT_BYTES)
        manifest = json.loads(raw.decode("utf-8"), object_pairs_hook=strict_object,
                              parse_float=reject_number, parse_constant=reject_number)
        fields(manifest, "schema evidence_kind producer_candidate_sha run_id result_key_hex native_evidence messages",
               "publication manifest")
        if manifest["schema"] != INPUT_SCHEMA or manifest["evidence_kind"] not in ("synthetic-adapter", "native-regional"):
            raise ValueError("invalid input schema or evidence kind")
        if (not matches(manifest["producer_candidate_sha"], r"[0-9a-f]{40}")
                or not matches(manifest["run_id"], r"[A-Za-z0-9_-]{1,64}")):
            raise ValueError("invalid input producer revision or run ID")
        key, native = manifest["result_key_hex"], manifest["native_evidence"]
        if key is not None and not matches(key, r"0001[0-9a-f]{128}"):
            raise ValueError("invalid canonical ResultKey")
        if manifest["evidence_kind"] == "native-regional" and (key is None or native is None):
            raise ValueError("native-regional input requires ResultKey and evidence provenance")
        if native is not None:
            fields(native, "h1_revision report_path report_sha256", "native evidence")
            if not matches(native["h1_revision"], r"[0-9a-f]{40}") or not matches(native["report_sha256"], r"[0-9a-f]{64}"):
                raise ValueError("invalid native evidence revision or SHA-256")
        messages = manifest["messages"]
        if type(messages) is not list or not 1 <= len(messages) <= MAX_INPUT_MESSAGES:
            raise ValueError("input must contain 1..11 messages")
        total = 0
        for ordinal, message in enumerate(messages):
            fields(message, "ordinal codec path length sha256", "input message")
            if type(message["ordinal"]) is not int or message["ordinal"] != ordinal:
                raise ValueError("input ordinals must be contiguous from zero")
            if type(message["codec"]) is not int or message["codec"] != GFR_PUBLICATION_CODEC:
                raise ValueError("input codec must be GFR-PUBLIC-1")
            length = message["length"]
            if type(length) is not int or not 1 <= length <= MAX_INPUT_BYTES:
                raise ValueError("invalid input payload length")
            total += length
            if total > MAX_INPUT_BYTES or not matches(message["sha256"], r"[0-9a-f]{64}"):
                raise ValueError("input aggregate capacity or invalid SHA-256")

        # Read each distinct path once. Repeated ordinals preserve their own
        # identity and must agree with the same immutable bytes.
        seen = {path.name: raw}

        def verified_file(name, limit, sha):
            if type(name) is not str or name == path.name:
                raise ValueError("input path conflicts with the manifest")
            if name not in seen:
                seen[name] = checked_input_file(root, name, limit)
            data = seen[name]
            if len(data) > limit or hashlib.sha256(data).hexdigest() != sha:
                raise ValueError(f"input SHA-256/size mismatch for {name!r}")
            return data

        payloads = tuple(verified_file(m["path"], m["length"], m["sha256"]) for m in messages)
        if any(len(data) != m["length"] for data, m in zip(payloads, messages)):
            raise ValueError("input payload does not have its declared length")
        report = None
        if native is not None:
            if any(native["report_path"] == message["path"] for message in messages):
                raise ValueError("native report path conflicts with a payload")
            report = verified_file(native["report_path"], MAX_INPUT_BYTES, native["report_sha256"])
        return manifest, raw, payloads, report
    finally:
        os.close(root)


@dataclass(frozen=True)
class Response:
    status: int | None
    body: bytes
    record_id: int

    def json(self):
        if self.status != 200:
            raise RuntimeError(f"HTTP record {self.record_id} has status {self.status}")
        return json.loads(self.body)


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def origin(service):
    return f"http://127.0.0.1:{PORTS[service]}"


def copy_payload_inputs(record, loaded, runner_sha):
    manifest, raw, payloads, report = loaded
    manifest_copy = "publication-input-manifest.json"
    record.artifact(manifest_copy, raw)
    native = manifest["native_evidence"]
    if native is not None:
        report_copy = "publication-input-native-report.bin"
        record.artifact(report_copy, report)
        native = dict(native, copied_path=report_copy)
    messages = []
    for message, data in zip(manifest["messages"], payloads):
        copied = f"publication-input-{message['ordinal']:03d}.bin"
        record.artifact(copied, data)
        messages.append(dict(message, copied_path=copied, submit_record_id=None, ack_hash=None,
                             occurrences=[], byte_equal=False))
    return {
        "manifest_path": manifest_copy, "manifest_sha256": hashlib.sha256(raw).hexdigest(),
        "producer_candidate_sha": manifest["producer_candidate_sha"], "producer_run_id": manifest["run_id"],
        "runner_candidate_sha": runner_sha, "result_key_hex": manifest["result_key_hex"],
        "native_evidence": native, "native_verification": "NOT_PERFORMED",
        "messages": messages, "repeat": None,
    }


class Recorder:
    def __init__(self, root):
        self.root = root
        self.records = []
        self.artifacts = []
        self.raw_bytes = 0
        (root / "raw").mkdir(parents=True)

    def file(self, filename, data):
        if len(data) > MAX_RESPONSE:
            raise ValueError(f"raw artifact exceeds 16 MiB: {filename}")
        if self.raw_bytes + len(data) > MAX_BUNDLE:
            raise ValueError("bundle exceeds 256 MiB")
        with (self.root / filename).open("xb") as out:
            out.write(data)
        self.raw_bytes += len(data)
        return {"path": filename, "sha256": hashlib.sha256(data).hexdigest()}

    def artifact(self, filename, data):
        self.artifacts.append(self.file(filename, data))

    def http(self, reader, method, path, *, service="api", body=None):
        if not path.startswith("/") or path.startswith("//") or "://" in path:
            raise ValueError(f"invalid task-local HTTP path {path!r}")
        record_id = len(self.records)
        stem = f"raw/{record_id:06d}"
        response_body = b""
        request = None if body is None else self.file(stem + ".request", body)
        headers = {"Accept": "application/json", "Accept-Encoding": "identity", "Connection": "close"}
        if body is not None:
            headers["Content-Type"] = "application/json"
        wall_started = time.time_ns()
        start = time.monotonic_ns()
        observation = {
            "origin": origin(service),
            "request_headers": headers,
            "wall_started_ns": wall_started,
            "monotonic_started_ns": start,
        }
        status = None
        response = None
        connection = http.client.HTTPConnection("127.0.0.1", PORTS[service], timeout=4)
        try:
            connection.request(method, path, body=body, headers=headers)
            with connection.getresponse() as result:
                observation["headers_received_monotonic_ns"] = time.monotonic_ns()
                observation["response_headers"] = result.getheaders()
                observation["reason"] = result.reason
                response_body = result.read(MAX_RESPONSE + 1)
                status = result.status
        except (OSError, http.client.HTTPException) as error:
            observation["transport_error"] = f"{type(error).__name__}: {error}"
        finally:
            observation["wall_completed_ns"] = time.time_ns()
            observation["monotonic_completed_ns"] = time.monotonic_ns()
            connection.close()
        if status is not None:
            response = self.file(stem + ".response", response_body)
        self.records.append({
            "id": record_id, "reader_id": reader, "method": method, "path": path,
            "request": request, "status": status, "response": response, "observation": observation,
        })
        if status is not None and 300 <= status < 400:
            raise RuntimeError(f"unexpected redirect at local HTTP record {record_id}")
        return Response(status, response_body, record_id)

    def seal(self, identity, configuration):
        fields = (
            "schema", "official_commit", "image_amd64_digest", "protocol_version",
            "api_version", "namespace", "run_id", "genesis_block_hash",
        )
        preimage = "".join(f"{identity[field]}\n" for field in fields).encode()
        source_id = "EC16:" + hashlib.sha256(preimage).hexdigest()
        for record in self.records:
            record["source_id"] = source_id
        self.file("manifest.json", json_bytes({
            "schema": "EC16-CAPTURE-1", "capture_kind": "official", "identity": identity,
            "source_id": source_id, "configuration": configuration, "records": self.records,
            "artifacts": self.artifacts,
        }))


class Processes:
    def __init__(self, directory):
        self.children = []
        self.dir = directory
        directory.mkdir()

    def start(self, name, argv, environment):
        with (self.dir / f"{name}.log").open("xb") as log:
            process = subprocess.Popen(
                argv, env=environment, cwd=self.dir, stdout=log, stderr=subprocess.STDOUT,
                start_new_session=True,
            )
        self.children.append((name, process))
        return process.pid

    def check(self):
        for name, process in self.children:
            if process.poll() is not None:
                raise RuntimeError(f"owned process {name} exited early with status {process.returncode}")

    def close(self):
        results = {}
        for name, process in reversed(self.children):
            if process.poll() is None:
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            try:
                status = process.wait(timeout=8)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                status = process.wait(timeout=3)
            results[name] = {"namespace_pid": process.pid, "returncode": status}
        return results


# Alloy's TestConfigBuilder first resolves anvil through PATH, then discards
# that bootstrap provider when given the dedicated L1 URL. Keep upstream's
# argv unchanged and attach every exec to the local capture.
_BOOTSTRAP_WRAPPER = r'''
import json
import os
from pathlib import Path
import sys

namespace_pid = os.getpid()
namespace_ppid = os.getppid()
start_ticks = int(Path("/proc/self/stat").read_text().rsplit(")", 1)[1].split()[19])
port_args = []
for index, arg in enumerate(sys.argv[1:], 1):
    if arg in ("--port", "-p"):
        port_args.append(sys.argv[index:index + 2])
    elif arg.startswith("--port=") or (arg.startswith("-p") and arg[2:].isdigit()):
        port_args.append([arg])
exec_argv = COMMAND + sys.argv[1:]
evidence = {
    "argv": sys.argv,
    "namespace_pid": namespace_pid,
    "namespace_ppid": namespace_ppid,
    "start_ticks": start_ticks,
    "port_args": port_args,
    "exec_argv": exec_argv,
}
path = Path(INVOCATIONS) / f"{namespace_pid}-{start_ticks}.json"
with path.open("x", encoding="utf-8") as stream:
    json.dump(evidence, stream, sort_keys=True)
    stream.write("\n")
    stream.flush()
    os.fsync(stream.fileno())
directory_fd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
try:
    os.fsync(directory_fd)
finally:
    os.close(directory_fd)
os.execv(exec_argv[0], exec_argv)
'''


class BootstrapAnvil:
    def __init__(self, run, configuration):
        self.invocations = run / "bootstrap-anvil-invocations"
        self.bin = run / "bin"
        self.wrapper = self.bin / "anvil"
        self.details = {
            "role": "upstream TestConfigBuilder bootstrap before the dedicated L1 override",
            "wrapper": str(self.wrapper),
            "invocations": [],
        }
        configuration["bootstrap_anvil"] = self.details

    def prepare(self, loader, native, recorder):
        self.bin.mkdir(mode=0o700)
        self.invocations.mkdir(mode=0o700)
        command = loader + [str(native / "anvil")]
        source = (
            "#!/usr/bin/python3\n"
            f"COMMAND = {command!r}\n"
            f"INVOCATIONS = {str(self.invocations)!r}\n"
            + _BOOTSTRAP_WRAPPER
        ).encode()
        with self.wrapper.open("xb") as output:
            output.write(source)
        self.wrapper.chmod(0o700)
        recorder.artifact("bootstrap-anvil-wrapper.py", source)
        self.details["wrapper_artifact"] = "bootstrap-anvil-wrapper.py"
        self.details["exec_command_prefix"] = command

    @staticmethod
    def still_present(invocation):
        pid = invocation["namespace_pid"]
        try:
            stat = Path(f"/proc/{pid}/stat").read_text()
        except (FileNotFoundError, ProcessLookupError):
            return False
        start_ticks = int(stat.rsplit(")", 1)[1].split()[19])
        return start_ticks == invocation["start_ticks"]

    def finish(self, node_pid, recorder):
        invocations = []
        if self.invocations.exists():
            for path in sorted(self.invocations.glob("*.json")):
                with path.open("rb") as stream:
                    raw = stream.read(MAX_RESPONSE + 1)
                name = f"bootstrap-anvil-{path.name}"
                recorder.artifact(name, raw)
                invocation = json.loads(raw)
                invocation["artifact"] = name
                invocations.append(invocation)
        self.details["invocations"] = invocations
        self.details["namespace_espresso_pid"] = node_pid
        if not invocations:
            self.details["cleanup"] = "no wrapper invocation recorded"
            if node_pid is not None:
                raise RuntimeError("upstream bootstrap Anvil invocation is missing")
            return

        # The wrapper execs the official binary with the same PID. Check the
        # original PID and start ticks together so reuse cannot target or mask
        # a leftover. The namespace supervisor owns final teardown on failure.
        deadline = time.monotonic() + 3
        while True:
            leftovers = [entry["namespace_pid"] for entry in invocations if self.still_present(entry)]
            if not leftovers or time.monotonic() >= deadline:
                break
            time.sleep(0.05)
        self.details["cleanup"] = {
            "checked_after_owned_parent_wait": True,
            "leftover_namespace_pids": leftovers,
        }
        if leftovers:
            raise RuntimeError(f"upstream bootstrap Anvil survived parent cleanup: {leftovers}")

        for entry in invocations:
            if entry["namespace_ppid"] != node_pid:
                raise RuntimeError("bootstrap Anvil parent does not match the owned Espresso PID")
        if not any(entry["port_args"] for entry in invocations):
            raise RuntimeError("no bootstrap Anvil invocation recorded a port argument")


def unavailable(status):
    return status is None or status in (404, 408, 429) or 500 <= status <= 599


def wait_for(get, condition, deadline, label, processes):
    while time.monotonic() < deadline:
        processes.check()
        outcome = get()
        if outcome.status == 200:
            value = outcome.json()
            if condition(value):
                return value
        elif not unavailable(outcome.status):
            raise RuntimeError(f"{label}: unexpected status {outcome.status}")
        time.sleep(0.3)
    raise TimeoutError(f"deadline awaiting {label}")


def available(response):
    if response.status == 200:
        return True
    if unavailable(response.status):
        return False
    raise RuntimeError(f"unexpected lookup status {response.status} at record {response.record_id}")


def read_block(record, reader, q):
    block = record.http(reader, "GET", f"{API}/block/{q}")
    if not available(block):
        return None
    block_value = block.json()
    count = block_value["num_transactions"]
    if type(count) is not int or not 0 <= count <= 1024:
        raise ValueError("invalid official transaction count")
    leaf = record.http(reader, "GET", f"{API}/leaf/{q}")
    if not available(leaf):
        return None
    transactions = []
    for i in range(count):
        tx = record.http(reader, "GET", f"{API}/transaction/{q}/{i}/noproof")
        if not available(tx):
            return None
        transactions.append(tx.json())
    return block_value, transactions


def await_block(record, reader, q, deadline, processes):
    while time.monotonic() < deadline:
        processes.check()
        block = read_block(record, reader, q)
        if block is not None:
            return block
        time.sleep(0.3)
    raise TimeoutError(f"deadline awaiting complete {reader} block {q}")


def submit(record, payload):
    request = json_bytes({"namespace": NAMESPACE, "payload": base64.b64encode(payload).decode()})
    return record.http("first", "POST", "/v1/submit/submit", body=request)


def submit_hash(response):
    value = response.json()
    if not isinstance(value, str) or not re.fullmatch(r"TX~[A-Za-z0-9_-]+", value):
        raise ValueError(f"unrecognized official submit ACK {value!r}")
    return value


def observe(record, args, processes, configuration, bootstrap, payloads=MARKERS):
    if {name for _, name in socket.if_nameindex()} != {"lo"}:
        raise RuntimeError("refusing official launch outside a loopback-only namespace")
    # Check our isolated namespace before starting any server that binds the
    # wildcard address. A host-global ephemeral port scan is unnecessary.
    for port in PORTS.values():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
            probe.bind(("127.0.0.1", port))

    native = args.image / "native"
    loader = [str(native / "lib/ld-linux-x86-64.so.2"), "--library-path", str(native / "lib")]
    anvil = loader + [
        str(native / "anvil"), "--host", "127.0.0.1", "--port", str(PORTS["anvil"]),
        "--slots-in-an-epoch", "0", "--balance", "1000000", "--chain-id", "31337", "--silent",
    ]
    espresso = loader + [
        str(native / "espresso-dev-node"), "--rpc-url", origin("anvil"),
        "--sequencer-api-port", str(PORTS["api"]), "--dev-node-port", str(PORTS["dev"]),
        "--builder-port", str(PORTS["builder"]), "--prover-port", str(PORTS["prover"]),
        "--tonic-port", str(PORTS["tonic"]), "--path", str(args.run / "db"), "--version", "0.3",
    ]
    environment = {
        "PATH": "/usr/bin:/bin", "TMPDIR": "/tmp", "TZ": "UTC", "LC_ALL": "C",
        "AZTEC_SRS_PATH": str(native / SRS), "RUST_LOG": "warn",
        "TOKIO_WORKER_THREADS": "2", "RAYON_NUM_THREADS": "2",
    }
    bootstrap.prepare(loader, native, record)
    configuration["commands"].update({"anvil": anvil, "espresso": espresso})
    configuration["environment"] = environment
    configuration["l1"] = {"chain_id": 31337, "url": origin("anvil"), "mode": "dedicated Anvil"}
    configuration["processes"] = {}
    configuration["processes"]["anvil"] = processes.start("anvil", anvil, environment)
    body = json_bytes({"id": 1, "jsonrpc": "2.0", "method": "eth_chainId", "params": []})
    wait_for(
        lambda: record.http("first", "POST", "/", service="anvil", body=body),
        lambda value: value.get("result") == "0x7a69",
        time.monotonic() + 20, "private Anvil eth_chainId", processes,
    )
    espresso_environment = dict(environment, PATH=f"{bootstrap.bin}:/usr/bin:/bin")
    configuration["espresso_environment"] = espresso_environment
    configuration["processes"]["espresso"] = processes.start("espresso", espresso, espresso_environment)

    height_path = "/v1/node/block-height"
    readiness = time.monotonic() + 90
    start_height = wait_for(
        lambda: record.http("first", "GET", height_path),
        lambda value: type(value) is int and value >= 3,
        readiness, "official node height at least three", processes,
    )
    dev_info = wait_for(
        lambda: record.http("first", "GET", "/api/dev-info", service="dev"),
        lambda value: value["sequencer_api_port"] == PORTS["api"],
        readiness, "official dev-info", processes,
    )
    if dev_info["l1_url"].rstrip("/") != origin("anvil"):
        raise RuntimeError("official node reported an unexpected L1 endpoint")
    configuration["l1"]["light_client_address"] = dev_info["l1_light_client_address"]
    genesis, transactions = await_block(record, "first", 0, readiness, processes)
    identity = {
        "schema": "EC16-SOURCE-1",
        "official_commit": configuration["artifacts"]["official_commit"],
        "image_amd64_digest": configuration["artifacts"]["manifest_amd64"],
        "protocol_version": "0.3", "api_version": "v1", "namespace": NAMESPACE,
        "run_id": args.run_id, "genesis_block_hash": genesis["hash"],
    }
    # Persist the binding promptly so a failed run can retain a replayable
    # partial transcript once genesis has been observed.
    record.artifact("source-identity.json", json_bytes(identity))
    configuration["start_height_hint"] = start_height
    return observe_publications(record, processes, genesis, transactions, payloads,
                                configuration.get("input_identity"))


def observe_publications(record, processes, genesis, genesis_transactions, payloads=MARKERS, input_identity=None):
    initial = []
    for ordinal, payload in enumerate(payloads):
        response = submit(record, payload)
        initial.append(response)
        if input_identity is not None:
            input_identity["messages"][ordinal]["submit_record_id"] = response.record_id
    hashes = []
    for ordinal, response in enumerate(initial):
        row = input_identity["messages"][ordinal] if input_identity is not None else None
        tx_hash = submit_hash(response)
        if row is not None:
            row["ack_hash"] = tx_hash
        hashes.append(tx_hash)
    if input_identity is None and hashes[0] == hashes[1]:
        raise RuntimeError("distinct publication fixtures yielded the same ACK")
    repeat = submit(record, payloads[0])
    repeat_row = {"ordinal": 0, "submit_record_id": repeat.record_id, "status": repeat.status,
                  "ack_hash": None, "occurrences": [], "byte_equal": False}
    if input_identity is not None:
        input_identity["repeat"] = repeat_row
    if repeat.status == 200:
        repeat_row["ack_hash"] = submit_hash(repeat)
        if repeat_row["ack_hash"] != hashes[0]:
            raise RuntimeError("repeat submit acknowledged a different hash")

    wanted = {}
    for ordinal, pair in enumerate(zip(hashes, payloads)):
        wanted.setdefault(pair, []).append(ordinal)
    evidence = [[] for _ in payloads]
    if input_identity is not None:
        for row, positions in zip(input_identity["messages"], evidence):
            row["occurrences"] = positions
        if repeat_row["ack_hash"] is not None:
            repeat_row["occurrences"] = evidence[0]

    def collect(q, block, transactions):
        selected_index = 0
        for index, tx in enumerate(transactions):
            native = tx["transaction"]
            if native["namespace"] != NAMESPACE:
                continue
            data = base64.b64decode(native["payload"], validate=True)
            ordinals = wanted.get((tx["hash"], data), ())
            if ordinals:
                if (tx["index"] != index or tx["block_height"] != q or tx["block_hash"] != block["hash"]
                        or tx["namespace"] != NAMESPACE):
                    raise RuntimeError(f"official positional transaction mismatch at q={q} index={index}")
                occurrence = {"q": q, "source_transaction_index": index,
                              "pos_in_namespace": tx["pos_in_namespace"], "message_index": selected_index}
                for ordinal in ordinals:
                    evidence[ordinal].append(occurrence)
                    if input_identity is not None:
                        input_identity["messages"][ordinal]["byte_equal"] = True
                if repeat_row["ack_hash"] is not None and 0 in ordinals:
                    repeat_row["byte_equal"] = True
            selected_index += 1

    deadline = time.monotonic() + 65
    height_path = "/v1/node/block-height"
    tip = 0
    blocks = [genesis]
    collect(0, genesis, genesis_transactions)
    for q in range(1, MAX_CAPTURE_HEIGHT):
        if all(evidence) or time.monotonic() >= deadline:
            break
        wait_for(
            lambda: record.http("first", "GET", height_path),
            lambda value: type(value) is int and value > q,
            deadline, f"height hint beyond q={q}", processes,
        )
        block, transactions = await_block(record, "first", q, deadline, processes)
        blocks.append(block)
        tip = q
        collect(q, block, transactions)
    if not all(evidence):
        raise TimeoutError("publications not found in bounded complete source prefix")

    missing_position = f"{API}/transaction/0/{genesis['num_transactions']}/noproof"
    missing = record.http("first", "GET", missing_position)
    if missing.status != 404:
        raise RuntimeError(f"expected actual out-of-bounds transaction 404, got {missing.status}")

    deadline = time.monotonic() + 35
    for q in range(tip + 1):
        block, _ = await_block(record, "requery", q, deadline, processes)
        # Rust replays all semantic objects independently, including leaves and
        # per-position transactions, after the native node has been stopped.
        if block != blocks[q]:
            raise RuntimeError(f"second reader observed a different block at q={q}")

    return {
        "tip": tip, "distinct_submit_ids": [response.record_id for response in initial],
        "exact_repeat_id": repeat.record_id, "exact_repeat_status": repeat.status,
        "missing_record_id": missing.record_id, "missing_status": missing.status,
    }


def offline(args, observations, input_identity=None, payloads=MARKERS):
    binary = str(args.replay_binary)
    bundle = str(args.run / "bundle")
    first = str(args.run / "offline-first")
    repeated = str(args.run / "offline-first-delayed")
    requery = str(args.run / "offline-requery")
    compare = [binary, "compare", bundle, "first", "requery"]
    commands = [
        [binary, "replay", bundle, "first", first],
        [binary, "replay", bundle, "first", repeated, "--pause-ms", "37"],
        [binary, "replay", bundle, "requery", requery],
        compare,
    ]
    log_path = args.run / "offline-commands.json"
    with log_path.open("xb") as log:
        log.write(json_bytes(commands))
    for i, argv in enumerate(commands):
        result = subprocess.run(argv, check=False, capture_output=True, timeout=15)
        (args.run / f"offline-{i}.stdout").write_bytes(result.stdout)
        (args.run / f"offline-{i}.stderr").write_bytes(result.stderr)
        result.check_returncode()
    for filename in ("public-audit.bin", "public-projection.json"):
        expected = (Path(first) / filename).read_bytes()
        if any((Path(other) / filename).read_bytes() != expected for other in (repeated, requery)):
            raise RuntimeError(f"offline public projection mismatch for {filename}")
    receipts = json.loads((Path(first) / "receipts.json").read_bytes())
    by_id = {item["submit_record_id"]: item for item in receipts["publications"]}

    def verify_receipt(captured, row, payload):
        positions = [{key: occurrence[key] for key in ("q", "source_transaction_index")}
                     for occurrence in row["occurrences"]]
        if (not captured or bytes(captured["payload"]) != payload
                or captured["transaction_hash"] != row["ack_hash"] or captured["positions"] != positions):
            raise RuntimeError(f"offline replay changed manifest submit record {row['submit_record_id']}")

    positions = []
    for ordinal, record_id in enumerate(observations["distinct_submit_ids"]):
        captured = by_id.get(record_id)
        if not captured or not captured["positions"]:
            raise RuntimeError(f"offline replay could not confirm publication {record_id}")
        if input_identity is not None:
            verify_receipt(captured, input_identity["messages"][ordinal], payloads[ordinal])
        positions.append(captured["positions"])
    missing_id = observations["missing_record_id"]
    if missing_id not in receipts["unavailable_records"]:
        raise RuntimeError("actual out-of-bounds query did not remain unavailable in replay")
    repeat_id = observations["exact_repeat_id"]
    if input_identity is not None:
        if input_identity["repeat"]["ack_hash"] is not None:
            verify_receipt(by_id.get(repeat_id), input_identity["repeat"], payloads[0])
        observations["byte_equality"] = True
    observations.update({
        "public_sha256": receipts["public_sha256"], "distinct_publication_positions": positions,
        "repeat_publication_positions": by_id.get(repeat_id, {}).get("positions", []),
        "replayed_exactly": True, "requery_matches": True, "offline_pause_ms": [0, 37],
    })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", type=Path, required=True)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--replay-binary", type=Path, required=True)
    parser.add_argument("--payload-manifest", type=Path)
    args = parser.parse_args()
    if not re.fullmatch(r"[A-Za-z0-9_-]{1,64}", args.run_id):
        parser.error("invalid run ID")
    if not re.fullmatch(r"[a-f0-9]{40}", args.candidate):
        parser.error("invalid candidate SHA")
    loaded = None
    if args.payload_manifest is not None:
        try:
            loaded = load_payload_manifest(args.payload_manifest)
        except (OSError, ValueError, RecursionError) as exc:
            parser.error(f"invalid payload manifest: {exc}")

    started = time.monotonic_ns()
    (args.run / "bundle").mkdir()
    record = Recorder(args.run / "bundle")
    processes = Processes(args.run / "process")
    setup = json.loads((args.run / "image-verification.json").read_bytes())
    configuration = {
        "candidate_sha": args.candidate, "execution_method": "official-image binary native execution",
        "network": "isolated user/network/PID namespaces, loopback only",
        "seed": None, "protocol_version": "0.3", "api_version": "v1", "ports": PORTS,
        "publication_namespace": NAMESPACE, "commitment_trust": "trusted local official API",
        "artifacts": setup, "commands": {
            "wrapper": json.loads((args.run / "launch-command.json").read_bytes()),
        },
        "authentication": "unauthenticated HTTP restricted to task namespace loopback",
        "input_mode": "marker-default" if loaded is None else loaded[0]["evidence_kind"],
    }
    if loaded is not None:
        configuration["input_identity"] = copy_payload_inputs(record, loaded, args.candidate)
    input_identity = configuration.get("input_identity")
    payloads = MARKERS if loaded is None else loaded[2]
    record.artifact("image-verification.json", (args.run / "image-verification.json").read_bytes())
    record.artifact("build-command.txt", (args.run / "build-command.txt").read_bytes())
    for name in ("index.json", "manifest-amd64.json", "config.json"):
        record.artifact("official-" + name, (args.image / name).read_bytes())

    bootstrap = BootstrapAnvil(args.run, configuration)
    outcome = None
    error = None
    try:
        outcome = observe(record, args, processes, configuration, bootstrap, payloads)
    except BaseException as exc:
        error = exc
    finally:
        try:
            configuration["process_exit"] = processes.close()
        except (OSError, subprocess.SubprocessError) as cleanup_error:
            configuration["process_cleanup_error"] = str(cleanup_error)
            if error is None:
                error = cleanup_error
    try:
        try:
            bootstrap.finish(configuration.get("processes", {}).get("espresso"), record)
        except RuntimeError as cleanup_error:
            configuration["bootstrap_anvil"]["validation_error"] = str(cleanup_error)
            if error is None:
                error = cleanup_error
        for name in ("anvil", "espresso"):
            logfile = processes.dir / f"{name}.log"
            if logfile.exists():
                with logfile.open("rb") as stream:
                    record.artifact(f"{name}.log", stream.read(MAX_RESPONSE + 1))
        identity_path = record.root / "source-identity.json"
        if identity_path.exists():
            record.seal(json.loads(identity_path.read_bytes()), configuration)
        else:
            (args.run / "partial-requests.json").write_bytes(json_bytes(record.records))
            (args.run / "partial-configuration.json").write_bytes(json_bytes({
                "configuration": configuration, "artifacts": record.artifacts,
            }))
        if error is not None:
            raise error
        offline(args, outcome, input_identity, payloads)
    except BaseException as exc:
        (args.run / "result.json").write_bytes(json_bytes({
            "status": "INCOMPLETE", "capture_kind": "official", "error": str(exc),
            "candidate_sha": args.candidate, "run_id": args.run_id,
            **result_input_fields(configuration, started),
        }))
        raise
    (args.run / "result.json").write_bytes(json_bytes({
        "status": "PASS", "capture_kind": "official", "candidate_sha": args.candidate,
        "run_id": args.run_id, **outcome, **result_input_fields(configuration, started),
    }))
    print(f"recorded and replayed official dev-node evidence at {args.run}")


def result_input_fields(configuration, started):
    result = {
        "input_mode": configuration["input_mode"],
        "wall_capture_duration_ns": time.monotonic_ns() - started,
        "official_timing": {"proposed_at": "Unknown", "finalized_at": "Unknown", "live_at_proposal": "Unknown"},
        "pe_exports": "Unsupported", "commitment_trust": "trusted local official API; QC/VID unverified",
    }
    if "input_identity" in configuration:
        result["input_identity"] = configuration["input_identity"]
        result["publication_scope"] = "byte-publication-only"
    return result


if __name__ == "__main__":
    main()

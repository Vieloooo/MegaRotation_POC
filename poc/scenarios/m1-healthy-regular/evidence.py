"""Mechanical formats shared by the M1 launcher and offline checker."""

from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tomllib

CONTRACT = "poc/docs/contracts/m1-healthy-regular-harness-v1.md"
CONTRACT_SHA = "29e05ab86d156eab0473fd6a59a08cf2a5f962dfa3bf5c171d70dca941aeb647"
EXPECTATIONS = "poc/verification/m1-healthy-regular/source-expectations-v1.md"
EXPECTATIONS_SHA = "2abf64ce7000c272354490e70d6b28983a6e92ce84a8782684701735f34a13d3"
DISPATCH = "poc/docs/milestones/M1-implementation-dispatch-20261116.md"
DISPATCH_SHA = "d55d39f73593676325bdc0fbb7ba3845ca93025f"
DISPATCH_CONTENT_SHA = "4a211cd38a5db108dc3b8548a87c19e599cb3870e534e513b41e487b4c5cc8b6"
M0 = "35d382ac445cc8bc779f0d18fd23cb57e54680da"
BFT = "691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6"
RUST = "1.95.0"
COMMONWARE = "2026.10.0"
SCENARIOS = (
    "full-window", "session-end-preproduced", "session-end-late-slot",
    "official-same-bytes",
)
PORTS = (
    "h1.runtime", "h1.session", "h2.manager", "h3.bridge",
    "h3.official_artifact", "oracle.protocol",
)
GATES = (
    "BundleIntegrity", "ProcessAndCustody", "RegionalFinalAndDelivery",
    "PublicReplayAndLifecycle", "OfficialSameBytes",
)
STATUS = ("PASS", "FAIL", "NOT_RUN", "INCONCLUSIVE")
COMPONENTS = "h1 h2 h3"
FIXTURES = "config registry genesis dep workload l1 schedule"
MAX_BYTES = 1073741824
RUN_FIELDS = (
    "schema operation contract_sha256 input candidate revision_before revision_after "
    "worktree components source_files tool_versions commands ports scenario seed "
    "start_order wall_pause_ms limits status readiness reason gates artifacts cutoff pending"
)
LOCKS = ("poc/Cargo.lock", "poc/prototypes/i1b-runtime/Cargo.lock")
SOURCES = (
    CONTRACT, EXPECTATIONS, DISPATCH,
    "poc/scenarios/m1-healthy-regular/evidence.py",
    "poc/scenarios/m1-healthy-regular/run.py",
    "poc/verification/m1-healthy-regular/check.py",
)
PINS = {CONTRACT: CONTRACT_SHA, EXPECTATIONS: EXPECTATIONS_SHA, DISPATCH: DISPATCH_CONTENT_SHA}
GIT_OPTIONS = ("-c", "core.fsmonitor=false", "-c", "core.untrackedCache=false")
GIT_REVISION = ("rev-parse", "--show-toplevel", "HEAD")
GIT_BRANCH = ("branch", "--show-current")
GIT_STATUS = ("status", "--porcelain=v1", "--untracked-files=all", "--ignore-submodules=none")
GIT_TREE = ("ls-tree", "-r", "-z", "--full-tree")


class Invalid(ValueError):
    """Invalid input, candidate identity, or evidence."""


def require(condition, message):
    if not condition:
        raise Invalid(message)


def closed(value, fields, where):
    require(isinstance(value, dict), f"{where}: expected object")
    missing = set(fields.split()) - value.keys()
    unknown = value.keys() - set(fields.split())
    require(not (missing or unknown),
            f"{where}: missing={sorted(missing)}, unknown={sorted(unknown)}")


def array(value, where):
    require(isinstance(value, list), f"{where}: expected array")
    return value


def string(value, where):
    require(isinstance(value, str) and value and "\0" not in value
            and not any(0xD800 <= ord(c) <= 0xDFFF for c in value),
            f"{where}: expected nonempty text")


def integer(value, where):
    require(type(value) is int and value >= 0, f"{where}: expected nonnegative integer")


def hex_id(value, length, where):
    require(isinstance(value, str) and re.fullmatch(f"[0-9a-f]{{{length}}}", value),
            f"{where}: expected {length} lowercase hex characters")


def json_object(data, where):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, f"{where}: duplicate key {key!r}")
            result[key] = value
        return result

    def non_integer(number):
        raise Invalid(f"{where}: not an integer: {number}")

    try:
        return json.loads(data.decode("utf-8"), object_pairs_hook=unique,
                          parse_float=non_integer, parse_constant=non_integer)
    except Invalid:
        raise
    except (ValueError, RecursionError) as error:
        raise Invalid(f"{where}: {error}") from error


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=True) + "\n").encode()


def relative(name):
    string(name, "FileRef.path")
    require(not any(part in ("", ".", "..") for part in name.split("/"))
            and "\\" not in name and not re.match(r"^[a-zA-Z]:", name),
            f"invalid relative evidence path {name!r}")
    return PurePosixPath(name)


def file_ref(value):
    closed(value, "path sha256 bytes", "FileRef")
    relative(value["path"])
    hex_id(value["sha256"], 64, "FileRef.sha256")
    integer(value["bytes"], "FileRef.bytes")
    require(value["bytes"] <= MAX_BYTES, "FileRef exceeds maximum raw bytes")
    return value


@contextmanager
def directory(path):
    """Open a directory without following a symlink in any path component."""
    path = Path(path).absolute()
    require(".." not in path.parts, f"directory escapes through '..': {path}")
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    descriptor = os.open(path.anchor, flags)
    try:
        for part in path.parts[1:]:
            child = os.open(part, flags, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = child
        yield descriptor
    finally:
        os.close(descriptor)


@contextmanager
def regular(path):
    with directory(path.parent) as parent:
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
                             dir_fd=parent)
        with os.fdopen(descriptor, "rb") as source:
            info = os.fstat(source.fileno())
            require(stat.S_ISREG(info.st_mode), f"{path}: not a regular file")
            yield source, info.st_size


def fresh_destination(path):
    path = path.absolute()
    with directory(path.parent):
        pass
    require(not os.path.lexists(path), f"output already exists: {path}")
    return path


class Inventory:
    """Read a bounded inventory rooted in a single directory."""

    def __init__(self, root):
        self.root = root.absolute()
        self.refs = {}
        self.size = 0

    def inspect(self, name, expected=None, *, read=False, copy=None):
        path = relative(name)
        prior = self.refs.get(name)
        if expected is not None:
            file_ref(expected)
        if not read and copy is None and prior is not None:
            require(expected is None or prior == expected, f"{name}: conflicting FileRefs")
            return prior, None
        parts = [] if read else None
        with regular(self.root / path) as (source, size):
            require(size <= MAX_BYTES - self.size + (prior["bytes"] if prior else 0),
                    f"{name}: raw byte budget exceeded")
            require(expected is None or size == expected["bytes"], f"{name}: size mismatch")
            digest = hashlib.sha256()
            remaining = size
            while remaining:
                chunk = source.read(min(remaining, 1024 * 1024))
                require(chunk, f"{name}: truncated during read")
                remaining -= len(chunk)
                digest.update(chunk)
                if parts is not None:
                    parts.append(chunk)
                if copy is not None:
                    copy.write(chunk)
            require(not source.read(1), f"{name}: grew during read")
        actual = {"path": name, "sha256": digest.hexdigest(), "bytes": size}
        require(expected is None or actual == expected, f"{name}: SHA-256 mismatch")
        require(prior is None or actual == prior, f"{name}: changed during read")
        if prior is None:
            self.refs[name] = actual
            self.size += size
        return actual, b"".join(parts) if read else None

    def ref(self, value, base=PurePosixPath(), *, read=False):
        value = file_ref(value)
        name = (base / value["path"]).as_posix()
        _, data = self.inspect(name, {**value, "path": name}, read=read)
        return data

    def document(self, name):
        actual, data = self.inspect(name, read=True)
        return actual, json_object(data, name)


def exactly_ids(values, expected, where):
    require(len(values) == len(expected) and all(isinstance(v, str) for v in values)
            and set(values) == set(expected), f"{where}: expected exactly {list(expected)}")


def required_gates(scenario):
    return GATES if scenario == SCENARIOS[3] else GATES[:-1]


def missing_ports(scenario):
    # No producer adapter or independent oracle is implemented in this revision.
    return list(PORTS if scenario == SCENARIOS[3] else PORTS[:4] + PORTS[5:])


def pinned_ref(store, value, expected_hash, base):
    file_ref(value)
    require(value["sha256"] == expected_hash, f"{value['path']}: frozen SHA-256 mismatch")
    store.ref(value, base)


def expectations(store, name, scenario=None):
    actual, value = store.document(name)
    base = relative(name).parent
    closed(value, "schema contract source_expectations oracle_binding required_gates",
           "expectations")
    require(value["schema"] == "M1-EXPECTATIONS-1", "unsupported expectations schema")
    pinned_ref(store, value["contract"], CONTRACT_SHA, base)
    pinned_ref(store, value["source_expectations"], EXPECTATIONS_SHA, base)
    if value["oracle_binding"] is not None:
        store.ref(value["oracle_binding"], base)
    gates = array(value["required_gates"], "required_gates")
    expected = required_gates(scenario) if scenario is not None else GATES[:len(gates)]
    require(scenario is not None or len(gates) in (4, 5), "required_gates: expected 4 or 5 gates")
    exactly_ids(gates, expected, "required_gates")
    return actual, value


def components(store, values, base):
    closed(values, COMPONENTS, "components")
    for key, value in values.items():
        if value is not None:
            closed(value, "owner delivery_sha review contract", f"components.{key}")
            string(value["owner"], f"components.{key}.owner")
            hex_id(value["delivery_sha"], 40, f"components.{key}.delivery_sha")
            store.ref(value["review"], base)
            store.ref(value["contract"], base)


def ports(store, values, base, *, input_only):
    ids = []
    for port in array(values, "ports"):
        keys = "port_id owner binding" + ("" if input_only else " status")
        closed(port, keys, "ports[]")
        string(port["port_id"], "port_id")
        string(port["owner"], "port owner")
        if port["binding"] is not None:
            store.ref(port["binding"], base)
        if not input_only:
            require(port["status"] in ("Ready", "NotReady"), "invalid port status")
        ids.append(port["port_id"])
    exactly_ids(ids, PORTS, "ports")


def scenario_fields(value):
    require(value["scenario"] in SCENARIOS, "invalid scenario")
    integer(value["seed"], "seed")
    integer(value["wall_pause_ms"], "wall_pause_ms")
    require(value["seed"] in (7, 42, 99), "invalid seed")
    require(value["wall_pause_ms"] in (0, 2), "invalid wall_pause_ms")
    require(value["start_order"] in ("forward", "reverse"), "invalid start_order")
    if value["scenario"] == SCENARIOS[2]:
        require((value["seed"], value["start_order"], value["wall_pause_ms"]) == (7, "forward", 0),
                "late-slot diagnostic must use seed 7, forward, pause 0")


def input_limits(value):
    closed(value, "max_virtual_tick_us max_events wall_timeout_s max_raw_bytes", "limits")
    for key, expected in (("max_events", 1000000), ("wall_timeout_s", 1800),
                          ("max_raw_bytes", MAX_BYTES)):
        integer(value[key], f"limits.{key}")
        require(value[key] == expected, f"limits.{key}: expected {expected}")
    if value["max_virtual_tick_us"] is not None:
        integer(value["max_virtual_tick_us"], "limits.max_virtual_tick_us")


def source_run(store, value, base):
    closed(value, "manifest checker_report public_messages regional_results", "source_run")
    manifest = json_object(store.ref(value["manifest"], base, read=True), "source_run.manifest")
    report = json_object(store.ref(value["checker_report"], base, read=True),
                         "source_run.checker_report")
    closed(manifest, RUN_FIELDS, "source_run.manifest")
    require(manifest["schema"] == "M1-RUN-1" and manifest["operation"] == "run"
            and manifest["scenario"] == SCENARIOS[0], "source_run requires a full-window run")
    scenario_fields(manifest)
    require((manifest["seed"], manifest["start_order"], manifest["wall_pause_ms"])
            == (7, "forward", 0), "source_run requires seed 7, forward, pause 0")
    closed(report, "schema operation bundles expected checker commands gates status reason",
           "source_run.checker_report")
    require(report["schema"] == "M1-CHECK-1" and report["operation"] == "check",
            "source_run requires a source checker report")
    require(any(file_ref(ref)["sha256"] == value["manifest"]["sha256"]
                and ref["bytes"] == value["manifest"]["bytes"]
                for ref in array(report["bundles"], "source_run.checker_report.bundles")),
            "source checker report does not reference the source manifest")

    artifacts = artifact_refs(manifest["artifacts"])
    source_base = base / relative(value["manifest"]["path"]).parent
    indexed = {(source_base / ref["path"]).as_posix(): ref for ref in artifacts}
    for field in ("public_messages", "regional_results"):
        seen = set()
        refs = array(value[field], f"source_run.{field}")
        require(refs, f"source_run.{field}: empty inventory")
        for ref in refs:
            file_ref(ref)
            name = (base / ref["path"]).as_posix()
            indexed_ref = indexed.get(name)
            require(indexed_ref is not None and same_bytes(ref, indexed_ref),
                    f"{name}: not indexed by the original source run")
            require(name not in seen, f"{name}: duplicate source_run.{field} entry")
            seen.add(name)
            store.ref(ref, base)


def same_bytes(left, right):
    return left["sha256"] == right["sha256"] and left["bytes"] == right["bytes"]


def input_document(store, name):
    actual, value = store.document(name)
    base = relative(name).parent
    closed(value, "schema contract candidate m0 dispatch components scenario seed start_order "
           "wall_pause_ms fixtures profile ports expectations source_run limits", "input")
    require(value["schema"] == "M1-INPUT-1", "unsupported input schema")
    pinned_ref(store, value["contract"], CONTRACT_SHA, base)
    hex_id(value["candidate"], 40, "candidate")
    require(value["m0"] == M0 and value["dispatch"] == DISPATCH_SHA, "incorrect M0 or dispatch pin")
    scenario_fields(value)
    components(store, value["components"], base)
    ports(store, value["ports"], base, input_only=True)
    closed(value["fixtures"], FIXTURES, "fixtures")
    for ref in list(value["fixtures"].values()) + [value["profile"]]:
        if ref is not None:
            store.ref(ref, base)
    if value["expectations"] is not None:
        ref = file_ref(value["expectations"])
        actual_expectations, _ = expectations(store, (base / ref["path"]).as_posix(),
                                             value["scenario"])
        require(same_bytes(actual_expectations, ref), "input.expectations: FileRef mismatch")
    if value["source_run"] is not None:
        require(value["scenario"] == SCENARIOS[3], "source_run is only valid for H09")
        source_run(store, value["source_run"], base)
    input_limits(value["limits"])
    if value["fixtures"]["config"] is None or value["profile"] is None:
        require(value["limits"]["max_virtual_tick_us"] is None, "virtual cutoff needs config/profile")
    return actual, value


def missing_inputs(value):
    missing = [f"components.{key}" for key in COMPONENTS.split()
               if value["components"][key] is None]
    missing += [f"fixtures.{key}" for key in FIXTURES.split() if value["fixtures"][key] is None]
    missing += [key for key in ("profile", "expectations") if value[key] is None]
    if value["limits"]["max_virtual_tick_us"] is None:
        missing.append("limits.max_virtual_tick_us")
    if value["scenario"] == SCENARIOS[3] and value["source_run"] is None:
        missing.append("source_run")
    return missing


def artifact_refs(values):
    refs = []
    for artifact in array(values, "artifacts"):
        closed(artifact, "role producer schema path sha256 bytes", "artifact")
        for field in ("role", "producer", "schema"):
            string(artifact[field], f"artifact.{field}")
        refs.append(file_ref({key: artifact[key] for key in ("path", "sha256", "bytes")}))
    require(len(refs) == len({ref["path"] for ref in refs}), "duplicate artifact index")
    return refs


def dependency_versions(data, path):
    try:
        packages = tomllib.loads(data.decode("utf-8"))["package"]
        return [package["version"] for package in packages
                if package["name"].startswith("commonware-")]
    except (ValueError, KeyError, TypeError, AttributeError) as error:
        raise Invalid(f"{path}: malformed Cargo.lock: {error}") from error


def candidate_sources(tree):
    sources = {}
    bft_seen = False
    for row in tree.split(b"\0"):
        if not row:
            continue
        try:
            prefix, raw_path = row.split(b"\t", 1)
            mode, kind, object_id = prefix.decode("ascii").split()
        except (ValueError, UnicodeError) as error:
            raise Invalid("invalid candidate tree record") from error
        path = os.fsdecode(raw_path)
        if path == "regional_BFT_sim":
            require(not bft_seen and (mode, kind, object_id) == ("160000", "commit", BFT),
                    "wrong Regional BFT gitlink")
            bft_seen = True
        elif path.startswith("poc/") and (path in SOURCES
                or PurePosixPath(path).name in ("Cargo.toml", "Cargo.lock")
                or path.endswith(".proto")):
            relative(path)
            require(kind == "blob" and mode in ("100644", "100755") and path not in sources,
                    f"{path}: expected unique tracked regular source")
            hex_id(object_id, 40, "candidate tree blob")
            sources[path] = object_id
    require(bft_seen, "missing Regional BFT gitlink")
    require(set(SOURCES + LOCKS) <= sources.keys(), "missing required candidate source")
    return sources


def check_source(path, data, sha256, object_id):
    blob = hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()
    require(object_id == blob, f"{path}: differs from candidate blob")
    if path.endswith("/Cargo.lock"):
        versions = dependency_versions(data, path)
        if versions or path in LOCKS:
            require(versions and all(version == COMMONWARE for version in versions),
                    f"{path}: incorrect Commonware pin")
    require(path not in PINS or sha256 == PINS[path], f"{path}: frozen SHA-256 mismatch")


def check_git_index(tracked):
    require(tracked and all(line[0:1].isupper() and not line.startswith(b"S ")
                            for line in tracked.split(b"\0") if line),
            "candidate index has assume-unchanged or skip-worktree entries")

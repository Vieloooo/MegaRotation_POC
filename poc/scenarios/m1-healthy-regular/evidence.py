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
GIT_ROOT_TREE = ("ls-tree", "-z", "--full-tree")
GIT_POC_TREE = ("ls-tree", "-r", "-t", "-z", "--full-tree")


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


def unique_paths(refs, where):
    require(len(refs) == len({ref["path"] for ref in refs}), f"{where}: duplicate path")


def file_refs(values, where):
    refs = [file_ref(value) for value in array(values, where)]
    unique_paths(refs, where)
    return refs


def absolute_path(value, where):
    string(value, where)
    path = Path(value)
    require(path.is_absolute() and ".." not in path.parts, f"{where}: expected absolute path")


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


def component_refs(values):
    closed(values, COMPONENTS, "components")
    refs = []
    for key, value in values.items():
        if value is not None:
            closed(value, "owner delivery_sha review contract", f"components.{key}")
            string(value["owner"], f"components.{key}.owner")
            hex_id(value["delivery_sha"], 40, f"components.{key}.delivery_sha")
            refs.extend(file_ref(value[field]) for field in ("review", "contract"))
    return refs


def port_refs(values, *, input_only):
    ids = []
    refs = []
    for port in array(values, "ports"):
        keys = "port_id owner binding" + ("" if input_only else " status")
        closed(port, keys, "ports[]")
        string(port["port_id"], "port_id")
        string(port["owner"], "port owner")
        if port["binding"] is not None:
            refs.append(file_ref(port["binding"]))
        if not input_only:
            require(port["status"] in ("Ready", "NotReady"), "invalid port status")
        ids.append(port["port_id"])
    exactly_ids(ids, PORTS, "ports")
    return refs


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


def check_reason(reason):
    require(isinstance(reason, dict), "reason: expected object")
    string(reason.get("code"), "reason.code")
    if reason["code"] == "NotReady":
        ports = array(reason.get("missing_ports"), "reason.missing_ports")
        inputs = array(reason.get("missing_inputs"), "reason.missing_inputs")
        require(ports or inputs, "NotReady requires a missing capability or input")
        require(all(port in PORTS for port in ports), "unknown missing port")
        for field in inputs:
            string(field, "missing input")


def gate_refs(values, scenario):
    seen = set()
    refs = []
    for gate in array(values, "gates"):
        closed(gate, "id status reason evidence_refs", "gate")
        ident = gate["id"]
        require(ident in required_gates(scenario) and ident not in seen, "unknown or duplicate gate")
        seen.add(ident)
        require(gate["status"] in STATUS, "invalid gate status")
        check_reason(gate["reason"])
        refs.extend(file_ref(ref) for ref in array(gate["evidence_refs"], "gate.evidence_refs"))
    return refs


def validate_commands(values):
    refs = []
    fields = ("argv cwd binary_sha256 started_wall_ns elapsed_wall_ns exit_code "
              "timeout lock_owner stdout stderr")
    for item in array(values, "commands"):
        closed(item, fields, "command")
        argv = array(item["argv"], "command.argv")
        require(argv, "empty command argv")
        for arg in argv:
            string(arg, "command argument")
        absolute_path(argv[0], "command executable")
        absolute_path(item["cwd"], "command.cwd")
        hex_id(item["binary_sha256"], 64, "command binary SHA-256")
        for field in ("started_wall_ns", "elapsed_wall_ns"):
            integer(item[field], f"command.{field}")
        require(item["exit_code"] is None or type(item["exit_code"]) is int,
                "invalid command exit code")
        require(type(item["timeout"]) is bool, "invalid command timeout flag")
        if item["lock_owner"] is not None:
            string(item["lock_owner"], "command.lock_owner")
        refs.extend(file_ref(item[stream]) for stream in ("stdout", "stderr"))
    unique_paths(refs, "command streams")


def tool_version_fields(tools):
    closed(tools, "git python rustc cargo commonware bft_gitlink", "tool_versions")
    for name in ("git", "python", "rustc", "cargo"):
        tool = tools[name]
        if tool is not None:
            closed(tool, "path sha256 version", f"tool_versions.{name}")
            absolute_path(tool["path"], f"{name} path")
            hex_id(tool["sha256"], 64, f"{name} binary")
            string(tool["version"], f"{name} version")
    if tools["commonware"] is not None:
        string(tools["commonware"], "tool_versions.commonware")
    if tools["bft_gitlink"] is not None:
        hex_id(tools["bft_gitlink"], 40, "tool_versions.bft_gitlink")


def run_fields(value):
    closed(value, RUN_FIELDS, "manifest")
    require(value["schema"] == "M1-RUN-1", "unsupported run schema")
    require(value["operation"] in ("preflight", "run"), "invalid run operation")
    require(value["status"] in STATUS, "invalid run status")
    valid_readiness = ("Ready", "NotReady", "Reject") if value["operation"] == "preflight" else (None,)
    require(value["readiness"] in valid_readiness, "invalid run readiness")
    check_reason(value["reason"])
    absolute_path(value["worktree"], "worktree")
    hex_id(value["candidate"], 40, "manifest candidate")
    require(value["contract_sha256"] == CONTRACT_SHA, "wrong manifest contract pin")
    scenario_fields(value)
    input_limits(value["limits"])
    for field in ("revision_before", "revision_after"):
        revision = value[field]
        if revision is not None:
            closed(revision, "head branch status", field)
            hex_id(revision["head"], 40, f"{field}.head")
            if revision["branch"] != "":
                string(revision["branch"], f"{field}.branch")
            for line in array(revision["status"], f"{field}.status"):
                string(line, f"{field}.status line")
    cutoff = value["cutoff"]
    closed(cutoff, "virtual_tick_us event_id reader_pc reason", "cutoff")
    closed(cutoff["reader_pc"], "a b", "cutoff.reader_pc")
    string(cutoff["reason"], "cutoff.reason")
    for unit in ("virtual_tick_us", "event_id"):
        if cutoff[unit] is not None:
            integer(cutoff[unit], f"cutoff.{unit}")
    require(value["pending"] is None or isinstance(value["pending"], (dict, list)), "invalid pending evidence")
    file_ref(value["input"])
    component_refs(value["components"])
    port_refs(value["ports"], input_only=False)
    file_refs(value["source_files"], "source_files")
    tool_version_fields(value["tool_versions"])
    validate_commands(value["commands"])
    gate_refs(value["gates"], value["scenario"])
    return artifact_refs(value["artifacts"])


def check_report_fields(value, scenario):
    closed(value, "schema operation bundles expected checker commands gates status reason", "checker report")
    require(value["schema"] == "M1-CHECK-1", "unsupported checker report schema")
    require(value["operation"] in ("check", "compare"), "invalid checker operation")
    require(value["status"] in STATUS, "invalid checker status")
    check_reason(value["reason"])
    file_ref(value["expected"])
    bundles = array(value["bundles"], "checker bundles")
    count = 1 if value["operation"] == "check" else 2
    require(len(bundles) == count, "incorrect checker bundle count")
    for ref in bundles:
        file_ref(ref)
    checker = value["checker"]
    closed(checker, "scope source_root source_files bundle_roots expected_root", "checker")
    string(checker["scope"], "checker scope")
    absolute_path(checker["source_root"], "checker source root")
    absolute_path(checker["expected_root"], "checker expected root")
    roots = array(checker["bundle_roots"], "checker bundle roots")
    require(len(roots) == count, "incorrect checker bundle root count")
    for root in roots:
        absolute_path(root, "checker bundle root")
    file_refs(checker["source_files"], "checker source files")
    validate_commands(value["commands"])
    gate_refs(value["gates"], scenario)


def source_run(store, value, base):
    closed(value, "manifest checker_report public_messages regional_results", "source_run")
    manifest = json_object(store.ref(value["manifest"], base, read=True), "source_run.manifest")
    report = json_object(store.ref(value["checker_report"], base, read=True),
                         "source_run.checker_report")
    artifacts = run_fields(manifest)
    require(manifest["operation"] == "run" and manifest["scenario"] == SCENARIOS[0],
            "source_run requires a full-window run")
    require((manifest["seed"], manifest["start_order"], manifest["wall_pause_ms"])
            == (7, "forward", 0), "source_run requires seed 7, forward, pause 0")
    check_report_fields(report, manifest["scenario"])
    require(report["operation"] == "check", "source_run requires a source checker report")
    source_ref, = report["bundles"]
    require(source_ref["path"] == relative(value["manifest"]["path"]).name
            and same_bytes(source_ref, value["manifest"]),
            "source checker report does not reference the source manifest")

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
    for ref in component_refs(value["components"]) + port_refs(value["ports"], input_only=True):
        store.ref(ref, base)
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


def git_object_id(kind, data):
    digest = hashlib.sha1(f"{kind} {len(data)}\0".encode("ascii"))
    digest.update(data)
    return digest.hexdigest()


def tree_entries(data):
    modes = {b"040000": b"tree", b"100644": b"blob", b"100755": b"blob",
             b"120000": b"blob", b"160000": b"commit"}
    require(data.endswith(b"\0"), "invalid candidate tree record terminator")
    entries = {}
    for row in data[:-1].split(b"\0"):
        try:
            prefix, path = row.split(b"\t", 1)
            mode, kind, raw_id = prefix.split(b" ")
            object_id = raw_id.decode("ascii")
        except (ValueError, UnicodeError) as error:
            raise Invalid("invalid candidate tree record") from error
        require(mode in modes and modes[mode] == kind, "invalid candidate tree entry mode/type")
        require(path not in entries and all(part not in (b"", b".", b"..") for part in path.split(b"/")),
                "invalid or duplicate candidate tree path")
        hex_id(object_id, 40, "candidate tree entry")
        entries[path] = (mode.lstrip(b"0"), object_id)
    return entries


def verify_tree(entries, object_id, path):
    # Git sorts names as bytes and compares a directory as its name plus '/'.
    ordered = sorted(entries.items(), key=lambda item: item[0] + (b"/" if item[1][0] == b"40000" else b""))
    body = b"".join(mode + b" " + name + b"\0" + bytes.fromhex(target)
                    for name, (mode, target) in ordered)
    require(git_object_id("tree", body) == object_id, f"Git tree object ID mismatch: {path}")


def candidate_sources(candidate, commit, root_tree, poc_tree):
    require(git_object_id("commit", commit) == candidate, "Git commit object ID mismatch")
    header = re.fullmatch(rb"tree ([0-9a-f]{40})", commit.partition(b"\n")[0])
    require(header is not None, "Git commit is missing its root tree ID")
    root = tree_entries(root_tree)
    require(all(b"/" not in name for name in root), "root tree lists a non-root path")
    verify_tree(root, header[1].decode("ascii"), "/")
    require(root.get(b"regional_BFT_sim") == (b"160000", BFT), "wrong Regional BFT gitlink")
    poc = tree_entries(poc_tree)
    require(b"poc" in root and root[b"poc"][0] == b"40000" and poc.get(b"poc") == root[b"poc"],
            "poc tree ID is not bound to the candidate root")
    directories = {path: {} for path, (mode, _) in poc.items() if mode == b"40000"}
    for path, entry in poc.items():
        if path == b"poc":
            continue
        parent, _, name = path.rpartition(b"/")
        require(path.startswith(b"poc/") and parent in directories,
                "poc tree entry is missing its parent directory")
        directories[parent][name] = entry
    for directory_path, entries in directories.items():
        verify_tree(entries, poc[directory_path][1], os.fsdecode(directory_path))

    sources = {}
    for raw_path, (mode, object_id) in poc.items():
        path = os.fsdecode(raw_path)
        if path in SOURCES or PurePosixPath(path).name in ("Cargo.toml", "Cargo.lock") or path.endswith(".proto"):
            relative(path)
            require(mode in (b"100644", b"100755"), f"{path}: expected tracked regular source")
            sources[path] = object_id
    require(set(SOURCES + LOCKS) <= sources.keys(), "missing required candidate source")
    return sources


def check_source(path, data, sha256, object_id):
    require(object_id == git_object_id("blob", data), f"{path}: differs from candidate blob")
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

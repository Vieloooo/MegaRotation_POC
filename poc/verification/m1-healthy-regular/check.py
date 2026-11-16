#!/usr/bin/env python3
"""Check M1 evidence integrity offline. Protocol oracles await integration."""

import argparse
import hashlib
import os
from pathlib import Path, PurePosixPath
import stat
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "poc/scenarios/m1-healthy-regular"))

import evidence as ev


def check_reason(reason):
    ev.require(isinstance(reason, dict), "reason: expected object")
    ev.string(reason.get("code"), "reason.code")
    if reason["code"] == "NotReady":
        ports = ev.array(reason.get("missing_ports"), "reason.missing_ports")
        inputs = ev.array(reason.get("missing_inputs"), "reason.missing_inputs")
        ev.require(ports or inputs, "NotReady requires a missing capability or input")
        ev.require(all(port in ev.PORTS for port in ports), "unknown missing port")
        for field in inputs:
            ev.string(field, "missing input")


def validate_gates(values, scenario, store):
    seen = set()
    for gate in ev.array(values, "gates"):
        ev.closed(gate, "id status reason evidence_refs", "gate")
        ident = gate["id"]
        ev.require(ident in ev.required_gates(scenario) and ident not in seen,
                   "unknown or duplicate gate")
        seen.add(ident)
        ev.require(gate["status"] in ev.STATUS, "invalid gate status")
        check_reason(gate["reason"])
        for ref in ev.array(gate["evidence_refs"], "gate.evidence_refs"):
            ev.require(ev.file_ref(ref)["path"] in store.refs, "unindexed gate evidence")
            store.ref(ref)


def checked_index(store, refs, label):
    paths = set()
    for ref in ev.array(refs, label):
        path = ev.file_ref(ref)["path"]
        ev.require(path not in paths, f"{label}: duplicate path {path}")
        paths.add(path)
        store.ref(ref)
    return paths


def command_records(manifest, store, indexed):
    streams = set()
    fields = ("argv cwd binary_sha256 started_wall_ns elapsed_wall_ns exit_code "
              "timeout lock_owner stdout stderr")
    for item in ev.array(manifest["commands"], "commands"):
        ev.closed(item, fields, "command")
        argv = ev.array(item["argv"], "command.argv")
        ev.require(argv, "empty command argv")
        for arg in argv:
            ev.string(arg, "command argument")
        ev.require(Path(argv[0]).is_absolute(), "command executable must be absolute")
        ev.string(item["cwd"], "command.cwd")
        ev.require(Path(item["cwd"]).is_absolute(), "command cwd must be absolute")
        ev.hex_id(item["binary_sha256"], 64, "command binary SHA-256")
        for field in ("started_wall_ns", "elapsed_wall_ns"):
            ev.integer(item[field], f"command.{field}")
        ev.require(item["exit_code"] is None or type(item["exit_code"]) is int,
                   "invalid command exit code")
        ev.require(type(item["timeout"]) is bool, "invalid command timeout flag")
        if item["lock_owner"] is not None:
            ev.string(item["lock_owner"], "command.lock_owner")
        for stream in ("stdout", "stderr"):
            ref = ev.file_ref(item[stream])
            ev.require(ref["path"] in indexed and ref["path"] not in streams,
                       f"{ref['path']}: missing or reused command stream index")
            ev.require(indexed[ref["path"]]["role"] == f"command_{stream}",
                       f"{ref['path']}: wrong command stream role")
            store.ref(ref)
            streams.add(ref["path"])


def check_provenance(manifest, request, store, sources):
    tools = manifest["tool_versions"]
    ev.closed(tools, "git python rustc cargo commonware bft_gitlink", "tool_versions")
    if tools["git"] is None:
        ev.require(manifest["revision_before"] is None and manifest["revision_after"] is None
                   and not manifest["commands"] and not any(path.startswith("source/") for path in sources)
                   and all(tools[name] is None for name in ("rustc", "cargo", "commonware", "bft_gitlink")),
                   "partial preparation has inconsistent provenance")
        return ["tool_versions.git"]
    ev.require(tools["commonware"] == ev.COMMONWARE and tools["bft_gitlink"] == ev.BFT,
               "incorrect Commonware or Regional BFT pin")

    def probe(name, args, count=1):
        tool = tools[name]
        argv = [tool["path"], *args]
        matches = [item for item in manifest["commands"] if item["argv"] == argv]
        ev.require(len(matches) == count, f"{name}: missing or duplicate probe {args!r}")
        results = []
        for item in matches:
            ev.require(item["exit_code"] == 0 and not item["timeout"]
                       and item["cwd"] == manifest["worktree"]
                       and item["binary_sha256"] == tool["sha256"],
                       f"{name}: failed or conflicting probe {args!r}")
            results.append(store.ref(item["stdout"], read=True))
        return results

    missing = []
    for name in ("git", "python", "rustc", "cargo"):
        tool = tools[name]
        if tool is None and name in ("rustc", "cargo"):
            missing.append(f"tool_versions.{name}")
            continue
        ev.closed(tool, "path sha256 version", f"tool_versions.{name}")
        ev.string(tool["path"], f"{name} path")
        ev.require(Path(tool["path"]).is_absolute(), f"{name}: tool path must be absolute")
        ev.hex_id(tool["sha256"], 64, f"{name} binary")
        ev.string(tool["version"], f"{name} version")
        if name != "python":
            version, = probe(name, ("--version",))
            ev.require(version.decode().strip() == tool["version"], f"{name}: version evidence mismatch")
        if name in ("rustc", "cargo"):
            ev.require(tool["version"].split()[:2] == [name, ev.RUST], f"{name}: wrong version")

    before, after = manifest["revision_before"], manifest["revision_after"]
    ev.require(before == after, "revision changed during preflight")
    ev.closed(before, "head branch status", "revision")
    ev.require(before["head"] == manifest["candidate"] and before["status"] == []
               and isinstance(before["branch"], str), "candidate is stale or dirty")
    for args, output in (
        (ev.GIT_REVISION, [manifest["worktree"], manifest["candidate"]]),
        (ev.GIT_BRANCH, [before["branch"]] if before["branch"] else []),
        (ev.GIT_STATUS, []),
    ):
        for data in probe("git", (*ev.GIT_OPTIONS, *args), count=2):
            ev.require(data.decode().splitlines() == output, f"revision evidence mismatch: {args!r}")

    candidate = manifest["candidate"]
    ancestors = {ev.M0} | {value["delivery_sha"] for value in request["components"].values()
                           if value is not None}
    for ancestor in sorted(ancestors):
        kind, = probe("git", (*ev.GIT_OPTIONS, "cat-file", "-t", ancestor))
        ev.require(kind == b"commit\n", f"{ancestor}: expected a commit")
        probe("git", (*ev.GIT_OPTIONS, "merge-base", "--is-ancestor", ancestor, candidate))
    dispatch, = probe("git", (*ev.GIT_OPTIONS, "show", f"{ev.DISPATCH_SHA}:{ev.DISPATCH}"))
    ev.require(hashlib.sha256(dispatch).hexdigest() == ev.DISPATCH_CONTENT_SHA,
               "incorrect dispatch source bytes")
    index, = probe("git", (*ev.GIT_OPTIONS, "ls-files", "-v", "-z"))
    ev.check_git_index(index)
    tree, = probe("git", (*ev.GIT_OPTIONS, *ev.GIT_TREE, candidate, "--", "poc", "regional_BFT_sim"))
    candidate_paths = ev.candidate_sources(tree)
    ev.require({path for path in sources if path.startswith("source/")}
               == {f"source/{path}" for path in candidate_paths}, "candidate source inventory mismatch")
    for path, object_id in candidate_paths.items():
        ref, data = store.inspect(f"source/{path}", read=True)
        ev.check_source(path, data, ref["sha256"], object_id)
    return missing


def bundle_inventory(store):
    def inaccessible(error):
        raise error

    files = set()
    for folder, dirs, names in os.walk(store.root, followlinks=False, onerror=inaccessible):
        for name in dirs + names:
            path = Path(folder) / name
            mode = path.lstat().st_mode
            if name in names:
                ev.require(stat.S_ISREG(mode), f"{path}: not a regular file")
                files.add(path.relative_to(store.root).as_posix())
            else:
                ev.require(stat.S_ISDIR(mode), f"{path}: not a regular directory")
    ev.require(files == store.refs.keys(), "missing or unindexed bundle files")


def bundle(root, expected_ref, expected, reports):
    store = ev.Inventory(root)
    manifest_ref, data = store.inspect("manifest.json", read=True)
    reports.append(manifest_ref)
    _, seal = store.inspect("manifest.sha256", read=True)
    ev.require(seal == f"{manifest_ref['sha256']}  manifest.json\n".encode(), "invalid manifest seal")
    value = ev.json_object(data, "manifest.json")
    ev.closed(value, ev.RUN_FIELDS, "manifest")
    ev.require(value["schema"] == "M1-RUN-1", "unsupported run schema")
    ev.require(value["operation"] in ("preflight", "run"), "invalid run operation")
    ev.require(value["status"] in ev.STATUS, "invalid run status")
    valid_readiness = ("Ready", "NotReady", "Reject") if value["operation"] == "preflight" else (None,)
    ev.require(value["readiness"] in valid_readiness, "invalid run readiness")
    check_reason(value["reason"])
    ev.string(value["worktree"], "worktree")
    ev.require(Path(value["worktree"]).is_absolute(), "worktree must be absolute")
    ev.require(value["contract_sha256"] == ev.CONTRACT_SHA, "wrong manifest contract pin")
    ev.scenario_fields(value)
    ev.input_limits(value["limits"])
    ev.closed(value["cutoff"], "virtual_tick_us event_id reader_pc reason", "cutoff")
    ev.closed(value["cutoff"]["reader_pc"], "a b", "cutoff.reader_pc")
    ev.string(value["cutoff"]["reason"], "cutoff.reason")
    for unit in ("virtual_tick_us", "event_id"):
        if value["cutoff"][unit] is not None:
            ev.integer(value["cutoff"][unit], f"cutoff.{unit}")
    ev.require(value["pending"] is None or isinstance(value["pending"], (dict, list)),
               "invalid pending evidence")

    ref = ev.file_ref(value["input"])
    path = PurePosixPath(ref["path"])
    ev.require(len(path.parts) == 2 and path.parts[0] == "input", "input must be under input/")
    actual, request = ev.input_document(store, ref["path"])
    ev.require(actual == ref, "manifest input FileRef mismatch")
    input_paths = {name for name in store.refs if name.startswith("input/")}
    for key in ("candidate", "components", "scenario", "seed", "start_order", "wall_pause_ms", "limits"):
        ev.require(value[key] == request[key], f"manifest {key}: differs from input")
    ev.ports(store, value["ports"], path.parent, input_only=False)
    port_inputs = {item["port_id"]: item for item in request["ports"]}
    for port in value["ports"]:
        ev.require({key: port[key] for key in ("port_id", "owner", "binding")}
                   == port_inputs[port["port_id"]], "manifest port differs from input")
    ev.exactly_ids(expected["required_gates"], ev.required_gates(request["scenario"]), "required_gates")
    if request["expectations"] is not None:
        ev.require(ev.same_bytes(request["expectations"], expected_ref),
                   "--expected differs from the run's frozen expectations")

    sources = checked_index(store, value["source_files"], "source_files")
    ev.require({name for name in sources if name.startswith("input/")} == input_paths
               and all(name.startswith(("input/", "source/")) for name in sources),
               "input source inventory mismatch")
    artifact_refs = ev.artifact_refs(value["artifacts"])
    artifacts = checked_index(store, artifact_refs, "artifacts")
    ev.require(not (artifacts & (sources | {"manifest.json", "manifest.sha256"})),
               "overlapping bundle indexes")
    command_records(value, store, {item["path"]: item for item in value["artifacts"]})
    missing = check_provenance(value, request, store, sources)
    validate_gates(value["gates"], value["scenario"], store)
    bundle_inventory(store)
    return request, missing


def gate(ident, status, reason, refs=()):
    return {"id": ident, "status": status, "reason": reason, "evidence_refs": list(refs)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    options = parser.add_mutually_exclusive_group(required=True)
    options.add_argument("--bundle", type=Path)
    options.add_argument("--compare", nargs=2, type=Path, metavar=("LEFT_RUN", "RIGHT_RUN"))
    parser.add_argument("--expected", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    roots = [root.absolute() for root in (args.compare or [args.bundle])]
    expected_store = ev.Inventory(args.expected.parent)
    repository = ev.Inventory(ROOT)
    try:
        output = ev.fresh_destination(args.output)
        ev.require(not any(output.is_relative_to(root) for root in roots),
                   "checker output must be outside the bundles")
        ev.require(output != args.expected.absolute(), "checker output cannot be the expected input")
        source_names = [name for name in ev.SOURCES if name.endswith(".py") and
                        Path(name).name in ("evidence.py", "check.py")]
        checker = {
            "scope": "BundleIntegrity",
            "source_root": str(ROOT),
            "source_files": [repository.inspect(name)[0] for name in source_names],
            "bundle_roots": [str(root) for root in roots],
            "expected_root": str(expected_store.root),
        }
    except (ev.Invalid, OSError, UnicodeError) as error:
        print(f"m1 checker: {error}", file=sys.stderr)
        return 2

    report = {
        "schema": "M1-CHECK-1",
        "operation": "compare" if args.compare else "check",
        "bundles": [],
        "expected": None,
        "checker": checker,
        "commands": [],
        "gates": [],
        "status": "NOT_RUN",
        "reason": None,
    }
    exit_code = 3
    try:
        report["expected"], _ = expected_store.inspect(args.expected.name)
        expected_ref, expected = ev.expectations(expected_store, args.expected.name)
        requests = [bundle(root, expected_ref, expected, report["bundles"]) for root in roots]
        missing_ports = []
        missing_inputs = []
        for request, missing_tools in requests:
            missing_ports += ev.missing_ports(request["scenario"])
            missing_inputs += ev.missing_inputs(request) + missing_tools
        reason = {
            "code": "NotReady",
            "missing_ports": [port for port in ev.PORTS if port in missing_ports],
            "missing_inputs": sorted(set(missing_inputs)),
        }
        report["reason"] = reason
        integrity = gate("BundleIntegrity", "PASS", {"code": "VerifiedFileInventory"}, report["bundles"])
        if "tool_versions.git" in missing_inputs:
            integrity = gate("BundleIntegrity", "NOT_RUN", reason)
        report["gates"] = [
            integrity,
            *[gate(ident, "NOT_RUN", reason) for ident in ev.GATES[1:]
              if ident in expected["required_gates"]],
        ]
    except (ev.Invalid, OSError, UnicodeError) as error:
        reason = {"code": "InvalidEvidence", "detail": str(error)}
        report.update(status="FAIL", reason=reason,
                      gates=[gate("BundleIntegrity", "FAIL", reason, report["bundles"])])
        exit_code = 2
    try:
        with output.open("xb") as destination:
            destination.write(ev.json_bytes(report))
    except OSError as error:
        print(f"m1 checker: could not save report: {error}", file=sys.stderr)
        return 2
    return exit_code


if __name__ == "__main__":
    sys.exit(main())

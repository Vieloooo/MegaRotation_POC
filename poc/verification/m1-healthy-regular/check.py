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


def check_command_index(commands, store, indexed):
    for item in commands:
        for stream in ("stdout", "stderr"):
            ref = item[stream]
            ev.require(ref["path"] in indexed and indexed[ref["path"]]["role"] == f"command_{stream}",
                       f"{ref['path']}: missing command stream index or wrong role")
            store.ref(ref)


def check_provenance(manifest, request, store, sources):
    tools = manifest["tool_versions"]
    ev.require(tools["python"] is not None, "missing Python version record")
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
        if name != "python":
            version, = probe(name, ("--version",))
            ev.require(version.decode().strip() == tool["version"], f"{name}: version evidence mismatch")
        if name in ("rustc", "cargo"):
            ev.require(tool["version"].split()[:2] == [name, ev.RUST], f"{name}: wrong version")

    before, after = manifest["revision_before"], manifest["revision_after"]
    ev.require(before == after, "revision changed during preflight")
    ev.require(before is not None and before["head"] == manifest["candidate"] and before["status"] == [],
               "candidate is stale or dirty")
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
    commit, = probe("git", (*ev.GIT_OPTIONS, "cat-file", "commit", candidate))
    root_tree, = probe("git", (*ev.GIT_OPTIONS, *ev.GIT_ROOT_TREE, candidate))
    poc_tree, = probe("git", (*ev.GIT_OPTIONS, *ev.GIT_POC_TREE, candidate, "--", "poc"))
    candidate_paths = ev.candidate_sources(candidate, commit, root_tree, poc_tree)
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
    artifact_refs = ev.run_fields(value)
    ref = value["input"]
    path = PurePosixPath(ref["path"])
    ev.require(len(path.parts) == 2 and path.parts[0] == "input", "input must be under input/")
    actual, request = ev.input_document(store, ref["path"])
    ev.require(actual == ref, "manifest input FileRef mismatch")
    input_paths = {name for name in store.refs if name.startswith("input/")}
    for key in ("candidate", "components", "scenario", "seed", "start_order", "wall_pause_ms", "limits"):
        ev.require(value[key] == request[key], f"manifest {key}: differs from input")
    port_inputs = {item["port_id"]: item for item in request["ports"]}
    for port in value["ports"]:
        ev.require({key: port[key] for key in ("port_id", "owner", "binding")}
                   == port_inputs[port["port_id"]], "manifest port differs from input")
    ev.exactly_ids(expected["required_gates"], ev.required_gates(request["scenario"]), "required_gates")
    if request["expectations"] is not None:
        ev.require(ev.same_bytes(request["expectations"], expected_ref),
                   "--expected differs from the run's frozen expectations")

    sources = {item["path"] for item in value["source_files"]}
    ev.require({name for name in sources if name.startswith("input/")} == input_paths
               and all(name.startswith(("input/", "source/")) for name in sources),
               "input source inventory mismatch")
    artifacts = {item["path"] for item in artifact_refs}
    ev.require(not (artifacts & (sources | {"manifest.json", "manifest.sha256"})),
               "overlapping bundle indexes")
    for item in value["source_files"] + artifact_refs:
        store.ref(item)
    check_command_index(value["commands"], store, {item["path"]: item for item in value["artifacts"]})
    missing = check_provenance(value, request, store, sources)
    for gate in value["gates"]:
        for item in gate["evidence_refs"]:
            ev.require(item["path"] in store.refs, "unindexed gate evidence")
            store.ref(item)
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
        ev.check_report_fields(report, requests[0][0]["scenario"])
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

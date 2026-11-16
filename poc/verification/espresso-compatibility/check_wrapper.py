#!/usr/bin/env python3
"""Narrow, isolated wrapper provenance/lifecycle check; does not launch Espresso."""
import argparse
import hashlib
import http.client
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import socket
import subprocess
import sys
import time


def start_ticks(pid):
    return int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[19])


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--candidate", required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--image", type=Path, required=True)
    a = p.parse_args()
    repo = Path(__file__).resolve().parents[3]
    # The outer command records HEAD before entering the read-only namespace.
    # Git itself opens /dev/null read/write, which this namespace intentionally
    # disallows; the candidate runner does not invoke Git inside the namespace.
    assert (a.output / "candidate.txt").read_text().strip() == a.candidate
    assert {name for _, name in socket.if_nameindex()} == {"lo"}
    spec = importlib.util.spec_from_file_location("ec16_capture_under_review", repo / "poc/scenarios/espresso-compatibility/capture.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    root = a.output
    run = root / "lifecycle"
    run.mkdir()
    recorder = module.Recorder(run / "bundle")
    configuration = {}
    wrapper = module.BootstrapAnvil(run, configuration)
    native = a.image / "native"
    command = [str(native / "lib/ld-linux-x86-64.so.2"), "--library-path", str(native / "lib")]
    wrapper.prepare(command, native, recorder)
    original_args = ["-p", "0", "--slots-in-an-epoch", "0", "--balance", "1000000"]
    env = {"PATH": f"{wrapper.bin}:/usr/bin:/bin", "LC_ALL": "C", "TZ": "UTC"}
    with (run / "anvil.stdout").open("xb") as log:
        process = subprocess.Popen(["anvil", *original_args], env=env, stdout=log, stderr=subprocess.STDOUT,
                                   cwd=run, start_new_session=True)
    try:
        deadline = time.monotonic() + 20
        port = None
        while time.monotonic() < deadline:
            assert process.poll() is None, "verified Anvil exited before listening"
            match = re.search(r"Listening on (?:127\.0\.0\.1|0\.0\.0\.0):([0-9]+)", (run / "anvil.stdout").read_text())
            if match:
                port = int(match[1])
                break
            time.sleep(0.05)
        assert port is not None and port != 18775
        client = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
        body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "eth_chainId", "params": []}).encode()
        client.request("POST", "/", body=body, headers={"Content-Type": "application/json"})
        response = client.getresponse()
        reply = response.read()
        client.close()
        assert response.status == 200 and json.loads(reply)["result"] == "0x7a69"
        (run / "chain-id.request").write_bytes(body)
        (run / "chain-id.response").write_bytes(reply)
        entries = list(wrapper.invocations.glob("*.json"))
        assert len(entries) == 1
        invocation = json.loads(entries[0].read_bytes())
        assert invocation["argv"][1:] == original_args
        assert invocation["exec_argv"] == command + [str(native / "anvil")] + original_args
        assert invocation["namespace_pid"] == process.pid and invocation["namespace_ppid"] == os.getpid()
        assert invocation["start_ticks"] == start_ticks(process.pid)
        assert invocation["port_args"] == [["-p", "0"]]
        assert wrapper.still_present(invocation)
        wrong_start = dict(invocation, start_ticks=invocation["start_ticks"] + 1)
        assert not wrapper.still_present(wrong_start)
    finally:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
        try:
            exit_status = process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            exit_status = process.wait(timeout=3)
    wrapper.finish(os.getpid(), recorder)
    assert configuration["bootstrap_anvil"]["cleanup"]["leftover_namespace_pids"] == []
    assert not wrapper.still_present(invocation)

    # This helper intentionally requires reaping, not merely lack of a live server.
    # Demonstrate the distinction without inventing an upstream exit status.
    zombie = os.fork()
    if zombie == 0:
        os._exit(0)
    try:
        ident = {"namespace_pid": zombie, "start_ticks": start_ticks(zombie)}
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            state = Path(f"/proc/{zombie}/stat").read_text().rsplit(")", 1)[1].split()[0]
            if state == "Z":
                break
            time.sleep(0.01)
        assert state == "Z" and wrapper.still_present(ident)
    finally:
        waited, status = os.waitpid(zombie, 0)
    assert waited == zombie and os.waitstatus_to_exitcode(status) == 0
    assert not wrapper.still_present(ident)
    report = {"status": "PASS", "candidate_sha": a.candidate,
              "classification": "independent wrapper lifecycle smoke, no Espresso network",
              "original_arguments": original_args, "actual_local_port": port,
              "dedicated_l1_port_reserved_for_full_run": 18775,
              "namespace_parent_pid": os.getpid(), "namespace_child_pid": process.pid,
              "observed_child_exit_status": exit_status, "rpc_chain_id": "0x7a69",
              "pid_start_identity_check": "PASS", "already_reaped_check": "PASS",
              "zombie_then_reap_check": "PASS", "configuration": configuration,
              "wrapper_sha256": hashlib.sha256(wrapper.wrapper.read_bytes()).hexdigest(),
              "verified_anvil_sha256": hashlib.sha256((native / "anvil").read_bytes()).hexdigest()}
    (root / "wrapper-independent-report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": "PASS", "candidate": a.candidate, "port": port, "output": str(root)}, indent=2))


if __name__ == "__main__":
    main()

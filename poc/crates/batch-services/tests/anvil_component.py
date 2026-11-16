#!/usr/bin/env python3
"""Run one feature-11 scenario against a private manual-mining Anvil child."""
import argparse
import json
from pathlib import Path
import socket
import subprocess
import time
from urllib.error import URLError
from urllib.request import Request, urlopen


def save(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--anvil", type=Path, required=True)
    parser.add_argument("--driver", type=Path, required=True)
    parser.add_argument("--artifact", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--wall-pause-ms", type=int, required=True)
    args = parser.parse_args()
    args.out.mkdir()
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    clock_genesis = 1_700_000_000
    genesis = args.out / "anvil-genesis.json"
    save(genesis, {
        "config": {"chainId": 31337, "homesteadBlock": 0, "eip150Block": 0, "eip155Block": 0,
                   "eip158Block": 0, "byzantiumBlock": 0, "constantinopleBlock": 0,
                   "petersburgBlock": 0, "istanbulBlock": 0, "berlinBlock": 0, "londonBlock": 0,
                   "shanghaiTime": 0},
        "timestamp": hex(clock_genesis), "gasLimit": hex(100_000_000), "difficulty": "0x0",
        "alloc": {"0x7e5f4552091a69125d5dfcb7b8c2659029395bdf": {"balance": hex(10**22)}},
    })
    command = [str(args.anvil), "--host", "127.0.0.1", "--port", str(port), "--quiet",
               "--no-mining", "--order", "fifo", "--no-cors", "--accounts", "0", "--threads", "2",
               "--hardfork", "shanghai", "--timestamp", str(clock_genesis), "--chain-id", "31337",
               "--gas-limit", "100000000", "--disable-default-create2-deployer", "--init", str(genesis)]
    save(args.out / "anvil-command.json", command)
    with (args.out / "anvil.log").open("w") as log:
        node = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
        try:
            probe = Request(f"http://127.0.0.1:{port}",
                            data=b'{"jsonrpc":"2.0","id":1,"method":"eth_chainId","params":[]}',
                            headers={"Content-Type": "application/json"})
            for _ in range(100):
                if node.poll() is not None:
                    raise RuntimeError("Anvil child exited during startup")
                try:
                    with urlopen(probe, timeout=0.2) as response:
                        assert json.load(response)["result"] == "0x7a69"
                    break
                except (URLError, TimeoutError):
                    time.sleep(0.025)
            else:
                raise RuntimeError("Anvil did not become ready")
            run = [str(args.driver), f"127.0.0.1:{port}", str(args.artifact),
                   str(args.out / "run"), str(args.wall_pause_ms)]
            save(args.out / "scenario-command.json", run)
            with (args.out / "scenario.log").open("w") as scenario_log:
                subprocess.run(run, stdout=scenario_log, stderr=subprocess.STDOUT, check=True, timeout=90)
            result = json.loads((args.out / "run" / "result.json").read_text())
            print(json.dumps({**result, "path": str(args.out / "run")}))
        finally:
            if node.poll() is None:
                node.terminate()
                try:
                    node.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    node.kill()
                    node.wait()


if __name__ == "__main__":
    main()

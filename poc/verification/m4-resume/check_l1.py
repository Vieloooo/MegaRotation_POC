"""Bind real private-Anvil RPC observations to virtual driver finalization/reader steps."""

from collections import defaultdict

import independent_codec as crypto
import receipt_checker as receipt
from verified_inputs import need


def audit(bundle, rows):
    startup = [row["startup_rpc"] for row in rows if "startup_rpc" in row]
    need(len(startup) == 1, "missing one startup RPC inventory")
    chain = [row for row in startup[0] if row["request"]["method"] == "eth_chainId"]
    automine = [row for row in startup[0] if row["request"]["method"] == "anvil_getAutomine"]
    need(len(chain) == len(automine) == 1 and
         receipt.q(chain[0]["response"]["result"]) == bundle["config"]["chainId"] and
         automine[0]["response"]["result"] is False, "wrong Anvil scope or automine")
    mined = {}
    sent = {}
    finalized = {0: 0}
    delivered = defaultdict(dict)
    read = defaultdict(dict)
    blocks = bundle["blocks"]
    receipts = {row["transactionHash"].lower(): row for row in bundle["receipts"]}
    state = (0, 0, 0)
    visible = {}
    command_ids = {}
    for line in rows:
        if "command" not in line:
            continue
        cmd, ack = line["command"], line["reply"]
        # Batcher preparation queries a cursor but does not issue a chain write.
        if cmd["op"] == "m4_prepare_batch":
            need(ack["status"] == "ready" and not any(
                event["request"]["method"] in {"eth_sendRawTransaction", "evm_mine", "evm_setNextBlockTimestamp"}
                for event in line["rpc"]
            ), "unexpected write by batch preparation")
            continue
        need(ack["id"] == cmd["id"], "driver reply id")
        need(command_ids.setdefault(cmd["id"], cmd) == cmd, "driver command id reused with new bytes")
        now, included, final = (
            ack["reference_us"], ack["included_height"], ack["finalized_height"],
        )
        need(now >= state[0] and included >= state[1] and final >= state[2]
             and 0 <= final <= included < len(blocks), "nonmonotonic L1 driver")
        for reader, tip in ack["visible_heights"].items():
            need(visible.get(reader, 0) <= tip <= final, "unfinalized or regressed reader prefix")
            visible[reader] = tip
        rpc = defaultdict(list)
        for event in line["rpc"]:
            rpc[event["request"]["method"]].append(event)
        if cmd["op"] == "submit":
            for send in rpc["eth_sendRawTransaction"]:
                need(send["request"]["params"] == [cmd["raw"]], "driver changed transaction bytes")
                tx_hash = crypto.hx(crypto.keccak256(crypto.as_bytes(cmd["raw"])))
                response = send.get("response") or {}
                if "result" in response:
                    need(response["result"].lower() == tx_hash, "eth_sendRawTransaction wrong hash")
                    sent.setdefault(tx_hash, now)
        if ack["status"] != "ready":
            state = now, included, final
            continue
        if cmd["op"] == "mine":
            height = cmd["slot"]
            block = blocks[height]
            need(height not in mined and included == height
                 and rpc["evm_mine"] and rpc["evm_setNextBlockTimestamp"], "real manual mine missing")
            actual = ack["result"]["included_block"]
            need(actual["header_and_transactions"] == block, "mine output disagrees with raw block")
            need(any((call.get("response") or {}).get("result") == block
                     for call in rpc["eth_getBlockByNumber"]), "mine lacks raw RPC block")
            for tx in block["transactions"]:
                tx_hash = tx["hash"].lower()
                need(tx_hash in sent and sent[tx_hash] <= now, "mined tx lacks original submitted bytes")
                need(any((call.get("response") or {}).get("result") == receipts[tx_hash]
                         for call in rpc["eth_getTransactionReceipt"]), "mine lacks raw RPC receipt")
            mined[height] = now
        elif cmd["op"] == "finalize":
            need(not rpc["evm_mine"] and ack["result"]["finalized_through"] == final,
                 "invalid finalized boundary")
            for height in range(state[2] + 1, final + 1):
                need(height in mined and mined[height] < now, "finalization before actual mining")
                finalized[height] = now
        elif cmd["op"] == "deliver":
            reader, tip = cmd["reader"], cmd["through_height"]
            need(not rpc["evm_mine"] and visible[reader] == tip
                 and ack["result"]["delivered_through"] == tip, "invalid reader delivery")
            for height in range(1 + max(delivered[reader], default=0), tip + 1):
                need(height in finalized and finalized[height] <= now, "reader delivered before finality")
                delivered[reader][height] = now
        elif cmd["op"] == "read" and cmd["query"]["kind"] == "block":
            height, reader = cmd["query"]["height"], cmd["reader"]
            block = blocks[height]
            actual = ack["result"]["block"]
            need(actual["header_and_transactions"] == block
                 and actual["receipts"] == [receipts[tx["hash"].lower()] for tx in block["transactions"]]
                 and visible[reader] >= height, "verified block query differs from finalized raw prefix")
            read[reader].setdefault(height, now)
        state = now, included, final

    need(set(mined) == set(range(1, len(blocks))), "raw L1 evidence contains unrecorded blocks")
    return {"mine_us": mined, "submit_us": sent, "finalized_us": finalized, "deliver_us": dict(delivered),
            "read_us": dict(read), "reader_tips": visible}

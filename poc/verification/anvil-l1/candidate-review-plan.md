# Feature09 candidate review preparation, after interface freeze

Accepted interface SHA: `daad412e46eeecd64ef9b8af8175e64ec40edc53`.
Verifier independently checked all three conditions: fixed `(7,0,1,2,l1Floor)` P0 Policy in §2/A05; slot is actual continuous block number including deployment and finality uses relative slot-production tick; FROZEN with golden artifact anchor. Interface review is PASS, still not an implementation verdict. No Builder source has been read in preparing this file.

## Raw-receipt checker

`poc/verification/anvil-l1/receipt_checker.py` imports only the pre-implementation independent codec/golden type strings from `poc/verification/anvil-l1`. It includes its own strict ABI decoder and accepts ordinary Ethereum JSON-RPC structures; it does not import a candidate decoder or handler.

Expected bundle shape (extraction can adapt the candidate transcript later without changing expected semantics):

```json
{
  "config": {
    "chainId": 31337,
    "configRef": "0x...32 bytes...",
    "authority": "0x...20 bytes...",
    "inbox": "0x...20 bytes...",
    "submitter": "0x...20 bytes...",
    "signers": {"governance": "0x...", "certifier": "0x...", "da": "0x..."},
    "genesisHeight": 0,
    "genesisOrigin": 0,
    "genesisTimestampSeconds": 1700000000,
    "slotSeconds": 2,
    "announcementSeconds": 6
  },
  "blocks": ["actual eth_getBlockByNumber(number,true) objects, genesis onward"],
  "receipts": ["all actual eth_getTransactionReceipt objects, one per included tx"],
  "historicalCursorReads": [
    {"height": 3, "returnData": "actual eth_call currentCursor at block 3", "writtenAt": [3,0,0]}
  ]
}
```

The checker requires exact block/transaction/receipt/log identity, all receipts including failed/no-log transactions, continuous parent chain, timestamp slots, and actual signature recovery under actual deployment addresses. It independently decodes and checks configured-emitter events against calldata; applies accepted full-prev transitions; verifies control sequence and idempotent retry behavior; maps block-global RPC logIndex to transaction-local position before filtering; pairs the same successful transaction's Set and Resume; reconstructs original written_at and block-end history; compares optional actual historical state returns. Failed receipts must have no surviving logs.

Run preparation self-check:

```sh
python3 poc/verification/anvil-l1/receipt_checker.py --self-test
```

Observed: PASS for four complete business calldata roundtrips and four noncanonical trailing-byte rejections; **zero real receipts checked**. Later candidate evidence:

```sh
python3 poc/verification/anvil-l1/receipt_checker.py --bundle /ABSOLUTE/CANDIDATE/raw-rpc-bundle.json
```

This checker is a consistency/semantic check under the declared Anvil-RPC trust boundary; it does not establish MPT/PoS proof validity, grant/ack, DA persistence, actual canonical ReplayPosition meaning or D/ResumeCut.

## Candidate review order

1. Capture exact candidate SHA and clean state; verify authorized paths only and contract/interface SHA. Read diff and implementation only now. Match generated ABI against all frozen selectors/topics/type strings. Ensure exact field hash use, strict source encoding, signatures and sender/sequence/request guards, Overflow behavior and fixed Policy.
2. Trace one real runner invocation into the production Rust driver and actual JSON-RPC backend. Confirm deployment/raw-submit/mine/finalize/delivery use that same component. An uncalled Rust state machine plus Python mining is not A14–A17 evidence. Capture actual binary/build identity and the exact command script.
3. Follow request cache order: signature/format checks before cached result, then cache before stale seq/prev/target checks. Conflicting request content must not consume seq/rid or mutate state. Signatures themselves are not retry identity except exact DA certificate bytes are bound by the attestation.
4. Verify that only immutable Authority can call production Override and only SubmitResume exposes that path. Atomic rollback evidence needs a real included status0 tx plus state reads before/after. Precheck rejects alone do not exercise a later failure after inner writes; the allowed test-only harness must be isolated from production deployment.
5. Independently check all raw included headers/receipts/events/call data with the prepared checker. Record actual deployment addresses and public identities, not offline vector addresses. No successful eth_call is counted as an accepted tx. Failed tx and transactions with unrelated/no protocol logs still establish coverage.
6. Read actual historical cursor at deployment, batch, retry, override and later empty blocks. Compare immutable receipt replay and original written_at. Same-block position history must retain both Resume pairs; a final block tag read cannot replace either exact event.
7. Inspect and test reader A/B visibility: metadata known without receipts is WAIT for coverage; B cannot use A's prefix; future on-disk/RPC rows do not authorize reads; gaps and missing Set/Resume material do not advance visible prefix; conflicting complete data reject without modifying the original prefix.
8. Trace explicitly separated Tick/Submit/Mine/Finalize/Deliver. Inspect all wall-time and RPC paths for incidental automining/finalization/delivery. Test strict tick/slot boundaries, exact headers, pauses >=250ms, lower/backward input, and same script with different wall pauses. Compare canonical virtual traces rather than timing text or PID/port.
9. Command replay and unknown mine are independent cases. Exact repeated command must return cached original result with no second RPC side effect; changed payload same id rejects. For uncertain mine, inject lost response **after** the mine reaches Anvil; the driver must retain unresolved state and reconcile actual canonical head/slot before retry. A fake error before send does not exercise the hazard. No blind next mine may be issued while unresolved. Inspect crash persistence claims separately: the component contract does not automatically establish full01/02 crash recovery.
10. Mark every A01–A17 case PASS/FAIL/INCONCLUSIVE based on actual evidence. A18 remains INCONCLUSIVE until real02/03 integration SHA; A19 is consumer-owned. Report specific missing evidence even if nominal unit tests are green. Fixes return to Builder, then repeat relevant checks on the new exact candidate SHA before Tester.

All Rust/Foundry batches/heavy runs use one outer `flock /tmp/megaroation-poc-heavy.lock` with `CARGO_BUILD_JOBS=2`; no nested lock. This preparation ran only Python selfchecks, no Anvil, Rust or Foundry build.

候选前r1澄清已由lead/Verifier独立确认：所有写入口的参数区精确规范ABI编码；加入真实trailing/alias拒绝用例，规范签名与幂等对照不变。

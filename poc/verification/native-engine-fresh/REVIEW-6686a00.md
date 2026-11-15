# Independent NATIVE-ENGINE/FRESH review

Candidate `6686a009e0eeea261026dac807742efed76b17c6` passes the frozen fresh-run implementation checks described below. All 111 Rust tests pass, including 106 prior regressions and five new independent crypto/codec tests. The independent artifact checker accepts all 21 actual two-worker runs. Six same-seed wall-pause/start-order pairs preserve complete logical evidence and native storage bytes. No confirmed production finding remains in these checked profiles.

Both final envelope controls pass: a valid BodyRequest padded with protobuf unknown bytes to exactly 64 KiB is accepted by the real worker; 64 KiB + 1 is rejected atomically with zero committed Ack, clock transit or route. The wider formal 66-run matrix belongs to the integrated-revision simulation step; overall I1b is not complete.

Expectations were derived before implementation inspection from `task-contract-native-engine-fresh-v1.md`, the lead's proto/hash and WorkBinding freezes, and pinned Commonware 2026.10.0/upstream `92a2f68a0e131c8b69de41e84aec37da6280f212`. Contract SHA256 is `c72a62958a3e89113e30954058a8c2883ba524ec67cc8e6708c2a308dd10a9e3`. The verifier changed only this directory and the two approved manifest targets; no dependency, production, vendor or parent benchmark source was changed.

## Executed evidence

| Gate | Evidence and scope |
|---|---|
| F1 registry/process/origin | Every main run has two distinct successful child PIDs for workers 0/1. Public identity/BLS keys, PoPs, logical-member order, native signer indices and derived config/namespace hashes are independently checked. Worker vote bytes and real certificate signers link to actual guarded worker SignAudit history. Scripted 2–4 provenance, transcript IDs, entry sequences and received leader/body evidence are separate. |
| F2 wire/hash | Literal block, Grant/Ack and tie hashes derived before code inspection pass against compiled protobuf types. Independent native decoding rejects malformed/trailing bytes. Protobuf unknown fields and repeated-message/oneof merge controls pass. Root-run transport controls accept exact 1 MiB and reject 1 MiB + 1. Exact 64 KiB BODY decoding succeeds with unknown-field padding; 64 KiB + 1 rejects before commitment. |
| F3 durable atomic commitment | Raw `frames.bin`, every Grant/Ack and per-worker root journal/head match. Logical Ack, record-chain and protected head hashes are recomputed. A well-hashed invalid Ack containing a valid clock request followed by an invalid timer produces no AckCommitted, clock transit or route. Unsolicited and altered duplicate Acks fail;98 exact duplicate Acks commit once. |
| F4 quota/routing | Complete per-recipient/channel token history, one-based attempts, source/destination IDs, FIFO 20 ms links, directed queue 64/drop-newest and BODY hold deadlines are checked. The quota case has one actual rate rejection and one overflow; unlimited-quota control has two overflows. No empty-bucket/empty-route assumption is used. |
| F5 private work/body | Work IDs bind the complete frozen callback coordinates and generation. Request/start/due/completion/cancellation remain distinct. Computation waits for private body ancestry; 5 ms completion is explicit. Real sign subjects link to matching completed propose/verify/certify work and persistence. Proof-before-body observes worker 1's proof before its private body and delays contiguous delivery. |
| F6/F7 clock/signing | Across the 21 runs: 8,509 audited clock reads, 112 CREATED and 93 CACHE_HIT decisions. Each successful sign has a controlled task/grant, calibrated node projection, current sample/lease, strict lease and local-stop guard, exact native attestation and unchanged cached bytes. Real Nullify retries exercise cache reuse and distinct transport attempts. |
| F8 native crypto | Native four-distinct-member certificates verify, including a quorum excluding the local member. A mathematically valid aggregate of three distinct BLS signatures fails native quorum verification. Duplicate signers, wrong namespace/kind/epoch/index/PoP and malformed bytes reject. The wrong-kind and wrong-context process scenarios reject their declared injected input and continue with later valid traffic. |
| F9 provenance | Actual VOTE/CERTIFICATE sends require exact committed native artifact/witness/route authorization and valid receipts. BODY content/request provenance is checked separately. Source review confirms the resolver response gate uses bounded native certificate verification and does not invent a voter-witness requirement. These profiles emitted no RESOLVER packets: no executed resolver-repair/liveness coverage is claimed. |
| F10 delivery/archive | Native finalization proofs are verified independently; every delivered block belongs to a privately available, validated contiguous prefix covered by a finalization of it or a descendant. Each height delivers once. Closed nodes can observe existing proofs/deliver known data without new signing. |
| F11 scenarios | All 11 cases at seed 7 pass. Healthy delivers height 2 at both real workers; two-real-alone retains quorum 4 and delivers nothing. Close-propose/verify/certify cancel actual native work, and lease-loss retires at the last delivered lease boundary. |
| F12 replay | Healthy and two-real-alone run at seeds 7/42/99 with normal start/pause 0 and reversed start/pause 2 ms. All six pairs match raw JSONL, protobuf frames, both root/witness journals and heads, and native voter/body files:19 files per healthy pair and 14 per two-real-alone pair. Host PIDs, paths and wall diagnostics are excluded explicitly. |

The 21 main runs contain 14,655 trace rows, 3,755 committed Acks, 1,793 sends, 1,742 network deliveries, 38 proof observations and 38 block deliveries. These are fixture observations, not measured AWS performance or a four-live-replica quorum.

## Counterexamples and corrections

The early moving implementation had partial Ack application, missing exact-Ack deduplication, zero-based attempts, an archive BODY quota bypass, and an unnecessary protobuf re-encoding restriction. The named candidate closes those paths: see `native_hub.rs:553`/`AckPlan` at 1095, duplicate handling at 536, `native_network.rs:82`/277, and the BodyPacket decoder in `native_replica.rs`. The atomic, duplicate and wire controls use adversarial worker 0 stubs with a real worker 1; they are transport controls, not native quorum runs.

The first duplicate-control wrapper reported failure because it counted `DuplicateAck`; the implementation's event is `AckDuplicate`. Read-only rechecking found 196 raw worker 0 Acks representing 98 exact pairs, all equal to their corresponding committed Ack. The 91 committed clock-request effects produced exactly 91 clock transits. The original failed report is preserved; `socket-controls-recheck.json` records the correction and its source hash.

An early cancellation concern was **not reproduced**. Independent tonic-authority probes drove a genuine native worker through explicit Close, exact lease equality at 1,100,250 µs, and original local-stop equality at 1,799,000 µs. Each final grant records one discarded completion, zero completed work, zero body persistence and zero sign effects; the worker exits successfully. `driver.rs:763` disposes retired jobs before invoking the grant action, closing the completion receiver. `boundary-journal-recheck.json` independently binds the probe frames to the worker's protected root journals. No native callback consumption is inferred from an ignored sender result.

Eight semantic mutants preserve native signatures while recomputing every affected Grant, logical Ack, subsequent root record/header checksum and raw protobuf stream. They reject at the intended rule: expired signing, uncreated cache hit, false witness reference, scripted worker output, early work completion, proof-only delivery without private body, wrong work-parent context and wrong proposer/elector context. Fully materialized mutant directories are retained; none merely fails a stale checksum.

## Reproduction and artifacts

```text
python3 poc/verification/native-engine-fresh/run_verification.py
python3 poc/verification/native-engine-fresh/check.py RUN_DIR --output REPORT.json
python3 poc/verification/native-engine-fresh/verify_pairs.py PAIRS.json --output REPORT.json
```

The stable checker returns a JSON list with one entry per run. PASS entries include case, seed, rows, metrics, delivered heights, source and a digest of the complete canonical logical trace. It reads the resolved scenario, raw frames/trace, process diagnostics, root/witness ledgers and heads, and actual native storage files. The public-key oracle uses pinned native codecs/BLS primitives, not builder expectation helpers or signing keys.

- Full capped regression/provenance: `/tmp/native-fresh-independent-final-bcssj08p`.
- Latest complete 21-run checks: `validated-runs.json` beside this report.
- Main actual runs: `poc/artifacts/verification/native-engine-fresh/candidate-6686a00-seed7` and `candidate-6686a00-pairs`.
- Complete byte comparisons: `paired-byte-comparison.json`.
- Materialized semantic mutants: `/tmp/native-fresh-semantic-mutants-r0a5z2qj` and `semantic-mutants-report.json`.
- Initial six transport controls: `/tmp/native-fresh-socket-controls-f9u3ceyw`; preserve its original label-mismatch failure report.
- Genuine completion-boundary probes: `/tmp/native-fresh-completion-boundaries-o3uig2my`.
- Final two payload-boundary controls: `/tmp/native-fresh-socket-controls-o41jcg9d` and `payload-boundary-controls.json`.

Production native-hub SHA256 remains `0a78e18b8bff6de4db34c4c122326aed6d259c3e07dae11d053d4874d5b4386e`; native-worker remains `f53dfab1f581af3b968cfe678c2825f67790f9709af789930ca4caca960a3dc9`. Complete current source/helper hashes and commands are in `candidate-6686a00-report.json`. The helper gained formatting and raw-mutant/transport fixture support after earlier controls; production binaries did not change. Bootstrap formatting and whitespace checks pass. The full 111-test run preceded the last transport-fixture extension; the five independent pure tests were repeated successfully afterward. Production source and binaries stayed unchanged.

Acceptance is bounded to this fresh-run fixture and recorded checks. Same-cid crash recovery, original native deadline/auxiliary callback restoration, an executed resolver-repair scenario, v10 admission/windows and full I1b remain outside this result. Integrated checks and the approved formal 66-run collection follow before experimental claims.

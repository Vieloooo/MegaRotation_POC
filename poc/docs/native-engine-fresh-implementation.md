# Fresh native Simplex integration

The harness starts two actual worker processes on one controlled tonic stream each. Each worker constructs a genuine Commonware Simplex engine inside a session grant. Its private application uses the controlled native Blob adapter to persist proposal and certification data. The hub routes BODY, VOTE, CERTIFICATE and RESOLVER traffic. The optional scripts use only members 2–4 and wait for the actual leader vote and a valid body chain before signing. The native quorum is four of five.

A worker commits the complete wire Grant and logical Ack to its per-worker root step ledger before releasing the Ack. Its journal and head use separate domains from the native voter journal and witness ledger. An exact cached Grant returns the identical Ack. The hub stages all schedule, route, timer and counter changes in a draft before marking the Ack committed. It accepts only exact already-committed duplicate Acks and commits their effects once.

The guard runs before each underlying BLS signing operation and cached attestation reuse. Audit `subject_bytes` is the native Subject::message representation, Proposal bytes for Notarize/Finalize and Round bytes for Nullify. `attestation_bytes` is the native Attestation encoding. Same-kind/round subject conflicts fail closed. Native cross-kind eligibility remains in Simplex. `ProofObserved.finalization_bytes` is the inner native Finalization encoding. A valid proof can be observed before body delivery. Contiguous delivery starts from the configured external genesis fixture.

A transactional witness batch commits native VOTE and direct CERTIFICATE route authorization before the outer Ack. The stable route ID follows the witness contract. Each actual accepted sender call creates a checked per-message attempt starting at 1. Effect identity is worker, step and effect index in the ordered full Ack. The Ack hash binds payload, priority and provenance for each attempt. Duplicate Acks create no additional attempt.

## Canonical layout

Base profile JSON is compact serde struct encoding with fields in this order.

```text
profile,package,commonware,scenario,seed,registry_hash,epoch,start,stop,cutoff,
epsilon,lease,refresh,clock_transit,link_delay,link_jitter,queue_capacity,
overflowing,quota_per_second,mailbox_capacity,grpc_max_bytes,payload_bound,
work_us,payload_size,virtual_crypto_us,leader_timeout,certification_timeout,
timeout_retry,fetch_timeout,fetch_retry,height_limit,sections,view_retention,
scripted,body_hold_until,close_on_work,lose_clock_after,startup_skew,
stress_tick,stress_count
```

Resolved profile order is `base,members,config_hash,run_id,genesis,cid,namespace,native_config_hash,app_fixture_hash`. Each member is `member,identity,bls_key,pop,signer_index`. Configuration fields live in `scenario.json`. The registry uses sorted logical members and verified PoPs, independently of sorted native signer position. The stable deterministic elector maps logical `(view−1)%5` into that signer order.

WorkId uses H(`I1b/fresh-work/v1`, [cid, member u32BE, kind u32BE, epoch u64BE, view u64BE, parent_view u64BE-or-zero, parent_digest32-or-zero, requested_payload32-or-zero, request_generation u64BE]). Request generation increments within each worker in granted callback order. A zero placeholder marks an absent callback coordinate. Propose has no requested payload. Certify has no parent view/digest in its callback.

Application audit order is `work,network,signing`. The WorkBinding order is `work,member,epoch,view,parent_view,parent_digest,requested_payload,request_generation,request_tick_us`. WorkPending records the binding. WorkStarted adds `start_tick_us,due_tick_us,body_digest,body_parent,native_parent_view`. WorkCompleted has `completed_tick_us,result_valid,body_digest,body_parent,native_parent_view` after the binding. WorkDiscarded records `work,binding,cancel_tick_us,reason`. The optional native parent view comes from a verified native certificate for the exact view and body when available. The original request tick survives a body wait. An explicit WorkCompleted grant must match the computed due tick. A canceled request's late completion is discarded.

Envelope `payload_hash` is H(`I1b/native-wire/v1`, [channel u32BE, exact received payload bytes]). BODY provenance `content_id` is the block digest for push/reply or the requested digest for request. Request IDs come from the BodyPacket. Push uses ID zero, and requests retain IDs across retries. BodyPacket uses protobuf semantics. Each embedded fixture block has exact canonical encoding and deterministic bytes.

Resolver provenance `request_id` is the first native u64 BE. Its `source_record_id` is H(`I1b/resolver-source/v1`, [exact complete native resolver message]). Responses pass bounded typed certificate verification. The native resolver controls servability. Requests and errors have no vote witness prerequisite.

Script transcript ID is H(`I1b/fresh-script/v1`, [run_id, member u32BE]). A generated scripted vote has entry ID H(`I1b/fresh-script-entry/v1`, [transcript_id, sequence u64BE, typed native vote bytes]). Script audit labels every emitted packet as scripted. The negative fixture ID uses H(`I1b/fresh-script-negative/v1`, [transcript_id, scenario UTF-8, wrong-kind native wire bytes]).

## Scenario scheduling

- Healthy has height limit 2 and closes both sessions after both workers deliver height 2.
- Two-real-alone disables scripted peers and runs through cutoff with the quorum threshold unchanged.
- Proof-before-body has height limit 1 and holds BODY delivery to worker 1 until 1,250,000 µs.
- Close-propose, close-verify and close-certify schedule Close at the first matching real WorkStarted tick plus 2,500 µs. Work completion is scheduled at the original tick plus 5,000 µs.
- Lease-loss drops samples from service tick 1,050,000 µs onward. Workers retire at the last delivered lease boundary.
- Native-proof-negatives stages one wrong-kind Finalization by wrapping the attestation of the first valid notarization received by scripted member 2. Wrong-context also replaces the envelope cid with H(`I1b/wrong-context-fixture/v1`, [cid]) in a controlled delivery fault. No key is used to generate these invalid proofs.
- Quota and router-overflow disable scripts and issue 65 distinct BODY requests from worker 0 to worker 1 at 1,030,000 µs. Quota uses the 64-message quota. Router-overflow uses quota value zero for unlimited admission so the 64-envelope router limit applies directly.

Worker 0 requests calibration at 1,000,000 µs and worker 1 at 1,020,000 µs. Calibration replies and Enter are independently granted. Body, vote, certificate and resolver queues are directed and individually FIFO with 20 ms latency. Root archive requests and replies share native BODY quota and require a fresh node clock and root grant authority. Archive verification and retention continue after consensus close.

## Build and initial smoke

```sh
./poc/scripts/bootstrap.sh build --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline --bins
.build/target/debug/native-hub --scenario healthy --seed 7 --output /tmp/rotation-i1b-native-first
```

The initial root-run smoke exited successfully with two child PIDs, both exit codes zero and both worker delivery heights 2. It is available in `/tmp/rotation-i1b-native-first`. The smoke trace has 393 lines and SHA-256 `b7a972355df27c8418b516c0d0a074750df9bf28f9d13938a4f6dcdbc3716a17`. That binary preceded the final transactional Ack validation, one-based attempts, archive quota and refined audit records. Each run writes canonical `virtual-trace.jsonl`, `frames.bin`, the resolved scenario, process diagnostics, complete worker step journals, witness ledgers and native storage directories.

The later source passed all 106 tests with `./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline --no-fail-fast`. This preserves the 102 prior tests. The four fresh tests cover a forged well-hashed Ack with a valid send and bad timer, exact and altered duplicate Acks, native three-versus-four distinct-signer and wrong-kind proof controls, cached-signing denial at Close and lease equality, and a calibrated actual Engine proposal through the granted 5 ms completion and committed voter witness. The last test uses a 16 MiB debug test thread stack for the native Engine future. Locked offline binary compilation, scoped formatting and whitespace checks passed.

| Final build artifact | SHA-256 |
| --- | --- |
| native-hub | `0a78e18b8bff6de4db34c4c122326aed6d259c3e07dae11d053d4874d5b4386e` |
| native-worker | `f53dfab1f581af3b968cfe678c2825f67790f9709af789930ca4caca960a3dc9` |
| Cargo.lock | `1e9fc6c8527d646eeb00f232c70d20effc9d8f0d39895bd6f9e239fea61f27d2` |
| native.proto | `b8a1d4fdb4c2e69758600830bcd226b3828bd2372c184a8df430c1c55a95e759` |

The final binaries still require the fresh subprocess matrix and independent verification. The production root step cache covers duplicates during the same process lifetime. Its files retain the protected output record for independent checking.

Same-cid restart, durable voter timing and callback restoration, and v10 admission/window/finality integration remain separate work. Native scripted inputs exercise a four-signature certificate with two actual workers. They do not establish a four-live-replica quorum.

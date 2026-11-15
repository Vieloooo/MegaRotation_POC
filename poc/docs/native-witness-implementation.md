# Native voter journal witness

This slice uses actual Commonware 2026.10.0 Storage, Blob and voter artifact codecs over the controlled runtime. One registered fixture policy selects one worker, cid, epoch and journal partition. The policy verifies five public BLS proofs of possession and binds the native voting namespace to the run, config, cid and epoch. Finalize, Notarize and Nullify are verified against the local member's participant index. Notarization, Nullification and Finalization use the native quorum verifier. Certification has no wire payload.

Selected storage operations retain the exact submitted buffer and the open instance identity. The ledger replays a bounded physical image per open instance. Its open record includes the initial bytes without granting durability. Observations persist ordinary writes, resize, removal and uncertainty before the typed result is released. A range barrier explicitly represents one successful SYNC write and both updates and covers the exact bytes. A full barrier must equal the replayed image and extent. Observed I/O IDs increase within each generation. The ledger reconstructs each new witness by parsing from logical offset zero and checks its current byte provenance. Existing sealed witnesses remain historical across later valid writes. Identical preserved bytes retain coverage. Writes, resizes and range sync update only what the native operation establishes. Failed attempted mutations and failed full barriers leave the section uncertain, including after a same-generation reopen. The root-only lost-sync-response diagnostic waits for actual physical completion, records `IoSyncResponseLost`, persists uncertainty and returns the injected typed Closed response. It models a lost response. Prior sealed history remains available. Protected history is checked before selected writes, shrink or remove dispatch. The adapter validates a same-open readback after the barrier and seals only complete canonical frames with coverage over every involved page's entire selected CRC-authenticated prefix and footer slot, plus durable extent through the end of the physical page. A buffered flush may end inside the next frame. A later barrier can witness earlier complete frames and leaves the incomplete frame unsealed. Strict snapshot recovery rejects the partial source frame.

Root owns both the native completion slot and the following evidence slot. It persists each new witness batch and synchronizes the committed head before releasing the native completion. Cancellation can leave the retained evidence operation to finish under cleanup and discard the protocol result. The diagnostic hooks cover native completion before ledger persistence, synced frame before head commitment and committed head before release. During a grant, an explicit exact-wire send request stages destination IDs for commit. Only a successful grant exposes those committed authorizations. Destination input is a nonempty set of known peers excluding self, and the fixture uses fixed priority true. Duplicate authorization requests preserve the route ID and bytes. Transport attempts and delivery are later integration concerns.

Offline recovery copies the old raw blob files, ledger and head into a fresh target's snapshot directory and validates the copies against a minimum committed prefix receipt kept outside the source ledger. A matching later committed descendant can recover after a lost Ack. It opens only the fresh target with the native Runner, writes accepted page bytes to new native blobs, and performs new whole-blob barriers. The target continues a byte-for-byte copy of the committed predecessor chain and binds the generation transition to its exact previous record hash. A pending transition cannot authorize submissions. Completion validates the same policy, all new full barriers, exact source-image hashes, new frame witnesses and protected history. Old witnesses, payload identities and route authorizations remain in the target ledger. Later target opens validate that intrinsic chain independently of the audit copies. The source-file hash uses H(`I1b/source-raw/v1`, [raw file including its V1 header]). Explicit recovered submissions require an anchored target ledger with a completed generation link. No recovery call supplies clock samples, protocol grants or a signing key.

## Ordered record representation

All list and struct fields below are encoded in declaration order by compact serde JSON. Arrays of bytes use JSON integer arrays. Named ledger variants use `{"entry":type,"value":object}`. Barriers use integer kind 1 for a range and 2 for a full blob. No host paths or wall time contribute to these fields.

| Structure | Fields in order |
| --- | --- |
| LedgerRecord | schema, sequence, previous_record_hash, identity, entries |
| LedgerIdentity | run_id, config_hash, worker |
| PolicySeal | config, registry_hash, namespace_hash, local_signer |
| WitnessConfig | run_id, config_hash, cid, worker, member, epoch, generation, partition, sections |
| BlobSeal | id, open_id, section, initial_bytes |
| IoObservation | io_id, change |
| ObservedChange | operation tag, then blob_id/offset/bytes for write, blob_id/len for resize, section for remove, section/reason for uncertainty |
| Barrier | blob_id, io_id, kind, offset, bytes, id |
| ArtifactWitness | id, blob_id, artifact_hash, barrier_ids, frame |
| DecodedArtifact | section, logical_offset, frame_bytes, artifact_bytes, pages, wire |
| PageEvidence | physical_offset, selected_slot, selected_length, bytes |
| NativeWire | channel, bytes |
| EligiblePayload | id, witness_id, wire_hash, wire |
| Submission | id, payload_id, destination |
| RecoveryLink | old_generation, old_head_hash, source_files |

Registry identity applies H(`I1b/voter-registry/v1`, parts) to ascending member records. The components for each member are the member u32 big-endian followed by canonical Ed25519 public key, BLS public key and verified PoP bytes, each framed separately. Native scheme participant order is the BiMap key order. The member number in the outbox identity is independent of that signer index.

The ledger uses the fixed head and frame formats in the frozen contract. The `generation_started` and `recovery` entries both encode the same RecoveryLink. They delimit the new copied-generation evidence. It validates committed physical bounds, the chain and typed entries. Anchored recovery requires a trusted minimum receipt covering the policy identity, sequence and record hash. It verifies the exact committed prefix and may accept a later descendant. A lower or conflicting head rejects, including an otherwise valid replacement of both older files. The unanchored inspection API validates the available chain and head. The complete-tail promotion option must be selected explicitly.

## Validation

```sh
./poc/scripts/bootstrap.sh fmt --manifest-path prototypes/i1b-runtime/Cargo.toml -- --check
./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline
mkdir -p /tmp/rotation-i1b-native-witness-correction
I1B_WITNESS_TEST_EVIDENCE=/tmp/rotation-i1b-native-witness-correction/builder NATIVE_WITNESS_VERIFY_ARTIFACTS=/tmp/rotation-i1b-native-witness-correction/verifier ./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline --no-fail-fast
```

The corrective run passed all 82 builder and prior runtime/storage tests and 16 of 17 unchanged native-witness verifier tests, including the three original counterexamples. The remaining verifier fixture counts every recovered witness as new. The corrected ledger retains 9 predecessor witnesses and adds 9 new ones. The verifier can compare the new segment after the generation transition and also assert the retained predecessor prefix.

Fresh builder artifacts are in `/tmp/rotation-i1b-native-witness-correction/builder`. Each seed matched across 0 and 2 ms wall pauses for its full trace, native raw section, ledger and head. Paired logical trace SHA-256 values for seeds 7, 42 and 99 were `52d7ebe39eb80d4a79498125613980e6f31df38d10413f70b7797f5166bb48c0`, `895bc19032414f9fa4e8a3393cd96f2fa3682dcfa7e3314d438af641d1f25c1c`, and `5e0b5e7313f7e5bd7b0dfb2f41ba1203e878ed03a21c9484ffb4aa71133c1909`, respectively.

Literal vectors below use the seed-7 exported paired fixture. Framing, chain, witness and outbox identities were recomputed from the raw capture with Python hashlib. Registry hash is the fixture's exported value.

| Item | SHA-256 identity in hex |
| --- | --- |
| Voting namespace (run 0x11 × 32, config 0x22 × 32, cid 0x33 × 32, epoch 1) | `20ba426b3ebbcaa269c1a058dced7633e6f223925fb02595e03c53bc0bee883f` |
| Registry | `849458372215a66a4a252c576f9615776e3057ee040e85d753635d63fd9c0657` |
| Blob, generation 1, section 1, open 1 | `f8f7a7393bb09e32dd8dbafcf056fc8cc0e4ee3a069cdd24c70e3f88dc5f2b7c` |
| Covered bytes hash | `9330966504a39524d9ff5b7c42bfca6d9c1172f3df25483c9ab572c8666eba88` |
| Range barrier | `0a6cf729c76735dfba58121cd179566bed6fe595c58e73dcf6eaa24444be09ff` |
| First artifact hash | `27a3593053d843672aede77c94d886533eedfadac33b6e69a66cfbbc5d66ac44` |
| First artifact witness | `32c33919e1184d49aec13b69fe0af8489fb0b209da8bcd6091f10096d48ebbad` |
| First eligible wire hash | `80516b1d39ab9696d2b1dc2d249cb4a5193c91bb6c4618a926a4ae4c331a41d3` |
| First eligible payload ID | `6a1b2de743ebce72bd878c20a49652f8413dead2803da0c9a801912b67946c1a` |
| Finalize to member 2 submission | `ab0dee9b099af3b0b14204fa32c2eb29beb2acba9264ad2cfd1c97f6dacf1e05` |
| Last record hash | `3e47a2dc4e507571384e8803f839866fb8b91b598834d44d20a8a5eb89f31e3b` |
| Head checksum | `a3e2300cf4cd7acee95daa510a205d80d975bd8943446e733922cce6f51e26ba` |

The builder tests cover W1–W12 at the bounded fixture interfaces. The ledger records modeled failure cuts in the real sync sequence, and these tests exercise Runner shutdown and artifact recovery. Independent acceptance and process-crash or power-loss evidence are separate. The slice leaves native timing restore, guarded signing, actual engine, p2p and two-worker integration pending. Resolver and relay use no new eligibility rules in this slice.

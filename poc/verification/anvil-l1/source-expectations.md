# Feature 09 independent source expectations — preparation v1

Prepared before candidate implementation or Builder explanation. Agent: named bft_verifier (configured GPT-6 Astra / xhigh). Worktree `/home/jy/.codex/worktrees/65ef/megaroation_poc`; source baseline `0a951c482a9712678ab9de34765b781b9c3c9f42`. No implementation inspected, build run, repository edit, or acceptance claim. The accepted ABI contract is not yet frozen; byte-vector values are deliberately pending that contract.

## Authority and reviewed sources

* Root AGENTS.md/DESIGN.md; regional-bft-conformance skill.
* v2 handoff/setup/design, especially design §§3.2, 4.1, 5.1, 7, 8.6, 9.3, 10.3.
* workflow and dependency-plan-v2; feature handoffs 09/11/12/13.
* v10 00_arch §§1.2, 4.2; 01 Authority §§3.1–3.2, 3.9–3.11; 06 Certifier §§3.2, 3.4, CERT-1/2/3; 07 Inbox §§2, 3.1–3.7; 08 DA §§3–4; 09 Finality §§3.3–3.4, 4.6–4.7, 5.1–5.5.
* derivation-preparation-v1 and materialized MegaETH batch-inbox Emergency Cursor Override/Events. MegaETH snapshot `a738f9b14c7880953d8cff8788db14e8e3e79124` is the stated provenance; its native six-field Cursor/TEE/EigenDA/activation is not automatically the PoC ABI.
* 03 preparation contract at `183430e71e5d61533f54a76b2d7d22c5021f1704`, §5 and D-L1/D-REPLAY/D-ABI. Its proposed encoding is not approved Solidity or GFR encoding.
* Regional BFT DESIGN read from `/mnt/reth/jy/megaroation_poc/regional_BFT_sim`; actual HEAD verified `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; Commonware pin 2026.10.0. Feature 09 does not alter BFT or test BLS quorum behavior. P0 stays 3 regions × 5 equal-weight replicas, quorum 4, 10-height windows.

## Independent transition expectations

### A. ControlAuthority

A1. A valid Force changes only accepted control/request state. Its event body preserves actual signed seq, target branch and static policy; act = including header timestamp + M_a. Inclusion, successful execution, simulated finalization, delivery and GFR activation are five different observations. Contract cannot decide whether the branch is current, HALT or actually stalled.

A2. A valid Resume requires governance authorization, sequence, static policy and target > current Inbox.height. It writes cursor with target height, zero frames_digest, bounded origin <= including block number, source RESUME(seq), and emits matching ResumeRecorded and CursorSet in one transaction. New Inbox state is immediate in that transaction; GFR alignment happens later at public act. Contract must not reject a target merely because it is <= G/D head or outside governance time budget: those are off-chain obligations.

A3. Force and Resume share a strictly increasing nonnegative sequence. Source does not require seq=previous+1. Initial sentinel/first legal sequence must be specified without accidental exclusion of source value zero, or documented as an explicit bounded PoC convention. Same seq on a new request after acceptance rejects; a larger skipped seq should accept if all other guards hold.

A4. req=(authenticated caller,rid). Same req plus same canonical request content returns original result and emits no second control or CursorSet; state does not re-execute even if cursor/seq has since changed. Same req with changed content rejects. Authenticated caller must be bound into signed request identity so another submitter cannot reuse authorization with a different req. Signature encoding itself need not be the request identity; define retry equality and exact bytes.

A5. Control signature covers chain ID, verifying contract, Force-versus-Resume type, seq, target branch/height, config, policy, and request caller/rid. A valid signature over any changed field is a different authorization. Unknown/zero signer, wrong-length signature, invalid recovery byte, high-s alternative (if profile excludes malleability), zero r/s and cryptographically wrong signer reject. Domain uses deployment chain and actual authority address, not caller-supplied unchecked values.

A6. Atomicity negatives must observe no partial cursor, control seq, request entry or logs after failure. Invalid outer signature/target tests only prechecks; also demonstrate an internal Override failure or test-only revert after inner state mutation to prove no half-commit. No production test hook may bypass real authorization.

### B. BatchInbox / DA / attestation

B1. Genesis cursor is explicit config: full height/origin/zero frames_digest/source GENESIS, with independent read metadata. Deployment receipt/genesis checkpoint provenance must be specified; never fabricate (0,0,0) as a real event when none exists.

B2. Successful SubmitBatch requires actual fixed authorized submitter, exact equality of all prev fields, real Certifier signature on full BatchAttestation, publicly determined DA certificate check, and profile consistency. Success produces exactly one CursorAdvanced and state transition. Wrong caller or any prev height/origin/frames/source byte fails, even if the height matches.

B3. The attestation binds chain, Inbox address, complete prev and next cursor, commitment, hash of exact DA certificate bytes, completed range and full gpos. written_at is not a signed cursor field; Certifier cannot know future inclusion. Source GLOBAL(gpos) must agree exactly with attested gpos. Preserve source tags; GENESIS, GLOBAL(empty bytes) and RESUME(0) must not alias.

B4. Whole-frame P0 is an explicit simplification: accept only the frozen profile for a closed frame covering 1–2 contiguous new heights; first=prev.height+1, last=next.height, frames_digest=0. Unsupported partial/open frame behavior must reject clearly; do not silently claim native partial-frame coverage. Origins cannot refer to future L1 heights. Any stronger next-origin monotonicity/range rule needs contract declaration.

B5. Mock DA must have a real fixed-key signature/check over a domain-bound commitment/length/config (exact frozen fields TBD), not a bool flag. A signer-approved receipt does not prove payload availability by itself. Certifier/provider opens payload commitment; on-chain validation can authenticate receipt but cannot fetch payload. Empty payload is not absence. Accepted bytes later missing or mismatched remain WAIT for derivation; 09 must not fabricate a D result or ResumeCut.

B6. Old attestation signed before a successful Override fails afterward due to full prev mismatch. A same-request successful retry is idempotent, not a second accepted event; changed content under same req rejects. Distinguish stale new submission from retry of a previously accepted request.

B7. Changing each scalar, each source variant/value, gpos byte, commitment, DA certificate byte/hash, chain/address/config/range must break the original valid attestation or a binding consistency guard. Genuine crypto is exercised with fixed public vectors and cryptographic verification, not a mock true/false verifier.

### C. Exact L1 position and history

C1. Per 00_arch §1.2, p=(block,tx,log) uses transaction-local log ordinal. RPC logIndex is generally block-global; enumerate all receipt logs for transaction-local index, retaining actual RPC global index as evidence. Do not enumerate only filtered contract logs. Verify receipt blockHash/blockNumber/transactionHash/index and log address/topics/removed status against the complete block transaction list.

C2. CursorSet's event position and matching ResumeRecorded's position are distinct. Pair by same tx, configured addresses, exact seq/source/target/policy context. Never pair merely by block height, adjacency across transactions, latest seq or final cursor. Missing, duplicate or conflicting pair is WAIT for missing material or REJECT for confirmed inconsistency; no partial derived control escapes.

C3. Full successful receipt logs are necessary for pairing and local-index derivation. Failed transactions can be present in finalized blocks but provide no accepted control/Inbox input. A function call, successful eth_call, submitted calldata, a hash, or a positive send RPC result is not inclusion evidence.

C4. History queries are as-of-end-of-specific-block, separately gated by that reader's delivered finalized prefix. Same-block intermediate events must remain separately obtainable for exact cuts, even though eth_call(block) returns block-final state. Later cursor changes never overwrite stored earlier event pairs or historical reads. Missing history yields WAIT rather than falling back to latest.

C5. Metadata coverage and complete control/Inbox-log coverage are distinct. A header or empty eth_getLogs response alone does not prove complete coverage. At least validate all transaction receipts for each delivered block (including empty-block evidence) and canonical parent/hash chain before marking whole-block coverage. Finalization never rewinds.

C6. 03 owns canonical public L1Position/FinalizedPrefix and all ReplayPosition/ResumeCut declarations. 09 may expose ABI wire bytes and mapping contract only. gpos is opaque canonical bytes supplied by 03/07; 09 hashes all bytes and preserves them, but cannot validate full GFR semantics. Tests using opaque marker bytes are explicitly codec-boundary fixtures, not GFR replay evidence.

### D. Explicit driver and independent readers

D1. Only explicit authorized inputs change tick, submit queue, mine, finalize or reader delivery. No wall-clock automining, interval mining, wall-driven finalize or driver background latest access. Clock/transport live wiring remains full-feature gate; independent scripted ticks only establish component behavior.

D2. Freeze tick unit (v2 microseconds), genesis timestamp, L1 slot and integer-second mapping; test tick just before/equal/after boundary, duplicate/backward tick, mine before eligible slot, duplicate slot and arithmetic overflow. Blocks have strictly increasing timestamps; neither cast truncation nor Anvil auto-adjustment may silently pick another timestamp.

D3. Submission leaves chain head unchanged until explicit mine. Wall pauses leave chain head, simulated finalized frontier and both reader frontiers unchanged. Tick advance itself does not mine unless frozen input semantics explicitly include that action. Finalize cannot exceed included head or move backward; deliver cannot exceed finalized.

D4. A/B readers have independent delivery states. Later block before missing predecessor does not advance visible prefix; immutable duplicates are idempotent, conflicts reject preserving prior state. Available disk/RPC future material remains inaccessible through reader queries. At equal delivered position A/B values match despite different arrival schedules.

D5. Keep raw commands, RPC transcript, input ticks, headers, transactions, receipts, state reads, expected comparison, tool hashes and candidate SHA. Real Anvil/EVM with controlled component script must be demonstrated; pure model/Forge tests alone are not that evidence. Heavy commands use global flock and CARGO_BUILD_JOBS=2. Only own temporary port/PID. No new global configuration.

## Concrete source-level vectors (ABI-independent)

V1: M_a=6 seconds; included block10 timestamp100; Force seq4 target7 → act106. Finalized/delivered at arbitrary later ticks do not change act. Force seq9 in same block10 has act106 and sorts by actual transaction/local-log position, not sequence numeric or delivery time. Seq4 then seq3 rejects; seq4 then seq9 is valid monotonicity.

V2: cursor=(height5,origin3,frames0,GLOBAL(gposA)); Resume(seq10,target5) and target4 reject, target8 passes cursor guard even if independent D head10 (D unchanged by contract); origin must be <= including height. The contract makes no D/G assertion. Resume target8 is followed by stale batch with prev height5 → reject; subsequent batch prev exact RESUME(10) may pass with valid attestation.

V3: same block has transaction0: one Force log at local0/global0; transaction1: CursorSet local0/global1 then ResumeRecorded local1/global2; transaction2: CursorSet local0/global3 then ResumeRecorded local1/global4. Expected resume positions are (L,1,1),(L,2,1), distinct although local ordinal both1; set positions are (L,1,0),(L,2,0). A distracting unrelated log before the pair increases local ordinals and must be included in conversion. This is an abstract input vector, not a claim actual contract emits distracting logs.

V4: reader A delivered complete blocks0..4; B complete0..2, has buffered4 but no3. A query4 ready, B query3/4 WAIT. Deliver3 to B then4 becomes visible, both equal at4. Header4 without receipt coverage must not produce control_coverage4. Retry block3 identical no change; conflicting block3 rejects.

V5: successful request (callerA,rid7,payloadP) accepted at block3; retry same request at block5 returns reference/result of original acceptance, no new CursorAdvanced/ForceRecorded/CursorSet and original written_at remains. (callerA,rid7,payloadQ) rejects. (callerB,rid7,payloadP) has separate req identity but cannot reuse callerA-bound control signature.

## Items the lead/Docer must freeze before byte vectors

Exact ABI structs/function signatures/events, EIP-712 type strings/name/version/config binding and hashing order; seq sentinel; req equality scope and result references; genesis cursor and original-write provenance; resume origin input or deterministic rule; zero/unused tagged-source fields and gpos maximum length; DA signed fields and certificate encoding; signature canonicality; finality/timestamp policy; receipt proof/trust boundary and block-local/global-log conversion; explicit partial-frame unsupported boundary.

Engineering choices may be frozen by 09 lead within scope. Do not make future GFR/Derivation semantics or 03-owned canonical types local to 09. Once ABI freezes, independent vector generator will use specification-only encoding and emit scalar preimages, hashes, addresses, deterministic signatures and field-mutation rejection fixtures before Builder source is read. A fixed literal vector plus independent crypto implementation/library should cross-check EVM outcomes; an expected hash computed by candidate's own helper is insufficient.

## Current classification

Source expectation preparation: READY for contract review. Byte/signature vectors: pending ABI freeze. Solidity/EVM/driver tests: NOT RUN. Component acceptance and full feature 09 acceptance: INCONCLUSIVE. Pending real 02 Clock/controlled-output and 03 shared-object gateway wiring remains a full-feature gate even after component PASS.

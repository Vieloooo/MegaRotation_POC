# Regular integration preparation — not an implementation contract

Date: 2026-11-15. Status: read-only source preparation; implementation is gated on independently verified native recovery R1–R18 and its same-revision simulations. This document does not mark I1b, I2, regular handoff or any BFT liveness property passed.

Sources: pinned `formalization_v10/poc_design_zh_v2.md` SHA-256 `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`, v10 `00` §§1.3, 2.3, 3.2–3.4, `02` §§3.2–3.10 and 4.3–4.9, `03` §§3.4–3.10 and 4.1–4.4, `09` §§3.1–3.4 and 4.1–4.5, and `11` §§1.1–1.4 and 2.1–2.2. Native code mapping below is from verified `fec7ade1675cb60dd16a16fb28ebe81e3f76f508`, inspected with git show, not the changing recovery implementation.

## Two increments

REG-A expands to fifteen real OS workers and three independent five-member, quorum-four registries. One selected R0 session runs five actual native engines; the other ten workers register and exercise their own Clock/readiness without signing for R0. There are no scripted vote keys. Add slot-aware Build/Accept, light execution and finite public L1-input gateways. This establishes the process/admission boundary, not candidate concurrency or global regular handoff.

REG-B then adds candidate lifecycle/parking, Manager, persistent Publisher, SimEspresso and regular-only GFR. Exercise `(0,0)→(0,1)→(0,2)`, regions 0/1/2 and fixed windows `[1,10]`, `[11,20]`, `[21,30]`. Completion includes the first selected R2 result entering G. The full first-stage task still includes Web, Anvil governance/Inbox, Force, D and RESUME after these increments.

## Proposed identity and receipt rules

The lead accepts the following direction, subject to freezing exact encodings and interface vectors before implementation:

* PhysicalBlock is `(ph,h,ts,os,root,Txs)`; its canonical hash is the native Simplex payload digest. Native view is not a physical-block field or a height. Genesis payload is the validated anchor identity. Native parent view and digest must bind the exact physical parent or anchor; matching height alone is insufficient.
* `B.root` is the execution state root before the current block's Begin/DEP, after intervening padding. Native comparison hashes used later by G/D are a separate encoding of actual parent, metadata, DEP, Kept and execution-after root.
* Receipt key is `(cid,epoch,view,native_parent_view,H(B))`. Header ingress, complete-body ingress, authority validation state and validation-completion time are distinct facts. An authorized header/body pair establishes complete receipt at the later of its two original ingress times. Earliest applicable complete receipt is retained across callbacks, retries and process recovery.
* DIRECT_LEADER authority requires authenticated transport from the elected leader, participant/signer-index equality and decoded context. BLS status may still be PENDING, matching the pinned batcher's forwarding boundary. Forwarded headers instead require verified elected-leader attestation or valid native quorum certificate over the exact subject. Arbitrary peer headers/body alone cannot authorize or poison a legitimate proposal receipt. Body suppliers need not be leaders; content hash binds their data.
* Self proposal uses LOCAL_BUILD authority at the instant complete B exists, followed by local Accept before returning the payload. No circular requirement to create a native vote first.
* Delayed validation uses original candidate ingress time. Before sealing temporal Accept/Reject, resolve earlier admitted candidate evidence that could establish an earlier receipt. Input/authentication processing must be bounded, with cheap source/context checks before admission. Overflow before ingress is a recorded drop; exceeding a declared processing bound leaves admission pending and defeats the timing premise. Do not wait for quorum batching to verify an individual forwarded leader attestation.
* Preserve exact durable local Accept before self-propose return, verify(true) and local certify(true). WAIT(missing/time) retains a live response producer. Permanent temporal REJECT is local abstention, not native false. Historical FinalCert verification does not rerun observer-current-time Accept; it validates public proof/body/parent binding and relies on the audited earlier-honest-Accept argument.

Exact proposal-bundle bytes, state/physical/native hash domains and vectors, bounded authentication service, and monitored view/input-delay definitions remain unresolved integration choices. Do not infer permission to choose them in a builder.

## Proposed bounded fixture

Use h0=0, L2 genesis timestamp10s, slot interval1s, session `(0,0)` window1–10/start11s/stop21s, reference simulation start10s. Per-region offsets are −1000/−500/0/+500/+1000µs. Preserve lease100ms, refresh20ms and Clock RPC250/250µs. Intra-region delay20ms, later cross-region100ms; modeled build/verify/certify work5ms each and synchronous crypto explicitly zero virtual cost. T_win=200ms. Healthy readiness may use0/20/40/60/80ms; the exact v2 S1 vector0/20/100/500/1500ms and S2 cases are additionally required.

Proposed normal leader/certification timeouts are1.2/1.5s, with retry100ms. They replace the Fresh physical-count fixture's200/400ms only in the new profile. Monitor separate normal-path premises:

| Term | Bound |
| --- | --- |
| Next-slot wait plus clock difference W | 1002ms |
| Effective active view/readiness skew V | 40ms |
| Build plus durable self-Accept B | 10ms |
| Complete one-hop delivery Δ | 20ms |
| Additional input/authentication I | 40ms |
| Follower verification A / local certification C | 5ms each |

These give1112ms for complete authorized proposal availability,1162ms for local certification, and1202ms for final-proof delivery. V is not implied by clock error or arbitrary Enter skew. Missing data, quotas, overflow or delays outside the envelope invalidate this bounded-success premise. S1/S2 need their own fixed quorum/common-participation and synchronization interval. Initial session flooring retains actual Enter timestamps; original restored deadlines/retry anchors never shift. These proposed sums require actual native timer tests and are not a liveness proof.

Under v2 §10.2, deposits are fixed `dep[h]` inputs bound to config; attributes derive from metadata. This does not validate real L1 deposit allocation by origin. A finite L1 fixture can expose metadata and complete empty control-log coverage for heights0–30, timestamp2ℓ seconds, finalized at timestamp+1.5s, normally delivered contiguously by timestamp+2s. Genesis origin3, T_dep5s and T_drift15s are proposed. Each reader gateway exposes only actually delivered finalized prefixes; future disk rows are inaccessible and out-of-range is a fixture failure. Light execution must include a successful state-changing transfer and well-formed invalid-nonce/insufficient-balance transactions that Apply skips. Malformed BasicTx input rejects. A revert opcode can be omitted; adding one requires explicit execution/Kept vectors.

## Verified code migration boundary

All line anchors in this table refer to fec7ade, not current workspace line numbers.

| Existing file/anchor under poc/prototypes/i1b-runtime | Boundary to preserve or extend |
| --- | --- |
| src/driver.rs:299; src/context.rs:24 | One NodeClock and worker-wide driver already own cid-indexed sessions/jobs/timers/RNG roles. Keep one Clock, grant sequence and controlled executor. |
| src/driver.rs:282,801 | Session has stop/retired only; add actual pause parking, preserving jobs/wakes/state. Close drops work and is not Pause. |
| src/native_replica.rs:44,363 | One config/signer/network/archive/receiver set; introduce worker-owned cid registry and cid-directed dispatch. |
| src/native_application.rs:153,236 | Archive is naturally per cid; active=false/close discards pending work. Preserve operation identities, original timing and produced-but-unconsumed results while parked. |
| src/native_config.rs:33; src/native_replica.rs:125 | Fixed i1b-native-voter partition cannot host multiple candidates. Allocate exclusive stable cid partitions. |
| src/storage.rs:247 | Audit cid does not namespace raw filesystem partition names. Preserve blob-owner cid separately from the executing task and enforce access ownership. |
| src/driver.rs:350,496 | One witness policy/partition, configured before first grant. Add controlled per-cid policy/ledger ownership under the same serialized worker I/O coordinator. |
| src/driver.rs:215 | Staged route tuple drops cid; carry cid/policy through authorization, lookup and receipt production. |
| src/native_scheme.rs:24,149 | Signer cache assumes one cid, views1–8. Keep caches per cid and explicitly version profile/view/journal bounds. |
| src/native_step_journal.rs:25; proto/native.proto:96 | One root-step journal remains worker-wide; bind all affected cid recovery/witness heads into one coherent Ack. |
| src/native_replica.rs:551; src/native_hub.rs:57; src/native_config.rs:45,324 | Two actual workers and two offsets are fixture assumptions. Expand registration/launch/readiness to15 without starting/signing an unauthorized region. |
| src/native_network.rs:82 | Local member0–4 addressing needs explicit global pid↔regional member↔sorted signer-index mapping; committee remains n5. |

The current body-fixture store uses immutable digest-addressed bytes which bind cid (`native_application.rs:42,585`). A shared content store may remain possible if encoding, ownership, retention and recovery are explicit; Archive/execution state remains separate. One witness recovery helper is not recovery of every cid/body/root state on a worker.

Pause/resume must retain pending timers' cid/owner/absolute deadline/fired state (`driver.rs:1705`) and application work identities (`native_application.rs:236`). Before releasing parked work, use the same NodeClock and recheck permanent termination/original stop (`driver.rs:763,1594`). Expired parked instances terminate without receiving a new window.

## Next acceptance scope

REG-A covers fifteen genuine processes, quorum/identity/context negatives, Accept5a–5d exact boundaries, first complete receipt under delivery/retry/restart, next-slot and WAIT(time) timers, missing L1/body/parent, physical gaps and covered padding, Close/stop during work, and fixed-quorum participation versus3-ready/3–2-split controls.

REG-B covers both full-window handoff and SessionEnd fallback (h9 anchor retains successor window11–20; h11 admission may cover padding10), q_avail from public proof plus complete ancestry bodies, SessionEnd-before-Espresso at equal keys, WAIT for missing public watermark/L1 coverage, cached candidates' original q_avail versus current admission key/per-height Expire, and independent readers agreeing at equal replay position. Proposed Espresso cadence250ms, proposal timestamp, finalization+100ms, A/B delivery20/150ms; T_pub200ms, T_grace3s, M_a6s and W_exp30s need a frozen profile.

Before any implementation, the lead must resolve the remaining encodings, assign exact paths and API ownership, and freeze separate contracts. Worker-wide Clock/executor/transport/root-step ownership; cid-owned engine/signer/journal/policy/archive/pending operations; Manager-owned lifecycle are the intended separation.

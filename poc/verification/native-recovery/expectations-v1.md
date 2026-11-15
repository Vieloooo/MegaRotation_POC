# Independent native recovery expectations v1

Prepared by `/root/i1_verifier` from the primary source and frozen task contract through amendments1–6 and the signer-cache hash clarification at35abf79. Materialized by the lead without treating implementation progress as acceptance. The verifier had not inspected, built or executed the changing recovery candidate. These source-derived contract corrections are not candidate findings.

Contract: `poc/docs/contracts/task-contract-native-recovery-v1.md`. Commonware2026.10.0, upstream92a2f68a0e131c8b69de41e84aec37da6280f212. Main fixture: n5/q4, two actual workers plus explicitly scripted peers, Dynamic rotating elector and maximum physical height2. Overall I1b remains conditional on a named candidate, independent execution, integrated checks and same-revision simulation.

## Reconstruction oracle

Validated history/provenance → native replay → applied proposal facts → owned callback/actor phases → original timing → durable shared startup release.

A forwarded proposal is not an applied voter transition. Only a bound successful native voter application changes the restored projection. Retain unconsumed ingress in ordered custody for dispatch after release; do not resurrect coalesced messages. An ordinary completed cut's empty-mailbox premise must follow actual controlled quiescence, prompt application dispatch and drained I/O, not merely an absence of ready tasks.

## Cases

| ID | Fixture and independent expected behavior |
| --- | --- |
| R1 | Commit actual voter-selected Unverified proposal, pending Verify, missing body, no voter Artifact. New PID/incarnation before expiry restores exact proposal/first receipt/operation with live producer and receiver. Select original certification deadline, not leader deadline. Exercise authenticated absent-partition recovery without invented old file/barrier. |
| R2 | Separate duplicate/conflicting input references and native BLS validation from app verification. PENDING never becomes VERIFIED merely on restoration. Test applied, forwarded-unconsumed and coalesced cases at attainable cuts. |
| R3 | Restart claimed build before original5ms due. Preserve operation ID, WorkBinding, generation and captured parent; do not invoke ordinary queue allocation. Exactly one authorized original-due completion. |
| R4 | Cut after work-ready/before body sync and after body sync/before offer. Restore helper from bound plain evidence and exact body/digest. Readability alone is not durability; no positive native result before required sync. |
| R5 | Cut after durable OFFERED but before send/consumption for propose and verify. Retained producer/fresh receiver offers exact stored outcome once after startup release. A dropped-sender empty receiver is a failure. |
| R6 | Restore CONSUMED with and without corresponding Artifact, preserving applied/discarded transition and no redelivery. A no-corresponding-Artifact positive fixture is successful Verify after an existing same-view local Nullify: verification may settle while notarize remains prohibited. Recognize own-vote replay already supplying Verified without another callback/sign. |
| R7 | Native cancellation is recorded when its receiver is abandoned, not when the helper later notices. Stale completion after restart stays discarded. Exited-view propose/verify may be cancelled while exited-view certification can legitimately continue; do not cancel all old-view operation kinds. |
| R8 | Past-due work needs authenticated RecoveryWorkCompleted after release; preserve original due and reconnect generation. Future-due remains unchanged. Unrelated later cuts cannot invalidate an otherwise authorized restored operation; stale physical bindings/generations reject. |
| R9 | Restart before/at/after leader, certification and Nullify-retry deadlines, with explicit latch fixture. Compare original absolutes/arm/cause. Expired timers become eligible after fence, still requiring granted timer delivery. Never rebase to recovery-now+duration. |
| R10 | Dedicated optional-floor profile with staggered Enter and multiple genuinely new views before floor. Preserve true entered-at values; new normal deadlines use floor, restored deadlines/latches/retries do not. |
| R11 | Execute all named durability cuts, validate prerequisite frontiers/external floors, and distinguish pending-root completion from exact cached Ack. Test owned-I/O custody under cancellation, recovered-frame capacity/+1 overflow and incarnation varint growth. |
| R12 | Reject coherent rollback below trusted receipt, conflicting descendants, missing required evidence/records, wrong context and corrupt complete frames. Reject deleted protected partition disguised as empty, absent witness/root/head and false-empty manifest. Missing ordinary R1 input remains positive pending recovery. |
| R13 | Hold each readiness participant while staging input/results. No normal processing, private-key invocation, helper completion or publication before durable shared StartupReleased. Preserve staged ingress; pending body work must not deadlock readiness. |
| R14 | Restart after Close, original local-stop equality and old-lease expiry. New calibration cannot revive cid. Restore anti-replay frontiers but no usable old calibration; reject old replies/request-ID reuse. Historical data may remain available without a live engine claim. |
| R15 | Retain exact durable signatures/cache/authorizations/attempt bytes, including compatible prepared cache with absent Artifact. Legitimate retry is CACHE_HIT without copying prospective flags; genuinely lost pre-durability invocation is not falsely called cached. Repeated recovery has no repeated logical effects. |
| R16 | Execute actual native RESOLVER request/response, drop response, restart requester, complete repair. Restore role RNG before reconstruction draw and persist post-draw state before request release. Cancel old request generation/stale associations. Initial ID0 valid; outgoing Request IDMAX permanently fails before release/continued wrap; echoed response IDs exempt. |
| R17 | Separate Scheduled/rotating fixtures create WAITING/CLOSED through native handlers. Preserve exact request/parent. Waiting releases once on exact-parent certification/finality; native nullification/supersession cancellation remains. Closed retains forfeiture/timeout. No Stage or substitution for main Dynamic profile. |
| R18 | Repeat ordering-sensitive cases at seeds7/42/99, pauses0/2ms and reversed start. Compare complete logical cuts/deadlines/signatures/outcomes/stored bytes. Rehashed mutants reject false provenance, early offer/consume/publication, fabricated receiver ownership and altered deadlines. Actual OS restart and executed resolver repair are mandatory. |

## Cutpoint distinctions

| Boundary | Oracle |
| --- | --- |
| GrantIntent synced, before input | Exact input durable, protocol has not consumed it. |
| Proposal fact/claim synced | Retain actual voter application and receiver-installation distinction; ownership restored before readiness. |
| Work ready/body sync | Helper progress is not native OFFERED. |
| OFFERED synced, before send | Exact outcome can be reoffered, no invented consumption. |
| Native consumption, before preparation sync | Distinguish durable CONSUMED from lost volatile progress; successful sender delivery alone is not native consumption. |
| Preparation, before native barrier | Adopt compatible durable cache/counter knowledge, not prospective issuance/decision flags. |
| Native barrier, before final cut | Reconcile authorized preparation with actual native evidence; do not hide newer native history by selecting an arbitrary older cut. |
| Final cut, before root record | Use durable OUTPUT_INTENTS/equivalent exact template; explicitly finish or reconcile pending transaction. No completed cached old Ack exists yet. |
| Root record synced, before root head | Promote only complete verified authorized descendant, with required re-synchronization. |
| Root head, before Ack | RecoveredAck returns exact original Ack bytes/binding/hashes/attempts. |
| Actor restore before timing; timing before release | No partial restoration escapes startup fence. |

Already-applied recovered Acks have zero new effects. Lost Acks may adopt original sends once, while old physical clock/timer/work obligations are cancelled/rebound separately; old resolver requests stay cancelled. New incarnations retain clock request/sample frontiers and require new calibration.

Frame checks use the complete prospective outer protobuf frame, both original hash fields populated and maximal admitted future incarnation width. Oversize rejection leaves root journal/head/cache unchanged without falsely promising rollback of earlier native/auxiliary work. Signer cache hashing omits its own hash field.

Fresh regression covers canonical root-step JSON and resulting wire hashes, not protobuf omission alone. Independently inspect the three authorized old fixture defaults and the two field-specific skip-None attributes. Checking uses raw WAL, auxiliary/native/witness/root records and actual body evidence, never an implementation-produced valid=true field.

## Execution handoff requirements

Record candidate commit plus dirty state/source and patch hashes, exact command/profile/seeds, fault checkpoint, original/new PID/incarnation/generation, trusted floors, all raw output and expected ordering. Keep infrastructure errors, premise failures and protocol failures separate. Retain malicious/rehashed mutant inputs and rejection outputs. R1 in-memory reconstruction or fresh healthy traces do not discharge process recovery. Repeat affected checks on the integrated revision before formal simulations.

---
description: Emergency recovery — how the sequencer, publisher, and batcher return the chain to certified settlement after a height has settled as a deposit-only block.
spec: TBD
---

# Emergency Recovery

{% hint style="warning" %}
This page describes unstable behavior.
Block certification is not activated on any MegaETH network.
The recovery specification remains incomplete; unresolved questions are listed below.
{% endhint %}

Every mechanism elsewhere in this section exists to keep a height from settling as anything other than its selected content.
This page is the register of what happens when some part of that machinery fails: what triggers each failure, how it is detected, which escape hatch applies to it, what the protocol requires of each role while it is being worked through, and what residue it leaves behind.

## Motivation

The failures are not alike in severity.
Some cost only latency if recovery completes before the affected heights' submission deadlines.
One can lose content across a range of heights, and is the case the rest of the section's mechanisms are shaped to make rare.
Some cases have no remedy this specification defines; they are listed here and collected again under [What Has No Recovery Path](#what-has-no-recovery-path).

## Recovery Outcomes

**Lossless recovery** means that each affected height settles the ordinary transaction list of its selected record at that same height, byte-identically and in the same order, without content being discarded because of the failure.
It does not require unchanged settlement latency, block hashes, or execution results; an ordinary transaction that executes and reverts is still included.
Resubmitting a dropped transaction at a later height does not restore the original height's selected content.

Lossless recovery requires the affected blocks to remain execution-valid and their data to reach derivation through complete, available channels whose L1 inclusion satisfies the applicable sequencing-window and channel-timeout rules.
Restarting publication, registering a key, restoring a provider, or becoming eligible to submit does not by itself establish that outcome.
Successful delivery also assumes that an opener completes its channel within the [channel timeout](batch-inbox.md#acceptance), since no party, its opener included, may reset it before then; a channel that times out costs its heights nothing, because the inbox refuses the late continuation and the next opener resubmits them.
The [recommendation to submit a complete channel at once](batch-inbox.md#rationale) avoids that partial-channel race when followed, but is not enforced by the protocol.
If a height settles deposit-only and thereby loses its selected ordinary content, restoring the failed service does not repair that height; recovery proceeds under [S4](#s4-a-height-settles-as-a-deposit-only-block).

## The Three Escape Hatches

Three mechanisms are available when the normal settlement path stops working.
They are ordered by cost, and a scenario should be worked through the cheapest one that applies to it.

- **Submission by an alternative registered batcher.**
  [Recovery by an Alternative Batcher](batch-inbox.md#recovery-by-an-alternative-batcher) lets a second registered party assemble and submit the same batch from public data once the [exclusive window](batch-inbox.md#exclusive-window) has expired.
  It can recover a stalled settlement path using the public [selected record sequence](block-records.md#canonical-record-sequence), subject to the [conditions for preserving content](#recovery-outcomes).
- **The emergency cursor override.**
  The [emergency cursor override](batch-inbox.md#emergency-cursor-override) lets the `admin` assert a cursor position the contract's own state cannot reach, bypassing every acceptance check.
  It is advance-only: it moves the cursor past a position it is stuck at, and its ordered `CursorSet` event triggers [deposit-only gap filling](../rollup/derivation.md#emergency-cursor-overrides) through the supplied height without waiting for sequencing-window expiry.
  It preserves the established safe prefix and cannot restore abandoned ordinary content or replace finalized history.
- **Derivation's [deposit-only fallback](https://specs.optimism.io/protocol/derivation.html#batch-queue).**
  When a height's batch never arrives, or arrives only to be rejected, the pipeline fills that height with a deposit-only block — the pre-existing OP Stack behavior, modified by the [derivation pipeline deltas](derivation-pipeline.md), and not a mechanism this specification adds.
  It preserves liveness at the cost of the height's content.

The third is not a recovery so much as the failure being absorbed: nothing is repaired, the chain simply continues without the affected heights' transactions.
Some scenarios below have **no** hatch at all — no hatch recovers them, and none of the three can be pressed into service.
They are listed alongside the others rather than omitted, so that the register reads as a statement of coverage and not as an advertisement for it.

## Why a Height Cannot Be Repaired in Place

Once a height `N` settles as a deposit-only block while the [selected record sequence](block-records.md#canonical-record-sequence) holds a different, permanently fixed content for it, the height cannot be repaired: its record is final on Espresso and the block that settled is final on L1.
Dropping its ordinary transactions can leave sender nonces behind those expected by later records.
A later batch containing a transaction with an invalid nonce fails as a whole and is replaced by a deposit-only block, potentially leaving more senders' nonces behind.
This can cause a cascade, but does not make every later batch invalid: transactions from independent senders may still execute against the derived state.
Under [Δ6, keeping the channel after an engine rejection](derivation-pipeline.md#delta-6-keep-the-channel-after-an-engine-rejection), derivation retains the remaining batches and processes each against its actual derived parent; recovery cannot assume that the entire remaining range becomes deposit-only.

This does not violate [the Certification Claim](overview.md#the-certification-claim).
Each affected height settles as a deposit-only block, which is one of the Claim's two permitted outcomes; the Claim does not promise that selected content settles, only that nothing else does.

## Scenarios

Each subsection below states one failure, in the same order every time: the trigger, how it is detected, which hatch applies, what the protocol requires of each role, and what residue remains.

### S1: The Batcher Stops Submitting While Records Continue

**Trigger.** The primary batcher stops submitting, while the publisher goes on publishing records for every block the sequencer produces.
Settlement stalls; certification does not, because [publication is unconditional on settlement progress](block-records.md#publication).

**Detection.** [`cursor.epoch_num`](certifier.md#cursor) falls further behind the L1 head than [`SETTLEMENT_LAG_BOUND`](batch-inbox.md#constants), the bound the gap stays within while the whole pipeline is functioning.
The cursor is public L1 state, so any observer computes this.

**Hatch.** An [alternative batcher](batch-inbox.md#recovery-by-an-alternative-batcher), once the [exclusive window](batch-inbox.md#exclusive-window) expires — which it does, because the window is measured from the same `cursor.epoch_num` that has stopped advancing.

**Roles.** The alternative batcher builds its channel from the selected record sequence beginning at `cursor.block_number + 1`, on the terms [Recovery by an Alternative Batcher](batch-inbox.md#recovery-by-an-alternative-batcher) specifies, and is not required to reproduce the primary's packing.
The publisher continues publishing normally; the sequencer continues producing normally.

**Residue.** No selected content is lost if recovery meets the [lossless-recovery conditions](#recovery-outcomes).
The [exclusive-window margin](batch-inbox.md#exclusive-window) permits an alternative batcher to act before the sequencing deadline when its stated origin relation holds; selected-record and derived-origin divergence can invalidate that premise.
Even when it holds, the margin does not guarantee that recovery finishes before the deadline.
Heights whose content is lost to the deposit-only fallback proceed to S4.
If the sequencing window expires before any batcher acts, the deposit-only fallback fills the expired heights and may advance the derived origins ahead of the records the publisher went on publishing.
Later records are checked against their actual derived parents: an origin below the parent's triggers [Δ5 substitution](derivation-pipeline.md#delta-5-substitute-instead-of-dropping) when the batch reaches that check, while content passing all applicable checks can still settle, as [S4](#s4-a-height-settles-as-a-deposit-only-block) describes.

### S2: The Primary Stalls Part-Way Through a Channel

**Trigger.** The primary stops after submitting some but not all frames of a channel.
`cursor.frames_digest` is non-zero, and the stalled channel cannot be continued: a continuation frame would have to extend the compressed stream the earlier frames began, and the compressor state those frames left behind exists only in the primary's process; the frames themselves are on EigenDA, but they are not enough.

**Detection.** As [S1](#s1-the-batcher-stops-submitting-while-records-continue) — the same lag against `SETTLEMENT_LAG_BOUND` — with the non-zero `frames_digest` in the public cursor distinguishing this case from it.

**Hatch.** An alternative batcher abandons the channel and repacks from frame 0.
This is a path the inbox and the certifier already provide rather than an exception to them.
Once the stalled channel has timed out, the [acceptance check](batch-inbox.md#acceptance) expects a `prev_cursor` whose `frames_digest` is zero.
The frame-continuity check in the [verification procedure](certifier.md#verification-procedure) accepts a submission opening a channel at frame 0 with an empty prefix witness against that zeroed cursor.
The [cursor advance](certifier.md#advancing-the-cursor) resumes from `cursor.block_number`, which intermediate frames never moved.

**Roles.** The alternative batcher waits for the stalled channel to time out, reads the on-chain cursor and uses it as `prev_cursor` with `frames_digest` set to zero, and submits a payload beginning with frame 0 of a channel it built itself whose first L2 block is `cursor.block_number + 1`, as [Recovery by an Alternative Batcher](batch-inbox.md#recovery-by-an-alternative-batcher) requires.
A primary that restarts without the compressor state of its own open channel takes the same path: it cannot replace that channel early, so it waits for the channel to time out and then opens a fresh one.
No other role changes behavior.

**Residue.** No selected content is lost if the replacement channel meets the [lossless-recovery conditions](#recovery-outcomes); otherwise affected heights may proceed to S4.
The orphaned frames are discarded by derivation when the next channel's first frame arrives, or at the [channel timeout](https://specs.optimism.io/protocol/derivation.html#overview), exactly as Holocene's channel bank always has.
A frame of the stalled channel that the primary itself submits after that timeout is refused by the [inbox](batch-inbox.md#acceptance), so a late continuation cannot advance the cursor past a channel derivation has discarded.

### S3: An L1 Reorganization

**Trigger.** The L1 chain reorganizes, orphaning blocks that submissions were built against or included in.

**Detection.** By the node's own L1 view; no protocol-specific detection is involved.

**Hatch.** Ordinary L1 rollback handling, followed by deposit-only substitution if the new execution inputs make the selected record's block invalid.
A reorganization replaces a suffix of L1, so orphaning a submission rolls back its inbox state changes, including the cursor update.
Under [canonical L1 origin resolution](../rollup/derivation.md#canonical-l1-origin-resolution), a submission included on the replacement chain uses the canonical L1 block at `epoch_num` rather than the batch's stale `epoch_hash`.
The selected transaction list remains fixed on Espresso, but its execution must be reevaluated against the replacement L1 history.
Execution can succeed with changed results; that case does not trigger deposit-only substitution.
If the block is invalid under those inputs, derivation substitutes a deposit-only block at the same height; this is an expected escape hatch, not a violation of the [Certification Claim](overview.md#the-certification-claim).

**Roles.** Nodes perform ordinary L1 reorganization handling and reevaluate the affected blocks.
If a height loses its selected content through deposit-only substitution, recovery proceeds under [S4](#s4-a-height-settles-as-a-deposit-only-block).

**Residue.** Block hashes and execution results may change.
A height replaced by a deposit-only block loses its selected ordinary content; any resulting recovery is covered by S4.
Excluding deposits from the record does not remove their effects on ordinary transactions' balances, nonces, or execution state.

### S4: A Height Settles as a Deposit-Only Block

**Trigger.** Derivation fills some height `N` with a deposit-only block while the selected record sequence holds a different content for it.
The [nonce cascade above](#why-a-height-cannot-be-repaired-in-place) can cause further heights to settle deposit-only, while other already-final records remain executable.

**Detection.** Unspecified today, and the first of the [open questions](#open-questions) below.
A [full node](full-node.md#the-guarantee) can identify an incompatible record once the inputs needed for the relevant check are available, but who acts on that determination is currently a human decision.

**Hatch.** Derivation's deposit-only substitution or fallback absorbs it.
Recovery is a **drain**: let `N+K` be the last height for which a selected record already exists at the moment the divergence is detected.
The batcher's submissions advance the cursor across `N+1 .. N+K` while derivation processes those heights in order against their actual derived parents, preserving content that passes the applicable checks and using a deposit-only substitution where required.
Normal operation resumes at the first height whose record was produced on the derived chain and satisfies the scan's preceding-origin constraint, provided the three obligations below are met and delivery meets the [recovery conditions](#recovery-outcomes).
Pre-divergence records finalized beyond `N+K` are also processed individually, preserving content that remains valid, as [The Publisher Keeps Publishing](#the-publisher-keeps-publishing) states.
Following the derived chain alone does not establish that origin constraint; the [publisher subsection](#the-publisher-keeps-publishing) describes the case that needs an override to repair the scan anchor.

**Roles.** Three, stated in full below, together with the record-selection and delivery conditions above.

**Residue.** The ordinary content of `N` and of any later height that settles deposit-only is permanently dropped; other heights can retain their selected content.
The range of heights produced on the pre-divergence trajectory depends on how quickly the divergence is detected and the sequencer reorganizes onto the derived chain.
Without a bound on that delay, this specification does not bound content loss to one settlement lag.

#### The Batcher Continues Submitting

The batcher SHOULD keep constructing and submitting batches for heights in `N+1 .. N+K`, taking each height's content from its selected record, including heights expected to settle deposit-only.
These submissions advance the cursor and allow still-valid selected content to settle.
A changed derived parent does not itself prevent [certifier verification](certifier.md#advancing-the-cursor): the content still matches the selected record, and the core does not check `parent_hash`; the other submission checks still apply.
A batcher that stops submitting leaves the drain incomplete.
Recovery SHOULD first use an alternative registered batcher to resume submission from the current cursor under [Recovery by an Alternative Batcher](batch-inbox.md#recovery-by-an-alternative-batcher), after the exclusive window and any unfinished-channel timeout permit it.
An [emergency cursor override](batch-inbox.md#emergency-cursor-override) is the last resort when ordinary certified submission cannot continue, rather than a necessary consequence of the original batcher stopping.

#### The Sequencer Follows the Derived Chain

The sequencer reorganizes onto the actual derived chain as the drain proceeds, following both the blocks whose selected content still executes and the deposit-only substitutions.
It resumes from that chain's state and [L1 origin](https://specs.optimism.io/glossary.html#l1-origin), rather than assuming that every height in the drain is empty or retaining the pre-divergence state.
For a height filled without a batch, the deposit-only fallback determines the L1 origin; this does not determine the outcome of a later height whose batch remains valid.
A record whose `epoch_num` fails the [batch queue](https://specs.optimism.io/protocol/derivation.html#batch-queue)'s origin or timestamp conditions against the derived parent loses its height to a same-height deposit-only substitution under [Δ5, substitution instead of dropping](derivation-pipeline.md#delta-5-substitute-instead-of-dropping).
Producing further records against stale state can extend the loss, but does not imply that every such record will fail.

#### The Publisher Keeps Publishing

The publisher's obligations do not change during the drain: it publishes one record per block the sequencer produces, including the blocks the sequencer produces once it has reorganized onto the derived chain, as [Publication](block-records.md#publication) requires regardless of settlement progress.
Those records carry the derived chain's L1 origins, but [Record Validity](block-records.md#record-validity) compares each origin against the preceding selected record's origin, not against the derived parent.
An origin that has advanced beyond that scan anchor satisfies the non-decreasing-origin condition, however far it advanced; an origin below the anchor fails it.
An erroneous or malicious publisher can place the anchor ahead of the derived chain even without a reorganization or override, by selecting a record with an incompatible origin that nevertheless satisfies Record Validity and the inbox's L1-head bound.
That case is not caused by correct production and publication along an unchanged valid chain.
If records produced on the derived chain fall below the anchor, ordinary submissions cannot select them merely because the sequencer resumed correct behavior.
The [existing override](batch-inbox.md#emergency-cursor-override) can repair the anchor only if the admin supplies a consistent recovery cursor satisfying all its constraints, including a strictly greater L2 height; it can skip additional content and cannot restore content already lost.
Without such a recovery position, automatic resumption is not established.
This is a publisher fault, not a consequence of correct operation, and it adds no trust premise: a publisher that can stall settlement by going silent ([S6](#s6-the-publisher-goes-silent-no-recovery-path-exists)) can also stall it this way.
Here the override can repair the anchor, whereas S6 has no repair.
Records the publisher finalized for heights beyond `N+K` before the sequencer reorganized carry the pre-divergence content and origins.
Those that satisfy Record Validity extend the selected range; derivation preserves content that remains valid and substitutes deposit-only blocks where [Δ5](derivation-pipeline.md#delta-5-substitute-instead-of-dropping) or execution requires it.
Resuming production on the derived chain ends this source of stale records, but does not by itself repair an origin anchor ahead of that chain.
A publisher that paused instead would leave subsequent heights without records, preventing the drain from progressing past the first missing record.

### S5: The Publisher Publishes an Invalid Record

**Trigger.** The publisher publishes a record that fails [Record Validity](block-records.md#record-validity) — in practice one whose `epoch_num` decreases from the preceding selected record's, since the remaining conditions are on authenticity and on content shapes the publisher constructs itself.
A record that the origin's timestamp relation or the [sequencer drift](https://specs.optimism.io/protocol/derivation.html#batch-queue) bound condemns is a different case: such a record passes selection, and its height is lost to [S4](#s4-a-height-settles-as-a-deposit-only-block) rather than left unresolved here.

**Detection.** Immediately, by any [full node](full-node.md#required-capabilities), and by every other observer of the namespace: validity is a function of the record together with the height and preceding origin expected at that point, and of nothing else.

**Hatch.** Corrected publication, preceded by a sequencer reorganization when necessary.
Because validity rejects the record, it is never selected: the scan continues expecting that same height, and certification progress remains blocked until a valid replacement is selected or an override skips it.
An alternative batcher cannot supply missing publisher-authorized content; it can submit the replacement once the scan selects it and ordinary submission eligibility permits.

**Roles.** Recovery SHOULD first correct the rejected record and publish a valid replacement for the same unresolved height.
The [publication prohibition](block-records.md#publication) applies only to a height already governed by a selected record, so the rejected candidate does not prohibit replacement.
If the published origin was copied incorrectly, the replacement MUST faithfully report the block's actual origin under [Fields](block-records.md#fields).
If that actual origin is below the scan's preceding-origin anchor, faithful republication of the existing block cannot satisfy origin continuity.
Recovery SHOULD first reorganize the sequencer onto the canonical derived chain, rebuild the affected unselected height and publish a record for the rebuilt block.
The rebuilt block MUST satisfy derivation's origin and execution rules, and its replacement record MUST satisfy Record Validity, including the scan's preceding-origin constraint.
The publisher MUST NOT raise the record's origin merely to pass selection without rebuilding a block that legitimately uses that origin.
Earlier invalid publications remain in Espresso's finalized history; recovery publishes a new candidate rather than reorganizing that history.

An [emergency cursor override](batch-inbox.md#emergency-cursor-override) remains a last resort when ordinary correction, rebuilding and publication cannot restore progress, or the publisher does not act.
An anchor ahead of every origin permitted for the next derived block cannot necessarily be repaired by a sequencer reorganization; reorganization MUST NOT replace finalized derived history to make the anchor fit.
The override can skip the blocked height and repair the scan anchor only if the admin can supply a consistent resume position satisfying its existing advance-only constraints.
Neither mechanism guarantees recovery when no valid replacement or permitted resume position exists.

**Residue.** A rejected record contributes no selected content; preserving the ordinary content of an existing block requires faithful correction and delivery within the [recovery conditions](#recovery-outcomes).
Rebuilding can change that unselected block's content, origin and execution results.
If the height is skipped or its sequencing window expires before recovery completes, its ordinary content is lost and recovery proceeds under S4.

### S6: The Publisher Goes Silent (No Recovery Path Exists)

**Trigger.** The publisher stops publishing records for the heights its [`PUBLISHER_KEYS`](block-records.md#authorized-publisher-keys) entry governs.

**Detection.** An observer can determine that its verified finalized history contains no selected record for the next expected height and that its selected sequence has not advanced.
This symptom alone does not distinguish publisher silence from publication, finalization or data-retrieval delay, or establish that the silence is permanent.
This specification defines no fault-detection deadline.

**Hatch.** No alternative publisher is defined.
Already finalized selected records can still be attested and submitted while their data and required proofs remain available and the existing acceptance and delivery conditions hold.
Without an override, record progress stops at the first unresolved height lacking a selected record; a batcher cannot supply the missing publication.
The override cannot manufacture content; it can only skip past heights, so the most it can do is abandon everything the silent publisher did not publish.
Alternative-batcher submission is not a remedy here either: it addresses a stalled _settlement_ path, and this is a stalled _certification_ path.

**Roles.** Undefined.
This specification fixes which key authenticates which height and, as [Authorized Publisher Keys](block-records.md#authorized-publisher-keys) states, defines no procedure for replacing a publisher that has stopped publishing the heights its entry governs; key rotation is out of scope there.

**Residue.** Heights from the first missing record onward remain blocked from ordinary certified submission while that gap persists.
The existing finalized backlog is not invalidated by the silence, but preserving its content still requires the [lossless-recovery conditions](#recovery-outcomes).
This is the one scenario in this section for which the protocol currently offers nothing, and a reader should not infer from the existence of the other hatches that this case is covered by any of them.
It is accepted for the current stage, in which [`PUBLISHER_KEYS`](block-records.md#constants) holds a single pinned entry and no rotation is planned; an L1-governed table becomes the natural remedy once sequencer authority itself moves to L1, per the [publisher-key rationale](block-records.md#rationale).

### S7: A Certifier Enclave's Measurement Is Retired, or Its Key Is Compromised

**Trigger.** Governance removes a measurement from the authorized-measurement set, either to retire the image or in response to a compromised key registered under it.
Every key registered under that measurement becomes inadmissible; the protocol defines no separate individual-key revocation mechanism.

**Detection.** Submissions signed by those keys fail the inbox's [acceptance](batch-inbox.md#acceptance) verification, and the authorized-measurement set is public L1 state.

**Hatch.** [Certifier key registration](batch-inbox.md#certifier-key-registration) under another authorized, trusted measurement.
A batcher can use an already registered key under that measurement or register a replacement enclave's key.
A replacement image with a different measurement MUST generate and register a fresh key under [Key Sealing](certifier.md#key-sealing); it MUST NOT reuse or re-register the retired image's key under the replacement measurement.
Launching a new enclave under the retired measurement does not exclude the old keys: reauthorizing that measurement restores all its previously registered keys, including any compromised key.
The contract checks the recovered signing key and its registered measurement without requiring a particular key or enclave operator, per [Acceptance](batch-inbox.md#acceptance).
An alternative batcher's enclave is therefore subject to the same signature and measurement checks as the primary's.

**Roles.** The L1 super admin withdraws the affected measurement and authorizes a replacement if no suitable one remains authorized; the submitting batcher obtains a key registered under an authorized measurement.
Recovery depends on the replacement image and key satisfying the [certifier trust premises](full-node.md#conditions-under-which-it-holds), not merely having a different measurement.
The publisher and sequencer roles do not change.

**Residue.** Replacement authorization and key registration, when needed, add latency.
For content whose submission was prevented by withdrawal, no selected content is lost if recovery and subsequent submission meet the [lossless-recovery conditions](#recovery-outcomes); heights that instead lose their content to the deposit-only fallback proceed to S4.

### S8: A Previous Override Left the Cursor at an Unreachable Position

**Trigger.** An [emergency cursor override](batch-inbox.md#emergency-cursor-override) installs a cursor from which no valid continuation of the selected record scan can be established.
A trusted quorum from an earlier epoch is not itself such a failure: valid transitions can advance it to the required epoch.
An untrusted anchor can instead compromise authenticity, as the [override's quorum trust discussion](batch-inbox.md#emergency-cursor-override) explains; it is not guaranteed to produce the verification failure considered here.

**Detection.** Attempts to produce a valid continuation fail to advance the selected record scan.

**Hatch.** A further override can repair the cursor only if a correct recovery position with a strictly greater `block_number` can be supplied.
At the maximum value of the cursor's [`uint64` block number](certifier.md#cursor), no representable value satisfies that condition, so the existing override interface has no repair path.
A separately authorized implementation upgrade or state migration would be outside this interface; this specification defines no such recovery procedure.

**Roles.** The `admin` supplies the larger valid cursor on the terms the [override](batch-inbox.md#emergency-cursor-override) states, if such a position exists.

**Residue.** A successful repair advances past additional heights and does not restore already lost content.
Without a valid repair, the certified cursor remains unable to advance; the ordinary deposit-only fallback does not repair it.

### S9: Espresso Liveness Failure

**Trigger.** Espresso stops finalizing new blocks, so no additional records can become finalized.

**Detection.** An observer sees no newer authenticated finalized Espresso block.
That observation alone does not distinguish a finalization halt from delayed finalization or delayed delivery to the observer, or establish that the halt is permanent.
This specification defines no fault-detection deadline.

**Hatch.** **None applies.**
The halt does not invalidate already finalized history: selected records from it can still be verified, attested and submitted while their data and required proofs remain available and the existing acceptance and delivery conditions hold.
Progress requiring additional finalized history is blocked until Espresso resumes finalization; heights left without a usable batch follow derivation's ordinary deposit-only fallback.
Unavailability of historical data or proofs can prevent use of the existing backlog as well, but does not follow from a finalization halt alone.

**Roles.** The publisher remains subject to [Publication](block-records.md#publication).
The batcher can continue submitting the available finalized backlog under the existing rules, but cannot submit content that still requires finalization.

**Residue.** Once Espresso recovers, progress beyond the previously finalized range can resume, but lossless recovery still requires the [lossless-recovery conditions](#recovery-outcomes).
If the outage outlasts a height's sequencing window, derivation fills that height with a deposit-only block; subsequently finalizing its record does not restore the lost content, and recovery proceeds under S4.
This is distinct from [S6](#s6-the-publisher-goes-silent-no-recovery-path-exists): there the certification path is withheld by one party while everything else works, here it is broken for everyone, and it resolves when Espresso does rather than requiring anyone to be replaced.

### S10: The Data Availability Provider Is Unavailable

**Trigger.** The data availability provider cannot disperse a submission payload, so no certificate exists and a submission cannot be constructed — the inbox's [acceptance](batch-inbox.md#acceptance) requires a valid certificate committing to the attested bytes.

**Detection.** By the batcher, on dispersal failure, and by the stalling cursor thereafter as in [S1](#s1-the-batcher-stops-submitting-while-records-continue).

**Hatch.** If one dispersal service endpoint is unavailable, another endpoint can restore submission if it obtains EigenDA certificates that the certifier core verifies and the inbox's registry and freshness checks accept.
If EigenDA cannot complete dispersal and produce such certificates, switching endpoints does not establish recovery; submission waits for EigenDA to recover.
This specification defines no fallback to another DA network.
[Provider Substitution](../rollup/batch-submission.md#provider-substitution) connects the batcher's [alt-DA](https://specs.optimism.io/experimental/alt-da.html) interface to the certifier; it does not replace EigenDA.

**Roles.** The batcher continues to submit frames through the certifier; a replacement dispersal service uses the same EigenDA commitment and certificate verification rules.
The publisher and sequencer need no change solely for that endpoint replacement.

**Residue.** No selected content is lost if restored dispersal and subsequent submission meet the [lossless-recovery conditions](#recovery-outcomes).
Public availability of the selected record alone does not prevent expiry; heights whose content is lost to the deposit-only fallback proceed to S4.

## What Has No Recovery Path

The following cases have no recovery path defined here.

- [**S5 — an invalid record**](#s5-the-publisher-publishes-an-invalid-record), if the publisher does not republish.
  The height has no selected record, so nobody can construct a submission for it and the cursor stops there; the override can only abandon the height.
- [**S6 — a silent publisher**](#s6-the-publisher-goes-silent-no-recovery-path-exists).
  No records means no selected sequence to extend, and this specification defines no procedure for replacing the publisher whose key governs those heights.
- [**S8 — an unusable override cursor with no larger valid recovery position**](#s8-a-previous-override-left-the-cursor-at-an-unreachable-position).
  The advance-only override cannot repair it, including when `block_number` is already at its type's maximum; no implementation-upgrade or state-migration repair is specified here.
- [**S9 — an Espresso liveness failure**](#s9-espresso-liveness-failure).
  Nothing can be certified, so certified settlement halts while derivation fills heights with deposit-only blocks; this one is by design and resolves when Espresso does.
- [**S10 — EigenDA cannot complete dispersal**](#s10-the-data-availability-provider-is-unavailable).
  No alternative DA-network recovery is defined; submission waits for EigenDA to recover, subject to the lossless-recovery deadlines.

## Constants

| Constant or configuration parameter | Definition                                                                                     |
| ----------------------------------- | ---------------------------------------------------------------------------------------------- |
| `PUBLISHER_KEYS`                    | [Block record constants](block-records.md#constants).                                          |
| `SETTLEMENT_LAG_BOUND`              | [Batch inbox constants](batch-inbox.md#constants).                                             |
| `seq_window_size`                   | OP Stack rollup configuration, used under [Exclusive Window](batch-inbox.md#exclusive-window). |

## Open Questions

- **Detection, and who decides to enter recovery.** Nothing above states how the divergence in [S4](#s4-a-height-settles-as-a-deposit-only-block) is detected or who declares recovery. Entering the drain is operator-controlled, so the trigger is currently a human one; whether that is acceptable, and what a node should surface to make it timely, is unspecified.
- **When the drain does not complete on its own.** The drain completes without an override if all three obligations are met. It is worth stating the conditions under which it does not, so that the override is reached for a named reason rather than by default.
- **Interaction with the exclusive window.** During the drain `cursor.epoch_num` advances across the pre-divergence range, so the [exclusive window](batch-inbox.md#exclusive-window) is measured against stale origins throughout. Alternative-batcher takeover uses the existing window and channel-timeout rules; this specification does not bound the remaining delivery budget throughout the drain.
- **Correction deadline.** [S5](#s5-the-publisher-publishes-an-invalid-record) specifies correction and rebuilding before an override, but does not define a deadline for the publisher or sequencer to perform that recovery.

## Spec History

Block certification is not activated on any network.

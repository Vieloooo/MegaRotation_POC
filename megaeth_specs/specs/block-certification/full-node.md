---
description: Full node verification — required capabilities, the independent-finality claim, its assumptions, and open questions.
spec: TBD
---

# Full Node Verification

{% hint style="warning" %}
This page is a draft of unstable behavior.
Block certification is not activated on any MegaETH network.
It records the property established so far, the conditions it rests on, and the questions still open; the normative rules are not yet written.
{% endhint %}

This page describes the capabilities needed for independent certification verification and the proposed guarantee they provide.

## Motivation

A full node is the only party holding all three of the [selected record sequence](block-records.md#canonical-record-sequence), the L1 chain, and the ability to execute, so it is the only party that can decide a height's fate before any L1 transaction for that height exists.
The [certifier core](certifier.md#certifier-core) is a pure function over Espresso data, supplied no L1 data and performing no execution; the [inbox contract](batch-inbox.md) sees only its own cursor and a bounded window of L1 history.
[Derivation](derivation-pipeline.md) has everything it needs but learns the answer only when it arrives at that height, which is the point at which knowing it is no longer useful.

## Required Capabilities

The capabilities below distinguish the node's verification from the checks performed by the certifier and inbox.

### Read the Espresso Namespace and Select Records

The node scans the finalized namespace forward and authenticates [record batches](block-records.md#record-batch) and [record fragments](block-records.md#record-fragment).
It selects complete records through the [record selection pipeline](block-records.md#record-selection-pipeline), starting each scan from a cursor under [Sync Start](block-records.md#sync-start).
It applies [Record Validity](block-records.md#record-validity), and keeps the first valid record for each successive L2 height — producing the [selected record sequence](block-records.md#canonical-record-sequence) itself rather than accepting one from any data source.
This requires no trust in whoever supplies the bytes: observers using the same authenticated Espresso history, protocol parameters and L1-established starting cursor compute the same selection.
The local scan does not make records canonical; the node tracks that status separately from accepted [`CursorAdvanced`](batch-inbox.md#events) transitions.
A node that instead took the sequence from an endpoint would hold no guarantee at all, only a claim: the whole of what follows is conditional on the node verifying its selected sequence independently.
The scan begins where the [inbox](activation.md#initial-cursor) fixes it and restarts at every [override](batch-inbox.md#emergency-cursor-override), which the node locates through the inbox's [`CursorSet`](batch-inbox.md#events) events; a node that scanned only from the initial cursor would stall at the first skipped height while the certifier went on.

### Verify Espresso Consensus Rather Than Trust an Endpoint

The node performs the same three operations the certifier relies on, defined under [Espresso Verification](certifier.md#espresso-verification): that a block was finalized, that the quorum governing an epoch is reachable across epoch boundaries from an already-trusted quorum, and that a namespace payload is either complete or genuinely absent.
Every normally established quorum MUST chain back to the snapshot matching the scheduled `INITIAL_QUORUM_HASH`; overrides retain their explicit administrative correctness premise.
The namespace-completeness part is the load-bearing one for this page.
[Record selection](block-records.md#canonicality) chooses the _first_ valid record for a height, so a data source able to omit part of a block's namespace payload undetected could suppress the record that should have won and promote a later one in its place.
That would change which records the node selects without ever producing an invalid record.
The node verifies these properties itself rather than taking the certifier's result as its own view of the selected record sequence.
Settlement enforcement still relies on the [certifier's TEE trust assumptions](certifier.md#attestation); independent verification does not remove those assumptions.

### Evaluate the L1-Dependent Conditions Against Its Own L1 View

For a certified determination, a node MUST check the record's origin continuity against the same correct parent used for its execution prediction: if that parent's L1 origin number is `e`, the record's `epoch_num` MUST be either `e` or `e + 1`.
[Record Validity](block-records.md#record-validity)'s monotonicity check against the preceding selected record does not establish this condition.
A record failing it remains subject to selection under Record Validity, but its height undergoes [Δ5 substitution](../rollup/derivation.md#same-height-deposit-only-substitution) if the batch reaches that check; successful execution of its ordinary transactions alone does not justify certified status.

The node also checks the relation between the block's timestamp and its [L1 origin](https://specs.optimism.io/glossary.html#l1-origin)'s timestamp, and the [sequencer drift](https://specs.optimism.io/protocol/derivation.html#batch-queue) bound.
These timestamp checks compare the record — its height, which fixes the timestamp through the [protocol-fixed bijection](block-records.md#fields), and its `epoch_num`, which names the origin — against the required canonical L1 headers.
The checks do not require a batch to exist; if the correct parent or required canonical inputs are unavailable, the certified determination MUST remain pending.
The node MUST reevaluate the determination if that parent or those inputs change.
A mismatch against the current L1 view alone does not establish that the publisher violated the timestamp relation when it signed the record.
These checks belong to the full node's certified determination, not Record Validity: given the same scan cursor and Espresso history, selection is independent of the L1 execution inputs, while the prediction of settlement depends on those inputs.
The certifier cannot pre-empt these failures either, because it receives neither authenticated L1 data nor the derived parent; Δ5 handles them in derivation.

### Execute the Record's Transactions Against Its Own Head

Executing against the correct parent state lets the node anticipate an [engine rejection](derivation-pipeline.md#the-two-kinds-of-failure).
The certifier cannot perform this check because it does not execute against that state.
A node can evaluate execution validity against its current parent state and L1 view before derivation reaches the height.
That determination must be reevaluated if an L1 reorganization changes its execution inputs.
A transaction that merely reverts does not make the block invalid and is not what this check looks for; what does is a pre-execution failure — a nonce that no longer matches, for instance — which aborts the whole block rather than one transaction.

### Optionally, the Ability to Self-Rescue

A node that additionally runs a certifier enclave, registers its key under [Certifier Key Registration](batch-inbox.md#certifier-key-registration), and holds membership in the [batcher registry](batch-inbox.md#batcher-registry) can submit for a height itself rather than waiting for someone else to.
Verification and rescue are separate capabilities: a node that is not registered in both can verify everything above and still only know that a height will settle _on the condition that_ some registered party submits for it.
Key registration is permissionless and registry membership is not, so the second is the binding one — see [the registration condition](#conditions-under-which-it-holds).

### Surface What It Has Determined

Detection is worthless if nobody acts on it, and intervention on a doomed height is time-bounded by the sequencing window, so what a node surfaces and to whom is part of the capability rather than incidental to it.
What is reported, in what form, and to which party is undecided.

## The Guarantee

**Certified status (independent finality).**
A full node treats height `n` as certified after independently verifying its selected record, L1-dependent validity and execution against the correct parent state: under the [conditions below](#conditions-under-which-it-holds), that content will settle without [the deposit-only fallback](https://specs.optimism.io/protocol/derivation.html#batch-queue) (the empty batch derivation produces when a height's sequencing window elapses).
This determination requires no L1 submission for `n`.
Its content-correctness component requires no trust in the operators of the sequencer, batcher or certifier, beyond the release policy of any [sealed](certifier.md#key-sealing) certifier key under condition 7 and the [verification-governance trust assumptions](batch-inbox.md#certifier-key-registration).
Its prediction of settlement without fallback additionally depends on timely delivery by at least one registered batcher and timely admin intervention against a repeatedly stalling primary, under condition 4.
It does not itself make the record canonical; that status requires inbox acceptance of the closing submission advancing through `n`.

This is strictly stronger than the [Certification Claim](overview.md#the-certification-claim).
The Claim rules out a third settled-content outcome: if a height settles, it settles either as the selected content made canonical by inbox acceptance or as a [deposit-only block](https://specs.optimism.io/protocol/derivation.html#batch-queue).
Independent finality predicts which outcome a height will have under those conditions; it does not make the L1 inputs final.

### Content Correctness and Timeliness

Under the Claim's safety premises, a batcher cannot make a different ordinary transaction list settle, whether or not it completes its channels on time.
Content correctness does not depend on every channel opener delivering a complete channel.
The prediction that the selected ordinary content settles without fallback additionally assumes the delivery, availability and execution conditions below.
A batcher that misses the delivery deadline can cause deposit-only fallback; accepted data that cannot be retrieved instead stalls derivation under [Batch Commitment Resolution](../rollup/data-availability.md#batch-commitment-resolution).
Neither outcome permits substitution of arbitrary ordinary content.

### What It Does Not Cover

Certified status predicts settlement of the selected ordinary transaction list under the stated conditions, not execution validity under every future L1 history or an unconditional resulting block hash.
An L1 reorganization can change deposits, account balances or nonces, and other execution inputs.
Execution can remain valid with different results, or a previously valid certified block can become invalid.
If execution rejects the block, derivation substitutes a deposit-only block at that height and follows the [recovery path for L1 reorganizations](recovery.md#s3-an-l1-reorganization).
This is an expected escape hatch that satisfies the Certification Claim; it does not alter the record finalized on Espresso.

### Conditions Under Which It Holds

1. **Derivation ignores the batch-supplied `parent_hash` and `epoch_hash`** — the normative [parent-hash rule](../rollup/derivation.md#batch-parent-hash) and [canonical origin resolution](../rollup/derivation.md#canonical-l1-origin-resolution).
   Without them the property is false rather than merely weaker: a registered alternative batcher can submit a batch whose content is entirely correct with either field poisoned, and the node can neither predict which nor prevent it.
2. **Induction from the activation checkpoint.**
   The guarantee for `n` depends on the correct state at the [activation checkpoint](activation.md#initial-cursor) and on the same guarantee holding for every intervening height, because the execution check is only meaningful against the correct parent state.
   The scheduled activation height fixes the preceding checkpoint; its state is determined by legacy derivation and the deterministic [activation fill](activation.md#activation-deposit-only-fill).
   The [initial-cursor rules](activation.md#initial-cursor) constrain bootstrap; its supplied origin does not authenticate or select the checkpoint state.
   A node independently deriving the pre-activation history obtains the checkpoint state through that derivation; a node starting from a trusted snapshot retains its existing snapshot trust assumption.
   Certification and bootstrap do not authenticate that earlier history or discharge the snapshot trust assumption.
   Every [emergency cursor override](batch-inbox.md#emergency-cursor-override) also has to name either the index-0 fragment of an attempt for its `block_number` that completes with a record the scan [re-selects](block-records.md#sync-start), or a position from which the scan finds the publisher's record for `block_number + 1` first, an origin floor that the next record (and, in the first case, the re-selected record) does not go below, and a quorum anchor reachable from the scheduled trusted Espresso quorum.
   The node resumes its own scan from each override, so a correct override keeps the node's sequence and the certifier's identical.
   This is an admin correctness assumption, not a consequence of the override's advance-only check; an incorrect override can invalidate the guarantee for subsequent heights.
   The prediction for a not-yet-accepted height also assumes no override skips it or changes its selected record before acceptance; the node reevaluates affected certified determinations when an override occurs.
3. **Registration**, per [the self-rescue capability](#optionally-the-ability-to-self-rescue).
   Absent it, the node's determination is conditional on some registered party submitting.
   Registry membership is curated because a registered address can reset a channel, that is, open a new channel at frame 0 once the one currently open has timed out, and thereby abandon it.
   Curation limits who may do so, the [exclusive window](batch-inbox.md#exclusive-window) limits when alternatives may submit, and [channel ownership](batch-inbox.md#acceptance) limits a reset to channels that have closed or timed out.
   None of the three ensures that an opener completes its channel; what ownership ensures is that no one else can interrupt it before the channel times out.
   Enclave registration is the permissionless half — authorization attaches to the attested image rather than to the operator, so any party running an authorized image may register a certifier key without anyone's cooperation.
   The curated set therefore also decides who may hold the rescue half of this guarantee.
   Verification is open to everyone — any node can determine that a height's selected content will settle if a registered party submits — while acting on that determination when nobody else submits is confined to the registry.
   That asymmetry is the guarantee's actual shape rather than a gap in it.
4. **Timely successful channel delivery.**
   The required frames must be accepted on L1 and form a complete channel that derivation can consume within the applicable sequencing-window and channel-timeout rules.
   This assumes at least one registered batcher delivers the required complete channel in time, rather than assuming that every channel opener completes its channel.
   L1 inclusion alone is insufficient because included transactions may revert or contribute only to abandoned partial channels.
   An abandoned channel costs at most one [channel timeout](https://specs.optimism.io/protocol/derivation.html#channel-bank) per occurrence, but repeated open-and-abandon cycles have no aggregate completion bound from that rule alone.
   Timeliness therefore also assumes that the admin removes or replaces a repeatedly stalling primary early enough for a delivering registered batcher to meet the affected heights' deadlines.
   The [complete-channel submission recommendation](batch-inbox.md#rationale) makes the ordinary case a single submission; it does not enforce delivery.
5. **Data availability for the submission.**
   The frame the commitment names has to be retrievable for derivation to read the content at all.
6. **Unchanged L1 execution dependencies.**
   The determination is conditional on the L1 history used to derive the parent state and the height's execution inputs remaining canonical.
   If a reorganization changes those dependencies, the node reevaluates the affected heights; the prior determination no longer guarantees that their selected content settles.
7. **Certifier TEE integrity.**
   The authorized program performs the specified checks, the TEE protects its execution and signing key, and hardware attestation correctly binds that key to the authorized image, as specified under [Attestation](certifier.md#attestation).
   Settlement enforcement depends on those assumptions and on the inbox's [key registration](batch-inbox.md#certifier-key-registration) and [signature verification](batch-inbox.md#acceptance), not on the certifier operator's honesty.
   Where a certifier key is [sealed](certifier.md#key-sealing), it also depends on the sealing service and on every party able to change its release policy, which can include the certifier operator.
   The same integrity now also covers data availability: the core verifies the EigenDA certificate's aggregate signature and stake thresholds in place of an on-chain check, while the inbox authenticates only the operator set, stakes, thresholds and reference block it used, as the [certifier page](certifier.md#security-considerations) states.

Condition 3 is what makes the [exclusive window](batch-inbox.md#exclusive-window) load-bearing for this property, and not only for [recovery](recovery.md).
The margin `seq_window_size - EXCLUSIVE_WINDOW` requires `cursor.epoch_num <= e`, where `e` is the L1 epoch governing the missing height's deposit-only timeout, as stated under [Exclusive Window](batch-inbox.md#exclusive-window).
Selected-record monotonicity alone does not establish that relation after the record and derived origins diverge; the fixed margin need not survive such a divergence.
It does not establish condition 4: any required preparation, registration, attestation, dispersal, and accepted channel submission must still finish within the remaining deadlines.
A conservative recovery budget includes `EXCLUSIVE_WINDOW + CHANNEL_TIMEOUT` plus the alternative batcher's recovery time, with all durations expressed in L1 blocks and kept below the affected height's remaining sequencing-window budget.
The recovery time includes any required preparation, enclave registration, certification, DA dispersal and successful L1 inclusion; timely admin replacement is an additional prerequisite when a primary repeatedly stalls progress.
The exclusive-window wait and channel-timeout wait can overlap, so this sum is a conservative budget, not a protocol-enforced completion bound.
The full `seq_window_size` is not a fresh budget for a height that is already part-way through its window.

## Constants

| Constant or configuration parameter | Definition                                                                                     |
| ----------------------------------- | ---------------------------------------------------------------------------------------------- |
| `INITIAL_QUORUM_HASH`               | [Activation constants](activation.md#constants).                                               |
| `EXCLUSIVE_WINDOW`                  | [Batch inbox constants](batch-inbox.md#constants).                                             |
| `CHANNEL_TIMEOUT`                   | [Batch inbox constants](batch-inbox.md#constants).                                             |
| `seq_window_size`                   | OP Stack rollup configuration, used under [Exclusive Window](batch-inbox.md#exclusive-window). |

## Open Questions

- **Whether the detection response is specified at all.** The detectability is a protocol fact and belongs here.
  The reaction — stopping rather than serving a chain that will be reorganized — may belong entirely to implementations, since the safe chain is derived from L1 regardless of what any node did with the unsafe stream, so two nodes that respond differently are indistinguishable to every other party.
  This page may state only what a node is in a position to know.
- **What a node surfaces to operators.** Per [the surfacing capability](#surface-what-it-has-determined), a detection that is not visible is not actionable, and intervention is time-bounded.
  What is reported, and how, is unspecified.
- **How certified status is exposed.** The [confirmation stages](overview.md#confirmation-stages) distinguish the full node's conditional pre-L1 determination from canonical records established by inbox acceptance.
  The node's API representation and notification mechanism remain unspecified.

## Spec History

Block certification is not activated on any network.

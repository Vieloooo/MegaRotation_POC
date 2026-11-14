# Interface design review for the main_v2 major revision

Scope: independent review of the approved six-section outline. This note does not edit the manuscript. Sources checked on 2026-09-28 are the published Sync HotStuff author PDF and the AFT 2025 compiler paper's publisher abstract.

## Recommended service boundary

Give the regional service an authenticated session/authority context, a starting prefix, and an output event

\[
  \mathsf{Deliver}_{P}(M_h,\pi_h).
\]

Here \(M_h\) contains the complete ordered transaction bytes, stream position, predecessor commitment, and producer authentication; \(\pi_h\) is an opaque confirmation proof verified by the implementation's public verifier. The service specification should never require the receiver to run a commitment timer, vote in a BFT instance, interpret a local leader view, or inspect a particular QC stage. Changing the BFT realization changes the proof verifier, not the semantic output contract.

Preserve the notation \(H_h,\mathcal X_h,q_h,M_h\). For the abstract interface, describe only their semantic roles. If legacy serialization retains \((c,v_G,v_L,u)\), treat local-view/attempt fields as implementation metadata and define them where the protocol uses them. Do not preserve the Cantor pairing just to make an old field tuple look unchanged.

State three properties within one authenticated regional session:

1. **Authenticity and integrity.** Every delivered value has an authorized producer, its complete ordered content matches its commitments, and its proof attests to regional commitment. Producer authentication alone is not confirmation.
2. **Safety.** Each receiver's delivered sequence only grows; any two correct receivers' delivered sequences are prefix comparable. Therefore they agree on both contents and predecessor commitments at every common height.
3. **Liveness.** While the regional execution conditions continue, the committed chain grows; every committed prefix is eventually delivered to a correct subscriber that can contact an honest retaining replica and continues retrieval. If idle mini-blocks are permitted, say explicitly that growth is not a transaction-inclusion guarantee.

Keep an explicit distinction between early receipt/caching of leader-pushed data and the above irrevocable delivery event. Speculative data need not create a second complicated public API.

The proposed outer CFT layer may treat a region as one logical correct participant only while the inner safety conditions hold. An arbitrary violation of the synchronous network assumption is not proven to be crash-only behavior. This can be one sentence in the architecture section.

## Standard SMR definition and concrete input mapping

Sync HotStuff Section I.A defines SMR by agreement at each committed log position and eventual commitment of each client request by every correct replica. Paraphrase these two clauses and give the deterministic-machine interpretation in one sentence. There is no need to add a deduplication table, request IDs, exactly-once semantics, FIFO ordering, or a bespoke client-authentication layer to state this definition.

Let a client-level command be immutable **batch contents**, e.g. an ordered finite list of transaction lists. This command has no consensus height, local view, parent digest, or vote certificate. These are assigned when the producer constructs a consensus proposal on a particular branch. A later leader may place unchanged content in a fresh proposal; no requirement says that every competing signed proposal is eventually committed.

There are two logs to distinguish in one explanatory sentence: the consensus block chain fixes both data and its authenticated stream encoding, while its payload projection is the SMR command log. Deterministic execution applies the payload commands in that order. The stream projection flattens the agreed batches. Avoid defining \(T\) as a function of payload alone that supposedly recreates view-dependent legacy headers: the committed consensus record already fixes those headers.

### Liveness issue that must not be hidden

The published steady-state Sync HotStuff protocol detects insufficient block production and equivocation. A Byzantine leader that keeps proposing valid batches while censoring a particular pending command can avoid both conditions. Thus chain growth alone does not imply the full correct-client request-liveness definition.

Without adding a censorship protocol, the honest theorem is:

* chain/stream growth follows from the BFT progress mechanism;
* per-command completion additionally uses eventual honest serving leadership and weakly fair selection of persistent admissible commands.

Weak fairness is an application assumption, not FIFO: every persistent eligible command is eventually chosen by a sufficiently long-lived honest producer. The manuscript can state this in a short paragraph, leaving transaction selection unspecified. It must not claim every mempool transaction or every competing proposal is included. If the design wants request liveness against a indefinitely productive censoring leader, it must supply additional rotation/censorship machinery and prove that machinery; it is not a consequence of a local rotation list by itself.

## Permissionless admission versus the fixed-configuration theorem

Describe L1 as an externally finalized registration/stake authority: anyone satisfying its staking rule can register for a future configuration. During a regional session, use the one activated snapshot to verify identities, stake and producer order; evidence does not change the denominator mid-session.

State that the present proof covers this regional fixed-configuration execution. Activating a changed stake table needs an authenticated state-transfer/reconfiguration procedure. Merely observing an updated L1 stake table does not provide that procedure. Do not describe snapshot loading as a complete permissionless consensus compiler.

The AFT 2025 paper *From Permissioned to Proof-of-Stake Consensus* is the right literature pointer for the systematic conversion question. Its stated preservation theorem assumes the base protocol already provides the preserved properties in the **partially synchronous** setting. Standard Sync HotStuff relies on a known synchronous bound for safety, so citing that theorem cannot establish this adaptation. Two or three sentences suffice; a long caveat section would distract from the protocol.

## Sources checked

- Abraham, Malkhi, Nayak, Ren, Yin, *Sync HotStuff: Simple and Practical Synchronous State Machine Replication*, IEEE S&P 2020: [author PDF](https://dahliamalkhi.wordpress.com/wp-content/uploads/2019/11/synchotstuff-sp2020.pdf), Section I.A for the SMR definition; Section II and Figures 1–2 for proposal-independent client values, block chaining, no-progress blame, and view changes.
- Komatovic, Lewis-Pye, Neu, Roughgarden, Tas, *From Permissioned to Proof-of-Stake Consensus*, AFT 2025: [publisher page](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2025.18), abstract for the exact partial-synchrony scope of the compiler guarantee.

## Acceptance checks for the revision

- No request-ID, deduplication, FIFO, or reqRoot mechanism has returned under a different name.
- Stream proof semantics are stable if the underlying BFT changes.
- Candidate proposal identity is not confused with immutable input contents or final stream position.
- Stream-growth and per-request liveness are not silently interchanged.
- A region-wide synchrony failure is not asserted to be a crash automatically.
- No claim of complete dynamic permissionless consensus rests on fixed snapshot verification alone.

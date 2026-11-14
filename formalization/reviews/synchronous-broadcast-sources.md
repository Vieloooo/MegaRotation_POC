# Synchronous broadcast with a two-delay good case

Reviewed 2026-09-25. The user subsequently confirmed the intended source as Appendix C.1 / Figure 10 of arXiv:2102.07240. The other titles below remain disambiguation references.

## Confirmed authenticated source

Ittai Abraham, Kartik Nayak, Ling Ren, and Zhuolun Xiang, *Good-case Latency of Byzantine Broadcast: A Complete Categorization*, [arXiv:2102.07240v3](https://arxiv.org/pdf/2102.07240v3), Section 5.1 and Appendix C.1, **Figure 10 and Theorem 16** (PDF pages 26–27). Figure 5 concerns a different threshold.

The paper proves authenticated synchronous Byzantine broadcast with good-case `2δ` for `f<n/3`. Its model has signatures/PKI, known honest-link delay bound `Δ`, unknown execution delay `δ≤Δ`, bounded start skew, and no clock drift. Section 2 assumes actual start skew at most `δ`; Figure 10 sets its conservative skew parameter `σ=Δ` (cutoffs then `3Δ` and `5Δ`). Synchronized starts permit `σ=0` (cutoffs `2Δ` and `3Δ`). Do not equate actual skew with the conservative parameter.

Compact reconstruction of Figure 10:

1. Initialize `lock=⊥`. The designated sender broadcasts its signed value.
2. Each party broadcasts one signed vote for its first valid proposal.
3. On `n-f` matching signed votes: forward this certificate and set `lock=v`; commit early only when local receipt time is at most `2Δ+σ`.
4. At local time `3Δ+2σ`, **every party, including early deciders**, runs Byzantine agreement with its current lock. Undecided parties commit its output; parties then terminate.

BA is a primitive here, requiring agreement, termination, honest-input unanimity validity, and tolerance of the start skew. The authenticated asynchronous BRB in Figure 1 needs no fallback but may never terminate with a faulty sender.

The exact BA input is a snapshot of `lock` at the fallback invocation: initially `⊥`, changed to `v` upon reception of a matching vote quorum. The single-shot paper's syntactic message-validity definition does not itself validate MegaETH execution or checkpoint ancestry. In our adaptation, a nonempty BA input carries the checkpoint and its vote certificate; the fallback has an explicit external-validity contract admitting only valid certified checkpoints or `⊥`. The certificate must be checked against the fixed instance, weights, sender, parent, and required data. BA input does not retroactively change upon late certificate arrival.

## Other plausible title matches

- Abraham, Ren, and Xiang, [*Good-Case and Bad-Case Latency of Unauthenticated Byzantine Broadcast: A Complete Categorization*](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.OPODIS.2021.5), OPODIS 2021. The `n≥4f` threshold belongs to the **unauthenticated** two-round case. It should not be imported into the signed construction above.
- Albouy, Frey, Raynal, and Taïani, [*Good-Case Early-Stopping Latency of Synchronous Byzantine Reliable Broadcast: The Deterministic Case*](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.DISC.2022.4), DISC 2022. Its focus includes Byzantine-majority settings; the advertised good-case bound is `max(2,t+3-c)` rounds, where `t` bounds faulty processes and `c` is the actual number of correct processes. This is not the same actual-delay `2δ` statement.

## Derived design implications for our protocol

These are our composition checks and proposed adaptations, not additional claims proved by the candidate paper.

### Weighted regional form

For fixed regional stake `W`, faulty stake `B<W/3`, and threshold `q>2W/3`, two vote certificates intersect in stake at least `2q-W>W/3>B`. Some honest voter would have to double-vote for two certificates, so at most one certified value exists. Honest stake alone exceeds the threshold, supporting the good case. Replace the integer thresholds only after supplying this argument and a **weighted-validity BA fallback**. A count-based fallback cannot inherit a stake fault bound silently.

With synchronized start `t0`, define early decision cutoff `t0+2Δ` and fallback-input cutoff `t0+3Δ`; process timely received messages before the cutoff transition. If an honest party decides `v` early, its forwarded certificate reaches every honest party by the fallback cutoff. Certificate uniqueness then forces every honest BA input to be `v`, so unanimity validity preserves the early decision. When no honest party decides early, fallback agreement resolves the outcome. This is the essential synchrony dependence.

### A raw vote certificate is not a universally transferable decision certificate

Suppose Byzantine voters withhold their votes until after the early-decision cutoff. A matching quorum might then be assembled even though no honest party decided early and the fallback inputs were not unanimously that value. A fallback decision of `⊥` is not excluded merely by the later existence of that quorum. Therefore:

- late quorum receipt must not trigger early decision;
- downstream consumers cannot infer irrevocable delivery solely from an undated vote quorum;
- a proposer that must verify a regional decision later needs authenticated decision attestations or a verifiable fallback decision record;
- this export can require an additional communication hop while the **regional decision** still occurs in `2δ`;
- a synchronized live observer can apply a timing rule only if its clock, forwarding duties, and delay assumptions are included in the model. A claimed timestamp supplied by a Byzantine proposer is not proof of timely receipt.

### Empty outcomes and sequential composition

A missing proposal causes no local empty commitment. At fallback, an agreed `⊥` can be deterministically interpreted as a no-op record for that instance. Its identity must bind configuration, region, height/slot, designated sender, and parent. Every participant must agree on these instance inputs before invocation. The next accepted data block extends the preceding **decided** outcome; leadership may change between instances according to an agreed schedule. An early-deciding party remains responsible for forwarding certificates and completing the old instance's recovery even while it processes later work.

Do not invoke a second leader at the same log position as an unrelated instance. That can create two internally safe broadcasts with conflicting log outcomes. Either finish the existing instance or make the replacement continuation preserve its pending decision. Likewise, raw leader proposals can be streamed speculatively, but committing a child requires its parent outcome to be fixed. The single-shot `2δ` theorem does not by itself prove arbitrary pipelined SMR, lease changes, or cross-region handoff.

### Concrete fallback options and remaining work

The candidate paper deliberately leaves BA abstract. It is legitimate to give BA a precise interface in a modular protocol draft, but incorrect to claim its Figure 10 is a complete fallback implementation. A simple, expensive cold-path instantiation can use authenticated synchronous broadcasts of each validator's lock and deterministically select from the common vector. It must preserve honest-input unanimity in **stake**, validate any nonempty locked value and its certificate, and define a common default. This requires a separate proof and should be marked as our construction.

A primary source for authenticated synchronous broadcast as a fallback building block is Dolev and Strong, [*Authenticated Algorithms for Byzantine Agreement*](https://doi.org/10.1137/0212045), SIAM J. Comput. 12(4), 656–666 (1983), [paper PDF](https://www2.imm.dtu.dk/courses/02220/2015/L12/DolevStrong83.pdf). It supplies bounded-round authenticated agreement; its round count is based on the number of faulty identities, not faulty stake. A stake bound alone does not supply a small identity bound.

The corrected fast-path claim should therefore read: **under fixed eligible regional membership, authenticated synchronous links, agreed instance start/parent, the stipulated BA fallback, and bounded processing, regional parties decide an honest sender's value in `2δ`; worst-case termination and publicly exportable decision latency are separate bounds.**

## Agreed integration direction

Use serial reference broadcast instances for checkpoints covering approximately ten miniblock/order packets. Future packets may stream speculatively. Preserve the mandatory BA stage even after early decision. Export a separate `>2W/3` certificate of `DECIDED` endorsements, each signed only after the signer locally decides that checkpoint. Its extra dissemination hop is excluded from the `2δ` regional-decision claim. This is a protocol integration choice, not a feature claimed by Figure 10.

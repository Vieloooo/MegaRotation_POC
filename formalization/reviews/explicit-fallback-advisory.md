# Explicit synchronous completion for the regional checkpoint protocol

Reviewed against the current regional protocol on 2026-09-25. This is a proposed implementation and its composition argument, not a theorem attributed to the fast-broadcast paper.

## Verdict

The proposed parallel authenticated-broadcast construction supplies the missing weighted Byzantine-agreement completion. It needs no small bound on the **number** of Byzantine identities: with `n` fixed members, use `n` signature-chain rounds, corresponding to the conservative identity bound `n-1`. It provides termination, agreement, honest-input unanimity, and externally valid non-bottom outputs under `B<W/3`.

Its principal implementation requirements are:

1. Compare checkpoint **values**, not `(checkpoint, QC)` encodings. A QC is an interchangeable witness for a value; different signer subsets or signature bytes do not create different candidate values.
2. Every forwarded non-bottom packet includes its complete deterministic validity witness and required data. Local LMD preference and current retrieval success are not part of the fallback acceptance predicate.
3. Process a chain of length `r` only for round `r`; buffer early packets, and discard packets arriving after that round's local deadline. Appending a signature without this timing discipline is unsafe.
4. Keep the snapshot immutable. A later QC does not change an origin's honest fallback input or reopen a completed broadcast.
5. Every fast decider participates through completion. The fallback remains mandatory on the good path in this reference implementation.

## Source checks

Dolev and Strong, [Authenticated Algorithms for Byzantine Agreement](https://doi.org/10.1137/0212045), SIAM J. Comput. 12(4), 656–666 (1983), [primary-paper PDF](https://www2.imm.dtu.dk/courses/02220/2015/L12/DolevStrong83.pdf), supplies the authenticated synchronous-broadcast construction and the dependence on an identity-fault bound. Its optimized communication bound should not be quoted for the simple implementation specified below.

Blum, Katz, and Loss, [Synchronous Consensus with Optimal Asynchronous Fallback Guarantees](https://eprint.iacr.org/2019/692.pdf), Section 3, Figure 1, explicitly describes timed authenticated broadcast, and Figure 2 uses parallel broadcasts to obtain a common input vector. Their displayed broadcast variant uses `n-1` rounds and signature sets excluding the receiver; the construction below instead uses `n` rounds, nested chains, and a separately proved weighted selection function. These distinctions prevent accidentally importing their exact theorem or threshold.

Abraham, Nayak, Ren, and Xiang, [Good-case Latency of Byzantine Broadcast: A Complete Categorization](https://arxiv.org/pdf/2102.07240), Appendix C.1, Figure 10, starts BA after a timed certificate-forwarding interval, even for early deciders. The implementation below instantiates that invocation; it does not remove its cost or insert another round before the two-delay regional decision.

## Fixed context and value encoding

Fix one regional instance `I`, committee `V={1,...,n}`, strictly positive configured weights `w_j`, total `W`, and Byzantine weight `B<W/3`. Here Byzantine identities include every identity whose signing key the adversary obtains during the invocation; an instantaneous mobile-fault bound without protection of retained keys is insufficient. All honest nodes have the same context: configuration, regional generation, instance identifier, authorized broadcaster, previous resolved checkpoint, and canonical serialization rules. Let `Δ` bound honest-to-honest transmission **and all mandatory validation/data transfer** for one fallback message. If processing is modeled separately, replace `Δ` below by that combined bound.

Each honest node `p` starts completion at real time `s_p`; `max_p s_p-min_p s_p <= σ`, with equal clock rates. Set `Λ=Δ+σ` and `R=n`. Local round `r` occupies elapsed time `[(r-1)Λ,rΛ]`. Its send event is its left boundary and its receive deadline is its right boundary. At a boundary, process on-time received packets and the ending round before sending for the next round.

A semantic input is `z=⊥` or `z=H(DOM_VALUE,I,Enc(x))`, where `x` is a canonically encoded checkpoint proposal. Reserve a disjoint encoding for `⊥`. A non-bottom witness is `π=(x, QC_I(x), D_x)`, where `D_x` contains the full incremental signed packet range and any fixed-context ancestry proof required by `ValidCheckpoint_I`. Require:

```
ValidWitness_I(z, π) :=
    (z=⊥ and π=empty)
    or
    (z=H(DOM_VALUE,I,Enc(x))
     and ValidCheckpoint_I(x,D_x)
     and QC_I(x) has >2W/3 weight of distinct valid VOTE(I,H(x)) signers).
```

`ValidCheckpoint_I` is a deterministic predicate of the fixed instance context and supplied bytes. Witnesses for the same `z` may differ. Honest nodes retain any valid witness they accept. Quorum intersection and honest one-vote behavior imply at most one non-bottom semantic `z` can have a valid witness for `I`. Thus the admissible semantic domain in an execution has size at most two, including `⊥`. This size bound is used only for the simple communication estimate; the agreement proof does not need it.

The signature-chain payload contains `(DOM_DS,I,j,z)`, where `j` is the origin of one broadcast. A chain is

```
C_r = ((p_1,α_1),...,(p_r,α_r)),  p_1=j,
α_k = Sign_{p_k}(Enc(DOM_DS,I,j,z,k,C_{k-1})),  C_0=empty.
```

`ChainValid(I,j,z,C_r)` requires `1<=r<=R`, exactly `r` distinct configured identities, first signer `j`, canonical encoding, and verification of every nested signature over its exact predecessor chain. Domain separation binds both the enclosing BB instance and the origin; a signature from another origin, checkpoint, configuration, or message class is ineligible. The witness is detached from the chain: signatures bind its semantic value `z`, and every receiver independently checks the supplied witness. This permits interchangeable valid QCs without allowing substitution of another checkpoint.

## Algorithm: weighted completion at member p

Persistent variables for this one invocation:

```
input_p := snapshot(lock_p)              // immutable; bottom or a valid semantic z
witness[z] := a checked witness, if any
witness[⊥] := empty
A[j] := empty set, for every j in V      // accepted semantic values
forwarded[j,z] := false                  // global across rounds, not per chain
Inbox[r] := empty, r=1,...,R
```

The multicast primitive sends to every member, including the sender. A self-delivery is immediate. Messages received before local invocation are buffered under their instance identifiers. Malformed packets, invalid witnesses, and packets of other contexts are discarded rather than counted as a bottom vote.

```
At local completion time 0:
    z := input_p
    C_1 := ((p, Sign_p(Enc(DOM_DS,I,p,z,1,empty))))
    forwarded[p,z] := true
    multicast DS(I,p,z,C_1,witness[z])

On reception of DS(I,j,z,C_r,π):
    if not ChainValid(I,j,z,C_r): discard
    if not ValidWitness_I(z,π): discard
    if local completion time > rΛ: discard
    retain one valid π as witness[z]
    insert the packet in Inbox[r]          // buffer even if round r is in the future

At local completion time rΛ, for r=1,...,R:
    for each semantic pair (j,z) represented by a packet in Inbox[r]:
        insert z into A[j]
        if r<R and not forwarded[j,z]:
            choose a received valid C_r not containing p, if one exists
            if such C_r exists:
                C_{r+1} := append (p,Sign_p(Enc(DOM_DS,I,j,z,r+1,C_r)))
                forwarded[j,z] := true
                multicast DS(I,j,z,C_{r+1},witness[z])
    if r=R:
        for j in V:
            D_j := sole element of A[j], if |A[j]|=1; otherwise ⊥
        E := { z != ⊥ : sum_{j:D_j=z} w_j > W/3 }
        return the sole z in E if |E|=1; otherwise return ⊥
```

The send at the end of round `r` is precisely the beginning of round `r+1`. For deterministic implementation, choose the lexicographically least eligible chain after canonical encoding; this choice is not a consensus choice and can differ across honest nodes. An honest node never creates a chain containing itself twice. When a received chain already contains `p`, `p` necessarily signed and multicast its relevant prefix earlier, so absence of an eligible chain requires no new forwarding.

A Byzantine origin can send both `⊥` and the certified checkpoint; its common output is then `⊥`. An origin that sends nothing or only invalid messages also has common output `⊥`. No vote weight is fabricated for absent messages: `D_j=⊥` is simply the agreed default of origin `j`'s broadcast.

## Correctness argument for this implementation

**Round delivery.** Suppose honest `p` sends a round-`r` chain at time `s_p+(r-1)Λ`. Any honest `q` receives it by

```
s_p+(r-1)Λ+Δ <= s_q+(r-1)Λ+σ+Δ = s_q+rΛ.
```

Thus it is on time for `q`'s round `r`; arrival before that round is handled by buffering. The same inequality applies to round-1 sender messages. Including complete validity witnesses makes the acceptance predicate identical for all honest recipients.

**Per-origin accepted-set agreement.** Take any semantic value `z` accepted by an honest node in some round `r`. If its chain contains an honest signer in position `k`, that signer multicast the corresponding prefix in round `k` (either as origin for `k=1`, or after processing round `k-1`). The round-delivery inequality makes every honest node accept `z` by round `k`. If the chain contains no honest signer, it has length at most `n-1`, since at least one configured member is honest. The accepting honest node therefore can extend and multicast it in round `r+1`, unless it previously forwarded `z`; in that latter case its earlier multicast already made all honest nodes accept `z`. Consequently every semantic value accepted anywhere honestly is accepted everywhere honestly by round `R=n`. All honest accepted sets `A[j]` are equal, so all their `D_j` outputs agree. Forwarding each `(origin,value)` only once preserves this argument and bounds outgoing communication.

**Honest-origin validity.** An honest origin signs only its snapshot value in this DS invocation. Unforgeability excludes another value with its first signature. Its round-1 multicast reaches all honest parties on time, so all honest outputs of that origin's broadcast equal its snapshot.

**Weighted agreement and termination.** Every honest node has the same vector `(D_1,...,D_n)`. Applying the same weights and deterministic selector yields the same final value. Every invocation returns at its local elapsed time `nΛ`, independently of Byzantine participation.

**Unanimity.** If all honest snapshots equal non-bottom `z`, honest-origin validity assigns `D_j=z` to origins of total weight `W-B>2W/3`; every other candidate has origin support at most `B<W/3`. Therefore the selector returns exactly `z`. If all honest snapshots equal `⊥`, every non-bottom candidate has support confined to Byzantine origins, of weight at most `B<W/3`, so the selector returns `⊥`. This also proves the stronger property that any non-bottom output was the snapshot of at least one honest member: support exceeding `W/3` cannot consist entirely of Byzantine origins.

**External validity.** A non-bottom output occurs only among accepted values, each with a valid checkpoint/QC/data witness. Hash binding identifies the same checkpoint at every honest node. Unlike a raw QC revealed after completion, this output is selected from the agreed vector of fixed snapshot broadcasts; it cannot rewrite an earlier completed outcome.

**Composition with an early regional decision.** A regional early decision at local BB time at most `2Δ+σ` causes timely forwarding of its QC. Every honest node receives it before its local BB snapshot cutoff `3Δ+2σ`. Quorum uniqueness then makes every honest snapshot that same checkpoint. Weighted unanimity preserves it. If there was no honest early decision, completion agreement supplies the common outcome. The existing `DECIDED` export mechanism remains necessary for external consumers because mere possession of a QC is not a proof of timely early decision.

## Bounds and scope

The concrete completion bound, measured from each honest node's own snapshot/start event, is

```
L_BA = n_i (Δ_loc + σ).
```

The latest honest completion is at most `σ+L_BA` after the earliest completion start. Substituting into the existing conservative checkpoint spacing gives

```
τ_C >= 4Δ_loc + 3σ + n_i(Δ_loc+σ).
```

For `n_i=10` this is ten fallback rounds, regardless of how many low-weight Byzantine identities are present. The completion still runs after every good-case two-delay decision; it is not a failure-only communication cost. A separately justified identity bound `t_count<n_i` permits `R=t_count+1`, but the stake bound alone does not permit replacing `n_i-1` by `floor((n_i-1)/3)`.

Under checkpoint-QC uniqueness, each honest member multicasts at most two semantic values per origin across all rounds. Thus the simple implementation sends `O(n_i^3)` honest point-to-point messages across all `n_i` parallel broadcasts. With maximum checkpoint-plus-witness payload `L` bits and signature/identity entry size `κ`, each message is `O(L+n_i κ)` bits, for `O(n_i^3 L+n_i^4 κ)` honest transmitted bits. These estimates exclude adversarial flooding, as usual; a deployment must budget bounded processing and message size. Sending once per `(origin,value,round)` instead would increase the simple message bound to `O(n_i^4)`.

This is a correctness-oriented baseline. It establishes neither optimal weighted bad-case latency nor competitive communication efficiency. It does not introduce PBFT and does not alter the regional `2δ` honest-leader decision path. Reducing mandatory completion overhead, overlapping dependent checkpoint instances, or replacing the baseline requires a new composition argument.

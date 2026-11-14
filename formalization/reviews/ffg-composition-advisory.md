# FFG control-chain composition advisory

This is a bounded independent design review, not an edit to `main.tex`. It checks the proposed replacement of global PBFT by a checkpoint tree with Gasper-style fork choice and Casper FFG. The findings refer to the replacement design supplied in the review request; the old PBFT-based theorem cannot simply be retained.

## Summary

**Verdict:** The safety composition is viable with the explicit rules below. A proof of operational Gasper liveness is not supplied. Two critical errors would remain if activation were based on justification, or if an unqualified Figure 10 vote bundle were treated as a portable local decision certificate.

1. **Finality circularity:** a ROTATE record cannot require its own finality before voters can validate its immediate successor, because that successor is needed to finalize ROTATE.
2. **Justified authority:** justification does not uniquely choose a permanent branch. Activating a new region on justification permits conflicting authorities without necessarily producing an FFG slashing violation.
3. **Local export timing:** a raw Figure 10 quorum of VOTE signatures does not prove that any honest party committed that value before its deadline.
4. **Liveness scope:** eventual synchrony and responsive stake alone do not establish convergence of the modified fork choice, common health reports, common justified source, or a common target.

## Minimal immutable state and validity

Each signed control record has a unique hash committing to its complete contents, an epoch `e`, exactly one parent hash, a fixed configuration, input tip `(k,q)`, action, certificates, and any finality witness. Parent epochs strictly increase. Epochs are independent of order-data production. A globally scheduled proposer can issue a no-data control record even when the active region is unavailable; missing wall-clock epochs create no records by themselves. Multiple proposals/forks before finality are allowed.

Derive immutable branch state from the parent plus the record's contents. State includes the current generation/region, input tip, health history, and at most one pending rotation. A certificate learned later changes a validator's knowledge and fork choice; it does not retroactively mutate a previously hashed record's state.

`ADVANCE` must extend the parent's complete input history, contain valid local exported decisions for every new range, and carry greater-than-two-thirds global data acknowledgments bound to that parent anchor and exact range. Honest acknowledgers possess the complete data and local decision evidence. `HOLD` preserves the tip. `ROTATE` also preserves the tip and creates a pending transition naming exactly the successor, generation, retained tip, reason, and readiness certificate. The departing generation remains explicit. Candidate selection/skipping must be a deterministic function of authenticated history; absence of a readiness message at one observer is not globally verifiable evidence that an earlier candidate is unready.

While a rotation is pending, descendants without an activation witness can only confirm/hold the same tip, with the same pending transition. They cannot advance either generation or insert another rotation. A later descendant may carry a complete finality proof of that exact pending ROTATE; it then deterministically activates the successor before processing its own action. Its proof must be independently verifiable and must not depend on votes for the record carrying it. New-generation batches cite that ROTATE and its finality proof. Data created in anticipation of finality remains speculative.

Example: justified `R_e=ROTATE`; `H_(e+1)` retains its tip and pending state; the link `R_e -> H_(e+1)` finalizes `R_e`. A later record `D_(e+2)` can include that link plus the justification proof of `R_e`, activate its named successor, and advance. Neither `R_e` nor `H_(e+1)` changes hash or state when the link arrives. The new region may begin locally as soon as it possesses finality evidence, while the next global ADVANCE carries the evidence explicitly.

This also prevents a successor from acquiring irreversible authority through a transient branch. A competing pending rotation may exist on another branch, but both cannot finalize under the FFG assumptions. Readiness signatures must bind the exact activation package and all retained data, rather than a numeric position alone.

## FFG rules and hidden certificates

A vote binds configuration, source epoch/hash, and target epoch/hash; source is an ancestor of target. A supermajority link from a justified source justifies its target. A justified source is finalized by a link to its consecutive-epoch descendant. Double-target and strict-surround conditions apply to complete signed messages, not merely the input roots. Persist the complete signing history, and abstain whenever a proposed vote would violate either condition. Genesis is justified/finalized.

Fork choice first excludes histories incompatible with the greatest known finalized checkpoint, then starts from the highest-epoch known justified compatible checkpoint and applies latest-message weighting among eligible descendants with a deterministic tie rule. Source selection must respect that justified root. If the selected target does not admit a slash-safe vote from an eligible justified source, abstain and continue dissemination; do not silently lower a source or assert that voting must always be possible.

Justification and finality are properties of complete evidence; an observer may learn that evidence later. A hidden certificate is not invalid merely because the proposer did not include it in a previous record. Its later arrival can change the provisional head/source. It cannot override an incompatible finalized checkpoint under the stated fault assumptions. Ordinary quorum intersection by itself establishes neither cross-epoch compatibility of justified checkpoints nor convergence of views.

Concrete counterexample to justification-based activation: everyone can vote `G_0 -> R_2` and, before learning the aggregate certificate, later vote `G_0 -> R'_3`, where the two rotation records lie on different branches. The intervals share their source and have different target epochs, so these votes need violate neither FFG rule. Both rotations may be justified. Neither should activate a region until finalized.

For no-data progress, use distinct records and hashes even when `(k,q)` agrees. If `C_(e-1) -> H_e -> H_(e+1)` are actual supermajority links, `H_e` is finalized. Two reports, two merely proposed HOLDs, or two justified HOLDs without the required consecutive link do not establish that finality. The second HOLD may have been justified from an older source. If health policy requires the first HOLD finalized, check its proof explicitly.

The underlying FFG safety theorem and the distinction between finality and plausible liveness are from [Casper FFG, Sections 2.1–2.2](https://arxiv.org/pdf/1710.09437). Gasper additionally makes the checkpoint/epoch pair and local-view semantics explicit; this proposed control-record construction is a specialization/design choice, not a verbatim implementation of [Gasper, Sections 3–4](https://arxiv.org/pdf/2003.03052).

## Figure 10 external certificate boundary

[Appendix C.1, Figure 10 and Theorem 16](https://arxiv.org/pdf/2102.07240) require synchrony with known `Delta`, bounded start skew, and `f<n/3`. Receiving `n-f` VOTE signatures permits early commit only by the stated local deadline. The fallback invokes BA at its specified later time; honest parties that committed early still participate. The `2 delta` bound is local good-case commitment, not global availability or certificate-delivery latency.

Counterexample: with `n=4,f=1`, the Byzantine broadcaster obtains votes for `v` from two honest parties plus itself, but withholds its own vote from honest parties. No honest party sees three votes before fallback, so all enter BA with bottom and decide bottom. The broadcaster later exposes a valid three-signature vote bundle for `v`. That bundle alone cannot certify a decision of `v`.

Export `DECIDED(instance_id,raw_decision,application_result)` signatures emitted only after actual Figure 10 commitment, and require `n-f` distinct regional signers. Agreement plus at least one honest signer makes this portable. Bind generation, height, parent, broadcaster and configuration in the instance/result. Gathering these acknowledgments adds a communication step beyond the local `2 delta` bound. Keep equal-vote `n,f` assumptions unless a weighted extension is separately proved.

The fallback BA contract must also expose its decision domain/validity. If it only guarantees unanimous-input validity, do not infer that an arbitrary nonbottom output is a valid application batch. A deterministic total interpretation may map bottom or invalid raw output to a GAP, while accepting a batch only with its complete authenticated, externally valid envelope and bytes. This interpretation and availability contract must be stated, not attributed to Figure 10.

## Safe claims and limits

The safety dependency chain is: local synchronous BB agreement -> one local decided result per instance -> parent-checked local prefix; fixed-weight FFG accountable safety -> compatible finalized control histories; deterministic record validity and finality-gated activation -> one finalized generation transition and one retained input prefix; deterministic execution plus the settlement verifier -> faithful A5 acceptance. LMD-GHOST only chooses a provisional head.

A3 should mean justified, data-backed, provisional acceptance. A4 is the durable finalized boundary. A3 can be reorganized, and a geographically abandoned speculative suffix may include A2/A3 data. On the selected branch, finalizing ROTATE also finalizes its ancestors and hence its retained tip; recovery cannot silently discard part of that finalized prefix. If the chosen retained tip is longer than the previously known A4 tip, the rotation's finality promotes that longer tip to A4.

The liveness claim should assume eventual common eligible targets and a common justified source that permits slash-safe voting, sufficient dissemination/cutoff time, a responsive supermajority, enough successful consecutive epochs, and a ready successor. Under those conditions, the specified links and activation eventually occur. This is a conditional progress argument. It is not a proof that arbitrary modified Gasper executions converge, and it gives no unconditional geographic deadline.

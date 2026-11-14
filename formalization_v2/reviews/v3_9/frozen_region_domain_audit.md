# Frozen regional-domain and ingress audit

## Summary

**Final verdict:** The frozen source supports bounded communication between correct participants in the same regional domain and explicitly retains data service after Completion. It does not uniquely specify that domain across different authority snapshots or the full participation lifetime of retired nodes. Therefore neither an unconditional cross-AUTH/post-retirement guarantee nor an unconditional impossibility follows. The original ingress condition does not imply request awareness before a newly introduced report event.

This is an independent source and assumption audit, not a review of a proposed protocol repair. Only the original frozen files listed below were used for substantive conclusions. No later protocol, companion explanation, or prior author/reviewer report was read. `Terminated` and `report` are treated as names supplied by the audit question: neither is an original frozen engine state or request-ingress milestone.

Findings:

1. **Justification Gap in an unconditional cross-AUTH inference.** “Within region $i$” has no explicit AUTH guard, so it must not be silently rewritten as “only within one AUTH.” Conversely, the source defines regional members through session snapshots and does not define a permanent physical-region universe. The broader cross-snapshot domain is a coherent reading, but is not uniquely formalized.
2. **Justification Gap in an unconditional post-retirement inference.** A stopped production engine is not a retired node. Complete engines explicitly keep serving data. A persistent node-level duty can use the existing network bound for endpoints that remain within its correct-participant domain. Whether every old node remains in that domain after retirement or while offline is not settled by the frozen source.
3. **Critical Error in the inference “eventual ingress implies before-report awareness,” if that inference is used.** An eventual receipt can occur after the report, including for every individually designated member. This is a quantifier/timing counterexample to that inference, not an impossibility theorem for a later protocol.
4. **No support for unrestricted mobile corruption of historical snapshots.** The lifetime fault clause counts old-key compromises while old certificates remain accepted. Removing a key from a later AUTH does not release its old-snapshot fault budget.

## Detailed verification log

### 1. Source boundary and notation

References below use:

- **M:** `formalization_v2/revisions/v3_1_frozen/model_and_assumptions.tex`.
- **R:** `formalization_v2/revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex`.
- **I:** `formalization_v2/revisions/v3_1_frozen/sync_hotstuff_rr_part1_core.tex`.

M is the separately frozen model section also appearing in R. Its SHA-256 is `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`; R is `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034`; I is `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3`.

M 31–39 says:

> “For each $(r,g)$, L1 supplies at most one immutable authority.”
>
> “$\mathsf{AUTH}_{r,0}$ fixes $\mathit{cfg}_r$, including the members, weights, $F_i$, $\Delta_i$ and $K$ of each region” and “For $g>0$, only the leader schedule changes.”

M 41–44 additionally forbids rebinding an already assigned authority when another branch reuses the same numeric $r$. M 145–147 says:

> “Membership and weights stay fixed for a whole session, including its recoveries. … A later session checks old certificates against the old snapshot.”

R 525–528 calls $V_i$ the session region's “regional members,” and R 820 identifies $\mathit{cfg}_r$ as the session's “fixed voting snapshot.” R 963–970 explicitly interprets $\Delta,F$ in regional protocols as the parameters of the session's region and authority.

**Verified:** Reusing the same $r$ or changing its recovery generation does not change its snapshot. Different numeric session indices are not required to share members, weights, fault bounds, or delay parameters. The notation suppresses some configuration dependence; writing $V_i^{(r)}$, $F_i^{(r)}$, and $\Delta_i^{(r)}$ in this audit makes that dependence explicit without changing the source.

**Not established:** A fixed set $V_i$ across all numeric $r$; a membership overlap across successive AUTHs; an identity map from cryptographic keys to persistent physical hosts; or a formally defined set of every process ever physically located in region $i$. “Physical region” is not itself a defined network-domain object in these files.

### 2. What the communication clause does and does not quantify over

M 171–179 states:

> “Messages use authenticated communication channels.”
>
> “Within region $i$, the actual delay of a correct-to-correct message is $\delta\le\Delta_i$; $\delta$ may differ across messages.”
>
> “The delay bound includes delivery and processing of complete validation dependencies.”

**Verified conditional implication:** Once a transmission is established to be a correct-to-correct message within the intended region-$i$ domain, its complete-delivery/processing bound is $\Delta_i$. The sentence has no express exclusion based on message type, AUTH inequality, engine Completion, or old branch closure. In particular, a claim that cross-AUTH messages are *expressly excluded* is false as a description of the literal wording.

However, the sentence does not define its endpoint universe, time domain, or correctness predicate. The surrounding snapshot notation admits the following distinct formalizations.

**Reading G: persistent regional process domain.** Let $U_i$ be the persistent processes of physical region $i$, containing the relevant members of every $V_i^{(r)}$. Let $C$ be correct processes, and suppose the domain continues to include those processes while their node-level duties persist. Then a fixed-profile reading is

$$
\forall s\ \forall u,v\in U_i\cap C:\quad
\operatorname{Send}(u,v,m,s)\Longrightarrow
\operatorname{Process}(v,m)\le s+\Delta_i.
$$

Under this reading, two correct endpoints from different AUTHs communicate within the bound even if their committees are disjoint. AUTH equality is irrelevant to transport. If $\Delta_i$ can change with $r$, an additional interpretation of which snapshot's parameter governs the transmission is needed—for example, its named protocol context. That parameter-binding issue is also suppressed by the original notation.

**Reading S: regional snapshot context.** For each session context, “region $i$” names that context's regional member set. A corresponding formula is

$$
\forall r\ \forall s\ \forall u,v\in V_i^{(r)}\cap C_r:\quad
\operatorname{Send}_r(u,v,m,s)\Longrightarrow
\operatorname{Process}(v,m)\le s+\Delta_i^{(r)}.
$$

This supplies all intra-snapshot transmissions required by the original regional engine, including recoveries, but by itself says nothing about a message from $u\in V_i^{(r)}\setminus V_i^{(r')}$ to $v\in V_i^{(r')}\setminus V_i^{(r)}$. Both sets may have correct majorities without intersecting. Silence about this pair is absence of a guarantee, not a prohibition on its communication.

**Quantifier verdict:** G is the direct broad reading of the unqualified words; S is supported by the configuration-local usage of $V_i,F_i,\Delta_i$ throughout the regional model. Neither the word “within” alone nor AUTH immutability resolves which endpoint universe was intended. S's context restriction is an interpretation, not an explicit source exception. G's permanent physical universe is likewise an interpretation, not an object defined in the source. An inference that needs one of them must state which it uses.

This does not require an invented distinction between an honestly behaving process in one AUTH and the same simultaneously honest process in another. A globally correct process follows every applicable duty. The uncertainty is which processes and participation times the frozen assumptions cover; authority authentication alone does not make a recipient correct or available.

### 3. Engine termination, participation lifetime, and availability

R 916–939 defines only Active, Frozen, and Complete. Its Complete definition says:

> “Production in session $(\beta,r)$ is over, even if the external handoff is still pending. The engine only serves data.”

The Completion operation, R 1092–1102, stops proposals, votes, progress checks, term changes, recovery, and timers; then it says:

> “Offer the complete terminal result to the Publish Layer.”
>
> “Keep serving data.”

R 1560–1564 says external branch closure does not run Quit or Completion for old engines. R 2139–2143 leaves early stopping of closed engines and staking retirement as open questions.

**Verified:** The original source does not equate the end of production with disappearance of the process, deletion of the old AUTH, or cessation of all old-session service. A proposed `Terminated` engine state cannot be assumed, merely from its name, to remove separate node-level duties. The frozen source itself already contains duties that survive Completion.

For persistent correct endpoints in the same old AUTH, adding a bounded local send/reply action is an ordinary protocol addition. Under the all-time form of the network assumption, changing which messages a correct participant sends does not itself add a new environment availability assumption. This is particularly clear for two still-participating old members: no cross-AUTH transport conclusion is needed.

Conversely, a message-delay bound is not a spontaneous-send rule. The desired result also needs the sender to know the object, a protocol rule triggering transmission, and the receiver to be in the modeled processing domain. No delay clause by itself causes a new report, retains an unknown request, or creates a reply obligation.

The source separately mentions offline behavior:

- M 211–214 bounds L1 visibility for “every correct online participant.”
- M 220–234 preserves authenticated history and globally accepted DATA “without the original producing region staying online”; a returning node catches up, with no constant bound for arbitrarily long history.
- M 238–249 gives Espresso visibility to each correct participant “that stays online and keeps up with the history.”
- M 282–283 requires offline or unsynchronized participants to catch up before confirming new output.
- R 1322–1325 says a bound on every replica's local commitment needs “the relevant online and receipt-processing conditions.”

These clauses make offline participants and nonconstant catch-up meaningful in the overall system. They do **not** expressly authorize an old regional replica to ignore its still-applicable data-serving duty, and they do **not** expressly qualify the intra-region delay sentence by “online.” Thus neither of these shortcuts is justified:

1. “The model mentions offline nodes, therefore any still-correct old regional member may vanish while a mandatory request to it is pending.”
2. “The regional delay sentence has no online qualifier, therefore every historical key holder must remain an available process forever.”

Two temporal completions remain possible:

- **Persistent-role interpretation:** Correct regional replicas remain in the synchronous domain for every surviving protocol duty. Failing that duty or its processing bound is not an admissible correct execution. Under this interpretation, post-engine-termination service can be a protocol change alone.
- **Participation-interval interpretation:** Correctness/communication guarantees apply during a role's participation interval $P_{i,r}(v)$, with retirement or offline periods governed separately. The network formula is conditional on the applicable interval covering the transmission and required processing. Under this interpretation, extending mandatory service beyond the original participation interval requires an availability/retirement premise in addition to a new protocol rule.

The participation-interval guard is not written in M 172–173; it is a possible formal completion motivated by the offline/retirement clauses. The frozen text neither defines these intervals nor sets their endpoints. In particular, assigning their endpoint to Completion would contradict “Keep serving data” unless the remaining data duties had separately been discharged. This audit therefore does not certify an offline counterexecution that simply violates an outstanding frozen duty.

**Verdict for same-old-AUTH service:** It follows conditionally for still-correct, still-covered old participants under surviving duties. It is not refuted merely by engine termination. An unconditional guarantee for arbitrary retired/offline old participants requires a temporal-domain resolution absent from the source.

### 4. Adaptive corruption and the lifetime fault bound

M 137–150 says regional Byzantine weight is at most $F_i<W_i/2$ and certificates require weight $>F_i$. It then specifies:

> “The bound $F_i$ counts every key of the snapshot that is compromised while its certificates are still accepted. Compromise of old keys after that is outside this model.”

For a fixed snapshot, let $B_{i,r}^{\mathrm{life}}$ include every Byzantine/compromised key counted over the lifetime in which its certificates are accepted. The stated interpretation requires

$$
\sum_{v\in B_{i,r}^{\mathrm{life}}}\omega_{i,v}^{(r)}\le F_i^{(r)}.
$$

Thus the uncorrupted remainder has weight at least $W_i^{(r)}-F_i^{(r)}>F_i^{(r)}$. A certificate of weight greater than $F_i^{(r)}$ cannot consist entirely of lifetime-compromised keys. Adaptive timing does not permit cycling through more than $F_i^{(r)}$ old weight while those certificates remain accepted. Membership replacement in a later AUTH does not reset this inequality.

This is an integrity/fault-budget conclusion. It does not itself prove that the uncorrupted keys' processes remain online, know every request, have entered a new session, or have a bounded request-retrieval service. Those require the communication/participation and protocol premises discussed above. Conversely, an offline episode is not automatically a permitted adversarial corruption: whether it is a fault or allowed nonparticipation depends on the unsettled role semantics.

I 153–154 has a fixed Byzantine set; I 196–206 disallows replenishing it by new corruptions or identities and says changing membership or quorum weights needs a separate cross-configuration argument. R imports the regional engine per session (R 957–970), while M supplies the more explicit historical-key lifetime protection. One cannot silently turn Part I's fixed-set premise into a global fixed committee across all Part II AUTHs, nor discard M's lifetime constraint when analyzing adaptive old-key attacks.

### 5. Request ingress supplies eventual reachability, not a report cutoff

M 310–312, duplicated at R 503–505, states:

> “Correct clients' admissible requests reach the regions that continue serving. Requests fit in DATA batches and have only finitely many older requests ahead of them in each local queue.”

R 1841–1858 says nodes queue by first receipt, may have different queue orders, and remove a request finally only when it is included in the global prefix. A discarded-suffix request remains pending. These are rules for received/known requests. They do not specify how an unreceived request is learned by a new committee.

The frozen Part I fairness clause is more explicit about endpoints, but still eventual:

> “Correct-client requests eventually reach all correct replicas.” (I 214–220)

Its request-liveness proof begins “Fix a request after dissemination” (I 936–944). In the fixed finite Part I committee, individual eventual receipt gives a finite maximum receipt time. It gives no claim about an earlier protocol event.

Let $a_v(x)$ be the time correct member $v$ first learns request $x$, and let $\rho_v$ be the time of a proposed report. Even granting the stronger assumption

$$
\forall v\ \exists a_v(x)<\infty,
$$

one cannot infer

$$
\forall v\quad a_v(x)\le\rho_v.
$$

For a single member, choosing $a_v(x)=\rho_v+\varepsilon$ already refutes the timing inference while preserving eventual delivery. For rotating, disjoint committees $V^{(1)},V^{(2)},\ldots$, choose report time $\rho_k$ and finite arrival $a_v(x)=\rho_k+\varepsilon$ for every $v\in V^{(k)}$, during a later part of its serving interval. Every committee eventually receives the request, and every report precedes that receipt. The example establishes absence of **before-report awareness** only; it does not by itself show permanent censorship or impossibility of a later forwarding mechanism.

With indefinitely many new members, even $\forall v\,\exists a_v<\infty$ does not imply one finite time after which every future member already knows $x$. If “reach the regions” instead means receipt by some member of each serving region, that is weaker still. The source does not choose a per-member ingress cutoff or a membership-handoff dissemination barrier.

The local finite-predecessor condition controls queue service after receipt; it does not control $a_v(x)$. Persistence of pending requests protects already known requests across branch changes; it does not establish continuity of knowledge across disjoint memberships. Regional synchrony can bound a subsequent *in-domain correct transmission*, but the original ingress clause neither initiates that transmission nor orders it before the new report.

### 6. Dependency and classification boundary

The relevant proof dependency is:

$$
\text{request/result known}
\;\land\;
\text{applicable send/reply duty}
\;\land\;
\text{correct covered endpoints}
\;\land\;
\text{applicable regional bound}
\Longrightarrow
\text{bounded delivery/processing}.
$$

An AUTH provides authenticated snapshot membership and parameters. The lifetime $F$ bound provides a correct/uncorrupted weight guarantee within that snapshot. Neither alone discharges every premise in this chain. Similarly, bounded delivery after a send does not imply knowledge before an unrelated earlier report.

No original-source protocol repair is supplied here. The review identifies exactly which premise a later argument must establish and preserves both coherent domain readings. There is no unconditional impossibility verdict and no unconditional cross-AUTH, offline-retirement, or report-awareness guarantee.

Validation performed: direct line-by-line inspection of the cited frozen clauses, exact-term searches for engine states/participation/offline/requests, and source SHA-256 recording. No executable/model-checking validation or proof of a later revision was performed. The three source files were not edited.

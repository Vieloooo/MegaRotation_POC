# Summary

**Final Verdict:** The frozen artifact is invalid as literally specified because the regional proposal-validation predicate omits the broadcaster's proposal signature, invalidating the claimed honest-broadcaster validity; the service-liveness argument additionally has two justification gaps concerning READY behavior and the connection between regional and global responsiveness. The cap-two authenticated broadcast, weighted completion, regional exported-certificate uniqueness, and global prefix-safety argument otherwise check out under the artifact's explicit strong synchronous envelope, subject to the qualifications below.

Artifact: `formalization/build/rewrite/review-v1.tex`, 1,244 lines.

SHA-256: `631a6c1c4f0a3bcc7129fe6e80b55288d4c28e879d89e3b81ef3b6e7fe56859f`.

Mode: independent verifier only, following `/Users/jin/.codex/skills/proof-verifier/SKILL.md`. This report does not edit or repair the protocol. All source-line references below refer to the frozen artifact, not a subsequently edited `main.tex`.

## List of Findings

- **F1 — Critical Error under the literal acceptance predicate.** **Location:** lines 317–320, “`ValidProposal(I,x)` checks the exact certified predecessor, configured leader and view, consecutive positions and parent digests, all body commitments, all ORDER signatures, and the count/size bounds”; lines 432–435, “On the first valid proposal”; lines 822–824, “An honest responsive leader's eligible proposal is decided.” **Issue:** the specified predicate does not check the leader's PROPOSE signature on the selected proposal digest. Authorized ORDER signatures on the component mini-blocks do not authenticate the broadcaster's selected batch endpoint. A relay can race a strict prefix of an honest leader's signed multi-block proposal as another predicate-valid proposal. The honest-broadcaster-validity proof is therefore invalid for the protocol literally given. If proposal-envelope authentication was intended to be implicit, that is a missing acceptance guard; it is not present in the explicit predicate or its witness definition.
- **F2 — Justification Gap.** **Location:** lines 605–609, “It signs one report for this epoch; READY refers to the parent's exact adopted tip”; lines 946–948, “Responsive members of a healthy standby region ... originate true READY reports.” **Issue:** the protocol specifies what a true READY means but does not require an honest ready member to set it to true. False READY reports remain valid indefinitely. The liveness proof imports a prescribed behavior that is absent from the protocol. “Healthy eligible region” is not defined sufficiently to rule out this execution by hypothesis.
- **F3 — Justification Gap.** **Location:** lines 183–187 define the global set `H_G` and separate regional sets `H_i`; lines 650–651 require more than two thirds of a region in READY reports; lines 947–948 infer a ready quorum; lines 707–709 infer a common regional start from the globally responsive set. **Issue:** local health establishes regional communication and responsive regional weight, whereas successful READY origin broadcasts and guaranteed receipt of the activation proof depend on the global envelope. No stated relation establishes that the healthy successor's requisite regional weight is also guaranteed to participate successfully in the global rounds. The inference from “healthy standby region” to a READY quorum/common initialized regional quorum is incomplete.

# Detailed Verification Log

## 1. Model, fault envelope, and dependency map

> Lines 179–187: “The Byzantine set B is fixed”; regional Byzantine weight is below one third; “an always-responsive honest set H_G of weight >2W/3”; a live regional interval has a separate always-responsive honest set H_i of weight >2W_i/3.

The proof assumes fixed membership, static Byzantine identities, uncompromised historical honest keys, persistent signing history, and a fixed always-responsive honest global cohort. It is not an eventual-synchrony proof. Lines 193–194 expressly make synchrony a safety premise for signed-chain completion. Consequently, the standard late-chain concern for asynchronous Dolev–Strong executions is outside this theorem's stated envelope, rather than an omitted assumption.

The responsive-cohort convention is important: authenticated-broadcast agreement is proved among the responsive honest participants. Other identities can be treated adversarially for that argument, even if they are honest but unavailable. The regional Byzantine-weight bound remains independently necessary for exported-certificate uniqueness after a regional network failure.

The main dependencies are:

1. Signature/hash assumptions and quorum arithmetic.
2. Cap-two authenticated origin broadcast.
3. Weighted completion and regional BB agreement.
4. Persistent DECIDED signing plus certified predecessor checks imply a unique exported regional chain.
5. Global report agreement plus deterministic transitions imply the common global chain and adjacent FFG links.
6. That chain and data retention imply permanent delivered-prefix safety and retrieval soundness.
7. Service progress additionally requires prescribed READY behavior, a suitably connected healthy successor, propagation of fresh regional evidence, and the stated capacity/tenure bounds.

F1 invalidates the honest-broadcaster-validity branch of item 3. It does not by itself refute quorum uniqueness, the cap-two primitive, or the separate safety argument for delivered ordered bytes. F2–F3 concern item 7 and its initialization assumptions.

## 2. Quorum intersection

> Lines 767–774: “Two quorums ... intersect in weight greater than W_Q/3.”

Correct. The inequality follows directly from inclusion-exclusion, and a Byzantine set of strictly smaller weight cannot cover the intersection. Applying the analogous calculation to the complement of an always-responsive cohort is also legitimate under the explicitly stronger effective-unavailability bound.

## 3. Cap-two authenticated broadcast

> Lines 1181–1185: “distinct configured identities, first signer o, and each nested signature”; witnesses must satisfy the common complete-object predicate.

These are the relevant chain-authentication conditions. Distinct identities are essential to the length-n argument; the fixed instance domain prevents cross-instance reuse. Semantic equality excludes alternative certificate/signature representations, so two witnesses for one value do not falsely exhaust the cap.

> Lines 1195–1205: “Associate a length-r chain only with round r”; ignore objects not completely valid by the deadline; retain at most two values; append a signer only when absent; forward a retained value at most once; continue prescribed forwarding after retaining two values.

The algorithm enforces the needed timed acceptance and forwarding obligations. Receiving early is harmless because processing remains associated with the chain's designated round. Processing the final round before returning is explicit. An honest responsive origin's round-one self-delivery is also covered.

> Lines 785–794: an honestly retained value reaches every responsive honest member, unless that member already retains two values.

Correct under the common delivery bound and immutable semantic validity predicate. If the received chain contains an always-responsive honest signer, that signer previously sent the corresponding prefix at the prescribed boundary. If it does not, the chain has length at most n-1 and the responsive receiver can extend it, unless an earlier send of that value already provides the required dissemination. A length-n chain necessarily contains an always-responsive honest signer because at least one such identity exists in the n-member committee.

> Lines 796–804: if one honest responsive member retains two values, all return bottom; otherwise a nonempty retained singleton propagates to all.

Correct. Applying the propagation property to each of two distinct values forces every other responsive honest member either to retain both or already to have a two-element set. Equality of the particular two-element sets is unnecessary. If no responsive member has two values, the same property forces every nonempty set to be the same singleton. Honest-origin validity follows from its unique origin signature and round-one complete delivery. I found no counterexample to the cap-two argument in the supplied algorithm.

This verifies the concrete supplied construction directly; it does not import a stronger theorem from the cited Dolev–Strong paper.

## 4. Weighted completion

> Lines 467–475: parallel origin broadcasts; return the unique value with report weight above two thirds, otherwise bottom.

Correct under the regional live-cohort assumption. The common output vector gives agreement. Two values cannot each have support above two thirds because each origin contributes its weight to at most one output. If all responsive honest inputs equal a valid non-bottom value, their origin outputs alone exceed the threshold. If all equal bottom, all non-bottom support lies in the complement, whose weight is below one third. The witness predicate supplies external validity. Running n_i broadcasts in parallel costs n_i round lengths under the assumed reserved capacity.

## 5. Regional BB agreement, validity, and timing

> Lines 436–444: forward a complete vote quorum; fast-decide by 2 Delta_R; freeze completion inputs at 3 Delta_R.

The agreement/completion argument is sound for fast decisions by responsive honest members: a timely forwarded complete quorum reaches all members of the responsive cohort by the input-snapshot deadline. Deadline messages are processed before the deadline event. There is at most one non-bottom vote-quorum value because honest members vote once per instance. Weighted unanimity therefore preserves a responsive fast decision. When none of those members fast-decides, completion agreement supplies a common decision.

The proof at lines 829–830 says “an honest member” although its statement concerns responsive honest members; the forwarding deadline is justified only for the responsive sender class. Reading that case as a responsive fast decision is consistent with the theorem statement. It should not be extrapolated to a node outside the guaranteed delivery cohort.

> Lines 428–431: the leader signs `(PROPOSE,I,H(x))`.
>
> Lines 315–320: the validity witness consists of the full range and its producer signatures; the enumerated `ValidProposal` checks include ORDER signatures but not PROPOSE authentication.

**F1: Critical Error under the literal predicate.** Let an honest leader propose two signed mini-blocks and endpoint h+2. An adversary obtains the first signed mini-block, constructs a proposal x' with the same certified predecessor but endpoint h+1, and sends that shorter complete object to voters before the leader's complete two-block proposal arrives. The network bound allows this ordering. The shortened proposal passes every check explicitly listed in `ValidProposal`: authorized ORDER signature, matching parent, legal view/instance, consecutive position, and legal size/count. It has no valid leader PROPOSE signature for H(x'), but that signature is not required by the predicate as defined.

Take a four-member equal-weight committee with one Byzantine member and an honest leader. The two other honest members can receive x' first and vote for it; together with the Byzantine vote they form the strict-supermajority quorum. The leader can already have voted for x. The committee can nevertheless fast-decide x' from the quorum, so the leader's actual proposal x is not decided. This directly defeats the inference at lines 837–839 that an honest leader's proposal receives all responsive honest votes.

The affected honest-broadcaster-validity conclusion is unverified. The separate quorum-uniqueness/agreement argument remains valid for the accepted value domain and is checked independently above. No protocol repair is supplied here.

> Lines 484–490: regional spacing at least (n_i+5) Delta_R.

The accounting is sufficient under the complete-object and capacity assumptions: three delays to the completion start, n_i rounds of completion, and two delays for endorsements/certificates. It does not imply successive confirmed batches at two-delay spacing, and the text correctly disclaims that inference.

## 6. Unique exported regional chain

> Lines 338–343: DECIDED endorses H(R_u); its verifier checks predecessor and transition; one endorsement per instance persists across restarts.
>
> Lines 849–857: quorum intersection and predecessor-certificate induction give uniqueness, even if a failed region cannot complete BB.

Correct under the regional Byzantine-weight and historical-key assumptions. Signing H(R_u) binds the parent as well as the outcome, so two certified parents for the same regional instance would require an honest double endorsement. Every new instance requires its predecessor certificate. The induction therefore provides a unique exported chain without assuming agreement among all internal fast decisions during an outage.

The content/authority part of this lemma concerns the signed mini-block data and deterministic transitions. F1 does not invalidate that narrower property: its strict-prefix proposal still contains legitimately signed producer data. It does invalidate the separate claim that honest-broadcaster validity forces the leader's particular selected endpoint.

## 7. Global report agreement, transition, and finality

> Lines 602–627: common parent/justification; N rounds of report agreement; deterministic target; adjacent FFG vote; two certificate-delivery phases; one margin phase.

Under the stated global cohort and synchronized-clock assumptions, the induction is sound. Every honest responsive reporter uses the same parent and effective activation context; authenticated broadcast provides the common report-output vector. Certificate witness encodings are excluded from semantic identifiers at lines 324–326, which is needed when members assemble different signer subsets for the same certified fact.

> Lines 642–664: pending rotation only CONFIRMs; otherwise choose the longest compatible regional extension; ROTATE retains the parent's tip and cursor; no rotation adopts the selected extension.

Correct for prefix safety. All valid reported extensions originate at the same cursor and belong to the unique exported regional chain. Thus selecting by longest chain cannot select an incomparable data history. A gap advances the regional cursor without deleting data. ROTATE intentionally does not combine adoption of a new extension with readiness for the old tip, avoiding a readiness/tip mismatch. The new extension in that case remains only a preview.

> Lines 688–703: an adjacent link finalizes its justified parent; the successor starts at the fixed r+2 boundary; a later descendant records activation without rewriting previous checkpoints.

The dependency order is consistent. C_r can create a pending rotation; C_{r+1} preserves it and supplies the link finalizing C_r; the r+2 pre-transition consumes that existing proof. It is not a circular proof requiring a checkpoint to be finalized before its own voter can validate it. The epoch budget gives the globally responsive cohort the proof before the fixed activation boundary.

All responsive honest global members sign the same adjacent link, so their weight suffices without cooperation from an unavailable active region. Persistent target-epoch signing constraints rule out another quorum-certified target at the same epoch. The supplied synchronous-chain proof does not need to assume FFG itself resolves inconsistent report vectors.

The further statement that a healthy selected regional quorum thereby has a common initialized R_0 is subject to F3 below: the globally responsive cohort and locally live cohort are separately defined.

## 8. Permanent prefix safety and data repair

> Lines 530–537: verify finality and data, append only undelivered contiguous positions, and remove only preview entries beyond the finalized rotation's retained tip.

The permanent-prefix safety argument follows from the unique exported regional chain and monotone global transition. Every finalized checkpoint lies on the common global chain, and every subsequent adopted tip extends or preserves the earlier one. A ROTATE can discard an unadopted local preview, but cannot discard globally adopted data, whether or not that data was already finalized at the moment the outage occurred. Generation binding prevents accepting an old producer's post-handoff data as a new global extension.

The cryptographic verification claims correctly concern ordered bytes and authorized statements, not transaction execution. Changing a serving peer does not affect the accepted object's commitment or authority proof.

> Lines 922–928: globally responsive members retain the complete adopted prefix before voting; retrieval with a failed suppliers costs at most (a+1) theta.

Correct given the explicit retention and request-deadline assumptions. Global voters retain data before providing the certificate, and their fixed responsive cohort prevents a globally adopted prefix from existing only in a completely lost region. The retrieval inequality is a direct sum of bounded attempts; multiple chunks require a sum rather than one constant deadline. This is an assumption-backed availability result, not a claim that signatures alone reconstruct bytes.

## 9. Liveness and startup dependencies

> Lines 940–944: repeated bottom outcomes advance the local schedule and eventually reach an honest responsive leader.

The local schedule reasoning is valid as a progress argument: if no batch is confirmed, consecutive failures cannot keep resetting the counter; a planned boundary can only cause an earlier leader change. The claim of confirmation of the honest leader's exact proposal inherits F1, although the attack described there still confirms a nonempty authorized prefix and does not independently refute all possible stream progress.

> Lines 946–949: healthy standby members “originate true READY reports” and “their weight gives a ready quorum.”

**F2: Justification Gap.** G1 requires one report and defines READY's reference tip, but does not state when an honest member sets READY to true. ValidReport does not reject false READY from a fully synchronized member. Consequently, the explicit algorithms allow standby members to keep issuing complete, valid reports with no extension and READY=false. Once the active region stops, the no-progress counter grows, but transition T3 finds no ready successor. The liveness proof excludes this execution by an unstated behavioral premise. The undefined phrase “healthy eligible region” cannot be silently expanded to add that premise during verification.

Assuming true READY is in fact prescribed, continue independently:

**F3: Justification Gap.** A live regional set H_j has more than two-thirds regional weight and the regional delay guarantee. Origin-broadcast validity on the backbone is guaranteed for H_G, which is a different set. Global majority weight does not imply that every locally healthy region has more than two-thirds of its weight in H_G. For example, a region can have weight 0.2W and local responsive weight 0.18W, while its intersection with H_G has weight only 0.1W; the remaining 0.8W outside that region can still make H_G exceed two-thirds globally. Local health alone therefore does not establish the quoted READY-report quorum or timely activation knowledge for that local cohort. The necessary connection is not defined in “healthy eligible” or elsewhere in the supplied assumptions.

Conditional on the missing READY and global/regional connectivity premises, the remaining progression is coherent: a due rotation with a ready successor is selected deterministically; CONFIRM finalizes it; a fixed later boundary activates it; the minimum-tenure condition leaves a conservative startup and synchronization interval before the geographic guards can force another replacement.

> Lines 988–1005 and 956–959: bounded report backlog, sufficient offered-load budget, and a minimum-tenure inequality.

These are explicit conditional assumptions. The proof does not establish an unconditional throughput guarantee. Given a finite retrievable backlog and the stipulated Q bound, earliest-contiguous reporting prevents skipping an intervening outcome. If a fresh certified batch is reported and adoption is not preempted by a rotation, adjacent global links finalize it and the retrieval lemma supplies delivery. This downstream conditional reasoning checks out; the full liveness theorem remains unverified because of F1–F3.

## 10. Remaining timing, evidence, and cost claims

> Lines 1007–1078: two-delay internal confirmation; separate export, backlog, finality, tail-draining, readiness, and recovery costs.

The bounds are explicitly conditional on finite retrieval/backlog quantities, an eligible proposal at the relevant start, and healthy successors. They budget exported confirmation separately from the regional two-delay event. The geographic accounting correctly includes a late old-region tail that may reset the failure counter; a new finalized Deliver is expressly charged its additional finalization interval. I found no independent algebraic contradiction in these conservative bounds. They inherit the unresolved liveness/eligibility premises and do not certify progress when the relevant quantities are infinite.

> Lines 738–759 and 963–974: typed conflict evidence; AB relaying is not a double vote; silence is not equivocation.

The listed comparisons bind their respective configuration and instance domains and distinguish necessary signed-chain forwarding from origin or consensus double-signing. Quorum sizes are not changed mid-instance. The result proves cryptographic attribution of inconsistent signed statements, not culpability for outages. The explicit regional broadcaster produces one proposal per instance; F1 concerns receiver authentication, not whether an honest broadcaster itself creates conflicting signatures.

> Lines 1084–1094: cap-two forwarding gives O(n^2) logical multicasts, O(n^3) direct transmissions, and O(En^2) backbone arc transmissions.

These order bounds follow for the prescribed forwarding traffic: n origins times n members times at most two forwarded values; a direct multicast has n recipients and an overlay multicast uses at most E arcs. Each chain has at most n signer/signature pairs. These are bounds on prescribed/protocol-admitted traffic under the stated quotas, not on arbitrary Byzantine wire traffic. The manuscript explicitly separates that distinction and excludes unbounded flooding.

> Lines 1117–1128: body-byte traffic is at most (a+1)M with object caching and capped failed responses.

Correct with the declared meaning of M as the sum of missing bytes over receivers and a as a bound on failed supplier attempts per object. Physical route length, announcements, and certificates are correctly additional costs. This accounting does not eliminate the full replicas required by global report verification.

## Verification boundary

No manuscript text, algorithm, proof, or parameter was changed during this review. No absent READY rule, proposal-authentication guard, or cohort-connectivity assumption was inserted into the verified artifact. A later revision must be checked on its own frozen text; this report's positive checks do not automatically certify changes to those dependencies.

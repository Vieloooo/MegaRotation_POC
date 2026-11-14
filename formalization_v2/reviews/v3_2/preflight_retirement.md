# v3.2 preflight: graceful retirement under the frozen v3.1 model

Date: 2026-10-18. Scope: independent analysis of `RequestClose` / `FinishClose`, not an edit of the protocol sources. Source line references below are to the current `sync_hotstuff_rr_part2_rotation_v3_1.tex` (II) and `sync_hotstuff_rr_part1_core.tex` (I).

## Verdict

A **restricted retirement rule is viable** without a new consensus or certificate: authenticate an already effective Force, relay its complete closure evidence regionally, close each old instance's admission/signing gate, preserve genuine Quit processing and existing commit timers, and retire the production instance after a grace period and a durable archival handoff. `Terminated` must mean that the production instance no longer executes; it cannot mean that its commitments, data, evidence, or outstanding publication responsibilities cease to exist.

With the precise rules below, **4Δ after each replica's own close event is a conservative bound for draining all honest pre-close timer events and their regional FINAL delivery**. **3Δ drains the timer events but is insufficient to flush the last regional FINAL messages.** Neither bound guarantees that each local commitment forms a complete receipt, that a terminal receipt exists, or that publication / Espresso acceptance has finished.

The change requires a **new retirement safety lemma**. Immediate vote cessation invalidates the universal protecting-QC conclusion in I, Lemma `lem:timer`; saying that Part I's proof is literally unchanged would be incorrect. The cessation does not, with the additional entry gate and evidence handling below, itself give a counterexample to regional commitment safety. It intentionally abandons old-session progress and complete-receipt guarantees for an unconfirmed suffix.

## Source facts that must remain distinct

| Obligation | Existing source |
|---|---|
| Every regional certificate has a correct signer; correct members can form one if they actually supply matching messages | II:330–337 |
| Correct replicas retain protocol state and committed blocks | II:345–346 |
| Correct-to-correct regional delivery **and processing of complete dependencies** takes at most Δ | II:360–372; I:159–171 |
| Publication begins only with a **complete** regional result, and forming FINAL receipts remains a regional task | II:374–390 |
| Globally accepted DATA remains recoverable without its producing region online; unbounded historical catch-up has no constant bound | II:411–426 |
| Each locally committed DATA causes FINAL to be sent to the region; matching weight greater than F forms a receipt | II:791–800 |
| Existing inconsistency: the Commit box instead names the Publisher Layer as FINAL destination | II:1067–1073 |
| A valid terminal receipt, rather than a local terminal commit, authorizes Completion; Completion cancels timers and retains data service | II:929–935, 1092–1102, 1116–1118 |
| Genuine Quit cancels timers, broadcasts its proof, retains commitments, and continues evidence collection | II:1148–1158 |
| Recovery entry waits at least Δ after the replica's genuine Quit | II:1161–1176 |
| An effective Force preserves G and closes the entire old branch, including speculative successors | II:1552–1564, 1739–1743 |
| Existing v3.1 Force deliberately does not stop or mutate regional engines | II:938–939, 1560–1564, 1633–1637 |
| Part I's protecting-QC proof uses every correct replica's vote, not merely the continued passage of time | I:749–776 |
| v3.1's external prefix argument is conditional on regional validity and explicitly does not re-prove regional DATA/DRAIN correctness or receipt formation | II:2065–2101 |

The FINAL destination inconsistency must be resolved explicitly in v3.2. For the bound below, retain the already stated send-to-region rule, and clarify the Commit box accordingly; a partial FINAL is not a complete result whose delivery is guaranteed by `ass:publish`. Do not silently treat the Publish Layer as a regional FINAL aggregator or as an additional bounded multicast service.

## Proposed concrete rule

Use `closing`, a close-proof reference, and a local close time as **fields of the existing instance**, not a seventh program phase. A closing Active instance remains Active until a genuine Quit, genuine Completion, or FinishClose; `closing` independently prohibits fresh production. A closing Frozen instance remains Frozen. Completion changes the phase to Completed only upon a valid terminal receipt, exactly as before.

### RequestClose

For every instantiated or potential session pair `(β,r)` of a closed branch:

1. **Verify the effective boundary.** Require the finalized triggering Espresso block and complete replay/control evidence establishing that this particular β was closed by the canonical interpreter. A pending L1 MSIG, an unfinalized header, a bare timestamp, a cooldown, or an ordinary OFF does not suffice. Do not wait for successor selection or successor Ready: branch closure can occur while the successor remains Waiting.
2. **Latch the close event once.** At local time `c_v`, record the verified boundary and set `closing=true`. Relay the complete closure bundle to the region's fixed membership. The bundle contains existing source evidence; the relay signature authenticates transport and is not a vote or new certificate. Duplicate or later deliveries never reset `c_v`. Record a persistent closed-β tombstone so a delayed witness cannot create or admit another old-β instance.
3. **Stop new activity in that instance.** Do not propose DATA/DRAIN or NEWVIEW; do not cast DATA/DRAIN/ANCHOR votes; do not start a new commit timer; do not run progress checks that create fresh BLAME; do not enter a new term or generation. Apply this gate to bootstrap entry and to queued/reentrant recovery callbacks as well as normal handlers. A previously received quit proof does not bypass the gate when a delayed authority finally arrives.
4. **Drain, without fabricating a protocol event.** Retain all commitments, locks, existing votes, proposals, and validation dependencies. Keep receiving and checking signed material against retained material. Process and broadcast every genuine current-generation quit proof under the existing Quit rule; only that genuine Quit or genuine Completion can cancel an existing commit timer. Let every otherwise-surviving timer commit normally, and send its prescribed regional FINAL. Continue collecting votes, QCs, FINAL messages, and valid receipts; offer every complete result to the Publish Layer. A late valid terminal receipt may still cause genuine Completion, but cannot reopen the old global branch or its normal successor.
5. **Keep ownership until transfer.** The instance or a persistent local owner continues data service, source-evidence retention, and pending publication/retry work. The successor may warm up and run concurrently in its own session context; it does not inherit the old signing state.

No rule changes the voting snapshot, threshold, quota, authenticated cutoff, retained global prefix, Force priority, or any service assumption. The regional state machine is nevertheless being extended: the new signing/entry gate must be visible in the text and in the safety argument.

### FinishClose

For an Active or Frozen instance, execute FinishClose only when all of these hold:

- `closing=true` and the local clock is at least `c_v+4Δ`, with received messages handled before a same-time finish event;
- all existing commit-timer callbacks have either executed, or been canceled by genuine Quit or Completion; no fresh callback can pass the close gate;
- the persistent archive has taken ownership of the closed-version tombstone, immutable committed log, all retained signed material / validation dependencies required to serve it, complete receipts, and pending offers / retries; old-session messages continue to route to that archive rather than disappear;
- every locally complete result has been offered under the existing Publish Layer interface. Offering does not mean Espresso inclusion or global acceptance, and the handoff retains the obligation until discharged.

Then set `phase=Terminated` and release the production/recovery worker and its live timer machinery. Keep the archived commitments, evidence, data service, and publication owner. A late archive message cannot restart the instance or sign in its session. It may complete an old receipt or satisfy a data request. Processing such archival evidence is not old-session production.

Waiting/Ready instances that have never admitted an engine have no regional timers to drain: a verified closure plus persistent tombstone and archival transfer can terminate them. A Completed instance already has the genuine terminal receipt and has canceled timers under Completion; it can likewise release the worker after archival transfer. Applying the uniform grace to these cases is conservative but unnecessary for timer safety.

The archive can be an ownership transfer inside the same correct replica; no new external archival service, agreement, or acknowledgement certificate is needed. The specified storage obligations already persist. There is **no bounded-memory claim**, and no wall-clock bound on a physical copy/checkpoint operation. If implementation handoff has not completed, FinishClose waits; the protocol cannot invent a storage-latency bound.

## Grace-period derivation

Fix a region and Δ from the old session's immutable snapshot. Let

\[
c=\min_{v\text{ correct}} c_v.
\]

The first correct RequestClose relay carries **all required closure validation evidence**. Under the existing regional complete-message assumption, every correct member processes that evidence by `c+Δ`, so

\[
c\le c_v\le c+\Delta.
\]

This is a bound from the first complete local verification and actual relay. It is not a bound from an unseen Espresso finalization, an unpublished MSIG signature, the Force timestamp, or the start of arbitrarily long history catch-up.

Every honest timer is started with its DATA/DRAIN vote, and no correct vote occurs after local close. Thus the last possible honest timer starts no later than `c+Δ` and expires no later than

\[
c+3\Delta.
\]

Genuine Quit / Completion can remove timer events but cannot postpone them. The FINAL messages generated by surviving honest timer events reach every correct regional recipient by

\[
c+4\Delta.
\]

Since `c_v≥c`, `c_v+4Δ` is a locally checkable conservative deadline. `c_v+3Δ` is enough to know that there can be no remaining honest pre-close timer expiry, but not enough to know that the latest honest FINAL delivery has occurred.

This calculation proves delivery of **the honest messages that are actually generated**. It proves neither a matching threshold nor the delivery of Byzantine messages. A complete receipt can also arrive arbitrarily late from a Byzantine holder, which is why an archive route remains necessary. Any complete result newly formed near the grace deadline is still subject to `T_pub` and then `T_E`; those quantities are not bounded by 4Δ and need not elapse before the production worker retires.

## Regional commitment safety: what must be re-proved

### The original lemma cannot be copied

I:758–766 derives a protecting QC by requiring every correct replica that receives B by `s+Δ` to cast its matching vote. A locally closing member now retains B but does not vote. Therefore the claim “every correct replica knows B's QC by timer expiry” can be false, even when B's timer correctly survives. The same omission invalidates a direct reuse of the all-replicas commit-window lemma at I:835–849.

### An adapted argument is available

The following addresses the additional close gate. It retains the source protocol's validation, evidence-first processing, Quit broadcasts, recovery wait, and existing sound treatment of Completion. It does not provide a new independent certification of the entire v3.1 DATA/DRAIN extension, which v3.1 itself assumes at II:479–484 and 2065–2101.

Take a correct **surviving direct timer** for B in generation g, started at s and expiring at `s+2Δ`. If the first correct genuine Quit of g occurred at `a≤s+Δ`, its broadcast would reach this committer by expiry and cancel the timer. Close does not disable this processing. Therefore, if there is any genuine Quit of g,

\[
a>s+\Delta.
\]

Every correct entry into g+1 waits at least Δ after its own Quit, so

\[
e_v(g+1,0)\ge a_v+\Delta\ge a+\Delta>s+2\Delta.
\]

Now split:

1. **Early vote window: `s+Δ<c`.** No member's new close gate can suppress a vote during B's complete-proposal dissemination window. The original protecting-QC and recovery-lock reasoning applies to this timer as it did before the retirement extension, subject to v3.1's unchanged Completion treatment.
2. **Late vote window: `s+Δ≥c`.** B expires at or after `c+Δ`. Every correct member has closed by `c+Δ`; every later-generation entry would have to occur strictly after B's expiry and is therefore prohibited. No correct member can enter any generation above g, and every such regional QC would require a correct signer. In this case the missing protecting QC cannot be exploited by a later recovery: the recovery cannot occur.

Within one generation, conflicting direct commitments are ruled out by complete-proposal forwarding and evidence-first processing even without universal voting. The first correct voter forwards its complete signed history. A correct voter on an incompatible history either receives the first history before voting and detects the fork, or has already forwarded the incompatible history early enough that the first timer receives the proof by expiry. The source `lem:fork` (I:735–747), the Δ transport bound, genuine Quit, and message-before-timer ordering apply. Merely retaining without comparing would not suffice.

For different generations, the lower-generation direct commitment cannot be of the late type if a correct higher-generation vote exists; hence the original protecting-QC/lock extension argument protects that commitment. Ancestor commits inherit comparability. Receipt-based commitment remains grounded in at least one correct FINAL signer who already committed the named DATA; Completion does not create a new independent commitment justification. No commitment is deleted or renamed at either RequestClose or FinishClose.

This establishes why **gate plus drain** preserves the regional safety property while discarding old-session liveness. The later-generation gate and continued evidence processing are substantive premises of this argument, not implementation conveniences.

## Concrete failure traces and limits

### A. Arbitrarily long grace cannot force a complete receipt

Use unit weights, `n=3`, `f=F=1`, correct H0 and H1, Byzantine F, and an already admitted unfrozen generation with a valid opening QC. H0 verifies closure at time 0 and relays it. H1 processes closure at Δ. At `s=ε`, `0<ε<Δ`, F supplies a valid next proposal B to H1; H1 votes and forwards it. H0 receives and retains B after its own close but starts no vote or timer. F supplies no matching vote or FINAL. No quit proof or terminal receipt appears. H1's timer commits B at `ε+2Δ`, after local close, and H1 sends FINAL. The only matching FINAL weight is 1, which is not greater than F.

B is a real local commitment with no protecting QC and no complete regional receipt, forever. No existing Publish Layer premise applies to that incomplete result. A 4Δ grace preserves H1's state and FINAL but cannot produce another signature. If B is terminal, H1 still has not legitimately Completed; FinishClose can only terminate it with its residual archive and explicit abandonment of old-session progress.

### B. 3Δ is a timer bound, not a regional FINAL flush

Let H0 close at 0 and H1 close at Δ. H1 casts its last vote just before learning closure, at `Δ−ε`; its timer expires at `3Δ−ε`, and the FINAL reaches H0 at `4Δ−ε`. If H0 destroys the only receiving route at its local `3Δ` deadline, that honest FINAL is lost. An archive receiver can make earlier worker release safe, but then the claimed flush is performed by the archive, not by 3Δ elapsed time.

### C. Suppressing evidence on close can break regional safety

Again use two correct members H0/H1 and one Byzantine leader F. H0 votes a valid proposal B at `s=0`. H1 locally closes at `ε<Δ`. F supplies a different admissible proposal B' for the same position to H1 just before H1 closes, so H1 votes and starts its own timer. Deliver H0's B to H1 and H1's B' to H0 after H1's close but within Δ. If closing means neither replica processes incoming equivocation evidence (H0 can close slightly later, still before those deliveries), the two timers can commit conflicting blocks. The full source rule would expose the signed fork and run genuine Quit before either expiry. The close gate must preserve that rule through the timer drain.

### D. Dropping the recovery-entry gate removes the proposed proof

Trace A shows a surviving timer can lack a protecting QC. If an implementation then allows a queued recovery callback to enter g+1 after closing, the original proof has no QC/lock guarantee protecting that timer's block. This is a **justification gap** unless a separate safety argument is supplied; it is not enough to cite Part I unchanged. The proposed rule blocks that callback. No claim of a concrete conflicting-commit trace is made here without also specifying how later votes are re-enabled.

### E. “Termination deletes the old state” violates existing retention obligations

Suppose B from trace A exists only as an honest local commitment and partial FINAL at H1. It never became a complete regional result and was never globally accepted. II's Espresso retrieval assumption covers global DATA and replay evidence; it does not magically retain this unpublished B. Deleting H1's committed log at retirement loses that commitment and its only complete data copy. Persistent local archival ownership avoids the problem without strengthening Espresso or the Publish Layer.

## Consequences for the v3.2 text

- Keep the external proof of global G preservation and old-β rejection separate from regional retirement. It already closes the whole old branch independently of when old instances stop.
- State exactly six program phases, with an independent monotone `closing` flag during Active/Frozen draining. Never label a Force-induced retirement as Quit or Completion.
- Change “regional rules remain unchanged” to the narrower correct claim: normal running preserves the DATA/DRAIN, vote, commit, and genuine recovery rules; authenticated retirement adds a signing/entry gate and a bounded drain.
- Describe 4Δ only as the bound derived above. Do not promise a terminal receipt, all-replica local commitment, final publication, bounded archive size, or bounded storage-transfer latency.
- Keep a stable owner and dispatcher for old archives after Terminated. New-session warmup and bootstrap have separate `(β,r)` contexts and need not await old worker retirement.
- Add explicit `not closing` checks to every fresh-signing or entry site, including bootstrap, NEWVIEW, recovery Status/entry, ANCHOR vote, progress checks, term advance, and buffered-message callbacks.
- Resolve FINAL's inconsistent destination explicitly; the 4Δ regional delivery argument uses only the already specified regional broadcast, not an imagined publication guarantee for partial signatures.

Validation performed: manual source-level dependency and timeline analysis against the two files above. No executable model checking, simulation, or independent audit of the entire regional protocol was performed.

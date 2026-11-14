# v3.8 network-domain audit

Date: 2026-10-19. Read-only source audit; no protocol or manuscript changes. Scope: the frozen v3.1 model and request interface, the current v3.7 manager/request lifecycle, and the v3.4 retirement fragment imported by v3.7. No other current audit outputs were read. The proof-verifier discipline was used to separate an existing guarantee from an additional premise.

## Decision

**The frozen model supports node-level service after worker termination within a historical fixed regional snapshot. It does not entail a bounded direct exchange between disjoint old and new voting rosters merely because both have the same physical region ID. Taking the maximum of their two advertised delay bounds does not repair that missing endpoint domain.**

A protocol may require a correct owner to retain a pending request and withhold resource release until an actual durable transfer has occurred. That is a permissible correctness/release rule. It is not a proof that a remote transfer or physical withdrawal finishes in finite time. Existing retirement makes precisely this distinction: it permits local ownership transfer, imposes no transfer-latency bound, and does not authorize stake withdrawal.

Authentic AUTH records already provide an unambiguous, publicly authenticated **membership lineage indexed by slot**. A new trusted membership directory is unnecessary. They do **not** provide an ownership lineage for individual pending requests, a completed-transfer fact, a bound on future AUTH acquisition, or the missing old-to-new transport bound.

The concrete boundary is therefore: a bounded request exchange may charge an original regional delta only after identifying a fixed authenticated snapshot containing its correct sender and receiver, or a sequence of such covered hops. A same-region label by itself is insufficient.

## 1. What “correct replica” ranges over

The frozen model declares regional voting members, rather than a second global set of all machines ever physically associated with a region:

- `revisions/v3_1_frozen/model_and_assumptions.tex:31–39`: AUTH[r,0] fixes the members, weights, F_i, Delta_i and K of each region; g > 0 changes only the leader schedule.
- The same file, `137–150`: V_i is the weighted membership; Byzantine weight is bounded within that membership; membership and weights remain fixed through a session and its recoveries; later sessions validate historical certificates under the historical snapshot. The fault bound tracks every compromised key of that snapshot while its certificates remain accepted.
- `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex:523–528`: V_i is “its regional members,” Delta_i is the regional synchronous bound, and cfg_r is the fixed session snapshot.
- The inherited Part I domain is explicitly a fixed voting membership, with a fixed Byzantine set (`revisions/v3_1_frozen/sync_hotstuff_rr_part1_core.tex:151–171`). Part II replaces that one fixed committee with authority-indexed regional snapshots; it does not introduce a network-membership union over future configurations.

Thus, making the suppressed configuration argument explicit, the directly justified regional premise is:

\[
u,v\in C_{i,r}\quad\Longrightarrow\quad
\operatorname{delay}(u\to v)\le\Delta_{i,r},
\]

where C_{i,r} denotes the correct members of region i in cfg_r. This is explanatory notation, not a proposed manuscript change. The quoted network statement is universal over correct-to-correct messages **in its declared regional domain**; it is not restricted to votes or Active workers. Its processing scope includes complete validation dependencies (`model_and_assumptions.tex:167–179`).

The source does not define a separate set P_i = union_r V_{i,r}, say that every pair in P_i remains mutually reachable while duties persist, or assign a delta to a pair belonging to no common snapshot. “Correct participant” in the L1/Espresso service assumptions is broader, but those services have their own payload and online-reader premises (`208–215`, `218–250`); they do not turn every participant-to-participant message into a regional message.

The current liveness argument confirms this scope rather than widening it: it fixes one installed slot, explicitly disclaims a union over all future memberships or a uniform future-configuration bound, and proves progress for installed regions meeting their communication premise (`sections/rotation_v3_7_liveness.tex:44–59`, `65–75`, `354–360`).

**Assessment:** reading “within region i” as an independent all-ever-members physical network would be a possible additional model convention, but it is not a consequence of the definitions above. Under that convention, an old/new direct edge would indeed be covered if both live correct owners belonged to that physical-network domain. One must then state which bound governs the overlap interval. Using that convention to prove the requested extension silently adds the precise cross-configuration service that is at issue.

## 2. Why the existing closure relay is covered

The retirement fragment begins by fixing a session and taking Delta from **its immutable voting snapshot** (`sections/rotation_v3_4_retirement.tex:3–4`). It requires each historical member to keep a manager/control record even in Waiting, Completed, Terminated, or without a worker (`15–20`). Complete receipts and close bundles go to that same fixed membership (`24–41`, `57–83`).

Termination releases the production/recovery worker; it does not remove the correct historical member or its receive route from the modeled region. A persistent owner must possess the evidence and route before the old owner releases them, and the transfer may remain local to the same correct replica (`135–156`). The subsequent closure theorem expressly ranges over **this snapshot's correct members**, including manager/archive owners (`196–220`).

This use of Delta does not require a new network endpoint or a new roster. It changes which local component handles the message. Accordingly, “worker exit invalidates Delta” is an incorrect objection to the existing relay. Conversely, the fact that this relay is valid does not establish an old-only member to new-only member channel.

The current node-level source handler follows the same pattern: SOURCESTEP is addressed to an **installed fixed regional snapshot** (`sections/rotation_v3_7_source_step.tex:133–142`); its prepared timing claim concerns hydrated members under a fixed installed slot (`213–250`). Node-level persistence is a storage/lifecycle choice, not by itself a change in the communication domain.

## 3. Old/new exchanges and the role of max(Delta_old, Delta_new)

Let O and N be consecutive authorized rosters for the same physical region, with bounds Delta_O and Delta_N. The following cases are distinct.

| Exchange | Existing justification | Valid timing conclusion |
| --- | --- | --- |
| Correct old member to correct old member, including after their workers terminate | Historical snapshot O and its retained receive owners | Delta_O |
| Correct sender and receiver both belong to N | Snapshot N | Delta_N |
| Sender and receiver share some other still-applicable fixed snapshot S | That explicitly identified snapshot S | Delta_S |
| Old-only sender to new-only receiver; a correct member belongs to both O and N and actually relays | Two separately covered regional hops | Delta_O + Delta_N, not generally their maximum |
| Old-only sender to new-only receiver with no guaranteed correct bridge | No declared common snapshot or covered path | Neither direct bounded delivery nor a bound given by max(Delta_O, Delta_N) follows |

The table concerns delivery of a complete admitted message. An unbounded backlog, arbitrary CPU job, unresolved fetch, or unknown AUTH cannot be hidden inside a delta charge (`sections/rotation_v3_7_source_step.tex:238–250`).

Each configuration having correct weight greater than F does not imply that O and N overlap, much less that their overlap contains a correct identity. They may be completely disjoint. A maximum bounds numbers already known to bound the relevant action; it cannot supply a missing action-to-bound premise.

If membership is unchanged and only Delta changes, a bounded action may use the bound of an identified applicable snapshot. A finite maximum can conservatively dominate such already-valid charges. This does not permit the old regional timer proofs to abandon their own immutable Delta_O while their duties remain active. Nor does a finite maximum for two known configurations imply a finite uniform bound over every future configuration.

### Distinguishing execution

Take one physical region i. At slot r its roster is O = {a,b,c}, with unit weights, F_O = 1 and Delta_O = 1. At slot r+1 its roster is N = {d,e,f}, with unit weights, F_N = 1 and Delta_N = 2. Let all six identities be correct; the fault bounds are then satisfied. All members stay online. The old program may have terminated, but its node-level owners retain all required data and routes.

Deliver every O-to-O complete message within 1 and every N-to-N complete message within 2. Supply the required AUTH and public source evidence; all new-session regional execution and publication can proceed normally. Let a retain admissible pending request x, and at time s let it send x and a putative duty-transfer message to every member of N.

Choose any finite proposed cross-roster bound B, including B = max(1,2). Delay these cross-roster messages until s+M, where M > B. There is no common roster containing a and d/e/f, so no declared snapshot-domain edge violates its regional bound. This execution does not rely on an old correct replica disappearing, a crash, a Byzantine receiver, or an incomplete packet.

For compatibility even with the eventual input condition, arrange that the correct client separately delivers x to a serving new member at s+M, or let the delayed original message arrive then. The request has finitely many local predecessors and eventually reaches the serving region (`model_and_assumptions.tex:310–312`). Normal DATA growth can occur meanwhile. No request-only publication service is invoked: the bounded Publish Layer starts with a complete regional result, not a queued request (`181–196`).

Therefore every proposed fixed cross-roster deadline can fail while all declared snapshot-local services and eventual request delivery hold. This example proves **non-entailment of a bound**, not permanent loss of x. Under an added physical-region-union network premise the example is excluded, precisely because that extra premise covers the cross edge.

## 4. Retention and transfer before physical withdrawal

The pending queue is already node-based: first receipt fixes local position, duplication does not reset it, and only global inclusion removes a request (`revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex:1839–1858`; unchanged current interface at `sync_hotstuff_rr_part2_rotation_v3_7.tex:2221–2240`). The current manager preserves pending requests, and discarded-suffix requests remain available for reproposal (`1623–1629`, `2141–2151`). These rules can legitimately be implemented by a persistent node-level owner independent of BFT worker phases.

It is also legitimate to enforce “do not release the last required copy or receiving responsibility until a successor owner has actually accepted it.” This is a conditional program guard, not a promise that the guard will become true. Retirement already uses such guards, expressly allows same-replica transfer, and says that four deltas bound neither later storage copies nor transfer latency (`rotation_v3_4_retirement.tex:135–169`, `223–236`). The main lifecycle states that resource transfer has no new time bound and that Terminated does not authorize stake withdrawal (`sync_hotstuff_rr_part2_rotation_v3_7.tex:2117–2151`).

Consequently:

1. Requiring persistence across session/worker changes does not strengthen the network model.
2. Requiring an actual acknowledged durable transfer before release is also a permissible protocol rule. A local transfer can satisfy it without a network premise.
3. Claiming that every owner can finish a remote cross-roster transfer, or physically withdraw after finite time, requires an independently justified path, recipient, and eventual/bounded response. Such a conclusion does not follow merely from the guard.
4. Defining a correct node to remain until its outstanding duties are transferred must not be restated as a derived finite availability guarantee. The node may have to remain indefinitely if the transfer premise never materializes. The frozen model supplies no timed withdrawal theorem.

Thus “finite duty before withdrawal” is acceptable if it means a finite completed transfer transcript is a release prerequisite. It is unsupported if it means the protocol has proved that every pending owner completes transfer within finite time under arbitrary changing rosters.

## 5. What AUTH already makes discoverable

AUTH[r,0] fixes every region's roster and parameters; it is unique and immutable at that index. Recovery AUTH[r,g] only changes the leader schedule. Reusing r under another beta reuses the same authority, so it cannot secretly replace the roster (`model_and_assumptions.tex:13–16`, `30–44`). Finalized L1 authority records are common and authenticated, with complete control history (`159–165`).

This already suffices to authenticate “these are region i's members under slot r'” and to enumerate the intended voting recipients once AUTH[r',0] is available. There is no need for a new trusted directory to certify that membership. Current public replay and handoff evidence determine which slot/region actually becomes the successor; a future AUTH by itself does not activate it (`sync_hotstuff_rr_part2_rotation_v3_7.tex:1791–1839`). Zero-output retries reuse the slot, so a beta increase is not a membership successor event.

However, no AUTH field names the current owner of request x, records which queued requests that owner durably accepted, acknowledges an ownership handoff, or retains a discoverable chain of arbitrary storage-owner substitutions. An algorithm could define deterministic destinations using the existing authenticated roster, but the correctness and transport obligations of that new rule must still be checked. This observation is not a proposed repair.

Nor does authenticity give a constant acquisition time. The frozen model requires needed authorities to become available eventually (`307–309`); the current cache's sequential task concerns recovery authorities within an installed slot (`sync_hotstuff_rr_part2_rotation_v3_7.tex:1757–1766`). The source explicitly leaves initial authority acquisition unbounded (`sections/rotation_v3_7_liveness.tex:5–15`). The bounded L1 control-operation service must not be silently converted into a deadline for every future AUTH or every future membership handoff.

## Verification verdict

No defect was found in the existing claim that historical-snapshot managers continue complete closure/receipt relays under their original Delta after worker termination. The claimed extension to arbitrary same-physical-region old/new rosters is a **Justification Gap** unless a common covered snapshot/path is identified; an unconditional max-of-two-deltas bound is false under the declared snapshot-domain model, as witnessed above. AUTH resolves membership authenticity and slot identity, but not request ownership or the missing delivery edge. These conclusions were obtained by source inspection and a hand-constructed model execution; no executable/model-checking validation was run.

# v9 v0.1 wording review

Date: 2026-11-02.

After the first draft of v9 was completed, three independent review agents each read all files, focusing on one aspect:

- **A, notation:** whether notation is clear, consistent, and self-contained; each record is defined only once; and cross-references are correct.
- **B, readability:** whether both intuition and formal definitions are provided, the reading order is smooth, and examples match the rules.
- **C, composability:** whether each interface input has a producer and each output a consumer, types match on both sides, and end-to-end flows can be followed step by step under the specification.

The findings and changes are summarized below by topic. File numbers refer to v9 files.

## 1. Composability (primarily C)

| Issue | Change |
| --- | --- |
| Force and RESUME Plan inputs (Ref, source region, session number, window) were undefined, so parties could not compute the same $\Pi_s$ | 09 §4.6 gives complete Boundaries for Force and RESUME: the new RESUME session is $(\beta+1,0)$, source region $\bot$, window $[h_{\rm res}+1,h_{\rm res}+\mathrm{TB}]$; Force adds step 6 (Plan, SessionAssigned or PENDING, acceptance from $J_{q+1}$). 09 §4.5 states the regular-handoff Boundary. 01 §3.2 explains Ref in all three cases. |
| The Manager had no interface to compute the successor Plan; W14 had no call site | SessionAssigned processing in 03 §3.10 computes the successor Plan; ObservePublic in 03 §3.4 adds calls to `Responsibility` and `Chain`. |
| Cross-region relay of regional results was not connected | Added W26; HandleProgramEvent in 03 §3.6 relays to the successor region and notes that the same result can be assembled from the GFR publication index as a fallback. |
| ProgramEntry did not store $\pi_A$, so Enter and Res assembly had nowhere to read it | Added anchor_evidence to ProgramEntry. |
| The RESUME anchor state had no path to the Manager | Resumed processing in 03 §3.10 calls `DeriveAtResume`, stores anchor_state as StateMaterial, and waits for SessionAssigned before PrepareCandidate; step 14 of flow 3 in 10 was aligned. |
| State material for REGIONAL / FORCE anchors had no source | 03 §3.7 states the source of state material ($D$ checkpoint + $G$ + candidate chain); added the Manager as a W15 consumer. |
| Evidence-path types mismatched: SubmitOffense needs pid, but EvidenceTask lacked it; nothing triggered submission | Added accused_pid to EvidenceTask; 04 adds `QueueEvidence` (§3.6), moves SubmitEvidence to §3.7 and specifies the call `SubmitOffense(ν, accused_pid, ev)`; added W28. |
| The Batcher's and Certifier's local GFRs need resumeReader to process RESUME | Expanded W23 consumers to local GFRs throughout; added W23 and W19 to the upstream/downstream tables in 05 and 06. |
| The G-based kill condition “a higher predecessor block has entered $G$” had no event carrying it | 03 §4.4 states that it is read from GFR.`Chain`. |
| The interface connection table disagreed with module tables | Changed W16 to Batcher→DA; added W27 (CertificationRequest); removed Authority from W20 (the contract reads InboxState.cursor directly); added Certifier to W21 and Publisher to W13; changed W25's producer to the new `InputRef`; aligned module upstream/downstream tables with the new numbering. |
| CertificationRequest included prefix_frames, but CertifyBatch had only four parameters | Removed the field and stated that the Certifier obtains the frame prefix independently from L1 acceptance events and DA. |
| W25 had no producing interface | Added §5.6 `InputRef` to 09. |
| “Assignment cannot start” produced no event | The unified rule in 09 outputs SessionClosed (EXPIRED). |
| Materializing $G'$ requires historical $G_{\mathit{gpos}}$, but Chain returns only the current $G$ | 07 §3.7 explains that current $G$ may be used between two RESUMEs. |
| Flow 3 omitted SessionEnd before Expire in replay order | Added SessionEnd, regular anchor 108, and subsequent session handling to flow 3 step 8 in 10. |

## 2. Notation (primarily A)

| Issue | Change |
| --- | --- |
| References “§4.4 / §4.5” without file prefixes in 10 pointed to other sections in that file | Changed all to `[09 §4.4]`, `[09 §4.5]`. |
| SessionEnd close reasons were inconsistent (TIME / COMPLETE) | Standardized on COMPLETE; the CloseReason description in 03 distinguishes TIME (local clock reaches $\tau_s^{\rm stop}$) from COMPLETE (GFR public completion). |
| Region names A / B / C conflicted with anchor notation $A$; `cid(A)` violated the definition of cid(·) | Changed region names in 10 to a / b / c and rewrote `cid(A)`. |
| 00 §4.6 omitted CloseReason | Added it. |
| `Deliver(y)` was inconsistent with its input description | Changed to `Deliver(y, prev, channelInfo)`. |
| BuildPayload's range $I$ had the same name as the certificate's completed range $I$ | Changed BuildPayload's range to $J$ and return label `Payload` to `Built`, avoiding a name collision with Builder.Payload. |
| $\tau_{\rm del}$, $L$ in $D_L$, $\mathcal X$, and $\mathcal D$ were undefined or not linked | 00 §2.4 defines $\tau_{\rm del}$; 09 §4.6 defines $L$ and explains that the cut is bound to $p_{\rm res}$; 03 and 05 link to 09. |
| The anchor was assigned directly to a virtual block | 09 §4.6 specifies taking the corresponding Anchor, with its linking identifier computed under 00 §3.2. |
| WAIT had three forms | Standardized streaming interfaces on `WAIT{next_position, missing}`, explained in 00 §4.2. |
| “position” meant both a stake registration and an L1 position | Renamed the L1 position field L1_position in Authority outputs, ControlEntry, Inbox events, and Batcher delivery results. |
| Boundary.cause REGULAR could be confused with AnchorEvidence.kind REGIONAL | Changed cause to HANDOFF. |
| The position in `DeriveAtResume(position)` was ambiguous | Changed to `DeriveAtResume(p_res)`, with $p_{\rm res}$ the position of the RESUME control record. |
| Chain names were used as heights ($R=115$, etc.) | Changed to $h_R=115$, etc. |
| The rule-version notation $\mathit{ver}$ in 00 §4.2 differed from rule_version | Standardized on rule_version. |

## 3. Readability (primarily B)

| Issue | Change |
| --- | --- |
| The timing example in flow 3 contradicted the compressed height values | Explained that times serve the timing rules and are separate from height values; reworded $h_{\rm res}$ as “fixed timestamp 22:21.” |
| Flow 2's starting point $G=100$ did not connect with flow 1's ending state | Changed flow 2 to “suppose a has decided 100.” |
| Example A reused (7,20), (7,21), with windows conflicting with flow 1 | Changed to (7,30), (7,31) and supplied windows. |
| Sequencing window and “window fallback” lacked intuitive definitions | Rewrote the final paragraph of 00 §3.4, giving intuition before $\mathrm{ddl}$ and $W_{\rm seq}$. |
| 09 §3 and §5 lacked introductory motivation | Added an intuitive paragraph to each; added links from Tail values to the rules. |
| The terms in the budget inequality were unclear | 09 §4.3 explains each term. |
| The “unified rule” in 09 was difficult to parse | Rewrote it. |
| Conditions for the synchronization starting point in 02 were crowded into one sentence | Gave intuition first, then listed the four conditions. |
| DA-RET mixed a DA guarantee with a custody assumption | Added A-CUSTODY in 08 and changed DA-RET to a composed guarantee with two parts. |
| PUB-5 lacked a continuous-online premise | Added it. |
| $T_{\rm prep}$ in 10 had a different meaning from 01 | Changed it to $T_{\rm lead}$ and defined it locally. |
| 00 retained English and wording referring to the v8 source; DA intuition was duplicated | Changed to Chinese; removed “following regional BFT's …” and “following the order of the Regional BFT abstraction”; removed the intuitive paragraph in 00 §1.5 duplicated in 08. |
| An “or” clause in 03's Kill criteria was a special case of the first item | Rewrote as “different native hashes must be confirmed; a typical case is …”. |
| Recovery-alignment wording and CERT-4 in 06 were convoluted | Rewrote them. |
| TB was unexplained | Added “the number of heights available to each session” in 00 §2.2. |

## 4. Not adopted or deferred

- **Position field in derivation state.** DerivationState.position means the derivation processing position, defined in 09 §5.2; it was not renamed.
- **v0.1 source explanation.** The opening of 00 explains how this version was split from v8; it remains there for comparison.
- **G3.3 leader-sequence restriction and Builder fair selection.** These are known gaps in the properties themselves. This review checked wording only and added no new assumptions.
- **20-minute recovery budget and 10h / 12h parameters.** Both use the example values from 261027 and still need calibration against actual parameters.

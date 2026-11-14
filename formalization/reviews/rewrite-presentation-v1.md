# Rewrite presentation and scope audit

Reviewed the complete current `main.tex` and both native TikZ inputs. This is a presentation/interface audit, not the independent correctness or complexity review. Locations refer to the inspected 1,244-line rewrite and may shift during revisions. No manuscript or figure edits were made in this audit.

## Findings to resolve

1. **The pipeline must name the exported regional certificate.** `figures/stream-pipeline.tex:27–29,37–56` labels the peer events only “Validate / confirm,” followed by an unlabeled arrow to Preview. In `main.tex:436–450,480–483,525–529`, an internal decision or VOTE quorum is insufficient: Preview needs the DECIDED quorum `RC` and complete data. Rename the peer event or label its outgoing arrow as the exported `RC`. The figure does **not** literally preview missing data: the first/third data pushes arrive before Preview, and the middle repair arrives before its Preview. Its current ambiguity is which certificate the peer arrow represents.

2. **The pipeline needs the manuscript's checkpoint notation and its global convergence stage.** `figures/stream-pipeline.tex:67–72` uses checkpoint `c`, whereas `main.tex:119–129,354–359` uses `c` for configuration and `C_e` for checkpoint. Use `C_e`. The lower first box should also name report agreement/target derivation, or the caption should explicitly say that this mandatory phase precedes the shown target. `main.tex:610–617,631–635` makes this phase essential; an unlabeled jump from a regional prefix to FFG votes invites the interpretation that FFG itself selects the common target. “Later votes” should identify the adjacent link `C_e => C_{e+1}`.

3. **Explain the batching and serialization represented by the trace.** The top panel draws independent confirmation events for `M_h`, `M_{h+1}`, and `M_{h+2}`. The reference actually confirms bounded batches and serializes dependent instances through mandatory completion (`main.tex:484–500`). State in the figure caption that the three positions are schematic one-mini-block batches, with inter-instance completion intervals suppressed; alternatively depict one certified batch. Otherwise the trace can be read as an overlapping one-confirmation-per-mini-block pipeline. Its independent local/global time scales are already explicit and appropriate.

4. **Put the report selection rule into G1 and remove “newest.”** `main.tex:605–609` says each member snapshots its “newest valid bounded regional extension,” but `668–671` prescribes the earliest contiguous extension that fits, forbidding skipped outcomes or a truncated certified batch. The latter is the exact algorithmic rule and belongs in G1; “newest” can mean a suffix that omits a necessary intermediate outcome.

5. **Make READY generation an executable rule.** `main.tex:371–375` defines the meaning of a true READY, and `650–651` defines its quorum. G1 (`605–609`) never says when an honest node sets READY true rather than false. The liveness argument then relies on healthy standby members originating true reports (`946–949`). Add the exact local condition in G1: complete verified retained parent prefix/evidence, willingness and capacity to join its region at the prescribed activation; otherwise false. The parent/epoch/region binding is already described.

6. **Define the regional schedule once.** The model supplies synchronized boundaries and `tau_R`, but `main.tex:417–426,484–490,580–582` never explicitly maps instance `(c,a,u)` to its common start. Add the reference formula using the generation's activation boundary, e.g. `t(a,u)=t_act(a)+(u-1) tau_R(i)`, and identify the genesis start. This closes the meaning of “future scheduled instance” without adding a clock implementation.

7. **Do not describe signing history as peer-supplied recovery state.** `main.tex:586–589` says a recovering member “pulls the certified state and signing history.” The model instead requires an honest node's signing history to persist across restarts (`180–182`). Say that it restores its own persisted signing history and pulls missing certified public state/data. This makes the restart assumption and recovery interface consistent.

8. **Move mandatory evidence extraction into an algorithm entry rule.** `main.tex:754–755` mandates extracting evidence before ordinary conflicting-message discard, but Protocol E only accepts an already identified pair. Add a common signed-message receipt rule or a short preamble to the boxes that invokes evidence checking before a handler discards an authenticated conflict. This avoids a required processing step existing only in prose.

9. **The optional HEAD mechanism remains prose-only and has an undefined slot duration.** `main.tex:405–413` introduces signed HEAD slots, latest-message state, equivocation exclusion, and a recursive selection rule; no box specifies its invocation/receipt handling, and `s` has no slot length or schedule in `Theta`. Either remove this optional mechanism from the reference construction or give it a short self-contained auxiliary algorithm and a configured slot definition. Its current exclusion from confirmation guards is clear.

## Scope and figure checks that passed

- The service ends at proposer verification/receipt. Execution, EVM packaging, DA publication, and settlement are not performed by any protocol rule or figure. Mentions of downstream execution/publication explicitly exclude them.
- The architecture contains exactly three sequencer symbols and the caption explicitly disclaims an `n=3,f=1` interpretation. L1, users, and P are outside the service boundary.
- The lower pipeline places Deliver after finality and explicitly requires complete data and proof verification. Its global timeline carries no numerical latency claim.
- The top pipeline's missing middle object is repaired before Preview; it is not silently skipped.
- The architecture omits a separate global-proof-to-P route even though G5 permits global members to push/announce finality evidence. This is acceptable as an overview if the caption states that proof delivery and global-holder repair use the same external consumer interface; otherwise add one compact labeled route when revising the figure.
- Configuration installation, cryptographic primitives and the physical clock implementation are explicitly established assumptions, so their lack of an implementation algorithm is not a hidden consumer-side operation.

## Suggested repair order

First fix the report/READY/schedule/signing-history interface wording; then synchronize the pipeline labels and caption; finally either box or remove the optional HEAD procedure and add the common evidence-receipt rule. These are local presentation repairs and do not require expanding the consumer boundary.

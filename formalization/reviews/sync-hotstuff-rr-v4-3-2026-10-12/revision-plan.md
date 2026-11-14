# Sync HotStuff RR v4.2 → v4.3: review and revision plan

Date: 12 October 2026
Input: `formalization/sync_hotstuff_rr_v4_2.tex` (11 pages)
Style baseline: `formalization/sync_hotstuff_rr_v3_3.tex`
Output: `formalization/sync_hotstuff_rr_v4_3.tex`

## 1. What v4.2 specifies (reading summary)

- Regions take turns. One turn of region `i_r` is session `r`. A view is
  `(r,g,t)`: session, recovery generation, leader term.
- L1 supplies each session's authority (stake configuration, leader
  schedule, DATA quota `TB_r`) and status updates (status map, force flag,
  target session, anchor height `h_*`, activation height).
- Inside a session, the region runs RR v3.x: proposal timers commit, terminal
  QCs rotate leaders, quit proofs trigger lock-based recovery. New pieces:
  stake-weighted certificates (`> F_i`), a session quota `H_r = h(A_r)+TB_r`
  with a short last term, one DRAIN in a recovery generation that opens at
  `H_r`, FINAL receipts, and completion on a receipt for `T_r`.
- The external chain keeps a DATA prefix of each session. A regular handoff
  keeps `T_r`; an activated L1 force keeps the prefix through `h_*`, even if
  the old region commits higher blocks. Each session is Open, Truncated, or
  Sealed.
- A boundary witness fixes the anchor, successor (`NextLive` over the status
  map at a selection height), and selection height. The successor's members
  enter `(r+1,0,0)` on a local Ready event and run the recovery-style
  NEWVIEW/ANCHOR exchange from the inherited anchor.

## 2. Problems found in v4.2

### A. Structure
- A1. Definitions, protocol actions, and assumptions are mixed. §2.3 holds
  states and endpoints, committee duties, actions ("discard", "rebuild"),
  and liveness conditions ("Requests and progress").
- A2. Cross-region definitions (witness, selection height, Ready) sit inside
  the Protocols section, between Protocol 2 and Protocol 3.
- A3. Protocol 3 mixes two roles: the external-chain watcher (every node)
  and the bootstrap of the successor (successor members only). Its steps 5–6
  repeat Protocol 2 steps 4–5.
- A4. Many rules are stated several times. "Force does not touch regional
  timers" appears six times (Table 2, §2.3, P1 step 5, P2 step 2, P3 step 3,
  remark after P2). The bootstrap relay appears twice; evidence-first
  processing three times.
- A5. Red marks changes relative to v3.1, not v3.3. The boxes are dense
  paragraphs, unlike v3.3's one-rule-per-line format.

### B. Undefined or inconsistent terms
- B1. "drain generation" and "DATA generation" are used in Protocols 1–2 but
  never defined (v4.1 defined the first; v4.2 dropped it).
- B2. "closed session" (§2.3, external acceptance) is undefined; the states
  are Open/Truncated/Sealed.
- B3. "opening control height" (§2.2) is undefined. It is the selection
  height of the boundary that opened the session (P3 step 4).
- B4. "Ready" and "boundary proof" are used in §2.5 before Ready is defined;
  "boundary proof" is never defined (it is the witness).
- B5. "timely-authority assumption" (P2 step 3) does not match the title of
  Assumption 1.1.
- B6. "cross-region infrastructure" (P1 step 3) vs "backbone" (Assumption 1.3).
- B7. "complete-message bound" means the delay bound of Assumption 1.2.
- B8. "registry proof" in the L1 reference is undefined.
- B9. "term-terminal" QC is not defined for short terms or for DRAIN.
- B10. "fresh" and "admissible" are used but not defined (v3.3 defined both).
- B11. "control history" means both the ordered L1/handoff history and the
  old region's DRAIN blocks.
- B12. Symbol `a` is both the activation height and the quit time `a_v`.
- B13. `L(r,g,t)=(i_r, ℓ_{r,g,t})` suggests L1 pre-assigns each session's
  region, but `i_{r+1}` is chosen at the handoff by `NextLive`.
- B14. §3 says "the session index r ... implicit throughout", but the boxes
  write `r` and `Δ_{i_r}`. P1 step 4 reuses `t` for the QC's term (v3.3 used
  `t'`). P2 step 5 introduces a new variable `u`.
- B15. "the existing deadline" (bootstrap timing) is the first progress
  check `e_v+6Δ`.

### C. Ambiguities and likely errors (fixed in v4.3 as noted)
- C1. Definition 3.1: "FORCE *additionally* requires finalized L1 ..." reads
  as if FORCE also needs REGULAR's terminal receipt and full quota. Fix:
  "instead".
- C2. Force selection height: "at or after both `a` and `λ(A)`" does not pick
  one height, so nodes could pick different status maps and successors. Fix:
  `λ_s` is the least height `≥ λ_0` with a live successor, where
  `λ_0 = λ(T_r)` (regular) or `max{λ_a, λ(A)}` (force). This matches "the
  first admissible later prefix with an eligible successor seals the waiting
  boundary".
- C3. Assumption 1.1 orders "finalized L1 updates and regional handoffs", but
  a regular handoff posts nothing to L1, and the text also says an old L1
  reference cannot bypass a cutoff. v4.3 states the control order as an
  assumed service, says `λ(T_r)` only chooses the status map, and gives one
  rule for which event decides a boundary. How the order is built remains
  open (§4 below).
- C4. P1 step 3 says only Quit cancels timers, but P1 step 5 (completion) also
  cancels them. Fix: "Only Quit and Completion cancel a timer."
- C5. Assumption 1.3 delivers the stream "to the next region", which is
  unknown before selection, and gives no delivery guarantee. Protocol 3 needs
  every prospective successor and verifier to receive it. Fix: "eventually
  ... to all other nodes; no fixed bound".
- C6. "The multisignature committee is honest" does not say what honest
  means. The duties are spread over §2.3 and the progress paragraph. Fix: one
  list in §5.2 (current session at posting, `h(A_r) ≤ h_* ≤ H_r`, committed
  and externally available anchor); no-churn moves to the progress
  conditions.
- C7. External acceptance (iii) "The control history assigns its height to r"
  repeats (ii). Removed.
- C8. Truncated vs Sealed: only Sealed is called irrevocable, but activation
  already fixes `End(r)` for good. Fix: Truncated fixes `End(r)`; Sealed also
  fixes anchor and successor; the request pool still waits for sealing.
- C9. A force for a session that has not started is not covered. Fix:
  committee duty "target is the current external session when posted"; a
  target sealed during the notice period changes only the status map (as in
  v4.2).
- C10. The base justification `Q_r^0` is never said to be a concrete object.
  Fix: the receipt of `A_r` carried by the boundary witness.
- C11. P2 step 1 "check an already-due deadline when its opening becomes
  known" is unclear. Fix: the `p=1` check is always scheduled at entry; the
  others are scheduled when `q` is known and run at once if overdue.
- C12. P3 step 1 repeats step 3 (discard above cutoffs). Removed.
- C13. The witness tuple carries both `r` and `r+1`. Kept `r` only.
- C14. Genesis: v4.2 opens session 0 through the bootstrap exchange (with an
  ANCHOR QC), while v3.3 says genesis needs no ANCHOR exchange. v4.3 states
  this difference explicitly.

### D. Wording
- Long sentences with stacked clauses (e.g. the key-compromise sentence in
  §1.2), jargon ("electing a leader by reporting a tip", "passive status
  entry", "correctly operating prefix", "authenticated control cursor", "a
  newer local L1 head cannot reselect it"), an orphan opening line in §2, a
  redundant "Equivalently, f_i = F_i/W_i" sentence, and a view table whose
  last row mixes the successor's entry with the old region's timers.

## 3. Revision plan

1. **Structure (mirror v3.3, two halves).**
   §1 Overview (three layers, two endings, four view changes);
   §2 Model and assumptions (L1 records split into authority and status
   update; stake; four assumptions; conditions for progress);
   §3 Regional engine: definitions; §4 Regional protocols (Protocols 1–2);
   §5 External chain and handoff: definitions; §6 Cross-region protocols
   (Protocols 3–4); §7 Examples.
2. **Protocol boxes in v3.3 format.** Header line "Replica v runs these
   steps ...", bold numbered steps, one rule per bullet, (a)–(g) sub-steps.
   Protocols 1–2 copy v3.3's wording wherever the rule is unchanged.
3. **Red = difference from v3.3.** Applies to the boxes, the
   message-processing order, and the quit-proof list. Pure renamings
   (replica `r`→`v`, `B_k`→`B`, check index `i`→`p`, implicit session `r`)
   are listed once and not marked. Protocol 3 is new and fully red.
   Protocol 4 reuses v3.3's recovery steps 3–5; only its differences are red.
4. **Split Protocol 3.** Protocol 3 = external chain (every node). Protocol 4
   = bootstrap (successor members), reusing Protocol 2 step 5.
5. **State each rule once.** Definitions in §3/§5, actions in the boxes,
   one short note after each box.
6. **Define every term before use** (B1–B15) and **fix C1–C14.**
7. **Simple sentences.** One idea per sentence; no stacked qualifiers.
8. **Figures.** A small state diagram (Open/Truncated/Sealed) and one figure
   for the two endings, using the running example `h(A_r)=100`, `TB_r=10`,
   `K=4`.
9. **Scope.** No protocol rule is added or removed beyond C1–C14. Security
   and complexity analyses stay deferred.

## 4. Open issues left to the author

- How the control order of Assumption 2.1 is realized without an L1
  transaction per regular handoff. A Byzantine leader can choose an old
  `λ(T_r)`, so the order cannot simply be "by `λ(T_r)`"; v4.2 and v4.3 assume
  it as a service.
- The exact content of the registry proof in `ρ`.
- Liveness when the committee forces a working region repeatedly (now an
  explicit progress condition, not an assumption).
- The old region's engine keeps running after a force until it completes;
  the draft does not say when it may stop.

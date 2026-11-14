# Sync HotStuff RR v4.4 → v4.5: review and revision plan

Date: 12 October 2026
Input: `formalization/sync_hotstuff_rr_v4_4.tex` (1,550 lines, 20 pages)
Output: `formalization/sync_hotstuff_rr_v4_5.tex` (24 pages, including a 2.5-page change log)
Diff: `output/pdf/sync_hotstuff_rr_v4_5_diff_v4_4.pdf` (latexdiff, v4.4 → v4.5)

Scope: clarity, presentation, and fixes to ambiguous or unhandled cases.
The protocol workflow is unchanged. Every protocol step of v4.4 is kept,
in the same order and under the same numbers (Protocols 1–6, Sections 1–9).
Line numbers below refer to the v4.4 source.

## 1. What v4.4 specifies (reading summary)

- Regions take turns. One turn is session `r`, served by region `i_r`. A view
  is `(r, g, t)`: session, recovery generation, and leader term.
- Inside a session, the region runs RR v3.3 with stake weights (`> F_i`), a
  DATA quota (`H_r = h(A_{r,0,0}) + TB_r`), DRAIN for recovery at the endpoint,
  FINAL receipts, and completion on a receipt for `T_r` (Protocols 1–2).
- Sessions end by a regular handoff (receipt for `T_r`, successor chosen by
  `NextLive` at a selection height) or a forced handoff (an activated L1 Force
  that keeps the target's prefix through `h_*`). Protocols 3–4.
- Finality has two levels. Regional finality means a valid receipt from the
  assigned region and session. Global finality at observer `u` also needs
  `z ≥ s_u(B) + W` and that no finalized Force removes `B`, where `s_u(B)` is
  the time `u` relays the complete evidence to the committee.
- The honest committee registers `m(B)`, protects blocks at `m(B) + T_msig`,
  and keeps one outstanding Force. A Force goes through Submitted,
  L1-finalized, and L1-activated stages (Protocol 5). Observers promote
  blocks with Protocol 6. Section 9.3 gives a conditional protection argument.

## 2. Problems found in v4.4

### A. Structure, redundancy, and verbosity

- **A1. The abstract and overview are not standalone.** The abstract
  describes v4.4 relative to v4.3 (l. 60) and uses terms that are never
  defined (KEEP, l. 72; "the waiting rule", l. 102). The overview lists three
  layers but never says who runs which protocol.
- **A2. The Force deletion scope is restated six times, in different
  words:** "removes B, directly or by replacing an ancestor's regular
  boundary" (Def. 5.2, l. 1072); "higher blocks and every descendant of its
  replaced regular boundary" (P3, l. 1168); "its whole discarded subtree"
  (§7.1, l. 1270); "the target's DATA above h_* and all descendants" (P5,
  l. 1347); "the affected regional suffix and its old regular descendants"
  (P6, l. 1410); "full history retained by the force must extend G_msig"
  (P5, l. 1325). A reader cannot tell whether these mean the same set.
- **A3. "The wait starts at relay, not at the L1 reference" appears seven
  times:** abstract, §1 (l. 102), §3.1 (l. 422–426), §5.2, all of §5.3
  (l. 1090–1102), §8 introduction (l. 1379), and §9.1.
- **A4. "A finalized but inactive Force blocks only global promotion"
  appears seven times:** §2.1 (l. 182), §5.1, Def. 5.2, Table 6, P5 step 2,
  P6 step 2, and the Figure 1 caption.
- **A5. Session identity ("by opening and hash, not by index or height") is
  restated six times** (§1, §6.1, P3 step 3, §6.3, §8, §9.2), but never as a
  formula.
- **A6. Other duplicates.** The one-outstanding-Force rule is in §2.1
  (l. 193) and §7.2 (l. 1303). The request-ordering rule is in §2.4
  (l. 313–317) and §8.1.
- **A7. Assumption 5 mixes kinds of statements.** `T_mon` and `T_L1` are
  environment bounds, but the `T_msig` bullet (l. 272–275) is a committee
  protocol rule, which §7.1 repeats.
- **A8. Forward references.** §5 (finality) uses the branch, session
  identity, "applicable" Force, and deletion scope, all of which §6–§7
  define later.
- **A9. The committee's event order is long prose** (l. 1263–1268). It mixes
  ordering, a recheck, and a no-delay rule.
- **A10. Protocol 5 mixes roles.** Step 1 is the committee's and steps 2–3
  belong to "L1 watchers", a role defined nowhere. The step titles do not
  say which.

### B. Terms and notation

- **B1. "Opening" has two meanings.** It means the generation's opening
  block `A_{r,g,0}` (v3.3 sense: "opening identity", l. 563; quit proof (c);
  Protocol 2). It also means the boundary that opened a session ("session's
  opening boundary", l. 539; "session opening", l. 688; "opening identity" in
  P3 step 3, l. 1181, and §8, l. 1418). "Opening identity" is defined only in
  the first sense but used in the second.
- **B2. "Branch" is used 31 times without a definition.**
- **B3. Roles are inconsistent.** The text uses node, observer, L1 watcher,
  verifier, multisignature service, monitor, and committee, without saying
  how they relate.
- **B4. Undefined terms:** passive status update, outstanding Force,
  applicable Force, terminal receipt, regional output, and "currently
  applicable" boundary witness.
- **B5. "KEEP message" is never defined.** It refers to a rejected
  alternative (l. 72, 1085, 1238, 1471).
- **B6. Symbol clashes and unused symbols.**
  - `W` (the waiting period, l. 368) clashes with `W_i` (regional weight).
  - `m(B)` (committee time, l. 365) clashes with `m_{r,g,t}` (term length).
  - `e` (first Ready time, l. 1224) is undefined and clashes with `e_v`.
  - `Ξ`, `\mathcal W`, `\RF`, and `\GF` are defined but never used.
  - `θ_a` is defined, but no rule uses it.
- **B7. `F_i` is a certificate threshold but is not listed among the
  configured parameters** (Table 2, l. 152), so it reads as a model-only
  bound.
- **B8. The notice period has no unit and no reference point** (l. 180):
  L1 blocks or time, counted from posting or from finalization?

### C. Ambiguities and unhandled cases

- **C1. `AUTH_{r,g}` is ambiguous.** It is "for generation (r,g) on the
  branch opened by the session boundary" (l. 138). L1 cannot see off-chain
  regular boundaries, and a Force may reuse index `r`. It is also unclear
  what an authority for `g > 0` may change, given that membership, weights,
  and the quota are fixed per session.
- **C2. Status updates can be reordered on L1.** v4.4 requires activation
  heights to increase in L1 order (l. 184), yet lets passive updates be
  submitted while a Force is outstanding (l. 195). If L1 includes the later
  passive update first, the Force's activation height no longer increases.
  The Force is then malformed, but the committee has already stopped
  protecting its scope. Nothing says what nodes do with such a record.
- **C3. It is never stated that `χ(λ)` is fixed once height `λ` is
  finalized.** That property is what makes successor selection
  deterministic, and it follows from the notice rule.
- **C4. Genesis is underspecified.** "If no region is live, initialization
  waits" (l. 200), and `λ^open_0` is undefined. The first DATA block of
  session 0 needs it.
- **C5. Receipt validation needs replica-local state.** Def. 3.1(iii)
  requires "admission", a replica-local event. An external verifier has no
  admission state.
- **C6. Commit from a receipt is underspecified.** "A replica may commit the
  DATA prefix of a valid receipt" (l. 615) is in no protocol box, and its
  effect on FINAL and timers is unstated.
- **C7. A lagging node cannot verify a Force's target.** P5 step 2 says
  "verify ... target opening" (l. 1345). The backbone has no delay bound, so
  a node may not yet know the target session. If it rejects the record while
  other nodes accept it, nodes diverge.
- **C8. Node-side validity checks for a Force are not listed.** "Malformed or
  unauthorized records are rejected" gives no list: what about the range of
  `h_*`, a target that was already forced, or a second Force before the
  first activates?
- **C9. P3 says "Transfer its full bundle"** (l. 1161, 1175) but not to
  whom.
- **C10. P3 step 2 says "Keep the same protected and global prefix"**
  (l. 1171), but nodes have no protected prefix; only the committee does.
- **C11. The promotion test needs a current L1 snapshot.** The safety
  argument needs an L1 snapshot that is current at promotion time `z`, and
  Def. 5.2 and P6 do not require one.
- **C12. Wrong cross-reference.** `e_v(r,0,0)` is said to be set "in step 2
  of Protocol 4" (l. 867); it is set in step 1.
- **C13. Table 1's regular-handoff timer cell is inaccurate.** It says "The
  old engine is already complete" (l. 121), which need not hold when the
  successor enters. The old engine completes in its own Completion step.
- **C14. Unclear Force-target wording.** "Each exact target opening can be
  forced only once" and "an opening ... whose endpoint has not already been
  fixed by a Force" (l. 1306–1308) are hard to read.
- **C15. `G_u` has two meanings:** the trusted starting checkpoint
  (l. 1007) and the moving end of the global prefix (P6).
- **C16. "B's position" in Def. 5.1(i) is ambiguous** (l. 1017): DATA height
  or physical position?

## 3. Revision plan

1. **Keep the workflow.** No protocol step is added, removed, or reordered.
   Section and protocol numbers stay as in v4.4. Protocols 1–2 keep their
   v3.3 wording and red marks.
2. **Define each concept once and cite it:**
   - the deletion scope `Del(F)` (§2.1), used by Def. 5.2, §7, and Protocols
     3, 5, and 6 (A2, A8);
   - the session ID `σ_{r+1} = H(σ_r, mode, H(anchor), i_{r+1}, λ_s, Force
     ID)` (§6.1) (A5, B1);
   - branch and current branch (§1, §6.1) (B2).
3. **Rename for one meaning per word.** "Opening" keeps only its v3.3 sense,
   and the session sense becomes "session ID" (B1). `W` becomes `T_wait`,
   `m(B)` becomes `μ(B)`, and the first Ready time is written as a minimum of
   `e_v` (B6). "L1 watcher" becomes "node", and "monitor" becomes
   "committee" (B3).
4. **Add a participants table to §1** and rewrite the abstract as a
   standalone summary (A1, B3).
5. **Remove repetition.** The rationale for the relay-based wait appears
   once (§5.3, condensed) (A3). The "finalized Force blocks promotion" rule
   stays only where it is normative (A4). Duplicated rules are merged (A6).
6. **Split Assumption 5.** `T_mon` and `T_L1` remain environment bounds, and
   `T_msig` becomes a parameter of the committee rule (A7). The committee's
   event order becomes a numbered rule (A9). Protocol 5's steps are labelled
   by role (A10).
7. **Fix C1–C16** with the smallest rule that resolves each one:
   - AUTH is keyed by index; for `g > 0` only the leader schedule is used
     (C1).
   - The committee submits one update at a time, and nodes ignore updates
     that break the notice or ordering rule (C2, C8).
   - Determinism of `χ(λ)` is stated (C3).
   - Genesis uses the general selection rule from `λ_gen` (C4).
   - Verifiers check admission witnesses in the evidence (C5).
   - Commit from a receipt sends no FINAL (C6).
   - A node keeps a Force with an unknown target until the target's boundary
     arrives (C7).
   - The remaining items are wording and reference fixes (C9–C16).
8. **Mark every change.** Blue text is new or changed relative to v4.4, and a
   blue tag links each change to its entry in Appendix A. Red keeps its v4.4
   meaning (a difference from v3.3). `\revmarksfalse` in the preamble gives a
   clean copy. A latexdiff PDF shows deletions inline.

## 4. Open issues left to the author (not changed)

- **D1. Old-engine lifetime.** An engine whose session was cut or discarded
  keeps running until it completes. When may it stop proposing?
- **D2. The format of the registry proof in `ρ`.**
- **D3. Leader influence on successor selection.** A Byzantine leader may
  choose an old `λ(T_r)` (bounded below by `λ^open_r`) to influence
  successor selection.
- **D4. One live region halts rotation.** With only one live region, quota
  rotation stops because `NextLive` excludes the current region.
- **D5. Publishing future authorities.** How L1 publishes `AUTH_{r,g}` for
  future indices without an L1 transaction per regular handoff.
- **D6. Activation-height margin.** The margin in `λ_a` that the committee
  needs depends on the L1 block rate, which the model does not bound.
- **C1 is an interpretation.** v4.5 reads "AUTH keyed by index, shared by
  sessions that reuse the index". If L1 is meant to key authorities
  differently, the paragraph "Authority records" in §2.1 is the only place
  to change.

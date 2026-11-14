---
name: review-spec
description: Reviews pages of the MegaETH Specification the way an auditor will read them — adversarially, against eight ranked criteria (precise semantics, cross-page consistency, explicit assumptions, safety under adversarial behavior, liveness and recovery, sound rationale, verifiability, presentation last). Use when asked to review, audit, or attack a spec page or section, or to check a draft before it goes to audit. Produces a findings register under .sisyphus/plans/, tracks the author's concurrent edits by file hash, and keeps implementation discrepancies separate from specification defects. Suitable for delegation — a subagent given a scope and this skill has everything it needs.
---

# Review Specification Pages

Review MegaETH Specification pages in this repository: $ARGUMENTS

Parse the arguments to determine scope.
Accepted inputs:

- A page path (e.g., `specs/block-certification/batch-inbox.md`) — review that page together with every page whose rules it depends on or whose guarantees depend on it.
- A section (e.g., `specs/block-certification/`) — review the section's pages as one unit, because cross-page consistency is a criterion and most defects live between pages.
- `all` — the whole specification.

The deliverable is a findings register, not an edited specification.
The review touches `specs/` only for structure-and-presentation fixes, as exact replacements that change no rule (see [Phase 4](#phase-4--classify-and-hand-over)).

## Stance

- The specification is the object under review, and the reviewer is its adversary: the aim is to find inputs, orderings, and parties for which the text as written produces a wrong, divergent, or undefined outcome, not to confirm that it reads well.
- The eight criteria are ranked, and conciseness wins every tie with readability: a dense page that resolves every ambiguity is a good page, and wording is a finding only where it produces ambiguity.
- The author keeps editing while the review runs, so nothing counts as reviewed until its SHA-256 is recorded, and every later change to a reviewed file triggers a diff and a re-evaluation.
- The implementation is a side channel: consult it for facts, log where it differs, and never let it decide what the specification means.
- The reviewer proposes; the author decides.
- A design change is applied only after the author has chosen it, and then exactly as chosen.

## Criteria

### 1. Precise and Complete Semantics

Every rule is a total function of its stated inputs.

- Rewrite every MUST as "actor, on event, MUST do X" and refuse any slot left empty: a MUST without an actor is unenforceable, and a MUST without an event never fires.
- Check that every decoder, predicate, and procedure is total over extremal and aberrant inputs: empty, maximal, duplicate, out-of-order, overlapping, truncated, wrapping counters, and the boundary value of every constant.
- Reject any definition made by reference to unpinned behavior: an implementation, "the usual" handling, or an upstream page cited without an exact section and version.
- Enforce one term, one definition site, per the [terminology discipline](../write-spec/SKILL.md#terminology-discipline) of `write-spec`; a term used two ways across pages is a semantic defect, not a style nit.
- Require that every MUST admits a test assertion; if the assertion cannot be written, the rule is not yet precise enough to implement twice identically.

### 2. Consistency and Interoperability

Two independent implementations built from the text alone must produce byte-identical outputs.

- Trace every guarantee stated on one page to the rules on other pages that make it true; a guarantee no rule supports is a finding under criterion 3.
- Build a cross-rule interaction matrix: rules across the columns, disruptive events down the rows (L1 reorganization, privileged override, key rotation, attestation or measurement retirement, activation on a running chain, channel or message timeout, window expiry, forced substitution, external consensus epoch change, contract upgrade), and answer every cell or mark it open.
- For every value two implementations could compute differently — an ordering, a tie-break, a rounding, a resume point, an encoding — either find the sentence that pins it or record the divergence.
- For delta pages, pin the exact upstream section and version, and check that the page's divergence register lists every delta the page actually makes and no other.

### 3. Explicit Assumptions and Guarantees

Every guarantee is a claim of the form "under assumptions A, property P holds", and every assumption is written down where it enters.

- Write the threat model out: every actor the pages name, whether each is treated as honest, rational, or Byzantine, and which combinations each guarantee tolerates.
- Decompose each guarantee into the proof obligations its argument needs, and discharge each obligation by naming the rule that meets it; an obligation with no rule is a gap.
- Name the trust premises — governance, enclave integrity, external consensus, data availability, correct initialization — at the point the argument relies on them, not in a closing disclaimer.
- Flag every invented guarantee: a claim that holds in the reference implementation but that no rule in the specification implies is a finding even though it happens to be true today.

### 4. Safety Under Adversarial Behavior

Each rule is attacked by every party that could profit from breaking it.

- Run an enforcement test on every MUST: if the addressed party ignores it, who detects the violation, what rejects the result, and what happens if nothing does; a MUST nothing enforces is either a gap or an out-of-scope policy under the repository's [scope principle](../../../AGENTS.md#scope).
- List the invariants first, then attack each with the cheapest violation available to each actor.
- Build attack trees by goal — settle uncertified content, lose certified content, stall progress, split observers into disagreeing views, escalate a role's privilege — and record for each leaf whether a rule blocks it.
- Apply the [RFC 3552](https://www.rfc-editor.org/rfc/rfc3552) attack list — replay, insertion, deletion, modification, reordering, timing (delay, staleness, future-dating), and denial of service — to every message type the pages define: records, envelopes, frames, attestations, contract calls, and events.
- Examine escape hatches first: overrides, administrative functions, upgrade paths, and initialization, because they carry the broadest authority; their breadth is a design decision to record, and their observability is a defect when it is missing.
- Consider rational deviation as well as malicious: what a party gains by deviating while staying inside every rule.

### 5. Liveness and Recovery

Every failure mode has a detection and either a recovery path or an explicitly accepted limitation.

- For each failure mode, record who notices, from what evidence, and how soon; a failure only a privileged party can see is a finding.
- Make timing and contention assumptions explicit: deadlines, windows, timeouts, who can race whom, and what a lost race costs.
- Compose failures in pairs — a reorganization during recovery, a timeout during an override, a key rotation across an activation — because single-failure scenarios are where the author has already looked.
- Qualify every no-loss or eventual-progress claim by exactly the conditions under which it holds, and reject any claim that holds only if an external service never stops.

### 6. Sound Rationale and Security Analysis

Rationale and Security Considerations sections argue; they do not restate.

- Check that Security Considerations follow the RFC 3552 shape — assets, threat model, mitigated attacks, residual risks — and that no paragraph merely restates a rule from a behavioral section.
- Require every Rationale paragraph to name the alternative and say why it lost; a rationale that only repeats the rule is a finding.
- Test every claimed necessity by removal: delete the rule the paragraph defends and see what breaks; if nothing does, the necessity is invented.
- Audit at paragraph level: judge every Rationale and Security Considerations paragraph as relevant or not and supported or not under the page's own assumptions, and record every judgment, including the supported ones, so coverage is visible.

### 7. Verifiability, Conformance and Traceability

Every fact the review relies on is checkable by someone else, later.

- Verify upstream facts at the cited source at a pinned revision, and record the sources used with their revisions.
- Consult implementations fetch-only, never changing a checkout, and log every discrepancy in a separate implementation register — never as a specification defect, and never silently resolved in either direction; the author decides which side is intended.
- Write an executable counterexample for any trace longer than a few steps: a small script that models the rule as written and prints the divergence, with its output saved beside it.
- Check activation and compatibility: how each rule applies to a running chain at the boundary, with in-flight state on both sides.
- Require a test vector, or a placeholder calling for one, for every consensus-critical encoding, digest, and event signature.

### 8. Presentation, Ranked Last

Conciseness wins every tie.

- Report presentation only where ordering or wording produces ambiguity or a misleading heading; pure style is not a finding.
- Fix structure-and-presentation findings directly as exact replacements instead of reporting them, and log each edit.

## Workflow

### Phase 1 — Map Before Attacking

1. Read every in-scope page in full, then record the SHA-256 of each in `reviewed-revisions.json` before writing anything else.
2. Build `MAP.md`: the actors and the trust each page places in them, the invariants the pages claim, the enforcement map (rule, who checks it, what happens on violation), the interaction matrix of criterion 2, and each guarantee decomposed as in criterion 3.
3. Pin the sources: the upstream sections a delta page modifies, and the exact revisions of any implementation consulted.
4. Settle the criteria with the user before attacking, and record them at the top of `REVIEW.md`; the eight above are the default.

### Phase 2 — Attack

1. Work the criteria in rank order and note every hit as a candidate finding.
2. Try to kill each candidate before writing it up: a misreading, a rule on another page that covers it, or the scope principle placing it outside the specification; record dismissed hypotheses too, so the next reviewer does not repeat them.
3. Write an executable counterexample for every candidate whose trace exceeds a few steps.

### Phase 3 — Write Findings

Use the entry format in [references/finding-template.md](references/finding-template.md): a title that states the defect as a claim, Source, Prerequisites, Trace (or the competing interpretations, for an ambiguity), Violated property, Impact, Severity and Confidence recorded separately, Class, Correction, cross-references, Status.
Rank findings most severe first.
Keep demonstrated defects, open questions, accepted limitations, and editorial observations apart; several findings that describe failed premises of one stronger guarantee are one guarantee's problem, and the register says so.

### Phase 4 — Classify and Hand Over

Place every finding in exactly one of three categories, and handle each category differently:

1. **Structure and presentation** — the reviewer fixes it directly in `specs/`, as exact replacements that change no rule, one sentence per line, followed by `mise run lint`; each edit is logged.
2. **Unclear or ambiguous statements** — the reviewer presents the competing readings and a proposal; the author chooses the meaning; the reviewer applies the choice.
3. **Protocol defects or gaps needing a design change** — the reviewer presents the counterexample and a proposal with its trade-offs; the author decides; nothing is applied without a decision, and what is applied is exactly what was decided.

Never change a rule on the reviewer's own authority, and never widen a decided change while applying it.

### Phase 5 — Track Revisions Until the Review Closes

- Whenever an in-scope file's SHA-256 differs from `reviewed-revisions.json`: diff it against the reviewed revision, re-evaluate every finding whose Source cites that file, log the outcome in `REVIEW.md`, and update the hash.
- Keep a snapshot of every observed revision keyed by its hash, so a diff is always against the bytes that were reviewed rather than a reconstruction.
- Never delete a finding: its status moves among open, resolved, weakened, superseded, and withdrawn, with the hash it was last evaluated against.
- Guard the reviewer's own edits with the hash they were prepared against: if the file moved meanwhile, skip it, diff the author's change, and re-issue the edit against the new revision.

## Register Layout

The register lives at `.sisyphus/plans/<review-name>/`, which is gitignored, so a review survives across sessions without entering version control.

- `REVIEW.md` — the criteria as settled with the user, the scope, the revision-tracking rule, the artifact list, and an activity log with UTC timestamps taken from the clock, not estimated.
- `FINDINGS.md` — the findings, most severe first, a Classification section, and the implementation register at the end.
- `MAP.md`, `reviewed-revisions.json`, `counterexamples.py` with its saved output, and a sources file or section with pinned revisions.

## Conventions

- Cite sections by heading, not by line number; line numbers do not survive the author's next edit.
- Confirm every "the implementation does X" by reading the implementation at a recorded revision, fetch-only.
- Keep implementation content out of the specification, and specification content out of the public documentation repository.
- Write one sentence per line in every register file and every edit, as the repository requires.

## Common Pitfalls

**Accepting a page's own security claim as the property to check.**
A page that says a fault is "self-authenticating" or that an attack is "prevented, not detected" has stated a conclusion, not a property; derive the property from the rules, check whether the rules deliver it, and treat the claim's wording as a finding when they do not.

**Resolving a design ambiguity from the reference implementation.**
The implementation shows what one team built, not what the specification means; when the two differ, the discrepancy goes to the implementation register and the question goes to the author.

**Deleting a finding because the author's edit removed the text it cited.**
Mark it superseded, diff the replacement, and re-check whether the defect moved rather than disappeared; the same defect often survives a rewording.

**Reporting a stall as a design choice.**
A rule that can leave the protocol unable to make progress — a cursor nothing can advance, a channel nobody can close — is a defect under criterion 5 whatever the page's rationale says, unless the page explicitly accepts the limitation and names the recovery.

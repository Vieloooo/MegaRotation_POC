# Finding Template and Checklists

## Finding Entry

Every finding in `FINDINGS.md` uses this shape; the identifier prefix is the review's own (for example `CR-` or `BC-`).

```markdown
## XX-NN. <The defect stated as a claim: what goes wrong, for whom, under what condition>

**Source.** <page, section heading, and the quoted phrase the finding rests on>
**Prerequisites.** <what must hold for the trace: assumptions, actors, prior state; "None" if the defect is unconditional>
**Trace.** <step by step from the rules as written to the wrong outcome; for an ambiguity, the competing interpretations and what each implies>
**Violated.** <the invariant or guarantee, by the identifier MAP.md gives it>
**Impact.** <who loses what, and whether the trace is certain in normal operation, attacker-triggered, or needs a coincidence>
**Severity.** <Critical | High | Medium | Low>. **Confidence.** <High | Medium | Low, with the reason>. **Class.** <Defect | Ambiguity | Accepted limitation | Editorial | Open question>.
**Correction.** <the smallest change that closes it, and the alternative if there is one, with the trade-off>
**Cross-reference.** <the other review's finding, the upstream issue, the implementation location>
**Status (<date> <time> UTC).** <open | resolved | weakened | superseded | withdrawn; what changed, and the hash it was evaluated against>
```

## Severity and Confidence

Severity is what the defect costs if it is real; confidence is how sure the reviewer is that it is real.
Record them separately: a Critical entry at Low confidence is a lead to run down before anything else, and a Low entry at High confidence is a fix to make without discussion.

## Status Vocabulary

- `open` — the defect stands against the hash recorded.
- `resolved` — the author's revision or the decided edit closes it; the line says what changed.
- `weakened` — the revision reduces the impact or narrows the prerequisites but does not close it; the line says what remains.
- `superseded` — the cited text is gone; the finding is re-evaluated against the replacement in a new status line or a new finding.
- `withdrawn` — the reviewer's own error; the line says what was misread, so the mistake is not repeated.

## RFC 3552 Attack Grid

One row per message type the pages define; one column per attack class.
A cell holds the rule that blocks the attack, or `open`.
An empty cell is a review gap, not a clean result.

| Message type    | Replay | Insertion | Deletion | Modification | Reordering | Timing | Denial of service |
| --------------- | ------ | --------- | -------- | ------------ | ---------- | ------ | ----------------- |
| <record>        |        |           |          |              |            |        |                   |
| <envelope>      |        |           |          |              |            |        |                   |
| <frame>         |        |           |          |              |            |        |                   |
| <attestation>   |        |           |          |              |            |        |                   |
| <contract call> |        |           |          |              |            |        |                   |
| <event>         |        |           |          |              |            |        |                   |

## Interaction Matrix

Rules across the columns, disruptive events down the rows.
A cell holds the sentence that resolves the interaction, `open`, or `n/a` with the reason.

| Event / Rule                          | <rule 1> | <rule 2> | <rule 3> |
| ------------------------------------- | -------- | -------- | -------- |
| L1 reorganization                     |          |          |          |
| Privileged override                   |          |          |          |
| Key rotation                          |          |          |          |
| Attestation or measurement retirement |          |          |          |
| Activation on a running chain         |          |          |          |
| Channel or message timeout            |          |          |          |
| Window expiry                         |          |          |          |
| Forced substitution                   |          |          |          |
| External consensus epoch change       |          |          |          |
| Contract upgrade                      |          |          |          |

## REVIEW.md Skeleton

- `## Criteria` — the ranked criteria as settled with the user, verbatim.
- `## Scope` — the in-scope files, as they stand in the working tree including uncommitted edits.
- `## Revision Tracking` — the hash rule: what is recorded, what a change triggers, and that a finding is never deleted.
- `## Artifacts` — one line per register file.
- `## Activity` — one line per event, timestamped in UTC from the clock, newest last.

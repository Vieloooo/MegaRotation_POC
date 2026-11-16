# Source projection and RESUME coverage clarification

Date: 2026-11-16. **PASS as a source interpretation of frozen r1.** This review read v10 06 §§3.2–3.4, CERT-1/3/4; v10 07 §3.7; and batch-services r1 §§4–5 at `8fb46c296808dbe97c95df7a2761678412b95f62`. No evolving M2 source or Builder implementation was inspected.

A fixed immutable gpos snapshot containing G[1..4] may certify [1,2] and then [3,4] without selecting a newer gpos, provided each request's **full prev** equals the Certifier's own current finalized reader cursor and all other DA/source/range guards pass. Ordinary acceptance advances Inbox progress; it does not alter G or create a RESUME obligation. v10 06 §3.4 requires the latest relevant RESUME to be processed, not GFR to replay every later ordinary receipt or to use gpos newer than cursor `written_at`.

Keep the two completeness claims distinct. The Certifier's **current complete 09 history** establishes the current cursor and whether a relevant RESUME exists. If that history establishes `latest_resume=None`, the source's declared control coverage need not extend to the later ordinary acceptance position. An unverified `None`, missing history, or a source-only assertion of no RESUME is insufficient and must WAIT. This clarification removes no current-prev, independent execution, source identity or recovery-covered-height guard.

If `latest_resume=Some`, select the last **CursorSet at or before `cursor.written_at`**, then require the source to have processed that Set's exact paired Resume position and control sequence. Declared source control coverage must include that Resume position. For Set `(b,t,0)` and paired Resume `(b,t,1)`, a cursor written at `(b,t,0)` already requires the Resume at log 1; comparing Resume itself with written_at would incorrectly omit it. The obligation persists when later ordinary acceptance changes sourceKind back to GLOBAL. Gpos byte ordering or matching sequence without matching position cannot satisfy it.

For suffix projection, start from the snapshot's authenticated/declared actual anchor state and native parent, execute its contiguous raw virtual prefix through `prev.height`, and only then collect requested records `[prev.height+1..last]`. Validate each prefix metadata/DEP/pre-state-root along that path. Do not restart execution at genesis for every batch, derive state from cursor height alone, or trust Batcher-provided Kept. Missing prefix/state/DEP is WAIT; a known contradictory pre-state root/content is REJECT. A valid precomputed state checkpoint could serve as the starting anchor only under the existing source evidence boundary; this review does not add such a capability.

Independent expected cases (IDs `projection:SP01–SP06`, separate from contract:B01–B18):

| ID | Input | Expected |
| --- | --- | --- |
| SP01 | One immutable gpos contains heights1–4; complete current 09 history proves no RESUME. Accept1–2, then request3–4 with the new full prev. Source control coverage predates the first receipt. | Both may certify; same gpos; second batch executes from source state after height2. No coverage-to-new-receipt or new-gpos guard. |
| SP02 | Same request but current L1 history is incomplete and `None` is only a caller assertion. | WAIT; absence of RESUME has not been established. |
| SP03 | Set `(7,2,0)` writes current cursor; paired Resume `(7,2,1)`, seq9. | Source must declare exact `(7,2,1,seq9)` processed and cover that position. Coverage only through Set or missing processed pair gives WAIT. |
| SP04 | An ordinary batch later writes a GLOBAL cursor; the last relevant Set is still the one in SP03. | Same precise Resume requirement remains; GLOBAL tag does not clear it. |
| SP05 | A request begins at height3, but source's anchor is height0 and height1/2 input/state is missing or contradictory. | Missing material WAIT; known mismatch REJECT. No direct projection using height0 state. |
| SP06 | The same gpos identifier is reused with altered source data or alleged different processed history. | Immutable-source conflict, not a permissible refresh. A changed snapshot needs its own valid source identity. |

Concrete manual execution oracle for SP01: anchor accounts `1=(nonce0,balance10)`, account2 absent; each height has only its matching Attr in DEP. h1 Transfer(1,2,0,3) yields `1=(1,7),2=(0,3)`; h2 Transfer(1,2,1,2) yields `1=(2,5),2=(0,5)`; h3 Transfer(2,1,0,1) yields `1=(2,6),2=(1,4)`; h4 Transfer(1,2,2,4) yields `1=(3,2),2=(1,8)`. Every transfer is Kept. Applying h3/h4 directly to the anchor would drop both and is an independently detectable failure.

No wire, signature domain, role, source trust model or protocol requirement changes are needed. Lead may record this clarification in r2 for implementation traceability before the affected candidate is finalized. It is not M2 implementation PASS, actual GFR proof, or an exemption from complete current L1 history.

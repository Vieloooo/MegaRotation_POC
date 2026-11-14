# v3.10 input integration notes

These fragments integrate the reviewed hydration-to-Ready adapter. The frozen Model and all v3.9 sources are unchanged by this author. The parent explicitly authorized a v3.10 copy of the public-import fragment because its old live-pool REPORT paragraph and accepted-report-awareness sentence conflict with the replacement rule; its import algorithm and proposition are retained.

## Files and include graph

- Main replaces the old requests include with `sections/rotation_v3_10_requests.tex`.
- Requests includes `rotation_v3_10_public_requests.tex`, `rotation_v3_10_ownership.tex`, then `rotation_v3_10_input_selection.tex`; all remain inside the requests subsection. Main must not also include ownership or selection separately.
- Main replaces the old ReadyBundle include with `sections/rotation_v3_10_ready_bundle.tex`.
- Ownership must provide `sec:request-ownership`, `prop:ownership-formation` and `prop:ownership-coverage`. The coverage interface quantifies every valid HYDRATED certificate outside a finite exceptional target set; each places complete x bytes and RawJournal handles at every correct target strictly before that certificate exists. No honest-set intersection is assumed.
- Progress can cite `prop:opening-formation`, `prop:opening-coverage`, `prop:session-input-progress`, and the retained alias `prop:v38-report-timing`. `prop:v38-request-inclusion` is a compatibility alias for the new session-first theorem, not the obsolete report-awareness statement.

## Main and protocol hooks

1. Ready definition: require one complete `CheckOC(A, AUTH[r,0], OC)` certificate matching the exact preparation key. The transmitted gate is OC, not a second HYDRATED signature set. Initial Enter durably attaches the independently validated executable state and this certificate before recording the entry time and signatures.
2. Preparation: node-level manager routes and retains raw receipt handles, STATE_CANDIDATE, offers and complete OC. A candidate contains the original executable Ready state/dependencies, grants no entry or timer, and is sent before waiting for offers or Ready. Every correct offer follows a full HYDRATED receive and actual completion of a finite frozen raw/canonical cut at exact A. All ordinary public/control evidence still reaches its original handler.
3. Initial and recovery NEWVIEW: when own-session DATA is absent from the complete selected-parent ancestry, include the complete cached matching OC and one persistent initializer-signed INPUT_ADVERTISE binding exact RC, canonical OC digest and selected key. Original signed NEWVIEW bytes and equivocation predicate remain unchanged. A descendant history carries the historical first-DATA witness instead.
4. REPORT: original ordinary-entry and NEWVIEW trigger times remain. Opening reports acknowledge one initializer-advertised OC; descendant reports are DISCHARGED/EMPTY. One durable report per RC. Invalid/missing sidecars prevent only REPORT and do not suppress original NEWVIEW forwarding, ANCHOR voting, evidence inspection or timers. A later valid sidecar can complete the action only under the still-current fresh-action gate.
5. P1 proposal: at term-first j=0 collect >F matching reports for the exact RC and mode. At an opening parent require the same advertised OC digest/key; a nonempty selected request executes first. Later terms use ordinary FIFO once the first-own-session ancestry obligation is discharged. Whole batches retain original application checks.
6. Full regional validation: check the first-own-session opening witness and each required term-first report sidecar in Vote, NEWVIEW/recovery histories, own-session receipt/Completion, later DATA and required DRAIN ancestry. Recovery returning to A reenforces selection; recovery retaining a descendant retains the earlier first-DATA witness. No validator compares an OC with its private queue or attached OC.
7. Preserve original signed DATA/PROPOSE/NEWVIEW/vote/QC/FINAL bytes, rank, lock, proposal identity and timers. Sidecar variants remain witness variants. Original evidence-first processing must inspect signed objects even if input wrappers fail.
8. Honest-FINAL/public-kernel argument: trace an actual correct FINAL signature to its timer-origin Vote and genuine augmented-valid ancestry. That provenance supplies the selected witness; a publisher annotation is insufficient. Public input remains complete original R0, not a reduced kernel. Regional W_R alone contains OC/advertisement/REPORT sidecars and is erased from public publication.

## Retirement and ownership hooks

- Fresh INPUT_ADVERTISE and INPUT_REPORT use the old current/unfrozen/retirement signing gate. OPENING_OFFER/HYDRATED/STATE_CANDIDATE are node-level preparation, not BFT actions or entry permission; closing one beta worker cannot destroy still-required fixed-K preparation/service.
- Transfer RawJournal/canonical/pending state, SelectionJob cut/cursor/native read context, SelectionDone, signed offer, all complete OC variants required by evidence, session attachment, and every promised send/serve/relay obligation to a durable owner before worker destruction. Existing public-import journals and source bytes retain their unchanged owners.
- Retain W_R with STATUS/NEWVIEW histories, recovery receipts, Completion and timer-origin evidence. Public erasure or G advancement does not authorize deletion of the last required witness copy.
- Same-K beta retries preserve state, job, offer and OC; fresh RC advertisements/reports do not rescan. Changed A/AUTH/native domain makes a new K. A late callback cannot overwrite the entered attachment or entry time. Physical node exit remains governed by the separate persistent-request owner contract.

## Timing and actual resource scope

Ordinary first DATA: QC/entry by e+2Delta, report at leader by e+3Delta, first vote everywhere by e+4Delta. Initial/recovery zero: NEWVIEW by e+3Delta, independent ANCHOR votes and reports by e+4Delta, both inputs at leader by e+5Delta, first vote by e+6Delta. No post-Enter private-pool scan is in either argument.

Ready contains at most n offer signatures/keys and one selected body. Opening NEWVIEW contains one OC and one advertisement, REPORT contains only RC/mode/digest/key/signature, and proposal contains one OC, one advertisement, and at most n report signatures with the selected body already in the batch. The Ready fragment gives a concrete size formula and names the old packet/load inequalities. These are original complete-handler/native-validation scope obligations, not a new availability or latency premise. No codec benchmark or unconditional deployment timing claim is made.

## Checks and remaining joins

Manual source comparison against the reviewed adapter checked finite post-hydration cuts, STATE_CANDIDATE ordering, every-certificate coverage, nonexclusive OC adoption, signed initializer binding, opening/descendant recovery, independent ANCHOR processing, original public R0 and the session-first finite potential. Old `E_v(P)` live-minimum and accepted-report-awareness claims were removed from the new request/import files. Static TeX brace/environment checks pass for all four authored fragments. Integration compilation and rendered-page checks are owned by the parent.

The only external joins are the named ownership coverage/formation lemmas, the main/protocol/retirement hooks above, and the already explicit original complete-message/native/resource scope. This author added no new communication, external availability or latency assumption. The user-confirmed same-physical-region old/new correct AUTH communication scope is used without rewriting frozen Model prose.

## Content hashes

- `sections/rotation_v3_10_requests.tex`: `c54b6baed157d833bbc0696742bcf478135412a7be4818d9b0e414de3cb572e8`
- `sections/rotation_v3_10_public_requests.tex`: `1be92deba4065fe976b82238f5d0ea827eb0abe66bd0c2703bfd983f96a711da`
- `sections/rotation_v3_10_input_selection.tex`: `484249a49dbc293140d61bbf518f3df6594cbff37d3ec7cd7b7bc7cb1d1ddce8`
- `sections/rotation_v3_10_ready_bundle.tex`: `1f8265b0d5d5f15678ea1ab771788105d825e04046c274ab8249b541cba1ad05`

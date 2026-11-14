# Ownership fragment integration

Authored only `sections/rotation_v3_10_ownership.tex` and this note. The frozen Model and earlier versions were not edited.

The user's clarified original within-region communication domain covers old and new correct AUTH members of the same physical region. The fragment uses `d_i(r)=max_{0<=k<=r} Delta_i(k)` and does not introduce a separate transport service. It preserves the original complete-message, source-authentication, native-preparation and load boundaries.

## Hooks

- Include inside Requests, after base request definitions/public import and before input selection. The fragment uses `subsubsection`/`paragraph` only.
- Section labels: `sec:ownership`, `sec:request-ownership`.
- Proposition labels: `prop:ownership-formation`, `prop:ownership-hydration`, `prop:ownership-finite-exceptions`, `prop:ownership-coverage`.
- External labels: `ass:network` and `sec:input-selection`. No new macros are needed.
- The first authenticated request-receipt hook persists Pending/ReceiptJournal and schedules individual relays. Public import and newly canonicalized selection records must call it. Input author confirmed this hook.
- RawJournal appends an exact retained raw handle at incremental fragment completion. Canonicalization and exact-parent admission happen during actual finite selection, not inside the last-chunk/Ready/report handler.
- Each correct opening-offer signer first verifies HYDRATED, then freezes its selection cut. Ready transmits OC only; it does not add another HYDRATED signature set or historical directory capsule. Input author confirmed this interface.
- Manager/retirement guards retain Pending, journals, cuts, contexts, fragments, share/certificate collectors and unfinished sends. Root owns these main/retirement hooks.

## Proof interfaces

Actual admitted slot lineage establishes the finite AUTH prefix `0..r`: genesis zero, regular/positive-output Force advance by one, zero-output retry reuses the slot. Directory construction retrieves those known keys; it requires no all-authority enumeration or missing-slot oracle.

`release(r)=tau_0+r*tau_p`, with positive `tau_p`, gates the actual first query for `(i,r,v)`. It neither changes public timestamps nor resets any Ready/engine/window timer. All ownership state excludes beta.

`prop:ownership-coverage` quantifies **every** valid HYDRATED for a nonexceptional target. Each has a correct signer whose completed all-member dissemination places x's raw/canonical handle at every correct target strictly before certificate formation. For owner `p in V_i(k)` and receipt time `t_x`, the excluded slots are `E_x union {0,...,k-1}`, where `E_x={r>=0:tau_0+r*tau_p<=t_x}`. Finitely many such slots supply finitely many accepted contributions. There is no honest-intersection requirement between hydration, offer and REPORT signers.

Physical exit cannot discard the original correct owner of a pending request. A same-logical-owner machine handoff must already preserve full state, authenticated route and continuing service within the same physical region. Worker retirement does not certify physical departure or bounded storage.

## Validation

An isolated wrapper using the current manuscript preamble and dummy targets for the two external references compiled successfully with `latexmk -pdf -interaction=nonstopmode -halt-on-error`. Its final log had no warnings, unresolved references, or overfull/underfull boxes. This checks the fragment's TeX integration surface, not the complete main document or protocol correctness. No model checker or executable protocol test was run.

Fragment SHA-256: `5952df7ab66e12fdbbdbb48f8b6c3c4dd3beeca03ac6d11db9e79ecd97319f1c`.

# v3.10 progress integration notes

This authoring pass writes only `sections/rotation_v3_10_liveness.tex` and this note. The parent integrator owns the main document; the ownership and input authors own their fragments. No frozen Model text, existing manuscript, old review, or authority/health mechanism was edited by this pass.

## Proof chain and integration hooks

1. `prop:v35-regional-response` retains the finite fair-initializer prefix and response constants. Its finite preparation maximum now contains the historical AUTH prefix, ownership release/context/cuts/fragments/dissemination/share collection, exact executable opening state, and opening certificate. The historical ownership authorities are known to exist through the installed slot: genesis starts at zero; all advances are consecutive; zero-output retries reuse the slot. Recovery AUTH fetching remains its original separate persistent task.
2. `prop:ownership-formation` provides finite hydration for a fixed target. `prop:opening-formation` provides finite exact-anchor selection without a Ready/offer cycle. Both keys omit beta, so unfinished jobs and completed certificates survive zero-output retries. New requests never extend the frozen finite jobs. No extra pool work is charged to the post-Enter regional response.
3. `prop:v35-public-response` still derives growth-or-Force from actual complete regional publication and retained public kernels. Prepared Ready attaches OC evidence; its correct offer already follows a complete HYDRATED check, so the hydration signature set is not retransmitted in Ready. REPORT validates supplied evidence or already validated first-session ancestry. The same original complete-handler abstraction carries those finite validation dependencies. There is no new native, pool, archive, ingress or future-authority deadline.
4. `prop:v35-global-progress` and `prop:v35-regional-service`, including their adjoining Owed discussion, are byte-identical to the corresponding v3.9 source blocks. The singleton argument, escalating beta tolerance, finite-control tail, regular unrestricted NextLive, Force Owed filter and strict opening-height increase are preserved.
5. New `prop:v310-finite-exceptions` uses the actual first-query gate `s_v >= tau_0 + r tau_p`, with `tau_p > 0`. The exceptional set includes equality and every index below the original owner's membership index k. Only authenticated target rosters contribute cuts; no nonexistent AUTH is presumed. Every nonexceptional correct query starts after the request receipt, and the original owner retains the pending request. Completed all-target dissemination precedes every valid hydration certificate through its correct signer. No quorum-intersection argument is substituted for that delivery.
6. Monotone canonical r and fixed finite quota imply a finite number of accepted DATA, hence accepted session-first DATA, from the finite exceptional slots. Zero-output retries spend none; positive-output closure advances r; strict cutoff prevents an old pair contributing again after Force.
7. New `prop:v310-request-progress` matches the original ingress quantifier. Global DATA progress and the finite physical region cycle produce some region with infinitely many accepted contributions. That region continues serving, so the original ingress statement gives one actual receipt by a correct regional owner. No assumption that a permanently eligible region exists is needed for this existence step. The regional-service proposition additionally supplies repeated service for every particular region satisfying its own persistent-eligibility premises.
8. Finite quota yields infinitely many nonexceptional accepted session-first DATA. `prop:opening-coverage` gives a real selected request at most x in the finite-predecessor order. The common first-own-session predicate and original correct-FINAL provenance make it an actual fresh executed primary. The finite potential over accepted logical identities decreases once per counted first DATA; speculative executions do not spend it. This repeats the final implication exported as `prop:session-input-progress` to make the full ingress-to-inclusion chain locally inspectable.

The new local labels are `prop:v310-finite-exceptions` and `prop:v310-request-progress`. Historical labels `sec:v35-request-limit` and all existing v3.5 progress labels remain stable. Required external labels are `sec:ownership`, `prop:ownership-formation`, `prop:ownership-coverage`, `prop:opening-formation`, `prop:opening-coverage`, `prop:session-input-progress`, and the retained timing alias `prop:v38-report-timing`. The retained public-import proposition is supplementary and is not a first-carrier premise of this proof.

## Scope and checked edge cases

The user's clarification supplies the original within-physical-region communication interpretation across old and new correct AUTH members. The fragment does not edit the frozen Model to add a distinct environmental service. It keeps the original abstract complete-message handler and executable-state adoption boundaries; it does not create numerical codec, CPU, aggregate-load or deployment benchmark obligations.

The request theorem retains actual correct-owner receipt, single-DATA fit and continued admissibility at accepted opening states. It does not promise inclusion after a request becomes inadmissible, service to permanently OFF regions, or a bounded request latency. These are the original ingress/admission/service scope, rather than the former unproved all-future-roster-awareness premise.

Manual integration checks covered a receipt exactly at a first query, late assembly from an early share, future-member receipt at index k, unbounded changing rosters, finite versus missing target authorities, zero-output retry reuse, positive-output slot advance, quota-one DATA, DRAIN ancestry, recovery returning to A versus retaining first-DATA ancestry, Byzantine accepted proposers, and multiple ancestors accepted within one EXPORT. No wait was inserted into an already running window or timer. No historical-task scheduler or new MSIG policy was introduced.

The earlier focused reviews were read as scoped evidence about their exact candidate/adapter interfaces, not as whole-v3.10 certification. This pass manually checked the integration it authored. It did not independently certify the entire protocol, compile the integrated main, run an executable protocol test/model checker, or benchmark any implementation. Whole-document compilation and fresh independent review remain parent-integration work. The final ownership/input hooks were checked against this liveness fragment, including their OC-only Ready design. No additional local proof gap was identified in this authored chain, subject to the named ownership/input hooks being implemented as stated and the inherited serving/complete-handler premises.

## Source hashes

Paths are relative to `formalization_v2/`. The output hash is the authored liveness fragment; ownership/input fragments may still be under authoring and are not pinned by this table.

| File | SHA-256 |
| --- | --- |
| `sections/rotation_v3_9_liveness.tex` | `5b3376a83cee0f6e6133819b383ae4b97236bf5fb612a02bd8ae99cf73eeb4bd` |
| `sections/rotation_v3_10_liveness.tex` | `3c0c1a5943df36908c09ec34dac34dcbcae845fd39c3074ed3ac7cc830615ea1` |
| `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |
| `reviews/v3_8/v39_ownership_candidate_v3.md` | `a71b64b8bb845463987de8d79b92fed6d8ef61db7a60eec0719dfe1b77a67bea` |
| `reviews/v3_8/v39_fresh_ownership_v3_review.md` | `4008b823b71e316520cc3fa4a50d8e636d1db1f146599012adc633560302b2a1` |
| `reviews/v3_8/v39_hydration_ready_adapter.md` | `efbd9c31860f891c7aa1c298b4428644dd7710354b4e2df17b884afc7eb0d3c7` |
| `reviews/v3_8/v39_hydration_ready_adapter_focused_review.md` | `2e94352daad879677fab930374ad2d0710e8e9c792a1b341eacdce3d9538b8f1` |

Local structural checks: six proposition environments and six proof environments have matching ends; brace counts balance; the global-progress/repeated-region-service source block equals v3.9 byte for byte. These checks are not a LaTeX compilation or proof-verifier verdict.

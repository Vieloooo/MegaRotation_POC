# H3 publication/GFR implementation contract v1

2026-11-16. ID **H3-PUB-GFR-IMPL-1**. Implementation is authorized by commit `d55d39f73593676325bdc0fbb7ba3845ca93025f`, `poc/docs/milestones/M1-implementation-dispatch-20261116.md`, read in full. This contract supersedes only the old preparation-only restriction. Existing contracts, evidence, public encodings and native verification gates remain intact.

Working checkout `/home/jy/.codex/worktrees/1e2f/megaroation_poc`, branch `codex/h3-publication-gfr`; accepted M0 `35d382ac445cc8bc779f0d18fd23cb57e54680da`, own preparation replay baseline `a1bee707b92b6b550a1b53c914e667994db15c84`. Old07 branch/evidence is preserved. Cross-feature merges, shared root wiring and system launcher belong to the independent M1 merge owner.

## Authority and scope

Source/version/section mapping is retained in [sealed preparation §§1–3](h3-publication-gfr-healthy-preparation-v1.md). Re-read rules remain v10 02 §§3.2,3.10,4.1–4.5;03 §§3.5–3.10,4.1;04 §§3.1–3.5/PUB-1–4;09 §§3.2–3.4,4.1–4.3,4.7–4.8; v2 §§3–6,9.1,10.1,12.2–12.3. Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`, DESIGN §§2.1,3.1,5 and Commonware2026.10.0 Simplex/faults source stay pinned. Existing03/06/07/16 APIs at accepted M0 are authoritative. The regional-bft-conformance skill supplies source-first boundary cases; protocol-spec-writer remains unavailable.

M1 remains15 healthy real worker processes,3×5 committees/quorum4, fixed10-height windows, R0→R1→R2 with first eligible R2 result accepted into G; separate SessionEnd evidence and separate official same-byte publication. Force/M2, D/RESUME, recovery expansion and Web are outside this implementation. Native finality, complete validated bodies, public completeness, G admission, Manager committed obligations, applied actions and worker Ready/Enter are different events.

| Slice | Local freeze / work allowed | Remaining gate does not block unrelated slice work |
| --- | --- | --- |
| [A](h3-publication-gfr-slice-a-v1.md) | Shared07 public-completeness helper; Native-only bridge injection; bounded canonical publication/private storage;06 Publisher adapter | Real H1 verifier/context provider and genuine run evidence are required for native/M1 PASS, not for implementing these existing trait boundaries. |
| B | Durable public stop/late-offer/dispatch gating; full-record IDs/material references; controlled service custody and bounded transport | H1 owns actual root/aux and ports; H2 owns ManagerDelta/action intents. No copied healthy-session-api or root receipt impersonation. Annex freezes these adapters separately. |
| C | Minimal artifact input to existing16 runner; same-byte recording/checking, marker compatibility | Genuine M1 artifact required for official real-result PASS; synthetic input-adapter tests remain labeled. |

Each annex is a small independently reviewable freeze. Lead freezes its concrete interface after Verifier expectations, then single Builder implements it immediately. Unresolved local H1 methods do not block A/C or unrelated B logic. All implementation outcomes cite exact candidates and bounded evidence; no slice PASS equals end-to-end M1 PASS.

## Ownership and integration

Docer writes only this new contract and its assigned new slice annexes; old sealed preparation files are read-only. One Builder owns `poc/crates/publication-bridge/`, necessary extraction/wiring inside `poc/crates/viewer-gfr/`, and the formally transferred minimal16 paths `poc/scenarios/espresso-compatibility/{capture.py,run-local.sh,README.md}` plus `poc/verification/espresso-compatibility/check_capture.py`. Verifier owns independently assigned `poc/verification/publication-gfr-healthy/` material. Lead owns freeze/integration/reports/Git. No06, root types/manifests, H1/H2 implementation, Regional BFT or old evidence writes. Others share the checkout; preserve their work.

H1 validates execution source/state and issues its own ReadyToken; H3 never emits a worker Ready claim. Dispatch decision2 fixes the lifecycle boundary: after full source/structure precheck, H2 delta commits close/tombstones, Selected+Preparing, cursor and action intents together as `Committed(actions_pending)`; later `ActionsApplied` is separate. Missing successor execution material cannot delay known valid closure or permit Enter. Durable terminal/ClockUnsafe never rolls back on later error. Concrete H1/H2 receipt interfaces are consumed when delivered; H3 may use its own injected opaque evidence references without declaring them actual root receipts.

Tests run only for the reviewed slice, using existing07 data/target/artifact conventions, host heavy-task lock and2 build jobs. One build/heavy simulation at a time. Preserve known old07 vectors and profile labels; run affected regressions after extraction. Native3/4/domain/historical cases and healthy15-worker/official checks remain local gates until actual inputs arrive.

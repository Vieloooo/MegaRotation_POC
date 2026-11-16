# H3 Slice A interface/component handoff

2026-11-16. **Slice A component PASS; real H1 finality and healthy M1 remain OPEN.** This is the first production-code slice under `d55d39f73593676325bdc0fbb7ba3845ca93025f`, not an end-to-end milestone result.

| Revision | Meaning |
| --- | --- |
| `35d382ac445cc8bc779f0d18fd23cb57e54680da` | Accepted M0 foundation |
| `c4a944a754e6f9cfa50f57a2399b707a2555ce94` | Preserved preparation on old `codex/poc-viewer-gfr` |
| `a1bee707b92b6b550a1b53c914e667994db15c84` | Only own preparation replayed onto M0 in `codex/h3-publication-gfr` |
| `348eb6db613e171336b16b3e3a779513f63fa9a3` | Slice A contract and independent expectations |
| `5a33f6714b093c7d75b755b3dcde73b64b517cac` | Pre-code size arithmetic correction and independent wire literals |
| **`e59d026e8c08b64ff744b57c63cc7756f0b34019`** | **Consumable Slice A source, independent review and Tester SHA** |

Worktree `/home/jy/.codex/worktrees/1e2f/megaroation_poc`, implementation branch `codex/h3-publication-gfr`. New `poc-publication-bridge` is an independent workspace with its own lockfile. Existing `viewer-gfr` reuses the same internal completeness evaluator; GFR-PUBLIC-1/POS-1/LIFE-1 bytes and accepted replay output are unchanged. No 06, root types/manifests or native runtime edits were made. Full contract: [H3-A-1](contracts/h3-publication-gfr-slice-a-v1.md).

## Consumer operations

`poc_viewer_gfr::public_completeness` takes a sealed 06 `VerifiedPrefix`, explicit `through_q`, configuration/profile, descriptor/target and verifier binding. It returns complete evidence with original locations/earliest q, public incompleteness, verification WAIT or rejection. It does not require NEXT or enforce G acceptance timing. The fresh wrapper never trusts a caller's mutable index or cached verdict. Existing GFR uses the same core with its own sealed same-profile cache; an empty later q does not demand new verification of already trusted history.

`poc_publication_bridge` exports `BridgeResult`, `BridgeStateCodec`, `publication_messages`, `ContextVerifier`, `BridgeInputs` and `NativePublicationVerifier`. Construct the Native adapter from immutable config/profile/GFR parameters/source ID, then call existing `poc_publisher::Publisher::create(directory, store_binding, adapter, BridgeStateCodec)`. Supply `BridgeInputs` to `offer` and `confirm_recorded`. The historical context capability validates Plan/config/anchor provenance and returns an ExecutionAnchor; H3 checks identity and actual 03 metadata/Merge/DEP. This is not an H1 worker ReadyToken.

The private H3RS v1 storage envelope retains exact 03 canonical fields/proof bytes and can roundtrip incomplete proof/chain material without treating it as a complete Offer. Public messages use existing GFR encoders, ordered bodies then final proof. Native constructor rejects fixture profiles; no production fallback/default verifier is supplied. A valid complete unselected candidate can be RECORDED without G acceptance. Receipt proof position and actual supporting prefix remain separate from earliest completeness. Invalid public proof variants yield NotYetPublic until a valid variant arrives.

Bounded profile: at most four valid transfers/body and ten physical bodies, proof 32,768 bytes, descriptor 16,384 bytes, private result 128 KiB. Maximum admitted public attempt is 51,560 encoded bytes. The pre-code four-byte arithmetic undercount is retained in the independent correction record; no wire or source limit changed. Actual encoders and 06 validators enforce the bounds.

## Evidence

- Independent review: **14/14** new cases, **167/167** existing codec/capacity cases and **117/117** existing rule cases, all exit 0; no candidate finding or harness source fix. [Review](../verification/publication-gfr-healthy/slice-a/e59d026e8c08b64ff744b57c63cc7756f0b34019/review.md), [manifest](../verification/publication-gfr-healthy/slice-a/e59d026e8c08b64ff744b57c63cc7756f0b34019/result.json).
- Tester on that same SHA: **14/14** cases and G1/G2 seed **42**, wall pauses **0/2 ms** (four runs, two pairs). Every canonical stdout matched the accepted 07 seed-42 trace byte-for-byte, including the raw source row; no timeout/incomplete case. [Tester report](../verification/publication-gfr-healthy/slice-a/tester/report.md), [manifest](../verification/publication-gfr-healthy/slice-a/tester/manifest.json).
- Raw review archive: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/h3-a-e59d026-independent-v1`.
- Raw Tester archive: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/simulation/h3-a-e59d026-acceptance-v1`; raw manifest SHA-256 `4d32a56fc24e1217b97de73854dbc0f2c65bdc7f7d5ab63bf898d42a6551aa95`.

Commands, tool/source/lock/binary hashes and captured stdout/stderr are in those manifests. Rust/Cargo 1.95.0, `--locked --offline`, two build jobs and one host-wide flock were used. Wall build time is diagnostic, not public time. Synthetic exact-subject callbacks test dependency injection and state rules; even tests using the Native profile tag do not provide actual H1 certificates or runtime receipts.

Docer/Verifier used named `bft_spec`/`bft_verifier` Astra/xhigh; one Builder and Tester used `bft_builder`/`bft_simulator` GPT-6.1 Sol/high. Main configuration remains Astra/ultra without independent runtime attestation. Source preceded independent review/testing; no cross-feature merge or push occurred. B/C follow immediately under their own freezes; actual H1 checker/context, controlled root/aux/channel wiring, H2 and real M1/official artifact evidence remain local integration gates.

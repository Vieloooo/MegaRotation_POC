# Feature 03 independent acceptance preparation

2026-11-15. **Preparation: PASS. Feature/implementation: INCONCLUSIVE, NOT RUN. `waiting-dependency=01`.** This directory is owned by `bft_verifier` (named role configuration: GPT-6 Astra / xhigh). No Builder or Tester was started. The lead saves the final preparation SHA separately; a document cannot contain its own enclosing commit SHA.

Worktree: `/home/jy/.codex/worktrees/1c43/megaroation_poc`; branch: `codex/poc-mock-plan-execution`; source baseline: `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`. The required independent derivation preceded review of the new contract and any candidate implementation. No `poc/` production modules were read or imported to create expected results. Source fingerprints, pinned Regional BFT HEAD and Python version are in [source-manifest.json](source-manifest.json).

| Material | Purpose |
| --- | --- |
| [source-expectations.md](source-expectations.md) | Source/trigger/pre-state/guard/output mapping; deferred actual implementation cases. |
| [semantic-vectors.json](semantic-vectors.json) | Fixed schedule, mapping, origin, window, execution, replay/lifecycle, L1 reader inputs and manual outcomes; registry/certificate obligations. |
| [proposal-codec-v0.md](proposal-codec-v0.md) | `MPX-PREP-LE64-SHA256-v0`, explicitly provisional bytes/arithmetic illustration. |
| [golden-vectors.json](golden-vectors.json) | 29 fixed byte strings and SHA-256 digests; no generated expectations at check time. |
| [check_vectors.py](check_vectors.py) | Small standalone Python oracle; uses only standard library, no Rust/build/production imports, no update mode. |
| [prepared-results.json](prepared-results.json) | Raw stdout: 172 passing reference assertions, not 172 production scenarios. |
| [preparation-review.md](preparation-review.md) | Independent review of the preparation contract and coverage limitations. |

Run from the named worktree with Python 3.12.8 (no random generator; `seed=null`, exhaustive only over the explicitly listed finite cases):

```sh
python3 /home/jy/.codex/worktrees/1c43/megaroation_poc/poc/verification/mock-plan-execution/check_vectors.py
```

The exact evidence-producing command used was:

```sh
python3 poc/verification/mock-plan-execution/check_vectors.py > poc/verification/mock-plan-execution/prepared-results.json
```

Exit code was 0. The oracle verifies known-answer SHA-256, fixed arithmetic/ordering outcomes, manual state/Kept values, literal bytes/digests and input-binding mutations. It does not call a candidate binary. `S` assertions check source-defined math/order in this reference oracle; `P` assertions depend on the unapproved proposal profile. Error names in the L1 oracle are descriptive labels, not approved enum values.

The key execution case starts with anchor h0 A=(nonce0,balance100), B=(0,0), C=(0,7). Padding h1 credits A+5; padding h2 credits B+2. At physical h3, Begin credits A+10; seven raw ordinary transactions leave Kept indices `[1,3,4,6]` and final accounts A=(2,90), B=(2,15), C=(0,19). The fixed total rises from107 to124 through credits17. The h3 physical root is h2 post-root; its physical parent remains the anchor identity, while its native parent is native h2.

| P-profile value | SHA-256 |
| --- | --- |
| h3 physical pre-root / h2 post-root | `715449d8f2b26210b5f8af39eb5e04c62600991ef1c6582ac12a2aacfd7d4d2f` |
| h3 post-root | `e25abd3da5f34da3f63f67e9a1fffa51ad588e27264110eb76d2de7115254f1b` |
| h3 physical hash | `c849f448b883b77fbb4837dfeefaa5f0fbeb18e8e11f76f5d5141a3859627814` |
| h3 native hash | `e2f8c67f5910afc8deb3c6f0b68e647ae381142103e27f10bcefed2cabb8b15b` |
| semantic-vectors.json file | `b45de1c35727eb01a642d85378e8d4ae39d61274f08f7ad736e9a981ee485143` |
| golden-vectors.json file | `85924d2876282d1058b0340418ecafa4556639b403b32ebef50d2121095ecd3e` |
| check_vectors.py file | `bab434600ae2972df5aef4f24eb3a5c6c88a5711e102245d7802f4bf1a97b3e9` |
| prepared-results.json file | `48c47943b1c92362d6b37902aa16ba2332fb4bdc2ffe2f1102ccd5d6df762099` |

Removing the skipped invalid raw transactions changes the physical hash but preserves the actual Kept/post-state/native hash inputs. Changing any native-parent, metadata, DEP, Kept, or post-root byte is detected in the P example. These checks do not prove cryptographic collision resistance or a deployed codec.

L1 inputs cover heights0–3, finalization at `2h+1.5s`, A delivery at `2h+2s`, and B delay on h2/h3 to9/10s. At8s A can read h3 while B waits; B succeeds at10s. Future rows, transient gaps, missing control coverage, corrupt evidence, permanent manifest holes and range exhaustion have separate expectations. These timings are an independent bounded example, not a replacement for the planned REG-A profile or a real Anvil run.

Pending acceptance includes genuine 15-key registry and PoPs, actual BLS/native certificates and signer/subject rejection, the approved registry/Plan/Descriptor/physical/native codec, detailed arithmetic edge policy, finalized provenance validation, Rust implementation, runtime/crash/grant-Ack integration, Anvil/ABI, complete historical ResumeCut, and same-candidate verifier/tester matrices. Synthetic sort labels are not public-key material, and certificate obligations are not fabricated signatures. No safety or liveness claim is inferred from these finite reference checks.

The coordinator must provide the accepted 01 SHA and shared interface decisions before any implementation. Downstream consumption needs 02 route identities, 04 native proof/subject mapping, 07 replay/Lifecycle encoding and 09/12 reader/ABI/derived-origin decisions. The proposal may be adopted only by an explicit contract revision and refreshed independent expectations.

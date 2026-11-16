# Independent C review — bca02b0

**PASS for the bounded artifact-input/offline readback slice.** Candidate `bca02b095bac1e5701eb78e352d605c0ed887e10` passed the frozen52-case independent matrix without test-source corrections or production edits. No blocking candidate finding was identified in the four assigned paths. This does not close C07 genuine H1/M1 publication.

The expected semantics, raw namespace/transaction model, API bindings and matrix source were prepared before candidate inspection. Matrix source SHA-256 is `c04de4b13aa30b8922a035de050e30945849aa687fb2680ea42aff85cc7f13eb`. The current candidate and that committed source were checked before execution.

## Evidence and source-rule audit

| Source rule / code location | Checked behavior and replay evidence |
| --- | --- |
| Frozen C manifest bounds/canonical fields; `capture.py:46`/`:79`, independent checker`:75`/`:112`/`:131` | Separate loaders reject bool/float integers, duplicate JSON keys, nonfinite/unknown fields, wrong codec/version/hash/length, empty/oversized inputs and symlinks. Equality boundaries pass. File descriptors use bounded reads and no-follow traversal. Invalid inputs leave the empty run sentinel untouched before image/process access. |
| v10 publication identity and C immutable-copy contract; `capture.py:167` and checker`:356` | Loaded bytes survive mutation of the original file. Exact manifest/payload/report copies, original producer revision and runner revision remain distinct. Rehashed copy/manifest/report corruption is rejected, including automatic copy validation without an external-manifest flag. |
| Complete public receipt differs from submit ACK; `capture.py:577`, checker`:346`/`:395` | Ordered inputs and repeat-first behavior preserve all ordinals; eleven identical inputs may map to one actual source occurrence. Raw POST bytes/namespace, ACK and complete raw positional source agree. Ordinal/submit swaps, ACK-only input, missing flat position and requery gaps cannot produce a complete publication claim. Requery derives its own prefix against the original first-reader submit transcript. |
| Existing16 audit mapping and timing boundary; checker`:182`/`:279`/`:448`, `capture.py:829` | GFR input codec stays0x47465201 while legacy opaque audit codec stays0xEC160001. Native-tag metadata remains NOT_PERFORMED/byte-publication-only; proposed/finalized/live stay Unknown and PE export Unsupported. No Python hash comparison becomes a native certificate or QC/VID check. |
| Frozen launcher reuse/private namespace; `run-local.sh:16`/`:63` | Mock-only argv test confirms a host/tmp input directory is read-only bound at fixed `/tmp/ec16-publication-input` after the private `/tmp` mount. A spaced/metacharacter filename remains a literal argument; host/tmp is not mounted over guest/tmp. Source diff preserves the existing launcher/process-management path. This test launches no actual namespace, node or Anvil. |

All52 cases passed with exit0. The matrix directly generated raw evidence under `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/h3-c-bca02b-independent-v1`; `command.json`, runner JSONL/stderr, per-case fixtures/commands/reports and summary are preserved there. No Rust build or real Bubblewrap execution was needed.

## Saved official-marker compatibility

The candidate checker also rechecked the previously recorded official run `ec16-official-6248072-run02`, using that recording's actual runner SHA `6248072b3fd0eee1ae468f0961a4d3589291cba6` as provenance. Both first and requery readers passed against their saved audit bytes and JSON projections, covering6 complete blocks each. Both retained public audit SHA-256 `43f106c905bb0877c2131bd7926eb099701d4e3936c0845e96ec7fb36a4e7cf5` and projection SHA-256 `c0b466043f122fa83170e32327d4f89c4e8b1b05986c3e22281899cbf3147b6d`.

The old capture manifest remains byte-identical at SHA-256 `08bdc1aec6b5735df90ab3ceeab7a6fe13868fd0dc8212c78158ddbf2fe61dfb`. This is read-only compatibility with saved official marker evidence, not a new official launch or a genuine M1 artifact publication. New commands/results are preserved under the task07 run's `legacy-official/` directory; old16 evidence is untouched.

## Scope and limits

The52-case matrix uses synthetic HTTP responses, source blocks q0–q1, fixed inputs and no randomized seed. It includes1/11-message profiles and exact65536-byte boundaries. The wrapper test mocks Cargo/image/timeout/lock tools and checks argv construction only. Native-report fixtures are explicitly synthetic metadata; no final proof is verified. First/requery source decoding and byte comparison are real candidate operations against those fixtures.

C07 actual H1 artifact verification, new official-node publication, real namespace launch/isolation, HotShot/QC/VID validation, official timing bounds and healthy15-worker M1 remain **NOT_RUN**. The saved official marker compatibility check does not change those limits. Source B contract edits during this work do not alter the C candidate's four production paths or harness source.

The Tester may proceed on the same exact candidate SHA. Any source fix or interface change requires affected independent re-verification first.

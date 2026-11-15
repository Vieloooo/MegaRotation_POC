# Independent ANVIL-L1-ABI-v1 acceptance preparation

These files were derived from protocol sources and the feature09 contract before reading Builder implementation. They are interface expectations, not a declaration of Solidity, Anvil, controlled runtime, GFR or Derivation acceptance.

- `source-expectations.md`: original source-first transitions and boundaries. Its pending decisions are subsequently resolved by the contract and `contract-review.md`.
- `contract-review.md`: conditional interface approval and the required static-policy correction.
- `semantic-vectors.json`: literal transition/boundary expectations; no production public objects or ResumeCut substitutes.
- `independent_codec.py`: small reference ABI/EIP-712/Keccak aid; **not production cryptography**. Actual secp256k1 signing/recovery uses installed `coincurve`/libsecp256k1. No candidate code imports.
- `golden_vectors.py`, `golden-vectors.json`: exact type strings, preimages, digests, signatures, recovered public addresses, calldata/selectors/event topics, 57 changed-field/domain and 40 malformed-signature cases.
- `preparation-results.json`: observed independent codec/vector checks and tool hashes. EVM/driver/component acceptance remains INCONCLUSIVE.

From the worktree root:

```sh
python3 poc/verification/anvil-l1/independent_codec.py --self-test --cast /mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0/cast
python3 poc/verification/anvil-l1/golden_vectors.py check --cast /mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0/cast
```

Python 3.12.8 and coincurve 20.0.0 were used. Cast is the previously installed/pinned 1.0.0-stable (`e144b82070619b6e10485c38734b4d4d45aebe04`). The check command re-derives and verifies all public vectors without private keys; Cast is optional for local recomputation and required to reproduce the recorded cross-check count. Private test-only keys remain in an external private directory and are not needed to consume these artifacts. Regeneration with fresh keys produces different public signatures/addresses and DA-certificate-dependent batch hashes; do not replace frozen vectors casually.

`sourceKind=GLOBAL` and gpos `0x01020304` are explicit opaque-byte binding fixtures. They do not claim to encode a valid canonical ReplayPosition. Consumer03/07 supplies and validates actual bytes. The source positions in semantic vectors are literal expected triples, not a second canonical Rust type. Same-transaction Set/Resume fixtures do not construct a derived cut.

Submission, inclusion, successful EVM execution, simulated finalization and reader delivery must all be evidenced separately later. Only `CursorAdvanced` from a configured Inbox successful finalized receipt is ordinary batch acceptance; `CursorSet` is an override control input. Accepted DA that is missing or mismatched blocks downstream derivation. Full09 still requires real02/03 wiring; 11/12/13 outcomes are separate gates.

`receipt_checker.py` 与 `candidate-review-plan.md` 在读Builder源码前独立准备；它们可从真实原始RPC bundle重查签名、完整收据、精确位置配对和历史cursor。运行 `python3 poc/verification/anvil-l1/receipt_checker.py --bundle /ABSOLUTE/raw-rpc-bundle.json`。这不代替driver时间/reader/command路径审查。

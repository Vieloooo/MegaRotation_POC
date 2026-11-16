# Feature 11 interface release M1

2026-11-16. **BS-FRAME-v1 and BS-CRYPTO-v1: independently reviewed/tested PASS.** The concrete accepted-reader interface passed static review; its actual Anvil execution and sample delivery remain M2 work. This is an interface/component milestone, not complete feature 11 acceptance.

| Revision | SHA |
| --- | --- |
| Clean worktree start | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` |
| Integrated 03/09 baseline | `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac` |
| Frozen r1 contract and source-first independent vectors | `8fb46c296808dbe97c95df7a2761678412b95f62` |
| Consumable interface, reviewed and tested source | `75b574d40930b7c9c748a4c6b6e381117c937537` |

The source candidate includes normal merges of 03 `e9c15e04bc83b335668f3ecd837367aa87344a89` and 09 `c443ae925ed549b046731c2d8ef3dd9ce0311d24`. Worktree: `/home/jy/.codex/worktrees/f025/megaroation_poc`, branch `codex/poc-batch-services`. Main Astra/ultra was verified from actual turn context; named Docer/Verifier roles use Astra/xhigh and Builder uses Sol/high. The formal component Tester runs after M2 review.

## Consumer interface

Use `batch-services = { path = "../batch-services" }` in the consumer's independent workspace. [Crate README](../crates/batch-services/README.md) documents the exact APIs; the [frozen contract](contracts/task-contract-batch-services-v1.md) and [golden vectors](../verification/batch-services/frame-golden-vectors.json) define the one P0 format.

- `frame::{encode_closed, decode_closed, commitment, verify_payload}` operates on one complete frame containing one or two contiguous BlockRecords. Ordinary transactions use 03's existing codec. `verify_payload` returns records only after commitment and format checks.
- `crypto::{InboxDomain, InboxSigners, check_da, verify_batch, validate_submission, submit_batch_calldata}` reuses 09 wire types, complete EIP-712 domains and real secp256k1 recovery. Authentication, P0 structural checks and actual L1 acceptance remain distinct.
- `accepted_view::InboxGateway` owns the concrete 09 `Driver<AnvilRpc>`. Its named reader must receive the finalized prefix before an original successful `CursorAdvanced` produces a closed `AcceptedRecord`. Positions retain transaction-local and RPC-global log indices. `load_payload` verifies signatures, bytes, range and terminal origin; missing/corrupt DA waits at that accepted input.

The gateway starts at a fresh private Anvil genesis and receives one ordered command stream. Consumers of the same running driver use its named readers; they must not independently attach a fresh driver to an already advanced chain. Serialized records are audit evidence, not an unchecked trusted constructor. Gpos remains opaque and does not establish GFR replay validity.

## Evidence

The [independent review](../verification/batch-services/reports/m1-review-75b574d.md) records 70 positive frames, 19 malformed frames, 9,011 truncated prefixes, the 47,175-byte maximum, 22 signature field mutations, 13 malformed-signature checks, independent recovery of two production signatures, and exact comparison of 1,060 calldata bytes. Random-frame seed is `1112753713`; fixed cases use literal inputs. Builder's locked offline check and three vector test functions also passed. These counts are bounded implementation evidence.

Raw independent evidence: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m1-verifier-75b574d-run2/`. Consumer output SHA-256: `c8b63a6427ec5fe0e8fb1660712426582e75f67bc7889a22ee28fc5d27ad9217`. The [manifest](../verification/batch-services/reports/m1-manifest-75b574d.json) binds raw artifacts and all nine candidate source files; lead independently checked every listed hash. Exact commands, Rust 1.95.0, two build jobs and global flock usage are recorded in the review.

Actual Anvil receipt/DA samples are **not yet delivered at M1**. M2 continues immutable DA, separate Batcher/Certifier execution, persistent tasks/outboxes, process-crash recovery, real receipt acceptance and two wall-pause runs. Full 07/08 G/gpos, G′, 12/13 D/ResumeCut and 02 controlled integration remain open.

The updated dispatch (`0c15aac`, handoff revision `32a3eb6b31ff8c1dfe2d05dbc2ed26c0c031bb77`) was read from the source checkout without merging unrelated changes. BFT worker restart recovery is not a blanket prerequisite for this component or persistent-worker handoff profiles. Service delays and actual service process crashes remain separately tested; neither substitutes for BFT recovery, and accepted missing DA remains WAIT.

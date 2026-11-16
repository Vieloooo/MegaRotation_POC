# M0 foundation merge record

2026-11-16 (Asia/Shanghai). This records the completed integration operation; execution results and promotion are separate gates.

The managed worktree is `/home/jy/.codex/worktrees/440b/megaroation_poc`, on host `server` (`xuelab_jump`), branch `codex/poc-m1-foundation`. It shares `/mnt/reth/jy/megaroation_poc/.git`. Intake was clean and detached at `4a9a6b0a92a11e5778833988bb6524d929f4608e`; the new branch was explicitly created from verified old main `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`.

The Docer contract and independently prepared expectations were frozen before merging, in `865c90a8ab735d0ae3682525613c4c9274bc5161`. The subsequent precise documentation update `d5cbecb24836af80f8cd5414052043a17ca2ae17` was approved before freeze: relative to primary plan `4a9a6b0`, it changes only `poc/docs/commander/state.json`, recording safe checkpoints and H1/H2/H3 preparation. No floating source HEAD was used as a component input.

| Order | Input | Normal merge commit |
| --- | --- | --- |
| 03 | `e9c15e04bc83b335668f3ecd837367aa87344a89` | `affada3013eaa2a3f7a36bf1fd479549db3f4e71` |
| 02 | `e1a435712ec492cae2bbff120a51ffd507eebf45` | `7cecaa0d2509cda89116a33d559504995493f2fe` |
| 06 | `4b034dfbb6c3e1c7143208347b546daab8dfe298` | `5e6e9afd0cab4117c1133157b891a778b9966284` |
| 07 | `788291735ce6db3c3eb78b4594a28d730cce71cd` | `2c599c65b89006358344dc9bd20d058eaf80c901` |
| 16 | `5c34b400ba5245d7775a281566e21bf59708b147` | `a3f3c6fc6ab02da2e529dfc9ea9a1369874ab963` |
| 09 | `c443ae925ed549b046731c2d8ef3dd9ce0311d24` | `7e8e1d419c54a9d59c069e4cb939a7485dac6589` |
| Documentation | `d5cbecb24836af80f8cd5414052043a17ca2ae17` | `a0073f336357aa840467cf623ba98334416787ee` |

Each operation used `git merge --no-ff --no-edit INPUT` (with `--no-stat` after the first merge). All used Git's normal `ort` merge and completed without conflicts. 07's existing 03/06 ancestry and 16's 06 ancestry were preserved; no cherry-picks were used. No component implementation, manifest, lockfile, module declaration or original harness needed conflict edits. M0 adds only its contract, audit/report artifacts and a parameterized validation runner. Standalone component workspaces remain standalone.

The named component integration SHA is `a0073f336357aa840467cf623ba98334416787ee`. The BFT gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; its existing initialized source copy was read only, and this worktree's submodule was not initialized or changed. Native prototype/vendor/proto and worker/runtime/hub paths remain identical to old main. The prototype differs from historical Fresh `fec7ade1675cb60dd16a16fb28ebe81e3f76f508`, so that matrix's PASS is not transferred.

Neither 01's checkpoint `0183b401f805e5508ae5fdb20d53457eb4149504` / recovery WIP `f7bda20b`, 11's `aab8a5180801ed7f97f7e437d147c97f972b4dd8`, nor 12's `1f6d1b3b4076ff16908dc59f272a06686b1d1ea5` was merged. Existing baseline recovery files remain intact. No source commander registration, other worktree, remote ref or service configuration was edited.

The task data root is `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m0-foundation`; worktree `.build` points there. Cargo home and target are task-local; only `cargo/registry` links the existing public registry cache. No credentials were copied. Rust is 1.95.0 and Commonware is 2026.10.0. All builds/heavy runs use a single `/tmp/megaroation-poc-heavy.lock` acquisition and `CARGO_BUILD_JOBS=2`. The runner never wraps an already locked legacy runner. All raw run attempts use fresh private output directories.

Actual own-turn metadata is retained in [model evidence](../../../verification/m0-foundation/model-evidence.json): lead Astra/ultra; Docer and Verifier Astra/xhigh; Builder and Tester GPT-6.1 Sol/high. Full-history child logs also contain inherited parent contexts; these were not mistaken for each child's own configuration. One Builder owned the only new executable file; independent review prepared expectations before reading the integrated implementation.

For acceptance use the [frozen contract](M0-task-contract-v1.md), [independent expectations](../../../verification/m0-foundation/expectations-v1.md), source identity audit and the final M0 result. M0 supplies reusable components, not completed M1 regular or M2 Force. Main promotion requires the exact final tested SHA, unchanged old main and no worktree checking out main; no push or deployment is authorized.

# M1 健康满窗 Smoke 验收

2026-11-16 · **PASS** · `9154785732d803c1f2f69aee3b2af67220a12f58` · seed 42 · Rust 1.95.0 · Commonware 2026.10.0 · Regional BFT `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`。

15 个独立 OS worker 组成三个五人委员会，全部退出码为 0。R0 和 R1 分别在 h10、h20 的 `G` 接纳后触发接棒。后继窗口为 `[11,20]` 和 `[21,30]`，anchor 分别为 h10、h20。R2 的 h21 于虚拟时间 `31,085,000 µs` 被 GFR 接纳。[冻结 smoke 合同](../../contracts/m1-healthy-smoke-v1.md)

独立复核验证了 21 份原生 Final 和 21 个 `G` 条目，核对 4,571 对已持久化的 Grant/Ack 与 2,040 个 Ack 后释放的输出。离线复跑结果与封存的 Verifier 和 Tester 输出逐字节一致。85 个封存源码文件的 Git 对象均与候选匹配。完整 M1 的 SessionEnd、多候选、多 seed/墙钟暂停矩阵及官方 Espresso 字节读回仍按原合同跟踪。[完整 M1 合同](../M1-regular-merge-zh.md)

## 本机证据与复跑

证据目录位于 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m1-regular/artifacts/`。

| 文件 | 内容 |
| --- | --- |
| `healthy-smoke-91547857-seed42-run01/manifest.json` | SHA-256 `4e76e4eeb8e41e801db9689441b6fd82b94b7060b6ad2dbb47034e695e9f78ca`，绑定候选、构建、运行和原始文件。 |
| `healthy-smoke-91547857-seed42-run01/raw/trace.jsonl` | SHA-256 `359f4c6f366d69cc406c07fc9071faff01e26ddf9c0c6fa9c1946f412d2fe1b8`，16,085 行，h21 `GAccepted` 为 seq 16038。 |
| `independent-verification-91547857-seed42-run01/report.json` | 独立验签、公开执行和受控输出检查。相邻 `public-replay.json` 给出逐块复算结果。 |
| `healthy-smoke-91547857-seed42-run01-tester/tester.json` | 真实进程及退出检查、h10/h20/h21 的位置以及两次 handoff。 |

在固定候选 checkout 运行，指定一个新的输出目录。

```sh
python3 -B poc/scenarios/m1-healthy-regular/run.py smoke \
  --candidate 9154785732d803c1f2f69aee3b2af67220a12f58 \
  --output "$M1_NEW_OUTPUT"
```

已有运行的离线检查命令记录在 Verifier `report.json` 与 smoke runner 的 `invocation.json` 中。M2 和 regular 回放以合并后的 `main` 为代码基线，以此精确 smoke 候选及其原始证据为已有通过记录。[后续任务交接](../M2-and-regular-gfr-replay-handoff-zh.md)

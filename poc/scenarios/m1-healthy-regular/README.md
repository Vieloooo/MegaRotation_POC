# M1 healthy regular harness

本目录实现冻结合同 M1-HARNESS-1 的机械准备阶段。当前 `preflight` 和 `run` 对有效准备包返回退出码 3 与 `NOT_RUN`。离线 checker 可给出 `BundleIntegrity=PASS`，协议 gates 和总状态保持 `NOT_RUN`。 [冻结合同](../../docs/contracts/m1-healthy-regular-harness-v1.md)

## 运行健康满窗口 smoke

在干净的 `codex/poc-m1-regular` 候选上，从现有 M1 worktree 执行。固定 seed 42，使用 M1 数据盘 Cargo 缓存。每次指定新的输出目录。

```bash
/home/jy/miniconda3/bin/python3 -B poc/scenarios/m1-healthy-regular/run.py smoke \
  --candidate "$(git rev-parse HEAD)" \
  --output /mnt/reth/jy/megaroation_poc/.build/commander-jobs/m1-regular/artifacts/healthy-smoke-seed42-repro
```

`smoke` 构建锁定版本并运行 15 个独立 worker，记录源版本、二进制 hash、命令和原始事件。证据包入口为 `manifest.json`、`manifest.sha256`、`raw/trace.jsonl` 和 `raw/frames.bin`。运行退出码与独立检查结果分开记录。健康路径的成功终点是 h21 经 Publisher 和 SimEspresso 进入 G。 [健康 smoke 合同](../../docs/contracts/m1-healthy-smoke-v1.md)

## 准备输入并运行

在干净的命名候选 checkout 根目录执行。需要 Python 3.11+ 和 Git。默认版本探针使用 xuelab 上固定的 Rust 1.95.0 工具目录。可用 `REGIONAL_BFT_RUST_BIN` 指定另一个安装同版本的目录。正式准备数据位于合同指定的数据盘。本例使用 `/tmp`。

```bash
M1_PREP=$(mktemp -d /tmp/m1-prep.XXXXXX)
M1_CANDIDATE=$(git rev-parse HEAD)

python3 -B - "$M1_PREP" "$M1_CANDIDATE" <<'PY'
import hashlib
import json
from pathlib import Path
import sys

root = Path(sys.argv[1]) / "input"
root.mkdir()
candidate = sys.argv[2]


def copy_document(source):
    source = Path(source)
    data = source.read_bytes()
    destination = root / source.name
    destination.write_bytes(data)
    return make_ref(destination)


def make_ref(path):
    data = path.read_bytes()
    return {"path": path.relative_to(root).as_posix(), "sha256": hashlib.sha256(data).hexdigest(),
            "bytes": len(data)}


def write_document(name, value):
    path = root / name
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    return make_ref(path)


contract = copy_document("poc/docs/contracts/m1-healthy-regular-harness-v1.md")
source = copy_document("poc/verification/m1-healthy-regular/source-expectations-v1.md")
expectations = write_document("expected.json", {
    "schema": "M1-EXPECTATIONS-1",
    "contract": contract,
    "source_expectations": source,
    "oracle_binding": None,
    "required_gates": ["BundleIntegrity", "ProcessAndCustody", "RegionalFinalAndDelivery",
                       "PublicReplayAndLifecycle"],
})
owners = {
    "h1.runtime": "H1", "h1.session": "H1", "h2.manager": "H2",
    "h3.bridge": "H3", "h3.official_artifact": "H3", "oracle.protocol": "Verifier",
}
write_document("input.json", {
    "schema": "M1-INPUT-1",
    "contract": contract,
    "candidate": candidate,
    "m0": "35d382ac445cc8bc779f0d18fd23cb57e54680da",
    "dispatch": "d55d39f73593676325bdc0fbb7ba3845ca93025f",
    "components": {"h1": None, "h2": None, "h3": None},
    "scenario": "full-window",
    "seed": 7,
    "start_order": "forward",
    "wall_pause_ms": 0,
    "fixtures": dict.fromkeys(("config", "registry", "genesis", "dep", "workload", "l1", "schedule")),
    "profile": None,
    "ports": [{"port_id": port, "owner": owner, "binding": None} for port, owner in owners.items()],
    "expectations": expectations,
    "source_run": None,
    "limits": {"max_virtual_tick_us": None, "max_events": 1_000_000,
               "wall_timeout_s": 1800, "max_raw_bytes": 1_073_741_824},
})
PY

python3 -B poc/scenarios/m1-healthy-regular/run.py preflight \
  --candidate "$M1_CANDIDATE" --input "$M1_PREP/input/input.json" \
  --output "$M1_PREP/preflight"

python3 -B poc/scenarios/m1-healthy-regular/run.py run \
  --candidate "$M1_CANDIDATE" --input "$M1_PREP/input/input.json" \
  --output "$M1_PREP/run"

python3 -B poc/verification/m1-healthy-regular/check.py \
  --bundle "$M1_PREP/run" --expected "$M1_PREP/input/expected.json" \
  --output "$M1_PREP/check.json"

python3 -B poc/verification/m1-healthy-regular/check.py \
  --compare "$M1_PREP/preflight" "$M1_PREP/run" --expected "$M1_PREP/input/expected.json" \
  --output "$M1_PREP/compare.json"
```

上述四条 CLI 的预期退出码均为 3，适合在交互 shell 逐条执行。无效输入、版本不匹配或损坏证据返回 2。已存在的输出路径会被拒绝。H09 使用 `official-same-bytes`，需要追加 `OfficialSameBytes` gate。当前以 `source_run: null` 准备 H09 时会报告该输入缺失。

## 证据文件

- `manifest.json` 采用 `M1-RUN-1`，最终由 `manifest.sha256` 封存。版本拒绝保留已采集的 revision、argv 和 stdout/stderr。无法解析或绑定输入时保存 `rejection.json`。
- 输入和预期封装中的 FileRefs 相对各自 JSON 所在目录。原始输入封存在包的 `input/`，候选源码快照封存在 `source/`。`source_files` 为相对包根的 FileRef 数组。准备阶段的候选源码快照涵盖实际 harness/checker、冻结合同、决议和来源预期、候选 Cargo manifests/lockfiles 与 protobuf。候选原始 Git commit、根目录及 `poc/` 的 NUL 分隔 Git tree 清单和版本命令输出在 `commands/`。
- `revision_before` 和 `revision_after` 各包含 Git `head`、`branch`、`status`。组件映射键为 `h1/h2/h3`。工具记录包含绝对 `path`、`version` 和实际 binary `sha256`。`tool_versions` 也记录 gitlink 和 lockfile 中的 Commonware pin。决议经过 cherry-pick 时按冻结来源 commit 的文档字节核对。
- `artifacts` 使用扁平的 `role/producer/schema` 与 FileRef。命令输出分别索引为 `command_stdout` 和 `command_stderr`，包括全部固定只读 Git 探针及工具版本探针。`invocation.json` 保存 launcher 原始 argv。
- `M1-CHECK-1` 的 `checker` 记录实际 checker 与共享机械校验源码 hash，以及输入和输出包的读取根。`bundles` 依读取根顺序引用 manifest。报告写在包外。

离线 checker 从封存证据重算候选 commit ID、根 Git tree 及 `poc/` 子树 ID，再核验源码的 Git blob ID 和 BFT gitlink。`--bundle` 和 `--compare` 的候选源码核验需要包含这组对象证明的准备包。H09 的源报告和源 manifest 复用机械结构校验，直接引用的源 manifest、报告和原始消息/结果按 FileRef 核对。报告记录的绝对根仅作诊断。签名与交付、独立执行、公共重放、官方读回和规范 trace 比较依赖待接线 oracle。非空 binding 当前只校验文件完整性。已提供的 config/profile/virtual cut 也需后续生产适配器核对语义。准备阶段仅执行固定工具探针，worker 数为零。

## 机械拒绝测试

```bash
python3 -B poc/verification/m1-healthy-regular/mechanical_tests.py
```

测试使用 `/tmp` 的一次性 Git checkout，当前工作区存在未提交文件时也可执行。包含重复字段、布尔值与整数混用、路径逃逸、symlink、hash/size、候选过期与 dirty、现有输出保护、自报 PASS、非空 binding 与比较缺 oracle。真实 checkout 的 `preflight` 和 `run` 强制要求指定已提交的干净候选。

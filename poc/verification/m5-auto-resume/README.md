# M5 独立回放

在干净的受测提交上运行。Tester 清单提供受测完整 SHA、独立 Rust checker 路径及其 SHA-256。输出目录需要尚未创建。

```sh
/home/jy/miniconda3/bin/python3 -B poc/verification/m5-auto-resume/replay.py \
  --candidate "$candidate" --matrix "$matrix" \
  --public-checker "$public_checker" --output "$review"
(cd "$review" && sha256sum -c SHA256SUMS)
```

离线入口校验 Tester 清单、命令日志及各场景选入的清单，以实际 Anvil 收据、RPC 查询、DA 对象、原生 BLS 签名、受控 grant/Ack 和各观察者的原始日志计算判定。报告包含 M5 各场景、主场景 650 ms 暂停复跑、M1–M4 的独立回放和 Tester 定向向量计数。原始输入目录可只读挂载。

单独回放一条 M5 轨迹时可直接使用 `check.py --repository "$PWD" --candidate "$candidate" --run "$matrix/primary" --repeat "$matrix/primary-pause650" --public-checker "$public_checker" --output "$review"`。其他场景提供各自的 `--run` 目录。

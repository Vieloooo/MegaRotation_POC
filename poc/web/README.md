# 实验回放入口

在浏览器打开 `index.html`，即可查看 M1–M5 的封存轨迹。五个回放页面均可独立离线打开。时间轴上的实心点标记新增或替换区块，M4 和 M5 还可按高度对照 G、G′、D，并查看区域 Final 的父子关系。[回放入口](index.html) · [回放证据索引](m1-gfr-replay/README.md)

在 Mac 终端运行（需要已有 `xuelab_direct` SSH 配置和服务器上的 Python 3；直连可用性须由 Mac 实际确认）：

```sh
ssh -tt -o ExitOnForwardFailure=yes -L 127.0.0.1:18762:127.0.0.1:18762 xuelab_direct \
  'cd /mnt/reth/jy/megaroation_poc/poc/web && python3 -m http.server 18762 --bind 127.0.0.1'
```

保持命令运行，在 Mac 浏览器打开 <http://127.0.0.1:18762/>；结束时按 Ctrl+C。若端口被占用，可更换本地和服务器的端口，并相应修改 URL。服务只监听服务器 loopback，隧道只对当前 SSH 会话有效。

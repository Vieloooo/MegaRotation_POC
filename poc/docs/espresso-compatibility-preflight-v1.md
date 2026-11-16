# Espresso compatibility preflight — 2026-11-16

This is setup evidence, not candidate acceptance. Worktree `/home/jy/.codex/worktrees/d2be/megaroation_poc`, branch `codex/poc-espresso-compatibility`. Clean initial baseline `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`; normal no-ff merge of only authorized dependency `4b034dfbb6c3e1c7143208347b546daab8dfe298` produced `bcd30b0879e6c17afc177bfea76f7fee170cc069`. No reset, push or main integration. Regional BFT gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; Commonware pin2026.10.0.

v2 design SHA256 `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`. Source dispatch and dependency-plan read from the source checkout, which remains read-only except shared Git metadata and the explicitly authorized task build directory.

Project `.codex/config.toml` requests gpt-6-astra/ultra. Named bft_spec and bft_verifier tools were actually launched with Astra/xhigh configuration; exact runtime model attestation was not exposed to this chat. Builder and simulator must use their named Sol/high roles. No substitution is authorized.

`.build` links to `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16`. Initial root partition free12GiB, data disk free1.4TiB. Rust1.95.0 is available in the instructed toolchain directory. Existing shared public Cargo caches are readable. All heavy work uses flock `/tmp/megaroation-poc-heavy.lock`; jobs2.

Docker28.4.0 client exists; Docker daemon socket access fails EACCES outside sandbox as well. No daemon/proxy/group settings changed. The existing system Anvil1.0.0 was inspected but the actual setup uses the pinned official image's bundled Anvil1.6.0-nightly, commit5e88010a83d1b87b8f4d13058e42a2949d3e9dc0.

The fixed official image release20260918, sourcefd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171, indexsha256:af73f55630690caa35aa0bc015d60dc03333791f966fdcc07b0798e33d4f768e, linux/amd64manifestsha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66 was fetched from public GHCR. Config's revision label matches the source commit. Only official executable, SRS and matching runtime libraries were extracted; executable bytes were not modified. Host glibc2.35 cannot directly execute the image binary requiring2.39; explicit image loader with local library-path successfully runs it without changing host libraries.

`bwrap --unshare-user --unshare-net --unshare-pid` provides task-only loopback and PID isolation. Root filesystem is read-only, only this task data directory write-bound and `/tmp` is private tmpfs. Upstream0.0.0.0 binds remain confined to that namespace. No public production or testnet endpoint is reachable. Dedicated ports18770–18775 cover Espresso/API/dev/builder/prover/tonic/Anvil. Proxy bypass is explicit in the local HTTP client; global settings remain intact.

Setup-only probe ran the unmodified official executable `--version 0.3`, API/v1, dedicated Anvil. Observed block-height1→2→3; captured q0 full block, q0 leaf and dev-info. The bounded probe stopped only its own children and exited0. Commands and raw evidence: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/setup/probe-01/`. The launcher is `/tmp/espresso16-setup-probe.py`; formal runner must be versioned and rerun on the reviewed candidate. This probe did not submit publication and does not count as G-API acceptance.

Full PELOG/PEReader export remains unsupported because authentic proposed_at/live/finalization virtual ticks are unavailable. Unknown must remain explicit. The local source API may be trusted only as declared in the contract; HTTP parsing and recording hashes are not cryptographic finality proof verification.

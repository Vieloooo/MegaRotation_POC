# Feature16 bootstrap runner review

2026-11-16, Asia/Shanghai. **PASS for runner candidate `6248072b3fd0eee1ae468f0961a4d3589291cba6`.** No blocking finding. This is a focused runner/provenance/lifecycle verdict; the official Espresso publication test still requires Tester execution on this exact candidate.

## Scope and carry-forward

The first formal run of `8750535d8f02e920353c1ca827c78046e72d578a` failed before genesis because the upstream default configuration spawned Anvil by name before applying the explicit L1 URL. The old [component review](candidate-review.md) and evidence seal remain unchanged; its G-API gate was already explicitly pending. This runtime failure is not a parser or audit mismatch.

Contract revision `2ed1540b507af076b904816ff0cadc47d2e286ef` froze a task-local wrapper for that upstream bootstrap process. The new source change is limited to `poc/scenarios/espresso-compatibility/capture.py` and its README. An independent exact Git diff from `8750535` to `6248072` confirms **no change** in the Rust crate or the frozen expectations/vectors. Their seven-test external harness and final30-case offline matrix therefore carry forward as unchanged-source evidence; they were not relabelled as new-SHA executions. The stable independent capture oracle remains SHA-256 `e42e68df3f40c991dcce199ba6fa53d03fd2c6d75f243d0bf3ace9c25ead157a`.

## Independent upstream expectations

The expectations were sent before reading the runner fix. Pinned Espresso source `fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171`, `crates/espresso/node/src/lib.rs:1514–1519`, unconditionally creates a bootstrap Anvil and provider. Its `l1_url()` setter at1244–1249 drops that provider when overriding the URL; `crates/espresso/dev-node/src/main.rs:332–338` performs that override after default construction. The node library source, read directly from the pinned archive, has SHA-256 `ff7a517fc698d4d9250a677c48621e7757a8a2313ee107ae199efde469711028`.

The pinned lockfile selects `alloy-node-bindings 1.8.3`, crates.io checksum `a9b2fda91b56bb08907cd44c5068130360e027e46a8c17612d386869fa7940be`. Its [Anvil launcher](https://docs.rs/alloy-node-bindings/1.8.3/src/alloy_node_bindings/nodes/anvil.rs.html) always adds `-p` with the configured port, defaulting to0. Its Drop invokes the [graceful shutdown helper](https://docs.rs/alloy-node-bindings/1.8.3/src/alloy_node_bindings/utils.rs.html), which signals and waits/reaps, with kill-and-wait fallback. Thus a recorded `-p 0` and an already-reaped bootstrap process are expected; bootstrap need not remain alive through the public capture.

Required boundaries: preserve upstream arguments; durably record original argv, exact loader/binary argv, namespace PID/PPID and start identity; execute the same verified image Anvil; avoid host Foundry fallback and fabricated server responses; distinguish bootstrap from dedicated L1 port18775; stop only owned processes and report observed cleanup state.

## Code review and targeted execution

The wrapper writes and fsyncs its invocation JSON and containing directory, then `execv`s the pinned loader/binary with the original arguments. It does not rewrite server output. Its private directory is placed before `/usr/bin:/bin` in the explicitly constructed node environment. The dedicated L1 still starts directly from the verified image at18775. PID and Linux start ticks are checked together, preventing PID reuse from being treated as an owned residual process.

After waiting for owned parents, cleanup requires the recorded process identity to disappear. An unreaped zombie is conservatively a residual process, not evidence that an Anvil server is still executing. The pinned normal provider Drop reaps; unusual failed cleanup remains an incomplete scenario. No exit status is invented for a child the recorder did not wait itself. The actual upstream run must still show the wrapper invocation and final cleanup record.

Independent helper [check_wrapper.py](check_wrapper.py) ran the **real verified Anvil**, without Espresso, in a fresh user/network/PID namespace with only loopback. It used the source-derived arguments `-p 0 --slots-in-an-epoch 0 --balance 1000000`, verified the actual listening port by RPC, checked original argv/exec argv/PID/PPID/start ticks, then terminated and waited only its owned child. A separate synthetic child tested zombie-versus-reaped behavior and PID-start mismatch handling.

| Observation | Result |
| --- | --- |
| Real Anvil listener / dedicated L1 separation | Port37137, distinct from18775; `eth_chainId` returned `0x7a69` |
| Namespace parent/child | PID2 / PID3; wrapper recorded exact values |
| Original/exec arguments and start identity | PASS; original `-p 0` preserved |
| Owned child exit | Observed wait status0 |
| Post-wait bootstrap cleanup | No leftover namespace PIDs |
| PID reuse check | Different start ticks at an existing PID is not the recorded process |
| Zombie/reap check | Unreaped zombie remains present; after `waitpid`, absent |
| Official image Anvil SHA-256 | `9db2381d82aab93e80254df301e55eb56628d131967626750a35a179aa49b24e`, equal to verified image member |

The first independent attempt (`wrapper-independent-01`) failed in the verifier helper's Git preflight before launching any candidate process: Git opens `/dev/null` read/write, disallowed by the read-only root bind. The helper was corrected to consume a candidate-HEAD record made by the outer command before entering the namespace. The complete lifecycle check then passed in fresh `wrapper-independent-02`; the failed preflight is preserved and not counted as a candidate failure. The candidate launcher already runs Git outside the namespace and was not changed for this verifier-only issue.

## Evidence and reproduction

Final evidence directory:

```text
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/verification/6248072b3fd0eee1ae468f0961a4d3589291cba6/wrapper-independent-02
```

It contains image verification, recorded HEAD, helper stdout/stderr, wrapper source/invocation artifacts, actual Anvil log, RPC request/response and `wrapper-independent-report.json`. The generated wrapper's paths are run-specific; its tested hash is `cc3fe88709ccc15ea2b9c13de5796dd8071077142bceae4a347230cf8c63cc4d`. Candidate `capture.py` hash is `e19fe38446656befa58f518a44b98b195dea11597c0c38c7ab763eef301d6660`; executed verifier helper hash is `ddea09d4fd0edc689ca964cc054f6ac15619f2f0e59df5f7198a427ff469283c`.

The exact execution used the host heavy-work flock, verified the cached pinned image, then invoked:

```sh
bwrap --unshare-user --unshare-net --unshare-pid --die-with-parent --new-session \
  --ro-bind / / --bind "$out" "$out" --tmpfs /tmp --proc /proc \
  /usr/bin/python3 poc/verification/espresso-compatibility/check_wrapper.py \
  --candidate 6248072b3fd0eee1ae468f0961a4d3589291cba6 \
  --output "$out" \
  --image /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/setup/official-20260918
```

Here `out` was the absolute evidence directory above, created fresh. The outer shell wrote `git rev-parse HEAD` to `out/candidate.txt` and required the exact candidate before invocation. Seeds N/A. No Rust build or unrelated matrix was repeated for this Python-only change. No source edits were made by the verifier.

G-API remains pending a fresh full Espresso run, raw capture/requery and independent source/audit checks on `6248072`. Unknown timing, unsupported complete PE export and the trusted-local-API cryptographic limit remain unchanged. This smoke proves the new bootstrap launch/record/cleanup path; it does not substitute for the official publication test.

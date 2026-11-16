# Independent C offline harness — prepared before candidate

This harness is written from frozen H3-C-1, the earlier independent C expectations/fixtures and the owner-provided function/metadata signatures. **No C candidate has been inspected or tested at preparation time.** It imports the named candidate only when explicitly run with its SHA. It writes all generated bundles/logs into the supplied new task-data output directory; it never edits the four candidate-owned16 paths.

The source model independently constructs raw namespace/transaction tables, positional responses and expected legacy EC16 audit bytes. Synthetic publication data is visibly labeled. The capture HTTP seam is replaced by that source, and real HTTP is forbidden during the seam test. The wrapper argv test uses an isolated copy and mocks Cargo, image verification, timeout and lock commands; it launches no process namespace, node or Anvil. This tests quoting/mount arguments, not official launch provenance or actual host locking.

Prepared coverage includes:

- Capture/checker loaders independently checked against the same literal input expectations: limits, bool/noninteger values, duplicate keys, unknown fields, hashes, unsafe/symlink paths, native key version and report bounds.
- Invalid manifests rejected before an empty run directory/image/binary is touched.
- Immutable loaded bytes copied after the original source file changes.
- Marker compatibility; ordered inputs and eleven duplicate ordinals; first-repeat semantics; producer/runner revisions kept distinct.
- Input file→immutable recorded copy→actual emitted POST→independently encoded raw positional readback, with namespace filtering and first/requery isolation. Requery receipts use the original first-reader submit transcript.
- Copied data/manifest/report rehashed after corruption; POST mismatch, changed ordinal/submit mapping, missing positional source messages and ACK-only inputs.
- Exact cloned input path accepted without changing public identity; private guest `/tmp` preserved with a separately read-only bound input directory and a metacharacter-containing manifest filename passed literally.

Candidate API seams are documented in `api-bindings-v1.json`; they are ordinary offline test hooks, not native proof capabilities. Actual H1 validation, C07 genuine M1 publication, real dev-node launch, QC/VID and official timing remain **NOT_RUN**.

After a named candidate is delivered and reviewed:

```sh
python3 poc/verification/publication-gfr-healthy/slice-c/independent-harness/tests.py \
  --repo /home/jy/.codex/worktrees/1e2f/megaroation_poc \
  --candidate EXACT_REVIEWED_C_SHA \
  --output /mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/NEW_C_OFFLINE_RUN
```

The output directory must not already exist. These stdlib tests need no Rust build or official service. Record exact command, candidate/owned-source hashes, fixture/harness hashes, raw output and any test-source adapter correction before reporting acceptance. Do not use this synthetic matrix as C07 or healthy M1 evidence.

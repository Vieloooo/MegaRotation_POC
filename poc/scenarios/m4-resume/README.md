# M4 independent D and preset RESUME

Run on the clean integrated candidate on the Xuelab host. Supply a fresh output
directory. The launcher owns the heavy build lock.

```sh
/home/jy/miniconda3/bin/python3 -B poc/scenarios/m4-resume/run.py \
  --candidate "$M4_CANDIDATE" --seed 42 --output "$M4_NEW_RUN"
```

Use `--wall-pause-ms 650` with a new output directory for the wall pause
comparison. The pause occurs before dispatch of the preset RESUME and leaves
the virtual scheduler unchanged.

The 15-worker Hub projects each selected batch's exact committed G position and
execution inputs. Batcher, Certifier, and DA run in the M4 port process with the
single existing L1 Driver. The independent A and B derivation states receive
only their own verified Inbox/DA histories and fixed genesis/configuration.
The port records each historical cut on arrival at the paired CursorSet.

The sealed directory contains the original virtual-time trace and native
frames under `raw/`. Normalization in `raw/trace.virtual.jsonl` removes only
the process-local diagnostic fields. `raw/l1/` contains the Anvil bundle,
driver and Hub streams, and the signed RESUME outbox. `raw/derivation/`
contains the batch source and execution records, DA bytes and receipts,
accepted inputs, independent named derivation streams, and the compact
ResumeCut files for A and B. The cut file name gives the position of
ResumeRecorded, using the transaction-local log index. Its digest is committed
in `GReset` and the Resume lifecycle record.

The replay record input uses tag 5, followed by framed canonical L1Position,
u64 sequence and framed `PoC/PolicyResume/v1` source bytes. Reset effect tag 2
stores the framed canonical L1Position, u64 target height and 32-byte cut
digest. The Resume result anchor source is tag 2 followed by a framed
L1Position and the existing source objects. Resumed uses lifecycle projection
tag 3, with sequence, branch, framed anchor and the array of framed virtual
blocks. Integer fields in these encodings are big-endian. Historical
authorization uses the M3 Plan format with the Resume cause, actual L1
ResumeRecorded position, and the derived target origin as the floor.

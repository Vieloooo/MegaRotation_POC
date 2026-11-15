# Native recovery and timing implementation

The pinned source is Commonware 2026.10.0 at upstream commit
`92a2f68a0e131c8b69de41e84aec37da6280f212`. The package checksum is
`ca8d109fab117ec237836aa956c9aba0950bf7eeefa5506c4aa4585e7fe235f5`.
The allowlisted source changes and lead's feature line are reproduced by
[the upstream-relative patch](../patches/commonware-consensus-native-recovery-v1.patch).
The contract is [NATIVE-RECOVERY/TIMING v1](contracts/task-contract-native-recovery-v1.md).

## Bounded process scenario

```sh
./poc/scripts/bootstrap.sh build --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline --bin native-hub --bin native-worker
.build/target/debug/native-hub --scenario recover-pending-verify --seed 7 --output <new-dir>
```

The hub launches two native workers and holds BODY until reference tick 1,250,000.
The resolved profile fixes worker 1 as the recovery target, an
`AFTER_ACK_PENDING_VERIFY` boundary and 20,000 virtual microseconds of downtime.
Worker 0 is an actual native voter and supplies its own witness-authorized BLS
Notarize. Members 2–4 have no active script in this bounded fixture. The hub
expects zero finalized application heights at the end of this trajectory.

The new config field `base.recovery` and private `Launch.recovery` are omitted
for Fresh. The private launch discriminator is `mode`, with `INITIAL` and
`RESUME` variants. Resume includes `source`, `fresh_target`, `floor`,
`old_incarnation`, `storage_generation` and `recovery_tick_us`.
The latter schedules a new incarnation and constrains downtime. Only a new
request and accepted response under the new incarnation calibrate its clock.
The recovery profile hash equals the resolved config hash. The source manifest
registration hash uses the existing length-framed
`I1b/recovery-state/v1` domain over the tag `EMPTY_NATIVE_BOUNDARY`, old
incarnation u64BE, source cut hash, witness sequence u64BE and witness hash.
The worker validates that the source boundary is actually empty before using
this hash in its resumed registration.

The worker writes its GrantIntent before delivering input. NativeRx records the
actual consumed reference before returning bytes, and the batcher carries that
opaque reference. The applied voter proposal record is staged only after
successful native `set_proposal`. The native Verify claim matches the real
application dispatch, which allocates and binds the original WorkBinding and
producer. Native preparation and actor/application evidence share the
controller's auxiliary writer and the existing owned I/O queue.

The completed root step binds a cut with ordered OutputIntents, the empty
archive/native evidence, a live pending Verify, absolute native timing, role RNG
cursors, clock and timer frontiers. The bounded path rejects evidence of native
signing or other archive/network history. A resumed worker validates the
external root and auxiliary receipt, reads the exact persisted Grant and applied
selection, and checks the matching operation. It copies protected source files
before reopening. An absent voter partition is accepted only with a verified
empty native boundary and a witness scan whose complete history has no native
storage or authorization records. The raw source snapshot records actual
`NotFound` in `source-snapshot/absence.json`. The target advances witness
generation through two linked witness records without a synthetic file or
barrier.

The application recollects the same WorkBinding and reattaches an actual
Archive producer. The voter uses the native verification claim path, compares
the reconstructed proposal/decision state and then installs original absolutes.
Startup parks the voter, batcher and resolver until the root binds all six
readiness observations into a synced StartupReleased record. The hub cancels
old-incarnation timer and clock obligations and validates the new physical
timer arm against the original local deadline.

## Checks and scope

The first builder development loopback run completed for seed 7 under
[poc/artifacts/verification/native-recovery/process-r1-seed7-attempt1](../artifacts/verification/native-recovery/process-r1-seed7-attempt1).
The hub retained root receipt sequence 4, auxiliary sequence 33, and witness
sequence 1 before crashing worker 1. At reference tick 1,025,500 the original
certificate timer had local deadline 1,419,500. A new worker PID and incarnation
released its startup fence at reference tick 1,046,000 and armed a new physical
timer at the same local deadline. The 54-line trace SHA-256 was
`8a505194e9d88585ca46d94c21ea1f8669428b0ac617cc8e316aca7b2e0a4f74`.
The worker-0 and resumed worker-1 processes exited 0. This run supplies
development evidence for R1 and leaves independent verification open.

The integrated development source passed 118 tests with
`./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline`.
The pinned rustfmt check, `git diff --check`, upstream patch dry run and locked
native binary build passed. The rebuilt hub SHA-256 is
`92534708e966e98050c0e4241b9924181c939e77281252a572e4f92afeb73f7b`.
The rebuilt worker SHA-256 is
`8a8f25112481a960c91a49c08d749627685d94e21bd93bf14cc33dfaabc711c7`.
The consensus patch SHA-256 is
`ca8f553fadc52e4469c3896f17c6ff4cc003945dd520547a226a5cd6aba97de3`.

The builder pending-Verify test ran with actual
witness-authorized leader bytes, decoded protected records, current incarnation
2 calibration, original RNG cursors, live Archive ownership and a durable
six-party fence. It is a controlled in-process fixture.

The lead's Fresh compatibility smoke used the earlier development binaries in
[poc/artifacts/verification/native-recovery/fresh-compat-35abf79-evidence.json](../artifacts/verification/native-recovery/fresh-compat-35abf79-evidence.json).
All 19 evidence files matched formal Fresh revision `fec7ade`. The builder
test also compares the first Fresh root-step record hash to the frozen original
`17a0801a09ecf0fe217fd0cb4b9469485521512e7ee0eb04df31f899b4933166`.
The two recovery-only protobuf fields continue to omit None in Fresh JSON.

This is a development handoff for the pending-Verify trajectory and recovery
primitives. R2–R8 and R17 still need complete proposal mailbox custody, the
other native actor phases, durable body receipts and the phase matrix. R9–R10
need the full retry/latch/floor tests. R11 and R15 need the work for authorized
interrupted and lost-Ack recovery, durable prepared cache and subsequent
outbox reconciliation. The bounded after-Ack path preserves exact cached root
Ack bytes and stages OutputIntents, but the interrupted pre-root transaction
completion policy has no implemented process path. R12–R14 have focused
guards, and the full negative process matrix remains due. The bounded startup
path rejects inputs outside its explicit new-clock, RecoverEngine and release
sequence. Holding or coalescing arbitrary ingress across the fence awaits the
broader recovery phase. R16 needs actual
resolver repair across restart. R18 needs independent process replay and
mutation results. Overall native recovery and I1b remain open.

## Crash-left head scratch

The auxiliary and root-step readers first validate the committed head, trusted
prefix and complete authorized descendant through EOF. In an exclusive mutable
working copy, they admit an owned regular single-link pending file only if it
contains a 0–88 byte prefix of that descendant's recomputed head. They preserve
the exact prefix as `<pending>.recovery.<candidate-sequence>.<prefix-length>`.
An existing backup can only deduplicate an identical regular singleton. The
readers sync preservation before installing the recomputed head. No pending or
backup bytes select a head or authorize a descendant. Pending scratch with a
missing, torn or unauthorized descendant fails before tail repair. The worker
copies possible scratch files from its immutable source with type, ownership and
size checks so the working-copy readers can enforce the same rules.

The two independent reproducers passed with the original fixtures and
assertions. The only harness change is
`use crate::aux as native_recovery_ledger;` so both actual production modules
refer to the same helper. Builder tests cover empty, partial and full images,
repeated preservation, the next append, conflicting backups, wrong binding and
prefixes, oversized and aliased files, missing or torn tails, and nondestructive
source copying. This closes the pending-file collision in the two readers.
The broader R11 process cut matrix remains open.
The complete locked offline prototype suite passed 129 tests after the fix,
including the Fresh golden root-step hash. The bounded worker still opens this
snapshot without authorizing an interrupted transaction. Process-level
authorization and reconciliation of an outstanding grant belong to R11.

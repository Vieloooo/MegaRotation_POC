# Planned independent coverage and limits

The original 22 Rust tests were prepared before execution and passed on candidate
`129c6674356a98a908289a94df07802808603ff1`. Five independently authored
capacity-edge tests were then added to close explicit R33 coverage gaps; they are
**NOT RUN at this preparation revision**. Subsequent execution status is in the
named-candidate verification report. All expectations come from the sealed
source-first matrix and frozen H2-REDUCER-1, not candidate output.

The implementation API declaration and read-only getter signatures were read to
align call spellings after the oracle/contract freeze. No H2 reducer/receipt/
scheduling algorithm was used to calculate expected values.

| Matrix rows | Independent Rust tests / checks |
| --- | --- |
| R01–R05 | unique descriptor/Create; metadata conflict; each PreparedRef binding mutation; exact preparation replay; fixed typed identity checks |
| R06–R10 | higher Preparing vs Ready; actual intent/receipt A→B→A; wrong predecessor/recursive candidate/same-height conflict; Selected+Preparing; Selected+Terminated still excludes B |
| R11 | source-level stop guard and original instance reference verified; actual deadline/future retention is OPEN native scope |
| R12–R17 | closed scope/late completion; same-pc multihandoff with no intermediate startup; source provenance, gap/replay/conflicting effects/input_ref/index; missing event validity leaves entire accepted state unchanged |
| R18–R22 | inert immutable plan, repeated close-delta intents, same input/different time commit mismatch, stale commit, duplicate commit, foreign fault latch, fixed origin_revision/ordinal/worker IDs |
| R23–R26 | intent vs applied entry; unconfirmed Enter parking dependency; held/cancelled starts, late success, unknown/conflicting receipt, success replay, stale failure cannot regress completed/cancelled actions |
| R27–R29 | stop−1/equality, ClockUnsafe at last accepted time, private cause/fence, no public cursor mutation, surviving Publish identity, no same-session fence bypass |
| R30 | accepted APPEND tip excludes lower anchor, higher regional candidate alone does not; delayed lower registration cannot create new engine |
| R31–R32 | wrong binding, unsupported recovery, nonmember initial duty and later assignment path, deterministic duplicate-clock/source history |
| R33 | independent exact/+1 checks for source records2048, combined session states128, results256, lifecycle list64; five added tests cover branch tombstones16, effects128, Programs64, timestamp overflow and h_max+1 overflow; rejection leaves accepted state intact |
| R34–R36 | distinct early h11-result-reference vs late lawful h14-reference traces, fixed successor11–20, no second Enter; shifted window rejects |

Action ledger4096, descriptor/lifecycle byte totals and dependencies64 are not
independently reached by this harness; their guards and reachability under the
other fixed bounds require explicit source review, reported separately. Branch
and effects capacity inputs are structural fixtures; they do not assert actual
M2 branches or public GFR no-op Duty records.
The harness does not count arbitrary private-state injection as an independent
reachable trace. A future targeted addition is justified by a concrete coverage
gap/finding and must be versioned before naming its tested candidate.

ReadyState cryptography/execution, native consensus/signature guards, real
deadline/firstSeen correctness, actual H1 root/fsync/aux persistence, precise
local admission/certification behavior, actual public GFR history, real
publication and all M1 process/end-to-end success remain OPEN. Synthetic
trusted-input references do not establish these facts. R34–R36 do not execute
Accept or construct a real h11/h14 block; they test the reducer's immutable
context and timing-appropriate intent/reference handling only.

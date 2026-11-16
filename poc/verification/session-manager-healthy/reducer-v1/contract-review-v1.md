# H2-REDUCER-1 independent freeze review v1

2026-11-16, bft_verifier GPT-6 Astra/xhigh. Scope: pure trusted-input Manager reducer and private plan/commit/action ledger. No implementation or executable test inspected/run at this review stage.

**Final disposition: approve pure-component freeze at contract SHA-256 `fe2f962d8480a0cb6b5d0bbb26ff6748d3e7149dfa6e6b05a9732c3c91c70a96`. CR1–CR3 are incorporated; no remaining pure-component freeze hardblock. No requirement to wait for H1 runtime or native proofs.** This review does not authorize changes outside the assigned H2 paths or close native/system gates.

## Source-first sequence and reviewed version

- Existing oracle retained unchanged: `11f745e81faa46c03d85d2ee1ed722a842f8631ef62e96dd556e8610cece7ceb`.
- Increment `source-first-expectations-v1.md` sealed before the new contract was read: `9ad71186b41e85a5e30100a7fd63d9a009435e12e4459046e3056ad8cd7965e5`.
- `case-matrix-v1.md`: `4d78b0c12f65f7be5f0c5a2da0048b7815f3ef9e027290e942960ac39cd87395`; 36 planned rows reuse existing source cases and add commit/receipt/guard-failure logic. All NOT RUN.
- Dispatch `d55d39f73593676325bdc0fbb7ba3845ca93025f` source bytes: `4a211cd38a5db108dc3b8548a87c19e599cb3870e534e513b41e487b4c5cc8b6`.
- Reviewed contract: `poc/docs/contracts/h2-session-manager-reducer-v1.md`, 180 lines, SHA-256 `caac7265ac724841a53bbe3c2a0ca6d151247ba3584a8deb955de14b988a3b24`.

## Findings and minimum freeze amendments

### CR1 — retain G APPEND exclusion in this pure slice

- **Location:** candidate contract line114 defers an actual accepted predecessor physical-tip exclusion to a later fact/interface version.
- **Source:** v10 SM:275–276 permanently excludes a lower candidate anchor once a higher predecessor physical block enters G; GFR:240 describes Append's last endpoint as the accepted physical target, with intermediate padding covered by that result. Independent R30 and existing H2-20 exercise this distinction.
- **Impact:** before successor selection, a previously entered low-anchor candidate could continue to be locally preferred or a late lower candidate could be created after it is already impossible to select. A source-record consumer should not silently advertise full candidate handling while omitting this known public exclusion.
- **Minimum accepted approach:** consume trusted `Append{last,source:Regional(ResultKey)}` in source effect order. Require the result source cid to belong to the verified selected predecessor history. The trusted adapter contract guarantees the existing GFR semantic fact that `last` is the result's physical target height. Maintain a monotonic accepted-physical-tip height per predecessor, and exclude/terminate every successor candidate referencing that predecessor with anchor.height<last, including Preparing/Paused/Entering. Check the same fact during later registration. Keep unknown/DER sources unsupported rather than inferring physical identity from arbitrary height ranges. No new public encoding or competing GFR replay is needed.
- **Replay evidence to run later:** R30: A7/A8 candidates for predecessor P; accepted APPEND from P with target8 excludes A7; merely observing regional8 does not. After APPEND, delayed Register(A7) remains nonrunnable. No trace executed yet.

### CR2 — unresolved Enter must also be parked before switching

- **Location:** lines100–101 currently require Pause for entered A and acknowledge that missing Entered is not proof of nonentry; the actual dependency rule must explicitly include an Enter intent without final receipt.
- **Source:** dispatch:28–30 requires monotonic guards and original-engine parking; SP:184 and SM:257 disallow reinitialization; one scheduled cid per session remains mandatory. Independent R23/R26 and C4 cover pending application and stale receipts.
- **Impact:** if H1 already applied Enter(A) but its receipt is delayed, choosing B based on `entered=false` could bypass parking A or produce contradictory local phase claims.
- **Minimum accepted wording:** when A is entered **or has an unresolved/possibly-applied Enter**, install A's disabled schedule guard and create/reuse Pause/Close on its original instance. New B's Enter/SelectBack depends on that safety action Applied. If A genuinely never entered, the park receipt only confirms the guard/parking; do not fabricate entered=true or the source-level Paused state. Keep the original Enter ID. Late Entered records history without reopening current schedule/terminal guards.
- **Replay evidence to run later:** withhold Entered(A), switch to Ready B, verify B is undispatchable before A's park receipt; then deliver old Entered(A) after the switch and prove A stays parked/nonrunnable. No trace executed yet.

### CR3 — commit identity includes the planning clock argument

- **Location:** plan signature line48 has separate `local_time_us`; CommitRef line71 and accept rules line133 bind full input/cause but must specify that time is part of it.
- **Source:** dispatch:28–29 exact immutable delta/guard commit; SM:259/ SP:416 stop depends on current local time. Independent R18–R20 require exact plan binding and stale-plan rejection.
- **Impact:** two plans from the same base with the same ManagerInput but different local time can have different terminal projections. Binding only input/revision could let one commit attestation install the other's plan.
- **Minimum accepted wording:** complete cause=(ManagerInput,local_time_us), plus exact manager/worker/context and base/target revision binding. No wire/hash is required; typed equality is sufficient for this fixture boundary. Failure latching must reject foreign-manager deltas while preserving any same-manager guard already durably installed. ClockUnsafe can use last accepted local time; acquiring a new trusted clock sample cannot be a prerequisite to installing its fail-closed guard.
- **Replay evidence to run later:** same base/input planned once before stop and once at stop; cross-use CommitRef must fail. Valid private Unsafe event at last accepted time still closes the gate. No trace executed yet.

## Accepted adapter choices and oracle interpretation

- `ActionId=(manager_id,worker,origin_revision,ordinal)` is a private typed identity, with no new wire bytes or hashing. Test its Eq/Ord values and association with exact action parameters; an independent bytes encoder is neither required nor useful here.
- Uniqueness is within one accepted manager history. Two mutually exclusive, never-committed and never-dispatched proposals can occupy the same speculative revision/ordinal; at most one may install, and its exact cause/action parameters must match. Retrying an existing accepted action never changes origin_revision, including after receipt revisions advance.
- Trusted-input constructors are expressly not authentication. The contract's binding checks and explicit fixture labeling satisfy the current pure component boundary; actual H1 state/ReadyToken production stays OPEN.
- Entering/Pausing/Resuming explicitly distinguish an intent from confirmed native execution. Receipt handling preserves first successful fields and terminal precedence.
- Faulted after guard persistence is a deliberately terminal component state with no recovery API. It may stop the run and retain pending audit rather than build recovery machinery. This meets the dispatch's minimum fail-closed requirement.
- ClockUnsafe's all-registered-cid closure and session fence are documented conservative adapter choices. It remains a private cause, does not change public GFR/cid encoding, and recovery never revives a terminal context.
- Complete ReplayRecord typed Eq and per-event GFR-LIFE-1 projection reuse are appropriate; no new public record hash/codec is necessary.
- Early h11 and later legal h14 traces remain distinct. The contract's +10s genesis offset is an explicitly separate numeric example from the original oracle's zero-origin example.

## Limits

No test PASS, actual durability, cryptographic validity, original native engine preservation, native timer semantics, H1 Accept/certify mapping, process recovery or M1 endpoint is established by this freeze review. Later harness must use the exact frozen API and named candidate, retain fixed inputs/results, and run heavy work only under the host lock/jobs2 constraints. Candidate file:line findings will be produced only after a candidate exists; the file/lines above identify contract ambiguities.

## Final read-only freeze check

Read the lead's amended 182-line contract and recomputed SHA-256 `fe2f962d8480a0cb6b5d0bbb26ff6748d3e7149dfa6e6b05a9732c3c91c70a96`. Line114 includes trusted APPEND physical-tip tracking, atomic lower-anchor exclusion and later registration barrier (CR1). Line100 explicitly parks unresolved Enter before another candidate becomes dispatchable and does not fabricate entered/Paused (CR2). Lines71/133 include local_time_us in the exact cause; line135 checks foreign/base failure latches while preserving durable guards; line124 permits Unsafe at last accepted time (CR3). The source-first increment and matrix remain unchanged. Contract now labels the slice FROZEN and explicitly retains native/H1/H3/system gates OPEN.

Next work is a separately assigned `/tmp` independent harness derived from these expectations and the frozen interface. No candidate build or actual tests may run until lead names the candidate and grants the single heavy-task slot.

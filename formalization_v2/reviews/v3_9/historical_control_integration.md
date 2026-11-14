# Historical task controller: arbitration and finite health interference

Date: 19 October 2026. Conditional controller design for JG2 in
`historical_service_fresh_check.md`. This file specifies only controller
work and its interface to the historical task runtime. It does not adopt
v3.10, change a frozen source, derive a complete historical catalogue, or
establish the historical service-domain premise.

## 1. Scope and fixed semantics

Assume that canonical registration and the historical task runtime have
already been supplied. Within one chain, write `tau=(i,r)` for the task
whose region is i and whose immutable authority is `AUTH[r,0]`. Every
reference also validates that authority's hash. Neither a new beta nor a
new roster for physical region i changes tau. Each registered task has
one outstanding token `z=(tau,serial)`. Its exact-task accepted DATA
discharges that token and atomically tail-enqueues the next token. Let
`s_tau` count these discharges. Selection, Ready, commitment, a receipt,
and raw publication do not increment it.

The runtime supplies the following rules, which this controller cannot
override:

* Force retains exact pre-payload G and provenance, increments beta
  once, and uses the public task selector. A continuously eligible
  unserved pin is retained; otherwise the oldest eligible token is
  selected. Temporarily ineligible tokens retain their queue age.
* A historical appointment creates fresh `(beta,r)` core state under the
  unchanged authority. Zero-output retries retain its token and exact
  anchor once that pin remains eligible. OFF does not retroactively
  revoke an already fixed handoff or assignment.
* One accepted DATA discharges at most one outstanding token. The
  continuing duty keeps its ordinary remaining quota; REGULAR remains
  unrestricted and preserves beta.
* Independent STATUS/CLEAR, matching MSIG, and AUTO retain their original
  prelude order and strict cutoff. The first due MSIG matching the
  **entry** beta acts. Other due MSIGs are consumed as stale. There is
  no new-version DATA in the crossing block.

All newly issued MSIGs below have empty attached configuration. They
carry exactly the existing chain, target beta, activation time, control
identity and authorization. Task/token identifiers and scheduling causes
are private journal metadata, not additional signed control fields.

The frozen committee is an honest coordinated service, not a health
oracle. Its enabled local work below is eventual, with no new decision,
signing, retry, catalogue, or submission latency bound. Once a valid
control is actually submitted, the original L1 bound applies. The frozen
text does not explicitly guarantee success of a local attempt to enter
that interface; Section 6 preserves this boundary. Local arbitration and
finite interference are proved below without an ingress assumption;
the public scheduling-opportunity implication needs actual submission.

## 2. Shared runtime interface

The runtime delivers monotonically ordered, authenticated
`ScheduleView(p)` events to this controller. A view identifies a
canonical source subtransition p and contains:

| Field | Meaning |
|---|---|
| `kappa, beta` | Accepted tip `(chain,height,hash,provenance)` and current branch version at p |
| `Register(tau,z)` deltas | Canonical task registration and initial token serial; no private discovery event |
| `Service(tau,z,z_next)` deltas | Exact accepted DATA discharge and atomic successor-token creation |
| `d_i, s_tau` | Monotone accepted physical-region DATA counts and exact-task discharge counts |
| `pin, duty` | Current outstanding pinned token, if any, and the assignment actually serving it |
| `Eligible`, `head` | Eligibility under each task's old AUTH and the oldest eligible outstanding token at this completed scheduler cut |
| `Force`, control deltas | Effective Force and canonically processed STATUS/CLEAR/MSIG identities, including stale consumption |

The catalogue and head projection must be complete for this view's fixed
cut. An unresolved catalogue job is `unknown`, not an empty catalogue or
proof of ineligibility. The controller may await its complete view
asynchronously. It may not skip missing registration/service events and
then sign from a later apparent tip. The runtime eventually exposes each
fixed required finite view; later arrivals do not enlarge that view.

The timed regional path does not call this controller, await its view,
obtain its signature, or scan its task-credit table. In particular P6
commits accepted DATA and its discharge/requeue transaction before any
controller catch-up. A prepared eligible pin retry uses its retained
projection and does not await later catalogue materialization. Establishing
the runtime's prepared-handler bound remains the runtime integration's
separate obligation.

Define

`ServingPin(V) := pin is outstanding, eligible, and assigned to V.duty`.

`Need(V) := V has an eligible outstanding token and not ServingPin(V)`.

A younger pin selected while an older token was parked may finish its
nonpreemptive turn after the older token becomes eligible. Thus Need is
**not** the test `head != pin`. Serviceability of every possible blocking
pin remains part of the conditional runtime theorem.

The controller maintains a durable scheduling obligation when Need is
true. It recomputes it on each complete view, especially exact discharge,
Force and eligibility changes. A merely reserved or unsubmitted control
does not itself satisfy this obligation. An effective Force supplies a
public selection opportunity, after which Need is recomputed. Exact
discharge supplies service and also triggers recomputation. Issued
controls are never revoked because this local predicate changes.

## 3. Durable controller state

All state below belongs to one coordinated service and survives crash,
restart, replacement, and roster/catalogue growth. There is one serialized
transaction order for replay, reservations and signature release.

| State | Durable content and update rule |
|---|---|
| `cursor,V` | Last complete ScheduleView; only forward replay is permitted |
| `f[kappa], ell[kappa], c[kappa]` | Initially unused MSIG, lease and optional CLEAR credits; each becomes spent once |
| `u_i[d_i]` | Existing physical-region lease credit, keyed by the observed accepted count |
| `u_tau[s_tau]` | Exact-task lease credit, keyed by the observed discharge count; never replaced by `u_i` or by a new AUTH |
| `NeedLatch` | Current scheduling obligation, its first observed event, and any associated operation; moving tips or new catalogue entries do not reset its age |
| `Op[oid]` | Existing L1 control identity; exact bytes and target; reservation keys; purpose; all release, submission, finalization and processing evidence |
| `Lease[lid]` | Region, its independent ON and at most one OFF, reserved credits, charged tasks, inherited charges, and completion state |
| `Legacy` | Complete finite adoption inventory, including reserved and unreleased operations, uncertain submissions, attached configurations, and restoration hazards |

An operation has monotone lifecycle flags `reserved`, `released`,
`submitted`, `finalized`, and `processed`; some observations can establish
more than one flag at once. `submitted` requires actual-submission
evidence, not a signing event or local send attempt. `processed` records
either execution or stale/duplicate consumption. A never-released record
may instead become definitively abandoned. A released record of uncertain
submission remains a reconciliation obligation even after it is no longer
safe to attempt a first submission.

No lifecycle state, stale target, expiry, failed submission, abandoned
operation, Force, Ready, new registration or new service provider refunds
a credit. Distinct tips at the same DATA height cannot be invented: kappa
includes the hash and original provenance of the monotone accepted chain.
Duplicate replay has no update effect.

## 4. Event and reservation table

The following transitions are atomic with respect to each other. A
signature is released only after the full corresponding durable
reservation has committed. Eligibility reports never bypass this table.

| Event | Required transition before another signing decision |
|---|---|
| Complete `ScheduleView(p)` | Verify predecessor/cut continuity; replay its deltas exactly once; commit cursor, tip, counts and pin; recompute Need before optional H1 arbitration |
| `Register(tau,z)` | Create its token/count mirror and task-credit namespace; attach every still outstanding lease or legacy OFF hazard for its region as described below; do not renew any older task's credit |
| `Service(tau,z,z_next)` | Advance only this task's `s_tau`; advance the producing region's `d_i`; mirror the runtime's atomic discharge/requeue and any matching pin clearance; preserve all spent old keys |
| Effective Force / OFF / ON / CLEAR | Mirror public effects; retain issued operation bytes and targets; recompute Need; no credit replenishment |
| Mandatory scheduling reservation | If Need has neither an equivalent actually submitted current-beta opportunity nor an unfinished local scheduling dispatch job and `f[kappa]` is unused, spend it for one ordinary empty-configuration MSIG before any optional H1 reservation |
| Optional H1 reservation | Allowed only when no scheduling obligation is awaiting that credit; spend unused `f[kappa]` for the existing empty-configuration MSIG |
| Lease reservation for i | Require unused `ell[kappa]`, `u_i[d_i]`, and `u_tau[s_tau]` for **every registered task of i**, plus no outstanding lease of i; spend all these keys in one transaction and bind the lease |
| Lease ON availability | Authenticate this lease's exact independent ON as finalized and available; only then may its one OFF gate run |
| Lease OFF gate | Recheck actual-time inequalities, ON evidence, and one-OFF limit; bind and reserve the exact OFF before release, or abandon OFF permanently without refund |
| Optional H3 reservation | Spend unused `c[kappa]` for one existing CLEAR; never alter eligibility or cooldown locally |
| Submission/finalization/processing evidence | Append evidence for the same oid; update lifecycle flags, never payload/target/credit keys |
| Lease completion | Retire its outstanding marker only after every issued record is canonically processed and any unissued OFF is definitively abandoned |
| Restart/replacement | Restore the journal before signing; reconcile uncertain operations by identity; never initialize fresh credits for an already seen key |

The task-credit check includes parked, pinned and temporarily ineligible
tokens. It is not restricted to the current head, newly discovered tasks,
the newest AUTH, or a roster selected by the health report. A task's
discharge and replacement token are one event, so there is no transient
gap during which a lease can omit that task.

On registration during an outstanding lease, record an inherited charge
`(tau,current s_tau,lid)` and mark `u_tau[current s_tau]` spent. This does
not modify the lease's signed records or cancel its already reserved
permission; it prevents the new task's fresh namespace from admitting a
second lease before its own service. Multiple finite legacy hazards may
all be recorded against that same spent bit; they are separately counted
in the backlog and do not mint extra new permissions. If no lease/hazard
is outstanding, the new task's first bit is unused. Its arrival still
cannot reset another task's bit. Reservations and registration are
serialized, so either the new task participates in the original complete
lease check or it inherits that reservation.

## 5. Mandatory scheduling and H1 arbitration

Controller work is scheduled as follows. Replay, lifecycle reconciliation,
mandatory dispatch and already obligated lease work receive eventual
execution. Optional H1/H2/H3 input cannot starve any of those classes.
Within the reservation transaction for each newly observed tip, mandatory
scheduling has strict priority over optional H1. A sustained stream of
new tips, catalogue additions or health reports cannot restart a pending
mandatory job or postpone its dispatch indefinitely.

1. Apply available complete view deltas and update Need. An unresolved
   view pauses new signing; it does not erase an existing obligation.
2. An equivalent opportunity is an **actually submitted** immutable MSIG
   whose target is the observed beta and which has not been canonically
   consumed. A health-origin MSIG can qualify; its empty configuration
   and public selector have the same effect. A merely reserved, released,
   expired-uncertain, or known-stale operation cannot indefinitely block
   a mandatory reservation. Its spent credit nevertheless stays spent.
3. When Need remains and no equivalent opportunity exists, reserve the
   first observed unused `f[kappa]` for scheduling. If this tip's credit
   was already spent, carry the obligation to the next tip. Do not
   steal, refund, repurpose, or re-sign the earlier operation.
   Keep at most one unfinished local scheduling dispatch job for the
   observed beta. This local slot prevents redundant drafting while its
   enabled steps execute; it is not a public opportunity. Expiry or
   definitive abandonment frees the slot without refund, and an observed
   beta change moves its immutable old job to reconciliation. A merely
   reserved optional H1 job holds no such scheduling slot.
4. Run this reserved operation's enabled local dispatch steps eventually,
   using the issue rule in Section 6. Do not wait for
   a failure report, catalogue expansion, a preferred health estimate,
   recovery, task completion, or a more convenient beta.
5. A new effective Force can make its original target stale. Keep the
   record and its lifecycle; recompute the scheduling obligation from
   the new public state. REGULAR alone cannot make its beta stale.
6. Only if no mandatory obligation is awaiting reservation may optional
   H1 reserve this tip's unused f credit. H2 and H3 use different credit
   namespaces but remain subject to eventual mandatory dispatch and
   their complete reservation rules.

Operations are not retargeted to the queue head. The public selector
chooses the task at the original first crossing. Scheduling an MSIG does
not suspend DATA, delay an existing matching control, or protect a
private retained prefix. Even if its motivating token is served before
activation, an issued matching record still acts.

## 6. Actual submission, signatures, and retries

Put `D = T_L1 + epsilon_E + eta`, with the frozen positive eta. For each
record define its strict latest first-submission boundary
`b = theta_a - D`. A first actual submission at s is permitted only if

`s < b`, equivalently `theta_a > s + T_L1 + epsilon_E + eta`.

The local ISSUE procedure prepares a finite future-dated record, reserves
its exact bytes and credits durably, releases its signature, and runs the
dispatch gate. The timestamp may be chosen just before signing, but
future validity is rechecked at actual admission. If signing or admission
has consumed the available interval, it cannot count a later invalid
submission as success. The record remains spent and follows the expiry
rules below; a later unused tip can admit a different operation. There is
no assumption that infinitely many such attempts must include a success.

For a successful actual submission the time s is the actual entry into
the L1 submission interface, not an earlier queue, signature, RPC start,
or guessed inclusion time. A concrete submission adapter must enforce
the gate there; checking only before an arbitrarily delayed network
request does not enforce it. If that adapter cannot be implemented from
the intended original L1 interface, this remains an implementation
boundary, not permission to relax the inequality.

The source distinction is material. The frozen Model, lines 199–206,
requires an honest coordinated committee that submits only authorized,
immutable, future-dated records. Lines 208–215 give a bound **after** a
valid operation is actually submitted. Lines 272–283 require the
actual-time inequality. Baseline H1 says to choose the timestamp using
actual submission time, but these clauses do not explicitly guarantee
that every attempted admission eventually succeeds. If the intended
abstract L1 interface makes actual Submit an enabled action under the
correct committee's control, eventual execution of that enabled action
is sufficient. That interface reading must be established; it is not
derived here from the conditional finalization bound.

Accordingly, Section 5 imposes eventual enabled local execution and
nonstarving dispatch, not guaranteed success of an external API. An
infinite sequence of correctly abandoned expired operations satisfies
the local finite-credit rules but does not establish a public Force.
The public-opportunity part of Proposition 3 states its actual-submission
condition explicitly. This file adds no L1 ingress service, successful
submission axiom, or signing-to-admission latency.

For every operation, including leases and imported legacy attempts:

* A retry carries identical signed bytes and the same control identity.
  A changed activation time or target is a different operation and needs
  its own permissible credit; it is never a refund/retry.
* If first submission is unconfirmed, every retry capable of becoming
  that first actual submission must pass `s<b`. After b, stop active
  submission and retain a reconciliation-only obligation. Expiry is not
  proof that the record was never submitted earlier.
* If original actual submission is established, finalization/visibility
  follow the frozen L1 premise. Retrieving its evidence or repeating an
  already recorded control identity grants no second action. A late
  network attempt must not be interpreted as a fresh first submission.
* Staleness never changes its timestamp, original beta or public order.
  If a record is canonically due while stale, consume it by the original
  rule. Do not delay a matching record until its task has completed.

These rules apply also to ON and OFF separately. An H2 reservation first
issues independent ON(i) at finite `theta_on` and waits for that exact
record's finalization and availability. Its OFF may then be actually
submitted at `s_off` only with

`s_off + D < theta_off < theta_on`.

There is one OFF record at most for the lease. If no such interval is
available before release, abandon OFF and preserve the ON and all spent
credits. If an already released OFF has uncertain submission, use only
identical gated retries and reconcile it; never sign a replacement OFF.
No new lease for i begins until the old lease's full completion rule
holds. A jump across both thresholds applies independent OFF then ON,
so restoration does not need a later recovery report.

## 7. Adoption and legacy operations

From genesis, managed flags begin ON and journals begin clean. For an
existing execution, first inventory the finite legacy prefix of every
reserved, released, submitted, uncertain and pending operation. This
includes operations whose signatures have not yet been released.
Preserve every issued record's original target, configuration, threshold
and effect. Import used credits conservatively; ambiguity never creates
an unused credit. Register outstanding legacy OFF hazards against the
region's existing and later arriving tasks as in Section 4.

Finite legacy backlog suffices for the **finite count** results below.
It does not alone establish restoration: a legacy OFF may have no ON,
and an attached OFF can overwrite an independent ON in the same prelude.
Restoration is claimed only after explicit reconciliation. Every possible
legacy effect must be canonically consumed or definitively prevented from
first submission; an unresolved released operation stays in the hazard
set. After observing the last effective legacy OFF, run the authorized
corrective independent ON procedure under the same actual-time rule;
restoration requires that ON actually be submitted. For attached
OFF, observation of canonical processing is essential; choosing a
nominally later ON threshold in advance is insufficient. No bounded
legacy cleanup or controller catch-up time is claimed.

The corrective ON does not grant any f, ell, c, regional or task credit.
Optional new leases for the affected region are paused until this
reconciliation completes. Permanent administrative removal is a separate
explicit operation outside that region's managed-service claim. Relabeling
health requests as emergencies or administration to bypass these rules
invalidates the theorem; it is not an uncounted exception.

## 8. Controller guarantees

**Proposition 1 (finite reservations and MSIG effects at fixed G).**
Suppose actual accepted G is fixed at `G*` from real time t0 onward.
Let `h*=h(G*)`, and let the controller's observed accepted height then
be `h0<=h*`. It makes at most `h*-h0+1` new reservations of each of f,
ell and c after t0. If N0 earlier reserved MSIG operations can still act,
including unreleased and legacy operations, the number of effective
MSIG Forces after t0 is at most

`N0 + h* - h0 + 1`.

*Proof.* Its monotone cursor visits only tips on G*'s ancestor chain at
heights h0 through h*. Each tip has one credit in each namespace. Mandatory
scheduling and optional H1 share f; they are not separate allowances.
Catalogue additions, new AUTHs, new beta values, observation retries and
task selection introduce no tip credit. Each exact MSIG identity has at
most one canonical effect, including under retry. Earlier reservations
are in N0 regardless of their release/submission stage. This proves both
the reservation and effect bounds. No unknown controller delay increases
the number of available keys. ∎

**Proposition 2 (finite task-specific OFF interference).**
Fix a registered task tau that remains in managed scope. If it receives
only finitely many accepted discharges, only finitely many health OFF
operations can subsequently affect its physical region. More precisely,
at a time when the controller has registered tau and observed count
`s0`, let its final actual count be `s*>=s0`, and let B0 be the finite
already reserved lease/legacy OFF backlog affecting that region. At most

`B0 + s* - s0 + 1`

such OFF records can occur subsequently. Counts are conservative: a
reserved lease may produce no OFF, and inherited charges can reduce the
number of new reservations further.

*Proof.* Every later new lease reservation must atomically spend
`u_tau[s]` for its then-observed count s. The cursor can see only s0
through s*, each once as a usable key. Every task stays in the check while
parked, pinned or replaced by its next token. Other tasks' DATA, including
new AUTHs of the same physical region, do not change this count. A
registration race is covered by the original reservation or inherited
charge, and existing reservations are in B0. A lease supplies at most one
OFF identity. Therefore neither an unbounded stream of new catalogued
tasks nor progress by another roster renews tau's finite allowance. ∎

**Corollary (eventual governance ON).**
For such a task under the clean policy, or after completed legacy
restoration, its region is eventually permanently governance-ON if
Espresso timestamps grow. Every new OFF has a pre-finalized finite later
independent ON; Proposition 2 leaves finitely many OFFs. The final one is
eventually restored, including on a simultaneous threshold crossing.
This says nothing about the online status of the old voters. Cooldown,
old AUTH weight, and regional service remain separate requirements.

**Proposition 3 (optional H1 cannot starve local scheduling work;
public opportunity after actual submission).**
Suppose a token remains outstanding and eligible after a finite point,
there is no serving pin blocking it, the controller's fixed finite views
and enabled local work complete eventually, and accepted DATA keeps
growing. Optional H1 cannot prevent a mandatory reservation and its
enabled dispatch work. If an equivalent record is already actually
submitted, or one of those mandatory operations is eventually actually
submitted validly before any effective Force, an effective Force
eventually provides a task-selection opportunity.

*Proof.* Suppose no such Force occurs thereafter. After the finite prefix
beta is constant, since REGULAR does not change it. A complete view
eventually exposes Need. If an equivalent actually submitted record is
already pending, its finite activation threshold is eventually crossed;
its target matches, or an earlier effective Force has already provided
the required opportunity. Both contradict the supposition. Otherwise a
spent current f cannot obstruct all later credits: unbounded accepted
DATA gives a later unused observed tip. The first such credit is reserved
for mandatory work before optional H1, and the durable job's enabled
dispatch steps execute eventually. Neither health traffic nor later task
arrivals can replace that job. If an attempt expires, its credit stays
spent and the same argument reserves a later unused tip for the still
pending obligation. This proves local nonstarvation. Under the stated
actual-submission condition, a submitted operation's finite threshold is
crossed under the same constant beta, yielding the required contradiction.
The public conclusion is an opportunity, not a promise that a particular
queued task beats its older eligible predecessors. ∎

At a fixed G, a burned f need not be replenished and the controller does
not manufacture another credit. The runtime's assigned-duty/AUTO argument
handles that case. Once actual valid submission is established,
Proposition 3 closes the infinite successful-REGULAR execution that a
Force-only selector would miss. If admission can fail forever without
violating the frozen Model, that execution remains possible despite fair
local controller work; this note does not claim to have removed it.

**Corollary (the control tail required by prepared historical retries).**
Fix an eligible unserved pin after registration and legacy reconciliation.
If its exact anchor stays G*, Propositions 1 and 2 leave only finitely
many matching MSIG effects, new health OFFs, and CLEARs, counting all
pending and legacy reservations. Their finite scheduled thresholds are
eventually processed if they were actually submitted; operations never
submitted have no public effect. After the resulting finite prefix and
expiry of old cooldowns, the pin is governance-ON and no further health
OFF occurs in that tail. Any unused lease bit need not have been spent;
the claim is finiteness of actual interventions. New catalogue arrivals
cannot replenish the fixed task's lease allowance or add a control
credit or displace this eligible unserved pin. The runtime may therefore
apply its conditional fixed-task preparation and growing-beta AUTO proof
after that prefix, without charging this controller's catalogue scans,
signing, health decisions or catch-up to its regional response bound.

This corollary requires the runtime to establish fixed-anchor pinning,
proper conditional AUTO cooldown, historical service coverage and its
own prepared-response bound. The controller results establish none of
those environmental or source-handler facts, and make no complete
request-liveness or whole-protocol safety claim.

## 9. Source and validation record

The preserved control semantics and credit baseline are in
`sections/rotation_v3_6_control.tex` and
`sections/rotation_v3_5_health.tex`. Actual submission, immutable
authority, advance announcement and the service assumptions are in the
frozen `revisions/v3_1_frozen/model_and_assumptions.tex` (also copied into
the v3.9 review snapshot). The fixed-G proof use is scoped against
`sections/rotation_v3_9_liveness.tex`. The task-specific changes and open
integration dependency are identified in `historical_service_design.md`
and JG2 of `historical_service_fresh_check.md`.

Validation is a source-level construction and invariant/progress
argument. No protocol source was changed, no new control encoding was
introduced, and no simulation, model checking, TeX compilation or
concrete signing/submission implementation was validated. The companion
runtime integration must use the same ScheduleView/Need/pin contract.

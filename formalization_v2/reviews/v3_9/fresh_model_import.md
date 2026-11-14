# Summary

**Final Verdict:** The v3.9 first-public-carrier import proposition is valid under its explicitly stated retained-byte, continued-local-work, and successfully prepared-parent conditions; the reviewed integration preserves the existing abstract complete-handler and DATA-progress dependency boundaries, while full request liveness remains OPEN.

**List of Findings:** No `Critical Error` or `Justification Gap` was found in this scoped model/interface and import proof audit. This is not a verdict that arbitrary-membership request inclusion has been proved, nor an implementation timing certification.

The frozen Model section is literally identical to v3.1. The new behavior is a specified local storage/scheduling duty, not a new publication, cross-roster, historical-query, or native-preparation service premise. The conditional import result does not imply that a suitable read context will be created by a deadline, that import finishes before any report, or that a first request-bearing complete result forms.

**Artifact integrity and scope.** Read only `review_scope.md`, `manifest.json`, and the frozen `snapshot/`, together with the explicitly required proof-verifier skill. No previous reviews or author notes were used. All 24 manifest entries matched before and after review. The main-file SHA-256 is `63dd6770de6aaed0f442c53c6fcd3685fb5a37b478140d67970ee2c4719058cb`. The extracted Model section equals both the v3.1 main-source section and `baseline/model_and_assumptions.tex`, whose SHA-256 is `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`. No source repair, executable model check, codec test, native benchmark, or compilation was performed.

# Detailed Verification Log

Locations below are relative to `snapshot/`; `main` denotes `sync_hotstuff_rr_part2_rotation_v3_9.tex`.

## 1. Frozen assumptions and the declared result

> “The Model section is unchanged from v3.1.” — main:126.
>
> “Import is deferred local work with an explicit continued service rule and no wall-clock bound.” — main:129–130.

**Checked.** A byte comparison, not a visual comparison, establishes the equality of the full Model section at main:228–548 with its frozen baseline. In particular, the publication clock still starts at availability of a complete regional result (main:408–423); Espresso visibility supplies complete records and Espresso evidence but leaves application processing separate (main:463–492); native state and authority preparation remain eventual (main:534–536); and client ingress retains its original wording (main:537–539).

The semantic boundary is also respected. The added algorithm directs a correct node to preserve and process work on bytes it actually holds. It does not enlarge the set of network endpoints covered by the old regional delay bound. The original verification interfaces remain soundness premises, not conclusions of import.

The dependency structure is:

- Original complete public result plus actual local acquisition and source-job service implies candidate-byte retention.
- Candidate retention plus an actually prepared, retained exact-parent read context, successful ordinary checks, and checking-job service implies authenticated pending insertion unless already in `G`.
- Pending status plus exact-parent eligibility and sufficiently many genuine accepted aware reports implies the separate conditional inclusion result.
- Prepared source processing, regional response, original complete-result publication, and the public kernel support DATA progress independently of the import loop.

No step in the last chain depends on finishing import. No step in the first two chains establishes the awareness premise of the third.

## 2. The original publication input and typed public objects

> “The Publish Layer's input remains exactly $R_0$. Erasure removes only the new separately framed sidecars” — `sections/rotation_v3_9_requests.tex`:179–180.
>
> “It is not merely the public kernel.” — the same file:166.

**Checked.** The definition at lines 163–186 retains all original DATA, FINAL, proposal, QC, ancestry, boundary, authority, native-validation, and public-source objects. `R_+` retains the regional REPORT sidecars, while only those new sidecars are erased for publication. The manager offers the complete original publication value at main:1620–1630; retirement preserves both the complete-result offer and the required regional sidecars (`rotation_v3_9_retirement.tex`:135–145). Import does not invoke the delivery guarantee on a bare request, partial FINAL, or kernel-only value.

> “A reference retains its baseline denotation … It is an edge to an object, not the object's bytes.” — `sections/rotation_v3_8_carrier.tex`:31–33.

**Checked.** The carrier definition preserves the old logical fields, exact signed bytes, authenticated source identities, and source denotations. A private cache cannot create a new public denotation. Invalid outer hints and unrelated failed proofs do not hide independently represented typed children. Missing bytes, an unresolved edge, arbitrary ciphertext, or an unspecified private resolver do not supply a request body. The carrier result is expressly logical, with concrete codec and broader-namespace discovery obligations left outside its claim (carrier:150–181). This avoids defining “carrying” by the success of the new extractor.

## 3. Source-store and manager hooks

> “Durably record the retained root and dictionary cut for the independent request import journal … These are descriptors, not an in-handler scan or request check.” — `sections/rotation_v3_9_source_step.tex`:35–39.
>
> “The node-level request owner receives native-preparation and source-store notifications independently of the BFT worker.” — main:1632–1633.

**Checked.** The source-availability path stores the complete raw namespace before cursor advance or input release, including rejected/malformed records. Acquiring an exact public target creates another enqueue obligation. The source reader therefore does not depend on P6 acceptance to preserve the evidence needed by import. Initial retained content receives one finite cut, and concurrent acquisitions append separate jobs (`rotation_v3_9_public_requests.tex`:23–31).

The actual SOURCESTEP send records its descriptor before its P6 result and explicitly does not wait for native preparation or the import worker (`rotation_v3_9_source_step.tex`:149–158). The manager's `NativeParentPrepared` event records fixed checking work without scanning (main:1601–1603). These are operative hooks, not merely promises in the proposition's proof.

The original source-availability, interpreter, regional-entry, and new import cursors remain separate. Checkpoint advancement or P6 rejection cannot mark unfinished request work complete. Missing historical bytes remain a source/hydration task under the existing source contract, not work silently inserted into a timed import or SOURCESTEP handler.

## 4. Lifecycle ownership and retirement

> “The journals, bytes, read contexts and pending enqueue obligations belong to the node, independently of a producing engine or import worker.” — `sections/rotation_v3_9_public_requests.tex`:109–110.

**Checked.** Lines 111–117 preserve these obligations through Waiting, Frozen, Completed, Terminated, RequestClose, Closed, checkpoint compaction, and advancement of `G`. Worker exit transfers exact cursors and evidence durably. The source-store restart rule restores import ownership/cursors (`rotation_v3_9_source_step.tex`:48–55), and FinishClose requires a persistent owner for public-import journals, fixed native read contexts, and pending-admission work before release (`rotation_v3_9_retirement.tex`:140–159).

Transfer need not wait for the queue to empty, obtain an acknowledgement from a new roster, or finish snapshot copying. The receiving owner must actually possess the retained material before the old owner releases it. The node-level import/relay duty can therefore outlive a BFT worker without reopening that worker's signing or entry gates.

## 5. Candidate extraction, malformed material, and cycles

> “it extracts the actual entries of $\mathit{batch}$ only when the local structure identifies a DATA body and supplies those entry bytes.” — `sections/rotation_v3_9_public_requests.tex`:40–41.
>
> “It saves an unresolved edge without waiting for or fetching its target; a later acquisition supplies another job.” — the same file:48–50.

**Checked.** The extraction rule follows the original tuple's batch position. DRAIN, request hashes, arbitrary witness strings, and sender object indexes do not produce candidates. Wrong outer domain/version/base hints or invalid enclosing proofs do not suppress a separately represented DATA body; the DATA's own signed domain is not used to certify its requests. These bytes remain unvalidated candidates until the ordinary request-domain and authentication checks.

Each source job has a fixed dictionary cut and visits each actual object once. Thus reference cycles cannot turn one job into an infinite traversal, and new acquisitions cannot expand that job's input indefinitely. A target acquired after the cut creates a new source job, so saving an unresolved edge does not permanently lose a later supplied request. No job obtains AUTH, native state, missing source history, or optional proof targets. The proposition's acquisition hypothesis is essential when the old logical carrier uses a reference.

## 6. Exact-byte deduplication and authenticated pending semantics

> “A claimed client identifier creates no duplicate marker. Invalid encodings and failed authentication cannot reserve an identifier or suppress a later valid request.” — `sections/rotation_v3_9_public_requests.tex`:60–63.

**Checked.** Before validation, deduplication uses exact byte equality or a binding content commitment and preserves source occurrences. Failed checks are keyed to those bytes and that parent, not to an asserted identity (lines 82–85). Consequently an adversary's malformed request asserting the same client identifier as a later genuine request cannot poison that genuine request's insertion or checking. Identical-byte deduplication is safe because the bytes already exist in the retained candidate store.

Only ordinary successful chain/domain, authentication, canonicalization, and application-admission checks establish the request's real identity. Successful retransmissions merge by that validated identity; a new entry obtains the next pending FIFO position, an existing entry retains its position, and an identity already in `G` creates no active pending entry (lines 86–94). Retention of an unvalidated candidate is not represented as authenticated first receipt. This preserves the stated ordinary pending FIFO semantics rather than assigning executable status to raw byte arrival.

## 7. Immutable native contexts and local work

> “Each job retains the already available read context needed by the ordinary finite request check, including its exact native state, until the check finishes.” — `sections/rotation_v3_9_public_requests.tex`:71–73.
>
> “Any required snapshot construction must actually finish before this notification” — the same file:75–76.

**Checked.** The code-level interface is abstract, but its logical contract is precise: a prepared-parent event means the immutable read inputs actually exist. A pointer to an engine state that later mutates is explicitly insufficient. Parent changes neither retarget nor cancel an old job; a successful old-parent check never grants admission at a newer parent. Snapshot construction, state reconstruction, and fetching absent validation inputs are not operations of the checking job. Ready does not wait for snapshot construction for import.

This is a conditional use of an available native context, not a proof that the original native adapter can manufacture persistent contexts instantly or within any new bound. An implementation which never provides such a context cannot invoke the promotion conclusion. That limitation is present in the statement and interface, rather than an omitted assumption.

> “The node continues these steps until each queued finite job is complete; session changes and repeated new arrivals cannot suspend it indefinitely.” — the same file:100–102.

**Checked.** Continued local work is expressly a protocol duty and is also a hypothesis of the proposition. Alternation gives progress to both journals; FIFO order prevents later jobs from overtaking a fixed job; finite local substeps and saved cursors prevent restart starvation; neither queue waits for a fetch or for the other queue to empty. For a fixed job with finitely many predecessors, induction over those predecessors and its finite steps proves eventual completion. No external fair scheduler, network service, throughput bound, finite-storage bound, or deadline is silently inferred. The proof would not hold under an implementation that indefinitely starves this prescribed loop, but that execution violates the stated local duty.

## 8. Exact-parent reports and the old complete-handler timing scope

> “An unvalidated entry of $U_v$ is not yet a pending request; deferred checking is not permission to omit an admitted pending request from $E_v(P)$.” — `sections/rotation_v3_9_public_requests.tex`:121–123.

**Checked.** `E_v(P)` continues to be the eligible projection of authenticated pending requests against the completely validated actual parent, exact native state, and full ancestry. The inherited global prefix and speculative ancestors are both included. REPORT emission and FIFO construction cannot reuse an import check at another parent; later batch entries also see earlier batch effects. Later imports do not amend immutable reports (public requests:119–132; requests:60–79, 91–108).

Source, P6, Ready, timers, and report emission explicitly do not wait for import backlog or a watermark (public requests:104–107). Only descriptors are added to their handlers. The already-required exact-parent eligibility processing remains inside the original completed request/parent/batch interface; the text does not reclassify an unfinished admitted-pending scan as an EMPTY report. Thus no extra scan or snapshot-copy delay is smuggled into the old `4 Delta`/`6 Delta` report deadline argument.

The preservation conclusion remains abstract and conditional. The ReadyBundle contract explicitly withholds its `e + Delta` conclusion when residual native installation remains outside bounded message processing (`rotation_v3_5_ready_bundle.tex`:107–146). The SOURCESTEP timing section likewise excludes arbitrary CPU work, archive catch-up, and database import (`rotation_v3_9_source_step.tex`:243–257). v3.9 has not supplied measurements demonstrating those original contracts for a concrete adapter; neither does its import result claim to do so.

## 9. The new proposition, step by step

> “If a correct node follows and retains those actual bytes and continues the local loop, it eventually retains $x$ in $U_v$” — `sections/rotation_v3_9_public_requests.tex`:138–140.

**Valid.** Actual root/target acquisition creates a durable source job. Finite predecessors, finite fixed traversal, and continued FIFO work reach the actual DATA batch position. Persistence precedes advancing that position. Exact-byte deduplication either stores those bytes or identifies their retained copy. Rejection and closure are not extraction gates. A still-unresolved reference alone does not satisfy the actual-acquisition hypothesis.

> “If … an exact parent at which $x$ passes the ordinary request checks is completely prepared … the request eventually enters the pending store unless its identity has already entered $G$.” — the same file:141–144.

**Valid.** If candidate arrival precedes preparation, the preparation event captures the candidate in its finite current cut. If preparation precedes candidate arrival, the arrival event schedules that candidate against the available prepared context. The fixed context prevents a moving-parent retry obligation. FIFO completion then runs the hypothesized successful finite check and performs the durable pending update. Validated-identity deduplication can only merge an existing pending entry or record already-global inclusion, which are the specified outcomes. The proof does not rely on the containing DATA being accepted or on the branch remaining open.

## 10. Global progress, protocol authority, and the open request objective

> “Source visibility alone is not that promotion, and no import watermark gates Ready or delays the DATA bounds above.” — `sections/rotation_v3_9_liveness.tex`:492–493.

**Checked.** The existing prepared-response chain still obtains the original complete regional result, uses its first original logical carrier, applies the public FINAL kernel, and witnesses `G` growth or closure (`rotation_v3_9_liveness.tex`:243–292). The new request-import queue is not used as an intermediate bound in that chain. Continuing global DATA progress and repeated accepted regional service remain distinct from request inclusion; empty DATA can still discharge a regional service obligation (liveness:440–444).

The protocol labels remain the original six. Import cannot write `G`, change a lock, sign a vote, issue a report for an exited engine, or authorize entry. Protocols 5/6 retain public authority (main:1697–1702, 2205–2235). The Force prelude captures its entry version, checks matching MSIGs against it, prioritizes MSIG over AUTO, preserves all pre-payload `G`, increments beta once, and rejects new-version DATA in the triggering block (`rotation_v3_6_control.tex`:207–249). Reading a request from rejected old DATA does not reaccept that DATA or weaken this cutoff.

> “The proposition neither forms the first $x$-bearing $R_0$ nor ensures that every future roster knows $x$ before its first DATA.” — `sections/rotation_v3_9_public_requests.tex`:170–172.

**Correct limitation, not a defect in the conditional proposition.** Same-snapshot full-request relay still applies after successful admission, including after worker exit. Its existing regional bound does not supply an old-only to new-only communication guarantee, arbitrary backlog dispatch, or a new publication path. The v3.8 accepted-awareness theorem still needs a genuine aware correct signer in infinitely many accepted first-DATA witnesses. First-carrier formation and that awareness property are not established by public import. The separate ownership/hydration proposal is not enabled, and full request liveness remains explicitly OPEN (main:130–133; liveness:466–504). This review does not substitute the valid conditional import theorem for that unresolved objective.

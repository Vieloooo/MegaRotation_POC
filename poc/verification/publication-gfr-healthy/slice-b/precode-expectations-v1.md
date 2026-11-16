# Slice B independent pre-code boundaries

2026-11-16. Derived from the sealed32 source/additive cases and full dispatch `d55d39f73593676325bdc0fbb7ba3845ca93025f`, then compared with H3-B-1. No B implementation inspected or executed. Actual H1 root/aux receipts remain an integration gate; they do not block pure plan/frame/custody code.

The dispatch is authoritative: H1 owns real durable commits and terminal guards; H2 emits ManagerDelta/actions; `Committed(actions_pending)` accepts the complete legal record and close obligations before successor Ready material, while `ActionsApplied` remains separate. H3 does not issue ReadyToken. Preparation-era all-or-nothing assignment descriptions cannot delay durable close/tombstone.

| ID | Trigger | Source-derived expected result |
| --- | --- | --- |
| IB01 | Two distinct lifecycle members at one pc; exact replay and conflicting record at same pc | Retain full record/effects/ordered LIFE identities; exact repeat idempotent, changed record conflicts. Sealed ReplayState is required; mutable caller record is not source authentication. |
| IB02 | plan→wrong/stale/cross-instance receipt versus matching synthetic commit port | No state change before valid owner receipt; exact candidate bytes/identity/revision bound. Synthetic port success is a component result, never real root custody evidence. |
| IB03 | Public close before first Offer; late new key; stop after PREPARED and before actual commit | Durable scope blocks new release; unrelated/local TIME/Pause/ClockUnsafe does not stop publication; root-committed historical submission remains in flight. |
| IB04 | Ledger scope committed but06 task update fails | Dispatch remains denied by durable scope; pending06 update retained; no rollback of exclusion. |
| IB05 | Watermark deadline−1/=deadline; header-only or different reader/source | Only actual matching complete prefix at equality authorizes deadline stop; local/wall time and header alone do not. |
| IB06 | One complete bounded attempt, proof/body scheduled at different q; duplicate/conflicting admission | Preserve AttemptId and original ordinals/bytes; replay same intent idempotent; same ID/different bytes rejected. q1 public-incomplete is distinct from local source gap. |
| IB07 | H3TF lengths61440/61441/1048576/1048577; reduced payload limits83/84 | Exact canonical segmentation; every frame includes83-byte header and fits actual limit. Oversize/profile exhaustion rejected before allocation. Limit83 leaves no data capacity. Limit84 permits at most1024-byte objects under1024-frame cap. |
| IB08 | Missing/reverse/duplicate/conflicting chunks and cross-binding/run/incarnation splice | No partial public delivery; exact duplicate idempotent; conflict rejects without altering committed prefix; valid completion byte-identical once. |
| IB09 | Third incomplete1MiB destination object, fifth global object,4097th retained completion | Explicit capacity/backpressure; no eviction or forged acknowledgement. Correct retry retains original custody obligation. |
| IB10 | Fifteen-recipient fanout, unrelated02 traffic, missing real ACK | Grant uses at most4 frames/262144 bytes and2 recipients within actual shared credits; no reserved4MiB assumption, fake credit release or bypass. |

Header arithmetic is unambiguous: `4+2+1+32+32+4+2+2+4=83`. At65536 payload limit, chunk cap61440 means1MiB uses17 full chunks plus4096,18 frames total; framed bytes1050070. Four complete copies use4200280>4194304. Actual grant/copy credits must include duplication and other traffic.

Pre-freeze clarifications requested: exact H3/Record/v1 hash part boundaries and raw/canonical field widths; exact submission/finalized binding_id derivation; explicit zero-length-object handling. These affect reproducible identity/goldens, not permission to work on unrelated B logic. New wire literals use fixed caller binding IDs until the binding recipe is frozen. No protocol or durability behavior is PASS here.

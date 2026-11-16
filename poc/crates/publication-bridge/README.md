# M1 publication bridge

`BridgeResult::new` wraps a `poc_types::domain::RegionalResult` in the bounded H3-M1-SMALL-1 profile. `BridgeStateCodec` persists the private `H3RS` v1 envelope with existing canonical nested objects. Empty proof and chain material can be retained while an Offer is waiting. An encodable complete chain starts after the descriptor anchor, ends at the exact target, and includes the target once.

Construct `NativePublicationVerifier` from the public configuration, Native verification profile, GFR parameters and Espresso source ID. Pass the verifier and `BridgeStateCodec` to `poc_publisher::Publisher::create` or `Publisher::open`. Supply `BridgeInputs` on each observation with the matching `VerifierBinding`, a historical `ContextVerifier`, and the L1 and DEP readers. The context and basis providers supply historical verification material. The adapter checks the supplied proof, then verifies complete ancestry, metadata, DEP and execution using `poc-core` Merge. Publication emits the ordered bodies and final proof via GFR-PUBLIC-1.

For a receipt, pass the reader's `VerifiedPrefix` to `Publisher::confirm_recorded`. The adapter and GFR replay share the completeness evaluator. Earliest complete q includes all required bodies. The receipt retains the earliest valid proof position and the actual checked prefix tip. GFR applies selection and timing at its own admission step.

`Publisher::prepare_attempt` creates a durable intent for the controlled publication path. The bridge performs historical checks through injected capabilities. The component tests use an explicitly synthetic exact-byte finality verifier. The real H1 certificate and historical provider checks remain open.

See the [Slice A contract](../../docs/contracts/h3-publication-gfr-slice-a-v1.md) for limits, framing and diagnostic codes.

## Controlled publication plans

Capture a full `RecordDelivery` by ordinal from sealed GFR replay. Its ordered
lifecycle identities accompany the record into the Manager handoff. The H1
owner commits that handoff before any external delivery. `StopLedger` retains
public scopes and verified deadline facts in bounded candidate snapshots.
Use the matching owner receipt to confirm or restore a snapshot. The Publisher
actor checks retained scopes even for late Offers and rechecks `permit_dispatch`
before releasing a prepared intent. A failed Publisher stop update leaves the
committed stop in effect and its projection available to retry.

`AttemptCustody` validates a controlled submission through `SubmissionVerifier`
and returns plans with original attempt IDs and message ordinals. `Propose`
supports a split publication schedule. Controller receipts commit custody and
finalization plans. The reader establishes Recorded independently from public
evidence. Replaying an already committed custody operation returns
`AlreadyCommitted`.

`fragment` and `Reassembler` carry exact submission or finalized-block bytes.
Register the expected route and object binding, and supply a verifier for actual
route custody. `CreditWindow` plans a bounded batch of `ServiceEffect::Send`
intents for one sender. The controller checks shared capacity, route copies,
charges and releases through `CreditVerifier`.

The [Slice B contract](../../docs/contracts/h3-publication-gfr-slice-b-v1.md)
defines the record and transport encodings and the synthetic and runtime gates.
Local tests inject explicitly synthetic owner receipts and delivery attestations.

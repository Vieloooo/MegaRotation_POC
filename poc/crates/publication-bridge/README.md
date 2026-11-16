# M1 publication bridge

`BridgeResult::new` wraps a `poc_types::domain::RegionalResult` in the bounded H3-M1-SMALL-1 profile. `BridgeStateCodec` persists the private `H3RS` v1 envelope with existing canonical nested objects. Empty proof and chain material can be retained while an Offer is waiting. An encodable complete chain starts after the descriptor anchor, ends at the exact target, and includes the target once.

Construct `NativePublicationVerifier` from the public configuration, Native verification profile, GFR parameters and Espresso source ID. Pass the verifier and `BridgeStateCodec` to `poc_publisher::Publisher::create` or `Publisher::open`. Supply `BridgeInputs` on each observation with the matching `VerifierBinding`, a historical `ContextVerifier`, and the L1 and DEP readers. The context and basis providers supply historical verification material. The adapter checks the supplied proof, then verifies complete ancestry, metadata, DEP and execution using `poc-core` Merge. Publication emits the ordered bodies and final proof via GFR-PUBLIC-1.

For a receipt, pass the reader's `VerifiedPrefix` to `Publisher::confirm_recorded`. The adapter and GFR replay share the completeness evaluator. Earliest complete q includes all required bodies. The receipt retains the earliest valid proof position and the actual checked prefix tip. GFR applies selection and timing at its own admission step.

`Publisher::prepare_attempt` creates a durable intent for the controlled publication path. The bridge performs historical checks through injected capabilities. The component tests use an explicitly synthetic exact-byte finality verifier. The real H1 certificate and historical provider checks remain open.

See the [Slice A contract](../../docs/contracts/h3-publication-gfr-slice-a-v1.md) for limits, framing and diagnostic codes.

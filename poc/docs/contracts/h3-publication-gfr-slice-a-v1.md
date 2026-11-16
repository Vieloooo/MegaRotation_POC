# H3 Slice A — shared completeness and Publisher adapter

2026-11-16. ID **H3-A-1**. **FROZEN for implementation by lead after source-first independent review; requested chain clarification resolved.** Parent authority and exact M0 baseline: [implementation contract](h3-publication-gfr-implementation-v1.md). This slice can be implemented using delivered03/06/07 traits now. Real H1 evidence remains an acceptance gate, not a blanket implementation stop.

## 1. Existing sources and allowed change

S: v10 04 §§3.2–3.5/PUB-1–3 permits complete-result Offer and public receipt before G selection;09 §§3.2–3.4,4.2 defines earliest valid proof+physical ancestry completeness separately from local delivery, responsible cid, cutoff and expiry;02 §4.5 makes historical verification independent of live engine/local clock. A: GFR-07-v1 §§3,5,7 supplies Native profile/basis, GFR-PUBLIC-1 and canonical proof-location handling;06 contract §§4.1–4.4 supplies generic Publisher, private StateCodec and receipt semantics. Reuse these sources and the frozen03 object codecs.

Single Builder may create `poc/crates/publication-bridge/` as independent workspace (`poc-publication-bridge`, library `poc_publication_bridge`) and extract necessary shared logic in `poc/crates/viewer-gfr/`. No06/root types/root manifest/H1/H2 edits. Public GFR-PUBLIC-1/POS-1/LIFE-1 bytes and old07 replay behavior stay unchanged. B's durable stop/controlled output and C's official input are separate; A exposes no uncontrolled submit method.

## 2. Compiler-facing07 helper

Export these names from `poc_viewer_gfr`; reuse existing types and derive Clone/Debug/PartialEq/Eq for new owned values. Lifetimes may be stated equivalently.

```rust
pub struct PublicCompleteness {
    pub key: ResultKey,
    pub first_complete_q: u64,
    pub publication_position: sim_espresso::EspressoPosition,
    pub physical_chain: Vec<PhysicalBlock>, // anchor-exclusive, target-inclusive
    pub valid_proof_locations: Vec<sim_espresso::EspressoPosition>,
    pub body_locations: Vec<(Hash32, Vec<sim_espresso::EspressoPosition>)>,
}
pub enum CompletenessInvalid { Ancestry, FinalProof }
pub enum CompletenessOutcome {
    Complete(PublicCompleteness),
    Incomplete,
    Wait(Missing),
    Invalid(CompletenessInvalid),
    Reject(GfrError),
}
pub fn public_completeness(
    prefix: &sim_espresso::VerifiedPrefix<'_>, through_q: u64,
    config: &PublicConfig, expected: &VerificationProfile,
    descriptor: &Descriptor, target: Hash32,
    finality: Option<&VerifierBinding<'_>>,
) -> CompletenessOutcome;
```

The public wrapper accepts only06's sealed `VerifiedPrefix`, never a caller-mutable `PublicationIndex`. A cut beyond `prefix.tip().0` rejects with existing `GfrError::HistoricalBound`; no future object beyond through_q participates. Build/check the canonical index from the prefix's actual messages through that cut. Preserve ignored unknown/malformed publication diagnostics and reject invalid source envelopes according to existing07. The crate-private common evaluator is reused by GFR's trusted staged index and this wrapper; do not duplicate ancestry/q_avail logic in bridge. Its internal interface is Builder-owned, provided both paths demonstrably call it.

Validate configuration/expected profile and supplied binding equality using existing07 checks. Absence of a binding does not erase the expected profile. Before invoking `FinalVerifier::check`, require actual target body and `VerificationBasis::require`. Missing historical inputs→Wait, known invalid inputs→Reject/Invalid. Never construct a dummy body. Recompute proof checks against this exact descriptor/profile/basis; do not trust caller-mutated or fixture-cached `ProofVerdict::Valid`. No production fallback;07 may still use its explicit existing fixture profiles for old pure-rule tests.

Complete requires at least one actual valid target proof and all hash-linked actual physical bodies from descriptor anchor through target, with existing07 structural chain/window/timestamp checks retained. No proof/body in complete public prefix→Incomplete; present but invalid ancestry→Invalid(Ancestry); complete ancestry with matching proofs all checked false→Invalid(FinalProof). Invalid(FinalProof) does not permanently exclude a ResultKey from later valid proof variants. Missing local verification input stays Wait. Existing GFR diagnostic/proof-verdict projection must remain stable after extraction.

Compute `first_complete_q=max(earliest valid proof q, latest earliest body q)`. `publication_position` is least `(q,index)` among valid target-proof occurrences within cut, which may precede completeness. Locations sorted/deduped by exact position; retain different proof variants and public duplicate occurrences. Bodies/digests are in anchor-to-target order. Optional anchor provenance and local receive ticks do not enter this computation. A candidate need not hold NEXT to be complete. GFR alone applies selection/open-scope, Merge, cutoff and interval expiry; Publisher separately verifies Offer/context/Merge. This helper does not assign a descriptor, alter G or turn a public receipt into G acceptance.

## 3. Bounded M1 result and canonical bytes

A profile **H3-M1-SMALL-1** permits at most4 valid03 Transfer transactions per physical body, at most10 physical bodies including target, proof≤32,768 bytes, descriptor canonical bytes≤16,384, and at most16 anchor source objects each≤4,096 bytes (total≤65,536). The actual healthy workload must change execution state; this is checked against execution evidence, not inferred merely from nonempty bytes. These are H3 input limits, not changes to03 validity.

`RegionalResult.chain` is anchor-exclusive and target-inclusive and, when complete, its final block must equal `RegionalResult.block` exactly (v10 02 §3.2, source line135;03 contract-v2 object table, source line80). Frozen public order: each physical body in that chain order, then one FinalProof for the supplied target. Use07 `encode_publication` exclusively. Target occurs exactly once; mismatched tail or duplicate target is rejected rather than normalized, while an explicitly incomplete/empty chain may remain Verifying and cannot produce publication messages. Same ResultKey reoffer retains06's first validated immutable payload; different legitimate proof bytes do not create another task.

03 v1 PhysicalBlock encodes `2+32+24+32+4+N*(4+19)=94+23N` bytes, so N≤4 gives≤186; a GFR Body is≤197. A maximal admitted FinalProof is `4+2+1+4+16384+32+4+32768=49,199` bytes. With source_id≤256, one ten-body attempt is bounded by `47+256+10*(8+197)+(8+49199)=51,560` encoded06 bytes. These sums correct the pre-code four-byte undercount recorded in the independent bounds correction; the profile fields/encoding are unchanged. Every actual construction must still call the existing03/07 encoders and06 list validator; no unchecked estimate substitutes for validation. Full attempts remain one immutable payload list even when B later schedules different messages at different public q.

Private **H3-RESULT-STORE-1** encoding: ASCII `H3RS`, u16BE1; `frame(descriptor.encode_checked())`, `frame(block.encode_checked())`, `frame(final_proof)`, u32BE chain count then each framed canonical PhysicalBlock; anchor source tag0 Genesis or1 Regional then `frame(existing03 Canonical::encode(source_ref))`; u32BE source-object count then each frame. `frame=x.len:u32BE || x`. Genesis source_ref is ConfigRef; Regional is ResultKey. Force/Resume source variants are explicit unsupported-result errors in this M1 profile. Preserve exact proof/source-object bytes. Trailing bytes, invalid nested encoding, unsupported version/tag, oversized length/count or key mismatch reject. Maximum private container131,072 bytes; current bound is at most116,935 bytes even retaining target separately. Bounds checked before allocation; checked arithmetic throughout. This is a private durable container, not another publication encoding.

```rust
pub struct BridgeResult { /* private validated bounds + immutable RegionalResult */ }
impl BridgeResult {
    pub fn new(result: poc_types::domain::RegionalResult) -> Result<Self, BridgeError>;
    pub fn result(&self) -> &poc_types::domain::RegionalResult;
    pub fn key(&self) -> ResultKey;
    pub fn encode_store(&self) -> Vec<u8>;
    pub fn decode_store(bytes: &[u8]) -> Result<Self, BridgeError>;
}
pub struct BridgeStateCodec;
// Implements06 StateCodec<ResultKey, BridgeResult>; keys use03 Canonical.
pub fn publication_messages(result: &BridgeResult, source_id: &[u8])
    -> Result<Vec<sim_espresso::LogMessage>, BridgeError>;
pub enum BridgeError {
    InvalidConfig, NonNativeProfile, Capacity, InvalidEncoding,
    InvalidResult, SourceMismatch,
    Gfr(poc_viewer_gfr::GfrError), Input(poc_types::domain::InputError),
}
```

BridgeResult construction proves bounds/encoding/key consistency, not finality or verified queue eligibility. It can retain an empty/missing proof or incomplete chain while waiting, provided a real target body exists. A proof-only H1 observation without target body is kept outside this type. `publication_messages` requires complete ordered structural chain and proof present; only `verify_offer` may authorize release after cryptographic/context/execution checks. BridgeResult private fields prevent infallible06 encode methods from encountering an unchecked oversized object; codec decode maps invalid key/result to existing06 CodecError.

## 4. Compiler-facing Native Publisher boundary

```rust
pub trait ContextVerifier {
    fn verify_context(&self, config: &poc_types::domain::PublicConfig,
        result: &poc_types::domain::RegionalResult)
        -> poc_types::domain::InputStatus<poc_types::domain::ExecutionAnchor>;
}
pub struct BridgeInputs<'a> {
    pub finality: Option<poc_viewer_gfr::VerifierBinding<'a>>,
    pub context: &'a dyn ContextVerifier,
    pub l1: &'a dyn poc_types::domain::FinalizedL1Reader,
    pub dep: &'a dyn poc_core::fixture::DepProvider,
}
pub struct NativePublicationVerifier<'a> { /* owned immutable config/profile + lifetime marker */ }
impl<'a> NativePublicationVerifier<'a> {
    pub fn new(config: poc_types::domain::PublicConfig,
        expected: poc_viewer_gfr::VerificationProfile,
        parameters: poc_viewer_gfr::GfrParameters, source_id: Vec<u8>)
        -> Result<Self, BridgeError>;
}
// Implements06 PublicationVerifier:
// Key=ResultKey, Result=BridgeResult, PublicInputs=BridgeInputs<'a>.
```

This H3 ContextVerifier is a narrow injected validation capability, not a copied H1 port or worker ReadyToken. Its Ready output certifies historical Plan/config/anchor provenance and exact execution anchor; H3 independently checks returned anchor identity/state structure and invokes03 RegionalMetadata+Merge over the supplied physical chain/L1/DEP. Missing material returns Wait; malformed/conflicting material Reject. H1's eventual reviewed provider supplies actual provenance validation. No default, permissive or bool-based implementation is exported.

Constructor validates03 config, positive07 parameters, source length1..256 and `expected=Native` compatible with existing03 versions; NonCryptographicFixture is rejected. Existing03 presently validates rule/verifier version1; a different H1 version needs coordinator-owned shared delta, not a guessed number. The same actual reviewed FinalVerifier+VerificationBasis must be supplied to Publisher and GFR. The Native tag is provenance binding, not proof of a trait implementation's cryptographic behavior; synthetic tests explicitly identify their injected stub and cannot claim native PASS.

`verify_offer`: validate exact key/config/descriptor/profile; use context/source capability, exact actual proof through basis+checker, full chain and03 Merge. Incomplete materials→06 Wait; known contradiction→06 Reject. Verified output contains existing encoded messages, checked `deadline=l2_timestamp_us(h_max+1)+grace_us`, and nonempty bounded reproducible validation-basis reference. Profile/version mismatch never degrades to WAIT. No local clock/lease/Selected gate enters historical checking. Preserve06 reoffer and durable task rules.

`verify_recorded`: require matching source identity;06 Publisher separately checks reader identity. Invoke shared07 `public_completeness` on the actual checked prefix, exact task descriptor/target and expected Native binding. Complete returns existing06 RecordedAt with helper's earliest proof publication_position, `complete_through_q=prefix.tip().0`, source and traceable basis. It may succeed before selection or after public cutoff/stop; neither bypasses GFR admission. Incomplete→NotYetPublic; missing validation input→Wait; known bad source/profile→Reject. A later invalid proof cannot poison an earlier valid proof; actual public bytes are rechecked rather than trusting the task's local payload alone.

Freeze06 diagnostic codes for this adapter: missing1 verifier,2 historical basis,3 context/anchor,4 body/ancestry/L1/DEP; rejection101 config/profile,102 encoding/capacity,103 context/anchor,104 final proof,105 ancestry/execution,106 source. References are deterministic bounded≤65,536 bytes, retaining exact source/input identity or a digest plus resolvable audit record; never only `true`/empty. Full errors remain inspectable in the H3 layer through BridgeError. No submit/network function is part of A; `prepare_attempt` still returns06 durable intent only, with actual controlled release assigned to B/H1.

## 5. Immediate independent cases and local gates

| ID | Trigger / guard | Expected |
| --- | --- | --- |
| A01 | Genuine/synthetic-labeled complete prefix: proofq1, bodyq2; query cut1/cut2 | Incomplete then Complete at2 with proof positionq1; no NEXT requirement. |
| A02 | Reverse local delivery, future downloaded objects beyond explicitcut; incomplete earlier source block | Future cut excluded;06 prefix unavailable until actual hole fills; no fake public absence. |
| A03 | Multiple proof variants, duplicate publication, invalid proof then valid; altered mutable PublicationIndex in test | Earliest valid proof/body completeness; caller index/verdict is not an accepted public input; no cached fixture verdict trust. |
| A04 | No binding, missing basis, wrong expected profile, body absent | Explicit Wait when verification is required; profile mismatch Reject; no dummy body/bool fallback. |
| A05 | Bad ancestry/body/target, local chain root mismatch, missing context/L1/DEP | Known invalid Reject/Invalid; genuinely missing inputs Wait; no verified queue. |
| A06 | Small profile N4/N5, chain10/11, proof32768/32769, descriptor/anchor-store limits, truncated/trailing/private badversion | Bound equality passes structural codec where otherwise valid; +1 rejects before allocation/attempt. Exact nested bytes preserved. |
| A07 | Full Offer→durable06 task→same-key reoffer with alternate proof→actual prefix receipt | First validated bytes retained; one task; prefix receipt succeeds before G selection. ACK/header alone never Recorded. |
| A08 | Existing07 replay/codec regressions, invalid proof later repaired, same-q repeated regular | Canonical public output/old source rules unchanged; shared helper used on both paths. |
| A09 | Expected fixture profile passed to production constructor; fixture checkpoint/cache relabeled | Constructor rejects; native state comes from genuine checked genesis/replay and native revalidation, no profile relabel. |
| A10 | Later actual H1 n5 real certificates3/4, wrong-domain/target/kind, historical proof after Close/ClockUnsafe | Only valid4 passes; historical proof remains valid without live engine/current lease. Real gate OPEN until H1 revision/bytes supplied; synthetic tests do not close it. |

Verifier supplies independent inputs/expectations before Builder inspection; Builder then implements this slice immediately. Lead freezes local names/bytes/guards after that review; concrete H1 proof/provider delivery is only A10/native-integration gate. No tests have been executed merely by publishing this annex.

# Slice A pre-code contract review

**PASS — sufficient to freeze slice A and start its single Builder.** No actual implementation or cryptographic PASS is claimed. Ten independent cases remain NOT_RUN; real H1 evidence and the full healthy milestone remain open. Unknown H1 runtime APIs are not a blocker for these existing injected interfaces.

Review baseline `a1bee707b92b6b550a1b53c914e667994db15c84`; full dispatch `d55d39f73593676325bdc0fbb7ba3845ca93025f` was read. Source expectations and vectors were saved before reading the new slice contract. Their hashes/accepted-component pins are sealed in `expectations-freeze-v1.json`; no Builder candidate has been inspected.

Reviewed annex `poc/docs/contracts/h3-publication-gfr-slice-a-v1.md` SHA-256 `8217b3fad7d3b99f619461438ef7603382d13a73e0fbc8567c1370fd5bd270b3`; parent implementation contract SHA-256 `71bc98ae4c3c9f8cecb6e9a8e1faa0f842dec34709c92f02327b6f56edc503a5`. Lead may change freeze/status text without changing the reviewed semantics; final status hashes are recorded separately when available.

## Findings resolved before freeze

| Source / location | Concern and impact | Resolution / evidence |
| --- | --- | --- |
| v10 09 §§3.2/3.4; annex§2 line40 | Public mutable index/cached Valid flags could admit future data or fake proof verdicts into a historical query. | Helper accepts sealed06 VerifiedPrefix plus explicit through_q, rejects cut beyond tip, recomputes checks from actual bytes through that cut, and shares the private evaluator with GFR. Independent A02–A04 expectations cover this. |
| Frozen07 §§3/9; annex§2 line42 and§4 | Requiring Native unconditionally in the shared07 helper would break intentionally tagged old pure fixtures; permitting fixture fallback in production would falsely close the native gate. | Existing07 explicit profiles remain supported; production NativePublicationVerifier rejects fixture profiles and requires exact expected binding/basis. Synthetic injections use the same boundary but are explicitly noncrypto. |
| Frozen06 StateCodec; annex§3 lines56–78 | Infallible encode_result over an unchecked public RegionalResult could panic, allocate without bound or persist an invalid sentinel. | Private immutable BridgeResult proves structural encoding/bounds before infallible encoding. Partial material can roundtrip, while full Offer remains a separate semantic check. No production fixture implementation is exported. |
| v10 02 §3.2; accepted03 RegionalResult/Merge convention; annex§3 line52 | Ambiguous inclusion of anchor/target could duplicate target or omit it on the Publisher path. | Explicit anchor-exclusive, target-inclusive chain; final canonical block equals result.block exactly, target occurs once. Mismatch/duplicate rejects; incomplete/empty material stays Verifying and cannot emit publication messages. |
| v10 04 §3.5/PUB-2 vs09 §4.2; annex§4 line111 | Receipt could be improperly gated by current NEXT or confused with earliest completeness/G acceptance. | Recorded invokes the shared helper independent of selection/expiry, uses first valid proof position and actual prefix.tip for complete_through_q. q1 proof/q2 body gives distinct position/through values. |

Independent arithmetic confirms the small profile: PhysicalBlock max186; GFR Body197; FinalProof49195; one ten-body attempt with256-byte source51556; private maximum116935<131072, including target stored both separately and once in the chain. These are arithmetic expectations only. Actual encoders/list validators must still execute on the candidate; no arithmetic result proves a real result/proof or end-to-end transport.

## Matrix mapping and immediate verification scope

The independent files and annex both use local A-numbering; they are not identical labels. Use this mapping rather than merging their meanings:

| Independent case | Annex counterpart |
| --- | --- |
| A01 unselected Recorded | Annex A07 |
| A02 cut/proof/body | Annex A01/A02 |
| A03 proof variants | Annex A03 |
| A04 capability/profile guards | Annex A04/A09 |
| A05 invalid/missing materials | Annex A05 |
| A06 durable Offer/reoffer | Annex A07 |
| A07 split-q receipt | Annex A01/A07 |
| A08 actual source/reader receipt | Annex A02/A07 |
| A09 private/public codecs and bounds | Annex A06 |
| A10 existing07 regression | Annex A08 |

Annex A10 is explicitly future real-H1 cryptography, not a precondition for this implementation slice. Later native tests must retain actual3/4-signature and wrong-domain evidence. Synthetic exact-tuple/proof callbacks can verify adapter branching now but must not become a production fallback or cryptographic claim.

Additional private-wire literals may be prepared while Builder works, using only the frozen field recipe and accepted03 golden bytes. Candidate inspection starts after a named revision is delivered. Then run focused independent cases and relevant07 regressions under the host lock; no full32-case integration requirement or heavy build was imposed at this pre-code gate.

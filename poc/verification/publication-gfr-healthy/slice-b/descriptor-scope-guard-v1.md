# Independent B descriptor-scope guard

Source review before B implementation: v10 02 §3.2 fixes CID from session, anchor identity and ProgramConfig; v1004 §3.4/PUB-3 permits stop only under the result's actual public scope.03 Descriptor.validate checks structure, not CID derivation; A BridgeResult proves bounded storage, not historical authenticity.

Therefore public_stop/permit_dispatch must not infer a result's session/branch/window directly from an unchecked BridgeResult skeleton. Before any scope lookup/StopDecision, require the ledger config_ref, reconstruct existing03 ProgramConfig::from_plan and recompute `regional_cid(session, anchor.block.block_id, program)`, comparing it to descriptor.cid. Equivalent consumption of the actual immutable verified Publisher task scope is acceptable. This is a cheap identity check, not a demand to re-run signatures merely to interpret an already known public close. Deadline decisions still require the actual Publisher-verified deadline and reader prefix.

Negative recipe: create a legitimate open-session result R1 and keep its CID/physical body unchanged. Clone it but change plan.session to a publicly closed R0. The ResultKey remains identical, and the bounded structural wrapper can still be built. Without the derivation guard, the forged clone can produce StopDecision for R1's real task. Expected: reject before scope lookup/mutation; the legitimate key remains permitted. Repeat with changed window, anchor block_id and config_ref. Positive control: unchanged R1 remains open, while an authentic R0 result is stopped by the same public closure.

No actual B candidate, signature, runtime receipt or worker was executed for this preparation. The negative cases are added to the independent B matrix and do not delay C acceptance.

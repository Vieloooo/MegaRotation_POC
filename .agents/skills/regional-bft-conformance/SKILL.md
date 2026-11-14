---
name: regional-bft-conformance
description: Draft protocol acceptance cases or independently verify a Regional BFT backend against the pinned paper, Commonware release, and project task contract. Use for certificate, data delivery, recovery, or fault-injection changes.
---

# Regional BFT conformance

Read the relevant protocol mapping and verification criteria in the repository root's `DESIGN.md`. Confirm the paper revision, protocol parameters, and Commonware version before deriving test expectations. Read the paper rules and accepted task contract before using the implementation to locate handlers.

1. Identify the triggering event, pre-state, guard, and expected next state or output. State which assumptions the scenario exercises.
2. Select the boundary cases for the assigned rule. For certificates, check the quorum boundary, distinct signers, subject and domain. For data availability, deliver the certificate first and supply the required ancestry and validated body later. For restart or timeout, place the injected event at the relevant state or persistence boundary.
3. Assert evidence and delivery separately. Check cryptographic proof observation, reconstruction and application validity, and delivery of a contiguous verified prefix at their respective transitions. State the conditions for progress before asserting recovery. An intentionally invalid or incomplete trace should exercise the rejection or pending state.
4. Verify the candidate using independent expected results and reproducible input traces. Record exact command, version, seed or fixture, and outcome. For each finding, identify the source rule, the code location, the smallest reproducer, and the observed impact. Report finite simulation evidence with its scenario and bounds.

A starting example is Kudzu at n=15, f=4, p=1. Test 13 versus 14 distinct First votes, then stage a valid FirstCert before the complete block. Record proof observation first. Record delivery once the NotarCert, validated body decoded from six distinct checked fragments, and complete ancestry are available under the task's rules.

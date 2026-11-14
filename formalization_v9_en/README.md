# formalization_v9 — English version

This version reorganizes and splits v8's [arch v0.2](../formalization_v8/arch_zh_v0_2.md) and [Regional BFT v0.13](../formalization_v8/regional_bft_abstraction_zh_v0_13.md). The Chinese source for this translation is [formalization_v9](../formalization_v9/README.md).

## Reading order

1. [00 System architecture and module overview](00_arch_en_v0_1.md): external building blocks, participants and sessions, chain data and mappings; the module and interface connection tables (W1–W28), and the four chains $R,G,G',D$.
2. Module files, each containing “Role and intuition,” “Minimal assumptions,” and “Interface”; five also have “Properties”:
   - [01 L1 Authority](01_l1_authority_en_v0_1.md)
   - [02 Session Program (Regional BFT) and Block Builder](02_session_program_en_v0_1.md) — properties BFT-1–7, composition goals G1–G3
   - [03 Session Manager](03_session_manager_en_v0_1.md) — also includes lifecycle rules and Kill criteria
   - [04 Publisher](04_publisher_en_v0_1.md) — properties PUB-1–5
   - [05 Batcher](05_batcher_en_v0_1.md) — properties BAT-1–4
   - [06 Certifier](06_certifier_en_v0_1.md) — properties CERT-1–5
   - [07 Batch Inbox](07_batch_inbox_en_v0_1.md)
   - [08 DA](08_da_en_v0_1.md) — properties DA-S1, DA-S2, DA-L1, DA-L2, DA-RET
3. [09 Finality rules v0.2](09_finality_rules_en_v0_2.md): Global Finality Rule (events and order, state machine, records and interfaces), Derivation Rule, native settlement, and guarantees of results at each layer. From this version on, Force and RESUME both take L1 inclusion time plus $M_a$ as their activation time. The [previous v0.1](09_finality_rules_en_v0_1.md) is retained for comparison.
4. [10 End-to-end pipelines v0.2](10_pipelines_en_v0_2.md): Regular handoff, Force handoff, RESUME after a long Espresso outage, RESUME after the batch layer exceeds its window, short failures and the dead zone, and calibration conclusions, recalculated using explicit example parameters (1 s L2 blocks, an assumed finalized budget, conversion of a 12 h window, and so on). The [previous v0.1](10_pipelines_en_v0_1.md) uses compressed heights and is retained for comparison.

[review_en_v0_1.md](review_en_v0_1.md) records earlier wording checks.

[security_review_en_v0_1.md](security_review_en_v0_1.md) is an independent protocol security review listing counterexamples, model boundaries, and possible directions for fixes that remain to be decided. The arithmetic recalculation script is [checks/calibrate_261027.py](checks/calibrate_261027.py); the review baseline summary is [checks/security_review_baseline.json](checks/security_review_baseline.json).

[security_review_en_v0_2.md](security_review_en_v0_2.md) adds the second round's findings (S10–S27) after v0.1's S1–S9, two cross-item repair recommendations, two corrections to pipelines v0.2, and a complete manual-check order; the v0.1 items are retained. Its baseline summary is [checks/security_review_v0_2_baseline.json](checks/security_review_v0_2_baseline.json).

The copied scripts and baseline summaries are unchanged from the Chinese source. The baseline hashes refer to the original source files, not to the English translations.

## Conventions

- Each named record is defined in only one file; other files link to it. Definition locations are listed in [00 §4.6](00_arch_en_v0_1.md).
- Cross-module data exchanges are listed in the interface connection table in [00 §4.4](00_arch_en_v0_1.md); upstream and downstream tables in each module's “Minimal assumptions” section use the same numbers.
- “[0x §a.b]” refers to a section in the corresponding file.

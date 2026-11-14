# Version 3.1 normal local rotation

Started 9 October and completed 10 October 2026. This revision implements the user's vanilla design as a protocol draft, using the working v3 source preserved in [baseline/main_v3.tex](baseline/main_v3.tex).

- [Editable source](../../main_v3_1.tex)
- [Rendered PDF](../../main_v3_1.pdf)
- [Preserved-file hashes](protected-sha256.json)
- [Validation and proof-review status](validation.md)

## Protocol changes

The first three protocol boxes contain the steady-state revision, a new normal-entry path, and the revised BLAME/recovery path. Changed or added rules are red. Existing proposal forwarding and all-to-all voting remain; there is no ROTATE_ACK, READY or extra handoff-signature round.

1. A fixed local quota bounds fresh physical proposals in one leader view. A normal application-only view therefore produces that many batches. DRAIN occupies a physical proposal position but never consumes the outer application quota `K`.
2. A terminal QC enables the next scheduled leader's first child without waiting for terminal finality or re-certifying the normal anchor.
3. Votes and proposals bind one opening identifier per view, carried in existing messages. An honest voter never endorses incompatible openings through the normal and recovery paths.
4. Normal advancement retains the old view's commit timers. A valid quit proof cancels only its own view's pending timers, even when delivered late; it does not roll back the current view or completed commitments.
5. BLAME names the view whose proposal is missing. Once Alice's terminal QC authenticates Bob's normal entry, a missing first Bob proposal targets Bob, and recovery proceeds toward Caro.
6. **There is no standalone missing-QC BLAME rule.** Once a valid proposal triggers votes on the uninterrupted, ready synchronous path, forwarding and multicast votes complete its QC. Conflict/quit handling can interrupt that path. Merely lacking a local QC does not activate a next-leader watchdog.

## Deadlines and overlap

- The proposal-relative normal handoff bound is `theta + 3 Delta_R`.
- The QC-relative bound is `tau + 2 Delta_R` when the earlier proposal timing cannot be used.
- Initial/recovery entry retains the `6 Delta_R` first-proposal allowance.
- A predecessor quit arriving before the first fresh successor proposal receives a one-time conservative recovery allowance, up to its receipt time plus `7 Delta_R`. Duplicates do not restart it. This rule prevents an honest successor still completing the original recovery entry from being judged by the shorter normal-path clock.
- The old leader is not blamed for an absent `(K_loc + 1)`st proposal after its local quota. A terminal QC authenticates normal succession; a raw proposal alone does not.

Normal/recovery overlap uses one signed opening per view, fixed entry locks, monotone view state and authenticated entry history. A late terminal QC cannot make a leader replace an opening it already signed. A late BLAME for Alice does not mean BLAME for Bob.

## Mathematical status

The original proof that an honest leader remains forever in one view no longer applies. The regional statements and sketches in Section 7 are therefore labeled verification targets/conditional arguments. The normal-entry versus recovery-entry propagation argument and mixed-entry watchdog soundness remain explicit proof obligations. The document does not claim completed protocol verification or inherit the old theorem without qualification.

The good-case local boundary preserves the ideal `2 delta_R` proposal interval under the stated readiness and compatibility premises. Failure recovery retains the separate STATUS/NEWVIEW costs. QC forwarding does not change the existing asymptotic message/bit bounds, but its cost is explicitly included.

## Source preservation

The original v3 and v4 sources/PDFs and nine active v3 figures are protected by before/after hashes. Version 3.1 uses independent figure copies under `figures/v3_1/`. No original manuscript was edited.

Build from `formalization/`:

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v3-1-rotation main_v3_1.tex
```

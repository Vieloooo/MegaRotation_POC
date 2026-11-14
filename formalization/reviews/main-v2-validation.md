# Version 2 validation record

**Historical first-draft record.** This draft was superseded by the six-section
major revision. See [current validation](main-v2-major-validation.md).
The pre-revision source, PDF and figures were preserved under
`../build/pre-major-v2-20260928-152210/`; the file links below describe the
historical deliverable names rather than the current contents at those names.

Completed 28 September 2026. Deliverables: [main_v2.tex](../main_v2.tex),
[main_v2.pdf](../main_v2.pdf), and [eight native TikZ inputs](../figures/v2/README.md).

## Scope

The draft specifies fixed-L1-configuration regional PoS SMR, preserving v1's
mini-block header, transaction byte strings, identifier and ORDER-signature
constructor. It implements a conservative weighted adaptation of Sync HotStuff's
standard synchronous proposal/vote/timer core, with a fully stated scheduled
freeze/drain/status local handover. It is not a verbatim implementation of the
paper's blame-triggered pacemaker.

The regional model assumes a fixed faulty set below one third of regional stake,
the stated synchronous complete-object envelope, persistent honest signing state,
and fair admitted-request service. The regional claims do not certify arbitrary
mobile outages or timing-assumption violations. Cross-region CFT, election,
rotation, crash/recovery, reconfiguration and global delivery remain explicitly
unfinished in Section 10. No outer protocol correctness or global finality claim
is made.

## Source preservation

- `main_v1.tex` remained byte-for-byte unchanged: SHA-256
  `f918927188fe07fd5962e0b2b65abde65565e727f1dbb811b2e49db8386cb13e`.
- Final `main_v2.tex`: SHA-256
  `b85b88b64a445e475b36daf6b59eeb09d865762570a33da9c66f3f44aefcd0cf`.
- Delivered `main_v2.pdf`: SHA-256
  `f8b5940c197616ae4770c6e5001eb9b9845484b96ab700c94036102b0c33d5de`.
- The `H_h` and `M_h` constructors were checked directly against v1. Raw
  transaction bytes are unchanged; authenticated request descriptors are bound
  by a new batch-level `reqRoot`, not by a new mini-block field.
- `u` remains a proposal-attempt ID and now injectively encodes region/local
  view/batch position. Separate `k` indexes the replicated batch log. The draft
  explains why this distinction preserves the sign-once evidence domains.

## Independent reviews and repairs

- [Sync HotStuff design/source review](main-v2-sync-design.md).
- [PoS compiler and standard SMR sources](main-v2-pos-sources.md).
- [Initial regional review](main-v2-regional-review-v1.md) and
  [final regional review](main-v2-regional-review-final.md).
- [Initial interface review](main-v2-interface-review-v1.md) and
  [final interface review](main-v2-interface-review-final.md).

The independent reviews led to four substantive corrections: make the full
batch framing part of the deterministic replicated command; bind fork evidence
to the entire session; enforce the proposal bound on every admissible branch,
including a faulty producer's; and specify the exact mandatory proposal timing
used by the honest-view progress proof. Further fixes preserve raw transaction
bytes, commit request metadata through `x`, exclude all proof encodings from
semantic state, define local notation, and state the reliable live-client path.

The regional final review accepted the proofs at source hash
`e3d5bb95852c3f74615068f5f64e49afa79fbe77340d969e2c9abd2bf09bf54f`.
The subsequent interface final review closed all its findings at hash
`90a15545b45170931b4ceef066baf614f71fcd4f050102334eeaf4d5d0c3619a`.
Changes after the latter snapshot were presentation-only: nonbreaking protocol
boxes, anchored figure placement, space before protocol sections, and a separate
references page. The final source contains the reviewed algorithmic rules.

These are independent human-readable agent reviews under explicit assumptions,
not a mechanized proof, implementation test, or deployment security certification.

## Compilation and visual inspection

Build command, from `formalization/`:

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v2 main_v2.tex
```

- Final PDF: 15 A4 pages.
- Six numbered protocols and eight TikZ figures; each protocol has diagram
  coverage documented in the figure manifest.
- All figure inputs resolve. The figures compiled independently before manuscript
  integration; all figures fit the manuscript text column. The revised application overview uses the wider version-1 organization.
- The final LaTeX log has no warnings, undefined references/citations, overfull
  boxes or underfull boxes.
- All pages were rendered and visually inspected. Pages affected by final layout
  edits were re-rendered and checked; protocol boxes no longer split across pages,
  relevant section headings stay with their protocol, and references are together.
- The delivered PDF is copied from the final build. Scratch renders and figure
  harnesses are confined to the ignored `build/` directory.

The existing version-1 manuscript and its figures were not modified. No commit
or remote push was performed.

## Application-level Figure 1 revision

At the user's request, Figure 1 now follows version 1's application-level overview: users, L1 stake/key/list configuration, the producing region's sequencers, proposer P, mini-block push, decision evidence and data replies, peer pull, and other regional SMRs. Dotted components explicitly mark the unfinished cross-region CFT and rotation layer; dashed arrows denote pull requests. Only this figure, its caption, the figure manifest and a box pagination setting changed. The protocol definitions and proofs are unchanged. The 15-page PDF was rebuilt without warnings; the revised figure and reflowed pages were rendered and visually inspected. Version 1 remains byte-for-byte unchanged.

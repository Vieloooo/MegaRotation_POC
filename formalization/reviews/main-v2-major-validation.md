# Six-section major revision: validation record

Date: 28 September 2026. Current deliverables: [main_v2.tex](../main_v2.tex),
[main_v2.pdf](../main_v2.pdf), and [four TikZ figures](../figures/v2-major/README.md).

## Revision and source scope

The manuscript follows the approved six sections: two-layer structure; model
and stream/SMR definitions; regional PoS SMR; delivery and retrieval; security;
and complexity. Notation is introduced with its use. The public stream receipt
is abstract, while the regional section supplies its concrete implementation.

Removed the request-ID/FIFO/deduplication machinery, reqRoot, Cantor encoding,
extra outcome-record layer, fixed view windows and per-view batch quota.
The pipeline continues on parent certification. Recovery follows native
BLAME/equivocation, quit, status, and NEW-VIEW anchor re-certification.

**Pinned protocol source:** Sync HotStuff ePrint 2019/270, revision
`20200504:200638`, Section II, Figures 1–2. This revision adds the explicit
NEW-VIEW re-certification and two-delay collection used here; the older
November 2019 author PDF is not this revision's algorithmic source.
Source PDF SHA-256:
`51d2ba5a1fe109543ff5f3db5e68b6ea7c96f77a162abfe2e3a8259a147769a7`.
A read-only copy is cached at `../build/main-v2-major/sources/synchotstuff-20200504.pdf`.

The protocol uses >2/3 regional stake certificates under a <1/3 faulty-stake
and standard-synchrony assumption. It proves fixed-session regional claims.
Request-level liveness additionally uses the explicitly stated fair input and
honest-serving condition; stream growth is not presented as anti-censorship.
L1 admission is abstracted, but dynamic stake activation and cross-region CFT
are not claimed complete. Regional confirmation is distinguished from future
cross-region finality.

## Reviews

- [Pinned-source and native-protocol audit](main-v2-major-native-sync.md).
- [Stream/SMR interface design](main-v2-major-interface-design.md).
- [Independent regional verification](main-v2-major-regional-review.md).
- [Initial interface review](main-v2-major-interface-review.md) and
  [closed final interface review](main-v2-major-interface-review-final.md).

Both final reviewers examined source hash
`c0340188273735d89f094e39599129b00f95d1d6c34ce4346bb62667477a1fe4`
and reported no unresolved findings within their stated scopes. Repairs made
the NEW-VIEW older-view guard and initial-root exception explicit, required
nonempty mini-block ranges, aligned the pending-input convention with request
liveness, defined semantic equality independently of signature encodings,
scoped ORDER evidence to session/producing-view/height, and qualified the
handover latency by an honest successor.

The only later manuscript edits assign explicit LaTeX cross-reference types
to assumptions, lemmas and theorems. No protocol or proof text changed after
the reviewed snapshot. These are independent paper-level agent reviews, not
mechanized verification or an implementation security certification.

## Final checks

- Compiled with `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v2-major main_v2.tex`.
- Nine A4 pages, six sections, three protocol boxes and four native TikZ figures.
- No compilation warnings, undefined references/citations, overfull or underfull boxes.
- All nine final PDF pages rendered and visually inspected; algorithms and
  diagrams are legible and unclipped. Cross-references identify the correct
  theorem/lemma/assumption types.
- Programmatically checked the six-section structure, all four figure inputs,
  absence of the removed machinery, and the unchanged v1 hash.
- The application-level architecture diagram preserves the current working
  draft's Users/L1/regional sequencers/P arrangement. Earlier figure files remain
  historical and are not dependencies of this build.

Final `main_v2.tex` SHA-256:
`1a0ba32517e50a9f317b08c70e9a5e4a2d6f90ce0aab1009d45d32269966fead`.

Final delivered PDF SHA-256:
`a2970e70768667b891c1035c661df72f3c305f0a770b8e1e08ce24e0eaf5c0f3`.

`main_v1.tex` remains unchanged at
`f918927188fe07fd5962e0b2b65abde65565e727f1dbb811b2e49db8386cb13e`.
The pre-revision v2 source, PDF and figure directory were preserved under
`../build/pre-major-v2-20260928-152210/` (source hash
`b85b88b64a445e475b36daf6b59eeb09d865762570a33da9c66f3f44aefcd0cf`).
No commit or remote push was performed.

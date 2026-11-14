# main_v3 presentation review, first render

Reviewed the 25-page build `formalization/build/main-v3/main_v3.pdf` rendered at approximately 23:25 on 2026-09-28. This is a visual/readability review, not proof certification. Inspected every page through five contact sheets, then full-size pages 2, 4, 7, 9, 14, 16, 18, 19 and 23. The rendered source was also checked at definitions, control rules, rotation algorithms and the complexity table.

## Changes required before delivery

The first render had genuine overlaps in all four new v3 figures. These have been fixed in their TikZ sources, pending a second render:

1. **Figure 1, page 2:** the finalized-anchor arrow label overprinted the scheduler text and successor box. Its interface band is now taller, labels are shorter, and actual node boundaries determine arrows.
2. **Figure 5, page 14:** the old-region final-receipt label hid part of its finalization box; successor action boxes touched/overprinted and the outbound witness label hid the last proposal line. Node text has been shortened and causal gaps enlarged.
3. **Figure 6, page 16:** status and finalized-entry labels obscured participant actions; all three successor action boxes were too close after automatic wrapping. Labels and boxes now have dedicated vertical space, short intentional lines and a smaller safety-precondition note.
4. **Figure 8, page 19:** incoming arrows continued into the authorized-stream heading. They now stop at the actual node border, and text has been compressed without dropping the separation of source/rotation, DA and execution obligations.

The four revised files are `figures/v3/architecture.tex`, `regular-rotation.tex`, `forced-rotation.tex`, and `derivation.tex`. No manuscript edit was made in this review.

## Whole-manuscript readability

- The ordering is coherent: regional and cross-region service definitions precede the regional implementation; committee state precedes cross-region handoff; the derivation relation follows rotation; guarantees and costs come last.
- Section 2 preserves the existing regional definition and makes the wrapper interface concrete. The interface table is legible, and separate meanings for SMR position `k`, application batch `b`, mini-block `h`, term and session are explained before the new algorithms.
- The early three-transition paragraph, the example in Section 5.4 and Protocol 9 make the important distinction usable: a local view change, a regular quota boundary, and an active-region failure are different events. Passive recovery does not authorize premature handoff.
- Both preserved regional figures inspected at full size are legible. Figure 4's staged blame/NEWVIEW diagram is especially helpful; its caption explains the timer versus re-certification distinction.
- The regional-interface table and the four-path complexity table are readable at the actual page size. The latter states measurement endpoints and separates receipt latency from pipeline interval; the surrounding formula for amortized batch interval is easy to locate.
- Prose, equations, protocols and tables show no clipping or overlap in the first render. Protocol boxes remain intact. Headers, footers, labels and references are consistent.
- The interface and proof obligations clearly separate source authentication, rotation consistency, data correspondence, DA and execution. The verifier relation does not quietly replace the safe-cut recovery assumption with a proof-system claim.

## Minor presentation observations

- Page 16 is a centered figure-only page. This is acceptable for a detailed communication diagram, though top placement could reduce the impression of excess whitespace if final pagination permits it.
- Page 7 leaves whitespace because the intact view-change algorithm begins on page 8. Keeping the algorithm intact is preferable to splitting it.
- The symbol density in Section 2.4 is moderate-to-high, but the neighboring interface table and explicit meanings make it manageable. A separate notation table would be optional, not required for this revision.
- “Timeout cross-region rotation” is expressly defined as expiry of a fixed application-batch quota, not an independently introduced wall-clock timeout. This is consistent with the agreed fixed-term design and visible in both Sections 5.2 and 8.2.

## Outstanding validation

Recompile the manuscript after the four figure-source fixes, render the new PDF, and visually inspect all four revised figures plus the changed page breaks. The first render is not acceptable for delivery because of the overlaps listed above. Final approval should refer to the second render, not this review alone.

## Second-render verification (23:31:55 build)

Full-size inspection of pages 2, 14, 16, 19 and 25 confirms that all original text/arrow overlaps are resolved. The regular and forced pipelines are legible, distinct and causally ordered. The derivation arrows stop at node boundaries. The new four-case communication table on page 25 is readable with ample width and a useful caption; references are intact beneath it. The PDF remains 25 pages and its LaTeX log has no overfull, underfull or warning lines.

A final cosmetic source patch lowers Figure 1's enclosing service border by 3 mm so that the unavailable-region box is fully contained. It also removes awkward word hyphenation from a few short diagram labels in Figures 1, 6 and 8; no protocol content changes. Root will include these changes in its final build and check the affected diagrams. Apart from that last local containment fix, the second-render presentation is suitable for delivery.

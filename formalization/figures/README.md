# Active protocol figures

The rewritten manuscript directly inputs two native TikZ sources:

- `service-architecture.tex`: service boundary, L1 authorization, users, regional signed batches and HEAD votes, proposer push/pull, and the cross-region backbone.
- `stream-pipeline.tex`: scheduled regional production and LMD-GHOST, replaceable preview branches above an installed adopted anchor, missing-data repair, and the separate global report/FFG path to durable delivery.

Both require `tikz` and `arrows.meta`. They contain no document preamble or figure float; captions are in `main.tex`. The three depicted sequencers are architectural examples, not an n=3/f=1 resilience claim. The pipeline shows protocol dependencies rather than a message-round timeline. There is no regional quorum certificate, DECIDED phase, or completion for each batch. A new justified adopted prefix constrains regional branches when installed at the next epoch boundary; finalized Deliver remains a separate output.

Epoch-level signed report agreement remains part of the global protocol and retains its communication cost. These figures describe the proposed LMD/FFG composition, not the exact Ethereum deployment.

Older PDF/drawio artifacts in this directory are retained as historical files. They are not used by the rewritten manuscript.

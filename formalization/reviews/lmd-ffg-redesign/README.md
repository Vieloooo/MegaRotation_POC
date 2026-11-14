# Regional LMD / global FFG redesign

This record applies to the September 26 redesign of `formalization/main.tex`.
The user's current working copy, including `v_G`/`v_L` terminology and the
separation of honesty from availability, was the baseline. No historical
review is treated as validation of this redesign.

## Result and scope

- Regional production is clock-driven. Silent slots do not require a GAP or
  a regional certificate before the next producer can start.
- Producers extend their available LMD-GHOST head. Ordinary HEAD messages
  carry no per-batch quorum or finality guarantee. `SetPreview` can replace
  the speculative suffix, including across local producer changes.
- There is no regional VOTE quorum, DECIDED certificate, or per-batch
  authenticated-broadcast completion.
- Global epoch reports can contain conflicting branches. The common report
  vector is resolved by a deterministic LMD snapshot; one fresh active-region
  report endorsement counts once per origin. Truncation endorses the admitted
  prefix endpoint, never an unavailable suffix.
- The already-existing epoch-level report agreement remains explicit. This
  hybrid is inspired by Ethereum's separation of fork choice and Casper FFG;
  it is not the deployed Gasper protocol, and FFG is not assumed to resolve
  arbitrary different targets without a convergence mechanism.
- Each justified adopted prefix becomes the regional anchor at the next fixed
  epoch boundary. Adjacent FFG links finalize checkpoints and authorize
  ROTATE/CONFIRM handoffs. Delivered logs are append-only.
- Proven progress concerns some newly adopted data under the stated global
  envelope, capacity, healthy-region and tenure assumptions. It does not
  promise local-head convergence, a particular preview's survival, censorship
  resistance, transaction inclusion, bounded reorg depth, or EVM execution.

## Proof / flaw / repair cycles

The two cycle-1 reviewers were freshly spawned after the first manuscript
revision, with no prior conversation or historical reviews as inputs. A third
fresh reviewer independently read the second revision. The root performed a
separate interface/complexity audit and made all manuscript repairs. Reviewer
reports state when an additional concern was communicated for independent
checking; they are not represented as blind discoveries when they were not.

| Cycle | Review | Findings and action |
|---|---|---|
| 1 | [Safety and authority](cycle1-safety.md) | Report snapshot cutoff gap; missing pull-timeout dominance; consumer anchor/catch-up invariant needed an explicit rule. |
| 1 | [Liveness and recovery](cycle1-liveness.md) | Confirmed cutoff and repair-bound gaps; fixed-anchor height-growth strategy and tenure arithmetic otherwise viable. |
| Repair 1 | Revision `4e246692…` | Filter completed slots before snapshot LMD; exclude newer HEAD records without restoring old votes; install due anchors before Deliver; define complete GET/DATA timeout bounds. |
| 2 | [Liveness recheck](cycle2-liveness-recheck.md) | Earlier findings closed; found that the final report-vector size was incorrectly used as an intermediate AB payload bound. |
| 2 | [Fresh full review](cycle2-fresh-review.md) | Core proof package checked; independently confirmed intermediate equivocation-body and admission-accounting issues. |
| Repair 2 | Revision `3b19bd5e…` | Distinguish final vector, cap-two honest-forwarded union, and all admitted candidate witnesses; introduce `J_G` verification/transfer accounting; include the four-member/six-body counterexample. |
| 3 | [Closure review](cycle3-closure.md) | Both accounting findings closed; a final body-bytes versus complete-envelope clarification was applied and independently accepted. No remaining findings within the stated model. |

### Concrete repairs

1. At a global boundary, a Byzantine producer can have pre-sent the newly
   starting slot's batch. Merely snapshotting before honest production did not
   exclude it. The report tree is now filtered to slots that have **ended**
   before applying LMD and whole-batch truncation. A fresh activation's first
   snapshot is therefore empty.
2. Responsiveness alone did not imply that a response arrives within an
   arbitrarily chosen retry timeout. `theta_R` and `theta_G` now dominate
   complete admitted request, service, response and verification bounds,
   including consumer paths.
3. A lagging consumer now installs already-due anchors and activations before
   delivery/resumed preview. Its installed anchor must contain its delivered
   log; conflicting previews are pruned before further output.
4. `N B_G` bounds only the final report vector. One equivocating source in a
   four-member committee can cause six distinct report bodies to be honestly
   forwarded before its AB output becomes bottom. The text now separates
   `O(N^2 B_G)` honest-forwarded body union, `O(N^3 B_G)` uncached endpoint
   copies of those values, and total `O(J_G B_G)` candidate-witness accounting.
   Cap-two storage is explicitly not an input-rate/verification bound.

## Snapshots

Exact source snapshots are retained in the ignored
`formalization/build/lmd-redesign-baseline/` directory.

| Snapshot | SHA-256 |
|---|---|
| User working-copy baseline | `9091964174afef8659e2f30f0480ccb56cbcee56bd11335ec74d228fa59224bf` |
| Cycle 1 draft | `97d709fcfcfe9e72e93d05dfca9da679d79c82a282b27e41dbcce2412ef7f5e9` |
| Cycle 2 repaired draft | `4e246692d857d8269b5e6e0b195e98f67a03a30c523f30c54fbc31530685ba35` |
| Cycle 3 accounting repair | `3b19bd5ef9fa7b12c43a7cbddd201e4c49e93d71fae0ee0e84583d7c7f1004d2` |
| Final independently reviewed text | `a1697f0842252a3af2d2a60cbb268ee73b3d00d8034629b8059762a8c635615b` |
| Delivered source, layout-only change | `325ab6a05c0fd903f0f999a1d474218d122a302a54aa9c65a744a5dc989a4223` |

The delivered source differs from the final reviewed text only by a
`Needspace` directive keeping the consumer subsection heading with its protocol
box. No statement, algorithm, proof, or bound changed after final closure.
The complete main-text change from the user's starting working copy is
available as [main-from-user-baseline.diff](main-from-user-baseline.diff).

The old figure sources were also preserved there. Existing unrelated working
changes, old diagrams/PDFs, outlines and historical reviews were not reset.

## Validation

- [Finite abstract checks](model_checks.py) enumerate 1,296 conflicting report
  vectors under unequal weights, each in all 24 origin orders, plus two packing
  budgets. They check deterministic selection, nonempty progress and selection
  of an admitted complete path.
- Twelve state-edge checks cover latest-message equivocation, old-vote
  resurrection, future messages, silent-leader scheduling, report cutoffs,
  anchor pruning and the intermediate-body accounting counterexample.
- These checks complement the written proofs; they are not a machine-checked
  verification of the full distributed protocol.
- Final build/render status and artifact hashes are recorded in
  [validation.md](validation.md).

## Sources used for design boundaries

- [Ethereum fork-choice specification](https://ethereum.github.io/consensus-specs/specs/phase0/fork-choice/): latest-message subtree weighting and justified/finalized anchor constraints.
- [Combining GHOST and Casper](https://arxiv.org/abs/2003.03052): distinction between fork choice, accountable safety and differently scoped liveness results.
- [Casper FFG](https://arxiv.org/abs/1710.09437): checkpoint links and finality.
- [Ebb-and-Flow Protocols](https://arxiv.org/abs/2009.04987): why a blanket vanilla-LMD convergence claim would be unjustified.

These are scoped independent proof audits and finite checks, not a deployment
security certification or a machine-checked formal proof.

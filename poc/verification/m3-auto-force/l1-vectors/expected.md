# AF07 and AF08 Authority expectations

Derived before examining the M3 implementation from contract `27fd202cb88688914b8d0fa771cf6dad0237473c`, v10 Authority §§3.1–3.2 and 3.9, and the accepted EIP-712 request interface. The tests cover actual EVM inclusion and contract state at deterministic L1 block positions.

The trigger is a mined `submitForce` transaction. The contract checks exactly k valid, canonical signatures from distinct configured members on the same digest. The packed order is increasing recovered Ethereum address. The digest commits to chain ID, Authority address, config reference, caller, request ID, sequence, target branch, policy and Force type. Rejecting any guard rolls back writes and emits no Force event. A successful fresh call writes one event, advances the sequence, and returns an activation computed from its block timestamp. A valid retry returns the original receipt fields and leaves the sequence and control logs intact.

The default profile has three fixed governance members, k=2, and a ten-second announcement. A valid region mask may disable R0 (`6`) or every region (`0`). The cycle can be `[0,2,1]`. The EVM records these authorization fields for GFR. Test upper mask bits and a repeated cycle entry as invalid even when the payload has the correct quorum. The selected policy floor interpretation in GFR belongs to separate historical Plan tests.

| Stimulus against k=2 | Expected contract result |
| --- | --- |
| One member, zero members, three members, or a truncated packed signature | Revert with no Force record |
| The same member twice or the same two valid members in reverse order | Revert with no Force record |
| An outsider paired with a valid member | Revert with no Force record |
| A high-s encoding of a valid signature or a signature with a disallowed v byte | Revert with no Force record |
| Digests for a different chain, contract, config, caller, or RESUME type | Revert with no Force record |
| Reuse signatures after changing the policy, request ID, control sequence, or target branch | Revert with no Force record |
| Two valid signatures authorizing mask 8 or cycle [0,0,2] | Revert with no Force record |
| Two valid members sign the fresh request after the rejection vectors | Succeed with one Force record. Activation equals this block's timestamp plus ten seconds |
| Replay the exact request in a later block, including with another valid subset | Succeed with the original result and zero additional Force records |
| Retry that accepted request with too few signatures | Revert despite an existing result for the request ID |
| Authorize changed content under the accepted ID or reuse its sequence with a new ID | Revert with no Force record and no sequence change |
| Sign a new ID with a higher sequence and a valid zero mask | Succeed with a second Force record and advance the stored sequence |

For the alternate k=1 deployment, verify the deployed member count and threshold, reject a two-signature packet under the exact-length rule, accept one valid member, and accept a retry with another single valid member without a second control event. Check the wrong-contract domain independently with a packet of the expected length.

The independent evidence records the tested source revision, tools, transaction calldata and package bytes, complete RPC receipts, inclusion headers, Force log positions, sequence reads and returned activation values. Governance service persistence after an ambiguous broadcast is exercised in the M3 scenario.

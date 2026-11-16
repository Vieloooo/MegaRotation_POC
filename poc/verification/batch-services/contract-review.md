# Independent contract review: BS-COMPONENT-v1

Date: 2026-11-16. Reviewed draft: `draft-r1`, integrated dependency baseline `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac`. Reviewer: named `bft_verifier` (Astra/xhigh configured). Sources and independent semantic expectations were read/written before any feature-11 implementation. Only fixed 03/09 dependency interfaces were inspected.

**PASS for interface/contract freeze.** No blocking discrepancy remains between draft-r1 and the pinned source/P0 boundary. This is not candidate implementation PASS, real Anvil evidence, or full feature-11 acceptance.

The proposed BS-FRAME-v1 layout is explicitly a P0 encoding. Its counts, exact lengths, deterministic 16-byte channel identifier, complete-frame marker and 32-byte payload commitment are unambiguous. The maximum is independently calculated as `2 + 2*(20 + 1024*(4+19)) = 47146` frame-data bytes and 47175 payload bytes. A single empty record is 51 bytes. The codec references existing 03 raw Transfer decoding without changing 03 identity/object encodings. No unsupported origin monotonicity rule was introduced.

The independent frame suite contains the contract §9 single-height, two-height and padding examples, three additional boundary/execution examples, 19 malformed examples and the maximum-size recipe. Independent crypto vectors reuse the already frozen 09 domain/type strings and independently compute Keccak/ABI/signature recovery. They contain actual offline secp256k1 signatures, but explicitly fixture addresses/gpos and no Anvil acceptance claim. Production code must not import these verifier helpers.

The trust boundaries are stated sufficiently for implementation review:

- Actual reader-delivered finalized 09 Driver history, configured Inbox, successful original `CursorAdvanced`, and matching transaction/calldata/receipt metadata precede trusted AcceptedRecord construction. Public EventRecord or JSON alone is not a trust capability; a successful retry without Advanced is not a second acceptance.
- The G source is a bounded fixture, independently executed by Certifier. Missing exact source or recovery progress causes WAIT. 03's typed ReplayPosition is not silently assigned a new 11 canonical encoding.
- Certifier cache hits recheck current prev and the latest RESUME at/before cursor written_at. Inbox request dedup has different semantics and cannot be copied into Certifier.
- Payload data and directory durability precede DA signing; certificate/job/outbox durability precedes externally visible output. Raw transaction identity is durable before send. Recovery reconciles unknown outcomes and binds stored state to chain/Inbox/config/signer identity.
- Real process crash evidence is required at the named boundaries; object drop/reopen is separately labeled. Component scripted input does not claim completed 02 grant/ack integration.

Independent preparations were run with:

```sh
/home/jy/miniconda3/bin/python3 poc/verification/batch-services/frame_vectors.py check
/home/jy/miniconda3/bin/python3 poc/verification/batch-services/signature_vectors.py check
```

Both pass as reference self-consistency checks. Seed is `none (literal inputs)`; private offline signing keys were generated in a private `/tmp/batch-services-verifier-keys-*` directory and are not included in repository artifacts. Candidate comparison has not run.

Lead may mark r1 FROZEN and start one Builder. Semantic changes to fields, guards, bounds, trusted construction or persistence require revised contract and affected independent vectors. Remaining gates are exactly the contract's actual implementation/review/test matrix and explicit 02/07/08/12/13/full-system integration gates.

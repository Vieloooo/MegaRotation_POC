# NATIVE-ENGINE/FRESH — bounded replay PASS

Verified integrated revision fec7ade1675cb60dd16a16fb28ebe81e3f76f508. All66 runs passed the independent checker; all33 seed/wall/start-order pairs matched exact full virtual JSONL, frames, step/witness ledgers/heads and native files (513 file comparisons). No failures, timeouts or incomplete cases; source and executable hashes unchanged throughout.

Matrix: `poc/artifacts/simulation/native-engine-fresh/fec7ade1675cb60dd16a16fb28ebe81e3f76f508/matrix.json`, SHA256 f7290f02ee04647f20e42cf348377c8770f6bc7cdf8cde52d2a414ae6db530d1. Private evidence root is281MiB, mode700; generated private launch files mode600. Public exports exclude key material.

54,690 rows,14,100 Acks,108 proof and108 delivery effects. Healthy delivered two blocks to both real workers. Without scripts the two real workers formed no quorum and delivered no blocks. Proof-before-body worker1 observed proof at virtual1,085,500us and delivered at1,250,000us. Collector local wall451.693s includes checker/comparison; virtual final ticks ranged1,135,500–1,900,000us. No real-network/AWS measurement.

The fixture uses two real native processes plus up to three explicit scripted signers, with quorum4. There were no RESOLVER packets; executed repair remains pending. Same-cid recovery/timing/auxiliary state, v10 and15-node handoffs remain open. This does not pass overall I1b/I1.

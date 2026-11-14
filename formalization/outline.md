Now, in the [main.tex](formalization/main.tex) , I will formalize some part of this protocol:&#x20;


Participants: Sequencer, Proposer and L1. 

$S_{i, j}$ is the sequncer notation 
$P$ is the proposer 
$L1$ is the L1 ledger, $L2$ is the L2 ledger. 

role: 
- sequencer needs to provides a "reliable" miniblock stream. 
- its proposer's role to batch the miniblock into block, commit the state root in L1, and push the batched data into the DA layer. 


Network model: 
- sync network $\Delta$ be the upper bound, $\delta$ be the acc com time. 

adversary model: 
- byzantine sequencer and the proposer. 
- f < 1/3 stakes. 


The intuition is: 
- the active inter-region commitee ensures that the current leading sequencer is proposing a proper live miniblock stream. 
- the cross-regsion comitee ensures that the current leading region is healthy, do not disconnected from the global network. 
The core idea is, for the active region, we apply a LMD-ghost, and cross-regision, we use  Gasper FFG.   



Consuensus 
To implment the sequencer notation, I am inpsired by the GeoBFT protocol, and combinding the with casper PoS in the ethereum to proposing a herarchical BFT for the sequencer rotation. The core idea is, for each region, we apply a LMD-ghost and Gasper PoS BFT to ensure that the seqeucner is providing a reliable miniblock stream.  
The core diff is: 
- at every slot, only the inter-region super-major (i.e., out of this region) votes is required. Unlike Ethereum, in which each slot required all votes. 
- epoch time is much longer, e.g., a hour. the Gasper requires the the superlinks (counted by all region's stake) from all the sequencer region.. 
- The inter-regioncommunication is through a fast direct communication, at low cost. 
- The cross-region communication is costly, we can use a P2P flood backbone + heartbeat polling for this. 


- the rotation/view-change rule is also different: the core idea is, to ensure the latency and aviod viewchange cost: 
    - the inter-region view change per epoch (e.g., 10 min). 
    - a liveness inter-region view change: with a T (e.g., 30s, this can be dynamically adjusted based on the stablity score of this leading sequencer, some future game theory design here) consecutive empty block signed by supermajor inter-region commit, it auto trigger's the inter-region leader change, move the leader to the next leader in this region. Ideally, this does not create a fork, the new leader will building new block from the lasted inter-region major voted blocks. 

For a geo rotation triggers. SImilar to the above, we have a timeout change (e.g., every 24 hours) and a liveness view change (e.g., a )
    - the liveness view changing is: at the casper FFG per epoch, each sequencer will say "if it is satifiby with the current region's block proposing for this epoch" (this is a off-chain logic we will design later), 
    - If ok, votes for "C_{e-1} -> C_{e}$. 
    - if in-complete epoch data, or unsatifify with the current region, vote "C_{e-1} -> C_{e-1}$. 
    - supermajor denoted $C_{e-1} => C_{e-1}$. 
    - a liveness rotation trigger if the following $C_{e-1} => C_{e-1} => C_{e-1}$. This means a "empty epoch is finalized by one empty block",  



- within the region, we apply the following byzantine reliable broadcast protocol with pipeline per block: 
- at round 1, the leader propose the miniblock with sig, and "leader's vote" for this miniblock, then boardcast to every in-region sequencer + the proposer $P$. 
- at round 2, all sequencer votes for the miniblock if its derived from its own lastest committed tip. If no miniblock is received, its vote for a empty miniblock, and derive a new tip. Then each broadcast this votes. (noting here)




On-chain equivocation for the in-region PoS: 
- double vote: a sequencer signs two differ block header at the same height. 
- Surround vote: 






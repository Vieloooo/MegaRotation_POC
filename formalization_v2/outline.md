The module

## L1 
- The staking module, maintain the join/leave of the sequencer's stakes. This module help a static BFT protocol into a premissionless BFT that allows join/leave. The equivocation slash is handled here. 
- the region liveness and force fork module. The module allow the multi-sig to control the regional liveness and a force 
- The Rotation module, based on the staking module's current epoch stakeassign the authority leader rotation rule. 

Then based on the above 3 L1 module, the sequencer set can derive the current rotation rule + the force fork logic. 

- a verification module. This module ensure that the Proposer submitted batched block is global finalized. 


## sequencer level 

- a leader module, this module is response for the regional leader to proposal batched miniblock. (each miniblock batch can be considered as one sequencer-level block.) The core the the orders of the miniblock selections. （打包算法）。 but this is not our focus. We only specfity what is a correct miniblock, miniblock batches. 

- a consensus module, handles the regional BFT and cross-region rotations. 

## The multi-sig 
The monitor protocol run by the multi-sig commitee (we can first omit how the multi-sig is formed, and consider this as one entity to writ ehte monitor protocol)

## a backbone communication model 

Help the regional batches propogates cross regions, and deliver to the proposer. 

## proposer 

The proposer needs to interfact with the L1.verification module, these two moudle ensure that the proposer is rolluping up the correct miniblocks. 




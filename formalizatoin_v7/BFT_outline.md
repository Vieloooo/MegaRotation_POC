This doc provides the high level abstraction of the regional BFT layer 

The core change: 

- Due to we move to the Simplex variant, it seems that we can change the leader per block, no need for term 
- I want to sperate the L2 block with the regional BFT. 
    - the L2 block is a "virtual block" derived from the deposits tx and the regional BFT block. 

- I want any partial BFT be a sweepable model, i.e., if such regional BFT fits the interface and the properties we specified, it can sweep. 






## Building blocks and assumptions 

We keep the basic structure of the formalization v6, regional bft abstraction v0.3 

Compared with v0.2 the assumption change is: 

- For A3, we assume a no drift global clock. The core idea, the \tau drift is handled by the internal model of the global clock. so we assume a no drift \tau for simplifity. 


- and we assume a static leq 1/3 adversary in this session 


For the global building block 

We assume: 
- a always safe and live L1, and the L1 is proposing in a fixed pace. 
    - safety and liveness (including that the finality chain is accessable by all honest nodes)
    - a fixed pace means that for every L1 block/slot $ell$, its timestamp $ts(\ell) = ts(\ell+1)$ (This means a slot time is always fixed ) and $ts(0) = t_{L1_genesis}$. 

- The fixed pace of virtual L2 block, for each L2 block start for this esssion (i.e., the curent B2 in v0.2), sycned with the global clock. 


- The synced L1 pace and L2 pace: 
    - this means, for any L2 height h, we have a fixed finalized L1 height, $os(s)$, and a fixed time stamp $ts(h)$ for this L2 block.  
    - this L1 and L2 sync is crucial for our deposit only padding rule. 
    - this assumption does not iimple that the phsyical block proposed by the regional BFT should fit this assumption. 

- With a the sync L1-L2 assumption, the for every L2 height $h$, we have $Dep(h)$, i.e., the ordered deposit tx list that should be included at the front part of this L2 block. 

The rule is simple, as every finalized L1, there is a list of acticvated L2 txs, we puth into the $h$, such that $h$ is the first height that use this L1 height as origin. 

Our regional BFT does not rely on the espresso chain in this version, so we do not make any assumption on espresso here. 


## the block building rule

The block building rule is the crutial model ensure that the regional BFT phsyical block can be compatbile with the vritual L2 block. 

If you familiar with the post https://decentralizedthoughts.github.io/2026-05-20-from-single-shot-to-chained/ , the chained Simplex, this rule ensure, 
- When a new regional BFT physical block $B$ proposed after block $B_{prev}$ at time $t$, how to ensure the new proposed block is valid or not. 

Here we define two functions: 

here each phsyical regional Block payload contains the following structure: 
$$
<Hash(B_{prev}), h, ts(h), os(h), s, Txs>
$$
where: 
- h is the vrital L2 hegith 
- ts be the timestmap 
- os be the origion height 
- s be the state root 
- Txs be the transaction that decentralized sequencer layer added (excluding the deposit only txs)


the payload of the physical block can be considered as the "half-virtial" L2 block. combined with the Dep list we can derive the real "vritial L2 block" 


The meta data mapping: i.e., for a honest leader i view v ($L_{v}$) propose a new block at time $\tau$ in view $v$: 
- $Meta(v, \tau, B_{prev}) \to (h, ts(h), os(h))$, where 
    - h is the virtual block height that the new physical block. $h > B_{prev}$,but to  fit the B1 assumption, if the virtial L2 block should be pace with the real world time $\tau$, the $h$ should be $h$, such that $ts(h) \leq \tau$ but $ts_{h+1} > \tau$.  

The state mapping, ie. what is the state root of the new block $h$? 

- $(v, \tau, B_{prev}, h, ts(h), os(h), Dep) -> s(h)$, where $s(h)$ is the state rule. 
    - The $s(h)$ is dervied itered: 
        - start from $s' = B_{prev}.s$. 
        - $s' \gets Exec(B_{prev}.s, Dep(B_{prev}.h), B_{prev}.txs)$. This step apply the B_{prev}.h all the txs 
        - For i \in [B_{prev}.h + 1, h-1]$, 
        $s' \gets Exec(s', Dep(i))$. apply the deposit only state txs. 


Valid function 

Verification of the block building rule: 
- consider the block is proposed at $\tau$, they in the good network condition $\tau' < \tau + \Delta$, other replica will check this and decide if vote for this block, is the the valid rule. 
- to chec validity, we can see that replica check with 
    - for the \tau, if the h is one block proposed at any time $\tau''$ between $\tau' - \Delta$ to $\tau'$. its meta data check passed. formally, 
    $$\exsts \tau'' \in [\tau' - \Delta, \tau'], s.t. \text{Meta is gen from the Meta Mapping. }
    $$
    - Given the fixed Meta, we check the state root, note that the Dep are public 
    - If these passed, then we apply basic check to the Txs, i.e., each txs is valid tranaction (can be revoked, but valid txs, the minimal check for each txs)






Given the above rule, we can derive the full virtial L2 block as 

$$
< h, ts(h), os(h), s, Dep(h), Txs>
$$

The virtual L2 block has two types: 
- Deposit only, i.e., this virtual L2 block is derived from L1 and lastest nomral block. 
- normal block. containing both dep(h) and txs 
- the B_{prev} cites the latest normal block. 


In summary, the phsyical block's payload is the (parts of) "normal block of virtial blocks" 



## Regional BFT interface and properties 


Here, the core change is that for each view, it can propose at most one physical L2 block. (e.g., in the raw simplex protocol, it can be a dummy block). 

We remove the term. 


We keep the interface of the V0.3 for the regional BFT. 



## a few new design option 

- during the building and voting steps: 
    - here, we require the votes to double check the state root used in every block, which might be slow, but our fast finiality do provides a state commitment. 
        - a few options: 
            - we can remove the stateroot in the regional bft layer so no check 
            - we allow the state root be in-consistnecy with the 
                - $s_{h} \neq Apply(s_{h-1}, Dep(h), Txs(h))$, where Dep(h) are public info and Txs is sidnger by the leader at h, and $s_{h}$ is 
                 - But the bad things is, we might need to revoke finalized regional blocks 



a new chanllenge, how to determine the last block of a session (i.e., the session of a new block? )
- what if we wait for the G to finalize this block?
-  
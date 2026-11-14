# Simplex Intuition and Regional BFT Selection


## 1. Standard Simplex: protocol block

Based on [Simplex Bog](https://simplex.blog/protocol/), with views denoting its iterations, $n=3f+1$, and $q=n-f$ and the [Deconstructing Simplex post](https://decentralizedthoughts.github.io/2026-04-23-deconstructing-simplex/)
```text
Notation:
    C_v: a notarized chain through view v; C_0 is genesis.
    VC_v(B): q signed Vote(v, B) messages, for a real block B.
    SC_v:    q signed Vote(v, ⊥) messages, for the dummy block ⊥.
    DC_v(C_v):  C_v with q signed Finalize(v) messages.
All messages are signed and broadcast.

On entering view v:
    Start a T = 3Δ timer.
    The leader proposes B extending a notarized C_{v-1}.

On the first valid leader proposal in view v:
    Valid(C_w, B, aux) and the notarized parent chain.
    Broadcast Vote(v, B). 

On timeout, while still in view v:
    Broadcast Vote(v, ⊥).

On obtaining a notarized C_v, using a VC_v or SC_v at its tip:
    If the timer has not fired:
        Cancel it; broadcast Finalize(v).
    Broadcast C_v and enter view v+1.

At any time, on obtaining DC_v(C_v):
    Finalize the real-block prefix represented by C_v.
```

**Core inuition**: 
For a proposal in view $v$, suppose the reference is the last real block of a certified chain $C_w$. The evidence is:

$$
\underbrace{\mathrm{VC}_w(C_w)}_{\text{notarized reference}}
\quad+\quad
\underbrace{\mathrm{SC}_{w+1},\ldots,\mathrm{SC}_{v-1}}_{\text{every intervening view skipped}}  + \underbrace{\text{Valid(B)}}_{\text{valid new block}}
$$

The relationships are:

- **DC requires the corresponding VC.** A notarized real block alone is not a final decision.
- **VC and SC may coexist.** An honest node may vote for a real block and also vote for $\bot_v$.
- **SC and DC cannot coexist.** An honest node that votes for $\bot_v$ never sends `Finalize(v)`. Two quorums of $n-f=2f+1$ nodes must intersect in an honest node, so both certificates cannot form—even if messages arrive later. [Certificate properties](https://decentralizedthoughts.github.io/2026-04-23-deconstructing-simplex/)




## 2. Two-round good-case Simplex: more first-round votes replace the final round

The simplified [two-round Simplex-style construction](https://decentralizedthoughts.github.io/2025-07-18-two-round-Simplex/) uses $n=5f+1$:

```text
Round 1: the leader proposes B.
Round 2: nodes broadcast Vote(B); n-f = 4f+1 votes directly decide B.
```

The difficult case is a node deciding B and immediately becoming unreachable. Other nodes must preserve that decision even without seeing its complete certificate.

If B received $n-f$ votes, another observer collecting $n-f$ nodes' votes must still see at least $n-3f$ honest votes for B: subtract up to $f$ Byzantine voters and up to $f$ omitted voters. A conflicting ordinary value can have at most $2f$ votes: 

$$
n-3f>2f\quad\Longrightarrow\quad n\ge5f+1.
$$

**Intuition:** a fast confirmed (v, B) means no other block B' can got the VC at v, view with "no common consensus" will be consider skipped (i.e., a ). 



## 3. Concurrent Simplex variants. 

**[Concurrent Simplex](https://decentralizedthoughts.github.io/2025-07-29-2-round-3-round-simplex/) shares the proposal and first-round votes between a large-quorum two-round path and a smaller-quorum three-round path, running both simultaneously with coordinated certificate and skip rules so either can decide without waiting for a fast-path timeout.**

## 4. Concurrent Simplex comparison and Regional BFT selection

**Good-case finality is our priority, but geography determines the cost of reaching each quorum.** Following [Note 17](17_two_round_bft_literature_review_zh.md), compare the concurrent family—Banyan, Kudzu, Hydrangea, and Alpenglow—with standard Simplex and the pure two-round alternatives. Their shared intuition does not make their certificate rules interchangeable.

### 4.1 Safety tolerance, liveness tolerance, and fast-path tolerance

These are three different guarantees:

- **Safety tolerance:** how many Byzantine nodes can exist without two conflicting DCs forming. Agreement must hold even during asynchrony or a partition.
- **Liveness tolerance:** which combination of Byzantine and unavailable nodes still permits eventual finalization **after communication stabilizes**, with the protocol's data, state, and leader-selection assumptions satisfied.
- **Fast-path tolerance:** the fault budget under which an honest leader is guaranteed two-round finality. Exceeding it may disable the fast path while leaving safety and eventual liveness intact.

Let $b$ be the actual Byzantine count and $o$ the additional non-Byzantine offline/crashed count. They are disjoint. The configured parameters are $f$ (Byzantine budget), $c$ (additional crash budget, where provided), and $p$ (fast-path budget). In Byzantine-only models, permanent crashes consume the same total fault budget as Byzantine behavior; there is no independent extra crash allowance.

| Protocol and configuration | Safety fault tolerance | Eventual-liveness fault tolerance | Two-round good-case fault tolerance |
| --- | --- | --- | --- |
| **Standard Simplex**, $n\ge3f+1$ | $b\le f$; agreement survives arbitrary message delays | Guaranteed for $b+o\le f$; the remaining $n-f$ correct nodes must participate after stabilization | No two-round strong-finality path |
| **Banyan v3**, $n\ge\max(3f+2p-1,3f+1)$, $1\le p\le f$ | $b\le f$ | Guaranteed for $b+o\le f$, under the paper's partial-synchrony and leader assumptions | $b+o\le p$ suffices with an honest leader |
| **Kudzu v2**, $n\ge3f+2p+1$ | $b\le f$; formal analysis uses static corruption | Guaranteed for $b+o\le f$; valid data and certificate recovery must also complete | $b+o\le p$ suffices with an honest leader |
| **Hydrangea**, $n=3f+2c+k+1$ | Up to $f$ Byzantine in the stated mixed model, which also permits $c$ crashes | $b\le f$ **and** $o\le c$; at least $n-f-c$ correct participants remain | $b+o\le p=\lfloor(c+k)/2\rfloor$, within the mixed model |
| **Alpenglow / Votor**, stake weighted | Byzantine stake $B<20\%$; Anza states safety even with arbitrary offline stake and full asynchrony | The broader allowance $B<20\%$, offline stake $O<20\%$ needs the stated propagation conditions. With an equivocating leader, the cited guarantee requires **$B+O<20\%$** | At least 80% stake supplies the required votes; enough correct timely voting stake is needed to guarantee this |
| **Minimmit / ChonkyBFT**, $n\ge5f+1$ | $b\le f$ in their stated models | Guaranteed for $b+o\le f$; at least $n-f$ correct nodes remain to form a decision QC | $b+o\le f$ suffices with an honest leader |

The liveness column states the cited guarantees, not a maximal tolerance inferred from one quorum formula. **A smaller slow quorum alone does not prove tolerance of extra Byzantine or crashed nodes.** Offline nodes can stop progress without creating conflicting signatures; loss of liveness does not itself imply loss of safety.

Sources: [Simplex](https://simplex.blog/protocol/), [Banyan §§3 and 8](https://arxiv.org/html/2312.05869v3), [Kudzu §§3 and 5](https://arxiv.org/html/2505.08771v2), [Hydrangea Theorem 1](https://www.usenix.org/system/files/usenixsecurity26-shrestha.pdf), [Alpenglow's safety/liveness conditions](https://github.com/anza-xyz/alpenglow/blob/master/KNOWN_NON_ISSUES.md), [Minimmit](https://arxiv.org/html/2508.10862v5), and [ChonkyBFT](https://arxiv.org/html/2503.15380v1).

For **timing**, assume a valid proposal in a ready view, an honest leader, timely data/validation, and stable communication. A round is one message propagation, not a full RTT.

| Protocol | Good case: fast path available | Fast path unavailable, but within the liveness budget |
| --- | --- | --- |
| Standard Simplex | $3\delta$ strong finality | The same three-round path |
| Banyan | Two steps for the rank-0 proposal | ICC-style three-step finalization |
| Kudzu | $2\delta$ | $3\delta$; the slow path already runs concurrently |
| Hydrangea | $2\delta$ | $3\delta$ under its full mixed-fault budget |
| Alpenglow / Votor | After data delivery: one voting round reaching 80% stake | Two voting rounds reaching approximately 60% stake per round, subject to its model conditions |
| Minimmit / ChonkyBFT | $2\delta$ throughout their good-case fault budget | No independent three-round commit path; an unsuccessful view requires recovery and another proposal |

Two qualifications affect selection:

- **Hydrangea:** use the conservative $p$ definition in the abstract/Theorem 1. The PDF contains a different definition elsewhere; Note 17 records the discrepancy and the allowed $k$ ranges. The crash budget $c$ cannot be reclassified as additional Byzantine tolerance.
- **Alpenglow:** voting time is approximately $\min\{V(80\%),2V(60\%)\}$ **after block dissemination**. Its “20% Byzantine + 20% offline” description is neither 40% Byzantine safety tolerance nor an unconditional liveness guarantee under arbitrary leader equivocation. [Model conditions](https://github.com/anza-xyz/alpenglow/blob/master/KNOWN_NON_ISSUES.md)

### 4.2 What “bad case” actually means

**Losing the fast path and having a faulty leader are different cases.** The three-round column above covers an honest leader whose fast quorum does not arrive. A malicious leader may produce no decidable proposal at all.

| Bad-case setting | Theoretical statement we can use |
| --- | --- |
| **Network not yet stable / partitioned** | No finite finality bound in the partial-synchrony model. Safety is preserved within the safety fault budget, but progress may stop. |
| **Faulty leader, standard Simplex** | If all honest nodes have entered a view by time $t$ and communication is stable, the blog bounds their advancement to the next view by $t+3\Delta+\delta$. This is a **view-exit bound**, not a finality bound for that leader's block. |
| **Faulty leader, Kudzu** | Lemma 5.10 gives slot exit by $T+\Delta_{\rm timeout}+3\delta$ under its stated synchrony/entry conditions. A subsequent successful proposal still needs its commit path. |
| **Faulty leader, Banyan / Hydrangea / Votor** | Recovery depends on their timeout, evidence, and leader/window rules; use each protocol's recovery analysis rather than assigning the whole family one common numerical bound. |
| **Faulty leader, ChonkyBFT** | Lemma 8 bounds progress beyond the current maximum honest view by $\Delta_{\rm timeout}+2\Delta_{\rm rcv}$ after GST, under its delivery assumptions. This also measures view progress, not finality. |

These bounds use different starting events. [Simplex liveness](https://simplex.blog/protocol/#liveness-argument), [Kudzu Lemma 5.10](https://arxiv.org/html/2505.08771v2), and [ChonkyBFT Lemma 8](https://arxiv.org/html/2503.15380v1) should therefore not be ranked by their coefficients alone. Recovery time includes failed views, evidence/data catch-up, and the next successful leader's proposal. Finality times here concern consensus; transaction batching and execution add further latency.

### 4.3 QC size: required votes and encoded bytes

First compare **how many distinct signers a certificate must represent**. A small continuation VC is not a small finality QC.

| Protocol | Continuation / notarization evidence | Fast decision QC | Slow decision QC | Skip / recovery evidence |
| --- | --- | --- | --- | --- |
| Standard Simplex | VC: $n-f$ Votes | — | DC: $n-f$ Finalize messages, with the notarized chain | SC: $n-f$ Votes for $\bot$ |
| Banyan | $\lceil(n+f+1)/2\rceil$ notarization votes | $n-p$ rank-0 fast votes | $\lceil(n+f+1)/2\rceil$ finalization votes | ICC/Banyan-specific progress rules |
| Kudzu | NotarCert: $n-f-p$ notarization votes | FirstCert: $n-p$ first votes | FinalCert: $n-f-p$ final votes | Timeout certificate: $n-f-p$ timeout notarization votes |
| Hydrangea | Block certificate: $\lceil(n+f+1)/2\rceil$ votes | $n-p$ votes | $2f+c+1$ commit votes | TC: $n-f-c$ signed timeout reports |
| Alpenglow / Votor | Approximately 60% notarization stake | At least 80% notarization stake | Approximately 60% finalization stake | Stake-weighted, protocol-specific skip certificates |
| Minimmit | M-notarization: $2f+1$ votes | L-notarization: $n-f$ votes | — | $2f+1$ nullification messages |
| ChonkyBFT | CommitQC also supports the next proposal | CommitQC: $n-f$ commit votes | — | TimeoutQC: $n-f$ timeout reports |

The names and thresholds follow the linked papers, not a universal VC/SC implementation. Hydrangea and ChonkyBFT recovery certificates contain reports about prior votes/certificates, rather than merely the same bottom vote from every signer. See [Kudzu Definition 4.2](https://arxiv.org/html/2505.08771v2), [Hydrangea §5](https://www.usenix.org/system/files/usenixsecurity26-shrestha.pdf), and [ChonkyBFT §3](https://arxiv.org/html/2503.15380v1).

For a concrete **fixed-$n$ arithmetic example**, take $n=100$:

| Configuration | Safety: Byzantine budget | Liveness: combined faulty/offline budget | Two-round budget | Fast decision votes | Votes per slow stage |
| --- | ---: | --- | --- | ---: | ---: |
| Standard Simplex | $b\le33$ | $b+o\le33$ | — | — | 67 |
| Kudzu, $p=0$ | $b\le33$ | $b+o\le33$ | $b+o\le0$ | 100 | 67 |
| Banyan, $p=1$ | $b\le33$ | $b+o\le33$ | $b+o\le1$ | 99 | 67 |
| Kudzu, $p=12$ | $b\le25$ | $b+o\le25$ | $b+o\le12$ | 88 | 63 |
| Minimmit | $b\le19$ | $b+o\le19$ | $b+o\le19$ | 81 | — |

These are calculated thresholds under the preceding assumptions, not benchmarks. For example, Kudzu $p=0$ can lose its two-round guarantee with one withholding node while retaining safety and eventual liveness with up to 33 faulty/offline nodes in total. The last two rows reduce both the stated Byzantine safety budget and the total-fault liveness budget; holding the required guarantees fixed may instead require increasing $n$.

**Encoded QC size is a separate question.** With signature length $s$, an explicit list of $q$ signatures costs roughly $qs$ plus signer identifiers; an aggregate signature with a bitmap costs roughly $s+\lceil n/8\rceil$ bytes plus metadata; a threshold signature can make the signature component constant-sized. Timeout reports, ancestry, and a sequence of SCs can still add substantial evidence. Different quorum thresholds must remain cryptographically distinguishable: Minimmit §6.4 explicitly discusses separate threshold keys for its small and large certificates. [Signature and QC discussion in Note 17](17_two_round_bft_literature_review_zh.md#33-签名聚合解决什么)

### 4.4 Existing evaluations: geographic setup matters

The following are published results, not experiments reproduced here. Compare protocols **within the same study**; the studies use different machines, workloads, fault budgets, and finality observers.

#### A. Banyan: real AWS WAN experiments

The Go/Bamboo prototype uses AWS **t3.large** machines, 2 vCPUs and 8 GB RAM, with 120-second runs. Its three geographic layouts are distinct:

| Layout | Placement | Workload / result |
| --- | --- | --- |
| Four global datacenters | 19 replicas, distributed **5/5/5/4** | 400 KB blocks: ICC **239 ms**; Banyan $(f,p)=(6,1)$ **216 ms**; Banyan $(4,4)$ **179 ms** |
| Global spread | 19 replicas across **19 worldwide AWS datacenters** | 1 MB blocks: ICC **384 ms**; Banyan $(6,1)$ **362 ms**; Banyan $(4,4)$ **324 ms** |
| US crash experiment | 19 replicas across **four US datacenters** | Configured timeout **3 seconds**; under crashes, performance falls back to ICC behavior |

Locations are mapped in the paper's Figure 5; the main text does not enumerate the region IDs. The metric is mean proposal finalization measured at the proposer. In the $(6,1)$ configuration, fast finality requires **18/19** votes, while each slow stage needs **13/19**. Thus the fast path reaches every datacenter, whereas the smaller quorums can omit a distant cluster. The $(4,4)$ result uses a lower Byzantine budget. [Banyan §§9.2–9.5](https://arxiv.org/html/2312.05869v3)

#### B. Simplex / Kudzu / Minimmit: the same geographic simulator

The Minimmit study uses **50 simulated processors** and ten virtual AWS regions:

| Geographic location | AWS region |
| --- | --- |
| Northern California, US | `us-west-1` |
| Northern Virginia, US | `us-east-1` |
| Ireland | `eu-west-1` |
| Stockholm | `eu-north-1` |
| Frankfurt | `eu-central-1` |
| Tokyo | `ap-northeast-1` |
| Mumbai | `ap-south-1` |
| Seoul | `ap-northeast-2` |
| São Paulo | `sa-east-1` |
| Sydney | `ap-southeast-2` |

**Uniform:** five processors per region. **US-concentrated:** 13 in California, 12 in Virginia, and the remaining 25 spread across the other eight regions. Each processor has symmetric **1 Gbps** ingress/egress. Delays are sampled with measured inter-region p50 as the mean and p90−p50 as the standard deviation; transmission uses max-min bandwidth sharing. These are simulations, not 50 deployed AWS engines. [Minimmit §7](https://arxiv.org/html/2508.10862v5)

| Placement and payload | Metric, ms | Simplex | Kudzu | Minimmit |
| --- | --- | ---: | ---: | ---: |
| Uniform, 32 KB, no coding | Mean block finality | 299.34 | 220.31 | 220.30 |
| US-concentrated, 32 KB, no coding | Mean block finality | 222.32 | 185.16 | 187.67 |
| Uniform, 1 MB, erasure coded | Mean block finality | 335.36 | 251.29 | 290.32 |

No faults are injected; block production is instantaneous and votes carry digests. Simplex uses different Byzantine safety and total-fault liveness budgets from the two other configurations. The paper's “transaction latency” adds one view interval to block finality; it is not measured application execution latency. The US-concentrated result illustrates the concurrent benefit: **Kudzu's smaller-quorum slow path can beat waiting for the larger fast quorum**. The coded large-block result also depends on fragmentation parameters. [Minimmit Tables 3, 4, and 6](https://arxiv.org/html/2508.10862v5)



### 4.5 What this means for our regional deployment

Proposed steps: 
- by revise the commonware implementation on the modular simplex, we compare based on our real world depolyment requirement. 
- a shorter block gap seems 
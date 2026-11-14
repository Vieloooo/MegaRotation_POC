# Module 08: Data Availability (v9 working draft v0.1)

Date: 2026-11-02. The overview and shared model are in [00 arch](00_arch_en_v0_1.md); the sources are [arch v0.2 §1.5, §4.5 DAObject](../formalization_v8/arch_zh_v0_2.md) and [Part 0 v0.8 §2.5](../formalization_v6/part0_model_goals_zh_v0_8.md).

## 1. Role and intuition

DA is an external service that stores batch bytes, so L1 need only record a short commitment. The three objects have distinct roles:

- **Commitment $d$** identifies which data to retrieve.
- **Certificate $\pi_{\rm DA}$** proves that $d$ has been certified under the DA rules and can therefore be retrieved during the retention period.
- **Payload $y$** is the retrieved bytes, which Derivation decodes and executes.

Certificate verification and payload execution are separate steps: the certificate only states that the data can be retrieved, not that its contents are valid under the protocol. The Certifier and Derivation check content validity.

This file expresses the model in [00 §1.5](00_arch_en_v0_1.md) as interfaces and numbered properties.

## 2. Minimal assumptions

**Model assumptions used by this module.**

- Hash and signature assumptions ([00 §1.1](00_arch_en_v0_1.md)): commitment binding reduces to these assumptions, allowing a negligible probability of failure.
- Time parameters $T_{\rm DA},\Delta_{\rm DA},T_{\rm ret}>0$ ([00 §1.5](00_arch_en_v0_1.md)).

DA is an external service and does not itself depend on other modules. Retrievability after the retention period is not provided by DA; it depends on the custody assumption below.

**A-CUSTODY (custody, used only for liveness).** For every batch payload accepted by the Batch Inbox, after the DA retention period ends, at least one honest custodian (such as an honest Batcher or derivation node) retains the same bytes at least until a subsequent derivation checkpoint, RESUME cut, or settlement checkpoint has covered them and neither derivation nor recovery needs to read them anymore.

**Upstream inputs and downstream outputs.**

| Direction | Other module and interface | Object exchanged | Connection |
| --- | --- | --- | --- |
| Input | Batcher.`Deliver` | Submit bytes $y$ | W16, W17 |
| Output | Batcher → Certifier, Inbox | $(d,\pi_{\rm DA})$ | W17 |
| Output | Derivation.`Derive` / `DeriveAtResume`, Certifier | DAObject | W21 |

## 3. Interfaces

### 3.1 State

DA maintains a storage table $\mathsf{Store}:d\mapsto(y,\tau_{\rm cert}(d))$, where $\tau_{\rm cert}(d)$ is the time at which certification of $d$ completes. Entries are only added, never modified. $\mathsf{Store}$ is an abstraction for analysis; readers access it only through the interfaces below.

### 3.2 Data structure

```text
DAObject = {
    commitment, payload, certificate
}
```

All three fields are complete bit strings, corresponding to $d,y,\pi_{\rm DA}$. A DAObject represents data that has been obtained and checked: the certificate passes CheckDA, and the payload opens the commitment under the DA scheme's binding relation. An empty payload is not used to represent data that has not yet been downloaded.

### 3.3 Interface overview

| Interface | Main inputs | Main outputs | Purpose |
| --- | --- | --- | --- |
| `Disperse` | Payload $y$ | $(d,\pi_{\rm DA})$ or WAIT | Submit data and obtain certification |
| `CheckDA` | $d$, $\pi_{\rm DA}$ | 0 / 1 | Publicly verify a certificate |
| `Retrieve` | $d$ | DAObject or WAIT | Retrieve the complete payload |

### 3.4 Disperse(y)

**Input.** A finite bit string $y$ satisfying DA's admission conditions (size and fee).

**Output.** Returns $(d,\pi_{\rm DA})$ once certification completes; returns WAIT before completion.

**Processing.** Compute the commitment, complete the dispersal and certification required by the DA rules, and record $(y,\tau_{\rm cert}(d))$ in $\mathsf{Store}$. Retrying the same $y$ returns the same $d$.

### 3.5 CheckDA(d, π_DA)

**Input.** Commitment $d$ and certificate $\pi_{\rm DA}$.

**Output.** 1 if $\pi_{\rm DA}$ proves that $d$ was certified under the DA rules; 0 otherwise.

**Processing.** A public, deterministic verification predicate. It requires neither access to $\mathsf{Store}$ nor a synchronized clock.

### 3.6 Retrieve(d)

**Input.** Commitment $d$.

**Output.** A DAObject; returns WAIT if the data has not yet been obtained.

**Processing.** Obtain $y$, check that it opens $d$, and return it. Empty content or other bytes cannot be substituted for unknown data.

## 4. Properties

DA-S1 and DA-S2 are safety properties and hold for all executions; DA-L1, DA-L2, and DA-RET are liveness properties. DA unavailability affects only liveness.

**DA-S1 — Binding (safety).** Except with negligible probability, each $d$ can be validly opened to at most one payload; `Retrieve(d)` returns only that payload.

*Intuition:* Once the Certifier has signed $d$, the bytes accepted on L1 are uniquely determined; different derivation nodes do not retrieve different contents.

**DA-S2 — Certificate consistency (safety).** CheckDA is a public, deterministic predicate: all verifiers obtain the same result for the same $(d,\pi_{\rm DA})$; if $d$ is certified, $\tau_{\rm cert}(d)$ is the same for all participants.

**DA-L1 — Dispersal latency (liveness).** If DA is available over $[\tau,\tau+T_{\rm DA}]$, a `Disperse(y)` submitted at $\tau$ that satisfies the admission conditions returns $(d,\pi_{\rm DA})$ by $\tau+T_{\rm DA}$, and $\mathsf{CheckDA}(d,\pi_{\rm DA})=1$.

**DA-L2 — Retrieval within retention (liveness).** If $d$ is certified, $\tau\le\tau_{\rm cert}(d)+T_{\rm ret}$, and DA is available over $[\tau,\tau+\Delta_{\rm DA}]$, then a `Retrieve(d)` issued at $\tau$ returns a DAObject by $\tau+\Delta_{\rm DA}$.

**DA-RET — Retrievability of accepted data (liveness).** Every batch payload accepted by the Batch Inbox can be retrieved while Derivation, state preparation, or RESUME still needs it. This has two parts: DA-L2 guarantees retrieval during the retention period $T_{\rm ret}$; A-CUSTODY guarantees it afterwards. DA-RET is a composed system-level guarantee, not a property provided by the DA service alone.

If DA-RET is violated, derivation stops and waits at that input, rather than treating unknown content as empty. The forced-inclusion path and RESUME that depend on derivation also wait (R-WAIT in [09 §3.4](09_finality_rules_en_v0_2.md)). Safety is unaffected.

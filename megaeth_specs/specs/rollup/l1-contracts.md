---
description: MegaETH deltas to the OP Stack L1 contract set.
---

# L1 Contracts

{% hint style="warning" %}
This page is a stub — the normative specification has not been written yet.
The L1 contract deltas [block certification](../block-certification/overview.md) introduces are unstable and not activated on any network, and are specified on the pages linked below rather than here.
{% endhint %}

Block certification upgrades the implementation behind the [fixed batch inbox proxy](../block-certification/batch-inbox.md#proxy-address-and-upgrade), adding certification acceptance and a registry of permitted submitters, and adds a certifier verifier — embedded in the inbox or a separate contract under the same super admin — that holds the set of enclave measurements that may hold a certifier key and the keys registered under them.
Each of those deltas is specified where the behavior relying on it is specified: the inbox contract itself, including the state it maintains and the acceptance rules it applies, by the [L1 Batch Inbox](../block-certification/batch-inbox.md) page; the registration of certifier keys against an authorized measurement and the revocation of a measurement, by that page's [certifier key registration](../block-certification/batch-inbox.md#certifier-key-registration) rules; and the registry of permitted senders the inbox itself holds, by its [batcher registry](../block-certification/batch-inbox.md#batcher-registry) rules.

The remaining deltas — portal and system configuration changes and protocol-address protections — have not been written.

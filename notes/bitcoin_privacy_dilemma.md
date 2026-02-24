**Privacy Dilemma**

When I started building this wallet, the primary objective was uncompromising privacy and security. In theory, the path was clear. In practice, the trade-offs turned out to be much harsher than expected — especially in terms of usability.

### The core problem

Most mainstream wallets operate in a straightforward way: they send your addresses to public nodes to query UTXOs and balances. The consequence is obvious — the remote server sees your IP address and learns exactly which addresses belong to you and how much you hold. From a privacy standpoint, this is effectively zero. Howewer, the user can route traffic through the Tor network, which allows even centralized RPC calls to be used without revealing the real IP. This means that privacy can be maintained at the network layer regardless of whether the client is fully P2P or relies on a centralized server.

To mitigate privacy issues, light client protocols BIP 157/158 were introduced years ago. Instead of querying addresses directly, the client connects over P2P, downloads block headers and compact block filters (Golomb-Rice filters), and performs matching locally on the user’s device. Filters contain compressed representations of transaction data in each block and are typically only a few kilobytes in size. The client downloads full blocks only when a filter match occurs. In this model, no external party learns which addresses belong to the user.


### The cost

The downside is resource consumption and sync time.

Syncing approximately four years of history (starting from the activation of Taproot) took over an hour and consumed roughly 2 GB of disk space. And the blockchain continues to grow.

If we consider users in developing countries — where cryptocurrency often serves as protection against inflation, but bandwidth is slow and expensive — this becomes a serious usability barrier. Long initial sync times and significant disk requirements are not acceptable for most users. Realistically, the overwhelming majority will choose a fast, convenient wallet over a maximally private one.

### The storage dilemma

There is an additional architectural tension:

* Storing headers and filters for ~4 years already approaches ~6 GB of disk usage in practice. At that point, calling this a “light client” becomes questionable.
* If filters are not stored persistently, then importing a new child address later requires a full rescan. That resync process is slow and computationally heavy.
* One possible approach is pruning old headers and filters, but this introduces complexity and uncertainty around correctness, rescan guarantees, and UX expectations.

There is no trivial balance here.

### Strategic question

The fundamental question is product direction:

* Build a niche wallet optimized for maximal privacy, targeting technically aware users willing to tolerate sync cost and storage overhead.
* Or build a broadly usable wallet with fast onboarding and minimal storage footprint, accepting privacy compromises (e.g., server-side indexers, Electrum-style queries, or hybrid models).

The tension is not technical — it is philosophical and product-oriented. Privacy at protocol level is achievable. Privacy at scale, without sacrificing usability for constrained environments, is significantly harder.

At this stage, the open question is whether to optimize for ideological purity or for real-world adoption.

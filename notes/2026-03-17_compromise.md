# Development Update: Architectural Pivot to Electrum Integration

After re-evaluating the project’s long-term roadmap, I have decided to shift the primary synchronization strategy for the Bitcoin network. While the initial goal of implementing BIP-157/158 (Compact Block Filters) offered a sophisticated approach to privacy, I have decided to prioritize Electrum indexed nodes for the upcoming release.

## Rationale for the Pivot

My objective is to balance robust security with practical utility. For my current vision and development stage, I am prioritizing disk, network efficiency, and ease of use over the resource-intensive requirements of client-side filtering.

* Privacy via Proxying: I believe that high-level privacy can be effectively managed at the network layer. Integrating Tor proxying provides a more accessible and immediate privacy solution for sensitive transactions without the overhead of BIP-157.

* Threat Model & Risk Assessment: While large-scale surveillance via node-provider cooperation is a theoretical risk, I consider it a secondary concern compared to the immediate technical hurdles of maintaining a dual-sync architecture. Given the current regulatory landscape, a lean, functional implementation is my highest priority.

* Revised Roadmap: My immediate focus will be the robust implementation of Electrum synchronization and Tor integration. This ensures a faster, more reliable experience for users with limited hardware resources.

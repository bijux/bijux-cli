# Performance Evidence Contract

Performance evidence is split into workloads and baselines.

Requirements:
- Every perf scenario must declare `scenario_class`, workload class, baseline owner, and threshold owner.
- Every release-blocking perf scenario must declare a non-empty threshold reference.
- Advisory and experimental scenarios must not be marked release-blocking.
- Baselines must be versioned and tied to scenario identifiers.
- Perf metadata must declare an explicit release-relevant set.
- New perf scenarios require contract linkage and a threshold plan entry in metadata.

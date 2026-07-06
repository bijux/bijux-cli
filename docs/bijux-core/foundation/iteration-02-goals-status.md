# Iteration 02 Goal Status (bijux-core)

Source: repository foundation backlog baseline (reviewed on 2026-04-30).

Status legend: `not-started`, `in-progress`, `done`, `deferred`, `blocked`.

| Goal | Title | Status | Note |
| --- | --- | --- | --- |
| 1 | Freeze the workspace product map. | done | implemented in feat/deep-foundation iteration-02 |
| 2 | Separate released product truth from maintainer truth. | done | implemented in feat/deep-foundation iteration-02 |
| 3 | Enforce DAG crate dependency direction. | done | implemented in feat/deep-foundation iteration-02 |
| 4 | Enforce CLI crate dependency direction. | done | implemented in feat/deep-foundation iteration-02 |
| 5 | Define public versus internal modules. | done | implemented in feat/deep-foundation iteration-02 |
| 6 | Quarantine simulated product claims. | not-started | backlog baseline |
| 7 | Create a backlog-to-crate routing table. | done | implemented in feat/deep-foundation iteration-02 |
| 8 | Reduce root-level policy sprawl. | done | implemented in feat/deep-foundation iteration-02 |
| 9 | Codify version compatibility lanes. | done | implemented in feat/deep-foundation iteration-02 |
| 10 | Create the first hard release gate. | done | implemented in feat/deep-foundation iteration-02 |
| 11 | Stabilize the `bijux` root grammar. | done | implemented in feat/deep-foundation iteration-01 |
| 12 | Define the command envelope as product API. | in-progress | iteration-01 target |
| 13 | Define the error envelope as product API. | not-started | backlog baseline |
| 14 | Make official app mounting explicit. | not-started | backlog baseline |
| 15 | Harden legacy shim behavior. | done | implemented in feat/deep-foundation iteration-01 |
| 16 | Align Rust and Python entrypoints. | not-started | backlog baseline |
| 17 | Make root help app-aware. | not-started | backlog baseline |
| 18 | Standardize output mode handling. | done | implemented in feat/deep-foundation iteration-01 |
| 19 | Add script-safety rules. | done | implemented in feat/deep-foundation iteration-01 |
| 20 | Add root command explain. | done | implemented in feat/deep-foundation iteration-01 |
| 21 | Make layered config deterministic. | not-started | backlog baseline |
| 22 | Version the config schema registry. | not-started | backlog baseline |
| 23 | Add config diff and explain. | done | implemented in feat/deep-foundation iteration-01 |
| 24 | Harden config repair. | not-started | backlog baseline |
| 25 | Define plugin manifest contracts. | not-started | backlog baseline |
| 26 | Separate official apps from plugins. | done | implemented in feat/deep-foundation iteration-01 |
| 27 | Harden plugin process execution. | done | implemented in feat/deep-foundation iteration-01 |
| 28 | Make `doctor` actionable. | done | implemented in feat/deep-foundation iteration-02 |
| 29 | Add routing inventory export. | done | implemented in feat/deep-foundation iteration-01 |
| 30 | Make REPL behavior contract-driven. | not-started | backlog baseline |
| 31 | Freeze the graph spec header. | not-started | backlog baseline |
| 32 | Define node identity invariants. | not-started | backlog baseline |
| 33 | Define edge identity invariants. | not-started | backlog baseline |
| 34 | Define port contracts. | not-started | backlog baseline |
| 35 | Separate semantic kind from adapter kind. | not-started | backlog baseline |
| 36 | Define trigger-rule semantics. | not-started | backlog baseline |
| 37 | Define branch semantics. | not-started | backlog baseline |
| 38 | Define reducer and barrier semantics. | not-started | backlog baseline |
| 39 | Canonicalize graph bytes. | not-started | backlog baseline |
| 40 | Explain graph fingerprints. | not-started | backlog baseline |
| 41 | Make validation diagnostics precise. | not-started | backlog baseline |
| 42 | Reject ambiguous graphs early. | not-started | backlog baseline |
| 43 | Add semantic lint separate from strict validation. | not-started | backlog baseline |
| 44 | Add graph authoring examples. | not-started | backlog baseline |
| 45 | Harden the builder API. | not-started | backlog baseline |
| 46 | Add graph migration commands. | not-started | backlog baseline |
| 47 | Add graph package import validation. | not-started | backlog baseline |
| 48 | Add dry-plan surfaces. | not-started | backlog baseline |
| 49 | Add semantic graph diff. | not-started | backlog baseline |
| 50 | Add validation scale budgets. | not-started | backlog baseline |
| 51 | Freeze execution-plan schema. | not-started | backlog baseline |
| 52 | Preserve graph semantics in lowering. | not-started | backlog baseline |
| 53 | Make topological order deterministic. | not-started | backlog baseline |
| 54 | Define parameter resolution order. | not-started | backlog baseline |
| 55 | Separate plan warnings from refusals. | not-started | backlog baseline |
| 56 | Add planner capability negotiation. | not-started | backlog baseline |
| 57 | Add plan fingerprinting. | not-started | backlog baseline |
| 58 | Add dynamic expansion bounds. | not-started | backlog baseline |
| 59 | Add plan inspect output. | not-started | backlog baseline |
| 60 | Add planner benchmark gate. | not-started | backlog baseline |
| 61 | Freeze run state machine. | not-started | backlog baseline |
| 62 | Record readiness reasons. | not-started | backlog baseline |
| 63 | Respect trigger rules in scheduler. | not-started | backlog baseline |
| 64 | Make retries policy-driven. | not-started | backlog baseline |
| 65 | Implement idempotent cancellation. | not-started | backlog baseline |
| 66 | Define timeout semantics. | not-started | backlog baseline |
| 67 | Constrain concurrency. | not-started | backlog baseline |
| 68 | Recover after coordinator interruption. | not-started | backlog baseline |
| 69 | Emit heartbeats for long runs. | not-started | backlog baseline |
| 70 | Keep runtime behavior deterministic. | not-started | backlog baseline |
| 71 | Define adapter contract schema. | not-started | backlog baseline |
| 72 | Harden built-in const adapter. | not-started | backlog baseline |
| 73 | Harden built-in shell adapter. | not-started | backlog baseline |
| 74 | Clarify container adapter status. | not-started | backlog baseline |
| 75 | Clarify remote and distributed backend status. | not-started | backlog baseline |
| 76 | Add adapter failure taxonomy. | not-started | backlog baseline |
| 77 | Add adapter preflight. | not-started | backlog baseline |
| 78 | Add adapter sandbox boundaries. | not-started | backlog baseline |
| 79 | Add adapter SDK examples. | not-started | backlog baseline |
| 80 | Add backend capability matrix. | not-started | backlog baseline |
| 81 | Freeze run directory layout. | not-started | backlog baseline |
| 82 | Enforce governed artifact roots. | not-started | backlog baseline |
| 83 | Hash artifact content. | not-started | backlog baseline |
| 84 | Record materialization metadata. | not-started | backlog baseline |
| 85 | Enforce required outputs. | not-started | backlog baseline |
| 86 | Represent optional outputs honestly. | not-started | backlog baseline |
| 87 | Define cache key inputs. | not-started | backlog baseline |
| 88 | Reject unsafe cache hits. | not-started | backlog baseline |
| 89 | Record cache miss reasons. | not-started | backlog baseline |
| 90 | Verify evidence bundle integrity. | not-started | backlog baseline |
| 91 | Define failure classes. | not-started | backlog baseline |
| 92 | Persist stderr and exit details. | not-started | backlog baseline |
| 93 | Implement replay plan explain. | not-started | backlog baseline |
| 94 | Protect successful evidence during replay. | not-started | backlog baseline |
| 95 | Refuse incomplete replay. | not-started | backlog baseline |
| 96 | Implement run diff. | not-started | backlog baseline |
| 97 | Implement resume semantics. | not-started | backlog baseline |
| 98 | Add forced rerun controls. | not-started | backlog baseline |
| 99 | Add failure explain. | not-started | backlog baseline |
| 100 | Add minimal operator workflow docs. | not-started | backlog baseline |
| 101 | Version mount descriptors. | not-started | backlog baseline |
| 102 | Add app capability discovery. | not-started | backlog baseline |
| 103 | Add mounted app command parity. | not-started | backlog baseline |
| 104 | Add Python mounted app import checks. | not-started | backlog baseline |
| 105 | Add app route conflict resolution. | not-started | backlog baseline |
| 106 | Add deprecation lifecycle. | not-started | backlog baseline |
| 107 | Add command telemetry policy. | not-started | backlog baseline |
| 108 | Add install diagnostics bundle. | not-started | backlog baseline |
| 109 | Add shell completion contract. | not-started | backlog baseline |
| 110 | Add CLI SDK stability tests. | not-started | backlog baseline |
| 111 | Add typed subgraphs. | not-started | backlog baseline |
| 112 | Add matrix expansion. | not-started | backlog baseline |
| 113 | Add dataset partitions. | not-started | backlog baseline |
| 114 | Add optional upstream semantics. | not-started | backlog baseline |
| 115 | Add quorum triggers. | not-started | backlog baseline |
| 116 | Add branch convergence rules. | not-started | backlog baseline |
| 117 | Add decision artifacts. | not-started | backlog baseline |
| 118 | Add typed effects. | not-started | backlog baseline |
| 119 | Add non-cacheable semantics. | not-started | backlog baseline |
| 120 | Add semantic conformance profiles. | not-started | backlog baseline |
| 121 | Add resource-aware planning. | not-started | backlog baseline |
| 122 | Add pool placement hints. | not-started | backlog baseline |
| 123 | Add data-locality hints. | not-started | backlog baseline |
| 124 | Add cost explain. | not-started | backlog baseline |
| 125 | Add planner conflict detection. | not-started | backlog baseline |
| 126 | Add preflight as planning gate. | not-started | backlog baseline |
| 127 | Add plan normalization. | not-started | backlog baseline |
| 128 | Add plan package export. | not-started | backlog baseline |
| 129 | Add planning API surface. | not-started | backlog baseline |
| 130 | Add planner regression corpus. | not-started | backlog baseline |
| 131 | Add durable run queue. | not-started | backlog baseline |
| 132 | Add node leases. | not-started | backlog baseline |
| 133 | Add backpressure policy. | not-started | backlog baseline |
| 134 | Add circuit breakers. | not-started | backlog baseline |
| 135 | Add pause and resume controls. | not-started | backlog baseline |
| 136 | Add partial rerun selectors. | not-started | backlog baseline |
| 137 | Add checkpoint contracts. | not-started | backlog baseline |
| 138 | Add worker isolation policy. | not-started | backlog baseline |
| 139 | Add scheduler fairness. | not-started | backlog baseline |
| 140 | Add runtime admission policy. | not-started | backlog baseline |
| 141 | Version artifact schema descriptors. | not-started | backlog baseline |
| 142 | Add artifact promotion lifecycle. | not-started | backlog baseline |
| 143 | Add retention classes. | not-started | backlog baseline |
| 144 | Add evidence-safe cache garbage collection. | not-started | backlog baseline |
| 145 | Add cache corruption repair. | not-started | backlog baseline |
| 146 | Add cache import/export. | not-started | backlog baseline |
| 147 | Add portable run bundles. | not-started | backlog baseline |
| 148 | Add artifact lineage queries. | not-started | backlog baseline |
| 149 | Add artifact content inspection rules. | not-started | backlog baseline |
| 150 | Add artifact schema migration. | not-started | backlog baseline |
| 151 | Add unified event taxonomy. | not-started | backlog baseline |
| 152 | Add correlation IDs. | not-started | backlog baseline |
| 153 | Add timeline reconstruction. | not-started | backlog baseline |
| 154 | Add metrics contracts. | not-started | backlog baseline |
| 155 | Add compact run summaries. | not-started | backlog baseline |
| 156 | Add evidence completeness checks. | not-started | backlog baseline |
| 157 | Add cross-run comparison reports. | not-started | backlog baseline |
| 158 | Add flake and retry analysis. | not-started | backlog baseline |
| 159 | Add observability redaction. | not-started | backlog baseline |
| 160 | Add evidence API for apps. | not-started | backlog baseline |
| 161 | Rationalize DAG command groups. | not-started | backlog baseline |
| 162 | Add operator-first status output. | not-started | backlog baseline |
| 163 | Add selector grammar. | not-started | backlog baseline |
| 164 | Add pagination/filtering. | not-started | backlog baseline |
| 165 | Add diagnostics bundle for DAG runs. | not-started | backlog baseline |
| 166 | Add run history indexing. | not-started | backlog baseline |
| 167 | Add migration inspect. | not-started | backlog baseline |
| 168 | Add command-level preconditions. | not-started | backlog baseline |
| 169 | Add human output governance. | not-started | backlog baseline |
| 170 | Add docs-as-executable-recipes. | not-started | backlog baseline |
| 171 | Enforce filesystem allowlists. | not-started | backlog baseline |
| 172 | Enforce environment allowlists. | not-started | backlog baseline |
| 173 | Add network policy. | not-started | backlog baseline |
| 174 | Add command injection hardening. | not-started | backlog baseline |
| 175 | Add secret-bearing artifact rules. | not-started | backlog baseline |
| 176 | Add override audit trail. | not-started | backlog baseline |
| 177 | Add supply-chain inventory. | not-started | backlog baseline |
| 178 | Add trust classes. | not-started | backlog baseline |
| 179 | Add malformed-input fuzzing. | not-started | backlog baseline |
| 180 | Add dependency risk reporting. | not-started | backlog baseline |
| 181 | Set core latency budgets. | not-started | backlog baseline |
| 182 | Build large-graph corpora. | not-started | backlog baseline |
| 183 | Profile canonicalization. | not-started | backlog baseline |
| 184 | Profile scheduler churn. | not-started | backlog baseline |
| 185 | Profile artifact writes. | not-started | backlog baseline |
| 186 | Add memory ceilings. | not-started | backlog baseline |
| 187 | Add streaming output handling. | not-started | backlog baseline |
| 188 | Add run-history compaction. | not-started | backlog baseline |
| 189 | Add benchmark report governance. | not-started | backlog baseline |
| 190 | Add performance regression gates. | not-started | backlog baseline |
| 191 | Shrink overlarge modules. | not-started | backlog baseline |
| 192 | Replace stringly contracts. | not-started | backlog baseline |
| 193 | Add public API review. | not-started | backlog baseline |
| 194 | Align docs with crate contracts. | not-started | backlog baseline |
| 195 | Add compatibility fixtures. | not-started | backlog baseline |
| 196 | Add change-impact labels. | not-started | backlog baseline |
| 197 | Make release notes evidence-driven. | not-started | backlog baseline |
| 198 | Consolidate duplicate helpers. | not-started | backlog baseline |
| 199 | Add medium acceptance gate. | not-started | backlog baseline |
| 200 | Close Level 2 with a production candidate scenario. | not-started | backlog baseline |
| 201 | Make `bijux` the native home for official apps. | not-started | backlog baseline |
| 202 | Add language-neutral app contracts. | not-started | backlog baseline |
| 203 | Add command provenance. | not-started | backlog baseline |
| 204 | Add app workspace profiles. | not-started | backlog baseline |
| 205 | Add interactive discovery without magic. | not-started | backlog baseline |
| 206 | Add command impact preview. | not-started | backlog baseline |
| 207 | Add cross-app doctor. | not-started | backlog baseline |
| 208 | Add app compatibility dashboard. | not-started | backlog baseline |
| 209 | Add route stability scoring. | not-started | backlog baseline |
| 210 | Make automation friction low. | not-started | backlog baseline |
| 211 | Add formal graph normalization. | not-started | backlog baseline |
| 212 | Add typed intermediate contracts. | not-started | backlog baseline |
| 213 | Add service and sensor nodes. | not-started | backlog baseline |
| 214 | Add event-driven graph semantics. | not-started | backlog baseline |
| 215 | Add policy overlays. | not-started | backlog baseline |
| 216 | Add late-bound expansion. | not-started | backlog baseline |
| 217 | Add semantic subgraph library. | not-started | backlog baseline |
| 218 | Add semantic risk analysis. | not-started | backlog baseline |
| 219 | Add graph conformance profiles. | not-started | backlog baseline |
| 220 | Make graph semantics explainable. | not-started | backlog baseline |
| 221 | Add planner optimization modes. | not-started | backlog baseline |
| 222 | Add reversible optimizer passes. | not-started | backlog baseline |
| 223 | Add empirical partition tuning. | not-started | backlog baseline |
| 224 | Add transfer-cost modeling. | not-started | backlog baseline |
| 225 | Add capacity what-if planning. | not-started | backlog baseline |
| 226 | Add conflict-resolution explain. | not-started | backlog baseline |
| 227 | Add planner confidence scores. | not-started | backlog baseline |
| 228 | Add high-level intent synthesis. | not-started | backlog baseline |
| 229 | Add cross-run planning learning. | not-started | backlog baseline |
| 230 | Add planner transparency gate. | not-started | backlog baseline |
| 231 | Add multi-run scheduling. | not-started | backlog baseline |
| 232 | Add deadline-aware admission. | not-started | backlog baseline |
| 233 | Add graceful degradation. | not-started | backlog baseline |
| 234 | Add execution deduplication. | not-started | backlog baseline |
| 235 | Add noisy-node isolation. | not-started | backlog baseline |
| 236 | Add exactly-once claim boundaries. | not-started | backlog baseline |
| 237 | Add rolling upgrade safety. | not-started | backlog baseline |
| 238 | Add live control plane. | not-started | backlog baseline |
| 239 | Add worker protocol. | not-started | backlog baseline |
| 240 | Keep distributed features honest. | not-started | backlog baseline |
| 241 | Add queryable evidence graph. | not-started | backlog baseline |
| 242 | Add signed run bundles. | not-started | backlog baseline |
| 243 | Add evidence scoring. | not-started | backlog baseline |
| 244 | Add cache trust classes. | not-started | backlog baseline |
| 245 | Add deterministic output-change proof. | not-started | backlog baseline |
| 246 | Add long-lived archival bundles. | not-started | backlog baseline |
| 247 | Add artifact deduplication. | not-started | backlog baseline |
| 248 | Add evidence compression. | not-started | backlog baseline |
| 249 | Add external-review reports. | not-started | backlog baseline |
| 250 | Make replay reduce cognitive load. | not-started | backlog baseline |
| 251 | Add domain-neutral scientific artifact roles. | not-started | backlog baseline |
| 252 | Add sample and subject identity propagation. | not-started | backlog baseline |
| 253 | Add reference identity contracts. | not-started | backlog baseline |
| 254 | Add advisory versus enforced scientific findings. | not-started | backlog baseline |
| 255 | Add scientific run trust classes. | not-started | backlog baseline |
| 256 | Add cross-app evidence linking. | not-started | backlog baseline |
| 257 | Add scientific policy override audit. | not-started | backlog baseline |
| 258 | Add uncertainty surfaces. | not-started | backlog baseline |
| 259 | Add truth-set comparison hook. | not-started | backlog baseline |
| 260 | Make core safe for serious science. | not-started | backlog baseline |
| 261 | Add policy conformance bundles. | not-started | backlog baseline |
| 262 | Add operator-signed overrides. | not-started | backlog baseline |
| 263 | Add secret provenance controls. | not-started | backlog baseline |
| 264 | Add low-trust workload isolation. | not-started | backlog baseline |
| 265 | Add SBOM-to-run linking. | not-started | backlog baseline |
| 266 | Add vulnerability impact mapping. | not-started | backlog baseline |
| 267 | Add path authority proofs. | not-started | backlog baseline |
| 268 | Add forensic mode. | not-started | backlog baseline |
| 269 | Add compliance retention locks. | not-started | backlog baseline |
| 270 | Reduce core privilege. | not-started | backlog baseline |
| 271 | Benchmark against comparable workflow cores. | not-started | backlog baseline |
| 272 | Add continuous large-workflow replay. | not-started | backlog baseline |
| 273 | Optimize evidence write path. | not-started | backlog baseline |
| 274 | Optimize run-history queries. | not-started | backlog baseline |
| 275 | Add adaptive cache policy. | not-started | backlog baseline |
| 276 | Add resource prediction. | not-started | backlog baseline |
| 277 | Add saturation diagnostics. | not-started | backlog baseline |
| 278 | Add low-latency controls. | not-started | backlog baseline |
| 279 | Add binary-size and dependency budgets. | not-started | backlog baseline |
| 280 | Publish performance claims only from artifacts. | not-started | backlog baseline |
| 281 | Make crate ownership obvious. | not-started | backlog baseline |
| 282 | Make public contracts small and versioned. | not-started | backlog baseline |
| 283 | Eliminate passive drift. | not-started | backlog baseline |
| 284 | Make correct fixtures cheap. | not-started | backlog baseline |
| 285 | Unify error modeling. | not-started | backlog baseline |
| 286 | Unify evidence modeling. | not-started | backlog baseline |
| 287 | Keep `bijux-dev` secondary. | not-started | backlog baseline |
| 288 | Make docs concise but executable. | not-started | backlog baseline |
| 289 | Add contributor issue templates. | not-started | backlog baseline |
| 290 | Close Level 3 engineering with a clean-room contributor test. | not-started | backlog baseline |
| 291 | Prove root CLI excellence. | not-started | backlog baseline |
| 292 | Prove DAG semantic excellence. | not-started | backlog baseline |
| 293 | Prove runtime trust. | not-started | backlog baseline |
| 294 | Prove artifact lineage. | not-started | backlog baseline |
| 295 | Prove cache explainability. | not-started | backlog baseline |
| 296 | Prove replay explainability. | not-started | backlog baseline |
| 297 | Prove app ecosystem readiness. | not-started | backlog baseline |
| 298 | Prove release-grade compatibility. | not-started | backlog baseline |
| 299 | Prove operator usability. | not-started | backlog baseline |
| 300 | Make `bijux-core` worth building on. | not-started | backlog baseline |

# Root documentation audit: docs top-level files

| File | Topic | Audience | Owner | Disposition |
| --- | --- | --- | --- | --- |
| ACCEPTANCE_GATES.md | Quality and release gates | Maintainers | Reliability and docs owners | Move to `docs/reference/` |
| ADAPTERS.md | Adapter API surface and usage | Maintainers | Runtime and app teams | Move to `docs/reference/` |
| ADAPTER_SDK.md | SDK integration surface | Developers and adapters | Maintainers | Move to `docs/reference/` |
| ADAPTIVE_SCHEDULING_GUARDRAILS.md | Adaptive control safety | Maintainers | Scheduler platform team | Move to `docs/reports/ARCHIVED_EXPERIMENTAL_SCHEDULING.md` |
| ADAPTIVE_SCHEDULING_INTELLIGENCE.md | Adaptive scheduler behavior | Maintainers | Scheduler platform team | Move to `docs/reports/ARCHIVED_EXPERIMENTAL_SCHEDULING.md` |
| ADAPTIVE_SCHEDULING_SIMULATION.md | Adaptive scheduling simulation | Maintainers | Scheduler platform team | Move to `docs/reports/ARCHIVED_EXPERIMENTAL_SCHEDULING.md` |
| ADVANCED_DAG_SEMANTICS.md | Graph and planner semantics | Maintainers | Core spec team | Move to `docs/spec/` |
| AI_ASSISTED_GUARDRAILS.md | Operator AI guidance and safety | Operators | Docs + runtime maintainer | Move to `docs/reference/` |
| AI_ASSISTED_OPERATOR_WORKFLOWS.md | Operator workflow AI support | Operators | Docs + runtime maintainer | Move to `docs/reference/` |
| API_CONTRACT.md | API contract summary | Maintainers | Platform contract owner | Move to `docs/spec/` |
| ARCHITECTURE.md | Architecture boundary map overview | Maintainers and newcomers | Architecture team | Merge into `docs/architecture/README.md` |
| ARTIFACT_PLATFORM.md | Artifact execution platform semantics | Maintainers | Artifacts and runtime owners | Move to `docs/spec/` |
| ARTIFACT_SYSTEM.md | Artifact system model | Maintainers | Artifacts and runtime owners | Move to `docs/spec/` |
| AUTH_IDENTITY_TRUST.md | Identity and trust model | Maintainers | Security owners | Move to `docs/reference/` |
| BACKEND_EXECUTION_MATURITY.md | Backend execution readiness | Maintainers | Runtime maintainer | Move to `docs/spec/` |
| BACKEND_POLICY_OVERLAYS.md | Policy overlay model | Maintainers | Policy owners | Move to `docs/reference/` |
| CACHE_SEMANTICS.md | Cache contract behavior | Maintainers | Runtime owner | Merge into `docs/spec/` |
| CLI.md | CLI user/operations guide and contracts | Operators and maintainers | CLI teams | Keep and merge CLI docs |
| CLI_BACKWARD_COMPAT.md | CLI compatibility policy | Maintainers | CLI team | Merge into `docs/CLI.md` |
| CLI_COMMAND_TAXONOMY.md | CLI command organization | Operators and maintainers | CLI teams | Move to `docs/reference/COMMAND_TAXONOMY.md` |
| COMPATIBILITY.md | Compatibility model | Maintainers and users | Core architecture owners | Keep and merge compatibility docs |
| COMPATIBILITY_WINDOW_V0.1.md | v0.1 compatibility details | Maintainers | Compatibility owners | Merge into `docs/COMPATIBILITY.md` |
| CONTROL_PLANE.md | Control-plane boundaries | Maintainers | Architecture team | Move to `docs/architecture/CONTROL_PLANE.md` |
| CONTROL_PLANE_MIGRATION.md | Control-plane migration plan | Maintainers | Architecture team | Merge to `docs/architecture/CONTROL_PLANE.md` |
| COST_AWARE_SCHEDULING.md | Cost-aware scheduling model | Maintainers | Scheduler and finance teams | Archive as reference in `docs/reports/` |
| COST_OPTIMIZATION_SIMULATION.md | Cost simulation scenarios | Maintainers | Scheduler and finance teams | Move to `docs/reports/ARCHIVED_EXPERIMENTAL_SCHEDULING.md` |
| COST_OBSERVABILITY_REPORTS.md | Cost metrics evidence references | Maintainers | Operations and docs | Move to `docs/reference/` |
| DAG_API_SERVICE.md | API service integration notes | Maintainers | API owners | Move to `docs/reference/` |
| DAG_EXPLAIN_AND_PREVIEW.md | Explain/preview workflow | Operators | Docs + CLI teams | Move to `docs/user/` |
| DATASET_CATALOG_QUERY_MODEL.md | Dataset catalog queries | Operators | Data platform team | Merge into `docs/` archival dataset package |
| DATASET_EXAMPLE_WORKFLOWS.md | Dataset workflow examples | Operators | Data platform team | Merge into `docs/reports/` archived examples |
| DATASET_NATIVE_SEMANTICS.md | Dataset semantics | Operators and maintainers | Data platform team | Move to `docs/reference/` |
| DEPLOYMENT_BACKENDS.md | Backend capability matrix | Maintainers and operators | Platform operations | Keep in `docs/reference/` (move) |
| DEPRECATION_POLICY.md | Deprecation process | Maintainers | Docs and architecture owners | Move to `docs/reference/` |
| DEVELOPMENT.md | Contributor setup and tooling | Developers | Docs owners | Move to `docs/dev/` |
| DISTRIBUTED_EXECUTION_FOUNDATIONS.md | Runtime execution assumptions | Maintainers | Architecture team | Merge into `docs/architecture/` |
| DOCS_GENERATION_PLAN.md | Docs generation planning | Maintainers | Docs owners | Archive in `docs/reports/` |
| ECOSYSTEM_ADOPTION_GUIDE.md | Ecosystem positioning | Users and maintainers | Product owners | Move to `docs/user/` |
| ECOSYSTEM_PACKAGING_STRATEGY.md | Ecosystem packaging decisions | Maintainers | Product and releases | Archive `docs/reports/` unless active |
| ECOSYSTEM_RELEASE_NOTES_POLICY.md | Release notes process | Maintainers | Release owners | Move to `docs/reference/` |
| EFFECTS.md | Node and shell effects | Maintainers and operators | Runtime and CLI teams | Move to `docs/architecture/EFFECTS.md` |
| ENVIRONMENT_CAPABILITIES.md | Environment capability model | Operators | Operations owners | Move to `docs/reference/` |
| ENVIRONMENT_SCALE_PROFILES.md | Environment profile expectations | Operators | Operations owners | Move to `docs/reference/` |
| EXAMPLES.md | Authoring and usage examples | Users | Docs owners | Keep in `docs/user/` |
| EXPERIENCE_SURFACES.md | Product interaction surfaces | Maintainers | Product documentation | Archive in `docs/reports/` |
| EXTENSION_CATALOG_CONTRACTS.md | Extension catalog contracts | Maintainers | Platform extension owners | Move to `docs/spec/` |
| FEDERATED_SCHEDULING_ORCHESTRATION.md | Federation scheduling design | Maintainers | Architecture team | Merge into `docs/architecture/` |
| FEDERATED_SCHEDULING_SIMULATION.md | Federation simulation scenarios | Maintainers | Architecture team | Move to `docs/reports/ARCHIVED_EXPERIMENTAL_SCHEDULING.md` |
| FEDERATION_CONFORMANCE_GATES.md | Federation evidence gates | Maintainers | Architecture team | Merge into `docs/reference/` |
| FIXTURE_GOVERNANCE.md | Fixture governance | Maintainers | Test owners | Move to `docs/testing/` |
| FORMAL_ASSURANCE_ROADMAP.md | Assurance roadmap | Maintainers | Quality owners | Archive in `docs/reports/` |
| FORMAL_VERIFICATION_FRAMEWORK.md | Verification framework | Maintainers | Quality owners | Move to `docs/reference/` |
| GEO_FEDERATED_CONTROL_PLANE.md | Geo control-plane model | Maintainers | Platform architecture team | Merge into `docs/architecture/` |
| GEO_FEDERATION_DISASTER_RECOVERY.md | Geo DR playbook | Operators | Operations team | Move to `docs/operations/` |
| GEO_READY_ACCEPTANCE_GATES.md | Geo acceptance criteria | Maintainers | Operations team | Move to `docs/reference/` |
| HA_SCHEDULER_COORDINATION.md | HA scheduler contracts | Maintainers | Platform architecture team | Merge into `docs/architecture/` |
| LINEAGE_POLICY_HOOKS.md | Lineage policy points | Operators | Runtime and security owners | Move to `docs/reference/` |
| MAKE_DEV_RELATIONSHIP.md | Developer ownership process | Maintainers | Documentation owners | Archive in `docs/reports/` |
| MEMORY_BUDGET.md | Memory budget evidence | Maintainers | Runtime owners | Move to `docs/reports/` |
| MULTI_TENANT_ISOLATION.md | Tenant isolation model | Operators and maintainers | Runtime/security owners | Move to `docs/reference/` |
| OBSERVABILITY.md | Runtime observability and diagnostics | Operators and maintainers | Runtime and ops owners | Keep and merge observability cluster |
| OWNERSHIP.md | Ownership model | Maintainers | Governance owners | Keep and merge with roadmap ownership |
| PERFORMANCE_BASELINE.md | Benchmark baseline protocol | Maintainers | Perf and quality owners | Move to `docs/reference/` |
| PERFORMANCE_CAPACITY_ENGINEERING.md | Capacity engineering model | Maintainers | Capacity planning owners | Move to `docs/reference/` |
| PLANNER_ANALYSIS.md | Planner behavior and phases | Maintainers | Runtime planning owners | Move to `docs/spec/` |
| PLATFORM_INVARIANTS.md | Platform invariants | Maintainers | Governance owners | Move to `docs/architecture/` |
| PLATFORM_OPERATING_MODEL.md | Platform operating model | Maintainers | Governance owners | Archive in `docs/reports/` |
| PLATFORM_SUSTAINABILITY_GOVERNANCE.md | Governance model | Maintainers | Governance owners | Archive in `docs/reports/` |
| PLUGIN_DSL_ROADMAP.md | Plugin DSL roadmap | Maintainers | Plugin owners | Archive in `docs/reports/` |
| PLUGIN_SDK_EXAMPLES.md | Plugin SDK usage examples | Developers and maintainers | Plugin owners | Move to `docs/reference/PLUGIN_SDK_EXAMPLES.md` |
| POLICY.md | Policy gates and flags | Operators and maintainers | Runtime and security owners | Merge into `docs/SECURITY.md` |
| RBAC_AUTHZ_POLICY.md | RBAC and authorization model | Operators and maintainers | Security owners | Move to `docs/reference/` |
| README.md | Documentation entrypoint | All | Docs owner | Keep (entrypoint) |
| REGULATED_WORKFLOW_REFERENCE.md | Regulated workflow guidance | Operators and maintainers | Safety and governance owners | Move to `docs/reference/` |
| RELEASE_PROCESS.md | Release process | Maintainers | Release owners | Move to `docs/reference/` |
| REPLAY_GUARANTEES.md | Replay behavior guarantees | Operators and maintainers | Runtime owners | Keep and merge details from related docs |
| REPO_CONSTITUTION.md | Repository governance rules | Maintainers | Docs owners | Archive in `docs/reports/` |
| REPO_GUARDRAILS.md | Repository evidence and guardrails | Maintainers | Docs owners | Move to `docs/reference/` |
| RESOURCE_PROFILE_NOTES.md | Resource claim notes | Maintainers | Capacity owners | Move to `docs/reference/` |
| RESOURCE_PROFILE_TRENDING.md | Trending resource patterns | Maintainers | Capacity owners | Merge into `docs/reports/` |
| ROADMAP_OWNERSHIP.md | Ownership roadmap | Maintainers | Governance owners | Merge into `docs/OWNERSHIP.md` |
| RUN_RECOVERY_AND_RESILIENCE.md | Recovery model and run behavior | Operators and maintainers | Runtime owners | Merge into `docs/spec/` |
| RUST_DAG_API_COMPATIBILITY.md | Rust API compatibility checks | Maintainers | API owners | Archive in `docs/reports/` |
| SCHEDULER_MVP.md | Scheduler v1 scope | Maintainers | Scheduler owners | Archive `docs/reports/` unless active |
| SCHEDULER_WORKLOAD_MANAGEMENT.md | Scheduler workload policy | Maintainers | Scheduler owners | Move to `docs/spec/` |
| SECRET_LEAK_INCIDENT_PLAYBOOK.md | Incident process | Operators and security owners | Security owners | Move to `docs/operations/` |
| SECURE_DAG_AUTHORING.md | Secure DAG patterns | Operators and maintainers | Security owners | Merge into `docs/SECURITY.md` |
| SECURITY.md | Security policy and controls | Operators and maintainers | Security owners | Keep and merge security cluster |
| SEMANTIC_LINEAGE_EXPLAINABILITY.md | Semantic lineage explainability | Operators and data teams | Runtime owners | Move to `docs/reference/` |
| SEMANTIC_LINEAGE_INTELLIGENCE.md | Semantic lineage intelligence | Maintainers | Runtime owners | Move to `docs/reports/` (archival evidence) |
| SUPPLY_CHAIN_TRUST.md | Supply chain trust model | Operators and maintainers | Security owners | Move to `docs/reference/` |
| TASK_CONTRACT_TYPES.md | Task type contract taxonomy | Maintainers | Runtime and CLI owners | Move to `docs/spec/` |
| TESTING.md | Testing and test operations | Maintainers | QA and runtime owners | Keep and merge test taxonomy/topology |
| TEST_TAXONOMY.md | Test taxonomy | Maintainers | QA owners | Merge into `docs/TESTING.md` |
| TEST_TOPOLOGY.md | Test topology and fixture placement | Maintainers | QA owners | Merge into `docs/TESTING.md` |
| UPGRADE_COMPATIBILITY_GOVERNANCE.md | Upgrade and compatibility governance | Maintainers | Architecture and platform owners | Merge into `docs/COMPATIBILITY.md` |
| VERIFICATION_GATES.md | Gate checklist and evidence | Maintainers | QA owners | Move to `docs/reference/` |
| WORKFLOW_INNOVATION_ROADMAP.md | Future workflow roadmap | Maintainers | Product owners | Archive `docs/reports/` |
| WORKFLOW_OPERATING_SYSTEM.md | Product positioning | Maintainers | Product owners | Move to `docs/reference/` |
| WORK_STEALING_SCHEDULER_BOUNDARIES.md | Scheduler behavior boundary | Maintainers | Scheduler owners | Move to `docs/spec/` |
| INDEX.md | Root discovery entrypoint | All | Docs owner | Keep and reduce to high-signal map |

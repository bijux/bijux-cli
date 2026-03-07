# Naming audit record

## Purpose

Track naming debt and the normalized replacement vocabulary used across runtime surfaces.

## Roadmap-style and speculative names removed from runtime

- `planner_intelligence` -> `planner_analysis`
- `ecosystem_productization` -> `distribution_readiness`
- `scheduler_enterprise` -> `scheduler_workload`
- `plugin_ecosystem` -> `extension_catalog`
- `InnovationRoadmap` -> `EvolutionPlan`

## Current naming classes

- `core`: engine, scheduler, state, execution, cache
- `support`: adapters, extension catalog, observability, recovery
- `speculative`: not allowed in runtime module tree
- `misplaced`: must move to docs or governance crates

## Follow-up checks

- runtime module names must satisfy `docs/spec/NAMING_GUIDELINES.md`
- docs index labels must match canonical names
- tests and fixtures must follow renamed canonical surfaces

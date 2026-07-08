---
title: Generated Config Reference
audience: mixed
type: generated-reference
status: canonical
owner: bijux-cli-docs
generated_from: bijux-cli-config-schema-registry-v1
---

# Generated Config Reference

This page is generated from the built-in `bijux-cli` config schema registry.
Use `bijux config docs --format json` when you need the same content from the runtime.

## `agent`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `agent.profile` | `agent_profile` | `string` | `BIJUX_AGENT_PROFILE` | `no` | - | `active` | Named runtime profile for the mounted app. |
| `agent.workspace_dir` | `agent_workspace_dir` | `path` | `BIJUX_AGENT_WORKSPACE_DIR` | `no` | - | `active` | Preferred working directory for app execution and outputs. |
| `agent.log_level` | `agent_log_level` | `string` | `BIJUX_AGENT_LOG_LEVEL` | `no` | `info` | `active` | Per-app log verbosity override. |

## `atlas`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `atlas.profile` | `atlas_profile` | `string` | `BIJUX_ATLAS_PROFILE` | `no` | - | `active` | Named runtime profile for the mounted app. |
| `atlas.workspace_dir` | `atlas_workspace_dir` | `path` | `BIJUX_ATLAS_WORKSPACE_DIR` | `no` | - | `active` | Preferred working directory for app execution and outputs. |
| `atlas.log_level` | `atlas_log_level` | `string` | `BIJUX_ATLAS_LOG_LEVEL` | `no` | `info` | `active` | Per-app log verbosity override. |

## `cli`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cli.color` | `cli_color` | `string` | `BIJUXCLI_COLOR`<br>`BIJUX_CLI_COLOR` | `no` | `auto` | `active` | ANSI color policy for CLI text output. |
| `cli.log_level` | `cli_log_level` | `string` | `BIJUXCLI_LOG_LEVEL`<br>`BIJUX_CLI_LOG_LEVEL` | `no` | `info` | `active` | Global CLI log verbosity. |
| `cli.output_format` | `cli_output_format` | `string` | `BIJUXCLI_FORMAT`<br>`BIJUX_CLI_FORMAT` | `no` | - | `active` | Preferred machine output format. |
| `cli.profile` | `cli_profile` | `string` | `BIJUXCLI_PROFILE`<br>`BIJUX_PROFILE` | `no` | - | `active` | Selected named config profile. |
| `cli.access_token` | `cli_access_token` | `string` | `BIJUXCLI_ACCESS_TOKEN`<br>`BIJUX_CLI_ACCESS_TOKEN` | `yes` | - | `active` | Operator access token for authenticated CLI control-plane integrations. |

## `dag`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `dag.cache_dir` | `dag_cache_dir` | `path` | `BIJUX_DAG_CACHE_DIR` | `no` | - | `active` | Local DAG cache directory. |
| `dag.adapters_dir` | `dag_adapters_dir` | `path` | `BIJUX_DAG_ADAPTERS_DIR` | `no` | - | `active` | Directory containing external DAG adapters. |
| `dag.jobs` | `dag_jobs` | `integer` | `BIJUX_DAG_JOBS` | `no` | - | `active` | Maximum DAG execution parallelism. |
| `dag.cache_mode` | `dag_cache_mode` | `string` | `BIJUX_DAG_CACHE_MODE` | `no` | - | `active` | DAG cache policy mode. |
| `dag.materialize_inputs` | `dag_materialize_inputs` | `boolean` | `BIJUX_DAG_MATERIALIZE_INPUTS` | `no` | `false` | `active` | Whether DAG runtime materializes node inputs eagerly. |
| `dag.policy_json` | `dag_policy_json` | `json` | `BIJUX_DAG_POLICY_JSON` | `no` | - | `active` | Structured DAG runtime policy override. |

## `dna`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `dna.profile` | `dna_profile` | `string` | `BIJUX_DNA_PROFILE` | `no` | - | `active` | Named runtime profile for the mounted app. |
| `dna.workspace_dir` | `dna_workspace_dir` | `path` | `BIJUX_DNA_WORKSPACE_DIR` | `no` | - | `active` | Preferred working directory for app execution and outputs. |
| `dna.log_level` | `dna_log_level` | `string` | `BIJUX_DNA_LOG_LEVEL` | `no` | `info` | `active` | Per-app log verbosity override. |

## `gnss`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `gnss.profile` | `gnss_profile` | `string` | `BIJUX_GNSS_PROFILE` | `no` | - | `active` | Named runtime profile for the mounted app. |
| `gnss.workspace_dir` | `gnss_workspace_dir` | `path` | `BIJUX_GNSS_WORKSPACE_DIR` | `no` | - | `active` | Preferred working directory for app execution and outputs. |
| `gnss.log_level` | `gnss_log_level` | `string` | `BIJUX_GNSS_LOG_LEVEL` | `no` | `info` | `active` | Per-app log verbosity override. |

## `rag`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `rag.profile` | `rag_profile` | `string` | `BIJUX_RAG_PROFILE` | `no` | - | `active` | Named runtime profile for the mounted app. |
| `rag.workspace_dir` | `rag_workspace_dir` | `path` | `BIJUX_RAG_WORKSPACE_DIR` | `no` | - | `active` | Preferred working directory for app execution and outputs. |
| `rag.log_level` | `rag_log_level` | `string` | `BIJUX_RAG_LOG_LEVEL` | `no` | `info` | `active` | Per-app log verbosity override. |

## `rar`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `rar.profile` | `rar_profile` | `string` | `BIJUX_RAR_PROFILE` | `no` | - | `active` | Named runtime profile for the mounted app. |
| `rar.workspace_dir` | `rar_workspace_dir` | `path` | `BIJUX_RAR_WORKSPACE_DIR` | `no` | - | `active` | Preferred working directory for app execution and outputs. |
| `rar.log_level` | `rar_log_level` | `string` | `BIJUX_RAR_LOG_LEVEL` | `no` | `info` | `active` | Per-app log verbosity override. |

## `vex`

| Logical key | Storage key | Type | Environment | Sensitive | Default | Deprecation | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `vex.profile` | `vex_profile` | `string` | `BIJUX_VEX_PROFILE` | `no` | - | `active` | Named runtime profile for the mounted app. |
| `vex.workspace_dir` | `vex_workspace_dir` | `path` | `BIJUX_VEX_WORKSPACE_DIR` | `no` | - | `active` | Preferred working directory for app execution and outputs. |
| `vex.log_level` | `vex_log_level` | `string` | `BIJUX_VEX_LOG_LEVEL` | `no` | `info` | `active` | Per-app log verbosity override. |


# Config Paths and Environment Variables

## Source of truth
- `src/bijux_cli/infra/paths.py`
- `src/bijux_cli/cli/core/constants.py`
- `src/bijux_cli/services/config/__init__.py`
- `src/bijux_cli/services/history/__init__.py`
- `src/bijux_cli/plugins/__init__.py`
- `src/bijux_cli/cli/external_binaries.py`

## Default filesystem paths
- `~/.bijux/.env`
- `~/.bijux/.history`
- `~/.bijux/.memory.json`
- `~/.bijux/.plugins/`

## Config path behavior
- Active config path resolves from `BIJUXCLI_CONFIG` or default `~/.bijux/.env`.
- History path resolves by precedence:
1. explicit service path
2. `BIJUXCLI_HISTORY_FILE`
3. sibling `.bijux_history` next to `BIJUXCLI_CONFIG`
4. default `~/.bijux/.history`
- Plugin directory resolves from `BIJUXCLI_PLUGINS_DIR` or default `~/.bijux/.plugins`.

## Environment variables observed in source
- `BIJUXCLI_ALLOWED_COMMANDS`
- `BIJUXCLI_ALLOWED_PRODUCT_BINS`
- `BIJUXCLI_API_GUARD`
- `BIJUXCLI_BIN`
- `BIJUXCLI_COLOR`
- `BIJUXCLI_COMMAND_TIMEOUT`
- `BIJUXCLI_CONFIG`
- `BIJUXCLI_DEV_MODE`
- `BIJUXCLI_DI_LIMIT`
- `BIJUXCLI_DISABLE_HISTORY`
- `BIJUXCLI_DOCS_DIR`
- `BIJUXCLI_DOCS_OUT`
- `BIJUXCLI_ENFORCE_PRODUCT_MAJOR_MATCH`
- `BIJUXCLI_HISTORY_FILE`
- `BIJUXCLI_LOG_LEVEL`
- `BIJUXCLI_MAX_WORKERS`
- `BIJUXCLI_PLUGINS_DIR`
- `BIJUXCLI_PRODUCT_BIN_DIR`
- `BIJUXCLI_PRODUCT_BIN_DIRS`
- `BIJUXCLI_PRODUCT_BIN_PRECEDENCE`
- `BIJUXCLI_TELEMETRY`
- `BIJUXCLI_TEST_DISK_FULL`
- `BIJUXCLI_TEST_FORCE_SERIALIZE_FAIL`
- `BIJUXCLI_TEST_FORCE_UNHEALTHY`
- `BIJUXCLI_TEST_IO_FAIL`
- `BIJUXCLI_TEST_MODE`
- `BIJUXCLI_VERSION`
- `NO_COLOR`

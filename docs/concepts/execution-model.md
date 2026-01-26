# Execution model

Backbone flow:

CLI -> Intent -> Policy -> Runtime -> Exit

Fast paths:

- --help
- --version

These return early without DI or plugins.

Decisions happen in core:

- output routing
- exit codes
- policy resolution

Decisions never happen in infra.

# Command Routing

Command routing maps one parsed command to one authorized application workflow.
Routing is deterministic and governed by release lanes, preconditions, and
output policy.

## Command Authority

`commands/cli_model.rs` is the typed command-tree authority. `dag_command`
constructs the public Clap surface from that model. Checked-in reference
documentation is generated from the same authority.

Do not maintain separate command inventories in route modules, docs
generators, completion code, or tests. Snapshots and references verify the
authority; they do not replace it.

## Surface Lanes

Routes are classified as stable, experimental, simulated, internal, or
maintainer. The selected lane controls discovery and access:

- stable routes form the default operator promise;
- experimental routes require explicit opt-in;
- simulated routes expose modeled behavior and cannot imply production
  support;
- internal routes support repository-owned workflows;
- maintainer routes remain outside the runtime command.

Presence in source does not make a route public. Hidden help alone is not an
authorization boundary; execution checks the same surface policy.

## Preconditions

A route validates command-level facts before domain execution:

- required graph, run, artifact, and destination arguments;
- configuration and deprecation rules;
- source and output path relationships;
- backend and lane availability;
- mutation confirmation or preview behavior;
- selected output mode.

An invalid explicit argument is reported. It is never replaced silently with a
profile or default.

## Path Rules

Routes distinguish graph source files, run roots, run identifiers, artifact
identifiers, cache roots, import bundles, and export destinations. Resolution
uses the owning path helpers.

Read-only commands do not create missing roots. Mutating commands reject
self-overwrite, source/destination overlap, traversal, and ambiguous run
lookup before domain calls.

## Dispatch Rules

- Route handlers do not panic on operator input.
- Exactly one command workflow runs.
- Access denial uses the selected output contract.
- Unknown or unavailable surfaces remain distinct from malformed arguments.
- Domain errors retain their class when converted to exit status.
- Help and no-subcommand behavior do not mutate state.

## Verification

Command-tree snapshots, route-entrypoint no-panic contracts, operator malformed
input contracts, path preview contracts, config precedence/deprecation
contracts, and lane-policy tests are the primary evidence.

When adding a route, verify command discovery, denied access, human output,
JSON output, exit code, reference generation, and read/write behavior.

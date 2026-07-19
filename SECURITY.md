# Security Policy

Report suspected vulnerabilities privately. Do not open a public issue with
exploit details, credentials, private data, or an unpatched reproduction.

## Supported Releases

| Surface | Support status |
| --- | --- |
| latest official tagged release and its repository-published artifacts | receives security assessment and fixes when the issue is confirmed and a fix is feasible |
| older tagged releases | not ordinarily patched; reporters may be asked to verify the latest release |
| `main`, development branches, and local source builds | reports are accepted, but these are not supported release artifacts |
| private maintainer tooling | reports are assessed when they can compromise repository, CI, release, or published-artifact integrity |
| third-party plugins, container engines, executors, and infrastructure | maintained by their owners unless the vulnerability is caused by a Bijux host boundary or integration defect |

An official artifact is produced by this repository's release workflows from a
tag. A version string in an untagged checkout does not establish release
support.

## Report Privately

Preferred channel:

- [GitHub private vulnerability reporting](https://github.com/bijux/bijux-core/security/advisories/new)

Fallback:

- [bijan@bijux.io](mailto:bijan@bijux.io)

Include enough information to reproduce and assess the issue:

- affected package, command, version, and installation method;
- operating system, architecture, and relevant runtime or backend;
- minimal reproduction steps or proof of concept;
- expected and observed security boundary;
- impact, required privileges, and whether user interaction is required;
- whether secrets or private data may have been exposed.

Remove live credentials and personal data. Use synthetic values and attach
large or sensitive evidence only through a channel agreed during private
triage.

## Repository Trust Boundaries

### CLI plugins

Installed plugins execute with the invoking user's privileges. They are not
sandboxed. Namespace checks, manifest validation, checksums, environment
shaping, and timeouts protect routing and lifecycle integrity; they do not
establish publisher identity or restrict filesystem, network, credential, or
subprocess access.

A flaw that lets plugin metadata bypass documented host validation is in scope.
Malicious behavior by plugin code that the user deliberately installed is a
third-party plugin issue unless the host promised and failed to enforce the
affected boundary.

### DAG execution

The local shell backend runs trusted commands as host processes. Policy flags
validate declared effects but do not provide a general host sandbox. Container
execution delegates isolation to the selected engine. Replay sandboxing
protects retained source evidence from writes; it does not isolate executed
code.

Path traversal, artifact-integrity bypass, policy-enforcement bypass, or a
false isolation claim caused by repository code is in scope.

### Build and publication

Release workflows, package provenance, dependency handling, generated
artifacts, and credential boundaries are in scope when a defect can compromise
an official artifact or publication authority.

## In-Scope Examples

- path traversal or unauthorized writes outside an owned storage root;
- command or plugin namespace validation bypass with security impact;
- artifact, cache, replay, signature, checksum, or provenance validation bypass;
- secret disclosure through command output, logs, reports, packages, or release
  artifacts;
- supply-chain or workflow defects that allow unauthorized publication;
- isolation or policy behavior materially weaker than the documented enforced
  boundary;
- memory-safety defects in repository-owned native code.

## Out Of Scope

- behavior of third-party code within the privileges intentionally granted to
  it;
- unsupported versions or platforms unless the issue also affects a supported
  release;
- social engineering, physical access, and attacks requiring compromised
  maintainer accounts without a repository defect;
- availability testing that creates excessive traffic, resource exhaustion, or
  service disruption;
- vulnerabilities in external services with no repository-owned integration
  defect;
- version labels or untagged release-preparation state without security impact.

## Coordinated Handling

The maintainer will assess reproducibility, affected releases, impact, and the
appropriate remediation and disclosure route. The project is maintained on a
best-effort basis and does not guarantee a response or remediation deadline.
Please allow private triage before public disclosure.

When appropriate, remediation will include affected code, regression tests,
release artifacts, and a GitHub security advisory. A report may be closed as
out of scope, not reproducible, or already addressed, with the reasoning shared
privately.

There is no public bug bounty program. Non-security defects belong in regular
GitHub issues.

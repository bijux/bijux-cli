# Cache, Replay, And Evidence

Cache and replay are explained execution decisions. They reuse verified prior
evidence only when all governing identities and policies are compatible.

## Cache Identity

A cache key includes the factors that can change a node result, including:

- graph and node identity;
- resolved params and input artifact identities;
- adapter and backend identity;
- relevant environment and execution policy;
- container image or executable identity;
- declared effects and cache behavior;
- cache metadata contract version.

Secrets are excluded from retained key material. A secret-dependent node must
use policy that prevents unsafe reuse or records a non-secret compatibility
factor.

## Hit Validation

A key match alone is insufficient. A hit requires a supported manifest
version, required proof, compatible output schema, present content, valid
hashes, and safe reuse policy. Corrupt, missing, incomplete, or incompatible
entries are misses with explicit reasons.

Cache writes occur only after accepted node success and complete required
evidence. Partial attempts do not populate a valid entry.

## Replay Eligibility

Replay requires a retained source run with compatible graph, plan, selected
branch path, node semantics, adapter/backend context, and verified artifacts.
The replay plan identifies nodes that may reuse evidence and nodes requiring
execution.

Refusal is an outcome with reasons. The runtime must not launch a fresh run and
label it replay when source evidence is insufficient.

## Evidence Orchestration

Runtime owns when facts are emitted; `bijux-dag-artifacts` owns their formats
and persistence mechanics. Runtime records:

- manifest and effective policy;
- graph/plan identity and provenance;
- node transitions and attempt evidence;
- trigger and branch decisions;
- adapter, backend, container, and resource identity;
- output indexes, hashes, and lineage;
- cache decisions and replay ancestry;
- causal failure and terminal summary.

Persistence failure can change terminal classification. A process result that
cannot produce required evidence is not a fully successful node.

## Diff And Repair

Run comparison classifies semantic, execution, evidence, and presentation
differences. Repair creates explicit new records and lineage; it never mutates
historical proof invisibly. Resume and replay retain original causal attempts.

## Verification

```bash
cargo test --locked -p bijux-dag-runtime --test runtime_cache_contracts
cargo test --locked -p bijux-dag-runtime --test policy_cache_contract
cargo test --locked -p bijux-dag-runtime --test runtime_replay_contracts
cargo test --locked -p bijux-dag-runtime --test runtime_artifact_contracts
```

Replay determinism fuzz contracts and cache evolution contracts are required
when changing key factors, proof versions, or compatibility rules.

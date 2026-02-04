# Node Trace Schema v0.1

Required keys:
- `node_id` (string)
- `status` (string: success|failed|skipped|cached)
- `started_unix_ms` (number)
- `finished_unix_ms` (number)
- `attempt` (number)
- `fingerprint` (string)
- `adapter_id` (string)
- `adapter_version` (string)

Optional:
- `resources` (object)
- `inputs_index` (string)
- `resolved_params` (json)
- `cache_proof` (object)
- `failure` (object)

## cache_proof
```
{
  "hit": bool,
  "key": "string",
  "source": "local",
  "verified": bool,
  "reason": "string",
  "corrupt_detected": bool
}
```

## failure
```
{
  "kind": "Validation|Execution|Timeout|Cancelled|CacheCorrupt|Internal",
  "code": "string",
  "message": "string",
  "details": <json>?
}
```

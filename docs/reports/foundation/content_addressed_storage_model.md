# Content Addressed Storage Model

This report compares runtime semantics between local filesystem storage and object-model storage.

## Shared semantics

- content hash is `sha256`
- artifact identity carries provenance context (run + node + path)
- integrity verification is mandatory for trusted replay paths

## Local filesystem runtime (implemented)

- payload path is materialized under run directory
- hashes are read from index and can be re-verified against payload bytes
- GC planning can walk local lineage snapshot directly

## Object-model store (modeled)

- keying may be content-addressed but runtime execution integration is not implemented
- lifecycle semantics are contract-level only
- release evidence cannot claim object-store runtime execution support


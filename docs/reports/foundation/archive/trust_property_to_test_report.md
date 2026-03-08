# Trust Property To Test Report

## Purpose

Map each battle trust property to executable test surfaces and value class, so evidence quality is governed by trust-proof coverage instead of raw test totals.

## Source

- `configs/policy/trust_property_test_map.json`
- `configs/policy/battle_trust_properties.json`

## Coverage snapshot

- mapped trust properties: 14
- critical mappings: 12
- useful mappings: 1
- decorative mappings: 0
- delete mappings: 0

## Policy notes

- Every trust property must map to one or more executable tests.
- `critical` mappings are release-trust surfaces and cannot be replaced by metadata-only checks.
- `decorative` and `delete` classes are allowed by policy for pruning workflows but should remain empty for release trust surfaces.

# Lineage policy hooks

## Purpose

Policy decisions can use semantic lineage properties in addition to runtime metadata.

## Hook input model

A policy hook can evaluate:
- relationship count
- presence of policy dependencies
- lineage quality score

## Baseline guardrail

Operations depending on policy-sensitive ancestry should be denied when lineage verification coverage is below required threshold.

## Determinism

Policy hook evaluation must remain deterministic for identical lineage inputs.

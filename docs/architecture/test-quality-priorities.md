# Test Quality Priorities

This document tracks cleanup and missing-scenario priorities from generated audit data.

## Top 20 Weakest Tests
Source: `artifacts/status/test_quality_audit.json` → `top_20_weakest_tests`.

## Top 20 Missing Failure Scenarios
Source: `artifacts/status/test_quality_audit.json` → `top_20_missing_failure_scenarios`.

## Top 20 Missing Parity Scenarios
Source: `artifacts/status/test_quality_audit.json` → `top_20_missing_parity_scenarios`.

## Top 20 Missing Packaging Scenarios
Source: `artifacts/status/test_quality_audit.json` → `top_20_missing_packaging_scenarios`.

## Flaky Labeling
Source: `artifacts/status/flaky_tests.json`.
All flaky tests must carry the `flaky` label in CI artifacts and have explicit remediation tracking.

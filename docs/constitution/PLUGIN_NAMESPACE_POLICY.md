# Plugin Namespace Policy

## Purpose
Define compatibility and governance rules for plugin-exposed namespaces.

## Scope
This document governs namespace registration and ownership boundaries for plugins.

## Core Concepts
- Plugin namespaces must coexist without shadowing built-ins.
- Namespace governance protects users from ambiguous routing.

## Invariants
- Reserved root namespaces cannot be claimed by plugins.
- Plugin namespaces are normalized to lowercase kebab-case.
- Plugin namespace registration is rejected when it conflicts with built-ins or existing plugin namespaces.
- Plugins must declare namespace intent in manifest metadata.

## Rejection Rules
- Uses a reserved namespace.
- Shadows a built-in command path.
- Collides case-insensitively with an existing namespace.
- Uses invalid characters after normalization.

## Failure Modes
- Rejected registrations return stable validation errors.

## Design Rationale
- Preventing collisions preserves deterministic routing and user trust.

## Non-Goals
- Runtime behavior of plugin command handlers.

# Reference Runtime

The reference runtime is the authoritative implementation for:

- planner execution lowering
- scheduler dispatch semantics
- node execution and state transitions
- artifact commit and manifest finalization

Adapters are extension surfaces. They must not redefine core execution meaning.

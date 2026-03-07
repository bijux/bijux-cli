# Known Non-Hermetic Behaviors

This ledger lists intentional or currently unavoidable non-hermetic behaviors.

## Current gaps
- Local subprocess execution shares host kernel and process namespace.
- Network deny policy is effect-policy enforcement, not a kernel firewall.
- Container and remote backend isolation guarantees are backend-dependent.
- Host filesystem timestamps and scheduler timing remain observable side channels.
- Ambient environment can still influence execution if `clean_env=false`.

## How to use this ledger
- Every new non-hermetic behavior must be added here with owner and mitigation.
- Every removed non-hermetic behavior should be deleted in the same change set
  that adds tests and docs proving enforcement.

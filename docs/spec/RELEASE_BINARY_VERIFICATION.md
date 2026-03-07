# Release Binary Verification

## Verification suite
- verify version output:
  - `dag version --json`
- verify capabilities output:
  - `dag capabilities --json`
- verify dry command parsing:
  - `dag --help`
- verify inspection command availability:
  - `dag runs --help`

## Integrity policy
- Release artifacts must be checksumed.
- Signature policy is pending and tracked as release governance work.

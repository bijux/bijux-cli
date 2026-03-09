# Python Plugin Delegation Authoring

For delegated plugin execution:

- declare plugin kind as `python` or `delegated`
- declare required capabilities in manifest
- provide a stable entrypoint (`module:function`)

Compatibility and trust metadata are surfaced in plugin inspect outputs.

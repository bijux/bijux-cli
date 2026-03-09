# Upgrade for Existing PyPI Users

Use in-place upgrade and verify runtime ownership.

## Upgrade

```bash
python -m pip install --upgrade bijux-cli
```

If you previously installed compatibility alias `bijux`, keep it aligned:

```bash
python -m pip install --upgrade bijux
```

## Verify

```bash
bijux version
bijux cli paths
bijux cli doctor
```

`cli doctor` should report no wheel/runtime mismatch.


# Uninstall for Pip Users

Remove both canonical and compatibility package channels.

```bash
python -m pip uninstall -y bijux-cli bijux
```

After uninstall, validate that no Python-managed wrapper remains:

```bash
bijux cli doctor
```

If `bijux` still resolves, another install channel is active.


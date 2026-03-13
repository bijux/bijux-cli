# Configuration

Use `bijux cli config ...` for runtime configuration state.

## Common Commands

```bash
bijux cli config list
bijux cli config get KEY
bijux cli config set KEY=VALUE
bijux cli config unset KEY
bijux cli config export ./bijux.env
bijux cli config load ./bijux.env
```

## References

- [Config schema](../reference/config-schema.md)
- [Config parity report](../architecture/config-parity-report.md)

## Notes

- Treat `bijux cli config list` as the quickest state check.
- Use export and load for file-based handoff.
- Use the reference pages for exact field and format rules.

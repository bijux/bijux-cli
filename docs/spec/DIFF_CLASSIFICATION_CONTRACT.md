# Diff Classification Contract

`bijux-dag` distinguishes two classes:

- `semantic`: execution meaning changes (identity/fingerprint/outcomes/payload hashes)
- `cosmetic`: representation-only changes with no execution meaning change

Operator surfaces:

- `dag diff`
- `dag why-rerun`
- `dag trace-artifact`

must present semantic-cause summaries in machine-readable form.

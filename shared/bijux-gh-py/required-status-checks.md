# Required Status Checks

- Owner: `build-and-release`
- Source of truth: workflow and job names in `.github/workflows/*.yml`

Required branch-protection checks for `main`:

- `bijux_std` (from workflow `Bijux Standard Checks`)

Notes:

- Keep required checks limited to jobs that run on every pull request to avoid merge deadlocks from path-filtered workflows.
- If workflow or job names change, update this file and `.github/rulesets/main-branch-protection.json` in the same change.

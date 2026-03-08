# Awkward Command Families Simplification Report

This report identifies command families that still create operator friction through overlapping intent, noisy defaults, or split ownership.

## Families needing simplification
1. `summary` and `status` style command groups with overlapping quick-health signals.
2. `inspect` and deep diagnostics surfaces where JSON and text output tell different stories.
3. capability and support-matrix command families that duplicate backend inventory slices.
4. verification command families with similar pass/fail semantics but different naming.
5. report-generation command groups in `bijux-dev-dag` where evidence and inventory boundaries blur.

## Simplification direction
- Keep one canonical entrypoint per operator question.
- Keep detailed output opt-in and machine-stable.
- Keep cross-command wording and exit code taxonomy aligned.

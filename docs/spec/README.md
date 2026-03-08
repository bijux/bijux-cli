# Specification documentation

Audience: implementers and maintainers.  
Owner: platform architecture and protocol maintainers.  
Status: stable.

This directory is the contract layer.  
All behavior statements in `spec/` should be normative and implementation-grade.

## Directory role

- Keep only canonical contracts and their appendices.
- Do not keep explanatory narratives, how-to guides, or historical status commentary.
- De-duplicate overlapping documents; each contract family should have one canonical root file.
- If behavior belongs in a user guide, move it to `user/`.

## Stability contract

Every file should declare whether it is stable, evolving, or historical.

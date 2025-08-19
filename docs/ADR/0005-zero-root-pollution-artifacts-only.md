# ADR-0005: Enforcing Zero-Root-Pollution via Makefile-Orchestrated Artifact Containment

* **Date:** 2025-08-20
* **Status:** Accepted
* **Author:** Bijan Mousavi

## Context

Build, test, docs, and release steps produce transient outputs (wheels/sdists, coverage reports, HTML sites, schemas). When these spill into the repo root or source trees, they clutter the working copy, risk accidental commits, and make CI/release packaging brittle.

## Decision

All generated outputs **must** be written beneath a single top-level directory: `artifacts/`. No Make/Tox/CI task may write transient files to the repo root or source trees. Standard caches (e.g., `.venv/`, `.tox/`, `.pytest_cache/`) are permitted and ignored via `.gitignore`.

This policy is enforced **centrally by the Makefile system**. The root `Makefile` and `makefiles/*` define orchestration and output paths (`ARTIFACTS ?= artifacts`). Tox and GitHub Actions **call Make targets**; they do not choose paths.

### Canonical Layout (Generated Only)

```
artifacts/
  build/            # wheels/sdists built locally
  docs/
    docs/           # generated MkDocs inputs
    site/           # MkDocs output
  test/
    htmlcov/        # coverage HTML
    coverage.xml    # coverage XML
    junit*.xml      # test reports
    hypothesis/     # Hypothesis DB
    benchmarks/     # benchmark results
    tmp/            # temp test files
  api/              # schemas, API logs/reports
  sbom/             # SBOM outputs
  citation/         # citation exports
  quality/          # interrogate, vulture, deptry, reuse, etc.
  security/         # bandit, pip-audit, etc.
  lint/             # linter/type checker reports
```

> Note: Locally we emit to `artifacts/build/`. In CI, the uploaded artifact named **`dist`** extracts as `artifacts/dist/` when downloaded—both represent the same build bundle at different stages.

Tracked sources (e.g., `pyproject.toml`, `README.md`, `LICENSE`, `CITATION.cff`) remain in place and are **not** artifacts.

## Rationale

* **Clean Working Tree:** Routine tasks don’t dirty the repo; `git status` stays meaningful.
* **Deterministic Pipelines:** CI and docs deploy hydrate exclusively from `artifacts/**`.
* **Curated Releases:** GitHub Releases contain concise, named bundles (ZIP/tar.gz per subtree), not thousands of loose files.
* **Safe Docs Builds:** MkDocs reads `artifacts/docs/docs` → writes `artifacts/docs/site`; required pages are asserted.
* **Reproducibility:** Uniform paths across local and CI; caches remain standard and ignored.

## Enforcement

### Local (Make + Tox)

* `ARTIFACTS ?= artifacts` in the root `Makefile`; sub-recipes in `makefiles/*` route outputs under that root (e.g., `makefiles/test.mk` → `artifacts/test/`; integrates with ADR-0004 toolchain targets like `make lint` for logs/caches under `artifacts/lint/`).
* Tox environments call Make targets; they do **not** set output paths directly.
* For docs, `make docs-prep` copies tracked sources (e.g., `docs/ADR/`) into `artifacts/docs/docs/` before build (aligning with ADR-0003's storage rules).

### CI (GitHub Actions)

* **`ci.yml`** (CI workflow) uploads only from `artifacts/**`.
* **`deploy-docs.yml`** (Deploy Docs workflow) hydrates into `./artifacts/**`, builds from `artifacts/docs/docs` to `artifacts/docs/site`, and checks required pages.
* **`publish.yml`** (Publish to PyPI workflow) assembles release bundles from `artifacts/**`, computes checksums for the build bundle, and attaches curated ZIPs (tests per-py, lint, quality, security, api, docs, sbom, citation, build).

## Consequences

### Positive

* Consistent paths locally and in CI.
* Simpler evidence collection and release packaging.
* Lower risk of committing transients.
* Docs completeness enforced before deploy.

### Trade-offs

* Some tools require output redirection or a post-step move (handled in Make).
* Initial refactors to Make/Tox; validated by CI.
* PRs introducing new tools must adhere to the layout (review + CI enforce it).

## Invariants

* Make targets do **not** write outside `$(ARTIFACTS)` (except standard caches). 
* CI uploads/downloads **only** `artifacts/**`. 
* Docs build from `artifacts/docs/docs` → `artifacts/docs/site`. 
* Releases assemble from `artifacts/**` (build bundle appears as `artifacts/build/` locally and `artifacts/dist/` when retrieved from CI).

## Compliance Examples

* **Build Wheels/SDist**

  ```bash
  python -m build --outdir artifacts/build
  ```

* **Tests + Coverage + JUnit**

  ```bash
  pytest --cov \
         --cov-report=xml:artifacts/test/coverage.xml \
         --cov-report=html:artifacts/test/htmlcov \
         --junitxml=artifacts/test/junit.xml
  ```

* **MkDocs**

  ```yaml
  docs_dir: artifacts/docs/docs
  site_dir: artifacts/docs/site
  ```

## Alternatives Considered

* Tool defaults scattered across the repo — rejected (clutter, fragility). 
* Per-tool output roots — rejected (fragmentation). 
* CI-only containment — rejected (misses local benefits).

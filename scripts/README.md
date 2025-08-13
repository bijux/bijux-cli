# `scripts/`

A small collection of **purpose-built developer utilities** for code hygiene, documentation automation, and release workflow integration.

## Contents

### 1. **Comment Harvester** (`helper_comments.py`)

Scans a codebase for **all lines containing `#`** (including `# noqa`, `# type: ignore`, and TODO/FIXME notes).
Useful for:

* Building a **baseline** for linter/typing ignore rules.
* Periodic triage of technical debt and ignored checks.

**Example:**

```bash
python scripts/helper_comments.py src -o comments.txt -e .py .pyi
```

**Sample output:**

```
src/example.py
12: def foo():  # noqa: E999
47: # TODO: refactor
```

Notes:

* Any `#` is treated as a comment (including shebangs and inline hashes in strings).
* Skips unreadable files.
* Respects `--ext` filter.

---

### 2. **MkDocs API & Navigation Generator** (`helper_mkdocs.py`)

Generates **API reference pages** for all Python modules in `src/bijux_cli` and an auto-synced `nav.md` for [literate-nav](https://github.com/oprypin/mkdocs-literate-nav).

Integrated into `mkdocs.yml`:

```yaml
plugins:
  - gen-files:
      scripts:
        - scripts/helper_mkdocs.py
  - literate-nav
  - mkdocstrings
```

Run locally:

```bash
mkdocs serve    # preview
mkdocs build    # build static site
```

---

### 3. **Commit Hook – Changelog Fragment Generator** (`git-hooks/prepare-commit-msg`)

A **Git `prepare-commit-msg` hook** that automatically creates a [Towncrier](https://towncrier.readthedocs.io/) changelog fragment when committing with a Conventional Commit message.

Example:

```bash
feat(api): add streaming support
```

Automatically produces:

```
changelog.d/1691940245.feature.md
```

---

## Workflow Philosophy

* **Automation first**: anything repetitive belongs in a script or hook.
* **Debt visibility**: helper comments are tracked, triaged, and expire.
* **Docs in sync**: API navigation is regenerated automatically at build time.
* **Release-ready commits**: hooks enforce fragment creation for changelog accuracy.

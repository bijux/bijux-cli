# Scripts
<a id="top"></a>

A small collection of **purpose-built developer utilities** for code hygiene, documentation automation, and release workflow integration.

---

## Table of Contents

- [Directory](#directory)
- [Comment Harvester (`helper_comments.py`)](#comment-harvester)
- [MkDocs API & Navigation Generator (`helper_mkdocs.py`)](#mkdocs-api--navigation-generator)
- [Commit Hook – Changelog Fragment Generator (`git-hooks/prepare-commit-msg`)](#commit-hook--changelog-fragment-generator)
- [Towncrier Guard (`check-towncrier-fragment.sh`)](#towncrier-guard)
- [Workflow Philosophy](#workflow-philosophy)

[Back to top](#top)

---

<a id="directory"></a>

## Directory

```

scripts/
├── README.md
├── check-towncrier-fragment.sh
├── git-hooks/
│   └── prepare-commit-msg
├── helper\_comments.py
└── helper\_mkdocs.py

````

[Back to top](#top)

---

<a id="comment-harvester"></a>

## Comment Harvester (`helper_comments.py`)

Scans a codebase for **all lines containing `#`** (including `# noqa`, `# type: ignore`, and TODO/FIXME notes).

**Why?**  
Build a **baseline** for linter/typing ignore rules and periodically triage technical debt.

**Usage**

```bash
python scripts/helper_comments.py src -o comments.txt -e .py .pyi
````

**Sample output**

```
src/example.py
12: def foo():  # noqa: E999
47: # TODO: refactor
```

**Notes**

* Any `#` is treated as a comment (includes shebangs and inline `#` inside strings).
* Skips unreadable files.
* Respects `--ext` filter.
* Prints a summary path: `Violation lines written to <output>`.

[Back to top](#top)

---

<a id="mkdocs-api--navigation-generator"></a>

## MkDocs API & Navigation Generator (`helper_mkdocs.py`)

Runs via **mkdocs-gen-files** during MkDocs builds to keep docs in sync.

**What it does**

1. Copies root docs into the site with safe link rewrites:

   * `README.md` → `docs/index.md`
   * `USAGE.md` → `docs/usage.md`
   * `TESTS.md` → `docs/tests.md`
   * `PROJECT_TREE.md` → `docs/project_tree.md`
2. Ensures a hidden top anchor on copied pages.
3. Generates **API reference** pages for `src/bijux_cli/**.py` (mkdocstrings).
4. Creates per-package `reference/**/index.md`.
5. Builds `nav.md` with **Home / User Guide / Project Tree / Tests / API / Changelog / ADRs / Community**.

**MkDocs config snippet**

```yaml
plugins:
  - gen-files:
      scripts:
        - scripts/helper_mkdocs.py
  - literate-nav
  - mkdocstrings
```

**Local preview**

```bash
mkdocs serve    # preview
mkdocs build    # static site
```

[Back to top](#top)

---

<a id="commit-hook--changelog-fragment-generator"></a>

## Commit Hook – Changelog Fragment Generator (`git-hooks/prepare-commit-msg`)

A **Git `prepare-commit-msg` hook** that auto-creates a Towncrier fragment from a **Conventional Commit** subject.

**Example**

```
feat(api): add streaming support
```

Produces (timestamped):

```
changelog.d/1691940245.feature.md
```

**Type mapping**

* `feat` → `feature`
* `fix` → `bugfix`
* `refactor`/`docs`/`style`/`chore` → `misc`
* others → `misc`

**Install**

```bash
# from repo root
chmod +x scripts/git-hooks/prepare-commit-msg
ln -sf ../../scripts/git-hooks/prepare-commit-msg .git/hooks/prepare-commit-msg
make bootstrap   # installs hooks idempotently
```

[Back to top](#top)

---

<a id="towncrier-guard"></a>

## Towncrier Guard (`check-towncrier-fragment.sh`)

Blocks commits of types that **should** have a Towncrier fragment if none is staged.

**Behavior**

* Skips `Merge`, `Revert`, `fixup!`, `squash!`, and `chore: release` subjects.
* Looks for staged `changelog.d/*.md`.
* Enforces for common types: `feat`, `fix`, `refactor`, `perf`, `docs`.
* Allow override with `TOWNCRIER_ALLOW_SKIP=1`.

**Wire it up (pre-commit, commit-msg stage)**

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: towncrier-guard
        name: Towncrier fragment required
        entry: scripts/check-towncrier-fragment.sh
        language: system
        stages: [commit-msg]
```

Alternatively, call it in CI before merge.

[Back to top](#top)

---

<a id="workflow-philosophy"></a>

## Workflow Philosophy

* **Automation first** — anything repetitive belongs in a script or hook.
* **Debt visibility** — helper comments are tracked, triaged, and expire.
* **Docs in sync** — API/navigation are regenerated automatically at build time.
* **Release-ready commits** — hooks ensure changelog accuracy.

[Back to top](#top)

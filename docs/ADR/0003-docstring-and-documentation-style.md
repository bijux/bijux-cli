# ADR 0003: Docstring and Documentation Style

- **Date:** 2025‑08‑01
- **Status:** Accepted
- **Author:** Bijan Mousavi

---

## Context
Our codebase demands a rigorously consistent and machine-parsable documentation style to facilitate the seamless, automated generation of comprehensive and visually appealing webpages through MkDocs. This process depends entirely on full, detailed docstrings placed at the top of every Python file, which must thoroughly explain the complete contents of that file. User-facing documentation—encompassing guides, ADRs, and READMEs—must remain highly readable in plain text while rendering flawlessly on the web. By utilizing tools like MkDocs for rendering and linters for validation, we enable all contributors to produce and sustain uniformly high-quality documentation, with absolutely no exceptions permitted.

## Decision
### Docstrings
We enforce the exclusive use of the **Google Python Style Guide** for all in-code docstrings throughout the codebase, prohibiting any deviations or mixed styles to guarantee absolute unification and automated enforcement.

* Every Python file must commence with a comprehensive module-level docstring enclosed in triple quotes `"""…"""`, beginning with a concise one-sentence summary on the initial line.
* After the summary, insert a blank line followed by an exhaustive description of the file's entire code, covering its overall purpose, architectural structure, primary components, interdependencies, and pertinent usage guidelines. This docstring must be fully self-explanatory, eschewing abbreviated or partial content.
* Include the following sections in precise order, omitting only those that are wholly inapplicable:

     1. **Args:**
        ```
         Args:
             name (str): Description of the argument.
             count (int): Description of the argument.
        ```
     2. **Returns:**
        ```
        Returns:
            bool: Description of the return value.
        ```
     3. **Raises:**
        ```
        Raises:
            ValueError: Description of when and why the exception is raised.
        ```

* In docstrings, employ `*` exclusively for any bullet points or lists (such as in descriptions or output contracts) to ensure proper line-separated display in MkDocs-generated webpages. Example:
  ```
  Output Contract:
      * Success: {"version": str, "timestamp": float}
      * Verbose: {"python": str, "platform": str}
      * Error: {"error": str, "code": int}
  ```
* Avoid `-` or alternative markers that could lead to rendering inconsistencies.
* Extend full Google-style docstrings to every class, function, and method, with each providing a thorough account of its functionality and integration within the file.
* Prohibit the use of `<module>`, `:param:`, reST directives, or any non-Google elements—strict adherence to Google format is required.
* Mandate enforcement through tools like `pydocstyle` set to the "google" convention, embedded in CI/CD workflows to automatically block non-conforming submissions.

### Repository Documentation
* Store all ADRs in `docs/ADR/` with filenames formatted as `XXXX-<short-title>.md`, where `XXXX` represents a zero-padded integer.
* Restrict all repository documentation (ADRs, READMEs, guides) to Markdown format only.
* Prefer `*` for Markdown bullet points. `-` is also acceptable to match GitHub/MkDocs defaults.
* Mandate inclusion of Date, Status, and Author headers in ADRs, as exemplified in this document.
* Set up MkDocs to draw from `docs/` as the primary source directory for automated website publication.

## Consequences
### Pros
* Delivers a uniformly professional codebase with reliable introspection and automated API reference generation.
* Ensures out-of-the-box compatibility with tools like MkDocs for superior documentation websites.
* Provides contributors with unambiguous, enforceable guidelines, streamlining reviews through automation and objectivity.

### Cons
* Incurs upfront costs for refactoring existing non-compliant files.
* Necessitates adaptation for contributors unaccustomed to Google style or rigorous linting protocols.

## Enforcement
* Prohibit acceptance of any code or documentation pull request that fails to comply fully with this ADR.
* Empower reviewers and CI systems to reject submissions outright for issues such as absent module-level docstrings, incomplete explanations, stylistic inconsistencies, or improper bullet formatting.
* Establish this policy as irrevocably binding, with no provisions for negotiation or exceptions.

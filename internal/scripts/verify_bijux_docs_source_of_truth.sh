#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"

if [[ -d "${repo_root}/overrides" ]]; then
  echo "ERROR: root overrides/ must not exist; use shared/bijux-docs as docs source of truth" >&2
  exit 1
fi

compare_required() {
  local source_rel="$1"
  local generated_rel="$2"

  local source_path="${repo_root}/${source_rel}"
  local generated_path="${repo_root}/${generated_rel}"

  if [[ ! -f "${source_path}" ]]; then
    echo "ERROR: missing source file ${source_rel}" >&2
    exit 1
  fi

  if [[ ! -f "${generated_path}" ]]; then
    echo "ERROR: missing generated file ${generated_rel}" >&2
    echo "Run: make bijux-docs-sync" >&2
    exit 1
  fi

  if ! cmp -s "${source_path}" "${generated_path}"; then
    echo "ERROR: ${generated_rel} drifted from ${source_rel}" >&2
    echo "Run: make bijux-docs-sync" >&2
    exit 1
  fi
}

assert_absent() {
  local rel_path="$1"
  if [[ -e "${repo_root}/${rel_path}" ]]; then
    echo "ERROR: generated root path must not exist: ${rel_path}" >&2
    echo "Source-only root policy keeps generated site output out of main branch." >&2
    exit 1
  fi
}

# shared -> docs (authoritative docs source)
compare_required "shared/bijux-docs/partials/header.html" "docs/overrides/partials/header.html"
compare_required "shared/bijux-docs/partials/footer.html" "docs/overrides/partials/footer.html"
compare_required "shared/bijux-docs/partials/footer-profile-links.html" "docs/overrides/partials/footer-profile-links.html"
compare_required "shared/bijux-docs/partials/nav.html" "docs/overrides/partials/nav.html"
compare_required "shared/bijux-docs/partials/nav-item.html" "docs/overrides/partials/nav-item.html"
compare_required "shared/bijux-docs/partials/bijux-nav.html" "docs/overrides/partials/bijux-nav.html"

for style in 00-tokens.css 01-theme.css 02-layout.css 03-header.css 04-nav.css 05-content.css 06-components.css 07-utilities.css 08-responsive.css extra.css README.md; do
  compare_required "shared/bijux-docs/styles/${style}" "docs/assets/styles/${style}"
done

for script in bootstrap.js detail-tabs.js nav-reveal.js nav-state.js theme-persistence.js viewport-profile.js README.md; do
  compare_required "shared/bijux-docs/scripts/${script}" "docs/assets/javascripts/shell/${script}"
done
compare_required "shared/bijux-docs/scripts/nav-sync.js" "docs/assets/javascripts/navigation-sync.js"

# root must stay source-only
assert_absent "assets"
assert_absent "404.html"
assert_absent "index.html"
assert_absent "learning"
assert_absent "platform"
assert_absent "projects"
assert_absent "reading-paths"
assert_absent "search"
assert_absent "sitemap.xml"
assert_absent "sitemap.xml.gz"

echo "Bijux docs source-of-truth checks passed"

#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
shared_root="${repo_root}/shared/bijux-docs"

sync_file() {
  local src="$1"
  local dst="$2"
  mkdir -p "$(dirname "${dst}")"
  cp "${src}" "${dst}"
}

for required in \
  "${shared_root}/partials/header.html" \
  "${shared_root}/partials/footer.html" \
  "${shared_root}/partials/footer-profile-links.html" \
  "${shared_root}/partials/nav.html" \
  "${shared_root}/partials/nav-item.html" \
  "${shared_root}/partials/bijux-nav.html" \
  "${shared_root}/styles/extra.css" \
  "${shared_root}/scripts/mermaid-init.js" \
  "${shared_root}/scripts/nav-sync.js"; do
  if [[ ! -f "${required}" ]]; then
    echo "ERROR: missing shared bijux docs file ${required}" >&2
    exit 1
  fi
done

# shared -> docs
sync_file "${shared_root}/partials/header.html" "${repo_root}/docs/overrides/partials/header.html"
sync_file "${shared_root}/partials/footer.html" "${repo_root}/docs/overrides/partials/footer.html"
sync_file "${shared_root}/partials/footer-profile-links.html" "${repo_root}/docs/overrides/partials/footer-profile-links.html"
sync_file "${shared_root}/partials/nav.html" "${repo_root}/docs/overrides/partials/nav.html"
sync_file "${shared_root}/partials/nav-item.html" "${repo_root}/docs/overrides/partials/nav-item.html"
sync_file "${shared_root}/partials/bijux-nav.html" "${repo_root}/docs/overrides/partials/bijux-nav.html"

for style in 00-tokens.css 01-theme.css 02-layout.css 03-header.css 04-nav.css 05-content.css 06-components.css 07-utilities.css 08-responsive.css extra.css README.md; do
  sync_file "${shared_root}/styles/${style}" "${repo_root}/docs/assets/styles/${style}"
done

for script in bootstrap.js detail-tabs.js nav-reveal.js nav-state.js theme-persistence.js viewport-profile.js README.md; do
  sync_file "${shared_root}/scripts/${script}" "${repo_root}/docs/assets/javascripts/shell/${script}"
done
sync_file "${shared_root}/scripts/nav-sync.js" "${repo_root}/docs/assets/javascripts/navigation-sync.js"
sync_file "${shared_root}/scripts/mermaid-init.js" "${repo_root}/docs/assets/javascripts/mermaid-init.js"

echo "Bijux docs synchronized: shared -> docs"

#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
config_path="${BIJUX_STD_CONFIG:-${repo_root}/shared/bijux-checks/bijux-std-checks.yml}"

if [[ ! -f "${config_path}" ]]; then
  echo "ERROR: missing config ${config_path}" >&2
  exit 1
fi

read_scalar() {
  local key="$1"
  awk -F': ' -v key="${key}" '$1 == key {print $2; exit}' "${config_path}" | tr -d '"'
}

read_directories() {
  awk '/^directories:/{flag=1;next} /^remote:/{flag=0} flag && /^  - /{sub(/^  - /, ""); print}' "${config_path}"
}

manifest_rel="$(read_scalar manifest)"
git_url_default="$(read_scalar '  git_url')"
default_ref="$(read_scalar '  default_ref')"
tag_pattern_default="$(read_scalar '  tag_pattern')"

std_git_url="${BIJUX_STD_GIT_URL:-${git_url_default}}"
update_channel="${BIJUX_STD_UPDATE_CHANNEL:-branch}"
std_ref="${BIJUX_STD_REF:-${default_ref}}"
tag_pattern="${BIJUX_STD_TAG_PATTERN:-${tag_pattern_default}}"

resolve_ref() {
  if [[ "${update_channel}" == "tag" ]]; then
    local latest_tag
    latest_tag="$(git ls-remote --tags --refs "${std_git_url}" "${tag_pattern}" | awk '{print $2}' | sed 's#refs/tags/##' | sort -V | tail -n 1)"
    if [[ -z "${latest_tag}" ]]; then
      echo "ERROR: no tags found matching pattern '${tag_pattern}' in ${std_git_url}" >&2
      exit 1
    fi
    echo "${latest_tag}"
    return
  fi

  echo "${std_ref}"
}

resolved_ref="$(resolve_ref)"
tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT

clone_from_ref() {
  local ref_name="$1"
  git clone --depth 1 --branch "${ref_name}" "${std_git_url}" "${tmp_dir}/bijux-std" >/dev/null 2>&1
}

if ! clone_from_ref "${resolved_ref}"; then
  head_ref="$(git ls-remote --symref "${std_git_url}" HEAD 2>/dev/null | awk '/^ref:/ {print $2}' | sed 's#refs/heads/##')"
  if [[ -n "${head_ref}" && "${head_ref}" != "${resolved_ref}" ]]; then
    rm -rf "${tmp_dir}/bijux-std"
    if clone_from_ref "${head_ref}"; then
      echo "→ requested ref ${resolved_ref} unavailable; using remote HEAD branch ${head_ref}"
      resolved_ref="${head_ref}"
    else
      echo "ERROR: unable to clone ${std_git_url} using ref ${resolved_ref} or HEAD ${head_ref}" >&2
      exit 1
    fi
  else
    echo "ERROR: unable to clone ${std_git_url} using ref ${resolved_ref}" >&2
    exit 1
  fi
fi

while IFS= read -r dir_rel; do
  src="${tmp_dir}/bijux-std/${dir_rel}"
  dst="${repo_root}/${dir_rel}"
  if [[ ! -d "${src}" ]]; then
    echo "ERROR: missing source directory in bijux-std: ${dir_rel}" >&2
    exit 1
  fi
  rm -rf "${dst}"
  mkdir -p "$(dirname "${dst}")"
  cp -R "${src}" "${dst}"
  echo "→ updated ${dir_rel}"
done < <(read_directories)

cp "${tmp_dir}/bijux-std/${manifest_rel}" "${repo_root}/${manifest_rel}"
echo "→ updated ${manifest_rel}"

echo "✔ bijux-std shared directories updated from ${std_git_url}@${resolved_ref}"

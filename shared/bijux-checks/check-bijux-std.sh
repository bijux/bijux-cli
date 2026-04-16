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
repo_url_default="$(read_scalar '  repo_url')"
raw_base_default="$(read_scalar '  raw_base')"
git_url_default="$(read_scalar '  git_url')"
default_ref="$(read_scalar '  default_ref')"

std_ref="${BIJUX_STD_REF:-${default_ref}}"
std_remote="${BIJUX_STD_REMOTE:-${repo_url_default}}"
std_remote="${std_remote%/}"
std_root="${BIJUX_STD_ROOT:-${repo_root}/../bijux-std}"
strict_remote="${BIJUX_STD_STRICT_REMOTE:-0}"
manifest_path="${repo_root}/${manifest_rel}"

resolve_raw_base() {
  local remote="$1"
  local fallback_raw="$2"
  local owner_repo

  if [[ "${remote}" == https://raw.githubusercontent.com/* ]]; then
    echo "${remote%/}"
    return
  fi

  owner_repo="$(echo "${remote}" | sed -E 's#^https?://github\.com/##; s#\.git$##')"
  if [[ "${owner_repo}" == */* ]]; then
    echo "https://raw.githubusercontent.com/${owner_repo}"
    return
  fi

  if [[ -n "${fallback_raw}" ]]; then
    echo "${fallback_raw%/}"
    return
  fi

  echo "${remote%/}"
}

std_raw_base="$(resolve_raw_base "${std_remote}" "${raw_base_default}")"
std_manifest_url="${std_raw_base}/${std_ref}/${manifest_rel}"

if [[ ! -f "${manifest_path}" ]]; then
  echo "ERROR: missing local manifest ${manifest_path}" >&2
  echo "Hint: run make bijux-std-update" >&2
  exit 1
fi

fetch_url_to_file() {
  local url="$1"
  local out="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$out"
    return
  fi
  if command -v wget >/dev/null 2>&1; then
    wget -qO "$out" "$url"
    return
  fi
  echo "ERROR: neither curl nor wget is available" >&2
  exit 2
}

directory_tree_sha256() {
  local target_dir="$1"
  if [[ ! -d "${target_dir}" ]]; then
    echo "ERROR: missing directory ${target_dir}" >&2
    exit 1
  fi
  (
    cd "${target_dir}"
    find . -type f -print | LC_ALL=C sort | while IFS= read -r file_rel; do
      shasum -a 256 "${file_rel}"
    done
  ) | shasum -a 256 | awk '{print $1}'
}

manifest_sha_for_dir() {
  local manifest_file="$1"
  local dir_rel="$2"
  awk -v dir_rel="${dir_rel}" '$2 == dir_rel { print $1 }' "${manifest_file}"
}

verify_dir_against_manifests() {
  local dir_rel="$1"
  local remote_manifest="$2"

  local local_expected
  local remote_expected
  local actual_sha

  local_expected="$(manifest_sha_for_dir "${manifest_path}" "${dir_rel}")"
  remote_expected="$(manifest_sha_for_dir "${remote_manifest}" "${dir_rel}")"

  if [[ -z "${local_expected}" ]]; then
    echo "ERROR: local manifest missing entry for ${dir_rel}" >&2
    exit 1
  fi
  if [[ -z "${remote_expected}" ]]; then
    echo "ERROR: remote manifest missing entry for ${dir_rel}" >&2
    exit 1
  fi

  actual_sha="$(directory_tree_sha256 "${repo_root}/${dir_rel}")"

  if [[ "${local_expected}" != "${remote_expected}" ]]; then
    echo "ERROR: local manifest drift for ${dir_rel}" >&2
    echo "Local manifest:  ${local_expected}" >&2
    echo "Remote manifest: ${remote_expected}" >&2
    exit 1
  fi

  if [[ "${actual_sha}" != "${remote_expected}" ]]; then
    echo "ERROR: shared directory drift for ${dir_rel}" >&2
    echo "Expected: ${remote_expected}" >&2
    echo "Actual:   ${actual_sha}" >&2
    exit 1
  fi

  echo "✔ ${dir_rel} matches bijux-std (${remote_expected})"
}

tmp_manifest="$(mktemp)"
cleanup() {
  rm -f "${tmp_manifest}"
}
trap cleanup EXIT

if ! fetch_url_to_file "${std_manifest_url}" "${tmp_manifest}"; then
  local_std_manifest="${std_root}/${manifest_rel}"
  if [[ "${strict_remote}" == "1" ]]; then
    echo "ERROR: failed to fetch ${std_manifest_url} (strict remote mode)" >&2
    exit 1
  fi
  if [[ -f "${local_std_manifest}" ]]; then
    cp "${local_std_manifest}" "${tmp_manifest}"
    echo "→ remote manifest unavailable; using local bijux-std manifest at ${local_std_manifest}"
  else
    echo "ERROR: failed to fetch ${std_manifest_url}" >&2
    echo "ERROR: local fallback manifest not found at ${local_std_manifest}" >&2
    exit 1
  fi
fi

while IFS= read -r dir_rel; do
  verify_dir_against_manifests "${dir_rel}" "${tmp_manifest}"
done < <(read_directories)

echo "✔ bijux-std check passed (ref=${std_ref}, manifest=${manifest_rel}, remote=${git_url_default})"

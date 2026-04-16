#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

std_ref="${BIJUX_STD_REF:-main}"
std_remote="${BIJUX_STD_REMOTE:-https://raw.githubusercontent.com/bijux/bijux-std}"
std_remote="${std_remote%/}"
std_manifest_url="${std_remote}/${std_ref}/shared/shared-dir-sha256.txt"
std_root="${BIJUX_STD_ROOT:-${repo_root}/../bijux-std}"
strict_remote="${BIJUX_STD_STRICT_REMOTE:-0}"

local_manifest="${repo_root}/shared/shared-dir-sha256.txt"
if [[ ! -f "${local_manifest}" ]]; then
  echo "ERROR: missing local manifest ${local_manifest}" >&2
  echo "Hint: sync from bijux-std and commit shared/shared-dir-sha256.txt" >&2
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
  local manifest_path="$1"
  local dir_rel="$2"
  awk -v dir_rel="${dir_rel}" '$2 == dir_rel { print $1 }' "${manifest_path}"
}

verify_dir_against_manifests() {
  local dir_rel="$1"
  local remote_manifest="$2"

  local local_expected
  local remote_expected
  local actual_sha

  local_expected="$(manifest_sha_for_dir "${local_manifest}" "${dir_rel}")"
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
  local_std_manifest="${std_root}/shared/shared-dir-sha256.txt"
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

verify_dir_against_manifests "shared/bijux-docs" "${tmp_manifest}"
verify_dir_against_manifests "shared/bijux-makes-py" "${tmp_manifest}"

echo "✔ bijux-std check passed (ref=${std_ref})"

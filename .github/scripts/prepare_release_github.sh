#!/usr/bin/env bash
set -euo pipefail

: "${RELEASE_TAG:?RELEASE_TAG is required}"
: "${RELEASE_VERSION:?RELEASE_VERSION is required}"

release_tree="${GITHUB_WORKSPACE}/artifacts/release-tree"
release_assets="${GITHUB_WORKSPACE}/artifacts/github-release"

rm -rf "${release_tree}" "${release_assets}"
python3 .github/scripts/prepare_release_tree.py \
  --workspace-root . \
  --output-dir "${release_tree}" \
  --version "${RELEASE_VERSION}" >/dev/null
mkdir -p "${release_assets}"

python3 -m pip install --upgrade pip
python3 -m pip install "maturin==1.12.6"

maturin build \
  --release \
  --locked \
  --manifest-path "${release_tree}/crates/bijux-cli-python/Cargo.toml" \
  --interpreter python3.11 \
  --compatibility pypi \
  --out "${release_assets}"

maturin sdist \
  --manifest-path "${release_tree}/crates/bijux-cli-python/Cargo.toml" \
  --out "${release_assets}"

(
  cd "${release_assets}"
  shasum -a 256 ./* > sha256sums.txt
)

cat > "${release_assets}/release-notes.md" <<NOTES
Repository releases mirror the stamped tag artifacts for this version.

Release version: ${RELEASE_VERSION}
Tag: ${RELEASE_TAG}

Published package surfaces:
- crates.io: https://crates.io/crates/bijux-cli/${RELEASE_VERSION}
- PyPI: https://pypi.org/project/bijux-cli/${RELEASE_VERSION}/
- GHCR: https://github.com/${GITHUB_REPOSITORY}/pkgs/container/${GITHUB_REPOSITORY#*/}%2Fbijux-cli

Attached assets:
- manylinux wheel for the Python package
- source distribution for the Python package
- SHA-256 checksums for the attached files
NOTES

oras_version="1.2.3"
oras_tmp="$(mktemp -d)"
trap 'rm -rf "${oras_tmp}"' EXIT
curl -sSfL "https://github.com/oras-project/oras/releases/download/v${oras_version}/oras_${oras_version}_linux_amd64.tar.gz" -o "${oras_tmp}/oras.tgz"
tar -xzf "${oras_tmp}/oras.tgz" -C "${oras_tmp}" oras
mkdir -p "${HOME}/.local/bin"
cp "${oras_tmp}/oras" "${HOME}/.local/bin/oras"
chmod +x "${HOME}/.local/bin/oras"
export PATH="${HOME}/.local/bin:${PATH}"

echo "${GITHUB_TOKEN}" | docker login ghcr.io -u "${GITHUB_ACTOR}" --password-stdin

archive_dir="${GITHUB_WORKSPACE}/artifacts/ghcr"
archive_name="bijux-cli-${RELEASE_TAG}.tar.gz"
archive_path="${archive_dir}/${archive_name}"
package_ref="ghcr.io/${GITHUB_REPOSITORY}/bijux-cli"

mkdir -p "${archive_dir}"
tar -C "${release_assets}" -czf "${archive_path}" .

oras push "${package_ref}:${RELEASE_TAG}" \
  --artifact-type application/vnd.bijux.release-bundle.v1+tar \
  --annotation "org.opencontainers.image.title=bijux-cli" \
  --annotation "org.opencontainers.image.version=${RELEASE_VERSION}" \
  --annotation "org.opencontainers.image.source=https://github.com/${GITHUB_REPOSITORY}" \
  --annotation "org.opencontainers.image.revision=${GITHUB_SHA}" \
  "${archive_path}:application/gzip"
oras tag "${package_ref}:${RELEASE_TAG}" latest

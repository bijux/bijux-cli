#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <tag> <output_dir>" >&2
  exit 1
fi

tag="$1"
out_dir="$2"
mkdir -p "$out_dir"

outfile="${out_dir}/provenance-${tag}.json"
cat > "$outfile" <<EOF
{
  "tag": "${tag}",
  "generated_at_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "generator": "scripts/generate-provenance-statement.sh",
  "note": "Provenance hook scaffold. Replace with signed attestation workflow when enabled."
}
EOF

echo "wrote ${outfile}"

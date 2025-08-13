#!/usr/bin/env bash
set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Error: commit-msg hook did not receive filename argument." >&2
  exit 1
fi

msg_file="$1"
subject=$(head -n1 "$msg_file")

if [ -t 2 ]; then
  RED=$(tput setaf 1); YELLOW=$(tput setaf 3); BOLD=$(tput bold); RESET=$(tput sgr0)
else
  RED=""; YELLOW=""; BOLD=""; RESET=""
fi

if [[ "$subject" =~ ^(Merge|Revert|fixup!|squash!)\  ]]; then exit 0; fi
if [[ "$subject" =~ ^chore(\(.*\))?:\ release ]]; then exit 0; fi

staged_files=$(git diff-index --cached --name-only HEAD)

if [ -z "$staged_files" ]; then exit 0; fi
if [ "$staged_files" == "CHANGELOG.md" ]; then exit 0; fi
if [ "${TOWNCRIER_ALLOW_SKIP:-0}" = "1" ]; then exit 0; fi

type=$(echo "$subject" | sed -nE 's/^(feat|fix|refactor|perf|docs)(\(.*\))?!?:.*/\1/p')
if [ -z "$type" ]; then
  exit 0
fi

if echo "$staged_files" | grep -q 'changelog.d/.*\.md$'; then
  exit 0
fi

echo "${RED}✘ Commit type '${BOLD}${type}${RESET}${RED}' requires a Towncrier fragment under changelog.d/${RESET}" >&2
echo "  ${YELLOW}Please add a fragment, for example: 'changelog.d/123.${type}.md'${RESET}" >&2
echo "  (To override this check in rare cases, run: ${BOLD}TOWNCRIER_ALLOW_SKIP=1 git commit ...${RESET})" >&2
exit 1

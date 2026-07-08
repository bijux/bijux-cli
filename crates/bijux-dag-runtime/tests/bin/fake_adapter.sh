#!/bin/sh
if [ "$1" = "info" ]; then
  echo '{"protocol_version":"bijux-dag-adapter/v1","adapter_id":"fake","adapter_version":"0.1","required_effects":{"filesystem":true,"env":false,"network":false,"clock":false},"supported_kinds":["fake"],"output_schema":"v0.1"}'
  exit 0
fi
if [ "$1" = "execute" ]; then
  outdir=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --outdir)
        outdir="$2"; shift 2;;
      --workdir)
        shift 2;;
      --node-spec)
        shift 2;;
      --failure-path)
        shift 2;;
      *)
        shift;;
    esac
  done
  mkdir -p "$outdir"
  echo "ok" > "$outdir/out"
  exit 0
fi
exit 1

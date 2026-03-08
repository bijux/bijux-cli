# App Response Schema Coverage By Command Family

| command_family | response_surface | coverage_signal_count |
| --- | --- | --- |
| validate | JSON envelope + diagnostics array | 9 |
| plan | JSON envelope + plan payload variants | 8 |
| run | JSON envelope + run dir payload | 11 |
| inspect | JSON envelope + run/node/status payloads | 12 |
| history | JSON envelope + history query payloads | 8 |
| replay | JSON envelope + replay proof payloads | 10 |
| diff | JSON envelope + semantic diff payloads | 9 |
| prove_verify | JSON envelope + verification status payloads | 9 |
| export_import | JSON envelope + bundle summary payloads | 10 |
| artifact | JSON envelope + artifact lineage payloads | 9 |
| capability_query | JSON envelope + capability matrix payloads | 7 |
| diagnostics | JSON envelope + why/trace payloads | 8 |

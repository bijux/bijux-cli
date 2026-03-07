# Execution Support Policy

## Policy

Support claims are valid only when conformance is implemented and test-backed in this repository.

## Execution mode status

| Mode | Status | Contract meaning |
| --- | --- | --- |
| local process | implemented | production-claimable within repository scope |
| container | simulated | modeled contract and fixtures only |
| kubernetes | simulated | no production backend implementation in this repo |
| remote distributed | simulated | modeled boundary, not production execution mode |
| batch/HPC | simulated | modeled boundary, not production execution mode |

## Wording rules

- Docs must not imply production support outside implemented status.
- Experimental or simulated surfaces must be labeled explicitly.
- Release notes must link to evidence reports for claimed support.

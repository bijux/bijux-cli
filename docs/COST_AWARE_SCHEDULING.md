# Cost-aware scheduling and economic optimization

## Cost model scope

Cost modeling covers:
- node execution
- artifact storage
- artifact transfer and egress
- backend usage

## Pricing and attribution

Backend pricing can express CPU, memory, GPU, and network classes.

Attribution is tracked per tenant and environment, and can be rolled up by DAG, run, and dataset references.

## Placement and routing

Cost-aware scheduler hints can prefer cheaper equivalent placements when trust, latency, locality, and policy constraints remain satisfied.

## Cache and planning economics

Planner-level cost estimation supports comparing candidate plans.

Cache reuse scoring makes recompute vs reuse an explicit economic decision.

## Budgets and controls

Run budgets support soft and hard ceilings.

Tenant budget policy can trigger allow, throttle, or reroute actions.

## Forecasting and anomaly detection

The model supports pre-run forecasting for schedules, backfills, and replay campaigns.

Anomaly detection flags abnormal spikes in backend, storage, or egress cost.

## Safety boundaries

Cost optimization is only valid when it preserves:
- determinism
- trust constraints
- compliance constraints

## Maturity scorecard

Cost maturity requires readiness across:
- unit economics
- attribution quality
- optimization safety

# State machine visualization

## Node transitions

`pending -> eligible -> queued -> running -> (success|failed|cached|cancelled)`

`eligible|queued -> skipped`

## Run transitions

`submitted -> planning -> running -> (succeeded|failed|cancelling)`

`running -> paused -> running`

`running -> interrupted -> (running|cancelling)`

`cancelling -> cancelled`

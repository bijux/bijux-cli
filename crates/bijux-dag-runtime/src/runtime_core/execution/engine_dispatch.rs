use crate::scheduler::ScheduleDecision;
use crate::{Graph, ReadyQueue, RuntimeConfig, Scheduler};
use std::time::Instant;

pub fn next_scheduler_decision(
    scheduler: &mut dyn Scheduler,
    graph: &Graph,
    ready_queue: &mut ReadyQueue,
    options: &RuntimeConfig,
    started: Instant,
    cancellation_requested: bool,
) -> ScheduleDecision {
    scheduler.next_batch(graph, ready_queue, options, started, cancellation_requested)
}

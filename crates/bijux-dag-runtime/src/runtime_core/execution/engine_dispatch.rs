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

#[cfg(test)]
mod tests {
    use super::next_scheduler_decision;
    use crate::scheduler::ScheduleDecision;
    use crate::{Graph, ReadyQueue, RuntimeConfig, Scheduler};
    use bijux_dag_core::parse_graph_strict;
    use std::collections::HashMap;
    use std::time::Instant;

    struct StubScheduler {
        seen_cancel_flag: Option<bool>,
    }

    impl Scheduler for StubScheduler {
        fn next_batch(
            &mut self,
            _graph: &Graph,
            _ready_queue: &mut ReadyQueue,
            _options: &RuntimeConfig,
            _started: Instant,
            cancellation_requested: bool,
        ) -> ScheduleDecision {
            self.seen_cancel_flag = Some(cancellation_requested);
            ScheduleDecision {
                batch: vec!["n1".to_string()],
                blocked_by_budget: vec![],
                timed_out: false,
                cancelled: cancellation_requested,
            }
        }
    }

    #[test]
    fn delegates_to_scheduler_with_original_cancel_flag() {
        let graph = parse_graph_strict(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[{"id":"n1","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":1}}],
              "edges":[]
            }"#,
        )
        .expect("graph");
        let mut indegree = HashMap::new();
        indegree.insert("n1".to_string(), 0usize);
        let mut ready = ReadyQueue::from_indegree(&indegree);
        let mut scheduler = StubScheduler {
            seen_cancel_flag: None,
        };
        let decision = next_scheduler_decision(
            &mut scheduler,
            &graph,
            &mut ready,
            &RuntimeConfig::default(),
            Instant::now(),
            true,
        );
        assert_eq!(scheduler.seen_cancel_flag, Some(true));
        assert_eq!(decision.batch, vec!["n1".to_string()]);
        assert!(decision.cancelled);
    }
}

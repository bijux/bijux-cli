use criterion::{criterion_group, criterion_main, Criterion};

fn sample_graph() -> &'static str {
    r#"{
      \"spec\": \"dag/v0.1\",
      \"meta\": {\"name\": \"bench\"},
      \"nodes\": [
        {
          \"id\": \"a\",
          \"kind\": \"const\",
          \"inputs\": [],
          \"outputs\": [{\"name\":\"out\",\"path\":\"out\"}],
          \"params\": {\"value\":\"x\"}
        }
      ],
      \"edges\": []
    }"#
}

fn bench_parse_validate(c: &mut Criterion) {
    c.bench_function("core.parse_validate.single_node", |b| {
        b.iter(|| {
            let graph = bijux_dag_core::parse_graph_strict(sample_graph()).expect("parse");
            let _ = graph.validate_with_warnings();
        })
    });
}

criterion_group!(benches, bench_parse_validate);
criterion_main!(benches);

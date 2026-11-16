use mo_offline_range_query::{
    Dataset, Query, Strategy, naive, trace,
    workload::{Workload, generate},
};

#[test]
fn trace_states_match_their_actual_windows() {
    let data = Dataset {
        array: vec![1, -2, 1, 3, -2],
        queries: vec![
            Query { left: 3, right: 4 },
            Query { left: 0, right: 0 },
            Query { left: 1, right: 3 },
        ],
    };
    for strategy in [Strategy::Block, Strategy::Alternating] {
        let t = trace::collect(&data, strategy, None, 2000).unwrap();
        assert_eq!(t.frames.len(), t.total_events);
        for f in &t.frames {
            let distinct = data.array[f.left..f.right_exclusive]
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert_eq!(f.distinct, distinct);
            assert_eq!(
                f.metrics.pointer_moves(),
                f.metrics.additions + f.metrics.removals
            );
        }
        let last = t.frames.last().unwrap();
        assert_eq!(
            last.distinct,
            naive::solve(&data).unwrap()[last.query_id.unwrap()]
        );
        let svg = trace::svg(&data, strategy, &t);
        assert!(svg.contains("<animate"));
        assert!(svg.ends_with("</g></svg>\n"));
    }
}

#[test]
fn sampling_is_bounded_and_keeps_both_endpoints() {
    let data = generate(10000, 2000, 42, Workload::Random).unwrap();
    let t = trace::collect(&data, Strategy::Alternating, None, 20).unwrap();
    assert_eq!(t.frames.len(), 20);
    assert_eq!(t.frames[0].metrics.pointer_moves(), 0);
    let expected = mo_offline_range_query::solve(&data, Strategy::Alternating, None).unwrap();
    assert_eq!(t.frames.last().unwrap().metrics, expected.metrics);
    assert!(trace::svg(&data, Strategy::Alternating, &t).len() < 100_000);
}

#[test]
fn empty_batch_and_invalid_frame_limits() {
    let data = Dataset {
        array: vec![9],
        queries: vec![],
    };
    let t = trace::collect(&data, Strategy::Block, None, 2).unwrap();
    assert_eq!(t.frames.len(), 1);
    assert!(t.frames[0].query_id.is_none());
    assert!(trace::collect(&data, Strategy::Block, None, 1).is_err());
}

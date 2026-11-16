use mo_offline_range_query::{Dataset, Query, Strategy, naive, solve};

fn check(array: Vec<i64>, queries: Vec<Query>) {
    let data = Dataset { array, queries };
    let expected = naive::solve(&data).unwrap();
    for strategy in [Strategy::Block, Strategy::Alternating] {
        for block in [None, Some(1), Some(3), Some(100)] {
            let result = solve(&data, strategy, block).unwrap();
            assert_eq!(result.answers, expected);
            assert_eq!(
                result.metrics.pointer_moves(),
                result.metrics.additions + result.metrics.removals
            );
        }
    }
}

#[test]
fn boundary_and_value_patterns() {
    for array in [
        vec![7],
        vec![5; 9],
        (0..9).collect(),
        vec![i64::MIN, -2, i64::MAX, -2, 0, i64::MIN],
    ] {
        let mut queries = vec![];
        for left in 0..array.len() {
            for right in left..array.len() {
                queries.push(Query { left, right });
            }
        }
        queries.reverse();
        queries.push(Query { left: 0, right: 0 });
        check(array, queries);
    }
}

#[test]
fn exhaustive_small_arrays_and_all_ranges() {
    for n in 1..=7 {
        for bits in 0..(1usize << n) {
            let array = (0..n).map(|i| ((bits >> i) & 1) as i64 - 1).collect();
            let queries = (0..n)
                .flat_map(|left| (left..n).map(move |right| Query { left, right }))
                .collect();
            check(array, queries);
        }
    }
}

#[test]
fn empty_batches_and_invalid_inputs() {
    check(vec![1, 2], vec![]);
    for data in [
        Dataset {
            array: vec![],
            queries: vec![],
        },
        Dataset {
            array: vec![1],
            queries: vec![Query { left: 0, right: 1 }],
        },
        Dataset {
            array: vec![1, 2],
            queries: vec![Query { left: 1, right: 0 }],
        },
    ] {
        assert!(naive::solve(&data).is_err());
        assert!(solve(&data, Strategy::Block, None).is_err());
    }
    assert!(
        solve(
            &Dataset {
                array: vec![1],
                queries: vec![]
            },
            Strategy::Block,
            Some(0)
        )
        .is_err()
    );
}

use mo_offline_range_query::{
    metrics,
    workload::{Workload, generate},
};

#[test]
fn reproducible_workloads_verify_all_strategies() {
    for kind in [
        Workload::Random,
        Workload::Clustered,
        Workload::Repeated,
        Workload::Short,
        Workload::Boundary,
    ] {
        let a = generate(100, 200, 42, kind).unwrap();
        let b = generate(100, 200, 42, kind).unwrap();
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        let report = metrics::compare(&a, None, 2).unwrap();
        assert!(report.measurements.iter().all(|m| m.verified));
    }
    assert!(generate(0, 1, 0, Workload::Random).is_err());
    assert!(generate(10_001, 1, 0, Workload::Random).is_err());
    assert!(generate(100, 2001, 0, Workload::Random).is_err());
    assert!(metrics::compare(&generate(100, 0, 0, Workload::Short).unwrap(), None, 0).is_err());
}

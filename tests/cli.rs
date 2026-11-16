use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mo-offline-range-query"))
}

#[test]
fn json_report_and_error_exit_statuses() {
    let path = std::env::temp_dir().join(format!("mo-report-{}.json", std::process::id()));
    let output = cli()
        .args([
            "compare",
            "--input",
            "examples/small.json",
            "--repetitions",
            "1",
            "--output",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(report["answers"], serde_json::json!([5, 2, 3, 1, 3, 3, 4]));
    assert_eq!(report["measurements"].as_array().unwrap().len(), 3);
    for args in [
        vec!["compare", "--block-size", "0"],
        vec!["compare", "--n", "10001"],
        vec!["compare", "--input", "examples/small.json", "--seed", "42"],
        vec!["trace", "--max-frames", "1"],
        vec!["compare", "--input", "missing-file.json"],
    ] {
        let output = cli().args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("error"));
    }
}

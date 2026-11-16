use crate::{Dataset, Metrics, Strategy, naive, solve};
use serde::Serialize;
use std::{hint::black_box, time::Instant};

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Timings {
    pub preprocessing_ns: u128,
    pub sorting_ns: u128,
    pub solving_ns: u128,
    pub total_ns: u128,
}

#[derive(Debug, Serialize)]
pub struct Measurement {
    pub strategy: String,
    pub verified: bool,
    pub block_size: Option<usize>,
    pub pointer_moves: Option<u64>,
    pub operations: Option<Metrics>,
    pub elements_scanned: Option<u64>,
    pub buffer_capacity_bytes: Option<usize>,
    pub timings_median: Timings,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub input: Dataset,
    pub n: usize,
    pub queries: usize,
    pub repetitions: usize,
    pub debug_build: bool,
    pub memory_note: &'static str,
    pub measurements: Vec<Measurement>,
    pub answers: Vec<usize>,
}

fn median(mut values: Vec<u128>) -> u128 {
    values.sort_unstable();
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        values[mid - 1] + (values[mid] - values[mid - 1]) / 2
    } else {
        values[mid]
    }
}

fn medians(samples: &[Timings]) -> Timings {
    Timings {
        preprocessing_ns: median(samples.iter().map(|t| t.preprocessing_ns).collect()),
        sorting_ns: median(samples.iter().map(|t| t.sorting_ns).collect()),
        solving_ns: median(samples.iter().map(|t| t.solving_ns).collect()),
        total_ns: median(samples.iter().map(|t| t.total_ns).collect()),
    }
}

pub fn compare(data: &Dataset, block: Option<usize>, repetitions: usize) -> Result<Report, String> {
    data.validate()?;
    if !(1..=100).contains(&repetitions) {
        return Err("repetitions must be in 1..=100".into());
    }
    // One untimed warmup for each strategy; oracle is independent of compression.
    let reference = naive::solve_measured(data)?;
    let answers = reference.answers;
    let mut samples = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        let start = Instant::now();
        let actual = naive::solve(black_box(data))?;
        black_box(&actual);
        let elapsed = start.elapsed().as_nanos();
        if actual != answers {
            return Err("naive results changed between runs".into());
        }
        samples.push(Timings {
            solving_ns: elapsed,
            total_ns: elapsed,
            ..Timings::default()
        });
    }
    let mut measurements = vec![Measurement {
        strategy: "naive".into(),
        verified: true,
        block_size: None,
        pointer_moves: None,
        operations: None,
        elements_scanned: Some(
            data.queries
                .iter()
                .map(|q| (q.right - q.left + 1) as u64)
                .sum(),
        ),
        buffer_capacity_bytes: Some(reference.buffer_bytes),
        timings_median: medians(&samples),
    }];
    for strategy in [Strategy::Block, Strategy::Alternating] {
        let warmup = solve(data, strategy, block)?;
        if warmup.answers != answers {
            return Err(format!("{strategy:?} verification failed"));
        }
        samples.clear();
        for _ in 0..repetitions {
            let result = solve(black_box(data), strategy, block)?;
            black_box(&result);
            // Validation and trace serialization are never part of this comparison.
            if result.answers != answers {
                return Err(format!("{strategy:?} verification failed"));
            }
            samples.push(result.timings);
        }
        measurements.push(Measurement {
            strategy: match strategy {
                Strategy::Block => "block",
                Strategy::Alternating => "alternating",
            }
            .into(),
            verified: true,
            block_size: Some(warmup.block_size),
            pointer_moves: Some(warmup.metrics.pointer_moves()),
            operations: Some(warmup.metrics),
            elements_scanned: None,
            buffer_capacity_bytes: Some(warmup.buffer_bytes),
            timings_median: medians(&samples),
        });
    }
    Ok(Report {
        input: data.clone(),
        n: data.array.len(),
        queries: data.queries.len(),
        repetitions,
        debug_build: cfg!(debug_assertions),
        memory_note: "Capacity estimates, not process peak memory: Mo counts owned Vec payloads; naive counts answer Vec plus maximum HashSet key capacity payload (lower bound, excludes spare buckets and control bytes). Both exclude input, stack, allocator overhead and trace. Phase medians need not sum to total median. Timings include operation counters and input validation.",
        measurements,
        answers,
    })
}

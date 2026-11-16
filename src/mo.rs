use crate::Dataset;
use crate::metrics::Timings;
use serde::Serialize;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Strategy {
    Block,
    Alternating,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Metrics {
    pub left_moves: u64,
    pub right_moves: u64,
    pub additions: u64,
    pub removals: u64,
    pub sort_comparisons: u64,
}

impl Metrics {
    pub fn pointer_moves(&self) -> u64 {
        self.left_moves + self.right_moves
    }
}

#[derive(Debug)]
pub struct Solution {
    pub timings: Timings,
    pub answers: Vec<usize>,
    pub order: Vec<usize>,
    pub metrics: Metrics,
    pub block_size: usize,
    /// Owned Vec payload capacities; excludes input, stack, allocator overhead.
    pub buffer_bytes: usize,
}

pub fn solve(
    data: &Dataset,
    strategy: Strategy,
    block_size: Option<usize>,
) -> Result<Solution, String> {
    solve_observed(data, strategy, block_size, |_, _, _, _, _| {})
}

/// Observer receives (query ID, left, exclusive right, distinct, metrics)
/// initially, after each move, and when each answer is recorded.
pub fn solve_observed(
    data: &Dataset,
    strategy: Strategy,
    block_size: Option<usize>,
    mut observe: impl FnMut(usize, usize, usize, usize, &Metrics),
) -> Result<Solution, String> {
    let start = Instant::now();
    data.validate()?;
    let block_size = block_size.unwrap_or_else(|| (data.array.len() as f64).sqrt() as usize);
    if block_size == 0 {
        return Err("block size must be positive".into());
    }
    let mut values = data.array.clone();
    values.sort_unstable();
    values.dedup();
    let compressed: Vec<usize> = data
        .array
        .iter()
        .map(|v| values.binary_search(v).unwrap())
        .collect();
    let mut frequencies = vec![0usize; values.len()];
    let mut order: Vec<usize> = (0..data.queries.len()).collect();
    let mut metrics = Metrics::default();
    let preprocessing_ns = start.elapsed().as_nanos();
    let sorting_start = Instant::now();
    order.sort_unstable_by(|&a, &b| {
        metrics.sort_comparisons += 1;
        let x = data.queries[a];
        let y = data.queries[b];
        let bx = x.left / block_size;
        let by = y.left / block_size;
        bx.cmp(&by)
            .then_with(|| {
                if strategy == Strategy::Alternating && bx % 2 == 1 {
                    y.right.cmp(&x.right)
                } else {
                    x.right.cmp(&y.right)
                }
            })
            .then(a.cmp(&b))
    });
    let sorting_ns = sorting_start.elapsed().as_nanos();
    let solving_start = Instant::now();
    let mut answers = vec![0; data.queries.len()];
    let (mut left, mut right, mut distinct) = (0usize, 0usize, 0usize);
    for &id in &order {
        let target = data.queries[id];
        observe(id, left, right, distinct, &metrics);
        // Expand before shrinking so disjoint windows never remove absent items.
        while left > target.left {
            left -= 1;
            add(&mut frequencies, compressed[left], &mut distinct);
            metrics.left_moves += 1;
            metrics.additions += 1;
            observe(id, left, right, distinct, &metrics);
        }
        while right <= target.right {
            add(&mut frequencies, compressed[right], &mut distinct);
            right += 1;
            metrics.right_moves += 1;
            metrics.additions += 1;
            observe(id, left, right, distinct, &metrics);
        }
        while left < target.left {
            remove(&mut frequencies, compressed[left], &mut distinct);
            left += 1;
            metrics.left_moves += 1;
            metrics.removals += 1;
            observe(id, left, right, distinct, &metrics);
        }
        while right > target.right + 1 {
            right -= 1;
            remove(&mut frequencies, compressed[right], &mut distinct);
            metrics.right_moves += 1;
            metrics.removals += 1;
            observe(id, left, right, distinct, &metrics);
        }
        answers[id] = distinct;
        observe(id, left, right, distinct, &metrics);
    }
    let buffer_bytes = values.capacity() * size_of::<i64>()
        + (compressed.capacity() + frequencies.capacity() + order.capacity() + answers.capacity())
            * size_of::<usize>();
    Ok(Solution {
        timings: Timings {
            preprocessing_ns,
            sorting_ns,
            solving_ns: solving_start.elapsed().as_nanos(),
            total_ns: start.elapsed().as_nanos(),
        },
        answers,
        order,
        metrics,
        block_size,
        buffer_bytes,
    })
}

fn add(f: &mut [usize], value: usize, distinct: &mut usize) {
    if f[value] == 0 {
        *distinct += 1;
    }
    f[value] += 1;
}

fn remove(f: &mut [usize], value: usize, distinct: &mut usize) {
    debug_assert!(f[value] > 0);
    f[value] -= 1;
    if f[value] == 0 {
        *distinct -= 1;
    }
}

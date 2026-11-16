use crate::Dataset;
use std::collections::HashSet;

/// Independent oracle: no compression or shared state with the Mo engine.
pub fn solve(data: &Dataset) -> Result<Vec<usize>, String> {
    Ok(solve_measured(data)?.answers)
}

pub struct NaiveSolution {
    pub answers: Vec<usize>,
    pub buffer_bytes: usize,
}

pub fn solve_measured(data: &Dataset) -> Result<NaiveSolution, String> {
    data.validate()?;
    let mut answers = Vec::with_capacity(data.queries.len());
    let mut peak_keys = 0;
    for q in &data.queries {
        let set: HashSet<_> = data.array[q.left..=q.right].iter().collect();
        peak_keys = peak_keys.max(set.capacity());
        answers.push(set.len());
    }
    let buffer_bytes = answers.capacity() * size_of::<usize>() + peak_keys * size_of::<&i64>();
    Ok(NaiveSolution {
        answers,
        buffer_bytes,
    })
}

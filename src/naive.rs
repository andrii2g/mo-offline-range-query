use crate::Dataset;
use std::collections::HashSet;

/// Independent oracle: no compression or shared state with the Mo engine.
pub fn solve(data: &Dataset) -> Result<Vec<usize>, String> {
    data.validate()?;
    Ok(data
        .queries
        .iter()
        .map(|q| {
            data.array[q.left..=q.right]
                .iter()
                .collect::<HashSet<_>>()
                .len()
        })
        .collect())
}

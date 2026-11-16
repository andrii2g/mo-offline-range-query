use serde::{Deserialize, Serialize};

/// Zero-based inclusive bounds. Position in `Dataset::queries` is the query ID.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Query {
    pub left: usize,
    pub right: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dataset {
    pub array: Vec<i64>,
    pub queries: Vec<Query>,
}

impl Dataset {
    pub fn validate(&self) -> Result<(), String> {
        if self.array.is_empty() {
            return Err("array must contain at least one element".into());
        }
        for (id, q) in self.queries.iter().enumerate() {
            if q.left > q.right || q.right >= self.array.len() {
                return Err(format!(
                    "query {id}: invalid inclusive range [{}, {}] for array length {}",
                    q.left,
                    q.right,
                    self.array.len()
                ));
            }
        }
        Ok(())
    }
}

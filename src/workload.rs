use crate::{Dataset, Query};
use clap::ValueEnum;
use serde::Serialize;

#[derive(Clone, Copy, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Workload {
    Random,
    Clustered,
    Repeated,
    Short,
    Boundary,
}

/// SplitMix64: specified integer arithmetic makes seeds reproducible across platforms.
struct Generator(u64);
impl Generator {
    fn index(&mut self, bound: usize) -> usize {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((z ^ (z >> 31)) % bound as u64) as usize
    }
}

pub fn generate(n: usize, q: usize, seed: u64, kind: Workload) -> Result<Dataset, String> {
    if !(1..=10_000).contains(&n) || q > 2_000 {
        return Err("generated workloads require 1..=10000 elements and 0..=2000 queries".into());
    }
    let mut rng = Generator(seed);
    let alphabet = (n / 5).max(1);
    let array = (0..n)
        .map(|_| rng.index(alphabet) as i64 - alphabet as i64 / 2)
        .collect();
    let mut queries = Vec::with_capacity(q);
    for id in 0..q {
        let a = rng.index(n);
        let b = rng.index(n);
        let (left, right) = match kind {
            Workload::Random => (a.min(b), a.max(b)),
            Workload::Clustered => {
                let center = [n / 4, n / 2, 3 * n / 4][id % 3];
                let radius = (n / 20).max(1);
                (
                    center.saturating_sub(a % radius),
                    (center + b % radius).min(n - 1),
                )
            }
            Workload::Repeated => {
                let left = (id % 5) * (n / 10);
                (left, (left + n / 3).min(n - 1))
            }
            Workload::Short => (a, (a + b % 16).min(n - 1)),
            Workload::Boundary => {
                if id % 2 == 0 {
                    (0, a)
                } else {
                    (a, n - 1)
                }
            }
        };
        queries.push(Query { left, right });
    }
    Ok(Dataset { array, queries })
}

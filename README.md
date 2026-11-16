# mo-offline-range-query

A Rust laboratory for **offline distinct-element range queries**. Compare a naïve
reference with conventional and alternating-direction Mo orderings, verify every
answer, and export the moving window as a standalone animated SVG.

## Run

Install a current stable Rust toolchain, then run from this repository:

```powershell
cargo run --release -- compare --input examples/small.json
cargo run --release -- compare --n 10000 --queries 2000 --seed 42 --output results.json
cargo run --release -- compare --n 10000 --queries 2000 --workload short --block-size 223
cargo run --release -- trace --input examples/small.json --strategy alternating --output trace.svg
```

Open `trace.svg` directly in a modern browser to play the looping animation.
Markdown image previews may show only a static frame. No server or JavaScript is
needed. Small arrays show values; large arrays use at most 100 overview bins.
The default trace retains at most 240 frames; `--max-frames 2..2000` controls the
limit. Each frame shows the actual state and exact counters, even when intermediate
events are skipped. The initial and final events are always retained.

Generated workloads default to 1,000 elements, 1,000 queries, and seed 42. Choose
`--workload random|clustered|repeated|short|boundary`. Generation uses specified
SplitMix64 arithmetic for repeatable inputs; this is an experiment generator, not
a statistical randomness test. The array's value alphabet is about `n / 5`.

The CLI supports 1–10,000 integers and 0–2,000 queries, including the intended
100–10,000-element MVP range. The library has no artificial size limit. Bounds
are checked before execution. Malformed input, invalid bounds, zero block size,
and invalid options produce a nonzero exit status.

## Input and answers

```json
{
  "array": [1, -2, 1, 3],
  "queries": [
    { "left": 0, "right": 3 },
    { "left": 1, "right": 2 }
  ]
}
```

Bounds are **zero-based and inclusive**. Answers here are `[3, 2]`, returned in
the original query order. JSON reports include the full input and answers so an
experiment remains inspectable and reproducible. For replay, save the report's
`input` object as a new input JSON file. Input files cannot be combined with
generation flags. Integers are signed 64-bit values.

## How it works

The independent naïve solver builds a fresh `HashSet` for every range. Each Mo
solver compresses integer values to dense IDs, sorts query IDs, then reuses a
frequency vector and distinct counter while moving an active window `[L, R)`.
Frequency transitions `0 → 1` and `1 → 0` update the distinct count in constant
time. The engine expands before shrinking, including between disjoint ranges.

| Ordering | Sort key |
| --- | --- |
| Block | Left block ascending, right endpoint ascending, original ID |
| Alternating | Left block ascending, right endpoint ascending in even blocks and descending in odd blocks, original ID |

The default block size is `floor(sqrt(n))`; `--block-size` allows experiments.
Approximately `n / sqrt(q)` is another useful choice when `n` and `q` differ.
Alternating direction often saves movement but is not guaranteed to win for every
batch. See [Alex's Mo algorithm reference](https://algorithms.alexli.ca/mo-s-algorithm/).

For block size `B`, window updates cost `O(qB + n²/B)`, query sorting costs
`O(q log q)`, and compression costs `O(n log n)`. With `B ≈ sqrt(n)`, the usual
window bound is `O((n + q) sqrt(n))`. Algorithm buffers occupy `O(n + q + u)`
space for `u` unique values, excluding input and optional trace storage. No
persistent query index is built: the entire query batch must be available first.

## Reading measurements

The console gives a compact comparison; `--output results.json` includes:

- Left/right pointer moves, additions, removals, and total moves. For Mo,
  `moves = additions + removals`.
- Query-sort comparator calls. Compression comparisons are excluded; tie-breaking
  makes order deterministic. Comparator counts may change with Rust's sort implementation.
- Naïve elements scanned; pointer moves and sort operations are not applicable.
- Nanosecond timings for preprocessing, sorting, solving, and total execution.
- Explicit buffer-capacity estimates in bytes.

Every strategy receives an untimed warmup, followed by `--repetitions` timed runs
(default 7, allowed 1–100). The report contains independent per-phase medians;
these need not sum to the total median. Every run's answers are compared outside
its timed region. Tracing, generation, JSON serialization, and verification are
excluded; counters, allocation, and bounds validation are included. Mo
preprocessing includes validation, compression, and frequency/order allocation;
solving includes answer allocation. Naïve work is reported entirely as solving.
Timing is machine-dependent; use release mode and compare operation counts too.

Memory figures are **capacity estimates, not process peak memory**. Mo counts its
owned vector payloads. Naïve counts its answer vector and maximum live HashSet
key-capacity payload, a lower bound excluding spare buckets and control bytes.
Both exclude shared input, stack, and allocator overhead. The trace command
separately reports sampled-frame/order buffer capacity and SVG output bytes;
these are not a trace-generation peak-memory measurement.

For example, seed 42 with `n=10000`, `q=2000`, random queries and block size 100
produces 6,389,710 naïve element visits, 965,650 block-order moves, and 550,248
alternating-order moves. Short or repeated workloads can behave differently.

## Library

```rust
use mo_offline_range_query::{Dataset, Query, Strategy, solve};

let data = Dataset {
    array: vec![1, -2, 1, 3],
    queries: vec![Query { left: 0, right: 3 }],
};
let result = solve(&data, Strategy::Alternating, None)?;
assert_eq!(result.answers, vec![3]);
# Ok::<(), String>(())
```

## Validation

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Tests cover exhaustive binary arrays through length seven with every valid range,
all-equal/all-distinct arrays, extreme integers, repeated queries, empty batches,
invalid bounds, multiple block sizes, generated workload families, and trace
state invariants and sampling limits. CI runs the checks on Linux and Windows.

The project focuses on static arrays and distinct counts. Online updates, Hilbert
ordering, generic aggregates, and interactive animation controls are future work.

Licensed under MIT.

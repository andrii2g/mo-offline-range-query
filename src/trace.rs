use crate::{Dataset, Metrics, Strategy, mo, naive};
use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct Frame {
    pub query_id: Option<usize>,
    pub left: usize,
    pub right_exclusive: usize,
    pub distinct: usize,
    pub metrics: Metrics,
}

pub struct Trace {
    pub frames: Vec<Frame>,
    pub total_events: usize,
    pub buffer_bytes: usize,
    pub block_size: usize,
    pub order: Vec<usize>,
}

pub fn collect(
    data: &Dataset,
    strategy: Strategy,
    block: Option<usize>,
    max_frames: usize,
) -> Result<Trace, String> {
    if !(2..=2000).contains(&max_frames) {
        return Err("max frames must be in 2..=2000".into());
    }
    let baseline = mo::solve(data, strategy, block)?;
    if baseline.answers != naive::solve(data)? {
        return Err("trace verification failed".into());
    }
    let total_events = (baseline.metrics.pointer_moves() as usize + 2 * data.queries.len()).max(1);
    let count = total_events.min(max_frames);
    let mut frames = Vec::with_capacity(count);
    if data.queries.is_empty() {
        frames.push(Frame {
            query_id: None,
            left: 0,
            right_exclusive: 0,
            distinct: 0,
            metrics: Metrics::default(),
        });
    } else {
        let mut event = 0usize;
        mo::solve_observed(
            data,
            strategy,
            block,
            |id, left, right_exclusive, distinct, metrics| {
                let next = frames.len();
                if next < count && event == next * (total_events - 1) / (count - 1) {
                    frames.push(Frame {
                        query_id: Some(id),
                        left,
                        right_exclusive,
                        distinct,
                        metrics: metrics.clone(),
                    });
                }
                event += 1;
            },
        )?;
        debug_assert_eq!(event, total_events);
    }
    let buffer_bytes =
        frames.capacity() * size_of::<Frame>() + baseline.order.capacity() * size_of::<usize>();
    Ok(Trace {
        frames,
        total_events,
        buffer_bytes,
        block_size: baseline.block_size,
        order: baseline.order,
    })
}

/// Standalone SMIL SVG; open directly in a modern browser to play the looping trace.
pub fn svg(data: &Dataset, strategy: Strategy, trace: &Trace) -> String {
    let mut out = String::new();
    let n = data.array.len();
    let x = |i: usize| 60.0 + 1080.0 * i as f64 / n as f64;
    let duration = (trace.frames.len() as f64 * 0.25).clamp(4.0, 30.0);
    write!(out, r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 440" role="img" aria-labelledby="title desc">
<title id="title">Mo's algorithm: {strategy:?} order</title>
<desc id="desc">Animated distinct-count window. Teal is the active window, amber is the target range. Frames retain exact counters; large traces are sampled.</desc>
<rect width="1200" height="440" rx="16" fill="#0f172a"/>
<g font-family="system-ui, sans-serif" fill="#e2e8f0">
<text x="40" y="42" font-size="25" font-weight="700">Mo's algorithm / {strategy:?}</text>
<text x="40" y="72" font-size="15">n={n} · queries={} · block={} · {} / {} events · {:.1}s loop</text>
<rect x="60" y="150" width="1080" height="48" fill="#1e293b" stroke="#64748b"/>
"##, data.queries.len(), trace.block_size, trace.frames.len(), trace.total_events, duration).unwrap();
    // At most 100 grid bins, so overview size is independent of array length.
    let bins = n.min(100);
    for bin in 0..=bins {
        let px = 60.0 + 1080.0 * bin as f64 / bins as f64;
        write!(out, r##"<path d="M {px:.2} 150 v48" stroke="#475569"/>"##).unwrap();
    }
    if n <= 32 {
        for (i, value) in data.array.iter().enumerate() {
            write!(
                out,
                r##"<text x="{:.2}" y="180" text-anchor="middle" font-size="{}">{value}</text>"##,
                (x(i) + x(i + 1)) / 2.0,
                if value.to_string().len() > 5 { 7 } else { 15 }
            )
            .unwrap();
        }
    }
    write!(out, r##"<text x="60" y="219" font-size="13">index 0</text><text x="1140" y="219" text-anchor="end" font-size="13">exclusive end {n}</text>
<text x="40" y="362" font-size="14" fill="#5eead4">Teal: active window [L, R)</text>
<text x="370" y="362" font-size="14" fill="#fbbf24">Amber: target inclusive query</text>
<text x="40" y="392" font-size="13">Large arrays use an overview strip; sampled frames skip intermediate states, never counter updates.</text>
"##).unwrap();
    for (i, f) in trace.frames.iter().enumerate() {
        let start = i as f64 / trace.frames.len() as f64;
        let end = (i + 1) as f64 / trace.frames.len() as f64;
        let visibility = if i == 0 { "visible" } else { "hidden" };
        write!(out, r##"<g visibility="{visibility}" data-frame="{i}">
<animate attributeName="visibility" calcMode="discrete" values="hidden;visible;hidden;hidden" keyTimes="0;{start:.8};{end:.8};1" dur="{duration:.3}s" repeatCount="indefinite"/>
<rect x="{:.2}" y="150" width="{:.2}" height="48" fill="#2dd4bf" fill-opacity="0.3" stroke="#5eead4"/>
<path d="M {:.2} 143 v-15 M {:.2} 143 v-15" stroke="#5eead4" stroke-width="2"/>
<text x="{:.2}" y="122" text-anchor="middle" font-size="14" fill="#5eead4">L</text>
<text x="{:.2}" y="109" text-anchor="middle" font-size="14" fill="#5eead4">R</text>
"##, x(f.left), x(f.right_exclusive)-x(f.left), x(f.left), x(f.right_exclusive), x(f.left), x(f.right_exclusive)).unwrap();
        if let Some(id) = f.query_id {
            let q = data.queries[id];
            write!(
                out,
                r##"<rect x="{:.2}" y="234" width="{:.2}" height="8" fill="#fbbf24"/>
<text x="40" y="278" font-size="19">Query #{id}: [{}, {}] · active [{}, {}) · distinct={}</text>"##,
                x(q.left),
                x(q.right + 1) - x(q.left),
                q.left,
                q.right,
                f.left,
                f.right_exclusive,
                f.distinct
            )
            .unwrap();
        } else {
            out.push_str(r#"<text x="40" y="278" font-size="19">Empty query batch · empty active window · distinct=0</text>"#);
        }
        write!(out, r##"<text x="40" y="315" font-size="17">Moves={} (L={} / R={}) · additions={} · removals={} · frame {}/{}</text></g>"##,
            f.metrics.pointer_moves(), f.metrics.left_moves, f.metrics.right_moves, f.metrics.additions, f.metrics.removals, i+1, trace.frames.len()).unwrap();
    }
    out.push_str("</g></svg>\n");
    out
}

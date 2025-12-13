//! Common utilities for benchmarks

use criterion::Criterion;
use pprof::criterion::{Output, PProfProfiler};

/// Configure criterion with flamegraph profiling support
pub fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(3))
        .measurement_time(std::time::Duration::from_secs(10))
        .sample_size(100)
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)))
}
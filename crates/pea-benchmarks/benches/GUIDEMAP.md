# Benchmarks Directory

This directory contains comprehensive performance benchmarks for the Pea package manager with flamegraph profiling, comparison tools, and regression detection.

## Structure

### Benchmark Files
- `install.rs` - Installation performance benchmarks
- `resolve.rs` - Dependency resolution benchmarks  
- `hash.rs` - File hashing and CAS benchmarks
- `parse.rs` - Configuration parsing benchmarks

### Scripts (in `/scripts/`)
- `benchmark-workflow.sh` - Complete benchmarking workflow
- `profile-*.sh` - Flamegraph profiling scripts
- `bench-*.sh` - Hyperfine comparison scripts
- `store-benchmark-results.sh` - Result storage and tracking
- `generate-trend-report.sh` - Historical trend analysis
- `detect-regressions.sh` - Performance regression detection

## Running Benchmarks

### Quick Start
```bash
# Complete benchmarking workflow
./scripts/benchmark-workflow.sh

# Run criterion benchmarks only
cargo bench -p pea-benchmarks

# Run specific benchmark
cargo bench -p pea-benchmarks --bench install

# Test mode (faster)
cargo bench -p pea-benchmarks --bench install -- --test
```

### Profiling
```bash
# Generate flamegraphs for all components
./scripts/profile-all.sh

# Profile specific components
./scripts/profile-install.sh
./scripts/profile-resolve.sh
./scripts/profile-parse.sh
```

### Package Manager Comparisons
```bash
# Compare against npm, yarn, pnpm, bun
./scripts/bench-all.sh

# Individual comparisons
./scripts/bench-cold-start.sh
./scripts/bench-resolve.sh
./scripts/bench-install.sh
```

### Result Management
```bash
# Store results with metadata
./scripts/store-benchmark-results.sh

# Generate trend report
./scripts/generate-trend-report.sh

# Detect regressions (>10% threshold)
./scripts/detect-regressions.sh
```

## Benchmark Categories

### Installation Benchmarks (`install.rs`)
- Fresh installation performance (1-25 dependencies)
- Cached installation performance (10-100 packages)
- Parallel download performance (1-20 concurrent)
- node_modules creation performance (10-500 packages)

### Resolution Benchmarks (`resolve.rs`)
- Dependency tree resolution (10-1000 packages)
- Version selection algorithms (10-500 versions)
- Conflict detection (50-500 packages)
- Semver operations (parsing, comparison, matching)
- Parallel resolution (1-8 concurrent resolvers)

### Hashing Benchmarks (`hash.rs`)
- File hashing (1KB-10MB files)
- Memory hashing performance
- Parallel directory hashing (10-500 files)
- CAS store/retrieve operations
- Concurrent CAS operations (1-16 threads)
- Hash comparisons and conversions
- Integrity verification
- Algorithm comparisons (Blake3 vs SHA256 vs SHA1)

### Parsing Benchmarks (`parse.rs`)
- pea.toml parsing (10-500 dependencies)
- package.json parsing (10-500 dependencies)
- Semver parsing (simple and complex versions)
- Configuration validation
- TOML/JSON serialization performance

## Profiling Infrastructure

### Flamegraph Integration
- Integrated with pprof for flamegraph generation
- Automatic profiling during criterion benchmarks
- Separate profiling scripts for targeted analysis
- SVG flamegraphs for performance bottleneck identification

### Comparison Framework
- Hyperfine integration for package manager comparisons
- Automated comparison against npm, yarn, pnpm, bun
- JSON and Markdown result export
- Statistical analysis with warmup and multiple runs

## Result Storage and Analysis

### Historical Tracking
- JSON storage of all benchmark results
- Git commit and branch tracking
- System metadata (OS, arch, versions)
- Timestamped results with symlinks to latest

### Trend Analysis
- Historical performance trend reports
- Markdown table generation
- Performance improvement/degradation tracking
- Multi-run comparison analysis

### Regression Detection
- Configurable regression threshold (default: 10%)
- Automatic baseline comparison
- Detailed regression reports
- CI/CD integration ready

## Performance Targets

- **Cold start**: <5ms
- **Resolution**: Competitive with fastest package manager
- **Installation**: Competitive with fastest package manager
- **Parsing**: <2ms per file
- **Regression threshold**: <10% performance degradation

## Configuration

### Criterion Settings
- 3 second warmup period
- 10 second measurement time
- 100 samples for statistical significance
- Flamegraph profiling enabled
- HTML reports in `target/criterion/`

### Hyperfine Settings
- Configurable warmup and run counts
- JSON and Markdown export
- Preparation and cleanup commands
- Statistical analysis and comparison

## Generated Files

### Reports
- `benchmark-workflow-summary.md` - Complete workflow summary
- `benchmark-trend-report.md` - Historical trends
- `regression-report.md` - Regression analysis
- `*-benchmark.md` - Hyperfine comparison tables

### Data
- `benchmark-results/` - Historical data storage
- `*-benchmark.json` - Hyperfine results
- `target/criterion/` - Criterion results and flamegraphs
- `flamegraph-*.svg` - Profiling flamegraphs

## CI/CD Integration

The benchmarking infrastructure is designed for CI/CD integration:
- Exit codes indicate regression status
- JSON results for automated processing
- Configurable thresholds
- Historical baseline comparison
- Automated report generation
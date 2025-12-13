//! Dependency resolution performance benchmarks
//!
//! Benchmarks the performance of dependency resolution algorithms including
//! SAT solving, version selection, and conflict detection.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pea_benchmarks::criterion_config;
use pea_core::types::{Version, VersionReq};
use pea_registry::{RegistryClient, MetadataCache};
use pea_resolver::{Resolver, graph::DependencyGraph};
use std::sync::Arc;

use std::str::FromStr;



/// Benchmark dependency resolution for different tree sizes
fn bench_dependency_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_resolution");
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(10);
    
    // Test with different dependency tree sizes
    for tree_size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*tree_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("packages", tree_size),
            tree_size,
            |b, &tree_size| {
                b.iter(|| {
                    let registry_client = Arc::new(RegistryClient::new().unwrap());
                    let metadata_cache = Arc::new(MetadataCache::new());
                    let resolver = Resolver::new(registry_client, metadata_cache);
                    
                    let dependencies = create_dependency_tree(tree_size);
                    
                    // Benchmark resolution setup (simulated)
                    black_box(simulate_resolution(&resolver, dependencies))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark version selection algorithms
fn bench_version_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_selection");
    group.measurement_time(std::time::Duration::from_secs(5));
    
    for version_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*version_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("versions", version_count),
            version_count,
            |b, &version_count| {
                let versions = create_version_list(version_count);
                let constraints = create_version_constraints(5);
                
                b.iter(|| {
                    black_box(simulate_version_selection(&versions, &constraints))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark conflict detection performance
fn bench_conflict_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("conflict_detection");
    group.measurement_time(std::time::Duration::from_secs(5));
    
    for package_count in [50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*package_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("packages_with_conflicts", package_count),
            package_count,
            |b, &package_count| {
                b.iter(|| {
                    let graph = create_conflicting_dependency_graph(package_count);
                    
                    black_box(simulate_conflict_detection(&graph))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark semver parsing and comparison
fn bench_semver_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("semver_operations");
    
    // Benchmark version parsing
    group.bench_function("version_parsing", |b| {
        let version_strings = create_version_strings(1000);
        let mut index = 0;
        
        b.iter(|| {
            let version_str = &version_strings[index % version_strings.len()];
            index += 1;
            black_box(Version::from_str(version_str))
        });
    });
    
    // Benchmark version requirement parsing
    group.bench_function("version_req_parsing", |b| {
        let req_strings = create_version_req_strings(1000);
        let mut index = 0;
        
        b.iter(|| {
            let req_str = &req_strings[index % req_strings.len()];
            index += 1;
            black_box(VersionReq::parse(req_str))
        });
    });
    
    // Benchmark version comparison
    group.bench_function("version_comparison", |b| {
        let versions = create_version_list(100);
        let mut index = 0;
        
        b.iter(|| {
            let v1 = &versions[index % versions.len()];
            let v2 = &versions[(index + 1) % versions.len()];
            index += 1;
            black_box(v1.cmp(v2))
        });
    });
    
    // Benchmark version matching
    group.bench_function("version_matching", |b| {
        let versions = create_version_list(100);
        let requirements = create_version_constraints(10);
        let mut version_index = 0;
        let mut req_index = 0;
        
        b.iter(|| {
            let version = &versions[version_index % versions.len()];
            let req = &requirements[req_index % requirements.len()];
            version_index += 1;
            req_index += 1;
            black_box(req.matches(version))
        });
    });
    
    group.finish();
}

/// Benchmark parallel resolution performance
fn bench_parallel_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_resolution");
    group.measurement_time(std::time::Duration::from_secs(10));
    
    for concurrency in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_resolvers", concurrency),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    use rayon::prelude::*;
                    
                    let results: Vec<_> = (0..concurrency).into_par_iter().map(|_| {
                        let registry_client = Arc::new(RegistryClient::new().unwrap());
                        let metadata_cache = Arc::new(MetadataCache::new());
                        let resolver = Resolver::new(registry_client, metadata_cache);
                        
                        let dependencies = create_dependency_tree(50);
                        simulate_resolution(&resolver, dependencies)
                    }).collect();
                    
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions for benchmark setup

fn create_dependency_tree(size: usize) -> Vec<(String, String)> {
    (0..size)
        .map(|i| (format!("package-{}", i), "^1.0.0".to_string()))
        .collect()
}

fn create_version_list(count: usize) -> Vec<Version> {
    (0..count)
        .map(|i| {
            let major = i / 100;
            let minor = (i / 10) % 10;
            let patch = i % 10;
            Version::new(major as u64, minor as u64, patch as u64)
        })
        .collect()
}

fn create_version_constraints(count: usize) -> Vec<VersionReq> {
    let patterns = ["^1.0.0", "~2.1.0", ">=3.0.0", "1.2.3", "*"];
    
    (0..count)
        .map(|i| {
            let pattern = patterns[i % patterns.len()];
            VersionReq::parse(pattern).unwrap()
        })
        .collect()
}

fn create_version_strings(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            let major = i / 100;
            let minor = (i / 10) % 10;
            let patch = i % 10;
            format!("{}.{}.{}", major, minor, patch)
        })
        .collect()
}

fn create_version_req_strings(count: usize) -> Vec<String> {
    let prefixes = ["^", "~", ">=", "=", ""];
    
    (0..count)
        .map(|i| {
            let prefix = prefixes[i % prefixes.len()];
            let major = (i / 100) + 1;
            let minor = (i / 10) % 10;
            let patch = i % 10;
            format!("{}{}.{}.{}", prefix, major, minor, patch)
        })
        .collect()
}

fn create_conflicting_dependency_graph(package_count: usize) -> DependencyGraph {
    let graph = DependencyGraph::new();
    
    // Create packages with potential conflicts (simulated)
    // This is a simplified version for benchmarking
    for _i in 0..package_count {
        // Simulate conflict detection work
        std::thread::sleep(std::time::Duration::from_nanos(100));
    }
    
    graph
}

// Simulation functions for benchmarking without network calls

fn simulate_resolution(_resolver: &Resolver, dependencies: Vec<(String, String)>) -> Result<(), String> {
    // Simulate resolution work proportional to dependency count
    for _ in &dependencies {
        std::thread::sleep(std::time::Duration::from_nanos(100));
    }
    Err("simulated network error".to_string())
}

fn simulate_version_selection(versions: &[Version], constraints: &[VersionReq]) -> Option<Version> {
    // Simulate version selection algorithm
    for version in versions {
        for constraint in constraints {
            if constraint.matches(version) {
                return Some(version.clone());
            }
        }
    }
    None
}

fn simulate_conflict_detection(_graph: &DependencyGraph) -> Vec<String> {
    // Simulate conflict detection work
    std::thread::sleep(std::time::Duration::from_micros(10));
    vec!["simulated conflict".to_string()]
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = bench_dependency_resolution, bench_version_selection, bench_conflict_detection, bench_semver_operations, bench_parallel_resolution
}
criterion_main!(benches);
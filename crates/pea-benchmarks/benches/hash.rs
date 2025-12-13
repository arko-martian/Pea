//! File hashing and content-addressable storage performance benchmarks
//!
//! Benchmarks the performance of Blake3 hashing, parallel directory hashing,
//! and content-addressable storage operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pea_benchmarks::criterion_config;
use pea_cache::{CasStore, cas::hash::{ContentHash, compute_hash}};
use pea_core::utils::hash::{blake3_hash, blake3_hash_file};
use std::sync::Arc;
use tempfile::tempdir;
use camino::Utf8PathBuf;



/// Benchmark file hashing performance for different file sizes
fn bench_file_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_hashing");
    group.measurement_time(std::time::Duration::from_secs(10));
    
    // Test with different file sizes
    for size in [1024, 10_240, 102_400, 1_024_000, 10_240_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file");
        let content = create_test_content(*size);
        std::fs::write(&file_path, &content).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("blake3_hash_file", size),
            &file_path,
            |b, path| {
                b.iter(|| {
                    black_box(blake3_hash_file(path).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark in-memory hashing performance
fn bench_memory_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_hashing");
    group.measurement_time(std::time::Duration::from_secs(5));
    
    for size in [1024, 10_240, 102_400, 1_024_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        let content = create_test_content(*size);
        
        group.bench_with_input(
            BenchmarkId::new("blake3_hash", size),
            &content,
            |b, data| {
                b.iter(|| {
                    black_box(blake3_hash(data))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark parallel directory hashing
fn bench_parallel_directory_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_directory_hashing");
    group.measurement_time(std::time::Duration::from_secs(15));
    
    for file_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*file_count as u64));
        
        let temp_dir = create_test_directory(*file_count, 10_240);
        
        group.bench_with_input(
            BenchmarkId::new("files", file_count),
            &temp_dir,
            |b, dir| {
                b.iter(|| {
                    black_box(hash_directory_parallel(dir.path()))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark CAS store operations
fn bench_cas_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cas_operations");
    group.measurement_time(std::time::Duration::from_secs(10));
    
    // Benchmark store operations
    for size in [1024, 10_240, 102_400, 1_024_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        let content = create_test_content(*size);
        
        group.bench_with_input(
            BenchmarkId::new("store", size),
            &content,
            |b, data| {
                b.iter(|| {
                    let temp_dir = tempdir().unwrap();
                    let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().join("store")).unwrap();
                    let cas_store = CasStore::new(&store_path).unwrap();
                    
                    black_box(cas_store.store(data).unwrap())
                });
            },
        );
    }
    
    // Benchmark retrieve operations
    for size in [1024, 10_240, 102_400, 1_024_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        let content = create_test_content(*size);
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().join("store")).unwrap();
        let cas_store = CasStore::new(&store_path).unwrap();
        let hash = cas_store.store(&content).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("retrieve", size),
            &hash,
            |b, hash| {
                b.iter(|| {
                    black_box(cas_store.get(hash).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent CAS operations
fn bench_concurrent_cas_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_cas_operations");
    group.measurement_time(std::time::Duration::from_secs(15));
    
    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_stores", concurrency),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    use rayon::prelude::*;
                    
                    let temp_dir = tempdir().unwrap();
                    let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().join("store")).unwrap();
                    let cas_store = Arc::new(CasStore::new(&store_path).unwrap());
                    
                    let results: Vec<_> = (0..concurrency).into_par_iter().map(|_| {
                        let content = create_test_content(10_240);
                        cas_store.store(&content)
                    }).collect();
                    
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark hash comparison operations
fn bench_hash_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_comparison");
    
    // Create test hashes
    let hashes: Vec<ContentHash> = (0..1000)
        .map(|i| {
            let content = format!("test content {}", i);
            compute_hash(content.as_bytes())
        })
        .collect();
    
    group.bench_function("hash_equality", |b| {
        let mut index = 0;
        
        b.iter(|| {
            let hash1 = &hashes[index % hashes.len()];
            let hash2 = &hashes[(index + 1) % hashes.len()];
            index += 1;
            black_box(hash1 == hash2)
        });
    });
    
    group.bench_function("hash_to_hex", |b| {
        let mut index = 0;
        
        b.iter(|| {
            let hash = &hashes[index % hashes.len()];
            index += 1;
            black_box(hash.to_hex())
        });
    });
    
    group.bench_function("hash_from_hex", |b| {
        let hex_strings: Vec<String> = hashes.iter().map(|h| h.to_hex()).collect();
        let mut index = 0;
        
        b.iter(|| {
            let hex = &hex_strings[index % hex_strings.len()];
            index += 1;
            black_box(ContentHash::from_hex(hex))
        });
    });
    
    group.finish();
}

/// Benchmark integrity verification
fn bench_integrity_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("integrity_verification");
    group.measurement_time(std::time::Duration::from_secs(10));
    
    for size in [1024, 10_240, 102_400, 1_024_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        let content = create_test_content(*size);
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().join("store")).unwrap();
        let cas_store = CasStore::new(&store_path).unwrap();
        let hash = cas_store.store(&content).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("verify", size),
            &(cas_store, hash),
            |b, (store, hash)| {
                b.iter(|| {
                    black_box(store.verify(hash).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark hash algorithm comparison
fn bench_hash_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_algorithms");
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let content = create_test_content(102_400); // 100KB test content
    
    group.bench_function("blake3", |b| {
        b.iter(|| {
            black_box(blake3_hash(&content))
        });
    });
    
    group.bench_function("sha256", |b| {
        use sha2::{Sha256, Digest};
        
        b.iter(|| {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            black_box(hasher.finalize())
        });
    });
    
    group.bench_function("sha1", |b| {
        use sha1::{Sha1, Digest};
        
        b.iter(|| {
            let mut hasher = Sha1::new();
            hasher.update(&content);
            black_box(hasher.finalize())
        });
    });
    
    group.finish();
}

// Helper functions for benchmark setup

fn create_test_content(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn create_test_directory(file_count: usize, file_size: usize) -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    
    for i in 0..file_count {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));
        let content = create_test_content(file_size);
        std::fs::write(file_path, content).unwrap();
    }
    
    temp_dir
}

fn hash_directory_parallel(dir_path: &std::path::Path) -> Result<Vec<ContentHash>, Box<dyn std::error::Error>> {
    use rayon::prelude::*;
    use std::fs;
    
    let entries: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .collect();
    
    let hashes: Vec<ContentHash> = entries
        .par_iter()
        .map(|entry| {
            let content = fs::read(entry.path()).unwrap();
            compute_hash(&content)
        })
        .collect();
    
    Ok(hashes)
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = bench_file_hashing, bench_memory_hashing, bench_parallel_directory_hashing, bench_cas_operations, bench_concurrent_cas_operations, bench_hash_comparison, bench_integrity_verification, bench_hash_algorithms
}
criterion_main!(benches);
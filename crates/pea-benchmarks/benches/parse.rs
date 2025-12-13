//! Configuration and metadata parsing performance benchmarks
//!
//! Benchmarks the performance of parsing pea.toml, package.json, and semver
//! strings across different file sizes and complexity levels.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pea_benchmarks::criterion_config;
use pea_config::{toml::parse_pea_toml, json::parse_package_json, PeaToml, PackageJson, toml::serialize_pea_toml};
use pea_core::types::{Version, VersionReq};
use std::collections::HashMap;
use std::str::FromStr;



/// Benchmark pea.toml parsing performance
fn bench_pea_toml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("pea_toml_parsing");
    group.measurement_time(std::time::Duration::from_secs(5));
    
    // Test with different numbers of dependencies
    for dep_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*dep_count as u64));
        
        let toml_content = create_pea_toml_content(*dep_count);
        
        group.bench_with_input(
            BenchmarkId::new("dependencies", dep_count),
            &toml_content,
            |b, content| {
                b.iter(|| {
                    black_box(parse_pea_toml(content).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark package.json parsing performance
fn bench_package_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("package_json_parsing");
    group.measurement_time(std::time::Duration::from_secs(5));
    
    for dep_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*dep_count as u64));
        
        let json_content = create_package_json_content(*dep_count);
        
        group.bench_with_input(
            BenchmarkId::new("dependencies", dep_count),
            &json_content,
            |b, content| {
                b.iter(|| {
                    black_box(parse_package_json(content).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark semver parsing performance
fn bench_semver_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("semver_parsing");
    
    // Benchmark version parsing
    group.bench_function("version_parsing_simple", |b| {
        let versions = create_simple_versions(1000);
        let mut index = 0;
        
        b.iter(|| {
            let version = &versions[index % versions.len()];
            index += 1;
            black_box(Version::from_str(version))
        });
    });
    
    group.bench_function("version_parsing_complex", |b| {
        let versions = create_complex_versions(1000);
        let mut index = 0;
        
        b.iter(|| {
            let version = &versions[index % versions.len()];
            index += 1;
            black_box(Version::from_str(version))
        });
    });
    
    // Benchmark version requirement parsing
    group.bench_function("version_req_parsing_simple", |b| {
        let requirements = create_simple_requirements(1000);
        let mut index = 0;
        
        b.iter(|| {
            let req = &requirements[index % requirements.len()];
            index += 1;
            black_box(VersionReq::parse(req))
        });
    });
    
    group.bench_function("version_req_parsing_complex", |b| {
        let requirements = create_complex_requirements(1000);
        let mut index = 0;
        
        b.iter(|| {
            let req = &requirements[index % requirements.len()];
            index += 1;
            black_box(VersionReq::parse(req))
        });
    });
    
    group.finish();
}

/// Benchmark configuration validation performance
fn bench_config_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_validation");
    group.measurement_time(std::time::Duration::from_secs(5));
    
    for complexity in ["simple", "medium", "complex"].iter() {
        group.bench_with_input(
            BenchmarkId::new("validation", complexity),
            complexity,
            |b, &complexity| {
                b.iter(|| {
                    let config = create_config_for_validation(complexity);
                    
                    // Simulate validation process
                    black_box(validate_config(&config))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark TOML serialization performance
fn bench_toml_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_serialization");
    
    for dep_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*dep_count as u64));
        
        let config = create_pea_toml_struct(*dep_count);
        
        group.bench_with_input(
            BenchmarkId::new("serialize", dep_count),
            &config,
            |b, config| {
                b.iter(|| {
                    black_box(serialize_pea_toml(config).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark JSON serialization performance
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");
    
    for dep_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*dep_count as u64));
        
        let package_json = create_package_json_struct(*dep_count);
        
        group.bench_with_input(
            BenchmarkId::new("serialize", dep_count),
            &package_json,
            |b, package_json| {
                b.iter(|| {
                    black_box(serde_json::to_string_pretty(package_json).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions for benchmark setup

fn create_pea_toml_content(dep_count: usize) -> String {
    let mut content = String::from(
        r#"[package]
name = "benchmark-test"
version = "1.0.0"
description = "A test package for benchmarking"
license = "MIT"

[dependencies]
"#,
    );
    
    for i in 0..dep_count {
        content.push_str(&format!("package-{} = \"^{}.{}.{}\"\n", i, i % 5 + 1, i % 10, i % 5));
    }
    
    if dep_count > 10 {
        content.push_str("\n[dev-dependencies]\n");
        for i in 0..(dep_count / 4) {
            content.push_str(&format!("dev-package-{} = \"~{}.{}.{}\"\n", i, i % 3 + 1, i % 8, i % 3));
        }
    }
    
    if dep_count > 50 {
        content.push_str("\n[scripts]\n");
        content.push_str("build = \"tsc\"\n");
        content.push_str("test = \"jest\"\n");
        content.push_str("start = \"node dist/index.js\"\n");
        
        content.push_str("\n[features]\n");
        content.push_str("default = [\"feature1\"]\n");
        content.push_str("feature1 = []\n");
        content.push_str("feature2 = [\"feature1\"]\n");
    }
    
    content
}

fn create_package_json_content(dep_count: usize) -> String {
    let mut dependencies = HashMap::new();
    let mut dev_dependencies = HashMap::new();
    
    for i in 0..dep_count {
        dependencies.insert(
            format!("package-{}", i),
            format!("^{}.{}.{}", i % 5 + 1, i % 10, i % 5)
        );
    }
    
    for i in 0..(dep_count / 4) {
        dev_dependencies.insert(
            format!("dev-package-{}", i),
            format!("~{}.{}.{}", i % 3 + 1, i % 8, i % 3)
        );
    }
    
    let package_json = serde_json::json!({
        "name": "benchmark-test",
        "version": "1.0.0",
        "description": "A test package for benchmarking",
        "main": "index.js",
        "license": "MIT",
        "dependencies": dependencies,
        "devDependencies": dev_dependencies,
        "scripts": {
            "build": "tsc",
            "test": "jest",
            "start": "node dist/index.js"
        }
    });
    
    serde_json::to_string_pretty(&package_json).unwrap()
}

fn create_simple_versions(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("{}.{}.{}", i % 10, (i / 10) % 10, (i / 100) % 10))
        .collect()
}

fn create_complex_versions(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            if i % 4 == 0 {
                format!("{}.{}.{}-alpha.{}", i % 10, (i / 10) % 10, (i / 100) % 10, i % 5)
            } else if i % 4 == 1 {
                format!("{}.{}.{}-beta.{}+build.{}", i % 10, (i / 10) % 10, (i / 100) % 10, i % 3, i % 7)
            } else if i % 4 == 2 {
                format!("{}.{}.{}-rc.{}", i % 10, (i / 10) % 10, (i / 100) % 10, i % 2)
            } else {
                format!("{}.{}.{}", i % 10, (i / 10) % 10, (i / 100) % 10)
            }
        })
        .collect()
}

fn create_simple_requirements(count: usize) -> Vec<String> {
    let prefixes = ["^", "~", ">=", "="];
    
    (0..count)
        .map(|i| {
            let prefix = prefixes[i % prefixes.len()];
            format!("{}{}.{}.{}", prefix, i % 10, (i / 10) % 10, (i / 100) % 10)
        })
        .collect()
}

fn create_complex_requirements(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            match i % 6 {
                0 => format!("^{}.{}.{}", i % 10, (i / 10) % 10, (i / 100) % 10),
                1 => format!("~{}.{}.{}", i % 10, (i / 10) % 10, (i / 100) % 10),
                2 => format!(">={}.{}.{} <{}.{}.{}", i % 10, (i / 10) % 10, (i / 100) % 10, (i % 10) + 1, 0, 0),
                3 => format!("{}.{}.{} || {}.{}.{}", i % 10, (i / 10) % 10, (i / 100) % 10, (i % 10) + 1, 0, 0),
                4 => "*".to_string(),
                _ => format!("={}.{}.{}", i % 10, (i / 10) % 10, (i / 100) % 10),
            }
        })
        .collect()
}

fn create_config_for_validation(complexity: &str) -> PeaToml {
    use pea_config::{PackageSection, DependencySpec};
    use pea_core::types::Version;
    
    let dep_count = match complexity {
        "simple" => 5,
        "medium" => 25,
        "complex" => 100,
        _ => 10,
    };
    
    let mut dependencies = HashMap::new();
    for i in 0..dep_count {
        dependencies.insert(
            format!("package-{}", i),
            DependencySpec::Simple(format!("^{}.{}.{}", i % 5 + 1, i % 10, i % 5))
        );
    }
    
    PeaToml {
        package: PackageSection {
            name: format!("benchmark-{}", complexity),
            version: Version::new(1, 0, 0),
            description: Some(format!("A {} test package", complexity)),
            main: Some("index.js".to_string()),
            license: Some("MIT".to_string()),
            repository: None,
            keywords: vec!["benchmark".to_string(), complexity.to_string()],
            authors: Vec::new(),
            homepage: None,
        },
        dependencies,
        dev_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        workspace: None,
        profile: HashMap::new(),
        scripts: HashMap::new(),
        features: HashMap::new(),
    }
}

fn validate_config(config: &PeaToml) -> Result<(), String> {
    // Simulate validation logic
    if config.package.name.is_empty() {
        return Err("Package name cannot be empty".to_string());
    }
    
    // Validate dependencies
    for (name, spec) in &config.dependencies {
        if name.is_empty() {
            return Err("Dependency name cannot be empty".to_string());
        }
        
        match spec {
            pea_config::DependencySpec::Simple(version) => {
                if VersionReq::parse(version).is_err() {
                    return Err(format!("Invalid version requirement: {}", version));
                }
            }
            pea_config::DependencySpec::Detailed { version: Some(version), .. } => {
                if VersionReq::parse(version).is_err() {
                    return Err(format!("Invalid version requirement: {}", version));
                }
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn create_pea_toml_struct(dep_count: usize) -> PeaToml {
    use pea_config::{PackageSection, DependencySpec};
    use pea_core::types::Version;
    
    let mut dependencies = HashMap::new();
    for i in 0..dep_count {
        dependencies.insert(
            format!("package-{}", i),
            DependencySpec::Simple(format!("^{}.{}.{}", i % 5 + 1, i % 10, i % 5))
        );
    }
    
    PeaToml {
        package: PackageSection {
            name: "benchmark-test".to_string(),
            version: Version::new(1, 0, 0),
            description: Some("A test package for benchmarking".to_string()),
            main: Some("index.js".to_string()),
            license: Some("MIT".to_string()),
            repository: None,
            keywords: vec!["benchmark".to_string()],
            authors: Vec::new(),
            homepage: None,
        },
        dependencies,
        dev_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        workspace: None,
        profile: HashMap::new(),
        scripts: HashMap::new(),
        features: HashMap::new(),
    }
}

fn create_package_json_struct(dep_count: usize) -> PackageJson {
    let mut dependencies = HashMap::new();
    for i in 0..dep_count {
        dependencies.insert(
            format!("package-{}", i),
            format!("^{}.{}.{}", i % 5 + 1, i % 10, i % 5)
        );
    }
    
    PackageJson {
        name: "benchmark-test".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A test package for benchmarking".to_string()),
        main: Some("index.js".to_string()),
        license: Some("MIT".to_string()),
        repository: None,
        keywords: Vec::new(),
        author: None,
        authors: Vec::new(),
        homepage: None,
        dependencies,
        dev_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        bundled_dependencies: Vec::new(),
        scripts: HashMap::new(),
        workspaces: None,
        engines: HashMap::new(),
        os: Vec::new(),
        cpu: Vec::new(),
        private: false,
        files: Vec::new(),
        bin: None,
        module_type: None,
        exports: None,
        types: None,
        typings: None,
    }
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = bench_pea_toml_parsing, bench_package_json_parsing, bench_semver_parsing, bench_config_validation, bench_toml_serialization, bench_json_serialization
}
criterion_main!(benches);
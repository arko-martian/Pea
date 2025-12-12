# 🫛 Pea - Project Guide Map

> A blazingly fast JavaScript/TypeScript runtime & package manager written in Rust.

## Project Structure

```
pea/
├── Cargo.toml          # Workspace configuration
├── Cargo.lock          # Dependency lock file
├── GUIDEMAP.md         # This file - project overview
├── README.md           # Project documentation
├── rustfmt.toml        # Rust formatting rules
├── clippy.toml         # Clippy linting rules
├── .cargo/
│   └── config.toml     # Cargo build configuration
├── crates/             # All Rust crates (see below)
├── benches/            # Performance benchmarks
├── fuzz/               # Fuzz testing targets
├── tests/              # Integration tests
└── docs/               # Documentation (mdBook)
```

## Crate Architecture

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `pea-cli` | CLI entry point, command routing | All crates |
| `pea-core` | Shared types, errors, utilities | None |
| `pea-config` | pea.toml & package.json parsing | pea-core |
| `pea-registry` | npm registry client | pea-core |
| `pea-resolver` | Dependency resolution (SAT solver) | pea-core, pea-registry |
| `pea-cache` | Content-addressable storage | pea-core |
| `pea-lockfile` | Binary lockfile (rkyv) | pea-core |
| `pea-runtime` | JavaScript execution (JSC) | pea-core, pea-parser |
| `pea-parser` | TypeScript/JavaScript parsing (oxc) | pea-core |
| `pea-bundler` | Code bundling & optimization | pea-core, pea-parser |
| `pea-test` | Test runner | pea-core, pea-runtime |
| `pea-plugin` | Wasm plugin system | pea-core |

## Code Quality Rules

### 🔴 CRITICAL: Maximum 4 Public Functions Per File

Every source file MUST contain no more than 4 public functions. This ensures:
- High cohesion within modules
- Easy navigation and understanding
- Clear separation of concerns
- Maintainable codebase

### 🔴 CRITICAL: GUIDEMAP.md in Every Directory

Every directory MUST contain a `GUIDEMAP.md` file documenting:
- Purpose of the directory
- Contents and their responsibilities
- Relationships with other modules
- Usage examples where applicable

### Code Style

- **Formatting**: Run `cargo fmt` before committing
- **Linting**: Run `cargo lint` (alias for clippy) - zero warnings allowed
- **Documentation**: All public items must have doc comments
- **Testing**: All core logic must have unit tests
- **Unsafe**: Document safety invariants for any unsafe code

## Concurrency Model

```
┌─────────────────────────────────────────────────────┐
│                   Application Layer                  │
├─────────────────────────────────────────────────────┤
│  Tokio (I/O)  │  Rayon (CPU)  │  DashMap (State)   │
│  - Network    │  - Parsing    │  - Caches          │
│  - Filesystem │  - Hashing    │  - Resolved deps   │
│  - Async ops  │  - Resolution │  - Module cache    │
└─────────────────────────────────────────────────────┘
```

- **Tokio**: All I/O-bound operations (network, filesystem)
- **Rayon**: All CPU-bound operations (parsing, hashing, resolution)
- **DashMap**: Lock-free concurrent state management

## Performance Targets

| Metric | Target | Strategy |
|--------|--------|----------|
| Cold Start | <5ms | JSC pre-init, LTO |
| Install | <30ms | Rayon parallel, prefetch |
| TS Parse | <2ms/file | oxc + SIMD |
| HTTP RPS | 110k+ | Tokio + HTTP/3 |

## Development Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build

# Test
cargo test               # Run all tests
cargo test -p pea-core   # Test specific crate

# Lint
cargo lint               # Run clippy
cargo fmt-check          # Check formatting

# Benchmark
cargo bench              # Run benchmarks

# Run
cargo run -- --version   # Run CLI
cargo run -- new my-app  # Create new project
```

## Contributing

1. Follow the 4-function-per-file rule
2. Add GUIDEMAP.md to new directories
3. Write tests for all new functionality
4. Run `cargo lint` and `cargo fmt` before committing
5. Update documentation as needed

---

*Let's make JavaScript fast!* 🫛🚀

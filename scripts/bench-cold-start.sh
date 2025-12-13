#!/bin/bash
# Hyperfine comparison script for cold start performance
# Compares pea vs bun vs npm vs pnpm vs yarn for startup time

set -e

echo "❄️ Benchmarking cold start performance..."

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo "❌ hyperfine is not installed. Please install it first:"
    echo "   brew install hyperfine  # macOS"
    echo "   cargo install hyperfine # Cross-platform"
    exit 1
fi

echo "📊 Running cold start benchmarks..."

# Check which package managers are available
AVAILABLE_PMS=()
COMMANDS=()

# Check pea (our package manager)
if command -v pea &> /dev/null; then
    AVAILABLE_PMS+=("pea")
    COMMANDS+=("pea --version")
fi

# Check npm
if command -v npm &> /dev/null; then
    AVAILABLE_PMS+=("npm")
    COMMANDS+=("npm --version")
fi

# Check yarn
if command -v yarn &> /dev/null; then
    AVAILABLE_PMS+=("yarn")
    COMMANDS+=("yarn --version")
fi

# Check pnpm
if command -v pnpm &> /dev/null; then
    AVAILABLE_PMS+=("pnpm")
    COMMANDS+=("pnpm --version")
fi

# Check bun
if command -v bun &> /dev/null; then
    AVAILABLE_PMS+=("bun")
    COMMANDS+=("bun --version")
fi

if [ ${#AVAILABLE_PMS[@]} -eq 0 ]; then
    echo "❌ No package managers found. Please install at least one package manager."
    exit 1
fi

echo "📦 Found package managers: ${AVAILABLE_PMS[*]}"

# Run hyperfine benchmark for cold start
echo "🚀 Running hyperfine benchmark for cold start..."

hyperfine \
    --warmup 0 \
    --runs 50 \
    --export-json "cold-start-benchmark.json" \
    --export-markdown "cold-start-benchmark.md" \
    "${COMMANDS[@]}"

echo "✅ Cold start benchmark complete!"
echo ""
echo "📊 Results saved to:"
echo "  - cold-start-benchmark.json (JSON format)"
echo "  - cold-start-benchmark.md (Markdown table)"
echo ""
echo "🎯 Target: pea should start in <5ms"
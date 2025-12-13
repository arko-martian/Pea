#!/bin/bash
# Run all hyperfine comparison benchmarks

set -e

echo "🏁 Running comprehensive package manager benchmarks..."

# Make sure all scripts are executable
chmod +x scripts/bench-*.sh

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo "❌ hyperfine is not installed. Please install it first:"
    echo "   brew install hyperfine  # macOS"
    echo "   cargo install hyperfine # Cross-platform"
    exit 1
fi

echo "📊 Running all benchmark suites..."

# Run cold start benchmark
echo ""
echo "❄️ 1/3 Running cold start benchmark..."
./scripts/bench-cold-start.sh

# Run resolution benchmark
echo ""
echo "🧩 2/3 Running resolution benchmark..."
./scripts/bench-resolve.sh

# Run installation benchmark
echo ""
echo "📦 3/3 Running installation benchmark..."
./scripts/bench-install.sh

echo ""
echo "✅ All benchmarks complete!"
echo ""
echo "📊 Generated benchmark files:"
echo "  - cold-start-benchmark.json/md (startup time)"
echo "  - resolve-benchmark.json/md (dependency resolution)"
echo "  - install-benchmark.json/md (full installation)"
echo ""
echo "🎯 Performance targets:"
echo "  - Cold start: <5ms"
echo "  - Resolution: competitive with fastest PM"
echo "  - Installation: competitive with fastest PM"
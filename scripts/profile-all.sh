#!/bin/bash
# Run all profiling scripts and generate comprehensive flamegraphs

set -e

echo "🔥 Running comprehensive performance profiling..."

# Make sure all scripts are executable
chmod +x scripts/profile-*.sh

# Run all individual profiling scripts
echo "📊 Profiling installation performance..."
./scripts/profile-install.sh

echo "📊 Profiling resolution performance..."
./scripts/profile-resolve.sh

echo "📊 Profiling parsing performance..."
./scripts/profile-parse.sh

echo "✅ All profiling complete!"
echo ""
echo "Generated flamegraphs:"
echo "  - flamegraph-install.svg (installation performance)"
echo "  - flamegraph-resolve.svg (dependency resolution performance)"
echo "  - flamegraph-parse.svg (configuration parsing performance)"
echo ""
echo "🎯 Open these SVG files in a browser to analyze performance bottlenecks."
#!/bin/bash
# Profile installation performance with flamegraph

set -e

echo "🔥 Profiling pea install performance..."

# Ensure cargo-flamegraph is installed
if ! command -v cargo-flamegraph &> /dev/null; then
    echo "Installing cargo-flamegraph..."
    cargo install flamegraph
fi

# Create a temporary test project
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Create a test pea.toml with dependencies
cat > pea.toml << EOF
[package]
name = "profile-test"
version = "1.0.0"
description = "Test package for profiling"

[dependencies]
lodash = "^4.17.21"
express = "^4.18.2"
react = "^18.2.0"
typescript = "^5.0.0"
jest = "^29.0.0"
EOF

echo "📊 Running flamegraph profiling..."

# Profile the install command
cargo flamegraph --bin pea -- install --frozen

echo "✅ Flamegraph generated: flamegraph.svg"
echo "📁 Profile data saved in: $TEMP_DIR"

# Copy flamegraph back to project root
cp flamegraph.svg "$OLDPWD/flamegraph-install.svg"

echo "🎯 Flamegraph copied to: flamegraph-install.svg"

# Cleanup
cd "$OLDPWD"
rm -rf "$TEMP_DIR"
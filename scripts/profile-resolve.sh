#!/bin/bash
# Profile dependency resolution performance with flamegraph

set -e

echo "🔥 Profiling pea dependency resolution performance..."

# Ensure cargo-flamegraph is installed
if ! command -v cargo-flamegraph &> /dev/null; then
    echo "Installing cargo-flamegraph..."
    cargo install flamegraph
fi

# Create a temporary test project with complex dependencies
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Create a test pea.toml with many dependencies to stress resolution
cat > pea.toml << EOF
[package]
name = "resolve-profile-test"
version = "1.0.0"
description = "Test package for profiling resolution"

[dependencies]
# Popular packages with complex dependency trees
react = "^18.2.0"
react-dom = "^18.2.0"
next = "^14.0.0"
express = "^4.18.2"
lodash = "^4.17.21"
axios = "^1.6.0"
typescript = "^5.0.0"
jest = "^29.0.0"
webpack = "^5.89.0"
babel-core = "^6.26.3"
eslint = "^8.0.0"
prettier = "^3.0.0"
moment = "^2.29.4"
uuid = "^9.0.0"
chalk = "^5.3.0"
commander = "^11.0.0"
inquirer = "^9.2.0"
fs-extra = "^11.1.0"
glob = "^10.3.0"
rimraf = "^5.0.0"

[dev-dependencies]
@types/node = "^20.0.0"
@types/react = "^18.2.0"
@types/express = "^4.17.0"
@types/lodash = "^4.14.0"
@types/uuid = "^9.0.0"
@types/jest = "^29.0.0"
ts-node = "^10.9.0"
nodemon = "^3.0.0"
EOF

echo "📊 Running flamegraph profiling for resolution..."

# Profile just the resolution phase (without actual downloads)
cargo flamegraph --bin pea -- install --dry-run

echo "✅ Flamegraph generated: flamegraph.svg"
echo "📁 Profile data saved in: $TEMP_DIR"

# Copy flamegraph back to project root
cp flamegraph.svg "$OLDPWD/flamegraph-resolve.svg"

echo "🎯 Flamegraph copied to: flamegraph-resolve.svg"

# Cleanup
cd "$OLDPWD"
rm -rf "$TEMP_DIR"
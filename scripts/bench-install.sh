#!/bin/bash
# Hyperfine comparison script for package installation performance
# Compares pea vs bun vs npm vs pnpm vs yarn

set -e

echo "🏁 Benchmarking package installation performance..."

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo "❌ hyperfine is not installed. Please install it first:"
    echo "   brew install hyperfine  # macOS"
    echo "   cargo install hyperfine # Cross-platform"
    exit 1
fi

# Create a temporary test project
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "📁 Created test directory: $TEMP_DIR"

# Create a test package.json with common dependencies
cat > package.json << 'EOF'
{
  "name": "install-benchmark",
  "version": "1.0.0",
  "description": "Benchmark test for package managers",
  "dependencies": {
    "lodash": "^4.17.21",
    "express": "^4.18.2",
    "react": "^18.2.0",
    "axios": "^1.6.0",
    "moment": "^2.29.4",
    "uuid": "^9.0.0",
    "chalk": "^5.3.0",
    "commander": "^11.0.0",
    "fs-extra": "^11.1.0",
    "glob": "^10.3.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@types/lodash": "^4.14.0",
    "typescript": "^5.0.0",
    "jest": "^29.0.0",
    "eslint": "^8.0.0"
  }
}
EOF

# Create equivalent pea.toml
cat > pea.toml << 'EOF'
[package]
name = "install-benchmark"
version = "1.0.0"
description = "Benchmark test for package managers"

[dependencies]
lodash = "^4.17.21"
express = "^4.18.2"
react = "^18.2.0"
axios = "^1.6.0"
moment = "^2.29.4"
uuid = "^9.0.0"
chalk = "^5.3.0"
commander = "^11.0.0"
fs-extra = "^11.1.0"
glob = "^10.3.0"

[dev-dependencies]
"@types/node" = "^20.0.0"
"@types/lodash" = "^4.14.0"
typescript = "^5.0.0"
jest = "^29.0.0"
eslint = "^8.0.0"
EOF

echo "📊 Running installation benchmarks..."

# Prepare cleanup function
cleanup() {
    echo "🧹 Cleaning up..."
    rm -rf node_modules package-lock.json yarn.lock pnpm-lock.yaml bun.lockb pea.lock .pnpm-store
}

# Check which package managers are available
AVAILABLE_PMS=()
COMMANDS=()

# Check pea (our package manager)
if command -v pea &> /dev/null; then
    AVAILABLE_PMS+=("pea")
    COMMANDS+=("pea install")
fi

# Check npm
if command -v npm &> /dev/null; then
    AVAILABLE_PMS+=("npm")
    COMMANDS+=("npm install")
fi

# Check yarn
if command -v yarn &> /dev/null; then
    AVAILABLE_PMS+=("yarn")
    COMMANDS+=("yarn install")
fi

# Check pnpm
if command -v pnpm &> /dev/null; then
    AVAILABLE_PMS+=("pnpm")
    COMMANDS+=("pnpm install")
fi

# Check bun
if command -v bun &> /dev/null; then
    AVAILABLE_PMS+=("bun")
    COMMANDS+=("bun install")
fi

if [ ${#AVAILABLE_PMS[@]} -eq 0 ]; then
    echo "❌ No package managers found. Please install at least one package manager."
    exit 1
fi

echo "📦 Found package managers: ${AVAILABLE_PMS[*]}"

# Run hyperfine benchmark
echo "🚀 Running hyperfine benchmark..."

hyperfine \
    --warmup 1 \
    --runs 5 \
    --prepare "cleanup" \
    --export-json "$OLDPWD/install-benchmark.json" \
    --export-markdown "$OLDPWD/install-benchmark.md" \
    "${COMMANDS[@]}"

echo "✅ Benchmark complete!"
echo ""
echo "📊 Results saved to:"
echo "  - install-benchmark.json (JSON format)"
echo "  - install-benchmark.md (Markdown table)"
echo ""
echo "📁 Test directory: $TEMP_DIR"
echo "   (You can manually inspect the results there)"

# Cleanup
cd "$OLDPWD"
# Don't remove temp dir automatically so user can inspect if needed
echo "🗑️  To cleanup: rm -rf $TEMP_DIR"
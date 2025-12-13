#!/bin/bash
# Hyperfine comparison script for dependency resolution performance
# Compares pea vs bun vs npm vs pnpm vs yarn for resolution speed

set -e

echo "🧩 Benchmarking dependency resolution performance..."

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

# Create a complex package.json with many dependencies to stress resolution
cat > package.json << 'EOF'
{
  "name": "resolve-benchmark",
  "version": "1.0.0",
  "description": "Benchmark test for dependency resolution",
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "next": "^14.0.0",
    "express": "^4.18.2",
    "lodash": "^4.17.21",
    "axios": "^1.6.0",
    "typescript": "^5.0.0",
    "jest": "^29.0.0",
    "webpack": "^5.89.0",
    "babel-core": "^6.26.3",
    "eslint": "^8.0.0",
    "prettier": "^3.0.0",
    "moment": "^2.29.4",
    "uuid": "^9.0.0",
    "chalk": "^5.3.0",
    "commander": "^11.0.0",
    "inquirer": "^9.2.0",
    "fs-extra": "^11.1.0",
    "glob": "^10.3.0",
    "rimraf": "^5.0.0",
    "cross-env": "^7.0.3",
    "dotenv": "^16.3.0",
    "cors": "^2.8.5",
    "helmet": "^7.1.0",
    "morgan": "^1.10.0",
    "bcryptjs": "^2.4.3",
    "jsonwebtoken": "^9.0.2",
    "mongoose": "^8.0.0",
    "redis": "^4.6.0",
    "socket.io": "^4.7.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@types/react": "^18.2.0",
    "@types/express": "^4.17.0",
    "@types/lodash": "^4.14.0",
    "@types/uuid": "^9.0.0",
    "@types/jest": "^29.0.0",
    "@types/cors": "^2.8.0",
    "@types/morgan": "^1.9.0",
    "@types/bcryptjs": "^2.4.0",
    "@typescript-eslint/eslint-plugin": "^6.0.0",
    "@typescript-eslint/parser": "^6.0.0",
    "ts-node": "^10.9.0",
    "nodemon": "^3.0.0",
    "concurrently": "^8.2.0",
    "husky": "^8.0.0",
    "lint-staged": "^15.0.0"
  }
}
EOF

# Create equivalent pea.toml
cat > pea.toml << 'EOF'
[package]
name = "resolve-benchmark"
version = "1.0.0"
description = "Benchmark test for dependency resolution"

[dependencies]
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
cross-env = "^7.0.3"
dotenv = "^16.3.0"
cors = "^2.8.5"
helmet = "^7.1.0"
morgan = "^1.10.0"
bcryptjs = "^2.4.3"
jsonwebtoken = "^9.0.2"
mongoose = "^8.0.0"
redis = "^4.6.0"
"socket.io" = "^4.7.0"

[dev-dependencies]
"@types/node" = "^20.0.0"
"@types/react" = "^18.2.0"
"@types/express" = "^4.17.0"
"@types/lodash" = "^4.14.0"
"@types/uuid" = "^9.0.0"
"@types/jest" = "^29.0.0"
"@types/cors" = "^2.8.0"
"@types/morgan" = "^1.9.0"
"@types/bcryptjs" = "^2.4.0"
"@typescript-eslint/eslint-plugin" = "^6.0.0"
"@typescript-eslint/parser" = "^6.0.0"
ts-node = "^10.9.0"
nodemon = "^3.0.0"
concurrently = "^8.2.0"
husky = "^8.0.0"
lint-staged = "^15.0.0"
EOF

echo "📊 Running resolution benchmarks..."

# Prepare cleanup function
cleanup() {
    echo "🧹 Cleaning up lockfiles..."
    rm -rf node_modules package-lock.json yarn.lock pnpm-lock.yaml bun.lockb pea.lock .pnpm-store
}

# Check which package managers are available
AVAILABLE_PMS=()
COMMANDS=()

# Check pea (our package manager) - use dry-run to only test resolution
if command -v pea &> /dev/null; then
    AVAILABLE_PMS+=("pea")
    COMMANDS+=("pea install --dry-run")
fi

# Check npm - use --dry-run to only test resolution
if command -v npm &> /dev/null; then
    AVAILABLE_PMS+=("npm")
    COMMANDS+=("npm install --dry-run")
fi

# Check yarn - resolution only
if command -v yarn &> /dev/null; then
    AVAILABLE_PMS+=("yarn")
    # Yarn doesn't have a pure resolution flag, so we'll use install with --prefer-offline
    COMMANDS+=("yarn install --prefer-offline --ignore-scripts")
fi

# Check pnpm - resolution only
if command -v pnpm &> /dev/null; then
    AVAILABLE_PMS+=("pnpm")
    COMMANDS+=("pnpm install --dry-run")
fi

# Check bun - resolution only
if command -v bun &> /dev/null; then
    AVAILABLE_PMS+=("bun")
    COMMANDS+=("bun install --dry-run")
fi

if [ ${#AVAILABLE_PMS[@]} -eq 0 ]; then
    echo "❌ No package managers found. Please install at least one package manager."
    exit 1
fi

echo "📦 Found package managers: ${AVAILABLE_PMS[*]}"

# Run hyperfine benchmark
echo "🚀 Running hyperfine benchmark for resolution..."

hyperfine \
    --warmup 2 \
    --runs 10 \
    --prepare "cleanup" \
    --export-json "$OLDPWD/resolve-benchmark.json" \
    --export-markdown "$OLDPWD/resolve-benchmark.md" \
    "${COMMANDS[@]}"

echo "✅ Resolution benchmark complete!"
echo ""
echo "📊 Results saved to:"
echo "  - resolve-benchmark.json (JSON format)"
echo "  - resolve-benchmark.md (Markdown table)"
echo ""
echo "📁 Test directory: $TEMP_DIR"

# Cleanup
cd "$OLDPWD"
echo "🗑️  To cleanup: rm -rf $TEMP_DIR"
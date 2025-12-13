#!/bin/bash
# Profile configuration parsing performance with flamegraph

set -e

echo "🔥 Profiling pea configuration parsing performance..."

# Ensure cargo-flamegraph is installed
if ! command -v cargo-flamegraph &> /dev/null; then
    echo "Installing cargo-flamegraph..."
    cargo install flamegraph
fi

# Create a temporary test project with large config files
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Create a large pea.toml with many dependencies
cat > pea.toml << 'EOF'
[package]
name = "parse-profile-test"
version = "1.0.0"
description = "Test package for profiling parsing with many dependencies"
license = "MIT"
repository = "https://github.com/test/test"
keywords = ["test", "profiling", "parsing", "benchmark"]
authors = ["Test Author <test@example.com>"]

[dependencies]
EOF

# Generate many dependencies to stress the parser
for i in {1..500}; do
    echo "package-$i = \"^$((i % 10 + 1)).$((i % 20)).$((i % 5))\"" >> pea.toml
done

cat >> pea.toml << 'EOF'

[dev-dependencies]
EOF

# Generate dev dependencies
for i in {1..100}; do
    echo "dev-package-$i = \"~$((i % 5 + 1)).$((i % 10)).$((i % 3))\"" >> pea.toml
done

cat >> pea.toml << 'EOF'

[scripts]
build = "tsc"
test = "jest"
start = "node dist/index.js"
dev = "nodemon src/index.ts"
lint = "eslint src/**/*.ts"
format = "prettier --write src/**/*.ts"
clean = "rimraf dist"
prebuild = "npm run clean"
postbuild = "npm run test"
pretest = "npm run build"

[features]
default = ["feature1", "feature2"]
feature1 = []
feature2 = ["feature1"]
feature3 = ["feature2"]
experimental = ["feature3"]
debug = []
production = []

[profile.dev]
optimization = false
debug = true

[profile.release]
optimization = true
debug = false
minify = true
EOF

# Also create a large package.json for comparison
cat > package.json << 'EOF'
{
  "name": "parse-profile-test",
  "version": "1.0.0",
  "description": "Test package for profiling parsing",
  "main": "index.js",
  "license": "MIT",
  "dependencies": {
EOF

# Generate JSON dependencies
for i in {1..500}; do
    if [ $i -eq 500 ]; then
        echo "    \"json-package-$i\": \"^$((i % 10 + 1)).$((i % 20)).$((i % 5))\"" >> package.json
    else
        echo "    \"json-package-$i\": \"^$((i % 10 + 1)).$((i % 20)).$((i % 5))\"," >> package.json
    fi
done

cat >> package.json << 'EOF'
  },
  "devDependencies": {
EOF

# Generate JSON dev dependencies
for i in {1..100}; do
    if [ $i -eq 100 ]; then
        echo "    \"json-dev-package-$i\": \"~$((i % 5 + 1)).$((i % 10)).$((i % 3))\"" >> package.json
    else
        echo "    \"json-dev-package-$i\": \"~$((i % 5 + 1)).$((i % 10)).$((i % 3))\"," >> package.json
    fi
done

cat >> package.json << 'EOF'
  },
  "scripts": {
    "build": "tsc",
    "test": "jest",
    "start": "node dist/index.js",
    "dev": "nodemon src/index.ts",
    "lint": "eslint src/**/*.ts",
    "format": "prettier --write src/**/*.ts"
  }
}
EOF

echo "📊 Running flamegraph profiling for parsing..."

# Profile the parsing by running a command that parses config
cargo flamegraph --bin pea -- check

echo "✅ Flamegraph generated: flamegraph.svg"
echo "📁 Profile data saved in: $TEMP_DIR"

# Copy flamegraph back to project root
cp flamegraph.svg "$OLDPWD/flamegraph-parse.svg"

echo "🎯 Flamegraph copied to: flamegraph-parse.svg"

# Cleanup
cd "$OLDPWD"
rm -rf "$TEMP_DIR"
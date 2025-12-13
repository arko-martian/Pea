#!/bin/bash
# Store benchmark results with historical tracking

set -e

RESULTS_DIR="benchmark-results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")

echo "📊 Storing benchmark results..."

# Create results directory if it doesn't exist
mkdir -p "$RESULTS_DIR"

# Create metadata file
cat > "$RESULTS_DIR/metadata_$TIMESTAMP.json" << EOF
{
  "timestamp": "$TIMESTAMP",
  "date": "$(date -Iseconds)",
  "git_commit": "$GIT_COMMIT",
  "git_branch": "$GIT_BRANCH",
  "system": {
    "os": "$(uname -s)",
    "arch": "$(uname -m)",
    "kernel": "$(uname -r)",
    "hostname": "$(hostname)"
  },
  "rust_version": "$(rustc --version 2>/dev/null || echo 'unknown')",
  "cargo_version": "$(cargo --version 2>/dev/null || echo 'unknown')"
}
EOF

# Store criterion benchmark results if they exist
if [ -d "target/criterion" ]; then
    echo "📈 Storing criterion benchmark results..."
    
    # Create criterion results directory
    mkdir -p "$RESULTS_DIR/criterion_$TIMESTAMP"
    
    # Copy criterion results
    cp -r target/criterion/* "$RESULTS_DIR/criterion_$TIMESTAMP/"
    
    # Create criterion summary
    find target/criterion -name "estimates.json" | while read -r file; do
        benchmark_name=$(echo "$file" | sed 's|target/criterion/||' | sed 's|/estimates.json||')
        echo "Processing $benchmark_name..."
        
        # Extract key metrics
        jq -n \
            --arg name "$benchmark_name" \
            --arg timestamp "$TIMESTAMP" \
            --argjson estimates "$(cat "$file")" \
            '{
                benchmark: $name,
                timestamp: $timestamp,
                mean_ns: $estimates.mean.point_estimate,
                std_dev_ns: $estimates.std_dev.point_estimate,
                median_ns: $estimates.median.point_estimate,
                mad_ns: $estimates.median_abs_dev.point_estimate
            }' >> "$RESULTS_DIR/criterion_summary_$TIMESTAMP.json"
    done
fi

# Store hyperfine results if they exist
for file in *-benchmark.json; do
    if [ -f "$file" ]; then
        echo "📊 Storing hyperfine result: $file"
        cp "$file" "$RESULTS_DIR/${file%.json}_$TIMESTAMP.json"
    fi
done

# Create historical summary
echo "📚 Creating historical summary..."

cat > "$RESULTS_DIR/summary_$TIMESTAMP.json" << EOF
{
  "timestamp": "$TIMESTAMP",
  "date": "$(date -Iseconds)",
  "git_commit": "$GIT_COMMIT",
  "git_branch": "$GIT_BRANCH",
  "results": {
EOF

# Add criterion results to summary
if [ -f "$RESULTS_DIR/criterion_summary_$TIMESTAMP.json" ]; then
    echo '    "criterion": [' >> "$RESULTS_DIR/summary_$TIMESTAMP.json"
    
    # Read each line and add to summary
    first=true
    while IFS= read -r line; do
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$RESULTS_DIR/summary_$TIMESTAMP.json"
        fi
        echo "      $line" >> "$RESULTS_DIR/summary_$TIMESTAMP.json"
    done < "$RESULTS_DIR/criterion_summary_$TIMESTAMP.json"
    
    echo '    ],' >> "$RESULTS_DIR/summary_$TIMESTAMP.json"
fi

# Add hyperfine results to summary
echo '    "hyperfine": {' >> "$RESULTS_DIR/summary_$TIMESTAMP.json"

first=true
for file in "$RESULTS_DIR"/*-benchmark_$TIMESTAMP.json; do
    if [ -f "$file" ]; then
        benchmark_type=$(basename "$file" | sed "s/_$TIMESTAMP.json$//" | sed 's/-benchmark$//')
        
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$RESULTS_DIR/summary_$TIMESTAMP.json"
        fi
        
        echo "      \"$benchmark_type\": $(cat "$file")" >> "$RESULTS_DIR/summary_$TIMESTAMP.json"
    fi
done

echo '    }' >> "$RESULTS_DIR/summary_$TIMESTAMP.json"
echo '  }' >> "$RESULTS_DIR/summary_$TIMESTAMP.json"
echo '}' >> "$RESULTS_DIR/summary_$TIMESTAMP.json"

echo "✅ Benchmark results stored!"
echo ""
echo "📁 Results directory: $RESULTS_DIR"
echo "📊 Summary file: $RESULTS_DIR/summary_$TIMESTAMP.json"
echo "📈 Metadata file: $RESULTS_DIR/metadata_$TIMESTAMP.json"

# Create or update latest symlinks
ln -sf "summary_$TIMESTAMP.json" "$RESULTS_DIR/latest_summary.json"
ln -sf "metadata_$TIMESTAMP.json" "$RESULTS_DIR/latest_metadata.json"

echo "🔗 Latest results linked to:"
echo "   - $RESULTS_DIR/latest_summary.json"
echo "   - $RESULTS_DIR/latest_metadata.json"
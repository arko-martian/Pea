#!/bin/bash
# Generate trend reports from historical benchmark data

set -e

RESULTS_DIR="benchmark-results"

echo "📈 Generating benchmark trend report..."

if [ ! -d "$RESULTS_DIR" ]; then
    echo "❌ No benchmark results directory found. Run benchmarks first."
    exit 1
fi

# Check if we have any summary files
SUMMARY_FILES=("$RESULTS_DIR"/summary_*.json)
if [ ! -f "${SUMMARY_FILES[0]}" ]; then
    echo "❌ No benchmark summary files found. Run benchmarks first."
    exit 1
fi

REPORT_FILE="benchmark-trend-report.md"

echo "📊 Creating trend report: $REPORT_FILE"

# Start the report
cat > "$REPORT_FILE" << 'EOF'
# Pea Package Manager - Benchmark Trend Report

This report shows performance trends over time for the Pea package manager.

## Summary

EOF

# Get the latest results
LATEST_SUMMARY=$(ls -t "$RESULTS_DIR"/summary_*.json | head -1)
LATEST_METADATA=$(ls -t "$RESULTS_DIR"/metadata_*.json | head -1)

if [ -f "$LATEST_SUMMARY" ] && [ -f "$LATEST_METADATA" ]; then
    echo "### Latest Results" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # Extract metadata
    LATEST_DATE=$(jq -r '.date' "$LATEST_METADATA")
    LATEST_COMMIT=$(jq -r '.git_commit' "$LATEST_METADATA")
    LATEST_BRANCH=$(jq -r '.git_branch' "$LATEST_METADATA")
    SYSTEM_OS=$(jq -r '.system.os' "$LATEST_METADATA")
    SYSTEM_ARCH=$(jq -r '.system.arch' "$LATEST_METADATA")
    
    echo "- **Date**: $LATEST_DATE" >> "$REPORT_FILE"
    echo "- **Git Commit**: $LATEST_COMMIT" >> "$REPORT_FILE"
    echo "- **Git Branch**: $LATEST_BRANCH" >> "$REPORT_FILE"
    echo "- **System**: $SYSTEM_OS $SYSTEM_ARCH" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi

# Add criterion benchmark trends
echo "## Criterion Benchmark Trends" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Create a table of recent criterion results
echo "| Benchmark | Mean (ns) | Std Dev (ns) | Date | Commit |" >> "$REPORT_FILE"
echo "|-----------|-----------|--------------|------|--------|" >> "$REPORT_FILE"

# Process the last 10 summary files
ls -t "$RESULTS_DIR"/summary_*.json | head -10 | while read -r summary_file; do
    timestamp=$(basename "$summary_file" | sed 's/summary_//' | sed 's/.json//')
    metadata_file="$RESULTS_DIR/metadata_$timestamp.json"
    
    if [ -f "$metadata_file" ]; then
        date=$(jq -r '.date' "$metadata_file" | cut -d'T' -f1)
        commit=$(jq -r '.git_commit' "$metadata_file")
        
        # Extract criterion results if they exist
        if jq -e '.results.criterion' "$summary_file" > /dev/null 2>&1; then
            jq -r '.results.criterion[]? | "\(.benchmark) | \(.mean_ns) | \(.std_dev_ns) | '"$date"' | '"$commit"'"' "$summary_file" >> "$REPORT_FILE"
        fi
    fi
done

# Add hyperfine benchmark trends
echo "" >> "$REPORT_FILE"
echo "## Hyperfine Benchmark Trends" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Process hyperfine results
for benchmark_type in "cold-start" "resolve" "install"; do
    echo "### $benchmark_type Benchmark" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "| Command | Mean (s) | Min (s) | Max (s) | Date | Commit |" >> "$REPORT_FILE"
    echo "|---------|----------|---------|---------|------|--------|" >> "$REPORT_FILE"
    
    # Process recent results for this benchmark type
    ls -t "$RESULTS_DIR"/summary_*.json | head -5 | while read -r summary_file; do
        timestamp=$(basename "$summary_file" | sed 's/summary_//' | sed 's/.json//')
        metadata_file="$RESULTS_DIR/metadata_$timestamp.json"
        
        if [ -f "$metadata_file" ]; then
            date=$(jq -r '.date' "$metadata_file" | cut -d'T' -f1)
            commit=$(jq -r '.git_commit' "$metadata_file")
            
            # Extract hyperfine results for this benchmark type
            if jq -e ".results.hyperfine.\"$benchmark_type\"" "$summary_file" > /dev/null 2>&1; then
                jq -r ".results.hyperfine.\"$benchmark_type\".results[]? | \"\(.command) | \(.mean) | \(.min) | \(.max) | $date | $commit\"" "$summary_file" >> "$REPORT_FILE"
            fi
        fi
    done
    
    echo "" >> "$REPORT_FILE"
done

# Add performance analysis
echo "## Performance Analysis" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Calculate performance trends if we have multiple data points
SUMMARY_COUNT=$(ls "$RESULTS_DIR"/summary_*.json 2>/dev/null | wc -l)

if [ "$SUMMARY_COUNT" -gt 1 ]; then
    echo "### Trends" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # Get first and last results for comparison
    FIRST_SUMMARY=$(ls -t "$RESULTS_DIR"/summary_*.json | tail -1)
    LAST_SUMMARY=$(ls -t "$RESULTS_DIR"/summary_*.json | head -1)
    
    echo "- **Total benchmark runs**: $SUMMARY_COUNT" >> "$REPORT_FILE"
    echo "- **First benchmark**: $(jq -r '.date' "${FIRST_SUMMARY%summary*}metadata${FIRST_SUMMARY#*summary}" 2>/dev/null || echo 'Unknown')" >> "$REPORT_FILE"
    echo "- **Latest benchmark**: $(jq -r '.date' "${LAST_SUMMARY%summary*}metadata${LAST_SUMMARY#*summary}" 2>/dev/null || echo 'Unknown')" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
else
    echo "### Single Data Point" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "Only one benchmark run available. Run more benchmarks to see trends." >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi

# Add targets and goals
echo "## Performance Targets" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "- **Cold start**: <5ms" >> "$REPORT_FILE"
echo "- **Resolution**: Competitive with fastest package manager" >> "$REPORT_FILE"
echo "- **Installation**: Competitive with fastest package manager" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "*Report generated on $(date)*" >> "$REPORT_FILE"

echo "✅ Trend report generated: $REPORT_FILE"
echo ""
echo "📊 Summary:"
echo "  - Processed $SUMMARY_COUNT benchmark runs"
echo "  - Report includes criterion and hyperfine results"
echo "  - Trends and performance analysis included"
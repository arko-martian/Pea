#!/bin/bash
# Detect performance regressions by comparing against baseline

set -e

RESULTS_DIR="benchmark-results"
REGRESSION_THRESHOLD=10  # 10% regression threshold
BASELINE_FILE=""
CURRENT_FILE=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --baseline)
            BASELINE_FILE="$2"
            shift 2
            ;;
        --current)
            CURRENT_FILE="$2"
            shift 2
            ;;
        --threshold)
            REGRESSION_THRESHOLD="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--baseline FILE] [--current FILE] [--threshold PERCENT]"
            echo ""
            echo "Options:"
            echo "  --baseline FILE    Baseline benchmark results (default: auto-detect)"
            echo "  --current FILE     Current benchmark results (default: latest)"
            echo "  --threshold PERCENT Regression threshold percentage (default: 10)"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "🔍 Detecting performance regressions..."
echo "📊 Regression threshold: ${REGRESSION_THRESHOLD}%"

# Auto-detect files if not specified
if [ -z "$CURRENT_FILE" ]; then
    CURRENT_FILE=$(ls -t "$RESULTS_DIR"/summary_*.json 2>/dev/null | head -1)
    if [ -z "$CURRENT_FILE" ]; then
        echo "❌ No current benchmark results found. Run benchmarks first."
        exit 1
    fi
    echo "📈 Current results: $(basename "$CURRENT_FILE")"
fi

if [ -z "$BASELINE_FILE" ]; then
    # Use the second most recent file as baseline
    BASELINE_FILE=$(ls -t "$RESULTS_DIR"/summary_*.json 2>/dev/null | head -2 | tail -1)
    if [ -z "$BASELINE_FILE" ]; then
        echo "⚠️  No baseline found. Need at least 2 benchmark runs for regression detection."
        echo "   Current results will be used as future baseline."
        exit 0
    fi
    echo "📊 Baseline results: $(basename "$BASELINE_FILE")"
fi

# Verify files exist
if [ ! -f "$CURRENT_FILE" ]; then
    echo "❌ Current file not found: $CURRENT_FILE"
    exit 1
fi

if [ ! -f "$BASELINE_FILE" ]; then
    echo "❌ Baseline file not found: $BASELINE_FILE"
    exit 1
fi

# Create regression report
REPORT_FILE="regression-report.md"
REGRESSION_FOUND=false

echo "📝 Creating regression report: $REPORT_FILE"

cat > "$REPORT_FILE" << 'EOF'
# Performance Regression Report

This report compares current benchmark results against a baseline to detect performance regressions.

EOF

# Add metadata
CURRENT_TIMESTAMP=$(basename "$CURRENT_FILE" | sed 's/summary_//' | sed 's/.json//')
BASELINE_TIMESTAMP=$(basename "$BASELINE_FILE" | sed 's/summary_//' | sed 's/.json//')

CURRENT_METADATA="$RESULTS_DIR/metadata_$CURRENT_TIMESTAMP.json"
BASELINE_METADATA="$RESULTS_DIR/metadata_$BASELINE_TIMESTAMP.json"

if [ -f "$CURRENT_METADATA" ] && [ -f "$BASELINE_METADATA" ]; then
    echo "## Comparison Details" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "| Metric | Baseline | Current |" >> "$REPORT_FILE"
    echo "|--------|----------|---------|" >> "$REPORT_FILE"
    echo "| Date | $(jq -r '.date' "$BASELINE_METADATA" | cut -d'T' -f1) | $(jq -r '.date' "$CURRENT_METADATA" | cut -d'T' -f1) |" >> "$REPORT_FILE"
    echo "| Commit | $(jq -r '.git_commit' "$BASELINE_METADATA") | $(jq -r '.git_commit' "$CURRENT_METADATA") |" >> "$REPORT_FILE"
    echo "| Branch | $(jq -r '.git_branch' "$BASELINE_METADATA") | $(jq -r '.git_branch' "$CURRENT_METADATA") |" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi

echo "## Regression Analysis" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "**Threshold**: ${REGRESSION_THRESHOLD}% performance degradation" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Function to calculate percentage change
calculate_change() {
    local baseline=$1
    local current=$2
    
    if [ "$baseline" = "0" ] || [ "$baseline" = "null" ]; then
        echo "N/A"
        return
    fi
    
    # Use awk for floating point arithmetic
    awk "BEGIN { 
        change = (($current - $baseline) / $baseline) * 100
        printf \"%.2f\", change
    }"
}

# Function to check if regression exceeds threshold
is_regression() {
    local change=$1
    
    if [ "$change" = "N/A" ]; then
        return 1
    fi
    
    # Use awk to compare floating point numbers
    awk "BEGIN { exit ($change > $REGRESSION_THRESHOLD) ? 0 : 1 }"
}

# Analyze criterion benchmarks
echo "### Criterion Benchmarks" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if jq -e '.results.criterion' "$BASELINE_FILE" > /dev/null 2>&1 && jq -e '.results.criterion' "$CURRENT_FILE" > /dev/null 2>&1; then
    echo "| Benchmark | Baseline (ns) | Current (ns) | Change (%) | Status |" >> "$REPORT_FILE"
    echo "|-----------|---------------|--------------|------------|--------|" >> "$REPORT_FILE"
    
    # Get all benchmark names from both files
    BENCHMARK_NAMES=$(jq -r '.results.criterion[]?.benchmark' "$BASELINE_FILE" "$CURRENT_FILE" | sort -u)
    
    while IFS= read -r benchmark_name; do
        if [ -n "$benchmark_name" ]; then
            baseline_mean=$(jq -r ".results.criterion[]? | select(.benchmark == \"$benchmark_name\") | .mean_ns" "$BASELINE_FILE")
            current_mean=$(jq -r ".results.criterion[]? | select(.benchmark == \"$benchmark_name\") | .mean_ns" "$CURRENT_FILE")
            
            if [ "$baseline_mean" != "null" ] && [ "$current_mean" != "null" ]; then
                change=$(calculate_change "$baseline_mean" "$current_mean")
                
                if is_regression "$change"; then
                    status="🔴 REGRESSION"
                    REGRESSION_FOUND=true
                elif awk "BEGIN { exit ($change < -5) ? 0 : 1 }" 2>/dev/null; then
                    status="🟢 IMPROVEMENT"
                else
                    status="✅ OK"
                fi
                
                echo "| $benchmark_name | $baseline_mean | $current_mean | $change | $status |" >> "$REPORT_FILE"
            fi
        fi
    done <<< "$BENCHMARK_NAMES"
else
    echo "No criterion benchmark data available for comparison." >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"

# Analyze hyperfine benchmarks
echo "### Hyperfine Benchmarks" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

for benchmark_type in "cold-start" "resolve" "install"; do
    if jq -e ".results.hyperfine.\"$benchmark_type\"" "$BASELINE_FILE" > /dev/null 2>&1 && jq -e ".results.hyperfine.\"$benchmark_type\"" "$CURRENT_FILE" > /dev/null 2>&1; then
        echo "#### $benchmark_type" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "| Command | Baseline (s) | Current (s) | Change (%) | Status |" >> "$REPORT_FILE"
        echo "|---------|--------------|-------------|------------|--------|" >> "$REPORT_FILE"
        
        # Get all commands from both files
        COMMANDS=$(jq -r ".results.hyperfine.\"$benchmark_type\".results[]?.command" "$BASELINE_FILE" "$CURRENT_FILE" | sort -u)
        
        while IFS= read -r command; do
            if [ -n "$command" ]; then
                baseline_mean=$(jq -r ".results.hyperfine.\"$benchmark_type\".results[]? | select(.command == \"$command\") | .mean" "$BASELINE_FILE")
                current_mean=$(jq -r ".results.hyperfine.\"$benchmark_type\".results[]? | select(.command == \"$command\") | .mean" "$CURRENT_FILE")
                
                if [ "$baseline_mean" != "null" ] && [ "$current_mean" != "null" ]; then
                    change=$(calculate_change "$baseline_mean" "$current_mean")
                    
                    if is_regression "$change"; then
                        status="🔴 REGRESSION"
                        REGRESSION_FOUND=true
                    elif awk "BEGIN { exit ($change < -5) ? 0 : 1 }" 2>/dev/null; then
                        status="🟢 IMPROVEMENT"
                    else
                        status="✅ OK"
                    fi
                    
                    echo "| $command | $baseline_mean | $current_mean | $change | $status |" >> "$REPORT_FILE"
                fi
            fi
        done <<< "$COMMANDS"
        
        echo "" >> "$REPORT_FILE"
    fi
done

# Add summary
echo "## Summary" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if [ "$REGRESSION_FOUND" = true ]; then
    echo "🔴 **REGRESSIONS DETECTED** - Performance degradation exceeds ${REGRESSION_THRESHOLD}% threshold." >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "**Action Required**: Investigate and fix performance regressions before merging." >> "$REPORT_FILE"
else
    echo "✅ **NO REGRESSIONS** - All benchmarks are within acceptable performance range." >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "*Report generated on $(date)*" >> "$REPORT_FILE"

# Output results
echo ""
if [ "$REGRESSION_FOUND" = true ]; then
    echo "🔴 REGRESSIONS DETECTED!"
    echo "   Performance degradation exceeds ${REGRESSION_THRESHOLD}% threshold."
    echo "   See $REPORT_FILE for details."
    exit 1
else
    echo "✅ NO REGRESSIONS DETECTED"
    echo "   All benchmarks are within acceptable performance range."
    echo "   Report saved to: $REPORT_FILE"
    exit 0
fi
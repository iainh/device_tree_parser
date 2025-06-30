#!/bin/bash
# ABOUTME: Convenient benchmark runner script for device tree parser
# ABOUTME: Provides shortcuts for common benchmark operations

set -e

echo "🚀 Device Tree Parser Benchmarks"
echo "================================"

case "${1:-all}" in
    "all")
        echo "Running all benchmarks..."
        cargo bench
        ;;
    "quick")
        echo "Running quick benchmarks..."
        cargo bench -- --quick
        ;;
    "parsing")
        echo "Running core parsing benchmarks..."
        cargo bench parse_header parse_memory_reservations parse_tree
        ;;
    "properties")
        echo "Running property access benchmarks..."
        cargo bench property_access
        ;;
    "api")
        echo "Running high-level API benchmarks..."
        cargo bench high_level_api
        ;;
    "pipeline")
        echo "Running full pipeline benchmark..."
        cargo bench full_parsing_pipeline
        ;;
    "scaling")
        echo "Running data size scaling benchmarks..."
        cargo bench data_size_scaling
        ;;
    "baseline")
        echo "Saving baseline benchmark results..."
        mkdir -p benchmark_results
        cargo bench > "benchmark_results/baseline_$(date +%Y%m%d_%H%M%S).txt"
        echo "Baseline saved to benchmark_results/"
        ;;
    "compare")
        echo "Running comparison benchmarks..."
        cargo bench > "benchmark_results/comparison_$(date +%Y%m%d_%H%M%S).txt"
        echo "Results saved to benchmark_results/"
        ;;
    "report")
        echo "Opening HTML benchmark reports..."
        if [ -f "target/criterion/report/index.html" ]; then
            if command -v open >/dev/null; then
                open target/criterion/report/index.html
            elif command -v xdg-open >/dev/null; then
                xdg-open target/criterion/report/index.html
            else
                echo "HTML report available at: target/criterion/report/index.html"
            fi
        else
            echo "No HTML report found. Run benchmarks first."
        fi
        ;;
    "clean")
        echo "Cleaning benchmark data..."
        rm -rf target/criterion
        rm -rf benchmark_results
        echo "Benchmark data cleaned."
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  all        Run all benchmarks (default)"
        echo "  quick      Run benchmarks with quick measurement"
        echo "  parsing    Run core parsing benchmarks only"
        echo "  properties Run property access benchmarks only"
        echo "  api        Run high-level API benchmarks only"
        echo "  pipeline   Run full pipeline benchmark only"
        echo "  scaling    Run data size scaling benchmarks only"
        echo "  baseline   Save current results as baseline"
        echo "  compare    Run and save comparison results"
        echo "  report     Open HTML benchmark reports"
        echo "  clean      Clean benchmark data"
        echo "  help       Show this help message"
        ;;
    *)
        echo "Unknown command: $1"
        echo "Use '$0 help' for available commands."
        exit 1
        ;;
esac

echo "✅ Done!"
#!/usr/bin/env python3
import os
import json
import subprocess
import time
import sys
from statistics import mean
import argparse

def run_test(file_path, timeout):
    start_time = time.time()
    try:
        result = subprocess.run(['cargo', 'run'], input=open(file_path, 'r').read(),
                                capture_output=True, text=True, timeout=timeout)
        end_time = time.time()
        return result.stdout.strip(), end_time - start_time, False
    except subprocess.TimeoutExpired:
        return "TIMEOUT", timeout, True

def process_directory(directory, timeout):
    results = {}
    file_count = 0
    timeout_count = 0
    for root, dirs, files in os.walk(directory):
        for file in files:
            file_path = os.path.join(root, file)
            output, duration, timed_out = run_test(file_path, timeout)

            # Print a period to indicate progress
            print('.', end='', flush=True)
            file_count += 1

            # Determine the category (satisfiable or unsatisfiable)
            category = 'unsatisfiable' if 'unsatisfiable' in root else 'satisfiable'

            # Determine the number of clauses from the directory name
            clauses = root.split(os.path.sep)[-2] if category in root else root.split(os.path.sep)[-1]

            # Check if the output is correct
            expected_output = 'IS SAT' if category == 'satisfiable' else 'UNSAT'
            is_correct = expected_output in output

            # Log incorrect files and timeouts to stderr
            if timed_out:
                print(f"\nTimeout for file: {file_path}", file=sys.stderr)
                timeout_count += 1
            elif not is_correct:
                print(f"\nIncorrect output for file: {file_path}", file=sys.stderr)
                print(f"Expected: {expected_output}, Got: {output}", file=sys.stderr)

            # Store the result
            if clauses not in results:
                results[clauses] = {}
            if category not in results[clauses]:
                results[clauses][category] = {'times': [], 'correct': 0, 'total': 0, 'timeouts': 0}

            results[clauses][category]['times'].append(duration)
            results[clauses][category]['total'] += 1
            if timed_out:
                results[clauses][category]['timeouts'] += 1
            elif is_correct:
                results[clauses][category]['correct'] += 1

    # Print a newline after all files have been processed
    print(f"\nProcessed {file_count} files. {timeout_count} timeouts occurred.")

    # Calculate averages and format results
    formatted_results = {}
    for clauses, categories in results.items():
        formatted_results[clauses] = {}
        for category, data in categories.items():
            avg_time = mean(data['times'])
            accuracy = data['correct'] / (data['total'] - data['timeouts']) * 100 if data['total'] > data['timeouts'] else 0
            formatted_results[clauses][category] = {
                'average_time': avg_time,
                'accuracy': accuracy,
                'total_files': data['total'],
                'correct_files': data['correct'],
                'timeout_files': data['timeouts']
            }

    return formatted_results

def main():
    parser = argparse.ArgumentParser(description="Run SAT solver benchmarks with configurable timeout.")
    parser.add_argument('--timeout', type=float, default=60, help="Timeout in seconds for each test case (default: 60)")
    args = parser.parse_args()

    benchmark_dir = 'benchmarks'
    results = process_directory(benchmark_dir, args.timeout)

    with open('benchmark_results.json', 'w') as f:
        json.dump(results, f, indent=2)

    print("Benchmark results have been written to benchmark_results.json")

if __name__ == "__main__":
    main()

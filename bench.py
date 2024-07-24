#!/usr/bin/env python3
import os
import json
import subprocess
import time
import sys
from statistics import mean

def run_test(file_path):
    start_time = time.time()
    with open(file_path) as f:
        data = f.read()
    result = subprocess.run(['cargo', 'run', '--release'],
        input=data,
        capture_output=True,
        text=True)
    end_time = time.time()
    return result.stdout.strip(), end_time - start_time

def process_directory(directory):
    results = {}
    file_count = 0
    for root, dirs, files in os.walk(directory):
        for file in files:
            file_path = os.path.join(root, file)
            output, duration = run_test(file_path)

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

            # Log incorrect files to stderr
            if not is_correct:
                print(f"\nIncorrect output for file: {file_path}", file=sys.stderr)
                print(f"Expected: {expected_output}, Got: {output}", file=sys.stderr)

            # Store the result
            if clauses not in results:
                results[clauses] = {}
            if category not in results[clauses]:
                results[clauses][category] = {'times': [], 'correct': 0, 'total': 0}

            results[clauses][category]['times'].append(duration)
            results[clauses][category]['total'] += 1
            if is_correct:
                results[clauses][category]['correct'] += 1

    # Print a newline after all files have been processed
    print(f"\nProcessed {file_count} files.")

    # Calculate averages and format results
    formatted_results = {}
    for clauses, categories in results.items():
        formatted_results[clauses] = {}
        for category, data in categories.items():
            avg_time = mean(data['times'])
            accuracy = data['correct'] / data['total'] * 100
            formatted_results[clauses][category] = {
                'average_time': avg_time,
                'accuracy': accuracy
            }

    return formatted_results

def main():
    benchmark_dir = 'benchmarks'
    results = process_directory(benchmark_dir)

    with open('benchmark_results.json', 'w') as f:
        json.dump(results, f, indent=2)

    print("Benchmark results have been written to benchmark_results.json")

if __name__ == "__main__":
    main()

#!/usr/bin/env python3

import os
import time
import json
import subprocess
import statistics
import glob

class TimingResult:
    def __init__(self, average, median, all_outputs_valid):
        self.average = average
        self.median = median
        self.all_outputs_valid = all_outputs_valid

    def to_json(self):
        return json.dumps({
            "average": self.average,
            "median": self.median,
            "all_outputs_valid": self.all_outputs_valid
        })

def run_timing(folder):
    times = []
    all_outputs_valid = True

    for file in glob.glob(f'false/{folder}/**/*.cnf', recursive=True):
        start_time = time.time()

        with open(file, 'r') as f:
            result = subprocess.run(['cargo', 'run', '--release'], stdin=f, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)

        end_time = time.time()
        execution_time = end_time - start_time
        times.append(execution_time)

        # Check if the output contains "UNSAT"
        if "UNSAT" not in result.stdout:
            all_outputs_valid = False
            print(f"\nWarning: Output for {file} does not contain 'UNSAT'")

        print(".", end="", flush=True)

    # Calculate average
    average = statistics.mean(times)

    # Calculate median
    median = statistics.median(times)

    print(f"\nAverage execution time: {average:.6f} seconds")
    print(f"Median execution time: {median:.6f} seconds")
    print(f"All outputs contain 'UNSAT': {all_outputs_valid}")

    return TimingResult(average, median, all_outputs_valid)


folders = [753, 218, 645, 430, 860, 325, 538, 960, 1065]
folders.sort()


# Run timing for each folder and store results
results = {}
for folder in folders:
    print(f"\nRunning timing for folder: {folder}")
    results[folder] = run_timing(folder)

# Serialize results to disk
with open('timing_results.json', 'w') as f:
    json.dump({k: v.__dict__ for k, v in results.items()}, f, indent=2)

print("\nResults have been saved to timing_results.json")

#!/usr/bin/env python3

import os
import time
import subprocess
import statistics
import glob

times = []

for file in glob.glob('false/**/*.cnf', recursive=True):
    start_time = time.time()

    with open(file, 'r') as f:
        subprocess.run(['cargo', 'run', '-r'], stdin=f, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    end_time = time.time()
    execution_time = end_time - start_time
    times.append(execution_time)
    print(".", end="", flush=True)

# Calculate average
average = statistics.mean(times)

# Calculate median
median = statistics.median(times)

print(f"\nAverage execution time: {average:.6f} seconds")
print(f"Median execution time: {median:.6f} seconds")

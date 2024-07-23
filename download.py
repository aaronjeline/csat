import json
import os
import requests
import tarfile
from urllib.parse import urljoin
import tempfile
import shutil

def extract_flat(tar, path):
    for member in tar.getmembers():
        if member.isreg():  # regular file
            member.name = os.path.basename(member.name)
            tar.extract(member, path)

def download_and_extract(url, output_dir, category):
    response = requests.get(url)
    if response.status_code == 200:
        with tempfile.NamedTemporaryFile(delete=False) as temp_file:
            temp_file.write(response.content)
            temp_file_path = temp_file.name

        with tarfile.open(temp_file_path, 'r:gz') as tar:
            if category == 'unsatisfiable':
                extract_flat(tar, output_dir)
            else:
                tar.extractall(path=output_dir)
        
        os.unlink(temp_file_path)
    else:
        print(f"Failed to download {url}")

def process_benchmarks(json_file):
    with open(json_file, 'r') as f:
        data = json.load(f)
    
    base_url = data['base_url']
    benchmarks = data['uniform_random_3sat']
    
    for benchmark in benchmarks:
        clauses = benchmark['clauses']
        output_dir = f"benchmarks/{clauses}_clauses"
        os.makedirs(output_dir, exist_ok=True)
        
        if 'link' in benchmark:
            url = urljoin(base_url, benchmark['link'])
            download_and_extract(url, output_dir, 'satisfiable')
        elif 'links' in benchmark:
            for category, link in benchmark['links'].items():
                url = urljoin(base_url, link)
                category_dir = os.path.join(output_dir, category)
                os.makedirs(category_dir, exist_ok=True)
                download_and_extract(url, category_dir, category)

if __name__ == "__main__":
    process_benchmarks('benchmarks.json')
    print("Download and extraction complete.")

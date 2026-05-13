import glob
import os
import re

search_pattern = r'let diesel_pools = apiserver::diesel_pool::DieselPools::for_test\(\s*&std::env::var\("DATABASE_URL"\)\.unwrap_or_else\(\|_\| \{(?:.|\n)*?\}\),\s*\);'
replacement = 'let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());'

search_pattern2 = r'let diesel_pools = apiserver::diesel_pool::DieselPools::for_test\(\s*&std::env::var\("DATABASE_URL"\)\.unwrap_or_else\(\|_\| "[^"]*"\.to_string\(\)\)\s*\);'
replacement2 = 'let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());'

files = glob.glob('apiserver/tests/integration/**/*.rs', recursive=True)

for file in files:
    with open(file, 'r') as f:
        content = f.read()
    
    new_content = re.sub(search_pattern, replacement, content)
    new_content = re.sub(search_pattern2, replacement2, new_content)

    # Some of them might not use unwrap_or_else inside a block, like .unwrap_or_else(|_| "postgres...".to_string())
    
    if new_content != content:
        with open(file, 'w') as f:
            f.write(new_content)
        print(f"Updated {file}")


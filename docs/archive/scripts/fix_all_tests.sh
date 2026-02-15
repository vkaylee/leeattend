#!/bin/bash

# This script fixes all test files to include storage fields

echo "Fixing test files..."

for file in tests/*.rs; do
    # Check if file has AppState but is malformed
    if grep -q "coordinator: None.*storage" "$file"; then
        # Fix malformed coordinator line
        sed -i 's/coordinator: None.*storage_provider.*}/coordinator: None,\n        storage,\n        storage_provider,\n    }/' "$file"
    fi
    
    # If coordinator: None, exists but no storage fields
    if grep -q "coordinator: None," "$file" && ! grep -A1 "coordinator: None," "$file" | grep -q "storage,"; then
        sed -i '/coordinator: None,/a\        storage,\n        storage_provider,' "$file"
    fi
    
    # If AppState exists but no storage setup
    if grep -q "Arc::new(.*AppState {" "$file" && ! grep -B5 "Arc::new.*AppState" "$file" | grep -q "let storage ="; then
        # Add storage setup before Arc::new(AppState
        perl -i -pe 's/(    )(let \w+ = Arc::new\((?:apiserver::)?AppState \{)/$1\/\/ Setup test storage\n$1let storage = apiserver::StorageBackend::Local {\n$1    base_path: std::path::PathBuf::from(".\/test_storage"),\n$1};\n$1let storage_provider = apiserver::services::create_storage_provider(\&storage)\n$1    .await\n$1    .expect("Failed to create test storage");\n\n$1$2/' "$file"
    fi
done

echo "Done! Now run: cargo check --tests"

#!/bin/bash

set -e

# Get the current version from package.json
VERSION=$(jq -r '.version' package.json)
echo "Checking version: $VERSION"

# Check if this version already exists on crates.io
API_RESPONSE=$(curl -s -f https://crates.io/api/v1/crates/takumi/versions)

# Check if curl failed
if [ $? -ne 0 ]; then
    echo "Failed to fetch version data from crates.io"
    exit 1
fi

echo "API Response received, checking for version..."

# Check if the response contains errors
if echo "$API_RESPONSE" | jq -e '.errors' > /dev/null 2>&1; then
    echo "API returned errors, proceeding with publish..."
    cargo publish
    exit 0
fi

# Check if this version already exists
if echo "$API_RESPONSE" | jq -e ".versions[] | select(.num == \"$VERSION\")" > /dev/null 2>&1; then
    echo "Version $VERSION already exists on crates.io. Skipping publish."
else
    echo "Version $VERSION not found. Proceeding with publish..."
    cargo publish
fi

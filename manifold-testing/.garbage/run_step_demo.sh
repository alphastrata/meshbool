#!/bin/bash
# Script to run the step boolean demo with a chosen STEP file

set -e

# Check if STEP file argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <step_file.step>"
    echo "Example: $0 assets/LN_032.step"
    exit 1
fi

STEP_FILE="$1"

# Check if the STEP file exists
if [ ! -f "$STEP_FILE" ]; then
    echo "Error: STEP file '$STEP_FILE' not found"
    exit 1
fi

# Copy the STEP file to the correct assets directory
echo "Copying STEP file to assets directory..."
mkdir -p assets/real_parts/
cp "$STEP_FILE" assets/real_parts/

# Extract just the filename for the CLI argument
FILENAME=$(basename "$STEP_FILE")

echo "Running step_boolean_demo with $FILENAME..."
cd /home/midget/Documents/manifold-testing
PATH="$HOME/.cargo/bin:$PATH" timeout 30s cargo run --example step_boolean_demo -- --step-file "real_parts/$FILENAME"
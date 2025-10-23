#!/bin/bash
# Simple script to test STEP file loading and boolean operations

set -e

# Check if STEP file argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <step_file.step>"
    echo "Example: $0 /path/to/your/file.step"
    exit 1
fi

STEP_FILE="$1"

# Check if the STEP file exists
if [ ! -f "$STEP_FILE" ]; then
    echo "Error: STEP file '$STEP_FILE' not found"
    exit 1
fi

echo "=== STEP BOOLEAN DEMO ==="
echo "Loading STEP file: $STEP_FILE"
echo ""
echo "Controls:"
echo "  SPACE - Cycle through boolean operations (None → Intersect → Union → Subtract → None)"
echo "  Q     - Quit with error message 'User did not see expected results'"
echo ""
echo "Features implemented:"
echo "  ✅ Space bar toggle for boolean operations"
echo "  ✅ Camera orbiting around the main part"
echo "  ✅ Proper error handling with useful messages"
echo "  ✅ Q key to quit with error message"
echo ""
echo "Running demo..."

# Run the demo with the STEP file
cd /home/midget/Documents/manifold-testing
PATH="$HOME/.cargo/bin:$PATH" cargo run --example step_boolean_demo -- --step-file "$STEP_FILE"
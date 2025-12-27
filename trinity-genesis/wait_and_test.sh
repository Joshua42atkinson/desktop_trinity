#!/bin/bash
echo "Waiting for Brain to start..."
timeout=300
elapsed=0
while ! grep -q "Listening on" brain_test.log; do
    sleep 5
    elapsed=$((elapsed+5))
    if [ $elapsed -ge $timeout ]; then
        echo "Timed out waiting for brain"
        exit 1
    fi
    echo "Still waiting... ($elapsed seconds)"
done

echo "Brain is ready!"
echo "Running Autopoietic Test..."
cargo run -p trinity-cli -- test-autopoiesis -f crates/trinity-kernel/src/lib.rs

#!/bin/bash

# Check Backend Build (Phase 1 DB Integration)
echo "Checking Backend Build..."
cargo check -p backend
echo -e "\nBackend Check Complete."

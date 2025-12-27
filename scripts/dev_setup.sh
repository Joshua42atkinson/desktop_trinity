#!/bin/bash
# This script automates the setup of the development environment for the Daydream project.
# It is designed to be idempotent and combat non-persistent environments.

# Exit immediately if a command exits with a non-zero status.
set -e

# Function to print messages
print_message() {
  echo "===================================================================================================="
  echo "$1"
  echo "===================================================================================================="
}

# 1. Install system dependencies
print_message "Installing system dependencies..."
case "$(uname -s)" in
  Linux)
    if [ -x "$(command -v apt-get)" ]; then
      echo "Debian-based system detected. Installing dependencies..."
      sudo apt-get update && sudo apt-get install -y libasound2-dev libudev-dev
    else
      echo "Non-Debian Linux system detected."
      echo "Please install the following dependencies manually:"
      echo " - alsa-lib-devel (or equivalent for libasound2)"
      echo " - libudev-devel (or equivalent)"
    fi
    ;;
  Darwin)
    echo "macOS detected."
    echo "Please ensure you have the command line tools installed ('xcode-select --install')."
    echo "The necessary libraries are typically included with the base system or XCode."
    ;;
  *)
    echo "Unsupported operating system: $(uname -s)"
    echo "Please install the equivalent of 'libasound2' and 'libudev' for your system."
    ;;
esac

# 2. Add wasm32-unknown-unknown target
print_message "Adding wasm32-unknown-unknown Rust target..."
rustup target add wasm32-unknown-unknown --toolchain stable

# 3. Install required cargo tools
print_message "Installing required cargo tools (cargo-binstall, sqlx-cli, cargo-leptos)..."
# Use cargo install if cargo binstall is not found, ensuring persistence
if ! [ -x "$(command -v cargo-binstall)" ]; then
    echo "cargo-binstall not found. Installing via cargo install."
    cargo install cargo-binstall
fi

cargo binstall sqlx-cli -y || cargo install sqlx-cli
cargo binstall cargo-leptos -y || cargo install cargo-leptos

# 4. Create the necessary 'public' directory for Leptos assets
print_message "Creating required 'public' asset directory..."
mkdir -p public
echo "Created directory 'public/'"

# 5. Check for Docker
print_message "Checking for Docker installation..."
if ! [ -x "$(command -v docker)" ]; then
  echo "Docker is not installed. Please install Docker to run the database."
  echo "See https://docs.docker.com/engine/install/ for installation instructions."
  exit 1
fi
echo "Docker is installed."

# 6. Create .env file
print_message "Setting up the .env file..."
if [ -f "backend/.env.example" ]; then
  if [ ! -f "backend/.env" ]; then
    cp backend/.env.example backend/.env
    echo "Created backend/.env from backend/.env.example"
  else
    echo "backend/.env already exists. Skipping creation."
  fi
else
    echo "backend/.env.example not found. Please ensure it exists."
fi

print_message "Development environment setup is complete!"
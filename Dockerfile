# ---- Builder Stage ----
# Build the backend and frontend
FROM rust:1.78 as builder

# Install frontend dependencies
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --git https://github.com/leptos-rs/trunk

# Install backend dependencies
RUN cargo install cargo-binstall
RUN cargo binstall cargo-leptos -y

WORKDIR /app

# Copy the entire project
COPY . .

# Build the backend
RUN cd backend && cargo build --release

# Build the frontend
RUN cd frontend && trunk build --release

# ---- Runtime Stage ----
# Create the final, smaller image
FROM debian:bookworm-slim as runtime

# Set backend environment variables
ENV RUST_LOG="info"
ENV LEPTOS_OUTPUT_NAME="frontend"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_RELOAD_PORT="3001"

WORKDIR /app

# Copy the backend binary from the builder stage
COPY --from=builder /app/target/release/backend .

# Copy the frontend build artifacts from the builder stage
COPY --from=builder /app/frontend/dist ./site

EXPOSE 3000

# Run the backend server
CMD ["/app/backend"]

# syntax=docker/dockerfile:1
#
# Dockerfile.cloudflare - Phase 1: Axum-only Debug
# Minimal version to isolate the Docker exit issue
#
# This Dockerfile builds only the ore-ingest binary with minimal dependencies
# to test if basic Axum web server works in Docker

##########################################
## 1️⃣ Builder Stage (Debian)            ##
##########################################

FROM rust:1-bookworm AS build

# Set build-time arguments
ARG TARGETPLATFORM=linux/amd64

# Install build tools and required dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    libprotobuf-dev \
    libudev-dev \
    zlib1g-dev \
    clang \
    llvm \
    perl \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Set environment variables for static linking
ENV SOLANA_METRICS_CONFIG="host=127.0.0.1" \
    PKG_CONFIG_ALLOW_CROSS=1 \
    RUSTFLAGS="-C target-feature=+crt-static" \
    RUSTC_LINKER=clang

WORKDIR /app

# Copy all Cargo files and source files for caching (Phase 4)
COPY Cargo.toml Cargo.lock ./
COPY ingest/Cargo.toml ./ingest/
COPY api/Cargo.toml ./api/

# Copy source files early for caching
COPY ingest/src ./ingest/src/
COPY api/src ./api/src/

# Add target (Phase 1)
RUN case ${TARGETPLATFORM} in \
    "linux/amd64") RUST_TARGET="x86_64-unknown-linux-gnu" ;; \
    "linux/arm64") RUST_TARGET="aarch64-unknown-linux-gnu" ;; \
    esac && \
    rustup target add ${RUST_TARGET}

# Build packages (cache warming)
RUN case ${TARGETPLATFORM} in \
    "linux/amd64") RUST_TARGET="x86_64-unknown-linux-gnu" ;; \
    "linux/arm64") RUST_TARGET="aarch64-unknown-linux-gnu" ;; \
    esac && \
    cargo build --release --target ${RUST_TARGET} --package ore-ingest --features api

# Final build (Phase 4)
RUN case ${TARGETPLATFORM} in \
    "linux/amd64") RUST_TARGET="x86_64-unknown-linux-gnu" ;; \
    "linux/arm64") RUST_TARGET="aarch64-unknown-linux-gnu" ;; \
    esac && \
    cargo build --release --target ${RUST_TARGET} --package ore-ingest --features api && \
    echo "=== BINARY INFO ===" && \
    ls -la /app/target/${RUST_TARGET}/release/ore-ingest && \
    file /app/target/${RUST_TARGET}/release/ore-ingest

##########################################
## 2️⃣ Runtime Stage (Debian slim)      ##
##########################################

FROM debian:bookworm-slim

# Copy CA certificates for HTTPS
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy compiled binary from the build stage
COPY --from=build /app/target/x86_64-unknown-linux-gnu/release/ore-ingest /ore-ingest

# Copy required system libraries for SSL/crypto dependencies
COPY --from=build /usr/lib/x86_64-linux-gnu/libssl.so.3 /usr/lib/x86_64-linux-gnu/
COPY --from=build /usr/lib/x86_64-linux-gnu/libcrypto.so.3 /usr/lib/x86_64-linux-gnu/

# Expose the service ports
EXPOSE 3000

# Default entrypoint
ENTRYPOINT ["/ore-ingest"]

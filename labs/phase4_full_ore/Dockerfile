# syntax=docker/dockerfile:1
#
# Optimized Dockerfile for Phase 3 - Axum + Turso + Solana
# Based on reev project pattern with platform forcing for x86_64

# This ARG must be declared before the first FROM so it can be used there.
ARG BUILD_PLATFORM=linux/amd64

##########################################
## 1️⃣ Chef Stage (cargo-chef)          ##
##########################################

FROM --platform=${BUILD_PLATFORM} lukemathwalker/cargo-chef:0.1.72-rust-1.88.0-slim-bullseye AS chef

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    libudev-dev \
    zlib1g-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

##########################################
## 2️⃣ Planner Stage                   ##
##########################################

FROM chef AS planner

# Re-declare the ARG for this stage
ARG BUILD_PLATFORM

# Copy all the source files for dependency planning
COPY Cargo.toml ./
COPY src/ ./src/

# Prepare the recipe for building dependencies
RUN cargo chef prepare --recipe-path recipe.json

##########################################
## 3️⃣ Builder Stage                    ##
##########################################

FROM chef AS builder

# Re-declare the ARG for this stage
ARG BUILD_PLATFORM

# Copy the recipe from planner stage
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies using the recipe
RUN PKG_CONFIG_ALLOW_CROSS=1 \
    PROTOC=/usr/bin/protoc \
    RUSTFLAGS="-C target-cpu=generic" \
    cargo chef cook --release --recipe-path recipe.json

# Copy the actual source code
COPY Cargo.toml ./
COPY src/ ./src/

# Build the actual application binary
RUN PKG_CONFIG_ALLOW_CROSS=1 \
    PROTOC=/usr/bin/protoc \
    RUSTFLAGS="-C target-cpu=generic" \
    cargo build --release

##########################################
## 4️⃣ Runtime Stage (minimal, secure) ##
##########################################

FROM --platform=${BUILD_PLATFORM} ubuntu:20.04

# Install runtime dependencies including OpenSSL libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl1.1 \
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

# Create a dedicated non-root user for security
RUN groupadd -r app && \
    useradd -r -u 1000 -g app app

# Set the working directory for the runtime stage
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/phase4-full-ore /app/phase4-server

# Set correct ownership for all application files
RUN chown -R app:app /app

# Expose the service port
EXPOSE 3000

# Health check to ensure the API service is responsive
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s \
    CMD curl -f http://localhost:3000/health || exit 1

# Run as the non-root user
USER app

# Default entrypoint
ENTRYPOINT ["/app/phase4-server"]

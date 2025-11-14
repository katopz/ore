# syntax=docker/dockerfile:1
# Simple Dockerfile for Phase 4 - Axum + Solana + ORE
# Platform forcing for x86_64 architecture

ARG BUILD_PLATFORM=linux/amd64

##########################################
## 1️⃣ Builder Stage                    ##
##########################################

FROM --platform=${BUILD_PLATFORM} ubuntu:20.04 AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    libudev-dev \
    zlib1g-dev \
    curl \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Copy entire project including api directory
COPY . .

# Set environment variables for cross-compilation
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PROTOC=/usr/bin/protoc
ENV RUSTFLAGS="-C target-cpu=generic"

# Build application
RUN cargo build --release

##########################################
## 2️⃣ Runtime Stage ##
##########################################

FROM --platform=${BUILD_PLATFORM} ubuntu:20.04

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl1.1 \
    libudev1 && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r app && \
    useradd -r -u 1000 -g app app

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/phase4-full-ore /app/phase4-server

# Set ownership
RUN chown -R app:app /app

# Expose port
EXPOSE 3005

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s \
    CMD curl -f http://localhost:3005/ || exit 1

# Run as non-root user
USER app

# Default entrypoint
ENTRYPOINT ["/app/phase4-server"]

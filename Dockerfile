# syntax=docker/dockerfile:1
#
# ORE Ingest Dockerfile - Optimized for production
# Uses cargo-chef for optimal caching and cross-platform builds
# Uses Alpine Linux for minimal size and fast builds
# Optimized for ARM Mac builds with proper dependency handling

# This ARG must be declared before the first FROM so it can be used there.
ARG BUILD_PLATFORM=linux/amd64

##########################################
## 1️⃣ Chef Stage (cargo-chef)          ##
##########################################

FROM --platform=${BUILD_PLATFORM} lukemathwalker/cargo-chef:0.1.72-rust-1.88.0-slim-bullseye AS chef

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    git \
    build-essential \
    make \
    cmake \
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

# Copy workspace files for dependency planning
COPY Cargo.toml Cargo.lock ./
COPY ingest/Cargo.toml ./ingest/
COPY api/Cargo.toml ./api/

# Copy source files
COPY ingest/src ./ingest/src/
COPY api/src ./api/src/

# Prepare to recipe for building dependencies
RUN cargo chef prepare --recipe-path recipe.json

##########################################
## 3️⃣ Builder Stage                    ##
##########################################

FROM chef AS builder

# Re-declare the ARG for this stage
ARG BUILD_PLATFORM

# Copy to recipe from planner stage
COPY --from=planner /app/recipe.json recipe.json

# Build to dependencies using to recipe
RUN PKG_CONFIG_ALLOW_CROSS=1 \
    PROTOC=/usr/bin/protoc \
    RUSTFLAGS="-C target-cpu=generic" \
    cargo chef cook --release --recipe-path recipe.json

# Copy to actual source code
COPY Cargo.toml Cargo.lock ./
COPY ingest/Cargo.toml ./ingest/
COPY api/Cargo.toml ./api/
COPY ingest/src ./ingest/src/
COPY api/src ./api/src/

# Build to actual application binary
RUN cargo build --release --package ore-ingest --features api

##########################################
## 4️⃣ Runtime Stage (Alpine, minimal)  ##
##########################################

FROM --platform=${BUILD_PLATFORM} alpine:3.19

# Install only runtime dependencies
RUN apk update && \
    apk add --no-cache \
    ca-certificates \
    curl \
    libgcc \
    && rm -rf /var/cache/apk/*

# Create a dedicated non-root user for security
RUN addgroup -g 1000 app && \
    adduser -D -u 1000 -G app app

# Set working directory
WORKDIR /app

# Copy to compiled binary from the builder stage
COPY --from=builder /app/target/release/ore-ingest /app/ore-ingest

# Set correct ownership for all application files
RUN chown -R app:app /app

# Create data directory for database
RUN mkdir -p /app/data && chown app:app /app/data

# Expose service port
EXPOSE 4000

# Health check to ensure API service is responsive
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s \
    CMD curl -f http://localhost:4000/ || exit 1

# Run as a non-root user
USER app

# Default entrypoint with environment variables
ENV PORT=4000
ENV TURSO_URL=/app/data/ore.db
ENTRYPOINT ["/app/ore-ingest"]

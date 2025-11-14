# syntax=docker/dockerfile:1
# Optimized Dockerfile based on reev approach for Phase 3
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

ARG BUILD_PLATFORM

COPY Cargo.toml Cargo.lock ./
COPY src ./src/

# Prepare recipe for building dependencies
RUN cargo chef prepare --recipe-path recipe.json

##########################################
## 3️⃣ Builder Stage                    ##
##########################################

FROM chef AS builder

ARG BUILD_PLATFORM

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies using recipe
RUN PKG_CONFIG_ALLOW_CROSS=1 \
    PROTOC=/usr/bin/protoc \
    RUSTFLAGS="-C target-cpu=generic" \
    cargo chef cook --release --recipe-path recipe.json

# Copy actual source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src/

# Build actual application binary
RUN PKG_CONFIG_ALLOW_CROSS=1 \
    PROTOC=/usr/bin/protoc \
    RUSTFLAGS="-C target-cpu=generic" \
    cargo build --release

##########################################
## 4️⃣ Runtime Stage                    ##
##########################################

FROM --platform=${BUILD_PLATFORM} ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl1.1 \
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r app && \
    useradd -r -u 1000 -g app app

WORKDIR /app

COPY --from=builder /app/target/release/phase3-axum-turso-solana /app/

RUN chown -R app:app /app

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s \
    CMD curl -f http://localhost:3000/ || exit 1

USER app

ENTRYPOINT ["/app/phase3-axum-turso-solana"]

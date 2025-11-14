# syntax=docker/dockerfile:1
# Build stage - FORCE x86_64 platform
FROM --platform=linux/amd64 ubuntu:20.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update && apt-get install -y curl build-essential pkg-config

# Install Rust
RUN curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

# NATIVE compilation (no cross-compilation needed!)
RUN cargo build --release

# Runtime stage - also x86_64
FROM --platform=linux/amd64 ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y curl
COPY --from=0 /app/target/release/phase2-axum-turso /app/
WORKDIR /app
EXPOSE 3000
CMD ["/app/phase2-axum-turso"]

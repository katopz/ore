#!/bin/bash
# ORE Ubuntu Container Build Script
# Enables macOS users to build in Ubuntu containers without local Docker issues
# Spins up Ubuntu container, builds ORE stack, and outputs results
# Usage: ./build-local.sh [image-name] [command]
# Example: ./build-local.sh ore-ubuntu-build "docker build -t ore-ubuntu ."

set -e

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
CONTAINER_NAME="ore-build-$(date +%s)"
IMAGE_NAME="${1:-ore-ubuntu-build}"
BUILD_COMMAND="${2:-docker build -f docker/Dockerfile.ubuntu -t ore-ubuntu-test .}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up build container..."
    if docker ps -q -f name="${CONTAINER_NAME}" | grep -q .; then
        docker stop "${CONTAINER_NAME}" >/dev/null 2>&1 || true
    fi
    docker rm "${CONTAINER_NAME}" >/dev/null 2>&1 || true
    log_success "Cleanup completed"
}

# Set trap for cleanup on script exit
trap cleanup EXIT

# Function to check Docker daemon
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker daemon is not running or accessible"
        exit 1
    fi
    log_success "Docker daemon is running"
}

# Function to create base Ubuntu build image
create_base_image() {
    log_info "Creating base Ubuntu build image: ${IMAGE_NAME}"

    cat > "${PROJECT_ROOT}/Dockerfile.build" << 'EOF'
FROM ubuntu:20.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC
ENV PATH="$PATH:/root/.cargo/bin"

# Install comprehensive build dependencies
RUN apt-get update && \
    apt-get install --no-install-recommends -y \
    # Basic build tools
    build-essential \
    git \
    curl \
    ca-certificates \
    pkg-config \
    cmake \
    make \
    file \
    # OpenSSL and crypto libraries
    libssl-dev \
    libssl1.1 \
    # Solana dependencies
    libudev-dev \
    zlib1g-dev \
    clang \
    llvm \
    libprotobuf-dev \
    protobuf-compiler \
    libclang-dev \
    # Additional system libraries
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- --no-modify-path --profile minimal --default-toolchain stable -y && \
    rustup component add rustfmt clippy

WORKDIR /app
EOF

    # Build the base image
    if docker build -f "${PROJECT_ROOT}/Dockerfile.build" -t "${IMAGE_NAME}" "${PROJECT_ROOT}"; then
        log_success "Base Ubuntu image created successfully"
        rm -f "${PROJECT_ROOT}/Dockerfile.build"
    else
        log_error "Failed to create base Ubuntu image"
        exit 1
    fi
}

# Function to build project in container
build_in_container() {
    log_info "Starting build in Ubuntu container: ${CONTAINER_NAME}"

    # Create and start build container
    docker run -d \
        --name "${CONTAINER_NAME}" \
        -v "${PROJECT_ROOT}:/workspace" \
        -w /workspace \
        "${IMAGE_NAME}" \
        sleep infinity

    # Wait for container to be ready
    log_info "Waiting for container to be ready..."
    sleep 3

    # Copy project files and build
    log_info "Building ORE project in Ubuntu container..."

    if docker exec "${CONTAINER_NAME}" bash -c "
        set -e
        echo '=== Ubuntu Build Environment ==='
        uname -a
        rustc --version
        cargo --version

        echo '=== Starting Build ==='
        PROTOC=/usr/bin/protoc \
        PKG_CONFIG_ALLOW_CROSS=1 \
        cargo build --release \
        --package ore-ingest \
        --features api

        echo '=== Build Results ==='
        ls -la target/release/ore-ingest
        file target/release/ore-ingest
        du -h target/release/ore-ingest

        echo '=== Build Successful ==='
    "; then
        log_success "Build completed successfully in Ubuntu container"

        # Copy binary out of container
        log_info "Extracting built binary..."
        docker cp "${CONTAINER_NAME}:/workspace/target/release/ore-ingest" "${PROJECT_ROOT}/ore-ingest-ubuntu"
        log_success "Binary extracted: ${PROJECT_ROOT}/ore-ingest-ubuntu"

        # Show binary info
        ls -la "${PROJECT_ROOT}/ore-ingest-ubuntu"
        file "${PROJECT_ROOT}/ore-ingest-ubuntu"
        du -h "${PROJECT_ROOT}/ore-ingest-ubuntu"

    else
        log_error "Build failed in Ubuntu container"
        exit 1
    fi
}

# Function to test built binary
test_binary() {
    if [ -f "${PROJECT_ROOT}/ore-ingest-ubuntu" ]; then
        log_info "Testing built binary..."

        # Quick test run
        timeout 5s "${PROJECT_ROOT}/ore-ingest-ubuntu" --help >/dev/null 2>&1 || true
        log_success "Binary test completed"
    else
        log_warning "No binary found to test"
    fi
}

# Main execution
main() {
    log_info "Starting ORE Ubuntu Container Build"
    log_info "Project Root: ${PROJECT_ROOT}"
    log_info "Container Name: ${CONTAINER_NAME}"

    check_docker
    create_base_image
    build_in_container
    test_binary

    log_success "Build process completed successfully!"
    log_info "Built binary available at: ${PROJECT_ROOT}/ore-ingest-ubuntu"
}

# Run main function
main "$@"

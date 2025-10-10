# Multi-stage Dockerfile for langgraph-rs

# Build stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/langgraph-rs

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    python3 \
    python3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY langgraph-inspector/Cargo.toml langgraph-inspector/
COPY langgraph-generator/Cargo.toml langgraph-generator/
COPY langgraph-runtime/Cargo.toml langgraph-runtime/
COPY langgraph-cli/Cargo.toml langgraph-cli/

# Create dummy source files to cache dependencies
RUN mkdir -p langgraph-inspector/src langgraph-generator/src langgraph-runtime/src langgraph-cli/src && \
    echo "fn main() {}" > langgraph-cli/src/main.rs && \
    echo "pub fn dummy() {}" > langgraph-inspector/src/lib.rs && \
    echo "pub fn dummy() {}" > langgraph-generator/src/lib.rs && \
    echo "pub fn dummy() {}" > langgraph-runtime/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf langgraph-*/src

# Copy real source code
COPY langgraph-inspector langgraph-inspector
COPY langgraph-generator langgraph-generator
COPY langgraph-runtime langgraph-runtime
COPY langgraph-cli langgraph-cli

# Build the application
RUN cargo build --release --bin langgraph-rs

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    python3 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/langgraph-rs/target/release/langgraph-rs /usr/local/bin/langgraph-rs

# Create non-root user
RUN useradd -m -u 1000 langgraph && \
    mkdir -p /app && \
    chown langgraph:langgraph /app

USER langgraph
WORKDIR /app

ENTRYPOINT ["langgraph-rs"]
CMD ["--help"]

# Build stage
FROM --platform=linux/amd64 debian:12-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libgit2-dev \
    zlib1g-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# CA certificates stage
FROM --platform=linux/amd64 alpine:3.20.3 AS certs
RUN apk --no-cache add ca-certificates

# Runtime stage - minimal distroless image with specific version
FROM --platform=linux/amd64 gcr.io/distroless/cc-debian12:latest

# Copy CA certificates for HTTPS requests
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the missing zlib library from builder
COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1* /usr/lib/x86_64-linux-gnu/

# Copy the binary
COPY --from=builder /app/target/release/conventional-release-action /conventional-release-action

# Set the working directory
WORKDIR /github/workspace

# Set the entrypoint
ENTRYPOINT ["/conventional-release-action"]

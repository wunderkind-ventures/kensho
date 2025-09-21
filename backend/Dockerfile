# Build stage - using Alpine for the build
FROM --platform=$BUILDPLATFORM rust:1.88-alpine AS builder

# Install build dependencies for static linking
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create cargo config inline to avoid copy issues
RUN mkdir -p .cargo && \
    echo '[net]' >> .cargo/config.toml && \
    echo 'retry = 10' >> .cargo/config.toml && \
    echo 'timeout = 60' >> .cargo/config.toml && \
    echo '[registries.crates-io]' >> .cargo/config.toml && \
    echo 'protocol = "sparse"' >> .cargo/config.toml

# Create dummy main.rs to build dependencies
RUN mkdir src && \
    mkdir src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/backend-server.rs

# Build dependencies only (this layer will be cached)
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_TIMEOUT=60
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# For ARM64 on Alpine, we need specific flags
RUN if [ "$(uname -m)" = "aarch64" ]; then \
        export RUSTFLAGS="-C target-feature=+crt-static"; \
    else \
        export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-static"; \
    fi && \
    cargo build --release --bin backend-server && \
    rm -rf src

# Now copy actual source code
COPY src ./src

# Build the actual application (reuses cached dependencies)
RUN if [ "$(uname -m)" = "aarch64" ]; then \
        export RUSTFLAGS="-C target-feature=+crt-static"; \
    else \
        export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-static"; \
    fi && \
    touch src/main.rs && \
    cargo build --release --bin backend-server

# Runtime stage - using multi-arch distroless
FROM --platform=$TARGETPLATFORM gcr.io/distroless/static-debian12:nonroot

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/backend-server /app/backend-server

# Use non-root user (included in distroless)
USER nonroot:nonroot

# Expose port
EXPOSE 3000

# Run the binary
ENTRYPOINT ["/app/backend-server"]
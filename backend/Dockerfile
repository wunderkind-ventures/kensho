# Build stage - using Alpine for the build but avoid problematic static linking
FROM rust:1.88-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

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

# Build dependencies without problematic static linking flags
RUN cargo build --release --bin backend-server && rm -rf src

# Now copy actual source code
COPY src ./src

# Build the actual application
RUN touch src/main.rs && \
    cargo build --release --bin backend-server

# Runtime stage - using distroless base for dynamically linked binaries
FROM gcr.io/distroless/base-debian12:nonroot

WORKDIR /app

# Copy the statically linked binary from builder
COPY --from=builder /app/target/release/backend-server /app/backend-server

# Use non-root user (built into distroless)
USER nonroot:nonroot

# Expose port
EXPOSE 3000

# Run the binary
ENTRYPOINT ["/app/backend-server"]

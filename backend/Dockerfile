# Build stage (glibc so proc-macros work)
FROM rust:1.89-bookworm AS builder

# Skip apt-get for now - rust image should have basic build tools
RUN rustc --version

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/
RUN mkdir -p .cargo && \
    printf "[net]\nretry = 10\ntimeout = 60\n[registries.crates-io]\nprotocol = \"sparse\"\n" > .cargo/config.toml

# Build dependency layer
RUN mkdir -p backend/src/bin && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "fn main() {}" > backend/src/bin/backend-server.rs && \
    cargo build --release --bin backend-server && \
    rm -rf backend/src

# Build actual app
COPY backend/src ./backend/src
RUN touch backend/src/main.rs && cargo build --release --bin backend-server

# Runtime (distroless glibc)
FROM gcr.io/distroless/base-debian12:nonroot
WORKDIR /app
COPY --from=builder /app/target/release/backend-server /app/backend-server
EXPOSE 3000
USER nonroot:nonroot
ENTRYPOINT ["/app/backend-server"]

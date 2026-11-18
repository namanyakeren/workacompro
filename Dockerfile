# Multi-stage build for WORKA SOLUTION
# Stage 1: Builder
FROM rust:1-slim-bookworm AS builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src
COPY templates ./templates
COPY config ./config
COPY static ./static
COPY migrations ./migrations

# Build application
# Touch the entry point so Cargo always rebuilds the real binary instead of
# reusing the dummy one compiled above (copied sources may have older mtimes).
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    libsqlite3-0 \
    sqlite3 \
    ca-certificates \
    tini \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -u 1001 -U worka

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/worka-solution /app/worka-solution

# Copy static assets and templates
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static
COPY --from=builder /app/config ./config
COPY --from=builder /app/migrations ./migrations

# Create directories for data and uploads
RUN mkdir -p /app/data /app/static/uploads && chown -R worka:worka /app

# Switch to non-root user
USER worka

# Expose port (will be overridden by env)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 -O /dev/null http://127.0.0.1:8080/ || exit 1

# Use tini for proper signal handling
ENTRYPOINT ["tini", "--"]

# Run application
CMD ["/app/worka-solution"]
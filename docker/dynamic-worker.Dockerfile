# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the application
RUN cargo build --release -p dynamic-worker

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies including container runtime tools
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    iptables \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/dynamic-worker /usr/local/bin/dynamic-worker

# Create non-root user
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 9090

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

CMD ["dynamic-worker"]

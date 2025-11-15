# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libyara-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the application
RUN cargo build --release -p static-worker

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libyara9 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/static-worker /usr/local/bin/static-worker

# Create directories for YARA rules
RUN mkdir -p /etc/yara-rules

# Create non-root user
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app /etc/yara-rules

USER appuser

EXPOSE 9090

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

CMD ["static-worker"]

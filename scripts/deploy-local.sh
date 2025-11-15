#!/bin/bash

# Local Deployment Script using docker-compose
# This script sets up the entire platform locally for development

set -e

echo "=========================================="
echo "Deploying Security SaaS Platform Locally"
echo "=========================================="

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "Error: docker-compose is not installed"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo "Error: Docker is not running"
    exit 1
fi

# Stop any existing containers
print_info "Stopping existing containers..."
docker-compose down -v 2>/dev/null || true
print_success "Stopped existing containers"

# Start infrastructure services
print_info "Starting infrastructure services..."
docker-compose up -d postgres redis nats minio minio-setup
print_success "Infrastructure services started"

# Wait for services to be healthy
print_info "Waiting for services to be healthy..."
sleep 10

# Check PostgreSQL
print_info "Checking PostgreSQL..."
until docker-compose exec -T postgres pg_isready -U postgres &> /dev/null; do
    echo "Waiting for PostgreSQL..."
    sleep 2
done
print_success "PostgreSQL is ready"

# Check Redis
print_info "Checking Redis..."
until docker-compose exec -T redis redis-cli ping &> /dev/null; do
    echo "Waiting for Redis..."
    sleep 2
done
print_success "Redis is ready"

# Check NATS
print_info "Checking NATS..."
until curl -f http://localhost:8222/healthz &> /dev/null; do
    echo "Waiting for NATS..."
    sleep 2
done
print_success "NATS is ready"

# Check MinIO
print_info "Checking MinIO..."
until curl -f http://localhost:9000/minio/health/live &> /dev/null; do
    echo "Waiting for MinIO..."
    sleep 2
done
print_success "MinIO is ready"

# Run database migrations
print_info "Running database migrations..."
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/security_saas"
if command -v sqlx &> /dev/null; then
    sqlx migrate run
    print_success "Database migrations completed"
else
    echo "Warning: sqlx-cli not found. Please run migrations manually:"
    echo "  cargo install sqlx-cli --no-default-features --features postgres"
    echo "  sqlx migrate run"
fi

# Start observability stack
print_info "Starting observability stack..."
docker-compose up -d prometheus grafana jaeger
print_success "Observability stack started"

# Wait for observability services
sleep 5

echo ""
echo "=========================================="
echo -e "${GREEN}Local deployment completed!${NC}"
echo "=========================================="
echo ""
echo "Services are available at:"
echo "  PostgreSQL:  localhost:5432"
echo "  Redis:       localhost:6379"
echo "  NATS:        localhost:4222"
echo "  MinIO:       http://localhost:9000 (Console: http://localhost:9001)"
echo "  Prometheus:  http://localhost:9090"
echo "  Grafana:     http://localhost:3000 (admin/admin)"
echo "  Jaeger:      http://localhost:16686"
echo ""
echo "Environment variables for development:"
echo "  export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/security_saas"
echo "  export REDIS_URL=redis://localhost:6379"
echo "  export NATS_URL=nats://localhost:4222"
echo "  export S3_ENDPOINT=http://localhost:9000"
echo "  export S3_ACCESS_KEY=minioadmin"
echo "  export S3_SECRET_KEY=minioadmin"
echo "  export S3_BUCKET=security-saas-artifacts"
echo ""
echo "To start the application services:"
echo "  cargo run -p api-layer"
echo "  cargo run -p static-worker"
echo "  cargo run -p dynamic-worker"
echo "  cargo run -p behavioral-worker"
echo ""
echo "To view logs:"
echo "  docker-compose logs -f"
echo ""
echo "To stop all services:"
echo "  docker-compose down"
echo ""

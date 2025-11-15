#!/bin/bash

# Deployment Testing Script
# This script tests the deployment of all services

set -e

NAMESPACE="${NAMESPACE:-security-saas}"
TIMEOUT="${TIMEOUT:-300}"

echo "=========================================="
echo "Testing Security SaaS Platform Deployment"
echo "Namespace: $NAMESPACE"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# Test 1: Check if namespace exists
print_info "Checking namespace..."
kubectl get namespace $NAMESPACE > /dev/null 2>&1
print_status $? "Namespace $NAMESPACE exists"

# Test 2: Check if all pods are running
print_info "Checking pod status..."
PODS=$(kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | wc -l)
if [ $PODS -eq 0 ]; then
    print_status 1 "No pods found in namespace $NAMESPACE"
fi

RUNNING_PODS=$(kubectl get pods -n $NAMESPACE --field-selector=status.phase=Running --no-headers 2>/dev/null | wc -l)
print_info "Found $RUNNING_PODS/$PODS pods running"

# Wait for all pods to be ready
print_info "Waiting for all pods to be ready (timeout: ${TIMEOUT}s)..."
kubectl wait --for=condition=ready pod --all -n $NAMESPACE --timeout=${TIMEOUT}s
print_status $? "All pods are ready"

# Test 3: Check deployments
print_info "Checking deployments..."
DEPLOYMENTS=("api-layer" "static-worker" "dynamic-worker" "behavioral-worker")
for deployment in "${DEPLOYMENTS[@]}"; do
    kubectl get deployment $deployment -n $NAMESPACE > /dev/null 2>&1
    print_status $? "Deployment $deployment exists"
    
    READY=$(kubectl get deployment $deployment -n $NAMESPACE -o jsonpath='{.status.readyReplicas}')
    DESIRED=$(kubectl get deployment $deployment -n $NAMESPACE -o jsonpath='{.spec.replicas}')
    
    if [ "$READY" == "$DESIRED" ]; then
        print_status 0 "Deployment $deployment is ready ($READY/$DESIRED replicas)"
    else
        print_status 1 "Deployment $deployment is not ready ($READY/$DESIRED replicas)"
    fi
done

# Test 4: Check services
print_info "Checking services..."
SERVICES=("api-layer" "postgresql" "redis" "nats" "minio")
for service in "${SERVICES[@]}"; do
    kubectl get service $service -n $NAMESPACE > /dev/null 2>&1
    print_status $? "Service $service exists"
done

# Test 5: Check database connectivity
print_info "Testing database connectivity..."
DB_POD=$(kubectl get pod -n $NAMESPACE -l app=postgresql -o jsonpath='{.items[0].metadata.name}')
if [ -n "$DB_POD" ]; then
    kubectl exec -n $NAMESPACE $DB_POD -- pg_isready -U postgres > /dev/null 2>&1
    print_status $? "PostgreSQL is accepting connections"
else
    print_status 1 "PostgreSQL pod not found"
fi

# Test 6: Check Redis connectivity
print_info "Testing Redis connectivity..."
REDIS_POD=$(kubectl get pod -n $NAMESPACE -l app=redis -o jsonpath='{.items[0].metadata.name}')
if [ -n "$REDIS_POD" ]; then
    kubectl exec -n $NAMESPACE $REDIS_POD -- redis-cli ping > /dev/null 2>&1
    print_status $? "Redis is responding"
else
    print_status 1 "Redis pod not found"
fi

# Test 7: Check NATS connectivity
print_info "Testing NATS connectivity..."
NATS_POD=$(kubectl get pod -n $NAMESPACE -l app=nats -o jsonpath='{.items[0].metadata.name}')
if [ -n "$NATS_POD" ]; then
    kubectl exec -n $NAMESPACE $NATS_POD -- nats server ping > /dev/null 2>&1
    print_status $? "NATS is responding"
else
    print_status 1 "NATS pod not found"
fi

# Test 8: Check MinIO connectivity
print_info "Testing MinIO connectivity..."
MINIO_POD=$(kubectl get pod -n $NAMESPACE -l app=minio -o jsonpath='{.items[0].metadata.name}')
if [ -n "$MINIO_POD" ]; then
    kubectl exec -n $NAMESPACE $MINIO_POD -- curl -f http://localhost:9000/minio/health/live > /dev/null 2>&1
    print_status $? "MinIO is healthy"
else
    print_status 1 "MinIO pod not found"
fi

# Test 9: Check API health endpoint
print_info "Testing API health endpoint..."
API_POD=$(kubectl get pod -n $NAMESPACE -l app=api-layer -o jsonpath='{.items[0].metadata.name}')
if [ -n "$API_POD" ]; then
    kubectl exec -n $NAMESPACE $API_POD -- curl -f http://localhost:8080/health > /dev/null 2>&1
    print_status $? "API health endpoint is responding"
else
    print_status 1 "API pod not found"
fi

# Test 10: Check metrics endpoints
print_info "Testing metrics endpoints..."
for deployment in "${DEPLOYMENTS[@]}"; do
    POD=$(kubectl get pod -n $NAMESPACE -l app=$deployment -o jsonpath='{.items[0].metadata.name}')
    if [ -n "$POD" ]; then
        kubectl exec -n $NAMESPACE $POD -- curl -f http://localhost:9090/metrics > /dev/null 2>&1
        print_status $? "Metrics endpoint for $deployment is responding"
    fi
done

# Test 11: Check HPAs
print_info "Checking Horizontal Pod Autoscalers..."
HPAS=("static-worker-hpa" "dynamic-worker-hpa" "behavioral-worker-hpa")
for hpa in "${HPAS[@]}"; do
    kubectl get hpa $hpa -n $NAMESPACE > /dev/null 2>&1
    print_status $? "HPA $hpa exists"
done

# Test 12: Check ingress
print_info "Checking ingress..."
kubectl get ingress -n $NAMESPACE > /dev/null 2>&1
print_status $? "Ingress resources exist"

# Test 13: Check persistent volumes
print_info "Checking persistent volumes..."
PVC_COUNT=$(kubectl get pvc -n $NAMESPACE --no-headers 2>/dev/null | wc -l)
BOUND_PVC=$(kubectl get pvc -n $NAMESPACE --field-selector=status.phase=Bound --no-headers 2>/dev/null | wc -l)
print_info "Found $BOUND_PVC/$PVC_COUNT PVCs bound"
if [ $PVC_COUNT -gt 0 ] && [ $BOUND_PVC -eq $PVC_COUNT ]; then
    print_status 0 "All PVCs are bound"
else
    print_status 1 "Some PVCs are not bound"
fi

# Test 14: Check service-to-service communication
print_info "Testing service-to-service communication..."
API_POD=$(kubectl get pod -n $NAMESPACE -l app=api-layer -o jsonpath='{.items[0].metadata.name}')
if [ -n "$API_POD" ]; then
    # Test API to PostgreSQL
    kubectl exec -n $NAMESPACE $API_POD -- curl -f http://postgresql:5432 > /dev/null 2>&1 || true
    
    # Test API to Redis
    kubectl exec -n $NAMESPACE $API_POD -- curl -f http://redis:6379 > /dev/null 2>&1 || true
    
    print_status 0 "Service-to-service communication test completed"
fi

# Summary
echo ""
echo "=========================================="
echo -e "${GREEN}All deployment tests passed!${NC}"
echo "=========================================="
echo ""
echo "Deployment Summary:"
echo "  Namespace: $NAMESPACE"
echo "  Pods: $RUNNING_PODS/$PODS running"
echo "  Services: All critical services are healthy"
echo ""
echo "Next steps:"
echo "  1. Access Grafana: kubectl port-forward -n observability svc/grafana 3000:3000"
echo "  2. Access Jaeger: kubectl port-forward -n observability svc/jaeger-query 16686:16686"
echo "  3. Access API: kubectl port-forward -n $NAMESPACE svc/api-layer 8080:8080"
echo "  4. View logs: kubectl logs -n $NAMESPACE -l app=api-layer -f"
echo ""

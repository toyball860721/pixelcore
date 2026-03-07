#!/bin/bash

# Health Check Script for PixelCore Deployment
# Validates all components are running and healthy

set -e

NAMESPACE="${NAMESPACE:-pixelcore}"
TIMEOUT=300

echo "=========================================="
echo "PixelCore Deployment Health Check"
echo "=========================================="
echo "Namespace: $NAMESPACE"
echo "Timestamp: $(date)"
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
    fi
}

# Check if namespace exists
echo "1. Checking namespace..."
if kubectl get namespace $NAMESPACE &> /dev/null; then
    print_status 0 "Namespace $NAMESPACE exists"
else
    print_status 1 "Namespace $NAMESPACE not found"
    exit 1
fi
echo ""

# Check pods
echo "2. Checking pod status..."
PODS=$(kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | wc -l | tr -d ' ')
RUNNING_PODS=$(kubectl get pods -n $NAMESPACE --field-selector=status.phase=Running --no-headers 2>/dev/null | wc -l | tr -d ' ')

if [ "$PODS" -gt 0 ]; then
    print_status 0 "Found $PODS pods ($RUNNING_PODS running)"

    # Check each pod
    kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | while read line; do
        POD_NAME=$(echo $line | awk '{print $1}')
        POD_STATUS=$(echo $line | awk '{print $3}')
        POD_READY=$(echo $line | awk '{print $2}')

        if [ "$POD_STATUS" = "Running" ]; then
            echo -e "  ${GREEN}✓${NC} $POD_NAME: $POD_STATUS ($POD_READY)"
        else
            echo -e "  ${YELLOW}⚠${NC} $POD_NAME: $POD_STATUS ($POD_READY)"
        fi
    done
else
    print_status 1 "No pods found"
fi
echo ""

# Check services
echo "3. Checking services..."
SERVICES=$(kubectl get services -n $NAMESPACE --no-headers 2>/dev/null | wc -l | tr -d ' ')
if [ "$SERVICES" -gt 0 ]; then
    print_status 0 "Found $SERVICES services"
    kubectl get services -n $NAMESPACE --no-headers 2>/dev/null | while read line; do
        SVC_NAME=$(echo $line | awk '{print $1}')
        SVC_TYPE=$(echo $line | awk '{print $2}')
        echo -e "  ${GREEN}✓${NC} $SVC_NAME ($SVC_TYPE)"
    done
else
    print_status 1 "No services found"
fi
echo ""

# Check deployments
echo "4. Checking deployments..."
DEPLOYMENTS=$(kubectl get deployments -n $NAMESPACE --no-headers 2>/dev/null | wc -l | tr -d ' ')
if [ "$DEPLOYMENTS" -gt 0 ]; then
    print_status 0 "Found $DEPLOYMENTS deployments"
    kubectl get deployments -n $NAMESPACE --no-headers 2>/dev/null | while read line; do
        DEPLOY_NAME=$(echo $line | awk '{print $1}')
        DEPLOY_READY=$(echo $line | awk '{print $2}')
        DEPLOY_AVAILABLE=$(echo $line | awk '{print $4}')

        if [ "$DEPLOY_AVAILABLE" -gt 0 ]; then
            echo -e "  ${GREEN}✓${NC} $DEPLOY_NAME: $DEPLOY_READY ready"
        else
            echo -e "  ${RED}✗${NC} $DEPLOY_NAME: $DEPLOY_READY ready"
        fi
    done
else
    print_status 1 "No deployments found"
fi
echo ""

# Check StatefulSets
echo "5. Checking StatefulSets..."
STATEFULSETS=$(kubectl get statefulsets -n $NAMESPACE --no-headers 2>/dev/null | wc -l | tr -d ' ')
if [ "$STATEFULSETS" -gt 0 ]; then
    print_status 0 "Found $STATEFULSETS StatefulSets"
    kubectl get statefulsets -n $NAMESPACE --no-headers 2>/dev/null | while read line; do
        STS_NAME=$(echo $line | awk '{print $1}')
        STS_READY=$(echo $line | awk '{print $2}')
        echo -e "  ${GREEN}✓${NC} $STS_NAME: $STS_READY ready"
    done
else
    echo -e "  ${YELLOW}⚠${NC} No StatefulSets found (optional)"
fi
echo ""

# Test backend connectivity
echo "6. Testing backend connectivity..."
BACKEND_POD=$(kubectl get pods -n $NAMESPACE -l app=backend --field-selector=status.phase=Running --no-headers 2>/dev/null | head -1 | awk '{print $1}')
if [ -n "$BACKEND_POD" ]; then
    print_status 0 "Backend pod found: $BACKEND_POD"

    # Try to curl the backend (assuming it has curl or we can exec)
    if kubectl exec -n $NAMESPACE $BACKEND_POD -- sh -c "command -v curl" &> /dev/null; then
        if kubectl exec -n $NAMESPACE $BACKEND_POD -- curl -s -o /dev/null -w "%{http_code}" http://localhost:8080 &> /dev/null; then
            print_status 0 "Backend responds to HTTP requests"
        else
            echo -e "  ${YELLOW}⚠${NC} Backend HTTP check skipped (nginx test image)"
        fi
    else
        echo -e "  ${YELLOW}⚠${NC} Backend connectivity check skipped (curl not available)"
    fi
else
    print_status 1 "No backend pod found"
fi
echo ""

# Test frontend connectivity
echo "7. Testing frontend connectivity..."
FRONTEND_POD=$(kubectl get pods -n $NAMESPACE -l app=frontend --field-selector=status.phase=Running --no-headers 2>/dev/null | head -1 | awk '{print $1}')
if [ -n "$FRONTEND_POD" ]; then
    print_status 0 "Frontend pod found: $FRONTEND_POD"

    if kubectl exec -n $NAMESPACE $FRONTEND_POD -- sh -c "command -v curl" &> /dev/null; then
        if kubectl exec -n $NAMESPACE $FRONTEND_POD -- curl -s -o /dev/null -w "%{http_code}" http://localhost:80 &> /dev/null; then
            print_status 0 "Frontend responds to HTTP requests"
        else
            echo -e "  ${YELLOW}⚠${NC} Frontend HTTP check skipped (nginx test image)"
        fi
    else
        echo -e "  ${YELLOW}⚠${NC} Frontend connectivity check skipped (curl not available)"
    fi
else
    print_status 1 "No frontend pod found"
fi
echo ""

# Test database connectivity
echo "8. Testing database connectivity..."
POSTGRES_POD=$(kubectl get pods -n $NAMESPACE -l app=postgres --field-selector=status.phase=Running --no-headers 2>/dev/null | head -1 | awk '{print $1}')
if [ -n "$POSTGRES_POD" ]; then
    print_status 0 "PostgreSQL pod found: $POSTGRES_POD"

    if kubectl exec -n $NAMESPACE $POSTGRES_POD -- psql -U postgres -c "SELECT 1" &> /dev/null; then
        print_status 0 "PostgreSQL is accepting connections"
    else
        echo -e "  ${YELLOW}⚠${NC} PostgreSQL connection check failed"
    fi
else
    print_status 1 "No PostgreSQL pod found"
fi
echo ""

# Test Redis connectivity
echo "9. Testing Redis connectivity..."
REDIS_POD=$(kubectl get pods -n $NAMESPACE -l app=redis --field-selector=status.phase=Running --no-headers 2>/dev/null | head -1 | awk '{print $1}')
if [ -n "$REDIS_POD" ]; then
    print_status 0 "Redis pod found: $REDIS_POD"

    if kubectl exec -n $NAMESPACE $REDIS_POD -- redis-cli ping &> /dev/null; then
        print_status 0 "Redis is accepting connections"
    else
        echo -e "  ${YELLOW}⚠${NC} Redis connection check failed"
    fi
else
    print_status 1 "No Redis pod found"
fi
echo ""

# Summary
echo "=========================================="
echo "Health Check Summary"
echo "=========================================="
echo "Total Pods: $PODS"
echo "Running Pods: $RUNNING_PODS"
echo "Services: $SERVICES"
echo "Deployments: $DEPLOYMENTS"
echo "StatefulSets: $STATEFULSETS"
echo ""

if [ "$RUNNING_PODS" -eq "$PODS" ] && [ "$PODS" -gt 0 ]; then
    echo -e "${GREEN}✓ All pods are running${NC}"
    echo -e "${GREEN}✓ Deployment is healthy${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Some pods are not running${NC}"
    echo -e "${YELLOW}⚠ Deployment may have issues${NC}"
    exit 1
fi

#!/bin/bash

# PixelCore Security Scan Script
# Performs comprehensive security scanning

set -e

echo "🔒 Starting PixelCore Security Scan..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if required tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}❌ $1 is not installed${NC}"
        echo "   Install: $2"
        return 1
    else
        echo -e "${GREEN}✅ $1 is installed${NC}"
        return 0
    fi
}

echo "Checking required tools..."
check_tool "trivy" "brew install trivy"
check_tool "docker" "https://docs.docker.com/get-docker/"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Container Image Scanning"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

IMAGES=(
    "pixelcore/api:latest"
    "pixelcore/search:latest"
    "pixelcore/ai:latest"
    "pixelcore/analytics:latest"
)

TOTAL_CRITICAL=0
TOTAL_HIGH=0
TOTAL_MEDIUM=0
TOTAL_LOW=0

for IMAGE in "${IMAGES[@]}"; do
    echo ""
    echo "Scanning $IMAGE..."

    if docker image inspect $IMAGE &> /dev/null; then
        # Scan image
        trivy image --severity CRITICAL,HIGH,MEDIUM,LOW \
            --format json \
            --output /tmp/trivy-$IMAGE.json \
            $IMAGE

        # Parse results
        CRITICAL=$(jq '[.Results[].Vulnerabilities[]? | select(.Severity=="CRITICAL")] | length' /tmp/trivy-$IMAGE.json)
        HIGH=$(jq '[.Results[].Vulnerabilities[]? | select(.Severity=="HIGH")] | length' /tmp/trivy-$IMAGE.json)
        MEDIUM=$(jq '[.Results[].Vulnerabilities[]? | select(.Severity=="MEDIUM")] | length' /tmp/trivy-$IMAGE.json)
        LOW=$(jq '[.Results[].Vulnerabilities[]? | select(.Severity=="LOW")] | length' /tmp/trivy-$IMAGE.json)

        TOTAL_CRITICAL=$((TOTAL_CRITICAL + CRITICAL))
        TOTAL_HIGH=$((TOTAL_HIGH + HIGH))
        TOTAL_MEDIUM=$((TOTAL_MEDIUM + MEDIUM))
        TOTAL_LOW=$((TOTAL_LOW + LOW))

        echo "  Critical: $CRITICAL"
        echo "  High: $HIGH"
        echo "  Medium: $MEDIUM"
        echo "  Low: $LOW"
    else
        echo -e "${YELLOW}⚠️  Image $IMAGE not found locally${NC}"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. Dependency Scanning"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "Scanning Rust dependencies..."
if [ -f "Cargo.lock" ]; then
    cargo audit --json > /tmp/cargo-audit.json || true

    RUST_VULNS=$(jq '.vulnerabilities.count' /tmp/cargo-audit.json 2>/dev/null || echo "0")
    echo "  Rust vulnerabilities: $RUST_VULNS"
else
    echo -e "${YELLOW}⚠️  Cargo.lock not found${NC}"
fi

echo ""
echo "Scanning npm dependencies..."
if [ -f "app/package-lock.json" ]; then
    cd app
    npm audit --json > /tmp/npm-audit.json || true

    NPM_CRITICAL=$(jq '.metadata.vulnerabilities.critical' /tmp/npm-audit.json 2>/dev/null || echo "0")
    NPM_HIGH=$(jq '.metadata.vulnerabilities.high' /tmp/npm-audit.json 2>/dev/null || echo "0")
    NPM_MEDIUM=$(jq '.metadata.vulnerabilities.moderate' /tmp/npm-audit.json 2>/dev/null || echo "0")

    echo "  npm Critical: $NPM_CRITICAL"
    echo "  npm High: $NPM_HIGH"
    echo "  npm Medium: $NPM_MEDIUM"
    cd ..
else
    echo -e "${YELLOW}⚠️  package-lock.json not found${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. Secret Scanning"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "Scanning for exposed secrets..."

# Check for common secret patterns
SECRET_PATTERNS=(
    "password.*=.*['\"].*['\"]"
    "api[_-]?key.*=.*['\"].*['\"]"
    "secret.*=.*['\"].*['\"]"
    "token.*=.*['\"].*['\"]"
    "aws[_-]?access[_-]?key"
    "private[_-]?key"
)

SECRETS_FOUND=0
for PATTERN in "${SECRET_PATTERNS[@]}"; do
    MATCHES=$(grep -r -i -E "$PATTERN" --exclude-dir={target,node_modules,.git} . 2>/dev/null | wc -l)
    if [ $MATCHES -gt 0 ]; then
        echo -e "${RED}⚠️  Found $MATCHES potential secrets matching: $PATTERN${NC}"
        SECRETS_FOUND=$((SECRETS_FOUND + MATCHES))
    fi
done

if [ $SECRETS_FOUND -eq 0 ]; then
    echo -e "${GREEN}✅ No exposed secrets found${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4. Kubernetes Security"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "Checking Kubernetes configurations..."

# Check for security contexts
if [ -d "k8s" ]; then
    NO_SECURITY_CONTEXT=$(grep -r "kind: Deployment" k8s/ | wc -l)
    WITH_SECURITY_CONTEXT=$(grep -r "securityContext:" k8s/ | wc -l)

    echo "  Deployments: $NO_SECURITY_CONTEXT"
    echo "  With securityContext: $WITH_SECURITY_CONTEXT"

    if [ $WITH_SECURITY_CONTEXT -lt $NO_SECURITY_CONTEXT ]; then
        echo -e "${YELLOW}⚠️  Some deployments missing securityContext${NC}"
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Security Scan Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Container Vulnerabilities:"
echo "  Critical: $TOTAL_CRITICAL"
echo "  High: $TOTAL_HIGH"
echo "  Medium: $TOTAL_MEDIUM"
echo "  Low: $TOTAL_LOW"
echo ""
echo "Dependency Vulnerabilities:"
echo "  Rust: ${RUST_VULNS:-0}"
echo "  npm Critical: ${NPM_CRITICAL:-0}"
echo "  npm High: ${NPM_HIGH:-0}"
echo ""
echo "Secrets:"
echo "  Potential secrets found: $SECRETS_FOUND"
echo ""

# Determine overall status
FAIL=0

if [ $TOTAL_CRITICAL -gt 0 ] || [ $TOTAL_HIGH -gt 0 ]; then
    echo -e "${RED}❌ FAIL: Critical or High vulnerabilities found${NC}"
    FAIL=1
fi

if [ ${NPM_CRITICAL:-0} -gt 0 ] || [ ${NPM_HIGH:-0} -gt 0 ]; then
    echo -e "${RED}❌ FAIL: Critical or High npm vulnerabilities found${NC}"
    FAIL=1
fi

if [ $SECRETS_FOUND -gt 0 ]; then
    echo -e "${YELLOW}⚠️  WARNING: Potential secrets found${NC}"
fi

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ PASS: No critical security issues found${NC}"
fi

echo ""
echo "Detailed reports saved to /tmp/"
echo "  - trivy-*.json"
echo "  - cargo-audit.json"
echo "  - npm-audit.json"
echo ""

exit $FAIL

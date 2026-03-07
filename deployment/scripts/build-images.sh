#!/bin/bash

# Build Docker Images Script
# Builds backend and frontend Docker images for PixelCore

set -e

# Configuration
REGISTRY="${DOCKER_REGISTRY:-ghcr.io/your-org/pixelcore}"
VERSION="${VERSION:-latest}"
BACKEND_IMAGE="${REGISTRY}/backend:${VERSION}"
FRONTEND_IMAGE="${REGISTRY}/frontend:${VERSION}"

echo "=========================================="
echo "Building PixelCore Docker Images"
echo "=========================================="
echo "Registry: $REGISTRY"
echo "Version: $VERSION"
echo "Backend Image: $BACKEND_IMAGE"
echo "Frontend Image: $FRONTEND_IMAGE"
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

# Check if Docker is running
echo "1. Checking Docker..."
if docker info &> /dev/null; then
    print_status 0 "Docker is running"
else
    print_status 1 "Docker is not running. Please start Docker first."
fi
echo ""

# Build Backend Image
echo "2. Building Backend Image..."
echo "   Building Rust backend (this may take several minutes)..."

cd /Users/toyball/Desktop/ClaudeUse/pixelcore

if docker build -t $BACKEND_IMAGE -f Dockerfile . ; then
    print_status 0 "Backend image built successfully"

    # Also tag as 'backend:latest' for local use
    docker tag $BACKEND_IMAGE pixelcore/backend:latest
    print_status 0 "Tagged as pixelcore/backend:latest"
else
    print_status 1 "Backend image build failed"
fi
echo ""

# Build Frontend Image
echo "3. Building Frontend Image..."
echo "   Building React frontend..."

cd /Users/toyball/Desktop/ClaudeUse/pixelcore/app

if docker build -t $FRONTEND_IMAGE -f Dockerfile . ; then
    print_status 0 "Frontend image built successfully"

    # Also tag as 'frontend:latest' for local use
    docker tag $FRONTEND_IMAGE pixelcore/frontend:latest
    print_status 0 "Tagged as pixelcore/frontend:latest"
else
    print_status 1 "Frontend image build failed"
fi
echo ""

# List built images
echo "4. Built Images:"
docker images | grep -E "pixelcore|REPOSITORY" || true
echo ""

# Image sizes
echo "5. Image Sizes:"
echo "   Backend:  $(docker images $BACKEND_IMAGE --format '{{.Size}}')"
echo "   Frontend: $(docker images $FRONTEND_IMAGE --format '{{.Size}}')"
echo ""

# Summary
echo "=========================================="
echo "Build Summary"
echo "=========================================="
echo -e "${GREEN}✓ Backend image built: $BACKEND_IMAGE${NC}"
echo -e "${GREEN}✓ Frontend image built: $FRONTEND_IMAGE${NC}"
echo ""
echo "Local tags created:"
echo "  - pixelcore/backend:latest"
echo "  - pixelcore/frontend:latest"
echo ""

# Ask about pushing to registry
echo "=========================================="
echo "Next Steps"
echo "=========================================="
echo ""
echo "1. Test images locally:"
echo "   docker run -p 8080:8080 pixelcore/backend:latest"
echo "   docker run -p 80:80 pixelcore/frontend:latest"
echo ""
echo "2. Push to registry (if needed):"
echo "   docker push $BACKEND_IMAGE"
echo "   docker push $FRONTEND_IMAGE"
echo ""
echo "3. Deploy to Kubernetes:"
echo "   Update k8s manifests to use these images"
echo "   kubectl apply -f k8s/"
echo ""

# Optional: Push to registry
read -p "Do you want to push images to registry now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Pushing images to registry..."

    if docker push $BACKEND_IMAGE; then
        print_status 0 "Backend image pushed to registry"
    else
        print_status 1 "Failed to push backend image"
    fi

    if docker push $FRONTEND_IMAGE; then
        print_status 0 "Frontend image pushed to registry"
    else
        print_status 1 "Failed to push frontend image"
    fi

    echo ""
    echo -e "${GREEN}✓ Images pushed to registry successfully${NC}"
else
    echo ""
    echo "Skipping registry push. Images are available locally."
fi

echo ""
echo "=========================================="
echo "Build Complete!"
echo "=========================================="

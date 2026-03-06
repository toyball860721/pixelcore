# PixelCore Service Mesh

This directory contains the Istio service mesh configuration for PixelCore.

## Overview

The service mesh provides:
- Service-to-service communication management
- Traffic routing and load balancing
- Circuit breaking and fault injection
- mTLS encryption
- Distributed tracing
- Observability and monitoring

## Directory Structure

```
service-mesh/
├── README.md                 # This file
├── install.sh               # Istio installation script
├── gateway/                 # Ingress gateway configuration
├── virtual-services/        # Traffic routing rules
├── destination-rules/       # Load balancing and circuit breaking
├── policies/                # Security and rate limiting policies
└── monitoring/              # Observability configuration
```

## Quick Start

1. Install Istio:
   ```bash
   ./install.sh
   ```

2. Apply gateway configuration:
   ```bash
   kubectl apply -f gateway/
   ```

3. Apply virtual services:
   ```bash
   kubectl apply -f virtual-services/
   ```

4. Apply destination rules:
   ```bash
   kubectl apply -f destination-rules/
   ```

## Features

- **Traffic Management**: Canary deployments, blue-green deployments, A/B testing
- **Security**: mTLS, authorization policies, rate limiting
- **Observability**: Distributed tracing with Jaeger, metrics with Prometheus
- **Resilience**: Circuit breaking, retries, timeouts

## Documentation

See [SERVICE_MESH.md](../../docs/SERVICE_MESH.md) for detailed documentation.

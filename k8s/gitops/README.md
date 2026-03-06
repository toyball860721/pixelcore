# PixelCore GitOps

This directory contains GitOps configuration for PixelCore using ArgoCD.

## Overview

GitOps provides:
- Declarative infrastructure and application management
- Git as the single source of truth
- Automated synchronization and deployment
- Easy rollback and audit trail
- Multi-environment management

## Directory Structure

```
gitops/
├── README.md                    # This file
├── install-argocd.sh           # ArgoCD installation script
├── argocd/                     # ArgoCD configuration
│   ├── application.yaml        # Root application
│   └── projects.yaml           # ArgoCD projects
├── apps/                       # Application manifests
│   ├── pixelcore-api/
│   ├── pixelcore-search/
│   ├── pixelcore-ai/
│   └── pixelcore-analytics/
└── environments/               # Environment-specific configs
    ├── dev/
    ├── staging/
    └── production/
```

## Quick Start

1. Install ArgoCD:
   ```bash
   ./install-argocd.sh
   ```

2. Access ArgoCD UI:
   ```bash
   kubectl port-forward svc/argocd-server -n argocd 8080:443
   # Visit https://localhost:8080
   # Username: admin
   # Password: (get from kubectl -n argocd get secret argocd-initial-admin-secret)
   ```

3. Deploy applications:
   ```bash
   kubectl apply -f argocd/application.yaml
   ```

## Features

- **Automated Sync**: Automatically deploy changes from Git
- **Health Monitoring**: Track application health status
- **Rollback**: Easy one-click rollback to previous versions
- **Multi-Environment**: Manage dev, staging, and production
- **RBAC**: Role-based access control
- **Notifications**: Slack/Email notifications for deployments

## Documentation

See [GITOPS_GUIDE.md](../../docs/GITOPS_GUIDE.md) for detailed documentation.
